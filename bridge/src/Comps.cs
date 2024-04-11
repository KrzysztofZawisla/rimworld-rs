using System;
using System.Collections.Generic;
using System.Linq;
using System.Text.Json;
using Verse;

namespace RustBridge;

/// <summary>
/// The thing component the bridge itself hosts, so a Rust mod gets per-thing behaviour without
/// writing a companion assembly. Attach it to any def with an XML patch:
///
/// <code>
/// &lt;Operation Class="PatchOperationAdd"&gt;
///   &lt;xpath&gt;/Defs/ThingDef[defName="Steel"]&lt;/xpath&gt;
///   &lt;value&gt;&lt;comps&gt;&lt;li Class="RustBridge.RustThingCompProperties_MyMod"&gt;&lt;key&gt;Steel&lt;/key&gt;&lt;/li&gt;&lt;/comps&gt;&lt;/value&gt;
/// &lt;/Operation&gt;
/// </code>
///
/// The XML must name the generated per-mod subclass — <c>RustThingCompProperties_&lt;Mod&gt;</c>, written by
/// the csproj — and never the component itself: a def's `comps` list holds `CompProperties`, and a
/// `ThingComp` there makes the def fail to load. Two Rust mods both ship this base class, and the game
/// resolves an XML `Class` across every loaded assembly, so naming the base silently hands the comp to
/// whichever bridge loaded first.
///
/// Events delivered to Rust, each with the thing handle, this component's handle and the key:
/// <c>comp_spawned</c> (once per spawn, `respawning` says it came out of a save), <c>comp_despawned</c>,
/// <c>comp_tick</c> (only after `comp_tick_interval` enables it for the key), <c>comp_gizmos</c> and
/// <c>comp_inspect</c> (both pulled by the game while the thing is selected).
///
/// Per-thing state does not need this component: it is keyed by the thing's load id through
/// <see cref="CompState"/>, saved with the mod's own save data, and usable from any handler that
/// holds a thing handle.
/// </summary>
public class RustThingComp : ThingComp
{
    /// <summary>Tick intervals per comp key; empty — the default — means no Rust `comp_tick` at all.</summary>
    private static readonly Dictionary<string, int> TickIntervals = new(StringComparer.Ordinal);

    /// <summary>
    /// This comp's own tick counter. It ticks itself instead of asking the game for a hash interval
    /// (this build has no `Thing.IsHashIntervalTick`), and starts offset by the thing's load id so
    /// that things spawned together do not all call into Rust on the same frame.
    /// </summary>
    private int tickCounter;

    /// <summary>The comp key Rust sees: the XML `key`, or the thing's def name when it is empty.</summary>
    public string CompKey
    {
        get
        {
            if (props is RustThingCompProperties named && !string.IsNullOrEmpty(named.key))
            {
                return named.key;
            }

            return parent?.def?.defName;
        }
    }

    /// <summary>The `comp_tick_interval` operation: `ticks` of zero or less turns ticking off again.</summary>
    internal static void SetTickInterval(string key, int ticks)
    {
        if (string.IsNullOrEmpty(key))
        {
            throw new BridgeException("comp_tick_interval needs a comp key");
        }

        if (ticks <= 0)
        {
            TickIntervals.Remove(key);
            return;
        }

        TickIntervals[key] = ticks;
    }

    public override void PostSpawnSetup(bool respawningAfterLoad)
    {
        if (parent != null && TickIntervals.TryGetValue(CompKey ?? string.Empty, out int interval) && interval > 0)
        {
            tickCounter = Math.Abs(parent.thingIDNumber) % interval;
        }

        Dispatch("comp_spawned", respawningAfterLoad);
    }

    public override void PostDeSpawn(Map map, DestroyMode mode)
    {
        Dispatch("comp_despawned", false);
    }

    public override void CompTick()
    {
        string key = CompKey;
        if (string.IsNullOrEmpty(key) || parent == null || !parent.Spawned)
        {
            return;
        }

        if (!TickIntervals.TryGetValue(key, out int interval) || interval <= 0)
        {
            return;
        }

        if (++tickCounter < interval)
        {
            return;
        }

        tickCounter = 0;
        Dispatch("comp_tick", false);
    }

    public override IEnumerable<Gizmo> CompGetGizmosExtra()
    {
        RustHost host = RustHost.Current;
        if (host == null || parent == null)
        {
            return Array.Empty<Gizmo>();
        }

        using JsonDocument document = host.DispatchWithPayload("comp_gizmos", Payload(false));
        JsonElement root = document.RootElement;
        if (!(root.TryGetProperty("ok", out JsonElement ok) && ok.ValueKind == JsonValueKind.True))
        {
            string error = root.TryGetProperty("error", out JsonElement message)
                ? message.GetString()
                : "unknown error";
            Log.Error($"[Rust] the comp gizmo request for '{CompKey}' failed: {error}");
            return Array.Empty<Gizmo>();
        }

        if (!root.TryGetProperty("result", out JsonElement declared) || declared.ValueKind != JsonValueKind.Array)
        {
            return Array.Empty<Gizmo>();
        }

        return GizmoCommands.FromJson(declared).ToList();
    }

    public override string CompInspectStringExtra()
    {
        RustHost host = RustHost.Current;
        if (host == null || parent == null)
        {
            return null;
        }

        using JsonDocument document = host.DispatchWithPayload("comp_inspect", Payload(false));
        JsonElement root = document.RootElement;
        if (!(root.TryGetProperty("ok", out JsonElement ok) && ok.ValueKind == JsonValueKind.True))
        {
            string error = root.TryGetProperty("error", out JsonElement message)
                ? message.GetString()
                : "unknown error";
            Log.Error($"[Rust] the comp inspect request for '{CompKey}' failed: {error}");
            return null;
        }

        return root.TryGetProperty("result", out JsonElement text) && text.ValueKind == JsonValueKind.String
            ? text.GetString()
            : null;
    }

