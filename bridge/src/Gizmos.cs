using System.Collections.Generic;
using System.Text.Json;
using UnityEngine;
using Verse;

namespace RustBridge;

/// <summary>
/// Shows the gizmos a Rust mod declares for a thing: attach it to a def with
///
/// <code>
/// &lt;Operation Class="PatchOperationAdd"&gt;
///   &lt;xpath&gt;/Defs/ThingDef[defName="Steel"]&lt;/xpath&gt;
///   &lt;value&gt;&lt;comps&gt;&lt;li Class="RustBridge.RustGizmoCompProperties_MyMod"/&gt;&lt;/comps&gt;&lt;/value&gt;
/// &lt;/Operation&gt;
/// </code>
///
/// and answer <see cref="RustBridge.Mod"/>.on_gizmos in Rust. Clicks arrive in
/// <c>on_ui_action</c> with the `action` name of the gizmo.
///
/// The XML must name the generated per-mod subclass — <c>RustGizmoCompProperties_&lt;Mod&gt;</c>, written by
/// the csproj — and never the component itself: a def's `comps` list holds `CompProperties`, and a
/// `ThingComp` there makes the def fail to load (the game logs "The value ... is not of type
/// CompProperties", then resets the mod list on the next start).
/// </summary>
public class RustGizmoComp : ThingComp
{
    public override IEnumerable<Gizmo> CompGetGizmosExtra()
    {
        RustHost host = RustHost.Current;
        if (host == null || parent == null)
        {
            yield break;
        }

        using JsonDocument document = host.DispatchWithPayload(
            "gizmos",
            new Dictionary<string, object> { ["thing"] = parent });
        JsonElement root = document.RootElement;
        if (!(root.TryGetProperty("ok", out JsonElement ok) && ok.ValueKind == JsonValueKind.True))
        {
            string error = root.TryGetProperty("error", out JsonElement message) ? message.GetString() : "unknown error";
            Log.Error($"[Rust] the gizmo request failed: {error}");
            yield break;
        }

        if (!root.TryGetProperty("result", out JsonElement declared) || declared.ValueKind != JsonValueKind.Array)
        {
            yield break;
        }

        foreach (Gizmo command in GizmoCommands.FromJson(declared))
        {
            yield return command;
        }
    }
}

/// <summary>
/// Turns the gizmos a Rust handler declared into game commands. Shared by <see cref="RustGizmoComp"/>
/// and <see cref="RustThingComp"/>, so both offer exactly the same features.
/// </summary>
internal static class GizmoCommands
{
    public static List<Gizmo> FromJson(JsonElement declared)
    {
        var commands = new List<Gizmo>();
        foreach (JsonElement entry in declared.EnumerateArray())
        {
            string label = Text(entry, "label") ?? "Rust gizmo";
            string tooltip = Text(entry, "tooltip");
            string action = Text(entry, "action");
            string hotkey = Text(entry, "hotkey");
            bool disabled = entry.TryGetProperty("disabled", out JsonElement off) && off.ValueKind == JsonValueKind.True;

            var command = new Command_Action
            {
                defaultLabel = label,
                defaultDesc = tooltip ?? string.Empty,
                action = () => RustHost.Current?.DispatchVoid(
                    "ui_action",
                    new Dictionary<string, object> { ["handler"] = action, ["id"] = string.Empty, ["value"] = "clicked" }),
            };

            if (hotkey != null)
            {
                command.hotKey = DefDatabase<KeyBindingDef>.GetNamedSilentFail(hotkey);
            }

            if (disabled)
            {
                command.Disable("the mod disabled this gizmo");
            }

            commands.Add(command);
        }

        return commands;
    }

    private static string Text(JsonElement element, string name)
    {
        return element.TryGetProperty(name, out JsonElement value) && value.ValueKind == JsonValueKind.String
            ? value.GetString()
            : null;
    }
}

/// <summary>
/// The def entry that attaches <see cref="RustGizmoComp"/>. A def's `comps` list holds
/// `CompProperties`, so the XML names this class and `compClass` points at the component: naming the
/// component itself makes the def fail to load, and the game then resets the mod list on the next
/// start.
///
/// The XML must name the generated per-mod subclass (<c>RustGizmoCompProperties_&lt;Mod&gt;</c>, see the
/// csproj), never this base class: the base exists in every bridge, and the game would give the comp
/// to whichever one loaded first.
/// </summary>
public class RustGizmoCompProperties : CompProperties
{
    public RustGizmoCompProperties()
    {
        compClass = typeof(RustGizmoComp);
        XmlCompOwner.AnnounceCompOf(GetType());
    }
}
