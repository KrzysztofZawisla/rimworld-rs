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
/// The `mock_ops` half of <see cref="Program"/>.
/// </summary>
internal static partial class Program
{

    /// <summary>Runs the mock's own operations for a batch, so batching is covered offline.</summary>
    private static string HandleBatch(JsonElement request)
    {
        if (!request.TryGetProperty("requests", out JsonElement requests) || requests.ValueKind != JsonValueKind.Array)
        {
            return "{\"ok\":false,\"error\":\"the mock needs a 'requests' array\"}";
        }

        var results = new List<string>();
        foreach (JsonElement entry in requests.EnumerateArray())
        {
            string op = entry.TryGetProperty("op", out JsonElement opValue) && opValue.ValueKind == JsonValueKind.String
                ? opValue.GetString()
                : null;
            if (string.IsNullOrEmpty(op))
            {
                return "{\"ok\":false,\"error\":\"a batch request is missing 'op'\"}";
            }

            string answer = MockRespond(op, entry);
            if (!Ok(answer))
            {
                return answer;
            }

            using JsonDocument document = JsonDocument.Parse(answer);
            results.Add(document.RootElement.GetProperty("result").GetRawText());
        }

        return $"{{\"ok\":true,\"result\":[{string.Join(",", results)}]}}";
    }
    private static string HandleLog(JsonElement request)
    {
        string message = request.TryGetProperty("message", out JsonElement messageElement)
            ? messageElement.GetString()
            : string.Empty;
        Log.Add(message);
        return "{\"ok\":true,\"result\":null}";
    }

    /// <summary>Mimics the bridge's Harmony integration, including its error paths.</summary>
    private static string HandleHarmonyPatch(JsonElement request)
    {
        string type = request.TryGetProperty("type", out JsonElement typeElement) ? typeElement.GetString() : string.Empty;
        string method = request.TryGetProperty("method", out JsonElement methodElement) ? methodElement.GetString() : string.Empty;
        bool hasParameterTypes = request.TryGetProperty("paramTypes", out JsonElement paramTypes)
            && paramTypes.ValueKind == JsonValueKind.Array
            && paramTypes.GetArrayLength() > 0;

        if (type.Contains("NoSuchType", StringComparison.Ordinal))
        {
            return $"{{\"ok\":false,\"error\":\"type not found: {type}\"}}";
        }

        if (method == "Spawn" && !hasParameterTypes)
        {
            return "{\"ok\":false,\"error\":\"method Spawn has 4 overloads; pass paramTypes\"}";
        }

        return "{\"ok\":true,\"result\":null}";
    }

    /// <summary>The mock plant's growth, so setting it reads back.</summary>
    private static string MockPlantJson()
    {
        string growth = MockPlantGrowth.ToString(System.Globalization.CultureInfo.InvariantCulture);
        return $"{{\"ok\":true,\"result\":{{\"defName\":\"Plant_Potato\",\"label\":\"potato plant\",\"growth\":{growth},\"growthRate\":0.1,\"percent\":\"58%\"}}}}";
    }
    private static string HandleSetPlantGrowth(JsonElement request)
    {
        MockPlantGrowth = request.TryGetProperty("growth", out JsonElement value) && value.ValueKind == JsonValueKind.Number
            ? value.GetSingle()
            : MockPlantGrowth;
        return MockPlantJson();
    }
    private static string HandleSaveSet(JsonElement request)
    {
        string key = SaveKey(request);
        if (request.TryGetProperty("value", out JsonElement value) && value.ValueKind == JsonValueKind.String)
        {
            SaveStore[key] = value.GetString();
        }
        else
        {
            SaveStore.Remove(key);
        }

        return "{\"ok\":true,\"result\":null}";
    }

