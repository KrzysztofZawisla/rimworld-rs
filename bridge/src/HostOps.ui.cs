using System;
using System.Collections;
using System.Collections.Generic;
using System.Diagnostics;
using System.Globalization;
using System.Linq;
using System.Reflection;
using System.Text;
using System.Text.Json;
using RimWorld;
using Verse;
namespace RustBridge;

/// <summary>
/// The `ui` half of <see cref="HostOps"/>.
/// </summary>
internal static partial class HostOps
{

    /// <summary>Highlighting a circle of cells; only meaningful inside the draw callback.</summary>
    private static void DrawRadiusRing(JsonElement request)
    {
        RequireDraw("draw_radius_ring");
        var center = (IntVec3)JsonValueBridge.Read(
            request.GetProperty("cell"),
            typeof(IntVec3),
            RefContext.Current);
        float radius = (float)JsonArgs.GetDouble(request, "radius", 1);
        GenDraw.DrawRadiusRing(center, radius);
    }

    /// <summary>A line between two cells; only meaningful inside the draw callback.</summary>
    private static void DrawLine(JsonElement request)
    {
        RequireDraw("draw_line");
        IntVec3 from = ReadCell(request, "from");
        IntVec3 to = ReadCell(request, "to");
        GenDraw.DrawLineBetween(from.ToVector3Shifted(), to.ToVector3Shifted());
    }

    /// <summary>A circle outline around a cell; only meaningful inside the draw callback.</summary>
    private static void DrawCircle(JsonElement request)
    {
        RequireDraw("draw_circle");
        IntVec3 center = ReadCell(request, "center");
        float radius = (float)JsonArgs.GetDouble(request, "radius", 1);
        GenDraw.DrawCircleOutline(center.ToVector3Shifted(), radius);
    }

    /// <summary>Floating text above a cell; only meaningful inside the draw callback.</summary>
    private static void DrawText(JsonElement request)
    {
        RequireDraw("draw_text");
        IntVec3 cell = ReadCell(request, "cell");
        Map map = Find.CurrentMap ?? throw new BridgeException("there is no current map to draw on");
        MoteMaker.ThrowText(cell.ToVector3Shifted(), map, JsonArgs.GetString(request, "text"), 3f);
    }
    private static IntVec3 ReadCell(JsonElement request, string name)
    {
        if (!request.TryGetProperty(name, out JsonElement cell))
        {
            throw new BridgeException($"the request needs a '{name}' cell handle (create one with ctx.vec3)");
        }

        return (IntVec3)JsonValueBridge.Read(cell, typeof(IntVec3), RefContext.Current);
    }

    /// <summary>Highlighting single cells with a colour; only meaningful inside the draw callback.</summary>
    private static void DrawCells(JsonElement request)
    {
        RequireDraw("draw_cells");
        var cells = new List<IntVec3>();
        if (request.TryGetProperty("cells", out JsonElement list) && list.ValueKind == JsonValueKind.Array)
        {
            foreach (JsonElement entry in list.EnumerateArray())
            {
                cells.Add((IntVec3)JsonValueBridge.Read(entry, typeof(IntVec3), RefContext.Current));
            }
        }

        GenDraw.DrawFieldEdges(cells, ReadColor(request), null, null, 0);
    }
    private static UnityEngine.Color ReadColor(JsonElement request)
    {
        if (!request.TryGetProperty("color", out JsonElement color) || color.ValueKind != JsonValueKind.Object)
        {
            return UnityEngine.Color.white;
        }

        float Channel(string name)
        {
            return color.TryGetProperty(name, out JsonElement value) && value.ValueKind == JsonValueKind.Number
                ? (float)value.GetDouble()
                : 1f;
        }

        return new UnityEngine.Color(Channel("r"), Channel("g"), Channel("b"), Channel("a"));
    }

    /// <summary>
    /// The alerts the game currently shows, so a mod can see whether its own is up.
    ///
    /// `AlertsReadout.activeAlerts` is not public to the compiler, so it is read through the bridge's
    /// own reflection (the contract check verifies the name against the game's metadata).
    /// </summary>
    private static object AlertsList()
    {
        var alerts = new List<Dictionary<string, object>>();
        AlertsReadout readout = Find.Alerts;
        if (readout == null)
        {
            throw new BridgeException("there is no alert readout (no game is running)");
        }

        if (MemberAccess.Get(readout, "activeAlerts") is not List<Alert> active)
        {
            throw new BridgeException("the game's alert readout did not hand out its active alerts");
        }

        foreach (Alert alert in active)
        {
            alerts.Add(new Dictionary<string, object>
            {
                ["typeName"] = alert.GetType().FullName,
                ["active"] = alert.Active,
                ["label"] = alert.GetLabel(),
            });
        }

        // The readout rebuilds its active list on a frame of its own, so the mod's alert may not be
        // in it yet (or at all, when the readout cached its types before this assembly was scanned):
        // list it directly, with the state Rust pushed.
        if (!active.Any(alert => alert is RustAlert))
        {
            alerts.Add(new Dictionary<string, object>
            {
                ["typeName"] = typeof(RustAlert).FullName,
                ["active"] = RustAlert.IsActive,
                ["label"] = "Rust mod alert",
            });
        }

        return alerts;
    }
    private static void UiMessage(JsonElement request)
    {
        RequireMainThread();
        string kind = (JsonArgs.GetStringOrNull(request, "kind") ?? "neutral").ToLowerInvariant();
        string defName = GameNames.Def("MessageTypeDef", kind, "neutral");
        MessageTypeDef def = DefDatabase<MessageTypeDef>.GetNamedSilentFail(defName)
            ?? throw new BridgeException($"no MessageTypeDef named '{defName}' (declared in names.json); is the Core content loaded?");
        Messages.Message(JsonArgs.GetString(request, "text"), def);
    }
    private static void UiDialog(JsonElement request)
    {
        RequireMainThread();
        string handler = JsonArgs.GetStringOrNull(request, "handler");
        Action action = handler == null
            ? null
            : () => RustHost.Current?.DispatchVoid(
                "ui_action",
                new Dictionary<string, object> { ["handler"] = handler });
        var dialog = new Dialog_MessageBox(
            JsonArgs.GetString(request, "text"),
            JsonArgs.GetStringOrNull(request, "button") ?? "OK",
            action,
            title: JsonArgs.GetStringOrNull(request, "title"));
        Find.WindowStack.Add(dialog);
    }
}
