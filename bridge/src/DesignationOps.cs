using System;
using System.Collections.Generic;
using System.Linq;
using System.Text.Json;
using RimWorld;
using Verse;

namespace RustBridge;

/// <summary>
/// The layer a mod uses to tell the colony what to do — designations (mine, sow, chop, haul, ...) and
/// storage (stockpiles, shelves, hoppers and their filters).
///
/// Both directions are here: reading what is designated and designating it, reading a storage's
/// settings and changing them. A designation the game has no def for is refused with the reason.
/// </summary>
internal static class DesignationOps
{
    public static object Designations(JsonElement request)
    {
        Map map = ResolveMap(request);
        var list = new List<Dictionary<string, object>>();
        List<Designation> all = map.designationManager?.AllDesignations;
        if (all != null)
        {
            foreach (Designation designation in all)
            {
                if (designation?.def == null)
                {
                    continue;
                }

                LocalTargetInfo target = designation.target;
                var entry = new Dictionary<string, object>
                {
                    ["defName"] = designation.def.defName,
                    ["label"] = designation.def.LabelCap.ToString(),
                    ["targetType"] = designation.def.targetType.ToString(),
                    ["x"] = target.Cell.x,
                    ["z"] = target.Cell.z,
                };
                if (target.HasThing)
                {
                    entry["targetDefName"] = target.Thing?.def?.defName;
                    entry["targetHandle"] = target.Thing;
                }

                list.Add(entry);
            }
        }

        return list;
    }

    public static object DesignateCell(JsonElement request)
    {
        Map map = ResolveMap(request);
        IntVec3 cell = ResolveCell(request);
        RequireInBounds(map, cell);
        DesignationDef def = ResolveDef(request);
        RequireTargetType(def, cell: true);
        if (map.designationManager.DesignationAt(cell, def) != null)
        {
            return Answer(false, "already designated", def, cell);
        }

        map.designationManager.AddDesignation(
            new Designation(new LocalTargetInfo(cell), def, ColorDefOf.PlanGray));
        return Answer(true, null, def, cell);
    }

    public static object UndesignateCell(JsonElement request)
    {
        Map map = ResolveMap(request);
        IntVec3 cell = ResolveCell(request);
        RequireInBounds(map, cell);
        DesignationDef def = ResolveDef(request);
        bool present = map.designationManager.DesignationAt(cell, def) != null;
        map.designationManager.TryRemoveDesignation(cell, def);
        return Answer(present, present ? null : "nothing was designated there", def, cell);
    }

    public static object DesignateThing(JsonElement request)
    {
        Map map = ResolveMap(request);
        Thing thing = ThingOps.ResolveThing(request);
        DesignationDef def = ResolveDef(request);
        RequireTargetType(def, cell: false);
        if (thing.Map?.uniqueID != map.uniqueID)
        {
            throw new BridgeException(
                $"'{thing.LabelShort}' is on map {thing.Map?.uniqueID ?? -1}, not on map {map.uniqueID}");
        }

        if (map.designationManager.DesignationOn(thing, def) != null)
        {
            return new Dictionary<string, object>
            {
                ["changed"] = false,
                ["reason"] = "already designated",
                ["defName"] = def.defName,
                ["label"] = thing.LabelShort,
            };
        }

        map.designationManager.AddDesignation(
            new Designation(new LocalTargetInfo(thing), def, ColorDefOf.PlanGray));
        return new Dictionary<string, object>
        {
            ["changed"] = true,
            ["defName"] = def.defName,
            ["label"] = thing.LabelShort,
        };
    }

    public static object UndesignateThing(JsonElement request)
    {
        Map map = ResolveMap(request);
        Thing thing = ThingOps.ResolveThing(request);
        DesignationDef def = ResolveDef(request);
        bool present = map.designationManager.DesignationOn(thing, def) != null;
        map.designationManager.TryRemoveDesignationOn(thing, def);
        return new Dictionary<string, object>
        {
            ["changed"] = present,
            ["reason"] = present ? null : "nothing was designated on that thing",
            ["defName"] = def.defName,
            ["label"] = thing.LabelShort,
        };
    }

