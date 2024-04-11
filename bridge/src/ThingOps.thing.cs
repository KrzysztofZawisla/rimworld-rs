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
/// The `thing` half of <see cref="ThingOps"/>.
/// </summary>
internal static partial class ThingOps
{
    public static object Components(JsonElement request)
    {
        Thing thing = ResolveThing(request);
        var comps = new List<Dictionary<string, object>>();
        List<ThingComp> all = (thing as ThingWithComps)?.AllComps;
        if (all != null)
        {
            foreach (ThingComp comp in all)
            {
                comps.Add(new Dictionary<string, object>
                {
                    ["typeName"] = comp.GetType().FullName,
                    ["handle"] = comp,
                });
            }
        }

        return comps;
    }
    public static object Comp(JsonElement request)
    {
        Thing thing = ResolveThing(request);
        string wanted = JsonArgs.GetString(request, "typeName");
        List<ThingComp> all = (thing as ThingWithComps)?.AllComps;
        ThingComp comp = all?.FirstOrDefault(candidate =>
            candidate?.GetType().FullName == wanted
            || candidate?.GetType().Name == wanted
            || candidate?.GetType().FullName?.EndsWith("." + wanted, StringComparison.Ordinal) == true);

        if (comp == null)
        {
            string known = string.Join(", ", all?.Select(candidate => candidate?.GetType().Name) ?? Enumerable.Empty<string>());
            throw new BridgeException($"'{thing.LabelShort}' has no comp '{wanted}'{(known.Length > 0 ? $" (it has: {known})" : "")}");
        }

        return comp;
    }

    /// <summary>
    /// Sets a thing's forbidden flag through the game's own `CompForbiddable` and answers with the
    /// value the game kept, so a mod never has to guess whether the flag took.
    /// </summary>
    public static object SetForbidden(JsonElement request)
    {
        Thing thing = ResolveThing(request);
        bool forbidden = JsonArgs.GetBool(request, "forbidden");
        CompForbiddable comp = (thing as ThingWithComps)?.GetComp<CompForbiddable>();
        if (comp == null)
        {
            throw new BridgeException(
                $"'{thing.LabelShort}' has no forbid component (CompForbiddable), so its flag cannot be set");
        }

        comp.Forbidden = forbidden;
        return new Dictionary<string, object> { ["forbidden"] = comp.Forbidden };
    }

    /// <summary>Sets a thing's rotation (`Rot4`, 0..3) and answers with what the game kept.</summary>
    public static object SetRotation(JsonElement request)
    {
        Thing thing = ResolveThing(request);
        int rotation = JsonArgs.GetInt(request, "rotation", -1);
        if (rotation < 0 || rotation > 3)
        {
            throw new BridgeException($"rotation {rotation} is not a Rot4 (the game expects 0..3)");
        }

        thing.Rotation = new Rot4((byte)rotation);
        return new Dictionary<string, object> { ["rotation"] = thing.Rotation.AsInt };
    }

    /// <summary>
    /// Sets a thing's hit points, clamped to the game's own maximum for that thing, and answers with
    /// both numbers so the caller can see whether it was clamped.
    /// </summary>
    public static object SetHitPoints(JsonElement request)
    {
        Thing thing = ResolveThing(request);
        int wanted = JsonArgs.GetInt(request, "hitPoints", -1);
        int clamped = Mathf.Clamp(wanted, 0, thing.MaxHitPoints);
        thing.HitPoints = clamped;
        return new Dictionary<string, object>
        {
            ["hitPoints"] = thing.HitPoints,
            ["maxHitPoints"] = thing.MaxHitPoints,
            ["clamped"] = clamped != wanted,
        };
    }

    /// <summary>
    /// Sets a stack's size through the game's own field, clamped into 1..the def's stack limit, and
    /// answers with what the stack reports afterwards.
    /// </summary>
    public static object SetStackCount(JsonElement request)
    {
        Thing thing = ResolveThing(request);
        int wanted = JsonArgs.GetInt(request, "count", -1);
        int limit = thing.def?.stackLimit ?? 1;
        int count = Math.Clamp(wanted, 1, limit);
        thing.stackCount = count;
        return new Dictionary<string, object>
        {
            ["stackCount"] = thing.stackCount,
            ["stackLimit"] = limit,
            ["clamped"] = count != wanted,
        };
    }

