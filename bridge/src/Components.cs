using System;
using System.Collections.Generic;
using System.Text.Json;
using UnityEngine;
using Verse;

namespace RustBridge;

/// <summary>
/// The mod class RimWorld discovers in the bridge assembly. Its only job is to load the Rust
/// library of this mod, forward the lifecycle into Rust and own the persisted settings.
/// </summary>
public sealed class RustMod : Mod
{
    public static RustMod Current { get; private set; }

    private readonly List<SettingField> settingFields = new();

    internal RustHost Host { get; private set; }

    internal RustModSettings Settings { get; private set; }

    /// <summary>Settings declared from Rust with `settings_define`.</summary>
    internal IReadOnlyList<SettingField> SettingFields => settingFields;

    internal void SetSettingFields(List<SettingField> fields)
    {
        settingFields.Clear();
        settingFields.AddRange(fields);
    }

    public RustMod(ModContentPack content) : base(content)
    {
        Current = this;
        Settings = (RustModSettings)RustSettingsFactory.Create(this);
        Host = RustHost.TryCreate(content.Name, content.RootDir);
        Host?.DispatchVoid("mod_loaded", new Dictionary<string, object>
        {
            ["modName"] = content.Name,
            ["packageId"] = content.PackageId,
            ["rootDir"] = content.RootDir,
            ["gameVersion"] = RimWorld.VersionControl.CurrentVersionString,
        });
    }

    /// <summary>The mod options window; the button appears once Rust declared settings.</summary>
    public override void DoSettingsWindowContents(Rect inRect)
    {
        SettingsWindow.Draw(this, inRect);
    }

    public override string SettingsCategory()
    {
        return Content?.Name ?? "Rust mod";
    }

    public override void WriteSettings()
    {
        base.WriteSettings();
    }
}

/// <summary>
/// String key/value settings owned by a Rust mod; written to the mod's config file.
///
/// This is the base class: every mod's bridge assembly generates its own class on top of it
/// (`RustModSettingsRimWorldRustBridgeHelloRustMod`) because the game's config file stores settings
/// by class name, and two assemblies defining the same name would resolve to the wrong one.
/// </summary>
public class RustModSettings : ModSettings
{
    private Dictionary<string, string> values = new();

    public string Get(string key)
    {
        return key != null && values.TryGetValue(key, out string value) ? value : null;
    }

    /// <summary>Stores a value and rewrites the mod's config file.</summary>
    public void Set(string key, string value)
    {
        SetSilent(key, value);
        RustMod.Current?.WriteSettings();
    }

    /// <summary>Stores a value without touching the disk (used while a settings window is open).</summary>
    public void SetSilent(string key, string value)
    {
        if (key == null)
        {
            return;
        }

        if (value == null)
        {
            values.Remove(key);
        }
        else
        {
            values[key] = value;
        }
    }

    public override void ExposeData()
    {
        Scribe_Collections.Look(ref values, "values", LookMode.Value, LookMode.Value);
        if (Scribe.mode == LoadSaveMode.PostLoadInit && values == null)
        {
            values = new Dictionary<string, string>();
        }
    }
}

/// <summary>
/// Teaches the game-agnostic JSON layer about game types that behave like primitives:
/// <c>Verse.TaggedString</c> is what most labels, descriptions and letter texts actually are, and
/// treating it as an object handle would force every mod to call `ToString` by hand.
/// </summary>
internal static class TypeHooks
{
    private static bool installed;

    public static void Install()
    {
        if (installed)
        {
            return;
        }

        installed = true;
        JsonValueBridge.StringLikeConverter = value => value is TaggedString tagged ? tagged.ToString() : null;
        JsonValueBridge.StringLikeParser = (text, target) =>
            target == typeof(TaggedString) ? (object)(TaggedString)text : null;

        // A struct travels as an inline object (`{"x":1,"y":0,"z":2}`) where a *value type* has to be built: a
        // handle is no use there, because `ref_set` and a rewritten patch argument are assigned directly.
        // `IntVec3` is the one that matters - a map size, a cell, a position - and it is built by name here,
        // the way the string-like hook above names `TaggedString`.
        JsonValueBridge.StructParser = (element, target) =>
        {
            if (target == typeof(IntVec3))
            {
                return new IntVec3(StructInt(element, "x"), StructInt(element, "y"), StructInt(element, "z"));
            }

            return null;
        };
    }

    private static int StructInt(JsonElement element, string name) =>
        element.TryGetProperty(name, out JsonElement value) ? value.GetInt32() : 0;
}

/// <summary>
/// Saved data shared by the game and map components. Every access goes through a lock, because
/// Rust may write from a patch handler on a worker thread while the game scribes the save.
/// </summary>
public interface ISavedDataHolder
{
    string GetSaved(string key);

    void SetSaved(string key, string value);

