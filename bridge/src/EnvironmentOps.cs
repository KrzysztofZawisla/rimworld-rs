using System;
using System.Collections.Generic;
using System.Linq;
using System.Reflection;
using System.Text.Json;
using RimWorld;
using RimWorld.Planet;
using Verse;

namespace RustBridge;

/// <summary>
/// The write side of the map's environment: room temperature, heat, fire, filth and power (which
/// `map_cell_info` reads).
///
/// Every operation answers with what the game did with it — a fire the terrain refused, a room that
/// took the temperature, a switch that moved — so a mod never has to guess what happened.
/// </summary>
internal static class EnvironmentOps
{
    public static object SetRoomTemperature(JsonElement request)
    {
        Map map = ResolveMap(request);
        IntVec3 cell = ResolveCell(request);
        RequireInBounds(map, cell);
        float celsius = (float)JsonArgs.GetDouble(request, "celsius", 21);

        Room room = GridsUtility.GetRoom(cell, map);
        if (room == null)
        {
            throw new BridgeException(
                $"no room at {cell.x},{cell.z}: the cell is outdoors or unroofed, so there is no room temperature to set (push heat at the cell instead)");
        }

        float before = room.Temperature;
        room.Temperature = celsius;
        return new Dictionary<string, object>
        {
            ["roomId"] = room.ID,
            ["before"] = before,
            ["after"] = room.Temperature,
        };
    }

    public static object PushHeat(JsonElement request)
    {
        Map map = ResolveMap(request);
        IntVec3 cell = ResolveCell(request);
        RequireInBounds(map, cell);
        float energy = (float)JsonArgs.GetDouble(request, "energy", 100);

        // The game keeps pushed heat in the cell's *room* when it has one — the cell's own grid is not
        // what changed, so it answers false — and that is the whole difference between an indoor and an
        // outdoor cell. The room is read before and after, so `accepted` says whether the energy landed
        // anywhere and the numbers say where it landed.
        Room room = GridsUtility.GetRoom(cell, map);
        float roomBefore = room?.Temperature ?? 0f;
        bool accepted = GenTemperature.PushHeat(cell, map, energy);
        float roomAfter = room?.Temperature ?? 0f;
        return new Dictionary<string, object>
        {
            ["accepted"] = accepted || (room != null && roomAfter != roomBefore),
            // The game's own boolean, kept because it says which route took the heat.
            ["gridAccepted"] = accepted,
            ["room"] = room != null,
            ["temperature"] = GenTemperature.GetTemperatureForCell(cell, map),
            ["roomTemperature"] = room == null ? null : roomAfter,
        };
    }

    public static object StartFire(JsonElement request)
    {
        Map map = ResolveMap(request);
        IntVec3 cell = ResolveCell(request);
        RequireInBounds(map, cell);
        float size = (float)JsonArgs.GetDouble(request, "size", 1);

        TerrainDef terrain = GridsUtility.GetTerrain(cell, map);
        if (!FireUtility.TerrainFlammableNow(cell, map))
        {
            return new Dictionary<string, object>
            {
                ["started"] = false,
                ["reason"] = $"'{terrain?.defName}' at {cell.x},{cell.z} does not burn",
                ["x"] = cell.x,
                ["z"] = cell.z,
            };
        }

        if (FireUtility.ContainsStaticFire(cell, map))
        {
            return new Dictionary<string, object>
            {
                ["started"] = false,
                ["reason"] = $"a fire is already burning at {cell.x},{cell.z}",
                ["x"] = cell.x,
                ["z"] = cell.z,
            };
        }

        // The game itself lights a fire by spawning a Fire thing; the size stays inside the game's own
        // bounds so a mod cannot ask for a fire the fire code cannot simulate.
        var fire = (Fire)ThingMaker.MakeThing(ThingDefOf.Fire);
        fire.fireSize = Math.Min(Math.Max(size, Fire.MinFireSize), Fire.MaxFireSize);
        GenSpawn.Spawn(fire, cell, map);
        return new Dictionary<string, object>
        {
            ["started"] = true,
            ["size"] = fire.fireSize,
            ["firesAtCell"] = FireUtility.NumFiresAt(cell, map),
            ["handle"] = fire,
            ["x"] = cell.x,
            ["z"] = cell.z,
        };
    }