    /// <summary>
    /// Sends one comp notification (spawn, despawn, tick). The payload carries live game objects (the
    /// thing and this component), so it goes through <see cref="RustHost.SendWithPayload"/>: they travel
    /// as handles, and the event queues itself when the thing spawned inside a Rust host call —
    /// dispatching then would re-enter the native library on the thread that already holds the module
    /// lock and freeze the game (a queued event reports its own errors).
    /// </summary>
    private void Dispatch(string eventName, bool respawning)
    {
        RustHost host = RustHost.Current;
        if (host == null || parent == null)
        {
            return;
        }

        using JsonDocument document = host.SendWithPayload(eventName, Payload(respawning));
        JsonElement root = document.RootElement;
        if (!(root.TryGetProperty("ok", out JsonElement ok) && ok.ValueKind == JsonValueKind.True))
        {
            string error = root.TryGetProperty("error", out JsonElement message)
                ? message.GetString()
                : "unknown error";
            Log.Error($"[Rust] the comp handler for '{eventName}' failed: {error}");
        }
    }

    private Dictionary<string, object> Payload(bool respawning)
    {
        return new Dictionary<string, object>
        {
            ["thing"] = parent,
            ["comp"] = this,
            ["key"] = CompKey,
            // The load id: a handle's `$ref` is only an index into the handle table of one dispatch, so
            // two events about the same thing carry different numbers. The load id is the identity a
            // mod can compare and store (and it is what `comp_state_*` keys on).
            ["thingId"] = parent.thingIDNumber,
            ["respawning"] = respawning,
        };
    }
}

/// <summary>
/// Says which assembly owns the comps a def declared in XML.
///
/// The game resolves a patch's <c>Class</c> across every loaded assembly, by load order, so two Rust
/// mods that both name the same shared comp class hand the component to whichever bridge loaded
/// first: the other mod's Rust never sees its events and nothing is logged. Each bridge therefore
/// ships generated per-mod subclasses (see the csproj) and this line names the assembly the game
/// actually chose.
/// </summary>
internal static class XmlCompOwner
{
    private static readonly HashSet<string> Announced = new(StringComparer.Ordinal);

    public static void AnnounceCompOf(Type comp)
    {
        string owner = comp.Assembly.GetName().Name;
        lock (Announced)
        {
            if (!Announced.Add(owner))
            {
                return;
            }
        }

        Log.Message($"[Rust] {owner} owns the comps the defs of this mod declare in XML");
    }
}

/// <summary>
/// The def entry that attaches <see cref="RustThingComp"/>. Set <c>key</c> to name the comp for Rust
/// (it must then match `comp_tick_interval`); leaving it out uses the thing's def name.
///
/// The XML must name the generated per-mod subclass (<c>RustThingCompProperties_&lt;Mod&gt;</c>, see the
/// csproj), never this base class: the base exists in every bridge and the game would give the comp
/// to whichever one loaded first.
/// </summary>
public class RustThingCompProperties : CompProperties
{
    /// <summary>The name Rust sees for this comp; empty means "use the def name".</summary>
    public string key;

    public RustThingCompProperties()
    {
        compClass = typeof(RustThingComp);
        XmlCompOwner.AnnounceCompOf(GetType());
    }
}

/// <summary>
/// Per-thing state, stored in the mod's own save store (game scope) under `comp:&lt;loadId&gt;:&lt;name&gt;`.
///
/// It is keyed by the *thing*, not by a component, so it works for any handle a handler holds
/// (`spawn_thing`, a patch argument, `ref_get`) with or without <see cref="RustThingComp"/> attached,
/// and it travels with the save file. `comp_state_clear` removes everything stored for one thing;
/// there is no automatic cleanup when a thing is destroyed.
/// </summary>
internal static class CompState
{
    private const string Prefix = "comp:";

    public static string Get(ISavedDataHolder holder, Thing thing, string name)
    {
        return holder.GetSaved(Key(thing, name));
    }

    public static void Set(ISavedDataHolder holder, Thing thing, string name, string value)
    {
        holder.SetSaved(Key(thing, name), value);
    }

    public static List<string> Keys(ISavedDataHolder holder, Thing thing)
    {
        string prefix = PrefixOf(thing);
        var keys = new List<string>();
        foreach (string stored in holder.SavedKeys())
        {
            if (stored.StartsWith(prefix, StringComparison.Ordinal))
            {
                keys.Add(stored.Substring(prefix.Length));
            }
        }

        keys.Sort(StringComparer.Ordinal);
        return keys;
    }

    public static void Clear(ISavedDataHolder holder, Thing thing)
    {
        string prefix = PrefixOf(thing);
        foreach (string stored in holder.SavedKeys())
        {
            if (stored.StartsWith(prefix, StringComparison.Ordinal))
            {
                holder.SetSaved(stored, null);
            }
        }
    }

    private static string Key(Thing thing, string name)
    {
        if (string.IsNullOrEmpty(name))
        {
            throw new BridgeException("comp_state needs a key");
        }

        return PrefixOf(thing) + name;
    }

    private static string PrefixOf(Thing thing)
    {
        return Prefix + thing.thingIDNumber + ":";
    }
}