    /// <summary>
    /// Sets a thing's quality through the game's own `CompQuality.SetQuality` and answers with what the
    /// comp reports afterwards. A thing without quality is refused with the reason.
    /// </summary>
    public static object SetQuality(JsonElement request)
    {
        Thing thing = ResolveThing(request);
        CompQuality comp = (thing as ThingWithComps)?.GetComp<CompQuality>();
        if (comp == null)
        {
            throw new BridgeException($"'{thing.LabelShort}' has no quality (it has no CompQuality)");
        }

        string wanted = JsonArgs.GetString(request, "quality");
        if (!Enum.TryParse(wanted, ignoreCase: true, out QualityCategory category))
        {
            throw new BridgeException(
                $"unknown quality '{wanted}' (expected one of {string.Join(", ", Enum.GetNames(typeof(QualityCategory)))})");
        }

        comp.SetQuality(category, null);
        return new Dictionary<string, object> { ["quality"] = comp.Quality.ToString() };
    }

    /// <summary>
    /// Uninstalls a building the way the game's own uninstall order does (`MinifyUtility`) and answers
    /// with the minified thing that came out of it.
    /// </summary>
    public static object Uninstall(JsonElement request)
    {
        Thing thing = ResolveThing(request);
        Thing minified = MinifyUtility.TryMakeMinified(thing);
        if (minified == null)
        {
            throw new BridgeException($"the game cannot uninstall '{thing.LabelShort}' (it is not minifiable)");
        }

        return new Dictionary<string, object>
        {
            ["handle"] = minified,
            ["label"] = minified.LabelShort,
        };
    }

    /// <summary>
    /// Moves a thing to another cell of the map it is on, through the game's own `Position` setter, and
    /// answers with where it ended up. A thing with no map and a cell outside the map are both refused with
    /// the reason, because the game's own bounds check is what decides.
    /// </summary>
    public static object SetPosition(JsonElement request)
    {
        Thing thing = ResolveThing(request);
        Map map = thing.Map;
        if (map == null)
        {
            throw new BridgeException($"'{thing.LabelShort}' is not on a map, so it has nowhere to go");
        }

        IntVec3 cell = new IntVec3(JsonArgs.GetInt(request, "x", 0), 0, JsonArgs.GetInt(request, "z", 0));
        if (!cell.InBounds(map))
        {
            throw new BridgeException($"({cell.x}, {cell.z}) is outside map {map.uniqueID}");
        }

        thing.Position = cell;
        return new Dictionary<string, object>
        {
            ["label"] = thing.LabelShort,
            ["mapId"] = map.uniqueID,
            ["x"] = thing.Position.x,
            ["z"] = thing.Position.z,
        };
    }

    /// <summary>
    /// Sets how full a battery is through the game's own `SetStoredEnergyPct` and answers with what the
    /// battery reports, so a change the game clamped or refused is visible.
    /// </summary>
    public static object SetBattery(JsonElement request)
    {
        Thing thing = ResolveThing(request);
        CompPowerBattery battery = (thing as ThingWithComps)?.GetComp<CompPowerBattery>();
        if (battery == null)
        {
            throw new BridgeException(
                $"'{thing.LabelShort}' ({thing.GetType().Name}) stores no power (it has no CompPowerBattery)");
        }

        double pct = JsonArgs.GetDouble(request, "pct", 0.0);
        battery.SetStoredEnergyPct((float)Math.Max(0.0, Math.Min(1.0, pct)));
        return new Dictionary<string, object>
        {
            ["label"] = thing.LabelShort,
            ["storedEnergyPct"] = battery.StoredEnergyPct,
            ["storedEnergy"] = battery.StoredEnergy,
        };
    }

    /// <summary>
    /// Sets how far a thing has rotted through the game's own `RotProgress` and answers with the state
    /// afterwards, including whether the game calls it rotten now.
    /// </summary>
    public static object SetRot(JsonElement request)
    {
        Thing thing = ResolveThing(request);
        CompRottable rottable = (thing as ThingWithComps)?.GetComp<CompRottable>();
        if (rottable == null)
        {
            throw new BridgeException(
                $"'{thing.LabelShort}' ({thing.GetType().Name}) cannot rot (it has no CompRottable)");
        }

        double progress = JsonArgs.GetDouble(request, "progress", 0.0);
        rottable.RotProgress = (float)Math.Max(0.0, progress);
        return new Dictionary<string, object>
        {
            ["label"] = thing.LabelShort,
            ["rotProgress"] = rottable.RotProgress,
            ["rotProgressPct"] = rottable.RotProgressPct,
            ["rotten"] = rottable.Stage != RotStage.Fresh,
        };
    }