    public static object Zones(JsonElement request)
    {
        Map map = ResolveMap(request);
        var list = new List<Dictionary<string, object>>();
        List<Zone> zones = map.zoneManager?.AllZones;
        if (zones != null)
        {
            foreach (Zone zone in zones)
            {
                if (zone == null)
                {
                    continue;
                }

                list.Add(ZoneEntry(zone));
            }
        }

        return list;
    }

    /// <summary>
    /// How much of a zone is occupied by item stacks. The game has no native "fill" for a stockpile:
    /// a zone is a set of cells and its capacity depends on stack limits, item sizes and the storage
    /// filter. This reports the facts that can be measured — zone cells, occupied cells, item stacks
    /// and the total item count — plus `fill` as occupied cells / zone cells.
    /// </summary>
    public static object ZoneOccupancy(JsonElement request)
    {
        Zone zone = ResolveZone(request);
        Map map = zone.zoneManager?.map;
        if (map == null)
        {
            throw new BridgeException($"the zone '{zone.label}' has no map");
        }

        int cells = 0;
        int occupiedCells = 0;
        int stacks = 0;
        int items = 0;
        foreach (IntVec3 cell in zone.Cells)
        {
            if (map.zoneManager.ZoneAt(cell) != zone)
            {
                continue;
            }

            cells++;
            List<Thing> things = map.thingGrid?.ThingsListAt(cell);
            if (things == null)
            {
                continue;
            }

            bool holdsItems = false;
            foreach (Thing thing in things)
            {
                if (thing == null || thing.Destroyed || thing.def == null)
                {
                    continue;
                }

                if (thing.def.category != ThingCategory.Item)
                {
                    continue;
                }

                holdsItems = true;
                stacks++;
                items += thing.stackCount;
            }

            if (holdsItems)
            {
                occupiedCells++;
            }
        }

        return new Dictionary<string, object>
        {
            ["label"] = zone.label,
            ["cells"] = cells,
            ["occupiedCells"] = occupiedCells,
            ["stacks"] = stacks,
            ["items"] = items,
            ["fill"] = cells > 0 ? (float)occupiedCells / cells : 0f,
        };
    }

    public static object Storage(JsonElement request)
    {
        StorageSettings settings = ResolveSettings(request, out string label);
        return new Dictionary<string, object>
        {
            ["label"] = label,
            ["priority"] = settings.Priority.ToString(),
            ["allowedDefCount"] = settings.filter?.AllowedDefCount ?? 0,
            ["allowedDefs"] = settings.filter?.AllowedThingDefs?.Take(64).Select(def => def.defName).ToList()
                ?? new List<string>(),
        };
    }

    public static object SetPriority(JsonElement request)
    {
        ResolveSettings(request, out string label, out IStoreSettingsParent parent, out StorageSettings settings);
        string wanted = JsonArgs.GetString(request, "priority");
        if (!Enum.TryParse(wanted, ignoreCase: true, out StoragePriority priority))
        {
            throw new BridgeException(
                $"unknown storage priority '{wanted}' (expected one of {string.Join(", ", Enum.GetNames(typeof(StoragePriority)))})");
        }

        StoragePriority before = settings.Priority;
        settings.Priority = priority;
        parent.Notify_SettingsChanged();
        return new Dictionary<string, object>
        {
            ["label"] = label,
            ["before"] = before.ToString(),
            ["after"] = settings.Priority.ToString(),
        };
    }

    public static object SetAllow(JsonElement request)
    {
        ResolveSettings(request, out string label, out IStoreSettingsParent parent, out StorageSettings settings);
        string defName = JsonArgs.GetString(request, "defName");
        ThingDef def = DefDatabase<ThingDef>.GetNamedSilentFail(defName)
            ?? throw new BridgeException($"unknown ThingDef '{defName}'");
        bool allow = JsonArgs.GetBool(request, "allow");
        if (settings.filter == null)
        {
            throw new BridgeException($"the storage of '{label}' has no filter to change");
        }

