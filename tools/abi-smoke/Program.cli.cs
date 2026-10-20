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
/// The `cli` half of <see cref="Program"/>.
/// </summary>
internal static partial class Program
{
    private static int Main(string[] args)
    {
        if (args.Length == 0)
        {
            return RunJsonChecks();
        }

        var flags = new HashSet<string>(StringComparer.Ordinal);
        var values = new Dictionary<string, string>(StringComparer.Ordinal);
        var positional = new List<string>();
        for (int index = 0; index < args.Length; index++)
        {
            string argument = args[index];
            if (argument is "--dir" or "--members" or "--game" or "--names" or "--out" or "--root" or "--types" or "--filter" or "--repo")
            {
                if (index + 1 < args.Length)
                {
                    values[argument] = args[++index];
                }

                continue;
            }

            if (argument.StartsWith("--", StringComparison.Ordinal))
            {
                flags.Add(argument);
                continue;
            }

            positional.Add(argument);
        }

        string directory = values.TryGetValue("--dir", out string dir) ? dir : "game-refs";
        bool fixture = flags.Contains("--fixture");

        if (flags.Contains("--contract"))
        {
            // The folder is this mode's own argument (`--contract <folder with Assembly-CSharp.dll>`), which is how
            // the repository's documentation and `cargo xtask test` call it; `--dir` - the spelling the other modes
            // share - and the `game-refs` default keep working. Reading only `--dir` here meant every one of those
            // calls silently checked game-refs instead of the installation it named.
            string folder = positional.Count > 0 ? positional[0] : directory;
            return RunContractCheck(folder, values.TryGetValue("--repo", out string repo) ? repo : ".");
        }

        if (flags.Contains("--members-json"))
        {
            if (positional.Count == 0)
            {
                Console.Error.WriteLine("usage: abi-smoke --members-json <TypeName> [<TypeName> ...] [--dir <folder>]");
                return 2;
            }

            return RunMembersJson(directory, positional);
        }
        if (flags.Contains("--targets"))
        {
            if (positional.Count != 1)
            {
                Console.Error.WriteLine("usage: abi-smoke --targets <TypeName> [--members <filter>] [--dir <folder>]");
                return 2;
            }

            values.TryGetValue("--members", out string filter);
            return RunTargets(directory, positional[0], filter);
        }

        if (flags.Contains("--coverage"))
        {
            if (positional.Count != 1)
            {
                Console.Error.WriteLine("usage: abi-smoke --coverage <TypeName> [--dir <folder>]");
                return 2;
            }

            return RunCoverage(directory, positional[0]);
        }

        if (flags.Contains("--list-types"))
        {
            return RunListTypes(directory, positional.Count == 1 ? positional[0] : null);
        }

        if (flags.Contains("--defs"))
        {
            values.TryGetValue("--game", out string gameDirectory);
            values.TryGetValue("--names", out string namesPath);
            return RunDefNamesCheck(gameDirectory, namesPath ?? "bridge/names.json");
        }

        if (flags.Contains("--emit-defs") || flags.Contains("--list-defs"))
        {
            values.TryGetValue("--game", out string gameDirectory);
            values.TryGetValue("--out", out string outPath);
            values.TryGetValue("--root", out string rootPath);
            values.TryGetValue("--types", out string typeFilter);
            values.TryGetValue("--filter", out string nameFilter);
            return RunDefScan(
                gameDirectory,
                rootPath,
                outPath,
                typeFilter,
                nameFilter,
                emit: flags.Contains("--emit-defs"),
                countsOnly: flags.Contains("--counts-only"));
        }

        if (flags.Contains("--patches"))
        {
            values.TryGetValue("--patches", out string repoPath);
            values.TryGetValue("--game", out string gameDirectory);
            return RunPatchCheck(string.IsNullOrEmpty(repoPath) ? Directory.GetCurrentDirectory() : repoPath, gameDirectory);
        }

        if (flags.Contains("--op-parity"))
        {
            values.TryGetValue("--op-parity", out string repoPath);
            return RunOpParityCheck(string.IsNullOrEmpty(repoPath) ? Directory.GetCurrentDirectory() : repoPath);
        }

        if (positional.Count != 1 || !File.Exists(positional[0]))
        {
            Console.Error.WriteLine("usage: abi-smoke [<path to the Rust cdylib>] [--fixture]");
            Console.Error.WriteLine("       abi-smoke --contract [<folder with Assembly-CSharp.dll>]");
            Console.Error.WriteLine("       abi-smoke --targets <TypeName> [--members <filter>] [--dir <folder>]");
            Console.Error.WriteLine("       abi-smoke --list-types [<prefix>] [--dir <folder>]");
            Console.Error.WriteLine("       without arguments it runs the JSON marshaling self-test");
            return 2;
        }

        string libraryPath = positional[0];
        Console.WriteLine($"library       : {libraryPath}");
        IntPtr library = NativeLibrary.Load(Path.GetFullPath(libraryPath));
        Console.WriteLine("loaded        : ok");
        var abiVersion = Marshal.GetDelegateForFunctionPointer<AbiVersionFn>(
            NativeLibrary.GetExport(library, "rwrs_abi_version"));
        var init = Marshal.GetDelegateForFunctionPointer<InitFn>(NativeLibrary.GetExport(library, "rwrs_init"));
        var dispatch = Marshal.GetDelegateForFunctionPointer<DispatchFn>(NativeLibrary.GetExport(library, "rwrs_dispatch"));
        var freeBuf = Marshal.GetDelegateForFunctionPointer<FreeBufFn>(NativeLibrary.GetExport(library, "rwrs_free_buf"));
        Console.WriteLine("exports       : ok");

        uint abi = abiVersion();
        Console.WriteLine($"abi version   : {abi}");
        Check(abi == ExpectedAbiVersion, $"ABI version is {abi}, expected {ExpectedAbiVersion}");

        var abiLayout = Marshal.GetDelegateForFunctionPointer<AbiLayoutFn>(
            NativeLibrary.GetExport(library, "rwrs_abi_layout"));
        uint sizeOfBuf = (uint)Marshal.SizeOf<NativeBuf>();
        uint offsetOfLength = (uint)Marshal.OffsetOf<NativeBuf>(nameof(NativeBuf.Len)).ToInt64();
        uint sizeOfHostApi = (uint)(2 * IntPtr.Size);
        ulong expectedLayout = sizeOfBuf | ((ulong)offsetOfLength << 16) | ((ulong)sizeOfHostApi << 32);
        ulong layout = abiLayout();
        Console.WriteLine($"abi layout    : 0x{layout:x} (expected 0x{expectedLayout:x})");
        Check(layout == expectedLayout, "the ABI layout fingerprint matches the bridge layout");

        IntPtr hostApi = Marshal.AllocHGlobal(2 * IntPtr.Size);
        Marshal.WriteIntPtr(hostApi, IntPtr.Zero);
        Marshal.WriteIntPtr(hostApi + IntPtr.Size, Marshal.GetFunctionPointerForDelegate(HostCallThunk));
        Check(init(hostApi) == 0, "rwrs_init failed");
        Console.WriteLine("init          : ok");

        string modLoaded = Drive(dispatch, freeBuf, "mod_loaded", "{\"modName\":\"HelloRust\",\"packageId\":\"rimworld.rs.hello\",\"rootDir\":\"C:/Mods/HelloRustMod\",\"gameVersion\":\"1.7\"}");
        Console.WriteLine($"mod_loaded    : {modLoaded}");
        Check(Ok(modLoaded), $"mod_loaded responded {modLoaded}");

        string defsLoaded = Drive(dispatch, freeBuf, "defs_loaded", null);
        Console.WriteLine($"defs_loaded   : {defsLoaded}");
        Check(Ok(defsLoaded), $"defs_loaded responded {defsLoaded}");

        string gameStarted = Drive(dispatch, freeBuf, "game_started", null);
        Console.WriteLine($"game_started  : {gameStarted}");
        Check(Ok(gameStarted), $"game_started responded {gameStarted}");

        string gameReady = Drive(dispatch, freeBuf, "game_ready", null);
        Console.WriteLine($"game_ready    : {gameReady}");
        Check(Ok(gameReady), $"game_ready responded {gameReady}");

        string worldReady = Drive(dispatch, freeBuf, "world_ready", "{\"tileCount\":250,\"seed\":\"mockseed\"}");
        Console.WriteLine($"world_ready   : {worldReady}");
        Check(Ok(worldReady), $"world_ready responded {worldReady}");

        string mapGenerated = Drive(dispatch, freeBuf, "map_generated", "{\"id\":1,\"biome\":\"TemperateForest\",\"tile\":42,\"sizeX\":100,\"sizeZ\":100,\"colonists\":3,\"isPlayerHome\":true,\"generated\":true}");
        Console.WriteLine($"map_generated : {mapGenerated}");
        Check(Ok(mapGenerated), $"map_generated responded {mapGenerated}");

        string mapReady = Drive(dispatch, freeBuf, "map_ready", "{\"id\":1,\"biome\":\"TemperateForest\",\"tile\":42,\"sizeX\":100,\"sizeZ\":100,\"colonists\":3,\"isPlayerHome\":true}");
        Console.WriteLine($"map_ready     : {mapReady}");
        Check(Ok(mapReady), $"map_ready responded {mapReady}");

        // The bridge's own game-state events (it patches the game for these). The mock raises
        // them itself, before the tick below, so the self-test mod's checks see them: that proves the
        // routing offline, and the in-game run proves the game's own calls arrive.
        string pawnHandle = $"{{\"$ref\":{EnsureMockColonist()},\"$type\":\"Verse.Pawn\",\"$str\":\"Mock Colonist\"}}";
        string pawnSpawned = Drive(
            dispatch,
            freeBuf,
            "pawn_spawned",
            $"{{\"pawn\":{pawnHandle},\"respawningAfterLoad\":false}}");
        Console.WriteLine($"pawn_spawned  : {pawnSpawned}");
        Check(Ok(pawnSpawned), $"pawn_spawned responded {pawnSpawned}");

        string pawnDied = Drive(
            dispatch,
            freeBuf,
            "pawn_died",
            $"{{\"pawn\":{pawnHandle},\"defName\":\"Colonist\",\"label\":\"Mock Colonist\",\"killerDefName\":\"Gun\"}}");
        Console.WriteLine($"pawn_died     : {pawnDied}");
        Check(Ok(pawnDied), $"pawn_died responded {pawnDied}");

        string pawnDown = Drive(
            dispatch,
            freeBuf,
            "pawn_down",
            $"{{\"pawn\":{pawnHandle},\"label\":\"Mock Colonist\",\"downed\":true}}");
        Console.WriteLine($"pawn_down     : {pawnDown}");
        Check(Ok(pawnDown), $"pawn_down responded {pawnDown}");

        string pawnDamaged = Drive(
            dispatch,
            freeBuf,
            "pawn_damaged",
            $"{{\"pawn\":{pawnHandle},\"damage\":12.5,\"defName\":\"Cut\",\"hitPoints\":80,\"downed\":true}}");
        Console.WriteLine($"pawn_damaged  : {pawnDamaged}");
        Check(Ok(pawnDamaged), $"pawn_damaged responded {pawnDamaged}");

        string thingDestroyed = Drive(
            dispatch,
            freeBuf,
            "thing_destroyed",
            "{\"defName\":\"Steel\",\"label\":\"steel\",\"mode\":\"Vanish\",\"mapId\":1,\"x\":20,\"z\":20}");
        Console.WriteLine($"thing_destroyed: {thingDestroyed}");
        Check(Ok(thingDestroyed), $"thing_destroyed responded {thingDestroyed}");

        string buildingCompleted = Drive(
            dispatch,
            freeBuf,
            "building_completed",
            "{\"label\":\"wall\",\"defName\":\"Wall\",\"mapId\":1,\"x\":10,\"z\":10}");
        Console.WriteLine($"building_done : {buildingCompleted}");
        Check(Ok(buildingCompleted), $"building_completed responded {buildingCompleted}");

        string raidStarted = Drive(
            dispatch,
            freeBuf,
            "raid_started",
            "{\"map\":1,\"job\":\"LordJob_AssaultColony\"}");
        Console.WriteLine($"raid_started  : {raidStarted}");
        Check(Ok(raidStarted), $"raid_started responded {raidStarted}");

        string lordDestroyed = Drive(
            dispatch,
            freeBuf,
            "lord_destroyed",
            "{\"map\":1,\"job\":\"LordJob_AssaultColony\"}");
        Console.WriteLine($"lord_destroyed: {lordDestroyed}");
        Check(Ok(lordDestroyed), $"lord_destroyed responded {lordDestroyed}");

        string researchStarted = Drive(
            dispatch,
            freeBuf,
            "research_started",
            "{\"defName\":\"Smithing\",\"label\":\"Smithing\"}");
        Console.WriteLine($"research_start: {researchStarted}");
        Check(Ok(researchStarted), $"research_started responded {researchStarted}");

        string researchFinished = Drive(
            dispatch,
            freeBuf,
            "research_finished",
            "{\"defName\":\"Smithing\",\"label\":\"Smithing\"}");
        Console.WriteLine($"research      : {researchFinished}");
        Check(Ok(researchFinished), $"research_finished responded {researchFinished}");

        string letterShown = Drive(
            dispatch,
            freeBuf,
            "letter_shown",
            "{\"defName\":\"PositiveEvent\",\"label\":\"Rust API self-test\",\"text\":\"all good\"}");
        Console.WriteLine($"letter_shown  : {letterShown}");
        Check(Ok(letterShown), $"letter_shown responded {letterShown}");

        string incidentFired = Drive(
            dispatch,
            freeBuf,
            "incident_fired",
            "{\"defName\":\"RaidEnemy\",\"label\":\"Raid\",\"fired\":true}");
        Console.WriteLine($"incident_fired: {incidentFired}");
        Check(Ok(incidentFired), $"incident_fired responded {incidentFired}");

        string questEnded = Drive(
            dispatch,
            freeBuf,
            "quest_ended",
            "{\"questId\":7,\"name\":\"The Mockers\",\"outcome\":\"Success\",\"state\":\"Ended\"}");
        Console.WriteLine($"quest_ended   : {questEnded}");
        Check(Ok(questEnded), $"quest_ended responded {questEnded}");

        string messageShown = Drive(
            dispatch,
            freeBuf,
            "message_shown",
            "{\"text\":\"a message\",\"defName\":\"NeutralEvent\",\"historical\":false}");
        Console.WriteLine($"message_shown : {messageShown}");
        Check(Ok(messageShown), $"message_shown responded {messageShown}");

        string mentalStarted = Drive(
            dispatch,
            freeBuf,
            "mental_state_started",
            $"{{\"pawn\":{pawnHandle},\"label\":\"Mock Colonist\",\"defName\":\"Berserk\",\"forced\":true}}");
        Console.WriteLine($"mental_started: {mentalStarted}");
        Check(Ok(mentalStarted), $"mental_state_started responded {mentalStarted}");

        string mentalEnded = Drive(
            dispatch,
            freeBuf,
            "mental_state_ended",
            $"{{\"pawn\":{pawnHandle},\"label\":\"Mock Colonist\",\"defName\":\"Berserk\"}}");
        Console.WriteLine($"mental_ended  : {mentalEnded}");
        Check(Ok(mentalEnded), $"mental_state_ended responded {mentalEnded}");

        string tick = Drive(dispatch, freeBuf, "tick", "{\"tick\":1234}");
        Console.WriteLine($"tick          : {tick}");
        Check(Ok(tick), $"tick responded {tick}");

        // The self-test mod reads its event counters two ticks after the suite runs, because events
        // raised inside a host call are queued and delivered when that dispatch ends. Ticking a few
        // more times is what makes the event checks run offline at all (in game the game ticks by
        // itself); without them a wrong payload key would only ever show up in a running colony.
        for (int extra = 0; extra < 3; extra++)
        {
            string next = Drive(dispatch, freeBuf, "tick", $"{{\"tick\":{1235 + extra}}}");
            Check(Ok(next), $"tick {1235 + extra} responded {next}");
        }

        string patch = Drive(dispatch, freeBuf, "patch", "{\"handler\":\"map_update\",\"kind\":\"postfix\",\"target\":\"Verse.Map.MapUpdate()\",\"instance\":{\"$ref\":0,\"$type\":\"Verse.Map\"},\"args\":[]}");
        Console.WriteLine($"patch         : {patch}");
        Check(Ok(patch), $"patch responded {patch}");

        string uiAction = Drive(dispatch, freeBuf, "ui_action", "{\"handler\":\"dialog_ok\"}");
        Console.WriteLine($"ui_action     : {uiAction}");
        Check(Ok(uiAction), $"ui_action responded {uiAction}");

        // The save window: the mock refuses game-changing operations exactly like the bridge does.
        InsideScribe = true;
        string saving = Drive(dispatch, freeBuf, "game_saving", "{\"tick\":1234}");
        InsideScribe = false;
        Console.WriteLine($"game_saving   : {saving}");
        Check(Ok(saving), $"game_saving responded {saving}");

        string saved = Drive(dispatch, freeBuf, "game_saved", "{\"tick\":1234}");
        Console.WriteLine($"game_saved    : {saved}");
        Check(Ok(saved), $"game_saved responded {saved}");

        InsideDraw = true;
        string draw = Drive(dispatch, freeBuf, "draw", "{\"mapId\":1}");
        InsideDraw = false;
        Console.WriteLine($"draw          : {draw}");
        Check(Ok(draw), $"draw responded {draw}");

        string gizmos = Drive(
            dispatch,
            freeBuf,
            "gizmos",
            "{\"thing\":{\"$ref\":0,\"$type\":\"Verse.Thing\",\"$str\":\"Steel\"}}");
        Console.WriteLine($"gizmos        : {gizmos}");
        Check(Ok(gizmos), $"gizmos responded {gizmos}");
        if (Log.Any(line => line.Contains("[selftest]")))
        {
            Check(gizmos.Contains("\"Rust gizmo\""), "the self-test mod declared a gizmo for the thing");
        }

        Check(Operations.Contains("def_names"), "the mod never asked for def_names");
        Check(Log.Count > 0, "no log messages arrived");
        if (!fixture)
        {
            Check(Operations.Contains("game_info"), "the mod never asked for game_info");
            Check(Operations.Contains("harmony_patch"), "the mod never registered a Harmony patch");
            Check(Operations.Contains("spawn_thing"), "the mod never spawned a thing");
        }

        if (fixture)
        {
            Console.WriteLine();
            Console.WriteLine("fixture checks:");

            string mutated = Drive(
                dispatch,
                freeBuf,
                "patch",
                "{\"handler\":\"mutate\",\"kind\":\"prefix\",\"target\":\"T.M(Int32, Boolean)\",\"args\":[1,false]}");
            Check(Ok(mutated), $"mutate patch responded {mutated}");
            Check(mutated.Contains("\"0\":42"), "the handler replaced argument 0");
            Check(mutated.Contains("\"result\":true"), "the handler replaced the result");
            Console.WriteLine($"  mutate -> {mutated}");

            string skipped = Drive(
                dispatch,
                freeBuf,
                "patch",
                "{\"handler\":\"skip\",\"kind\":\"prefix\",\"target\":\"T.M()\",\"args\":[]}");
            Check(skipped.Contains("\"skipOriginal\":true"), "the handler skipped the original method");
            Console.WriteLine($"  skip    -> {skipped}");

            string panic = Drive(
                dispatch,
                freeBuf,
                "patch",
                "{\"handler\":\"panic\",\"kind\":\"prefix\",\"target\":\"T.M()\",\"args\":[]}");
            Check(!Ok(panic), "a panicking handler reports an error");
            Check(panic.Contains("panic"), "the error mentions the panic");
            Console.WriteLine($"  panic   -> {panic}");

            string afterPanic = Drive(dispatch, freeBuf, "tick", "{\"tick\":2}");
            Check(Ok(afterPanic), "the library still answers after a panic");
            Console.WriteLine($"  after   -> {afterPanic}");
        }

        Console.WriteLine();
        Console.WriteLine($"log messages  : {Log.Count}");
        foreach (string line in Log)
        {
            Console.WriteLine($"  {line}");
        }

        Console.WriteLine($"host ops seen : {string.Join(", ", Operations)}");

        if (Errors.Count > 0)
        {
            Console.Error.WriteLine();
            Console.Error.WriteLine("failures:");
            foreach (string error in Errors)
            {
                Console.Error.WriteLine($"  {error}");
            }

            return 1;
        }

        Console.WriteLine();
        Console.WriteLine("ABI smoke test passed");
        return 0;
    }
    private static string Drive(DispatchFn dispatch, FreeBufFn freeBuf, string name, string json)
    {
        byte[] nameBytes = Encoding.UTF8.GetBytes(name);
        byte[] payload = json == null ? Array.Empty<byte>() : Encoding.UTF8.GetBytes(json);
        IntPtr namePtr = ToNative(nameBytes);
        IntPtr payloadPtr = payload.Length == 0 ? IntPtr.Zero : ToNative(payload);
        IntPtr slot = Marshal.AllocHGlobal(2 * IntPtr.Size);
        string result;
        try
        {
            uint code = dispatch(namePtr, nameBytes.Length, payloadPtr, payload.Length, slot);
            Check(code == 0, $"rwrs_dispatch('{name}') returned {code}");
            IntPtr pointer = Marshal.ReadIntPtr(slot);
            uint length = (uint)Marshal.ReadInt32(slot + IntPtr.Size);
            if (pointer == IntPtr.Zero || length == 0)
            {
                result = "{}";
            }
            else if (length > int.MaxValue)
            {
                Check(false, $"rwrs_dispatch('{name}') returned {length} bytes, larger than the test can copy");
                freeBuf(pointer, length);
                result = "{}";
            }
            else
            {
                var bytes = new byte[(int)length];
                Marshal.Copy(pointer, bytes, 0, bytes.Length);
                freeBuf(pointer, length);
                result = Encoding.UTF8.GetString(bytes);
            }
        }
        finally
        {
            Marshal.FreeHGlobal(slot);
            Marshal.FreeHGlobal(namePtr);
            if (payloadPtr != IntPtr.Zero)
            {
                Marshal.FreeHGlobal(payloadPtr);
            }
        }

        return result;
    }
    private static bool Ok(string response)
    {
        using JsonDocument document = JsonDocument.Parse(response);
        return document.RootElement.TryGetProperty("ok", out JsonElement ok) && ok.ValueKind == JsonValueKind.True;
    }
    private static IntPtr ToNative(byte[] bytes)
    {
        IntPtr memory = Marshal.AllocHGlobal(bytes.Length);
        Marshal.Copy(bytes, 0, memory, bytes.Length);
        return memory;
    }
    private static uint HostCall(IntPtr userData, IntPtr requestPtr, int requestLen, IntPtr responseOut)
    {
        var bytes = new byte[requestLen];
        Marshal.Copy(requestPtr, bytes, 0, requestLen);
        using JsonDocument document = JsonDocument.Parse(Encoding.UTF8.GetString(bytes));
        JsonElement request = document.RootElement;
        string op = request.TryGetProperty("op", out JsonElement opElement) ? opElement.GetString() : "?";
        Operations.Add(op);

        string response = InsideScribe && ChangesTheGame(op)
            ? $"{{\"ok\":false,\"error\":\"'{op}' cannot run while the game is being saved\"}}"
            : MockRespond(op, request);

        byte[] responseBytes = Encoding.UTF8.GetBytes(response);
        IntPtr memory = Marshal.AllocHGlobal(responseBytes.Length);
        Marshal.Copy(responseBytes, 0, memory, responseBytes.Length);
        Marshal.WriteIntPtr(responseOut, memory);
        Marshal.WriteInt32(responseOut + IntPtr.Size, responseBytes.Length);
        return 0;
    }
    private static void Check(bool condition, string message)
    {
        if (!condition)
        {
            Errors.Add(message);
        }
    }
}