    List<string> SavedKeys();
}

internal sealed class SavedDataStore : ISavedDataHolder
{
    private readonly object gate = new();
    private Dictionary<string, string> values = new();

    public string GetSaved(string key)
    {
        lock (gate)
        {
            return key != null && values.TryGetValue(key, out string value) ? value : null;
        }
    }

    public void SetSaved(string key, string value)
    {
        lock (gate)
        {
            if (key == null)
            {
                return;
            }

            if (value == null)
            {
                values.Remove(key);
            }
            else
            {
                values[key] = value;
            }
        }
    }

    public List<string> SavedKeys()
    {
        lock (gate)
        {
            var keys = new List<string>(values.Keys);
            keys.Sort(StringComparer.Ordinal);
            return keys;
        }
    }

    /// <summary>Scribes the dictionary; called by the owning component's ExposeData.</summary>
    public void ExposeData()
    {
        lock (gate)
        {
            Scribe_Collections.Look(ref values, "rustData", LookMode.Value, LookMode.Value);
            if (Scribe.mode == LoadSaveMode.PostLoadInit && values == null)
            {
                values = new Dictionary<string, string>();
            }
        }
    }
}

/// <summary>
/// Ticks and game lifecycle events for every loaded game.
///
/// Abstract on purpose: the save file records a component by class name, so the type that gets written has to be
/// this mod's own - the csproj generates `RustGameComponent_&lt;Mod&gt;` for every bridge, and `RustPerModComponents`
/// is how the code below reaches it. Two mods sharing one concrete name would hand the save's component to
/// whichever assembly the game resolves first.
/// </summary>
public abstract class RustGameComponent : GameComponent, ISavedDataHolder
{
    private readonly Game game;
    private readonly SavedDataStore savedData = new();
    private bool savedThisSession;
    private bool worldReadyReported;

    protected RustGameComponent(Game game)
    {
        this.game = game;
    }

    public string GetSaved(string key) => savedData.GetSaved(key);

    public void SetSaved(string key, string value) => savedData.SetSaved(key, value);

    public List<string> SavedKeys() => savedData.SavedKeys();

    public override void ExposeData()
    {
        // `game_saving` runs before our own data is scribed, so anything Rust writes while
        // handling it ends up in this save file. Writing to the game is refused during this
        // window (see RustHost.InsideScribe): the game is mid-serialization.
        if (Scribe.mode == LoadSaveMode.Saving && RustHost.Current is RustHost host)
        {
            savedThisSession = true;
            host.InsideScribe = true;
            try
            {
                host.DispatchVoid("game_saving", new Dictionary<string, object>
                {
                    ["tick"] = game?.tickManager?.TicksGame ?? -1,
                });
            }
            finally
            {
                host.InsideScribe = false;
            }
        }

        savedData.ExposeData();
    }

    public override void GameComponentTick()
    {
        Presentation.MaintainSounds();

        RustHost host = RustMod.Current?.Host;
        if (host == null)
        {
            return;
        }

        ReportWorldReady(host);

        // The save is over once the next tick runs, and the game is usable again.
        if (savedThisSession)
        {
            savedThisSession = false;
            host.DispatchVoid("game_saved", new Dictionary<string, object>
            {
                ["tick"] = game?.tickManager?.TicksGame ?? -1,
            });
        }

        if (!host.ShouldDispatchTick())
        {
            return;
        }

        host.DispatchVoid("tick", new Dictionary<string, object>
        {
            ["tick"] = game.tickManager?.TicksGame ?? -1,
        });
    }

    public override void FinalizeInit()
    {
        Dispatch("game_ready");
    }

    public override void StartedNewGame()
    {
        Dispatch("game_started");
    }

    public override void LoadedGame()
    {
        Dispatch("game_loaded");
    }

    /// <summary>Reports the planet once, on the first tick that has one (new game and load alike).</summary>
    private void ReportWorldReady(RustHost host)
    {
        if (worldReadyReported)
        {
            return;
        }

        if (Find.World == null)
        {
            return;
        }

        worldReadyReported = true;
        host.DispatchVoid("world_ready", new Dictionary<string, object>
        {
            ["seed"] = Find.World.info?.seedString,
            ["tileCount"] = Find.WorldGrid?.TilesCount ?? 0,
        });
    }

    private void Dispatch(string name)
    {
        RustMod.Current?.Host?.DispatchVoid(name, null);
    }
}

/// <summary>
/// Map lifecycle events for every map, plus per-map saved data.
///
/// Abstract for the same reason as <see cref="RustGameComponent"/>: the map component is named by class in the
/// save, so every mod's bridge carries its own generated subclass (`RustMapComponent_&lt;Mod&gt;`).
/// </summary>
public abstract class RustMapComponent : MapComponent, ISavedDataHolder
{
    private readonly SavedDataStore savedData = new();
    private bool generated;