    public static object Fires(JsonElement request)
    {
        Map map = ResolveMap(request);
        var fires = new List<Dictionary<string, object>>();
        List<Thing> burning = map.listerThings?.ThingsOfDef(ThingDefOf.Fire);
        if (burning != null)
        {
            foreach (Thing thing in burning)
            {
                if (thing is not Fire fire || fire.Destroyed)
                {
                    continue;
                }

                IntVec3 cell = fire.Position;
                fires.Add(new Dictionary<string, object>
                {
                    ["handle"] = fire,
                    ["x"] = cell.x,
                    ["z"] = cell.z,
                    ["size"] = fire.fireSize,
                });
            }
        }

        return fires;
    }

    public static object MakeFilth(JsonElement request)
    {
        Map map = ResolveMap(request);
        IntVec3 cell = ResolveCell(request);
        RequireInBounds(map, cell);
        string defName = JsonArgs.GetString(request, "defName");
        ThingDef def = DefDatabase<ThingDef>.GetNamedSilentFail(defName)
            ?? throw new BridgeException($"unknown ThingDef '{defName}'");
        if (!def.IsFilth)
        {
            throw new BridgeException(
                $"'{defName}' is not filth: its ThingDef has no <filth> node (the game's own are Filth_Dirt, Filth_Blood, Filth_Trash_ ...)");
        }

        int count = Math.Max(1, JsonArgs.GetInt(request, "count", 1));
        bool made = FilthMaker.TryMakeFilth(cell, map, def, count, FilthSourceFlags.None, shouldPropagate: false);
        return new Dictionary<string, object>
        {
            ["made"] = made,
            ["thickness"] = FilthThicknessAt(map, cell, defName),
        };
    }

    private static int FilthThicknessAt(Map map, IntVec3 cell, string defName)
    {
        int total = 0;
        List<Thing> at = map.thingGrid?.ThingsListAt(cell);
        if (at != null)
        {
            foreach (Thing thing in at)
            {
                if (thing is Filth filth && filth.def?.defName == defName)
                {
                    total += filth.thickness;
                }
            }
        }

        return total;
    }

    public static object Filth(JsonElement request)
    {
        Map map = ResolveMap(request);
        IntVec3 center = ResolveCell(request);
        RequireInBounds(map, center);
        float radius = (float)JsonArgs.GetDouble(request, "radius", 0);
        string defFilter = JsonArgs.GetStringOrNull(request, "defName");
        int limit = Math.Min(JsonArgs.GetInt(request, "limit", 64), 512);

        var found = new List<Dictionary<string, object>>();
        foreach (IntVec3 cell in GenRadial.RadialCellsAround(center, Math.Max(radius, 0.5f), useCenter: true))
        {
            if (!cell.InBounds(map))
            {
                continue;
            }

            List<Thing> at = map.thingGrid?.ThingsListAt(cell);
            if (at == null)
            {
                continue;
            }

            foreach (Thing thing in at)
            {
                if (thing is not Filth filth || (defFilter != null && filth.def?.defName != defFilter))
                {
                    continue;
                }

                found.Add(new Dictionary<string, object>
                {
                    ["handle"] = filth,
                    ["defName"] = filth.def?.defName,
                    ["label"] = filth.Label,
                    ["thickness"] = filth.thickness,
                    ["x"] = cell.x,
                    ["z"] = cell.z,
                });
                if (found.Count >= limit)
                {
                    return found;
                }
            }
        }

        return found;
    }

    public static object Power(JsonElement request)
    {
        Map map = ResolveMap(request);
        var nets = new List<Dictionary<string, object>>();
        float stored = 0;
        float gain = 0;
        bool hasPower = false;

