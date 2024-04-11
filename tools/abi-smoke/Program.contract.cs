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
/// The `contract` half of <see cref="Program"/>.
/// </summary>
internal static partial class Program
{

    /// <summary>
    /// Verifies that the installed game still exposes everything the bridge compiles against by
    /// reading the metadata of Assembly-CSharp.dll directly. This catches game updates that would
    /// break the bridge, without launching the game.
    /// </summary>
    private static int RunContractCheck(string directory, string repo)
    {
        string path = Path.Combine(directory, "Assembly-CSharp.dll");
        if (!File.Exists(path))
        {
            Console.Error.WriteLine($"no Assembly-CSharp.dll in {Path.GetFullPath(directory)}");
            return 2;
        }

        using FileStream stream = File.OpenRead(path);
        using var peReader = new PEReader(stream);
        if (!peReader.HasMetadata)
        {
            Console.Error.WriteLine($"{path} has no managed metadata");
            return 2;
        }

        MetadataReader reader = peReader.GetMetadataReader();
        var types = new Dictionary<string, HashSet<string>>(StringComparer.Ordinal);
        foreach (TypeDefinitionHandle handle in reader.TypeDefinitions)
        {
            TypeDefinition type = reader.GetTypeDefinition(handle);
            string name = reader.GetString(type.Name);
            string namespaceName = reader.GetString(type.Namespace);
            string fullName = string.IsNullOrEmpty(namespaceName) ? name : namespaceName + "." + name;
            if (!types.TryGetValue(fullName, out HashSet<string> members))
            {
                members = new HashSet<string>(StringComparer.Ordinal);
                types[fullName] = members;
            }

            foreach (MethodDefinitionHandle method in type.GetMethods())
            {
                members.Add(reader.GetString(reader.GetMethodDefinition(method).Name));
            }

            foreach (FieldDefinitionHandle field in type.GetFields())
            {
                members.Add(reader.GetString(reader.GetFieldDefinition(field).Name));
            }

            foreach (PropertyDefinitionHandle property in type.GetProperties())
            {
                members.Add(reader.GetString(reader.GetPropertyDefinition(property).Name));
            }
        }

        Console.Error.WriteLine($"game assembly : {path}");
        int failures = 0;
        foreach ((string typeName, string memberName) in ContractRequirements)
        {
            string label = memberName.Length > 0 ? $"{typeName}.{memberName}" : typeName;
            if (!types.TryGetValue(typeName, out HashSet<string> members))
            {
                Console.WriteLine($"FAIL  missing type {typeName}");
                failures++;
            }
            else if (memberName.Length > 0 && !members.Contains(memberName))
            {
                Console.WriteLine($"FAIL  missing member {label}");
                failures++;
            }
            else
            {
                Console.WriteLine($"ok    {label}");
            }
        }

        Console.WriteLine();
        Console.WriteLine($"{ContractRequirements.Length - failures}/{ContractRequirements.Length} contract requirements satisfied");
        // The same surface the in-game `member_scan` walks, counted straight from the metadata: the number
        // a coverage claim rests on can be re-checked on any machine that has the assembly.
        long declared = types.Values.Sum(set => (long)set.Count);
        Console.WriteLine($"game assembly : {types.Count} types, {declared} declared members ({path})");
        Console.WriteLine();
        int hookFailures = CheckEventHooks(reader, repo);
        int wrapperFailures = CheckSubsystems(reader, repo);
        return failures == 0 && hookFailures == 0 && wrapperFailures == 0 ? 0 : 1;
    }

