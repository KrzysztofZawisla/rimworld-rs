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
/// The `building` half of <see cref="ThingOps"/>.
/// </summary>
internal static partial class ThingOps
{

    /// <summary>
    /// What a thing contains as an entity holder — an anomaly platform, a holding spot — through the game's
    /// own `CompEntityHolder`, with the containment strength the game reports. A thing without that comp
    /// is refused with its type name, so a mod learns where containment lives.
    /// </summary>
    public static object Containment(JsonElement request)
    {
        Thing thing = ResolveThing(request);
        CompEntityHolder holder = (thing as ThingWithComps)?.GetComp<CompEntityHolder>();
        if (holder == null)
        {
            throw new BridgeException(
                $"'{thing.LabelShort}' ({thing.GetType().Name}) holds no entities (it has no CompEntityHolder)");
        }

        // A snapshot on purpose: the container is the game's own live list.
        List<string> contained = holder.Container
            ?.OfType<Thing>()
            .Select(entry => entry?.LabelShort)
            .ToList();
        return new Dictionary<string, object>
        {
            ["label"] = thing.LabelShort,
            ["strength"] = holder.ContainmentStrength,
            ["contained"] = contained,
            ["count"] = contained?.Count ?? 0,
        };
    }

    /// <summary>
    /// What a construction still needs: a frame reports `Frame.WorkLeft` and its own `TotalMaterialCost`,
    /// a blueprint the whole job's `WorkTotal` and the same material list. A thing that is neither is
    /// refused with its type name.
    /// </summary>
    public static object ConstructionCost(JsonElement request)
    {
        Thing thing = ResolveThing(request);
        if (thing is Frame frame)
        {
            return new Dictionary<string, object>
            {
                ["label"] = frame.LabelShort,
                ["defName"] = frame.BuildDef?.defName,
                ["workLeft"] = frame.WorkLeft,
                ["cost"] = frame.TotalMaterialCost()?
                    .Select(entry => $"{entry?.thingDef?.defName} x{entry?.count}")
                    .ToList(),
            };
        }

        if (thing is Blueprint_Build blueprint)
        {
            // `WorkTotal` is protected in this build (only a frame's `WorkLeft` is public), so the game's
            // own number is read through the reflection route — the same one `ref_call` uses.
            object workTotal = MemberAccess.Get(blueprint, "WorkTotal");
            return new Dictionary<string, object>
            {
                ["label"] = blueprint.LabelShort,
                // A blueprint has no work done yet, so the game only knows the job's total.
                ["defName"] = blueprint.EntityToBuild()?.defName,
                ["workLeft"] = workTotal is float total ? total : 0f,
                ["cost"] = blueprint.TotalMaterialCost()?
                    .Select(entry => $"{entry?.thingDef?.defName} x{entry?.count}")
                    .ToList(),
            };
        }

        throw new BridgeException($"'{thing.LabelShort}' ({thing.GetType().Name}) is no frame or blueprint to price");
    }

    /// <summary>
    /// Finishes a construction immediately through the game's own calls: a blueprint is turned into a frame
    /// with `Blueprint_Build.MakeSolidThing(ref bool)` (which hands the frame back **unspawned**, so it is
    /// spawned where the blueprint stood — the same two steps the game's own construction job performs),
    /// and the frame is then finished with `Frame.CompleteConstruction(Pawn)`, the one method every
    /// finished construction goes through. A thing that is neither is refused with its type name, and a
    /// frame that is not on a map is refused with that fact instead of half-finishing.
    /// </summary>
    public static object CompleteConstruction(JsonElement request)
    {
        Thing thing = ResolveThing(request);
        Frame frame = thing as Frame;
        if (frame == null && thing is Blueprint_Build blueprint)
        {
            Map blueprintMap = blueprint.Map;
            IntVec3 blueprintCell = blueprint.Position;
            Rot4 rotation = blueprint.Rotation;
            // `MakeSolidThing` is protected in this build (the game's own construction job is inside the
            // same assembly), so the bridge reaches it through the reflection route, by-ref argument and all.
            bool shouldSelect = false;
            Thing solid = MemberAccess.Call(blueprint, "MakeSolidThing", new object[] { shouldSelect }) as Thing;
            if (solid == null)
            {
                throw new BridgeException($"the game made no frame out of '{blueprint.LabelShort}'");
            }

            if (blueprintMap == null)
            {
                throw new BridgeException(
                    $"'{blueprint.LabelShort}' is not on a map, so the frame it makes has nowhere to go");
            }

            GenSpawn.Spawn(solid, blueprintCell, blueprintMap, rotation, WipeMode.Vanish);
            frame = solid as Frame;
        }