        List<PowerNet> all = map.powerNetManager?.AllNetsListForReading;
        if (all != null)
        {
            foreach (PowerNet net in all)
            {
                if (net == null || net.Map?.uniqueID != map.uniqueID)
                {
                    continue;
                }

                float netStored = net.CurrentStoredEnergy();
                float netGain = net.CurrentEnergyGainRate();
                bool active = net.HasActivePowerSource;
                stored += netStored;
                gain += netGain;
                hasPower |= active;
                nets.Add(new Dictionary<string, object>
                {
                    ["stored"] = netStored,
                    ["gain"] = netGain,
                    ["hasActivePowerSource"] = active,
                    ["traders"] = net.powerComps?.Count ?? 0,
                    ["batteries"] = net.batteryComps?.Count ?? 0,
                });
            }
        }

        return new Dictionary<string, object>
        {
            ["nets"] = nets,
            ["storedTotal"] = stored,
            ["gainTotal"] = gain,
            ["hasPower"] = hasPower,
        };
    }

    public static object ThingPower(JsonElement request)
    {
        Thing thing = ThingOps.ResolveThing(request);
        List<ThingComp> comps = (thing as ThingWithComps)?.AllComps;
        var trader = comps?.OfType<CompPowerTrader>().FirstOrDefault();
        var battery = comps?.OfType<CompPowerBattery>().FirstOrDefault();
        var flickable = comps?.OfType<CompFlickable>().FirstOrDefault();
        if (trader == null && battery == null && flickable == null)
        {
            throw new BridgeException(
                $"'{thing.LabelShort}' has no power component (CompPowerTrader, CompPowerBattery or CompFlickable)");
        }

        return new Dictionary<string, object>
        {
            ["label"] = thing.LabelShort,
            ["powerOn"] = trader?.PowerOn,
            ["powerOutput"] = trader?.PowerOutput,
            ["storedEnergy"] = battery?.StoredEnergy,
            ["storedEnergyPct"] = battery?.StoredEnergyPct,
            ["storedEnergyCanAccept"] = battery?.AmountCanAccept,
            ["flickable"] = flickable?.SwitchIsOn,
        };
    }

    public static object SetFlickable(JsonElement request)
    {
        Thing thing = ThingOps.ResolveThing(request);
        bool on = JsonArgs.GetBool(request, "on");
        var flickable = (thing as ThingWithComps)?.AllComps?.OfType<CompFlickable>().FirstOrDefault();
        if (flickable == null)
        {
            throw new BridgeException(
                $"'{thing.LabelShort}' has no switch: a flickable building needs the game's CompFlickable (lamps, benches, turrets, ...)");
        }

        bool before = flickable.SwitchIsOn;
        if (before != on)
        {
            flickable.SwitchIsOn = on;
        }

        return new Dictionary<string, object>
        {
            ["label"] = thing.LabelShort,
            ["before"] = before,
            ["after"] = flickable.SwitchIsOn,
        };
    }

    /// <summary>Moves one cell's air temperature toward a target with the game's own controller.</summary>
    public static object SetCellTemperature(JsonElement request)
    {
        Map map = ResolveMap(request);
        IntVec3 cell = ResolveCell(request);
        RequireInBounds(map, cell);
        float target = (float)JsonArgs.GetDouble(request, "celsius", 21);
        float energyLimit = (float)JsonArgs.GetDouble(request, "energyLimit", 1000);

        // One call, the game's own semantics: it changes the cell's temperature by what the energy
        // allows. The temperature grid applies and equalizes on its own tick, so the read-back right
        // here can still be the old value — `controller` is what the game applied, `after` what the
        // grid reports now, and a mod that needs the settled value reads the cell again later.
        float before = GenTemperature.GetTemperatureForCell(cell, map);
        Room room = GridsUtility.GetRoom(cell, map);
        float roomBefore = room?.Temperature ?? 0f;
        float controller = GenTemperature.ControlTemperatureTempChange(cell, map, energyLimit, target);
        float after = GenTemperature.GetTemperatureForCell(cell, map);
        return new Dictionary<string, object>
        {
            ["target"] = target,
            ["before"] = before,
            ["after"] = after,
            ["controller"] = controller,
            ["steps"] = 1,
            ["reached"] = Math.Abs(after - target) <= 0.5f,
            // A cell inside a room is warmed through the room (`Room.Temperature`), which is what the
            // game's controller moves there; the cell's own grid only moves outdoors.
            ["room"] = room != null,
            ["roomTemperature"] = room == null ? null : room.Temperature,
            ["roomBefore"] = room == null ? null : roomBefore,
        };
    }