    /// <summary>
    /// Verifies the Rust subsystem wrappers (crates/rimworld-rs/src/subsystems.rs) against the game
    /// assembly. Every wrapper declares the game's own type it wraps, the type it hangs off and the member
    /// of that type, and every member it names — this walks those declarations and checks each name against
    /// the real metadata, so a member the SDK would reach for by a wrong name fails `cargo xtask test`
    /// instead of a running game.
    /// </summary>
    private static int CheckSubsystems(MetadataReader reader, string repo)
    {
        // Both wrapper modules: the hand-written one, which is a directory of theme files since the layout rule
        // split it, and the generated one, which declares its wrappers with the same shape (`subsystem!` and
        // `wrapper!`).
        string handWritten = Path.Combine(repo, "crates", "rimworld-rs", "src", "subsystems");
        string generated = Path.Combine(repo, "crates", "rimworld-rs", "src", "subsystems_extra.rs");
        if (!Directory.Exists(handWritten) && !File.Exists(generated))
        {
            Console.Error.WriteLine($"no {handWritten} and no {generated}: the subsystem wrappers cannot be checked");
            return 1;
        }

        string text = string.Empty;
        if (Directory.Exists(handWritten))
        {
            foreach (string file in Directory.GetFiles(handWritten, "*.rs", SearchOption.AllDirectories))
            {
                text += File.ReadAllText(file);
            }
        }

        if (File.Exists(generated))
        {
            text += File.ReadAllText(generated);
        }

        var provider = new TypeNameProvider(reader);
        int wrappers = 0;
        int members = 0;
        int failures = 0;

        bool HasMember(TypeDefinitionHandle handle, string name)
        {
            TypeDefinitionHandle? current = handle;
            while (current != null)
            {
                TypeDefinition type = reader.GetTypeDefinition(current.Value);
                foreach (FieldDefinitionHandle field in type.GetFields())
                {
                    if (reader.GetString(reader.GetFieldDefinition(field).Name) == name)
                    {
                        return true;
                    }
                }

                foreach (PropertyDefinitionHandle property in type.GetProperties())
                {
                    if (reader.GetString(reader.GetPropertyDefinition(property).Name) == name)
                    {
                        return true;
                    }
                }

                foreach (MethodDefinitionHandle method in type.GetMethods())
                {
                    if (reader.GetString(reader.GetMethodDefinition(method).Name) == name)
                    {
                        return true;
                    }
                }

                current = type.BaseType.IsNil ? null : FindType(reader, provider.TypeName(type.BaseType), quiet: true);
            }

            return false;
        }

        foreach (Match match in Regex.Matches(
                     text,
                     @"(?:subsystem|wrapper)!\(\s*(?:///[^\n]*\n\s*)*(?<name>[A-Za-z_0-9]+),\s*""(?<parent>[^""]*)""\s*,\s*""(?<type>[^""]*)""\s*,\s*""(?<accessor>[^""]*)""\s*,\s*\[(?<members>[\s\S]*?)\]\s*\);",
                     RegexOptions.Singleline))
        {
            wrappers++;
            string typeName = match.Groups["type"].Value;
            string parentName = match.Groups["parent"].Value;
            string accessor = match.Groups["accessor"].Value;

            TypeDefinitionHandle? handle = FindType(reader, typeName, quiet: true);
            if (handle == null)
            {
                Console.WriteLine($"FAIL  {typeName}: the game has no such type");
                failures++;
                continue;
            }

            if (parentName.Length > 0 && accessor.Length > 0)
            {
                TypeDefinitionHandle? parent = FindType(reader, parentName, quiet: true);
                if (parent == null)
                {
                    Console.WriteLine($"FAIL  {typeName}: the game has no parent type {parentName}");
                    failures++;
                }
                else if (!HasMember(parent.Value, accessor))
                {
                    Console.WriteLine($"FAIL  {parentName}.{accessor}: the accessor the wrapper declares is not there");
                    failures++;
                }
            }

            foreach (Match memberMatch in Regex.Matches(match.Groups["members"].Value, "\"([^\"]+)\""))
            {
                members++;
                string member = memberMatch.Groups[1].Value;
                if (!HasMember(handle.Value, member))
                {
                    Console.WriteLine($"FAIL  {typeName}.{member}: the wrapper names a member the game does not have");
                    failures++;
                }
            }
        }

        string KindOf(TypeDefinitionHandle handle, string name)
        {
            // The route walks the hierarchy and accepts a field, a property or a zero-argument method, so
            // this collects every shape the name has and reports the most readable one.
            bool data = false;
            int fewestParameters = int.MaxValue;
            TypeDefinitionHandle? current = handle;
            while (current != null)
            {
                TypeDefinition type = reader.GetTypeDefinition(current.Value);
                foreach (FieldDefinitionHandle field in type.GetFields())
                {
                    data |= reader.GetString(reader.GetFieldDefinition(field).Name) == name;
                }

                foreach (PropertyDefinitionHandle property in type.GetProperties())
                {
                    data |= reader.GetString(reader.GetPropertyDefinition(property).Name) == name;
                }

                foreach (MethodDefinitionHandle method in type.GetMethods())
                {
                    MethodDefinition definition = reader.GetMethodDefinition(method);
                    if (reader.GetString(definition.Name) == name)
                    {
                        // How many arguments the game's method takes: `ref_get` calls a zero-argument
                        // method for you, while a method that needs arguments must be called.
                        int parameters = definition.DecodeSignature(provider, null).ParameterTypes.Length;
                        fewestParameters = Math.Min(fewestParameters, parameters);
                    }
                }

                current = type.BaseType.IsNil ? null : FindType(reader, provider.TypeName(type.BaseType), quiet: true);
            }

            if (data)
            {
                return "field or property";
            }

            return fewestParameters == int.MaxValue ? "" : $"method:{fewestParameters}";
        }

        // How the wrappers read their members: `_reader!` macros are `ref_get` (a field or a property),
        // `_call!` macros are `ref_call` (a method). The game's own shape decides which one is right, so
        // each named member is checked against the kind the metadata says it is — the mistake this catches
        // is a method read as a property, which `ref_get` refuses at runtime with "did you mean X()?".
        // Which game type each wrapper wraps, so an `impl<'a> X<'a>` block can be traced back to it.
        var wrapperTypes = new Dictionary<string, string>(StringComparer.Ordinal);
        foreach (Match call in Regex.Matches(
                     text,
                     @"(?:subsystem|wrapper)!\(\s*(?:///[^\n]*\n\s*)*(?<name>[A-Za-z_0-9]+),\s*""[^""]*""\s*,\s*""(?<type>[^""]*)""",
                     RegexOptions.Singleline))
        {
            wrapperTypes[call.Groups["name"].Value] = call.Groups["type"].Value;
        }

        foreach (Match use in Regex.Matches(
                     text,
                     @"(?<macro>[a-z_0-9]+)_(?<shape>reader|call)!\(\s*(?<function>[a-z_0-9]+),\s*""(?<member>[^""]+)""",
                     RegexOptions.Singleline))
        {
            string enclosing = "";
            foreach (Match impl in Regex.Matches(text[..use.Index], @"impl<'a> (?<name>[A-Za-z_0-9]+)<'a> \{"))
            {
                enclosing = impl.Groups["name"].Value;
            }

            if (!wrapperTypes.TryGetValue(enclosing, out string typeName))
            {
                continue;
            }

            TypeDefinitionHandle? handle = FindType(reader, typeName, quiet: true);
            if (handle == null)
            {
                continue;
            }

            string member = use.Groups["member"].Value;
            string function = use.Groups["function"].Value;
            string kind = KindOf(handle.Value, member);
            if (kind.Length == 0)
            {
                continue; // already reported as a missing member above
            }

            bool reads = use.Groups["shape"].Value == "reader";
            if (reads && kind != "field or property" && kind != "method:0")
            {
                Console.WriteLine($"FAIL  {typeName}.{member}: {function} reads a `get` on a method that takes {kind.Replace("method:", "")} argument(s) — call it instead");
                failures++;
            }
            else if (!reads && !kind.StartsWith("method", StringComparison.Ordinal))
            {
                Console.WriteLine($"FAIL  {typeName}.{member}: {function} calls a {kind} — read it instead");
                failures++;
            }
        }

        Console.WriteLine($"{wrappers} subsystem wrapper(s), {members} member name(s), {failures} wrong");
        return failures == 0 ? 0 : 1;
    }