        if (frame == null)
        {
            throw new BridgeException($"'{thing.LabelShort}' ({thing.GetType().Name}) is no frame to finish");
        }

        if (!frame.Spawned)
        {
            throw new BridgeException(
                $"the frame '{frame.LabelShort}' is not on a map (the game hands frames out unspawned), so there is nothing to finish yet");
        }

        // Read where the frame really is *before* finishing it: completion takes it off the map, and the
        // answer has to describe the frame that was finished.
        string label = frame.LabelShort;
        ThingDef buildDef = frame.BuildDef;
        string defName = buildDef?.defName;
        Map map = frame.Map;
        IntVec3 cell = frame.Position;
        // An optional handle travels as a number (or null when there is none): the same route `ref` takes.
        int workerHandle = JsonArgs.GetInt(request, "workerRef", -1);
        Pawn worker = null;
        if (workerHandle >= 0)
        {
            worker = RefContext.Current.Get(workerHandle) as Pawn
                ?? throw new BridgeException($"handle {workerHandle} in 'workerRef' is no pawn");
        }
        frame.CompleteConstruction(worker);
        // Whatever the game built at that cell is what the frame turned into, so the answer can hand it
        // back: a mod that finishes a construction usually wants to keep talking about the building.
        Thing finished = map?.thingGrid?.ThingsListAt(cell)?
            .FirstOrDefault(candidate => candidate?.def == buildDef);
        return new Dictionary<string, object>
        {
            ["label"] = label,
            ["defName"] = defName,
            ["mapId"] = map?.uniqueID ?? -1,
            ["x"] = cell.x,
            ["z"] = cell.z,
            ["worker"] = worker,
            ["finished"] = finished,
        };
    }

    /// <summary>
    /// Sets what a bed is for through the game's own setters — medical, for prisoners, and the owner type the
    /// game uses. Each is optional, so a mod changes only what it names, and the answer says what the bed is
    /// now, including how many owners the game reports.
    /// </summary>
    public static object SetBedSettings(JsonElement request)
    {
        Thing thing = ResolveThing(request);
        if (!(thing is Building_Bed bed))
        {
            throw new BridgeException($"'{thing.LabelShort}' ({thing.GetType().Name}) is no bed");
        }

        if (request.TryGetProperty("medical", out JsonElement medical))
        {
            bed.Medical = medical.GetBoolean();
        }

        if (request.TryGetProperty("forPrisoners", out JsonElement prisoners))
        {
            bed.ForPrisoners = prisoners.GetBoolean();
        }

        string ownerType = JsonArgs.GetStringOrNull(request, "ownerType");
        if (ownerType != null)
        {
            if (!Enum.TryParse(ownerType, ignoreCase: true, out BedOwnerType parsed))
            {
                // Name the values the game really has: a caller learns them from the game's own enum, not
                // from a guess — the same rule every other refusal here follows.
                string known = string.Join(", ", Enum.GetNames(typeof(BedOwnerType)));
                throw new BridgeException($"there is no bed owner type '{ownerType}' (the game has: {known})");
            }

            bed.ForOwnerType = parsed;
        }

        return new Dictionary<string, object>
        {
            ["label"] = thing.LabelShort,
            ["medical"] = bed.Medical,
            ["forPrisoners"] = bed.ForPrisoners,
            ["ownerType"] = bed.ForOwnerType.ToString(),
            ["owners"] = bed.OwnersForReading?.Count ?? 0,
        };
    }

    /// <summary>
    /// Registers one ingredient on a thing that carries a `CompIngredients` (a meal) and answers with what
    /// the game derives from it, so a registration the game refused is visible. A thing without that comp
    /// is refused with its type name, and a def the game does not have is refused by name.
    /// </summary>
    public static object AddIngredient(JsonElement request)
    {
        Thing thing = ResolveThing(request);
        CompIngredients ingredients = (thing as ThingWithComps)?.GetComp<CompIngredients>();
        if (ingredients == null)
        {
            throw new BridgeException(
                $"'{thing.LabelShort}' ({thing.GetType().Name}) carries no ingredients (it has no CompIngredients)");
        }

        string defName = JsonArgs.GetString(request, "ingredient");
        ThingDef def = DefDatabase<ThingDef>.GetNamedSilentFail(defName);
        if (def == null)
        {
            throw new BridgeException($"there is no thing def '{defName}'");
        }

        ingredients.RegisterIngredient(def);
        string text = ingredients.GetIngredientsString(true, out bool mergeCompatible);
        return new Dictionary<string, object>
        {
            ["label"] = thing.LabelShort,
            ["ingredient"] = def.defName,
            ["count"] = ingredients.ingredients?.Count ?? 0,
            ["text"] = text,
            ["mergeCompatible"] = mergeCompatible,
        };
    }
}
