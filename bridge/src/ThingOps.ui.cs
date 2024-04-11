using System;
using System.Collections.Generic;
using System.Linq;
using System.Text.Json;
using RimWorld;
using UnityEngine;
using Verse;
using Verse.AI;
namespace RustBridge;

/// <summary>
/// The `ui` half of <see cref="ThingOps"/>.
/// </summary>
internal static partial class ThingOps
{
    public static object Selection()
    {
        var selected = new List<object>();
        foreach (object thing in Find.Selector?.SelectedObjects ?? new List<object>())
        {
            selected.Add(thing);
        }

        return selected;
    }
    public static void Select(JsonElement request)
    {
        object target = RefContext.Current.Get(JsonArgs.GetInt(request, "ref", -1))
            ?? throw new BridgeException("the object to select is not a valid handle");
        Selector selector = Find.Selector;
        if (selector == null)
        {
            throw new BridgeException("there is no selector (no game is running)");
        }

        if (JsonArgs.GetBool(request, "add"))
        {
            selector.Select(target);
        }
        else
        {
            selector.ClearSelection();
            selector.Select(target);
        }
    }
    public static void ClearSelection()
    {
        Find.Selector?.ClearSelection();
    }
    public static void FloatMenu(JsonElement request)
    {
        Map map = ResolveMap(request);
        var cell = new IntVec3(JsonArgs.GetInt(request, "x", 0), 0, JsonArgs.GetInt(request, "z", 0));
        if (!cell.InBounds(map))
        {
            throw new BridgeException($"{cell} is outside map {map.uniqueID}");
        }

        if (!request.TryGetProperty("options", out JsonElement options) || options.ValueKind != JsonValueKind.Array)
        {
            throw new BridgeException("float_menu needs an 'options' array of {label, handler}");
        }

        var menu = new List<FloatMenuOption>();
        foreach (JsonElement option in options.EnumerateArray())
        {
            string label = JsonArgs.GetString(option, "label");
            string handler = JsonArgs.GetString(option, "handler");
            menu.Add(new FloatMenuOption(label, () => RustHost.Current?.DispatchVoid(
                "ui_action",
                new Dictionary<string, object> { ["handler"] = handler, ["id"] = "float_menu" })));
        }

        if (menu.Count == 0)
        {
            throw new BridgeException("float_menu needs at least one option");
        }

        Find.WindowStack.Add(new FloatMenu(menu));
    }
}
