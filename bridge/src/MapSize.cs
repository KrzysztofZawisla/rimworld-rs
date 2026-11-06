using System;
using System.Collections.Generic;
using System.Reflection;
using System.Text.Json;
using Verse;

namespace RustBridge;

/// <summary>
/// A custom size for the **next** new game's starting map.
///
/// The game builds that map from `GameInitData.mapSize`, one square number, so a non-square size
/// needs two moments: a prefix on `Game.InitNewGame` replaces the number with the larger side
/// (vanilla then builds `new IntVec3(size, 1, size)`), and a prefix on `MapGenerator.GenerateMap`
/// swaps that placeholder for the exact width x height. A postfix on `InitNewGame` writes the exact
/// size into `World.info.initialMapSize`, which is what later settlements use. Better Map Sizes
/// does the same; this is the bridge's own version of it.
///
/// The size stays armed until a new game actually generates its starting map, so a failed attempt
/// does not consume it. A loaded save is never resized.
/// </summary>
internal static class MapSize
{
    /// <summary>The accepted side range; the game allocates width * height cells.</summary>
    private const int MinSide = 1;
    private const int MaxSide = 4096;

    private static readonly object Gate = new();
    private static readonly object HookGate = new();
    private static readonly List<string> HookErrors = new();

    /// <summary>The size waiting for the next new game; taken when its map is generated.</summary>
    private static (int Width, int Height)? armed;

    /// <summary>The new game currently being initialized, and the square value vanilla receives.</summary>
    private static (int Width, int Height)? pending;
    private static int placeholder;
    private static bool startingMapPending;
    private static bool newGameArmed;

    /// <summary>The exact size applied to the starting map of the game being initialized.</summary>
    private static (int Width, int Height)? appliedThisGame;

    /// <summary>The exact size of the last starting map this bridge generated with a custom size.</summary>
    private static (int Width, int Height)? lastApplied;

    private static bool generateMapHook;
    private static bool initNewGamePrefixHook;
    private static bool initNewGamePostfixHook;

    /// <summary>Arms a size for the next new game and answers with the state afterwards.</summary>
    public static object Arm(JsonElement request)
    {
        int width = JsonArgs.GetInt(request, "width", 0);
        int height = JsonArgs.GetInt(request, "height", 0);
        RequireSide(width, "width");
        RequireSide(height, "height");
        EnsureHooks();

        lock (Gate)
        {
            armed = (width, height);
        }

        Log.Message($"[Rust] the next new game will use a {width} x {height} starting map");
        return Snapshot();
    }

    /// <summary>The armed size and the size of the last starting map generated this way.</summary>
    public static object Get() => Snapshot();

    /// <summary>Disarms the size and answers whether one was armed.</summary>
    public static object Clear()
    {
        bool cleared;
        lock (Gate)
        {
            cleared = armed.HasValue;
            armed = null;
        }

        return Snapshot(cleared);
    }

    private static void RequireSide(int value, string side)
    {
        if (value < MinSide || value > MaxSide)
        {
            throw new BridgeException(
                $"the new game's map {side} must be between {MinSide} and {MaxSide} cells (got {value})");
        }
    }

    /// <summary>
    /// Installs the bridge's own prefixes and postfix once. A target this installation cannot patch
    /// fails the whole operation; the hooks that did install stay inert while nothing is armed.
    /// </summary>
    private static void EnsureHooks()
    {
        lock (HookGate)
        {
            if (generateMapHook && initNewGamePrefixHook && initNewGamePostfixHook)
            {
                return;
            }

            HookErrors.Clear();
            if (!generateMapHook)
            {
                generateMapHook = HookPrefix(
                    "Verse.MapGenerator",
                    "GenerateMap",
                    // The game's own parameter list, which also picks this overload: the same method
                    // name is used by pocket maps and the debug tools, and the bridge only swaps the
                    // starting map's placeholder.
                    new[]
                    {
                        "IntVec3", "MapParent", "MapGeneratorDef", "IEnumerable<GenStepWithParams>", "Action<Map>",
                        "bool", "bool",
                    },
                    nameof(GenerateMapPrefix));
            }

            if (!initNewGamePrefixHook)
            {
                initNewGamePrefixHook = HookPrefix("Verse.Game", "InitNewGame", Array.Empty<string>(), nameof(InitNewGamePrefix));
            }

            if (!initNewGamePostfixHook)
            {
                initNewGamePostfixHook = HookPostfix("Verse.Game", "InitNewGame", Array.Empty<string>(), nameof(InitNewGamePostfix));
            }

            if (!generateMapHook || !initNewGamePrefixHook || !initNewGamePostfixHook)
            {
                throw new BridgeException(
                    "the new-game map size needs Harmony hooks this installation cannot take: "
                    + string.Join("; ", HookErrors));
            }
        }
    }