    /// <summary>Mimics the bridge's spawn: unknown defs and unknown maps are errors, not successes.</summary>
    private static string HandleSpawn(JsonElement request)
    {
        string defName = StringArg(request, "defName");
        if (!KnownDefs.Contains(defName))
        {
            return $"{{\"ok\":false,\"error\":\"unknown ThingDef '{defName}'\"}}";
        }

        int mapId = request.TryGetProperty("mapId", out JsonElement map) && map.ValueKind == JsonValueKind.Number
            ? map.GetInt32()
            : -1;
        if (mapId != KnownMapId)
        {
            return $"{{\"ok\":false,\"error\":\"map {mapId} does not exist\"}}";
        }

        int cell = RegisterMockObject(new Dictionary<string, object> { ["x"] = 20, ["y"] = 0, ["z"] = 20 });
        int def = RegisterMockObject(new Dictionary<string, object> { ["defName"] = defName });
        int thing = RegisterMockObject(new Dictionary<string, object>
        {
            ["def"] = MockHandle(def, "Verse.ThingDef", defName),
            ["stackCount"] = 75,
            ["Destroyed"] = false,
            ["LabelShort"] = defName,
            ["Position"] = MockHandle(cell, "Verse.IntVec3", "(20, 0, 20)"),
        });
        return $"{{\"ok\":true,\"result\":{JsonSerializer.Serialize(MockHandle(thing, "Verse.Thing", defName))}}}";
    }
    private static string HandleMapGet(JsonElement request)
    {
        int mapId = request.TryGetProperty("mapId", out JsonElement map) && map.ValueKind == JsonValueKind.Number
            ? map.GetInt32()
            : -1;
        if (mapId != KnownMapId)
        {
            return $"{{\"ok\":false,\"error\":\"map {mapId} does not exist\"}}";
        }

        if (MockMap < 0)
        {
            int size = RegisterMockObject(new Dictionary<string, object> { ["x"] = 100, ["z"] = 100 });
            int biome = RegisterMockObject(new Dictionary<string, object> { ["defName"] = "TemperateForest" });
            MockMap = RegisterMockObject(new Dictionary<string, object>
            {
                ["uniqueID"] = KnownMapId,
                ["Size"] = MockHandle(size, "Verse.IntVec2", "(100, 100)"),
                ["Biome"] = MockHandle(biome, "Verse.BiomeDef", "TemperateForest"),
                ["IsPlayerHome"] = true,
            });
        }

        return $"{{\"ok\":true,\"result\":{JsonSerializer.Serialize(MockHandle(MockMap, "Verse.Map", "MockMap"))}}}";
    }
    private static string HandleMapColonists(JsonElement request)
    {
        int mapId = request.TryGetProperty("mapId", out JsonElement map) && map.ValueKind == JsonValueKind.Number
            ? map.GetInt32()
            : -1;
        if (mapId != KnownMapId)
        {
            return $"{{\"ok\":false,\"error\":\"map {mapId} does not exist\"}}";
        }

        return $"{{\"ok\":true,\"result\":[{JsonSerializer.Serialize(MockHandle(EnsureMockColonist(), "Verse.Pawn", "Mock Colonist"))}]}}";
    }