    public static object SetRoof(JsonElement request)
    {
        Map map = ResolveMap(request);
        IntVec3 cell = ResolveCell(request);
        RequireInBounds(map, cell);
        string roofName = JsonArgs.GetStringOrNull(request, "roofDefName");

        RoofDef def = null;
        if (roofName != null)
        {
            def = DefDatabase<RoofDef>.GetNamedSilentFail(roofName)
                ?? throw new BridgeException(
                    $"unknown RoofDef '{roofName}' (the game's own are RoofConstructed, RoofRockThin, RoofRockThick, VoidmetalRoof)");
        }

        map.roofGrid.SetRoof(cell, def);
        return new Dictionary<string, object>
        {
            ["roofed"] = map.roofGrid.Roofed(cell),
            ["roofDefName"] = map.roofGrid.RoofAt(cell)?.defName,
            ["x"] = cell.x,
            ["z"] = cell.z,
        };
    }

    public static object Gas(JsonElement request)
    {
        Map map = ResolveMap(request);
        IntVec3 cell = ResolveCell(request);
        RequireInBounds(map, cell);
        return GasAt(map, cell);
    }

    public static object AddGas(JsonElement request)
    {
        Map map = ResolveMap(request);
        IntVec3 cell = ResolveCell(request);
        RequireInBounds(map, cell);
        GasType type = ResolveGasType(request);
        int amount = JsonArgs.GetInt(request, "amount", 100);

        GasGrid grid = map.gasGrid;
        if (grid == null)
        {
            throw new BridgeException($"map {map.uniqueID} has no gas grid");
        }

        grid.AddGas(cell, type, amount, canOverflow: true);
        return GasAt(map, cell);
    }

    private static Dictionary<string, object> GasAt(Map map, IntVec3 cell)
    {
        GasGrid grid = map.gasGrid;
        if (grid == null)
        {
            throw new BridgeException($"map {map.uniqueID} has no gas grid");
        }

        return new Dictionary<string, object>
        {
            ["blindSmoke"] = grid.DensityAt(cell, GasType.BlindSmoke),
            ["toxGas"] = grid.DensityAt(cell, GasType.ToxGas),
            ["rotStink"] = grid.DensityAt(cell, GasType.RotStink),
            ["deadlifeDust"] = grid.DensityAt(cell, GasType.DeadlifeDust),
            ["any"] = grid.AnyGasAt(cell),
        };
    }

    private static GasType ResolveGasType(JsonElement request)
    {
        string name = JsonArgs.GetString(request, "gasType");
        if (!Enum.TryParse(name, ignoreCase: true, out GasType type))
        {
            throw new BridgeException(
                $"unknown gas '{name}' (expected one of {string.Join(", ", Enum.GetNames(typeof(GasType)))})");
        }

        return type;
    }

    /// <summary>
    /// Registers a game condition on a map through the game's own `GameConditionMaker` and
    /// `RegisterCondition`, with a duration (or none, which the game makes permanent), and answers with the
    /// conditions the map carries afterwards.
    /// </summary>
    public static object AddCondition(JsonElement request)
    {
        Map map = ResolveMap(request);
        string defName = JsonArgs.GetString(request, "condition");
        GameConditionDef def = DefDatabase<GameConditionDef>.GetNamedSilentFail(defName);
        if (def == null)
        {
            throw new BridgeException($"there is no game condition '{defName}'");
        }

        int duration = JsonArgs.GetInt(request, "duration", 0);
        GameCondition condition = duration > 0
            ? GameConditionMaker.MakeCondition(def, duration)
            : GameConditionMaker.MakeConditionPermanent(def);
        map.gameConditionManager?.RegisterCondition(condition);