    protected RustMapComponent(Map map) : base(map)
    {
    }

    public string GetSaved(string key) => savedData.GetSaved(key);

    public void SetSaved(string key, string value) => savedData.SetSaved(key, value);

    public List<string> SavedKeys() => savedData.SavedKeys();

    public override void ExposeData()
    {
        savedData.ExposeData();
    }

    /// <summary>
    /// The map is fully initialized: the first usable moment for a loaded map, and (verified in
    /// game) the moment *before* <see cref="MapGenerated"/> for a freshly generated one.
    /// </summary>
    public override void FinalizeInit()
    {
        RustHost host = RustMod.Current?.Host;
        if (host == null)
        {
            return;
        }

        host.DispatchVoid("map_ready", new Dictionary<string, object>
        {
            ["id"] = map.uniqueID,
            ["biome"] = map.Biome?.defName,
            ["tile"] = map.TileInfo.tile.tileId,
            ["sizeX"] = map.Size.x,
            ["sizeZ"] = map.Size.z,
            ["colonists"] = map.mapPawns?.FreeColonists?.Count ?? 0,
            ["isPlayerHome"] = map.IsPlayerHome,
        });
    }

    /// <summary>
    /// Draws what Rust asked for. Dormant until the mod enables it with `draw_enable(true)`,
    /// because it costs one host call per drawn frame.
    /// </summary>
    public override void MapComponentDraw()
    {
        RustHost host = RustMod.Current?.Host;
        if (host == null || !host.DrawingEnabled)
        {
            return;
        }

        host.InsideDraw = true;
        try
        {
            host.DispatchVoid("draw", new Dictionary<string, object> { ["mapId"] = map.uniqueID });
        }
        finally
        {
            host.InsideDraw = false;
        }
    }

    public override void MapGenerated()
    {
        generated = true;
        Dispatch("map_generated", generated);
    }

    public override void MapRemoved()
    {
        Dispatch("map_removed", generated);
    }

    private void Dispatch(string name, bool generatedFlag)
    {
        RustHost host = RustMod.Current?.Host;
        if (host == null)
        {
            return;
        }

        host.DispatchVoid(name, new Dictionary<string, object>
        {
            ["id"] = map.uniqueID,
            ["biome"] = map.Biome?.defName,
            ["tile"] = map.TileInfo.tile.tileId,
            ["sizeX"] = map.Size.x,
            ["sizeZ"] = map.Size.z,
            ["colonists"] = map.mapPawns?.FreeColonists?.Count ?? 0,
            ["isPlayerHome"] = map.IsPlayerHome,
            ["generated"] = generatedFlag,
        });
    }
}

/// <summary>Runs once after all defs are loaded, before the main menu shows up.</summary>
[StaticConstructorOnStartup]
public static class RustStartup
{
    static RustStartup()
    {
        GameEvents.Install();
        RustMod.Current?.Host?.DispatchVoid("defs_loaded", null);
    }
}

/// <summary>
/// Moves mod settings written by an older bridge onto the per-mod settings class.
///
/// Bridges before the per-mod classes stored settings under one shared name
/// (`RustBridge.RustModSettings`), which the game resolves to whichever mod's assembly the runtime
/// finds first — reading such a file for the *other* mod fails with
/// `Unable to cast object of type 'RustBridge.RustModSettings' to type
/// 'RustBridge.RustModSettings_<Assembly>'`. The class name is the root element of the mod's own
/// settings file (`Config/Mod_<folder>_<ModType>.xml`, verified on disk), so renaming that element
/// keeps the player's values and stops the error.
/// </summary>
internal static class RustSettingsMigration
{
    private const string LegacyClass = "<ModSettings Class=\"RustBridge.RustModSettings\">";

    public static void RenameLegacySettings(Mod mod, Type settingsType)
    {
        try
        {
            if (mod?.Content == null)
            {
                return;
            }

            string path = System.IO.Path.Combine(
                GenFilePaths.ConfigFolderPath,
                $"Mod_{mod.Content.FolderName}_{mod.GetType().Name}.xml");
            if (!System.IO.File.Exists(path))
            {
                return;
            }

            string text = System.IO.File.ReadAllText(path);
            if (!text.Contains(LegacyClass, StringComparison.Ordinal))
            {
                return;
            }

            string renamed = text.Replace(
                LegacyClass,
                $"<ModSettings Class=\"{settingsType.FullName}\">",
                StringComparison.Ordinal);
            System.IO.File.WriteAllText(path, renamed);
            Verse.Log.Message($"[Rust] moved the old mod settings of {mod.Content.Name} onto {settingsType.Name}");
        }
        catch (Exception error)
        {
            Verse.Log.Warning($"[Rust] could not migrate old mod settings: {error.Message}");
        }
    }
}