    /// <summary>The mock map's cell. Its air temperature is state, so `map_push_heat` can be read back.</summary>
    private static string HandleMapCellInfo()
    {
        string temperature = MockCellTemperature.ToString(System.Globalization.CultureInfo.InvariantCulture);
        string roof = MockRoofDefName == null ? "null" : $"\"{MockRoofDefName}\"";
        return $"{{\"ok\":true,\"result\":{{\"terrainDefName\":\"Soil\",\"roofDefName\":{roof},\"roofed\":{(MockRoofDefName == null ? "false" : "true")},\"temperature\":{temperature},\"light\":0.5,\"roomId\":-1,\"isPolluted\":false}}}}";
    }
    private static string HandleSetCellTemperature(JsonElement request)
    {
        float before = MockCellTemperature;
        MockCellTemperature = request.TryGetProperty("celsius", out JsonElement value) && value.ValueKind == JsonValueKind.Number
            ? value.GetSingle()
            : MockCellTemperature;
        string target = MockCellTemperature.ToString(System.Globalization.CultureInfo.InvariantCulture);
        string was = before.ToString(System.Globalization.CultureInfo.InvariantCulture);
        return $"{{\"ok\":true,\"result\":{{\"target\":{target},\"before\":{was},\"after\":{target},\"steps\":1,\"reached\":true}}}}";
    }
    private static string HandleSetRoof(JsonElement request)
    {
        MockRoofDefName = request.TryGetProperty("roofDefName", out JsonElement value) && value.ValueKind == JsonValueKind.String
            ? value.GetString()
            : null;
        string roof = MockRoofDefName == null ? "null" : $"\"{MockRoofDefName}\"";
        return $"{{\"ok\":true,\"result\":{{\"roofed\":{(MockRoofDefName == null ? "false" : "true")},\"roofDefName\":{roof},\"x\":20,\"z\":20}}}}";
    }
    private static string MockGasJson()
    {
        int At(string name) => MockGas.TryGetValue(name, out int density) ? density : 0;
        int any = At("BlindSmoke") + At("ToxGas") + At("RotStink") + At("DeadlifeDust");
        return $"{{\"ok\":true,\"result\":{{\"blindSmoke\":{At("BlindSmoke")},\"toxGas\":{At("ToxGas")},\"rotStink\":{At("RotStink")},\"deadlifeDust\":{At("DeadlifeDust")},\"any\":{(any > 0 ? "true" : "false")}}}}}";
    }
    private static string HandleAddGas(JsonElement request)
    {
        string type = StringArg(request, "gasType");
        int amount = NumberArg(request, "amount");
        MockGas[type] = (MockGas.TryGetValue(type, out int current) ? current : 0) + amount;
        return MockGasJson();
    }
    private static string HandleZone(int cells, bool deleted, bool hadCell = false)
    {
        int zone = RegisterMockObject(new Dictionary<string, object> { ["label"] = "Stockpile", ["cells"] = cells });
        MockZoneCells = cells;
        string extra = hadCell ? ",\"hadCell\":true" : string.Empty;
        return $"{{\"ok\":true,\"result\":{{\"handle\":{JsonSerializer.Serialize(MockHandle(zone, "Verse.Zone", "Stockpile"))},\"label\":\"Stockpile\",\"typeName\":\"Zone_Stockpile\",\"cells\":{cells},\"priority\":\"{MockStoragePriority}\",\"allowedDefCount\":3,\"deleted\":{(deleted ? "true" : "false")}{extra}}}}}";
    }
    private static string HandleMapPushHeat(JsonElement request)
    {
        float energy =
            request.TryGetProperty("energy", out JsonElement value) && value.ValueKind == JsonValueKind.Number
                ? value.GetSingle()
                : 0;
        MockCellTemperature += energy * 0.01f;
        string temperature = MockCellTemperature.ToString(System.Globalization.CultureInfo.InvariantCulture);
        return $"{{\"ok\":true,\"result\":{{\"accepted\":true,\"temperature\":{temperature}}}}}";
    }
    private static string MockFires()
    {
        int cell = RegisterMockObject(new Dictionary<string, object> { ["x"] = 20, ["y"] = 0, ["z"] = 20 });
        int fire = RegisterMockObject(new Dictionary<string, object>
        {
            ["fireSize"] = 1.0f,
            ["Position"] = MockHandle(cell, "Verse.IntVec3", "(20, 0, 20)"),
        });
        return $"{{\"ok\":true,\"result\":[{{\"handle\":{JsonSerializer.Serialize(MockHandle(fire, "RimWorld.Fire", "fire"))},\"x\":20,\"z\":20,\"size\":1.0}}]}}";
    }

    /// <summary>Lit a fire the mock then reports: the self-test reads it back and puts it out.</summary>
    private static string MockStartFire()
    {
        int cell = RegisterMockObject(new Dictionary<string, object> { ["x"] = 20, ["y"] = 0, ["z"] = 20 });
        int fire = RegisterMockObject(new Dictionary<string, object>
        {
            ["fireSize"] = 1.0f,
            ["Destroyed"] = false,
            ["Position"] = MockHandle(cell, "Verse.IntVec3", "(20, 0, 20)"),
        });
        return $"{{\"ok\":true,\"result\":{{\"started\":true,\"size\":1.0,\"firesAtCell\":1,\"handle\":{JsonSerializer.Serialize(MockHandle(fire, "RimWorld.Fire", "fire"))},\"x\":20,\"z\":20}}}}";
    }