        return new Dictionary<string, object>
        {
            ["mapId"] = map.uniqueID,
            ["def"] = def.defName,
            ["duration"] = duration,
            ["active"] = map.gameConditionManager?.ActiveConditions
                ?.Select(active => active?.def?.defName)
                .ToList(),
        };
    }

    /// <summary>
    /// What a colony is worth, through the game's own `WealthWatcher` — a read, so it works while the game
    /// saves. The numbers are the game's own, not a sum the bridge makes up.
    /// </summary>
    public static object Wealth(JsonElement request)
    {
        Map map = ResolveMap(request);
        WealthWatcher watcher = map.wealthWatcher;
        if (watcher == null)
        {
            throw new BridgeException($"map {map.uniqueID} has no wealth watcher");
        }

        return new Dictionary<string, object>
        {
            ["mapId"] = map.uniqueID,
            ["total"] = watcher.WealthTotal,
            ["items"] = watcher.WealthItems,
            ["biome"] = map.Biome?.defName,
        };
    }

    /// <summary>
    /// The components the game attached to a map, by type name — the same shape `world_components` answers
    /// with, so a mod can see what a map carries without touching it. A read, so it is allowed while the
    /// game saves.
    /// </summary>
    public static object MapComponents(JsonElement request)
    {
        Map map = ResolveMap(request);
        return map.components
            ?.Select(component => component?.GetType().Name)
            .ToList();
    }

    /// <summary>
    /// Every room of a map with what a mod asks about it: the role the game gave it, how many cells it
    /// covers, its temperature and whether it is at the mercy of the weather.
    /// </summary>
    public static object Rooms(JsonElement request)
    {
        Map map = ResolveMap(request);
        var rooms = new List<Dictionary<string, object>>();
        // A copy on purpose: a terrain or roof write elsewhere in the same tick has the game rebuild its
        // rooms, and enumerating the live list then throws "Collection was modified".
        foreach (Room room in map.regionGrid.AllRooms.ToList())
        {
            if (room == null)
            {
                continue;
            }

            CellRect extents = room.ExtentsClose;
            // Exact cells are compact enough for proper rooms and let the phone draw the real shape.
            // The huge outdoor "room" is skipped (the overlay filters it too) and a cap keeps a modded
            // megastructure from turning one room into a full-map payload.
            List<IntVec3> roomCells = room.ProperRoom && room.CellCount <= 2000
                ? room.Cells.Take(2000).ToList()
                : null;
            rooms.Add(new Dictionary<string, object>
            {
                ["id"] = room.ID,
                ["x"] = extents.minX,
                ["z"] = extents.minZ,
                ["w"] = extents.Width,
                ["h"] = extents.Height,
                ["roomCells"] = roomCells?.Select(cell => new[] { cell.x, cell.z }).ToList(),
                ["role"] = room.Role?.defName,
                ["roleLabel"] = room.Role?.label,
                ["cells"] = room.CellCount,
                ["temperature"] = room.Temperature,
                ["outdoor"] = room.UsesOutdoorTemperature,
                ["beauty"] = RoomStat(room, RoomStatDefOf.Beauty),
                ["cleanliness"] = RoomStat(room, RoomStatDefOf.Cleanliness),
                ["space"] = RoomStat(room, RoomStatDefOf.Space),
                ["impressiveness"] = RoomStat(room, RoomStatDefOf.Impressiveness),
                ["openRoofCount"] = room.OpenRoofCount,
                ["vacuum"] = room.Vacuum,
                ["prison"] = room.IsPrisonCell,
                ["proper"] = room.ProperRoom,
                ["fogged"] = room.Fogged,
                ["mapId"] = map.uniqueID,
            });
        }

        return rooms;
    }

    private static float RoomStat(Room room, RoomStatDef stat)
    {
        try
        {
            return room.GetStat(stat);
        }
        catch (Exception)
        {
            // A modded room can fail one stat without making the whole room read fail.
            return 0f;
        }
    }