        bool before = settings.filter.Allows(def);
        settings.filter.SetAllow(def, allow);
        parent.Notify_SettingsChanged();
        return new Dictionary<string, object>
        {
            ["label"] = label,
            ["defName"] = def.defName,
            ["before"] = before,
            ["after"] = settings.filter.Allows(def),
            ["allowedDefCount"] = settings.filter.AllowedDefCount,
        };
    }

    private static Dictionary<string, object> Answer(bool changed, string reason, DesignationDef def, IntVec3 cell)
    {
        var answer = new Dictionary<string, object>
        {
            ["changed"] = changed,
            ["defName"] = def.defName,
            ["x"] = cell.x,
            ["z"] = cell.z,
        };
        if (reason != null)
        {
            answer["reason"] = reason;
        }

        return answer;
    }

    public static object CreateStockpile(JsonElement request)
    {
        Map map = ResolveMap(request);
        IntVec3 cell = ResolveCell(request);
        RequireInBounds(map, cell);
        string presetName = JsonArgs.GetStringOrNull(request, "preset") ?? "DefaultStockpile";
        if (!Enum.TryParse(presetName, ignoreCase: true, out StorageSettingsPreset preset))
        {
            throw new BridgeException(
                $"unknown StorageSettingsPreset '{presetName}' (expected one of {string.Join(", ", Enum.GetNames(typeof(StorageSettingsPreset)))})");
        }

        Zone existing = map.zoneManager.ZoneAt(cell);
        if (existing != null)
        {
            throw new BridgeException(
                $"the cell {cell.x},{cell.z} is already in the zone '{existing.label}'");
        }

        RequireZoneable(map, cell);

        var zone = new Zone_Stockpile(preset, map.zoneManager);
        map.zoneManager.RegisterZone(zone);
        zone.AddCell(cell);
        return ZoneEntry(zone);
    }

    /// <summary>
    /// Sets which plant a growing zone sows, through the game's own `IPlantToGrowSettable`, and answers
    /// with what the zone reports afterwards. A zone that does not implement it is refused by name.
    /// </summary>
    public static object SetZonePlant(JsonElement request)
    {
        // The zone carries its own map, so the request needs nothing but the handle and the plant.
        Zone zone = ResolveZone(request);
        if (!(zone is IPlantToGrowSettable settable))
        {
            throw new BridgeException(
                $"the zone '{zone.label}' ({zone.GetType().Name}) does not take a plant (it does not implement RimWorld.IPlantToGrowSettable)");
        }

        string defName = JsonArgs.GetString(request, "plant");
        ThingDef plant = DefDatabase<ThingDef>.GetNamedSilentFail(defName);
        if (plant == null)
        {
            throw new BridgeException($"there is no plant '{defName}'");
        }

        settable.SetPlantDefToGrow(plant);
        return new Dictionary<string, object>
        {
            ["zone"] = zone.label,
            ["plant"] = settable.GetPlantDefToGrow()?.defName,
        };
    }

    /// <summary>
    /// Creates a growing zone on one cell through the game's own `Zone_Growing`, registered the way the
    /// game registers zones, and answers with the zone. Which plant it grows is the game's own business:
    /// this build exposes no public setter for it, so this op does not pretend to set one.
    /// </summary>
    public static object CreateGrowingZone(JsonElement request)
    {
        Map map = ResolveMap(request);
        IntVec3 cell = ResolveCell(request);
        RequireInBounds(map, cell);
        Zone existing = map.zoneManager.ZoneAt(cell);
        if (existing != null)
        {
            throw new BridgeException(
                $"the cell {cell.x},{cell.z} is already in the zone '{existing.label}'");
        }

        RequireZoneable(map, cell);
        var zone = new Zone_Growing(map.zoneManager);
        map.zoneManager.RegisterZone(zone);
        zone.AddCell(cell);
        return ZoneEntry(zone);
    }