    /// <summary>
    /// The bridge's game-state events (bridge/src/GameEvents.cs) are Harmony postfixes, and Harmony binds a
    /// handler's parameters **by name**: a parameter called `newLord` receives the game's `newLord`, while a
    /// guessed `lord` receives nothing and leaves the hook silently inactive — the only trace is the
    /// bridge's own warning at load time. This reads the hooks out of the source and checks every handler
    /// parameter against the target method's real parameters, so a guessed name fails here instead of in a
    /// running game.
    /// </summary>
    private static int CheckEventHooks(MetadataReader reader, string repo)
    {
        string source = Path.Combine(repo, "bridge", "src", "GameEvents.cs");
        if (!File.Exists(source))
        {
            Console.Error.WriteLine($"no {Path.GetFullPath(source)}: the game-state event hooks cannot be checked");
            return 1;
        }

        string text = File.ReadAllText(source);
        var handlers = new Dictionary<string, string>(StringComparer.Ordinal);
        foreach (Match match in Regex.Matches(
                     text,
                     @"private\s+static\s+void\s+(?<name>[A-Za-z_0-9]+)\s*\((?<params>[^)]*)\)",
                     RegexOptions.Singleline))
        {
            handlers[match.Groups["name"].Value] = match.Groups["params"].Value;
        }

        var provider = new TypeNameProvider(reader);
        int hooks = 0;
        int failures = 0;
        foreach (Match hook in Regex.Matches(
                     text,
                     @"Add(?:Prefix)?\(\s*""(?<type>[^""]+)""\s*,\s*""(?<method>[^""]+)""\s*,\s*(?<params>.*?),\s*nameof\((?<handler>[A-Za-z_0-9]+)\)\s*\)",
                     RegexOptions.Singleline))
        {
            hooks++;
            string typeName = hook.Groups["type"].Value;
            string methodName = hook.Groups["method"].Value;
            string handlerName = hook.Groups["handler"].Value;
            string label = $"{typeName}.{methodName} -> {handlerName}";
            string[] declared = Regex.Matches(hook.Groups["params"].Value, "\"([^\"]+)\"")
                .Select(match => match.Groups[1].Value)
                .ToArray();

            TypeDefinitionHandle? handle = FindType(reader, typeName);
            if (handle == null)
            {
                Console.WriteLine($"FAIL  {label}: no type {typeName}");
                failures++;
                continue;
            }

            TypeDefinition type = reader.GetTypeDefinition(handle.Value);
            var candidates = new List<(string Signature, string[] Types, string[] Names, bool Static)>();
            foreach (MethodDefinitionHandle methodHandle in type.GetMethods())
            {
                MethodDefinition method = reader.GetMethodDefinition(methodHandle);
                if (reader.GetString(method.Name) != methodName)
                {
                    continue;
                }

                string[] types = method.DecodeSignature(provider, null).ParameterTypes.ToArray();
                if (types.Length != declared.Length
                    || !types.Select(SimpleTypeName).SequenceEqual(declared.Select(SimpleTypeName)))
                {
                    continue;
                }

                string[] names = new string[types.Length];
                foreach (ParameterHandle parameterHandle in method.GetParameters())
                {
                    Parameter parameter = reader.GetParameter(parameterHandle);
                    if (parameter.SequenceNumber >= 1 && parameter.SequenceNumber <= names.Length && !parameter.Name.IsNil)
                    {
                        names[parameter.SequenceNumber - 1] = reader.GetString(parameter.Name);
                    }
                }

                string fullName = FullTypeName(reader, handle.Value);
                candidates.Add((
                    $"{fullName}.{methodName}({string.Join(", ", types.Zip(names, (t, n) => $"{t} {n}"))})",
                    types,
                    names,
                    (method.Attributes & MethodAttributes.Static) != 0));
            }

            if (candidates.Count == 0)
            {
                Console.WriteLine($"FAIL  {label}: {typeName} has no {methodName}({string.Join(", ", declared)})");
                failures++;
                continue;
            }

            if (candidates.Count > 1)
            {
                Console.WriteLine(
                    $"FAIL  {label}: {candidates.Count} overloads take ({string.Join(", ", declared)}), so the bridge would patch an arbitrary one");
                failures++;
                continue;
            }

            if (!handlers.TryGetValue(handlerName, out string handlerParameters))
            {
                Console.WriteLine($"FAIL  {label}: {handlerName} is not declared in {source}, so its parameters are unchecked");
                failures++;
                continue;
            }

            (string Signature, string[] Types, string[] Names, bool Static) target = candidates[0];
            bool bound = true;
            foreach ((string parameterType, string parameterName) in ParseParameters(handlerParameters))
            {
                if (parameterName.StartsWith("__", StringComparison.Ordinal))
                {
                    // Harmony's own names: `__instance` is the target object and has to match its type,
                    // while `__0`, `__result` and `__state` bind by position and need nothing from us.
                    if (parameterName == "__instance")
                    {
                        string declaring = SimpleTypeName(FullTypeName(reader, handle.Value));
                        if (target.Static)
                        {
                            Console.WriteLine($"FAIL  {label}: __instance cannot bind, {methodName} is static");
                            bound = false;
                        }
                        else if (SimpleTypeName(parameterType) != declaring)
                        {
                            Console.WriteLine(
                                $"FAIL  {label}: __instance is declared as {parameterType}, but the instance is a {declaring}");
                            bound = false;
                        }
                    }

                    continue;
                }

                int index = Array.IndexOf(target.Names, parameterName);
                if (index < 0)
                {
                    Console.WriteLine(
                        $"FAIL  {label}: Parameter \"{parameterName}\" not found in {target.Signature} (the game's parameters are: {string.Join(", ", target.Names)})");
                    bound = false;
                    continue;
                }

                if (SimpleTypeName(parameterType) != SimpleTypeName(target.Types[index]))
                {
                    Console.WriteLine(
                        $"FAIL  {label}: {parameterName} is declared as {parameterType}, but the game's {parameterName} is a {target.Types[index]}");
                    bound = false;
                }
            }

            if (!bound)
            {
                failures++;
                continue;
            }

            Console.WriteLine($"ok    {label} ({target.Signature})");
        }

        Console.WriteLine();
        Console.WriteLine($"{hooks - failures}/{hooks} game-state event hooks bind");
        return failures;
    }
}