    /// <summary>
    /// Sets the snow depth of one cell through the game's own `SnowGrid` and answers with the depth the
    /// grid reports afterwards.
    /// </summary>
    public static object SetSnow(JsonElement request)
    {
        Map map = ResolveMap(request);
        IntVec3 cell = ResolveCell(request);
        if (!cell.InBounds(map))
        {
            throw new BridgeException($"the cell {cell.x},{cell.z} is outside the map");
        }

        double depth = JsonArgs.GetDouble(request, "depth", -1);
        if (depth < 0)
        {
            throw new BridgeException($"snow depth {depth} cannot be negative");
        }

        map.snowGrid.SetDepth(cell, (float)depth);
        return new Dictionary<string, object>
        {
            ["depth"] = map.snowGrid.GetDepth(cell),
            ["x"] = cell.x,
            ["z"] = cell.z,
        };
    }

    /// <summary>
    /// Pollutes or cleans one cell through the game's own `PollutionGrid` and answers with the map-wide
    /// count before and after (the grid has no per-cell read). A cell the game does not allow to be
    /// polluted is refused instead of silently doing nothing.
    /// </summary>
    public static object SetPollution(JsonElement request)
    {
        Map map = ResolveMap(request);
        IntVec3 cell = ResolveCell(request);
        if (!cell.InBounds(map))
        {
            throw new BridgeException($"the cell {cell.x},{cell.z} is outside the map");
        }

        bool polluted = JsonArgs.GetBool(request, "polluted");
        int before = map.pollutionGrid.TotalPollution;
        map.pollutionGrid.SetPolluted(cell, polluted, silent: true);
        int after = map.pollutionGrid.TotalPollution;
        if (polluted && after == before)
        {
            throw new BridgeException(
                $"the cell {cell.x},{cell.z} cannot be polluted (the game does not allow it there)");
        }

        return new Dictionary<string, object>
        {
            ["polluted"] = polluted,
            ["totalBefore"] = before,
            ["totalAfter"] = after,
        };
    }

    /// <summary>
    /// Generates a pocket map through the game's own `MapGenerator` and a `PocketMapParent` (the class the
    /// game itself uses for its pocket maps) and answers with the map it made. The generator defaults to
    /// the installation's first `MapGeneratorDef` when the caller names none.
    /// </summary>
    public static object GeneratePocketMap(JsonElement request)
    {
        int size = Math.Clamp(JsonArgs.GetInt(request, "size", 75), 25, 250);
        string defName = JsonArgs.GetStringOrNull(request, "generator");
        MapGeneratorDef generator = defName == null
            ? DefDatabase<MapGeneratorDef>.AllDefsListForReading?.FirstOrDefault()
            : DefDatabase<MapGeneratorDef>.GetNamedSilentFail(defName);
        if (generator == null)
        {
            throw new BridgeException(defName == null
                ? "this installation has no map generator at all"
                : $"there is no map generator '{defName}'");
        }

        var parent = new PocketMapParent();
        Find.WorldObjects.Add(parent);
        Map map = MapGenerator.GenerateMap(
            new IntVec3(size, 1, size),
            parent,
            generator,
            extraGenStepDefs: null,
            extraInitBeforeContentGen: null,
            isPocketMap: true,
            stepDebugger: false);
        if (map == null)
        {
            throw new BridgeException($"the game generated no map with '{generator.defName}'");
        }

        return new Dictionary<string, object>
        {
            ["mapId"] = map.uniqueID,
            ["size"] = map.Size.x,
            ["parent"] = parent.GetType().Name,
            ["generator"] = generator.defName,
        };
    }