    /// <summary>
    /// Creates a named area (the kind pawns and animals are restricted to) through the game's own
    /// `Area_Allowed` and answers with it. A label the map already carries is refused.
    /// </summary>
    public static object CreateArea(JsonElement request)
    {
        Map map = ResolveMap(request);
        string label = JsonArgs.GetString(request, "label");
        if (label.Length == 0)
        {
            throw new BridgeException("an area needs a non-empty label");
        }

        Area existing = map.areaManager.AllAreas.FirstOrDefault(candidate =>
            candidate.RenamableLabel == label || candidate.BaseLabel == label);
        if (existing != null)
        {
            throw new BridgeException($"map {map.uniqueID} already has an area '{label}'");
        }

        var area = new Area_Allowed(map.areaManager, label);
        // The ctor does not put the area on the manager's list (the game's own area UI does), and every
        // reader - this bridge included - finds areas through that list.
        map.areaManager.AllAreas.Add(area);
        return AreaEntry(area, map);
    }

    /// <summary>
    /// Every cell a named area covers, so a mod can see what it restricts a pawn to. The game keeps the
    /// cell grid private and exposes no public way to put a cell into an area, so there is no add.
    /// </summary>
    public static object AreaCells(JsonElement request)
    {
        Map map = ResolveMap(request);
        Area area = ResolveArea(map, JsonArgs.GetString(request, "label"));
        return new Dictionary<string, object>
        {
            ["label"] = area.RenamableLabel,
            ["count"] = area.TrueCount,
            ["cells"] = area.ActiveCells.Select(cell => new[] { cell.x, cell.z }).ToList(),
        };
    }

    private static Area ResolveArea(Map map, string label)
    {
        Area area = map.areaManager.AllAreas.FirstOrDefault(candidate =>
            candidate.RenamableLabel == label || candidate.BaseLabel == label);
        if (area == null)
        {
            string known = string.Join(", ", map.areaManager.AllAreas.Select(candidate => candidate.RenamableLabel));
            throw new BridgeException(
                $"map {map.uniqueID} has no area '{label}'{(known.Length > 0 ? $" (it has: {known})" : " (it has no areas)")}");
        }

        return area;
    }

    private static Dictionary<string, object> AreaEntry(Area area, Map map)
    {
        return new Dictionary<string, object>
        {
            ["label"] = area.RenamableLabel,
            ["cells"] = area.TrueCount,
            ["mapId"] = map.uniqueID,
        };
    }

    public static object AddZoneCell(JsonElement request)
    {
        Map map = ResolveMap(request);
        Zone zone = ResolveZone(request);
        IntVec3 cell = ResolveCell(request);
        RequireInBounds(map, cell);
        Zone existing = map.zoneManager.ZoneAt(cell);
        if (existing != null && existing != zone)
        {
            throw new BridgeException(
                $"the cell {cell.x},{cell.z} is already in the zone '{existing.label}'");
        }

        RequireZoneable(map, cell);
        zone.AddCell(cell);
        return ZoneEntry(zone);
    }

    private static void RequireZoneable(Map map, IntVec3 cell)
    {
        AcceptanceReport verdict = Designator_ZoneAdd.IsZoneableCell(cell, map);
        if (verdict.Accepted)
        {
            return;
        }

        foreach (Thing thing in map.thingGrid.ThingsAt(cell))
        {
            if (!thing.def.CanOverlapZones)
            {
                throw new BridgeException(
                    $"the game cannot put a zone cell at {cell.x},{cell.z}: '{thing.def.defName}' cannot overlap a zone");
            }
        }

        throw new BridgeException(
            $"the game cannot put a zone cell at {cell.x},{cell.z}: {verdict.Reason ?? "it is not a zoneable cell"}");
    }

    public static object RemoveZoneCell(JsonElement request)
    {
        Zone zone = ResolveZone(request);
        IntVec3 cell = ResolveCell(request);
        bool had = zone.Cells?.Contains(cell) ?? false;
        zone.RemoveCell(cell);
        var answer = ZoneEntry(zone);
        answer["hadCell"] = had;
        return answer;
    }