    /// <summary>Designates or undesignates a cell, and remembers it, so the self-test can read it back.</summary>
    private static string HandleDesignateCell(JsonElement request, bool remove)
    {
        string defName = StringArg(request, "defName");
        int x = NumberArg(request, "x");
        int z = NumberArg(request, "z");
        string key = $"{defName}@{x},{z}";
        bool changed = remove ? MockDesignations.Remove(key) : MockDesignations.TryAdd(key, (x, z));
        string reason = changed
            ? string.Empty
            : $",\"reason\":\"{(remove ? "nothing was designated there" : "already designated")}\"";
        return $"{{\"ok\":true,\"result\":{{\"changed\":{(changed ? "true" : "false")},\"defName\":\"{defName}\",\"x\":{x},\"z\":{z}{reason}}}}}";
    }
    private static string MockDesignationsJson()
    {
        var entries = MockDesignations
            .OrderBy(pair => pair.Key, StringComparer.Ordinal)
            .Select(pair =>
            {
                string defName = pair.Key.Substring(0, pair.Key.IndexOf('@'));
                return $"{{\"defName\":\"{defName}\",\"label\":\"{defName}\",\"targetType\":\"Cell\",\"x\":{pair.Value.X},\"z\":{pair.Value.Z}}}";
            })
            .ToList();
        return $"{{\"ok\":true,\"result\":[{string.Join(",", entries)}]}}";
    }
    private static string MockZones()
    {
        int zone = RegisterMockObject(new Dictionary<string, object> { ["label"] = "Stockpile", ["cells"] = 9 });
        return $"{{\"ok\":true,\"result\":[{{\"handle\":{JsonSerializer.Serialize(MockHandle(zone, "Verse.Zone", "Stockpile"))},\"label\":\"Stockpile\",\"typeName\":\"Zone_Stockpile\",\"cells\":9,\"priority\":\"{MockStoragePriority}\",\"allowedDefCount\":3}}]}}";
    }