    /// <summary>
    /// Swaps a thing's material through the game's own `SetStuffDirect` and answers with what the thing
    /// reports afterwards. The game recomputes what it recomputes; anything it caches stays cached.
    /// </summary>
    public static object SetStuff(JsonElement request)
    {
        Thing thing = ResolveThing(request);
        string defName = JsonArgs.GetString(request, "stuff");
        ThingDef stuff = defName.Length == 0 ? null : DefDatabase<ThingDef>.GetNamedSilentFail(defName);
        if (defName.Length > 0 && stuff == null)
        {
            throw new BridgeException($"there is no material '{defName}'");
        }

        if (stuff != null && thing.def?.MadeFromStuff != true)
        {
            throw new BridgeException(
                $"'{thing.LabelShort}' is not made from a material, so it cannot take one");
        }

        thing.SetStuffDirect(stuff);
        return new Dictionary<string, object>
        {
            ["stuff"] = thing.Stuff?.defName,
            ["label"] = thing.LabelShort,
        };
    }

    /// <summary>
    /// Deconstructs a thing the way the game's own deconstruct order does (`Destroy` with the game's
    /// deconstruct mode) and answers with what is left of it.
    /// </summary>
    public static object Deconstruct(JsonElement request)
    {
        Thing thing = ResolveThing(request);
        string label = thing.LabelShort;
        thing.Destroy(DestroyMode.Deconstruct);
        return new Dictionary<string, object> { ["label"] = label, ["destroyed"] = thing.Destroyed };
    }

    /// <summary>
    /// Renames a thing through the game's own `Verse.IRenameable` and answers with what the game kept.
    /// A thing that does not implement it is refused with its type name, because the game has no other
    /// place where a player rename lives.
    /// </summary>
    public static object SetLabel(JsonElement request)
    {
        Thing thing = ResolveThing(request);
        if (!(thing is IRenameable renamable))
        {
            throw new BridgeException(
                $"'{thing.LabelShort}' ({thing.GetType().FullName}) cannot be renamed: it does not implement Verse.IRenameable");
        }

        string wanted = JsonArgs.GetString(request, "label");
        renamable.RenamableLabel = wanted.Length == 0 ? null : wanted;
        return new Dictionary<string, object>
        {
            ["label"] = renamable.RenamableLabel,
            ["baseLabel"] = renamable.BaseLabel,
        };
    }

    /// <summary>
    /// Sets the title of a work of art through the game's own `CompArt.Title` and answers with what the
    /// game kept, plus the description part the game derives. A thing without that comp is refused with
    /// its type name.
    /// </summary>
    public static object SetArtTitle(JsonElement request)
    {
        Thing thing = ResolveThing(request);
        CompArt art = (thing as ThingWithComps)?.GetComp<CompArt>();
        if (art == null)
        {
            throw new BridgeException(
                $"'{thing.LabelShort}' ({thing.GetType().Name}) is no work of art (it has no CompArt)");
        }

        string title = JsonArgs.GetString(request, "title");
        art.Title = title.Length == 0 ? null : title;
        return new Dictionary<string, object>
        {
            ["label"] = thing.LabelShort,
            ["title"] = art.Title,
            ["description"] = art.GetDescriptionPart(),
        };
    }
    public static object Quality(JsonElement request)
    {
        Thing thing = ResolveThing(request);
        object comp = (thing as ThingWithComps)?.AllComps?.FirstOrDefault(candidate =>
            candidate != null && candidate.GetType().FullName == "RimWorld.CompQuality");
        if (comp == null)
        {
            return null;
        }

        object quality = comp.GetType().GetProperty("Quality")?.GetValue(comp);
        return quality?.ToString();
    }
    public static object SplitOff(JsonElement request)
    {
        Thing thing = ResolveThing(request);
        int count = JsonArgs.GetInt(request, "count", 1);
        if (thing.stackCount < count)
        {
            throw new BridgeException($"{thing.LabelShort} has {thing.stackCount} in the stack, cannot split off {count}");
        }

        return thing.SplitOff(count);
    }
    internal static Thing ResolveThing(JsonElement request)
    {
        int handle = JsonArgs.GetInt(request, "ref", -1);
        object target = RefContext.Current.Get(handle);
        return target switch
        {
            null => throw new BridgeException($"the thing handle {handle} is not valid"),
            Thing thing => thing,
            _ => throw new BridgeException($"handle {handle} is a {target.GetType().Name}, not a thing"),
        };
    }
}