    public static object DeleteZone(JsonElement request)
    {
        Zone zone = ResolveZone(request);
        var answer = ZoneEntry(zone);
        zone.Delete();
        answer["deleted"] = true;
        return answer;
    }

    private static Dictionary<string, object> ZoneEntry(Zone zone)
    {
        var entry = new Dictionary<string, object>
        {
            ["handle"] = zone,
            ["label"] = zone.label,
            ["typeName"] = zone.GetType().Name,
            ["cells"] = zone.Cells?.Count ?? 0,
        };
        StorageSettings settings = (zone as IStoreSettingsParent)?.GetStoreSettings();
        if (settings != null)
        {
            entry["priority"] = settings.Priority.ToString();
            entry["allowedDefCount"] = settings.filter?.AllowedDefCount ?? 0;
        }

        return entry;
    }

    private static Zone ResolveZone(JsonElement request)
    {
        int handle = JsonArgs.GetInt(request, "ref", -1);
        object target = RefContext.Current.Get(handle);
        return target switch
        {
            null => throw new BridgeException($"the zone handle {handle} is not valid"),
            Zone zone => zone,
            _ => throw new BridgeException($"handle {handle} is a {target.GetType().Name}, not a zone"),
        };
    }

    private static StorageSettings ResolveSettings(JsonElement request, out string label)
    {
        return ResolveSettings(request, out label, out IStoreSettingsParent _, out StorageSettings _);
    }

    private static StorageSettings ResolveSettings(JsonElement request, out string label,
        out IStoreSettingsParent parent, out StorageSettings settings)
    {
        int handle = JsonArgs.GetInt(request, "ref", -1);
        if (handle < 0)
        {
            throw new BridgeException(
                "the request needs a 'ref' handle to a stockpile, a shelf, a hopper or another storage");
        }

        object target = RefContext.Current.Get(handle);
        if (target is not IStoreSettingsParent owner)
        {
            throw new BridgeException(
                $"a {target?.GetType().Name ?? "handle"} has no storage settings (a stockpile, a shelf and a hopper have them)");
        }

        settings = owner.GetStoreSettings();
        if (settings == null)
        {
            throw new BridgeException($"the {target.GetType().Name} has no store settings to work with");
        }

        parent = owner;
        label = target switch
        {
            Thing thing => thing.LabelShort,
            Zone zone => zone.label,
            _ => target.GetType().Name,
        };
        return settings;
    }

    private static DesignationDef ResolveDef(JsonElement request)
    {
        string defName = JsonArgs.GetString(request, "defName");
        return DefDatabase<DesignationDef>.GetNamedSilentFail(defName)
            ?? throw new BridgeException(
                $"unknown DesignationDef '{defName}' (the game's own are listed by def_names(\"DesignationDef\"))");
    }

    private static void RequireTargetType(DesignationDef def, bool cell)
    {
        TargetType expected = cell ? TargetType.Cell : TargetType.Thing;
        if (def.targetType != expected)
        {
            throw new BridgeException(
                $"'{def.defName}' designates {def.targetType.ToString().ToLowerInvariant()}s, not {(cell ? "cells" : "things")}: use "
                + (def.targetType == TargetType.Cell ? "map_designate_cell" : "thing_designate"));
        }
    }

    private static Map ResolveMap(JsonElement request)
    {
        int mapId = JsonArgs.GetInt(request, "mapId", -1);
        Map map = Find.Maps?.FirstOrDefault(candidate => candidate.uniqueID == mapId);
        if (map == null)
        {
            throw new BridgeException($"map {mapId} does not exist");
        }

        return map;
    }

    private static IntVec3 ResolveCell(JsonElement request)
    {
        return new IntVec3(JsonArgs.GetInt(request, "x", 0), 0, JsonArgs.GetInt(request, "z", 0));
    }

    private static void RequireInBounds(Map map, IntVec3 cell)
    {
        if (!cell.InBounds(map))
        {
            throw new BridgeException(
                $"{cell.x},{cell.z} is outside map {map.uniqueID} ({map.Size.x}x{map.Size.z})");
        }
    }
}