    private static bool HookPrefix(string typeName, string methodName, string[] paramTypes, string handlerName)
    {
        return Hook(typeName, methodName, paramTypes, handlerName, prefix: true);
    }

    private static bool HookPostfix(string typeName, string methodName, string[] paramTypes, string handlerName)
    {
        return Hook(typeName, methodName, paramTypes, handlerName, prefix: false);
    }

    private static bool Hook(string typeName, string methodName, string[] paramTypes, string handlerName, bool prefix)
    {
        MethodInfo handler = typeof(MapSize).GetMethod(handlerName, BindingFlags.NonPublic | BindingFlags.Static);
        bool installed = prefix
            ? HarmonyBridge.TryInstallPrefix(typeName, methodName, paramTypes, handler, out string error)
            : HarmonyBridge.TryInstallPostfix(typeName, methodName, paramTypes, handler, out error);
        if (!installed)
        {
            HookErrors.Add($"{typeName}.{methodName}: {error}");
        }

        return installed;
    }

    /// <summary>
    /// The game is about to build a starting map. Vanilla reads `GameInitData.mapSize` once, so a
    /// square placeholder is the only way to hand it a size it can build; the exact dimensions are
    /// swapped in at <see cref="GenerateMapPrefix"/>.
    /// </summary>
    private static void InitNewGamePrefix()
    {
        (int Width, int Height)? size;
        int square;
        lock (Gate)
        {
            // A previous attempt that failed before generating a map must not leak into this game.
            pending = null;
            startingMapPending = false;
            newGameArmed = false;
            appliedThisGame = null;

            size = armed;
            if (size == null)
            {
                return;
            }

            pending = size;
            placeholder = Math.Max(size.Value.Width, size.Value.Height);
            square = placeholder;
            startingMapPending = true;
            newGameArmed = true;
        }

        GameInitData data = Find.GameInitData;
        if (data == null)
        {
            lock (Gate)
            {
                pending = null;
                startingMapPending = false;
                newGameArmed = false;
            }

            Log.Warning("[Rust] a custom starting map size was armed, but the game has no GameInitData");
            return;
        }

        data.mapSize = square;
        Log.Message($"[Rust] a new game requests a {size.Value.Width} x {size.Value.Height} starting map");
    }

    /// <summary>
    /// The starting map is the first `GenerateMap` after `InitNewGame`; the placeholder identifies it,
    /// so a pocket map or a debug map with another size is left alone.
    /// </summary>
    private static void GenerateMapPrefix(ref IntVec3 mapSize)
    {
        (int Width, int Height)? size;
        int expected;
        lock (Gate)
        {
            if (!startingMapPending || pending == null)
            {
                return;
            }

            size = pending;
            expected = placeholder;
            startingMapPending = false;
        }

        if (mapSize.x != expected || mapSize.z != expected)
        {
            lock (Gate)
            {
                pending = null;
            }

            Log.Warning(
                $"[Rust] the starting map was requested as {mapSize.x} x {mapSize.z}, not the {expected} x {expected} placeholder; another mod changed it, so the custom size is not applied");
            return;
        }

        lock (Gate)
        {
            armed = null;
            pending = null;
            appliedThisGame = size;
            lastApplied = size;
        }

        mapSize = new IntVec3(size.Value.Width, 1, size.Value.Height);
        Log.Message($"[Rust] generating the starting map as {size.Value.Width} x {size.Value.Height}");
    }

    /// <summary>
    /// Vanilla stores the square placeholder in `World.info.initialMapSize` right after generating the
    /// starting map; later settlements read that value, so it has to carry the exact dimensions.
    /// </summary>
    private static void InitNewGamePostfix()
    {
        (int Width, int Height)? applied;
        lock (Gate)
        {
            applied = newGameArmed ? appliedThisGame : null;
            newGameArmed = false;
            startingMapPending = false;
            pending = null;
            appliedThisGame = null;
        }

        if (applied == null)
        {
            return;
        }

        if (Find.World?.info != null)
        {
            Find.World.info.initialMapSize = new IntVec3(applied.Value.Width, 1, applied.Value.Height);
        }
    }

    private static Dictionary<string, object> Snapshot(bool? cleared = null)
    {
        lock (Gate)
        {
            var state = new Dictionary<string, object>
            {
                ["armed"] = armed.HasValue,
                ["width"] = armed?.Width,
                ["height"] = armed?.Height,
                ["lastApplied"] = lastApplied.HasValue
                    ? new Dictionary<string, object>
                    {
                        ["width"] = lastApplied.Value.Width,
                        ["height"] = lastApplied.Value.Height,
                    }
                    : null,
            };
            if (cleared.HasValue)
            {
                state["cleared"] = cleared.Value;
            }

            return state;
        }
    }
}