    /// <summary>Allows or forbids the asked-for def: the mock's filter takes what the request says.</summary>
    private static string HandleStorageAllow(JsonElement request)
    {
        string defName = StringArg(request, "defName");
        bool allow = request.TryGetProperty("allow", out JsonElement value) && value.ValueKind == JsonValueKind.True;
        int count = allow ? 3 : 2;
        return $"{{\"ok\":true,\"result\":{{\"label\":\"Shelf\",\"defName\":\"{defName}\",\"before\":true,\"after\":{(allow ? "true" : "false")},\"allowedDefCount\":{count}}}}}";
    }
    private static string HandleStoragePriority(JsonElement request)
    {
        string before = MockStoragePriority;
        MockStoragePriority = StringArg(request, "priority");
        return $"{{\"ok\":true,\"result\":{{\"label\":\"Shelf\",\"before\":\"{before}\",\"after\":\"{MockStoragePriority}\"}}}}";
    }
    private static string MockFilth()
    {
        int cell = RegisterMockObject(new Dictionary<string, object> { ["x"] = 20, ["y"] = 0, ["z"] = 20 });
        int def = RegisterMockObject(new Dictionary<string, object> { ["defName"] = "Filth_Dirt" });
        int filth = RegisterMockObject(new Dictionary<string, object>
        {
            ["thickness"] = 1,
            ["Label"] = "dirt",
            ["def"] = MockHandle(def, "Verse.ThingDef", "Filth_Dirt"),
            ["Position"] = MockHandle(cell, "Verse.IntVec3", "(20, 0, 20)"),
        });
        return $"{{\"ok\":true,\"result\":[{{\"handle\":{JsonSerializer.Serialize(MockHandle(filth, "RimWorld.Filth", "dirt"))},\"defName\":\"Filth_Dirt\",\"label\":\"dirt\",\"thickness\":1,\"x\":20,\"z\":20}}]}}";
    }
    private static string HandleIntVec3(JsonElement request)
    {
        int x = request.TryGetProperty("x", out JsonElement px) && px.ValueKind == JsonValueKind.Number ? px.GetInt32() : 0;
        int y = request.TryGetProperty("y", out JsonElement py) && py.ValueKind == JsonValueKind.Number ? py.GetInt32() : 0;
        int z = request.TryGetProperty("z", out JsonElement pz) && pz.ValueKind == JsonValueKind.Number ? pz.GetInt32() : 0;
        int cell = RegisterMockObject(new Dictionary<string, object> { ["x"] = x, ["y"] = y, ["z"] = z });
        return $"{{\"ok\":true,\"result\":{JsonSerializer.Serialize(MockHandle(cell, "Verse.IntVec3", $"({x}, {y}, {z})"))}}}";
    }
    private static string HandleRefGet(JsonElement request)
    {
        if (!TryResolveMockObject(request, out Dictionary<string, object> members))
        {
            return "{\"ok\":true,\"result\":null}";
        }

        string member = StringArg(request, "member");
        return members.TryGetValue(member, out object value)
            ? $"{{\"ok\":true,\"result\":{JsonSerializer.Serialize(value)}}}"
            : "{\"ok\":true,\"result\":null}";
    }
    private static string HandleRefSet(JsonElement request)
    {
        if (!TryResolveMockObject(request, out Dictionary<string, object> members))
        {
            return "{\"ok\":false,\"error\":\"unknown handle\"}";
        }

        string member = StringArg(request, "member");
        members[member] = request.TryGetProperty("value", out JsonElement value) ? value.Clone() : null;
        return "{\"ok\":true,\"result\":null}";
    }
    private static string HandleRefCall(JsonElement request)
    {
        if (!TryResolveMockObject(request, out _))
        {
            return "{\"ok\":false,\"error\":\"unknown handle\"}";
        }

        string method = StringArg(request, "method");
        return method switch
        {
            "get_MaxHitPoints" => "{\"ok\":true,\"result\":100}",
            "Destroy" => "{\"ok\":true,\"result\":null}",
            _ => "{\"ok\":true,\"result\":null}",
        };
    }
    private static string HandleSettingsDefine(JsonElement request)
    {
        SettingSchema.Clear();
        if (request.TryGetProperty("fields", out JsonElement fields) && fields.ValueKind == JsonValueKind.Array)
        {
            foreach (JsonElement field in fields.EnumerateArray())
            {
                string key = StringArg(field, "key");
                string kind = field.TryGetProperty("kind", out JsonElement kindElement) && kindElement.ValueKind == JsonValueKind.String
                    ? kindElement.GetString()
                    : "string";
                string fallback = field.TryGetProperty("default", out JsonElement defaultElement) && defaultElement.ValueKind == JsonValueKind.String
                    ? defaultElement.GetString()
                    : null;
                SettingSchema.Add((key, kind, fallback));
                if (fallback != null && !SettingStore.ContainsKey(key))
                {
                    SettingStore[key] = fallback;
                }
            }
        }

        return "{\"ok\":true,\"result\":null}";
    }
    private static string HandleSettingsSchema()
    {
        var schema = SettingSchema
            .Select(field => new Dictionary<string, object>
            {
                ["key"] = field.Key,
                ["label"] = field.Key,
                ["kind"] = field.Kind,
                ["default"] = field.Default,
                ["min"] = 0f,
                ["max"] = 1f,
            })
            .ToList();
        return $"{{\"ok\":true,\"result\":{JsonSerializer.Serialize(schema)}}}";
    }
    private static string HandleSettingSet(JsonElement request)
    {
        string key = StringArg(request, "key");
        if (request.TryGetProperty("value", out JsonElement value) && value.ValueKind == JsonValueKind.String)
        {
            SettingStore[key] = value.GetString();
        }
        else
        {
            SettingStore.Remove(key);
        }

        return "{\"ok\":true,\"result\":null}";
    }
    private static string HandleDialog(JsonElement request)
    {
        if (request.TryGetProperty("handler", out JsonElement handler) && handler.ValueKind == JsonValueKind.String)
        {
            Dialogs.Add(handler.GetString());
        }

        return "{\"ok\":true,\"result\":null}";
    }
}
