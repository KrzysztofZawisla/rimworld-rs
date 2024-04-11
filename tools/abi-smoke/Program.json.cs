using System;
using System.Collections.Generic;
using System.Collections.Immutable;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Reflection.Emit;
using System.Reflection.Metadata;
using System.Reflection.PortableExecutable;
using System.Runtime.InteropServices;
using System.Text;
using System.Text.Json;
using System.Text.RegularExpressions;
using System.Xml.Linq;
namespace RustBridge.Smoke;

/// <summary>
/// The `json` half of <see cref="Program"/>.
/// </summary>
internal static partial class Program
{

    /// <summary>A random JSON-able value, with live objects mixed in as handles.</summary>
    private static object RandomValue(Random random, int depth)
    {
        int kind = random.Next(depth >= 3 ? 5 : 7);
        return kind switch
        {
            0 => null,
            1 => random.Next(2) == 0,
            2 => (long)random.Next(-100000, 100000),
            3 => random.NextDouble() * 1000 - 500,
            4 => "text" + random.Next(1000),
            5 => new Dictionary<string, object>
            {
                ["key"] = RandomValue(random, depth + 1),
                ["nested"] = new List<object> { RandomValue(random, depth + 1), RandomValue(random, depth + 1) },
            },
            _ => new List<object> { RandomValue(random, depth + 1), random.Next(10) },
        };
    }

    /// <summary>Checks that a serialized value still carries the same structure and leaves.</summary>
    private static bool RoundTrips(JsonElement element, object value)
    {
        switch (value)
        {
            case null:
                return element.ValueKind == JsonValueKind.Null;
            case bool flag:
                return element.ValueKind == (flag ? JsonValueKind.True : JsonValueKind.False);
            case long or int or short or sbyte or byte or ushort or uint or ulong:
                return element.ValueKind == JsonValueKind.Number
                    && Math.Abs(element.GetDouble() - Convert.ToDouble(value)) < 0.0001;
            case double or float or decimal:
                return element.ValueKind == JsonValueKind.Number
                    && Math.Abs(element.GetDouble() - Convert.ToDouble(value)) < 0.0001;
            case string text:
                return element.ValueKind == JsonValueKind.String && element.GetString() == text;
            case Dictionary<string, object> dictionary:
                if (element.ValueKind != JsonValueKind.Object)
                {
                    return false;
                }

                foreach ((string key, object nested) in dictionary)
                {
                    if (!element.TryGetProperty(key, out JsonElement child) || !RoundTrips(child, nested))
                    {
                        return false;
                    }
                }

                return element.EnumerateObject().Count() == dictionary.Count;
            case List<object> list:
                if (element.ValueKind != JsonValueKind.Array)
                {
                    return false;
                }

                JsonElement[] items = element.EnumerateArray().ToArray();
                if (items.Length != list.Count)
                {
                    return false;
                }

                for (int index = 0; index < items.Length; index++)
                {
                    if (!RoundTrips(items[index], list[index]))
                    {
                        return false;
                    }
                }

                return true;
            default:
                // anything else is a live object, which must travel as a handle
                return element.ValueKind == JsonValueKind.Object && element.TryGetProperty("$ref", out _);
        }
    }
    private static JsonElement Json(string text)
    {
        using JsonDocument document = JsonDocument.Parse(text);
        return document.RootElement.Clone();
    }