    /// <summary>
    /// Uncovers or covers the whole map through the game's own `FogGrid` and answers with what the grid
    /// reports for the map centre before and after, so a grid that ignored the call is visible.
    ///
    /// The two methods are called by name on purpose: Roslyn binds `Map.fogGrid` to a `Verse.FogGrid`
    /// that (as it sees the assembly) has no `SetAllFogged`/`ClearAllFog`, while the same assembly offers
    /// both and the running game has them on the very object this field holds. Guessing which member set
    /// wins is exactly what this project refuses to do, so the names are validated by the bridge contract
    /// check instead of by a hand-written call. Per-cell fog has no setter either way.
    /// </summary>
    public static object SetFogAll(JsonElement request)
    {
        Map map = ResolveMap(request);
        bool fogged = JsonArgs.GetBool(request, "fogged");
        var probe = new IntVec3(map.Size.x / 2, 0, map.Size.z / 2);
        bool before = map.fogGrid.IsFogged(probe);
        CallFogGrid(map.fogGrid, fogged ? "SetAllFogged" : "ClearAllFog");
        return new Dictionary<string, object>
        {
            ["before"] = before,
            ["fogged"] = map.fogGrid.IsFogged(probe),
            ["mapId"] = map.uniqueID,
        };
    }

    private static void CallFogGrid(object grid, string method)
    {
        MethodInfo call = grid?.GetType().GetMethod(method, Type.EmptyTypes);
        if (call == null)
        {
            // Name what this object really offers: the metadata and the running type disagree about fog,
            // so the refusal carries the evidence instead of a guess.
            string actual = grid == null
                ? "there is no fog grid"
                : string.Join(", ", grid.GetType()
                    .GetMethods(BindingFlags.Public | BindingFlags.Instance)
                    .Select(info => info.Name)
                    .Where(name => name.Contains("Fog", StringComparison.Ordinal))
                    .Distinct());
            throw new BridgeException(
                $"this game's fog grid ({grid?.GetType().FullName}) has no '{method}', only: {(actual.Length > 0 ? actual : "nothing with 'Fog' in its name")}");
        }

        call.Invoke(grid, null);
    }

    /// <summary>
    /// Moves the camera to a thing (or to a cell of a map) through the game's own `CameraJumper` and
    /// answers with where it went.
    /// </summary>
    public static object JumpTo(JsonElement request)
    {
        if (JsonArgs.GetInt(request, "ref", -1) >= 0)
        {
            Thing thing = ThingOps.ResolveThing(request);
            if (thing.Map == null || !thing.Spawned)
            {
                throw new BridgeException(
                    $"'{thing.LabelShort}' is not on a map, so there is nowhere to jump");
            }

            CameraJumper.TryJump(thing.Position, thing.Map, default);
            return new Dictionary<string, object>
            {
                ["mapId"] = thing.Map.uniqueID,
                ["x"] = thing.Position.x,
                ["z"] = thing.Position.z,
            };
        }

        Map map = ResolveMap(request);
        IntVec3 cell = ResolveCell(request);
        if (!cell.InBounds(map))
        {
            throw new BridgeException($"the cell {cell.x},{cell.z} is outside the map");
        }

        CameraJumper.TryJump(cell, map, default);
        return new Dictionary<string, object>
        {
            ["mapId"] = map.uniqueID,
            ["x"] = cell.x,
            ["z"] = cell.z,
        };
    }

    /// <summary>
    /// Replaces the terrain of one cell through the game's own `TerrainGrid` and answers with what the
    /// grid reports afterwards. Nothing else about the cell is touched (things on it stay).
    /// </summary>
    public static object SetTerrain(JsonElement request)
    {
        Map map = ResolveMap(request);
        IntVec3 cell = ResolveCell(request);
        if (!cell.InBounds(map))
        {
            throw new BridgeException($"the cell {cell.x},{cell.z} is outside the map");
        }

        string defName = JsonArgs.GetString(request, "terrain");
        TerrainDef terrain = DefDatabase<TerrainDef>.GetNamedSilentFail(defName);
        if (terrain == null)
        {
            throw new BridgeException($"there is no terrain '{defName}'");
        }

        map.terrainGrid.SetTerrain(cell, terrain);
        return new Dictionary<string, object>
        {
            ["terrain"] = map.terrainGrid.TerrainAt(cell).defName,
            ["x"] = cell.x,
            ["z"] = cell.z,
        };
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