    /// <summary>
    /// Tests the bridge's marshaling layer, which is compiled into this tool. It guards the wire
    /// format the game sees: structured results must be real JSON objects/arrays, while live game
    /// objects stay handles.
    /// </summary>
    private static int RunJsonChecks()
    {
        int checks = 0;
        int failures = 0;
        void Expect(bool condition, string description)
        {
            checks++;
            if (condition)
            {
                Console.WriteLine($"ok    {description}");
            }
            else
            {
                failures++;
                Console.WriteLine($"FAIL  {description}");
            }
        }

        string info = Serialize(new Dictionary<string, object>
        {
            ["defName"] = "Steel",
            ["stackCount"] = 3,
            ["craftable"] = true,
            ["nothing"] = null,
        }, new RefTable());
        using (JsonDocument document = JsonDocument.Parse(info))
        {
            JsonElement root = document.RootElement;
            Expect(root.ValueKind == JsonValueKind.Object, "a dictionary becomes a JSON object");
            Expect(root.TryGetProperty("defName", out JsonElement name) && name.GetString() == "Steel", "string values survive");
            Expect(root.TryGetProperty("stackCount", out JsonElement count) && count.GetInt32() == 3, "numbers survive");
            Expect(root.TryGetProperty("craftable", out JsonElement craftable) && craftable.GetBoolean(), "bools survive");
            Expect(root.TryGetProperty("nothing", out JsonElement nothing) && nothing.ValueKind == JsonValueKind.Null, "nulls survive");
        }

        string names = Serialize(new List<string> { "Steel", "Silver" }, new RefTable());
        using (JsonDocument document = JsonDocument.Parse(names))
        {
            Expect(document.RootElement.ValueKind == JsonValueKind.Array && document.RootElement.GetArrayLength() == 2, "a list becomes a JSON array");
        }

        string maps = Serialize(new Dictionary<string, object>
        {
            ["hasGame"] = true,
            ["maps"] = new List<object>
            {
                new Dictionary<string, object> { ["id"] = 7, ["biome"] = "TemperateForest" },
            },
        }, new RefTable());
        using (JsonDocument document = JsonDocument.Parse(maps))
        {
            JsonElement list = document.RootElement.GetProperty("maps");
            Expect(
                list.ValueKind == JsonValueKind.Array
                && list[0].GetProperty("id").GetInt32() == 7
                && list[0].GetProperty("biome").GetString() == "TemperateForest",
                "nested dictionaries inside lists stay structured");
        }

        // An inline JSON object can build a value type a hook names: that is how a patch handler's rewritten
        // struct argument (an `IntVec3` map size) and `ref_set` on a struct member get into the game, where a
        // handle would be no use. The game-free layer only knows the *shape*, so the hook is a stand-in here.
        JsonValueBridge.StructParser = (element, target) =>
            target == typeof((int, int))
                ? (element.GetProperty("x").GetInt32(), element.GetProperty("z").GetInt32())
                : null;
        using (JsonDocument document = JsonDocument.Parse("{\"x\":3,\"z\":4}"))
        {
            object built = JsonValueBridge.Read(document.RootElement, typeof((int, int)), new RefTable());
            Expect(built is (int x, int z) && x == 3 && z == 4, "an inline object builds the value type a hook names");
        }

        using (JsonDocument document = JsonDocument.Parse("{\"x\":3}"))
        {
            bool refused = false;
            try
            {
                JsonValueBridge.Read(document.RootElement, typeof(DateTime), new RefTable());
            }
            catch (BridgeException)
            {
                refused = true;
            }

            Expect(refused, "a value type no hook knows stays refused, by name");
        }

        JsonValueBridge.StructParser = null;

        var handleRefs = new RefTable();
        var handleTarget = new object();
        string handle = Serialize(handleTarget, handleRefs);
        using (JsonDocument document = JsonDocument.Parse(handle))
        {
            Expect(document.RootElement.TryGetProperty("$ref", out _), "plain objects become $ref handles");
            Expect(
                JsonValueBridge.Read(document.RootElement, typeof(object), handleRefs) is object read
                && ReferenceEquals(read, handleTarget),
                "$ref handles read back to the same instance");
        }

        using (JsonDocument document = JsonDocument.Parse("42"))
        {
            Expect(
                Equals(JsonValueBridge.Read(document.RootElement, typeof(int), new RefTable()), 42),
                "numbers convert to the requested type");
        }

        using (JsonDocument document = JsonDocument.Parse("\"Beta\""))
        {
            Expect(
                Equals(JsonValueBridge.Read(document.RootElement, typeof(Sample), new RefTable()), Sample.Beta),
                "enums are read by name");
        }

        // Member access is hierarchy aware: the game hands out concrete types whose members often
        // live on a base class (Verse.ThingWithComps vs Verse.Thing).
        var sample = new DerivedSample();
        Expect(Equals(MemberAccess.Get(sample, "secret"), 3), "private base class fields are readable");
        MemberAccess.Set(sample, "secret", Json("7"), new RefTable());
        Expect(Equals(MemberAccess.Get(sample, "secret"), 7), "private base class fields are writable");
        Expect(Equals(MemberAccess.Get(sample, "Label"), "base"), "base class properties are readable");
        MemberAccess.Set(sample, "Label", Json("\"renamed\""), new RefTable());
        Expect(Equals(MemberAccess.Get(sample, "Label"), "renamed"), "base class properties are writable");
        Expect(
            Equals(MemberAccess.Call(sample, "Describe", Array.Empty<object>()), "secret=7"),
            "protected base class methods are callable");
        Expect(
            Equals(MemberAccess.Call(sample, "WithDefault", Array.Empty<object>()), 42),
            "optional parameters fall back to their defaults");
        Expect(
            Equals(MemberAccess.Call(sample, "WithDefault", new object[] { 5 }), 5),
            "explicit arguments win over defaults");
        Expect(
            Equals(MemberAccess.Call(sample, "Bump", new object[] { 5 }), 6),
            "by-ref parameters take their argument and answer like any other call");
        try
        {
            MemberAccess.Call(sample, "Bump", Array.Empty<object>());
            Expect(false, "a by-ref parameter without a default has to be passed");
        }
        catch (BridgeException error)
        {
            Expect(
                error.Message.Contains("Bump"),
                "a by-ref parameter without a default has to be passed");
        }
        MemberAccess.Set(sample, "Mode", Json("\"Beta\""), new RefTable());
        Expect(Equals(MemberAccess.Get(sample, "Mode"), Sample.Beta), "enums are written from strings");
        MemberAccess.Set(sample, "Mode", Json("0"), new RefTable());
        Expect(Equals(MemberAccess.Get(sample, "Mode"), Sample.Alpha), "enums are written from numbers");
        Expect(
            MemberAccess.Get(sample, "ToString") is string description && description.Contains("DerivedSample"),
            "object methods are reachable through the hierarchy");

        try
        {
            MemberAccess.Get(sample, "SECRET");
            Expect(false, "unknown members produce a helpful error");
        }
        catch (BridgeException error)
        {
            Expect(
                error.Message.Contains("did you mean") && error.Message.Contains("secret"),
                "unknown members produce a helpful error");
        }

        // Overload selection. Rust code writes "string" and "int" while reflection reports "String"
        // and "Int32", and a call without parameter types must still work when only one overload
        // can take that many arguments.
        Expect(
            MethodSelection.Select(typeof(BaseSample), "BaseSample", "Describe", Array.Empty<string>(), 0)
                .GetParameters().Length == 0,
            "overload selection uses the argument count");
        Expect(
            MethodSelection.Select(typeof(BaseSample), "BaseSample", "Describe", Array.Empty<string>(), 1)
                .GetParameters()[0].ParameterType == typeof(string),
            "one argument picks the one-argument overload");
        Expect(
            MethodSelection.Select(typeof(BaseSample), "BaseSample", "Describe", new[] { "string" }, 1)
                .GetParameters()[0].ParameterType == typeof(string),
            "parameter type names accept C# aliases and lower case");
        Expect(
            MethodSelection.Select(typeof(BaseSample), "BaseSample", "Describe", new[] { "System.String" }, 1)
                .GetParameters()[0].ParameterType == typeof(string),
            "parameter type names accept full names");
        Expect(
            MethodSelection.Select(typeof(BaseSample), "BaseSample", "Take", new[] { "List<int>" }, 1)
                .GetParameters()[0].ParameterType == typeof(System.Collections.Generic.List<int>),
            "generic parameter type names accept aliases and short names");
        Expect(
            MethodSelection.Select(typeof(BaseSample), "BaseSample", "Bump", new[] { "int" }, 1)
                .GetParameters()[0].ParameterType.IsByRef,
            "ref parameters match the type they point at");
        try
        {
            MethodSelection.Select(typeof(BaseSample), "BaseSample", "Pick", Array.Empty<string>(), 1);
            Expect(false, "an overload that stays ambiguous reports the candidates");
        }
        catch (BridgeException error)
        {
            Expect(
                error.Message.Contains("2 overloads") && error.Message.Contains("Pick(Int32)") && error.Message.Contains("Pick(String)"),
                "an overload that stays ambiguous reports the candidates");
        }

        // The call-rewriting transpiler is pure logic once Harmony's instruction shape is faked, so
        // the whole rewriting rule set is checked here instead of in a running game.
        MethodInfo fromStatic = typeof(Program).GetMethod(nameof(StaticSource), BindingFlags.NonPublic | BindingFlags.Static);
        MethodInfo toStatic = typeof(Program).GetMethod(nameof(StaticReplacement), BindingFlags.NonPublic | BindingFlags.Static);
        MethodInfo fromInstance = typeof(BaseSample).GetMethod(nameof(BaseSample.InstanceSource));
        MethodInfo toInstance = typeof(BaseSample).GetMethod(nameof(BaseSample.InstanceReplacement));
        MethodInfo unrelated = typeof(BaseSample).GetMethod(nameof(BaseSample.Unrelated));
        var instructions = new List<FakeInstruction>
        {
            new FakeInstruction(OpCodes.Call, fromStatic),
            new FakeInstruction(OpCodes.Callvirt, fromInstance),
            new FakeInstruction(OpCodes.Call, unrelated),
            new FakeInstruction(OpCodes.Ldstr, "text"),
        };
        int rewritten = InstructionRewriting.Apply(
            instructions,
            typeof(FakeInstruction),
            new[]
            {
                new InstructionRewrite { From = fromStatic, To = toStatic },
                new InstructionRewrite { From = fromInstance, To = toInstance },
            },
            _ => { });

        Expect(rewritten == 2, "call rewriting replaces only the planned calls");
        Expect(ReferenceEquals(instructions[0].operand, toStatic), "a redirected call points at the replacement");
        Expect(
            Equals(instructions[0].opcode, InstructionRewriting.CallOpcode()),
            "a static replacement uses the call opcode");
        Expect(
            ReferenceEquals(instructions[1].operand, toInstance) && Equals(instructions[1].opcode, OpCodes.Callvirt),
            "an instance replacement keeps the original callvirt");
        Expect(ReferenceEquals(instructions[2].operand, unrelated), "unrelated calls stay untouched");
        Expect(
            InstructionRewriting.Apply(instructions, typeof(string), Array.Empty<InstructionRewrite>(), _ => { }) == -1,
            "an unexpected instruction layout is reported instead of guessed");
        Expect(
            InstructionRewriting.SameMethod(fromStatic, typeof(Program).GetMethod(nameof(StaticSource), BindingFlags.NonPublic | BindingFlags.Static)),
            "method handles compare by metadata, not by instance identity");

        // The def-name generator turns game names into Rust identifiers; these are the shapes real
        // def names have (verified against the game's own defs while writing it).
        Expect(ToConstant("Steel") == "STEEL", "def names become constants");
        Expect(ToConstant("WoodLog") == "WOOD_LOG", "camel case becomes snake case");
        Expect(ToConstant("Apparel_CowboyHat") == "APPAREL_COWBOY_HAT", "separators become underscores");
        Expect(ToConstant("2x4") == "_2X4", "names starting with a digit are usable");
        Expect(ToConstant("Self") == "SELF_", "uppercase Rust keywords are escaped");
        Expect(ToConstant("BionicArm_Archotech") == "BIONIC_ARM_ARCHOTECH", "mixed names keep their shape");
        Expect(ToSnakeCase("ThingDef") == "thing_def", "def types become module names");
        Expect(ToSnakeCase("MessageTypeDef") == "message_type_def", "multi-word def types become module names");
        Expect(
            ToSnakeCase("RoomPart_BarricadeDef") == "room_part_barricade_def",
            "def types with separators become valid module names");

        // A def name lands inside a generated string literal, so it is data: a quote, a backslash or a newline
        // in the game's XML must not close the literal and turn the rest of the name into Rust source. And the
        // module name has to be an identifier even when the XML element name is not.
        Expect(EscapeRust("Steel") == "Steel", "a plain def name is written as it is");
        Expect(EscapeRust("a\"b") == "a\\\"b", "a quote in a def name is escaped");
        Expect(EscapeRust("a\\b") == "a\\\\b", "a backslash in a def name is escaped");
        Expect(EscapeRust("a\nb") == "a\\nb", "a newline in a def name is escaped");
        Expect(RustModule("thing_def") == "thing_def", "an ordinary module name is kept");
        Expect(RustModule(ToSnakeCase("X.Def")) == "x__def", "a separator cannot reach the module name");
        Expect(RustModule("2x4") == "_2x4", "a module name never starts with a digit");

        // Randomized round-trips: whatever a mod hands to the marshaling layer (nested
        // dictionaries and lists, numbers of both kinds, strings, nulls, live handles) has to come
        // back structurally identical. The seed is fixed so a failure is reproducible.
        var random = new Random(20261014);
        int randomRounds = 0;
        string randomFailure = null;
        for (int round = 0; round < 200 && randomFailure == null; round++)
        {
            object value = RandomValue(random, 0);
            var randomRefs = new RefTable();
            try
            {
                string json = Serialize(value, randomRefs);
                using JsonDocument document = JsonDocument.Parse(json);
                if (!RoundTrips(document.RootElement, value))
                {
                    randomFailure = $"round {round}: {json}";
                }
            }
            catch (Exception error)
            {
                randomFailure = $"round {round}: {error.Message}";
            }

            randomRounds++;
        }

        Expect(randomFailure == null, $"random values round-trip through the wire format ({randomRounds} rounds)");
        if (randomFailure != null)
        {
            Console.WriteLine($"      {randomFailure}");
        }

        // Reentrancy guard: a host call inside a handler may trigger a patched method, which must
        // not re-enter the Rust side. The bridge detects this through RefContext nesting.
        Expect(!RefContext.IsDispatching, "no dispatch is active at the start");
        using (RefContext.Enter())
        {
            Expect(RefContext.IsDispatching, "a running dispatch marks the thread as busy");
            using (RefContext.Enter())
            {
                Expect(RefContext.IsDispatching, "nested scopes stay inside the same dispatch");
            }

            Expect(RefContext.IsDispatching, "nested scopes end one level at a time");
        }

        Expect(!RefContext.IsDispatching, "the dispatch scope ends cleanly");

        // A deferred patch call is dispatched after the outer event, with its own handle table.
        var queuedTable = new RefTable();
        queuedTable.Add("payload object");
        using (RefContext.Enter(queuedTable))
        {
            Expect(ReferenceEquals(RefContext.Current, queuedTable), "a queued patch installs its own handle table");
            Expect(Equals(RefContext.Current.Get(0), "payload object"), "the queued table keeps its objects alive");
        }

        Console.WriteLine();
        Console.WriteLine($"{checks - failures}/{checks} marshaling checks passed");
        return failures == 0 ? 0 : 1;
    }
    private static string Serialize(object value, RefTable refs)
    {
        using var stream = new MemoryStream();
        using (var writer = new Utf8JsonWriter(stream))
        {
            JsonValueBridge.Write(writer, value, refs);
        }

        return Encoding.UTF8.GetString(stream.ToArray());
    }
}
