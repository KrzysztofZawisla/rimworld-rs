using System;
using System.Collections.Generic;
using System.Linq;
using System.Text.Json;
using RimWorld;
using Verse;
using Verse.AI;
using Verse.AI.Group;

namespace RustBridge;

/// <summary>
/// What a pawn fights with: the equipment it carries, the verbs that equipment gives it, and the two
/// conveniences a mod asks for most — arm this pawn, and send it at that target.
///
/// Reading is the whole picture; the two writes answer with what the game did (the weapon that ended
/// up primary, the job that was queued), so a mod never has to guess.
/// </summary>
internal static partial class CombatOps
{
    public static object Equipment(JsonElement request)
    {
        Pawn pawn = ResolvePawn(request);
        Pawn_EquipmentTracker equipment = pawn.equipment;
        var all = new List<Dictionary<string, object>>();
        if (equipment != null)
        {
            foreach (ThingWithComps thing in equipment.AllEquipmentListForReading)
            {
                all.Add(Entry(thing));
            }
        }

        return new Dictionary<string, object>
        {
            ["primary"] = equipment?.Primary != null ? Entry(equipment.Primary) : null,
            ["all"] = all,
            ["hasAnything"] = equipment?.HasAnything() ?? false,
        };
    }

    public static object Verbs(JsonElement request)
    {
        Pawn pawn = ResolvePawn(request);
        var verbs = new List<Dictionary<string, object>>();
        Collect(pawn.VerbTracker?.AllVerbs, "pawn", verbs);
        Collect(pawn.equipment?.AllEquipmentVerbs, "equipment", verbs);
        return verbs;
    }

    public static object Equip(JsonElement request)
    {
        Pawn pawn = ResolvePawn(request);
        ThingWithComps item = ResolveItem(request, "itemRef");
        if (item.def?.IsWeapon != true)
        {
            throw new BridgeException(
                $"'{item.LabelShort}' is not a weapon (its ThingDef has IsWeapon = false)");
        }

        if (pawn.equipment == null)
        {
            throw new BridgeException($"'{pawn.LabelShort}' cannot carry equipment");
        }

        // The game's own equip path takes the thing out of the world before the pawn holds it.
        if (item.Spawned)
        {
            item.DeSpawn(DestroyMode.Vanish);
        }

        pawn.equipment.AddEquipment(item);
        return new Dictionary<string, object>
        {
            ["pawn"] = pawn.LabelShort,
            ["weapon"] = item.LabelShort,
            ["defName"] = item.def?.defName,
            ["primary"] = pawn.equipment.Primary == item,
        };
    }

    public static object Attack(JsonElement request)
    {
        Pawn pawn = ResolvePawn(request);
        Thing target = ResolveThing(request, "targetRef");
        if (pawn.jobs == null)
        {
            throw new BridgeException($"'{pawn.LabelShort}' cannot take jobs");
        }

        if (target.Map == null || pawn.Map == null || target.Map.uniqueID != pawn.Map.uniqueID)
        {
            throw new BridgeException(
                $"'{target.LabelShort}' is not on the same map as '{pawn.LabelShort}'");
        }

        var job = new Job(JobDefOf.AttackMelee, new LocalTargetInfo(target));
        bool queued = pawn.jobs.TryTakeOrderedJob(job, null, false);
        return new Dictionary<string, object>
        {
            ["attacker"] = pawn.LabelShort,
            ["target"] = target.LabelShort,
            ["job"] = JobDefOf.AttackMelee.defName,
            ["queued"] = queued,
            ["targetHandle"] = target,
        };
    }

    private static void Collect(IEnumerable<Verb> source, string origin, List<Dictionary<string, object>> into)
    {
        if (source == null)
        {
            return;
        }

        foreach (Verb verb in source)
        {
            VerbProperties props = verb?.verbProps;
            into.Add(new Dictionary<string, object>
            {
                ["source"] = origin,
                ["label"] = props?.label,
                ["range"] = props?.range ?? 0,
                ["minRange"] = props?.minRange ?? 0,
                ["ranged"] = props?.Ranged ?? false,
            });
        }
    }

    private static Dictionary<string, object> Entry(ThingWithComps thing)
    {
        return new Dictionary<string, object>
        {
            ["handle"] = thing,
            ["defName"] = thing.def?.defName,
            ["label"] = thing.LabelShort,
            ["quality"] = QualityOf(thing),
            ["isWeapon"] = thing.def?.IsWeapon ?? false,
            ["isRanged"] = thing.def?.IsRangedWeapon ?? false,
        };
    }

    /// <summary>The quality of a thing that has one, read the same way the `thing_quality` op reads it.</summary>
    private static string QualityOf(Thing thing)
    {
        object comp = (thing as ThingWithComps)?.AllComps?.FirstOrDefault(candidate =>
            candidate != null && candidate.GetType().FullName == "RimWorld.CompQuality");
        return comp?.GetType().GetProperty("Quality")?.GetValue(comp)?.ToString();
    }

    /// <summary>
    /// Adds a hediff to a pawn through the game's own health tracker and answers with what the game
    /// created, so a mod can hand out a wound, a disease or an implant and keep the handle. The
    /// optional `part` is a `BodyPartDef` name; a pawn without that part is refused with the reason.
    /// </summary>
    public static object HediffAdd(JsonElement request)
    {
        Pawn pawn = ResolvePawn(request);
        string defName = JsonArgs.GetString(request, "hediff");
        HediffDef def = DefDatabase<HediffDef>.GetNamedSilentFail(defName);
        if (def == null)
        {
            throw new BridgeException($"there is no hediff '{defName}'");
        }

        BodyPartRecord part = null;
        string partName = JsonArgs.GetStringOrNull(request, "part");
        if (partName != null)
        {
            BodyPartDef partDef = DefDatabase<BodyPartDef>.GetNamedSilentFail(partName);
            if (partDef == null)
            {
                throw new BridgeException($"there is no body part '{partName}'");
            }

            part = pawn.health.hediffSet.GetBodyPartRecord(partDef);
            if (part == null)
            {
                throw new BridgeException($"'{pawn.LabelShort}' has no body part '{partName}'");
            }
        }

        Hediff hediff = HediffMaker.MakeHediff(def, pawn, part);
        // `default` is the game's "no damage was dealt" result: this call hands out a hediff, it does
        // not attack anybody, so the result the tracker would report for damage stays untouched.
        pawn.health.AddHediff(hediff, part, null, default);
        return new Dictionary<string, object>
        {
            ["handle"] = hediff,
            ["defName"] = hediff.def.defName,
            ["label"] = hediff.LabelCap.ToString(),
            ["part"] = part?.def.defName,
            ["severity"] = hediff.Severity,
            ["count"] = pawn.health.hediffSet.hediffs.Count,
        };
    }

    /// <summary>
    /// Moves everything a pawn wears into its own inventory through the game's own
    /// `DropAllOrMoveAllToInventory` — the game's own "stow it" — and answers with what it wears afterwards.
    /// The predicate is the game's own "everything".
    /// </summary>
    public static object StowApparel(JsonElement request)
    {
        Pawn pawn = ResolvePawn(request);
        if (pawn.apparel == null)
        {
            throw new BridgeException(
                $"'{pawn.LabelShort}' ({pawn.GetType().Name}) wears nothing at all (it has no Pawn_ApparelTracker)");
        }

        int before = pawn.apparel.WornApparelCount;
        pawn.apparel.DropAllOrMoveAllToInventory((Apparel apparel) => true);
        return new Dictionary<string, object>
        {
            ["pawn"] = pawn.LabelShort,
            ["stowed"] = before - pawn.apparel.WornApparelCount,
            ["worn"] = pawn.apparel.WornApparelCount,
        };
    }

    /// <summary>
    /// Puts a weapon into a pawn's hands through the game's own `AddEquipment`. The weapon is made here from
    /// the def the mod names and spawned at the pawn's feet first, so nothing appears from nowhere; a def the
    /// game does not have is refused by name, and only a weapon is taken.
    /// </summary>
    public static object AddEquipment(JsonElement request)
    {
        Pawn pawn = ResolvePawn(request);
        if (pawn.equipment == null)
        {
            throw new BridgeException(
                $"'{pawn.LabelShort}' ({pawn.GetType().Name}) holds nothing at all (it has no Pawn_EquipmentTracker)");
        }

        string defName = JsonArgs.GetString(request, "equipment");
        ThingDef def = DefDatabase<ThingDef>.GetNamedSilentFail(defName)
            ?? throw new BridgeException($"there is no thing def '{defName}'");
        if (!def.IsWeapon)
        {
            throw new BridgeException($"'{defName}' is no weapon (the game says it is no weapon)");
        }

        Thing made = ThingMaker.MakeThing(def, GenStuff.DefaultStuffFor(def));
        made.stackCount = 1;
        GenSpawn.Spawn(made, pawn.Position, pawn.Map, Rot4.North, WipeMode.Vanish);
        if (!(made is ThingWithComps withComps))
        {
            throw new BridgeException($"'{defName}' is not a ThingWithComps, so it cannot be held");
        }

        pawn.equipment.AddEquipment(withComps);
        return new Dictionary<string, object>
        {
            ["pawn"] = pawn.LabelShort,
            ["added"] = pawn.equipment.Primary == withComps,
            ["label"] = withComps.LabelShort,
            ["primary"] = pawn.equipment.Primary?.LabelShort,
        };
    }

    /// <summary>
    /// Asks the game whether a hediff would put a pawn down — through its own `WouldBeDownedAfterAddingHediff`,
    /// so nothing is applied — and answers with that verdict plus whether the pawn is down right now. A
    /// hediff the game does not have is refused by name.
    /// </summary>
    public static object WouldBeDowned(JsonElement request)
    {
        Pawn pawn = ResolvePawn(request);
        string defName = JsonArgs.GetString(request, "hediff");
        HediffDef def = DefDatabase<HediffDef>.GetNamedSilentFail(defName)
            ?? throw new BridgeException($"there is no hediff '{defName}'");

        float severity = (float)JsonArgs.GetDouble(request, "severity", 1.0);
        return new Dictionary<string, object>
        {
            ["pawn"] = pawn.LabelShort,
            ["hediff"] = def.defName,
            ["wouldDown"] = pawn.health.WouldBeDownedAfterAddingHediff(def, null, severity),
            ["downedNow"] = pawn.Downed,
        };
    }

    /// <summary>
    /// Queues a surgery on a pawn through the game's own `surgeryBills` stack: the recipe is named, the bill
    /// is built here, and the game's own `BillStack` takes it. A recipe the game does not have is refused by
    /// name, and so is a recipe that is not surgery.
    /// </summary>
    public static object AddSurgery(JsonElement request)
    {
        Pawn pawn = ResolvePawn(request);
        BillStack stack = pawn.health?.surgeryBills;
        if (stack == null)
        {
            throw new BridgeException($"'{pawn.LabelShort}' has no surgery stack (its health tracker has none)");
        }

        string recipeName = JsonArgs.GetString(request, "recipe");
        RecipeDef recipe = DefDatabase<RecipeDef>.GetNamedSilentFail(recipeName);
        if (recipe == null)
        {
            throw new BridgeException($"there is no recipe '{recipeName}'");
        }

        if (recipe.workerClass == null || !typeof(Recipe_Surgery).IsAssignableFrom(recipe.workerClass))
        {
            throw new BridgeException(
                $"'{recipeName}' is no surgery (the game works it through {recipe.workerClass?.Name ?? "nothing"})");
        }

        var bill = new Bill_Medical(recipe, null);
        stack.AddBill(bill);
        return new Dictionary<string, object>
        {
            ["pawn"] = pawn.LabelShort,
            ["recipe"] = recipe.defName,
            ["bills"] = stack.Count,
        };
    }

    /// <summary>
    /// Takes every hediff off a pawn through the game's own `RemoveAllHediffs` — a full heal, the game's own
    /// way — and answers with how many were there and how many the game reports afterwards.
    /// </summary>
    public static object ClearHediffs(JsonElement request)
    {
        Pawn pawn = ResolvePawn(request);
        int before = pawn.health.hediffSet.hediffs.Count;
        pawn.health.RemoveAllHediffs();
        return new Dictionary<string, object>
        {
            ["pawn"] = pawn.LabelShort,
            ["removed"] = before - pawn.health.hediffSet.hediffs.Count,
            ["hediffs"] = pawn.health.hediffSet.hediffs.Count,
        };
    }

    /// <summary>
    /// Makes a pawn drop what it is holding, through the game's own `TryDropEquipment`, and answers with
    /// whether the game took it and what the pawn holds now. A pawn with no equipment tracker is refused
    /// with its type name.
    /// </summary>
    public static object DropEquipment(JsonElement request)
    {
        Pawn pawn = ResolvePawn(request);
        if (pawn.equipment == null)
        {
            throw new BridgeException(
                $"'{pawn.LabelShort}' ({pawn.GetType().Name}) holds nothing at all (it has no Pawn_EquipmentTracker)");
        }

        ThingWithComps dropped = null;
        bool took = pawn.equipment.TryDropEquipment(
            pawn.equipment.Primary,
            out dropped,
            pawn.Position,
            JsonArgs.GetBool(request, "forbid"));
        return new Dictionary<string, object>
        {
            ["pawn"] = pawn.LabelShort,
            ["dropped"] = took,
            ["label"] = dropped?.LabelShort,
            ["primary"] = pawn.equipment.Primary?.LabelShort,
        };
    }

    /// <summary>
    /// Makes a pawn drop everything it wears through the game's own `DropAll`, dropping it at the pawn's
    /// feet, and answers with how many pieces went. The predicate is the game's own "everything".
    /// </summary>
    public static object DropApparel(JsonElement request)
    {
        Pawn pawn = ResolvePawn(request);
        if (pawn.apparel == null)
        {
            throw new BridgeException(
                $"'{pawn.LabelShort}' ({pawn.GetType().Name}) wears nothing at all (it has no Pawn_ApparelTracker)");
        }

        int before = pawn.apparel.WornApparelCount;
        bool forbid = JsonArgs.GetBool(request, "forbid");
        pawn.apparel.DropAll(pawn.Position, forbid, dropLocked: false, (Apparel apparel) => true);
        return new Dictionary<string, object>
        {
            ["pawn"] = pawn.LabelShort,
            ["dropped"] = before - pawn.apparel.WornApparelCount,
            ["worn"] = pawn.apparel.WornApparelCount,
            ["forbidden"] = forbid,
        };
    }

    /// <summary>
    /// Restores one body part — or every part, when none is named — through the game's own `RestorePart`,
    /// and answers with how many hediffs the pawn still carries. A part the pawn does not have is refused
    /// by name, so a typo cannot silently heal nothing.
    /// </summary>
    public static object RestorePart(JsonElement request)
    {
        Pawn pawn = ResolvePawn(request);
        string partName = JsonArgs.GetStringOrNull(request, "part");
        BodyPartRecord part = null;
        if (partName != null)
        {
            BodyPartDef def = DefDatabase<BodyPartDef>.GetNamedSilentFail(partName);
            if (def == null)
            {
                throw new BridgeException($"there is no body part '{partName}'");
            }

            part = pawn.health.hediffSet.GetBodyPartRecord(def);
            if (part == null)
            {
                throw new BridgeException($"'{pawn.LabelShort}' has no body part '{partName}'");
            }
        }

        pawn.health.RestorePart(part, null, true);
        return new Dictionary<string, object>
        {
            ["pawn"] = pawn.LabelShort,
            ["part"] = partName,
            ["hediffs"] = pawn.health.hediffSet.hediffs.Count,
        };
    }

    /// <summary>
    /// Sets the severity of every hediff of one def through the game's own setter — the number the game
    /// scales an injury with — and answers with what the first one reports afterwards.
    /// </summary>
    public static object HediffSetSeverity(JsonElement request)
    {
        Pawn pawn = ResolvePawn(request);
        string defName = JsonArgs.GetString(request, "hediff");
        List<Hediff> matching = pawn.health.hediffSet.hediffs
            .Where(candidate => candidate.def.defName == defName)
            .ToList();
        if (matching.Count == 0)
        {
            throw new BridgeException($"'{pawn.LabelShort}' has no hediff '{defName}'");
        }

        float severity = (float)JsonArgs.GetDouble(request, "severity", 0.0);
        foreach (Hediff hediff in matching)
        {
            hediff.Severity = severity;
        }

        return new Dictionary<string, object>
        {
            ["pawn"] = pawn.LabelShort,
            ["def"] = defName,
            ["count"] = matching.Count,
            ["severity"] = matching[0].Severity,
            ["label"] = matching[0].SeverityLabel?.ToString(),
        };
    }

    /// <summary>
    /// Removes every hediff of one def from a pawn through the game's own health tracker and answers
    /// with how many went and how many are left, so nothing disappears silently.
    /// </summary>
    public static object HediffRemove(JsonElement request)
    {
        Pawn pawn = ResolvePawn(request);
        string defName = JsonArgs.GetString(request, "hediff");
        List<Hediff> matching = pawn.health.hediffSet.hediffs
            .Where(candidate => candidate.def.defName == defName)
            .ToList();
        if (matching.Count == 0)
        {
            throw new BridgeException($"'{pawn.LabelShort}' has no hediff '{defName}'");
        }

        foreach (Hediff hediff in matching)
        {
            pawn.health.RemoveHediff(hediff);
        }

        return new Dictionary<string, object>
        {
            ["removed"] = matching.Count,
            ["remaining"] = pawn.health.hediffSet.hediffs.Count,
        };
    }

    /// <summary>
    /// Makes an apparel item and puts it on a pawn the way the game's own apparel tracker does
    /// (`Wear`, which drops whatever it replaces), answering with the handle and whether the pawn
    /// wears it afterwards.
    /// </summary>
    public static object WearApparel(JsonElement request)
    {
        Pawn pawn = ResolvePawn(request);
        string defName = JsonArgs.GetString(request, "apparel");
        ThingDef def = DefDatabase<ThingDef>.GetNamedSilentFail(defName);
        if (def == null)
        {
            throw new BridgeException($"there is no apparel '{defName}'");
        }

        if (!def.IsApparel)
        {
            throw new BridgeException($"'{defName}' is not apparel, so a pawn cannot wear it");
        }

        if (pawn.apparel == null)
        {
            throw new BridgeException($"'{pawn.LabelShort}' has no apparel tracker, so it cannot wear anything");
        }

        // A def made from stuff needs one, or the game assigns a default and logs an error; the game's own
        // default is the honest choice here.
        ThingDef stuff = def.MadeFromStuff ? GenStuff.DefaultStuffFor(def) : null;
        Thing made = ThingMaker.MakeThing(def, stuff);
        Apparel apparel = made as Apparel;
        if (apparel == null)
        {
            throw new BridgeException($"the game made a {made.GetType().Name}, not apparel, for '{defName}'");
        }

        pawn.apparel.Wear(apparel, dropReplacedApparel: true, locked: false);
        return new Dictionary<string, object>
        {
            ["handle"] = apparel,
            ["wearing"] = pawn.apparel.Wearing(apparel),
        };
    }

    /// <summary>Takes apparel off a pawn and answers with whether the pawn still wears it.</summary>
    public static object RemoveApparel(JsonElement request)
    {
        Pawn pawn = ResolvePawn(request);
        Thing worn = ResolveThing(request, "apparelRef");
        if (!(worn is Apparel apparel))
        {
            throw new BridgeException($"'{worn.LabelShort}' is not apparel");
        }

        if (pawn.apparel == null)
        {
            throw new BridgeException($"'{pawn.LabelShort}' has no apparel tracker");
        }

        if (!pawn.apparel.Wearing(apparel))
        {
            throw new BridgeException($"'{pawn.LabelShort}' does not wear '{apparel.LabelShort}'");
        }

        pawn.apparel.Remove(apparel);
        return new Dictionary<string, object> { ["wearing"] = pawn.apparel.Wearing(apparel) };
    }

    /// <summary>
    /// The lords of a map (raids, parties, anything the game groups pawns under) with their job, their
    /// current toil and their pawns, so a mod can see and touch what the game has already gathered.
    /// </summary>
    public static object Lords(JsonElement request)
    {
        int mapId = JsonArgs.GetInt(request, "mapId", -1);
        Map map = Find.Maps?.FirstOrDefault(candidate => candidate.uniqueID == mapId)
            ?? throw new BridgeException($"map {mapId} does not exist");
        LordManager manager = map.lordManager;
        if (manager?.lords == null)
        {
            throw new BridgeException($"map {map.uniqueID} has no lord manager");
        }

        // A copy on purpose: a lord can be removed by the game while a mod reads the list in the same tick.
        return manager.lords.ToList().Select(lord => new Dictionary<string, object>
        {
            ["handle"] = lord,
            ["job"] = lord?.LordJob?.GetType().Name,
            ["toil"] = lord?.CurLordToil?.GetType().Name,
            ["pawns"] = lord?.ownedPawns?.Count ?? 0,
            ["factionDefName"] = lord?.faction?.def?.defName,
        }).ToList();
    }

    /// <summary>
    /// Puts a pawn into a lord through the game's own `CanAddPawn` + `AddPawn` and answers with the lord
    /// afterwards. A lord the game says no to is refused with that fact.
    /// </summary>
    public static object LordAddPawn(JsonElement request)
    {
        Lord lord = ResolveLord(request);
        Pawn pawn = ResolvePawnByMember(request, "pawnRef");
        string job = lord.LordJob?.GetType().Name ?? "jobless";
        // The game's own verdict, and its own failure when it cannot give one: some jobs walk state that a
        // lord built outside the game's flow does not have, and that throws instead of answering.
        bool canAdd;
        try
        {
            canAdd = lord.CanAddPawn(pawn);
        }
        catch (Exception error)
        {
            throw new BridgeException(
                $"the game could not say whether a '{job}' lord takes '{pawn.LabelShort}': {error.GetType().Name}: {error.Message}");
        }

        if (!canAdd)
        {
            throw new BridgeException(
                $"the game does not add '{pawn.LabelShort}' to that {job} lord");
        }

        lord.AddPawn(pawn);
        return LordAnswer(lord);
    }

    /// <summary>
    /// Swaps a lord's job through the game's own `SetJob`, building the job here the same way `lord_create`
    /// does: with the lord's own first pawn when the type takes one, and with no arguments when it does not.
    /// A class the game does not have, or one whose constructor needs something else, is refused by name.
    /// </summary>
    public static object LordSetJob(JsonElement request)
    {
        Lord lord = ResolveLord(request);
        string jobClass = JsonArgs.GetString(request, "jobClass");
        Type jobType = GenTypes.AllTypes.FirstOrDefault(type =>
            type != null
            && typeof(LordJob).IsAssignableFrom(type)
            && (type.Name == jobClass || type.FullName == jobClass));
        if (jobType == null)
        {
            throw new BridgeException($"there is no LordJob called '{jobClass}'");
        }

        Pawn pawn = lord.ownedPawns?.FirstOrDefault();
        LordJob job = pawn != null ? TryBuildJob(jobType, pawn) : null;
        job ??= TryBuildJob(jobType);
        if (job == null)
        {
            throw new BridgeException(
                $"'{jobClass}' takes arguments the bridge cannot build (it has neither a no-argument constructor nor one taking a pawn)");
        }

        lord.SetJob(job, false);
        return LordAnswer(lord);
    }

    /// <summary>
    /// Builds a lord — a group of pawns with one job — and adds it to the map through the game's own
    /// `LordManager.AddLord`. The job class is given by name (`LordJob_DefendPoint` and friends); the
    /// bridge builds it with the pawn when the type takes one, and with no arguments when it does not, so
    /// a job whose constructor needs something else is refused by name instead of half-built.
    /// </summary>
    public static object LordCreate(JsonElement request)
    {
        int mapId = JsonArgs.GetInt(request, "mapId", -1);
        Map map = Find.Maps?.FirstOrDefault(candidate => candidate.uniqueID == mapId)
            ?? throw new BridgeException($"map {mapId} does not exist");
        string jobClass = JsonArgs.GetString(request, "jobClass");
        Type jobType = GenTypes.AllTypes.FirstOrDefault(type =>
            type != null
            && typeof(LordJob).IsAssignableFrom(type)
            && (type.Name == jobClass || type.FullName == jobClass));
        if (jobType == null)
        {
            throw new BridgeException($"there is no LordJob called '{jobClass}'");
        }

        // A lord may be built without a pawn (a job that fills its own ranks), so the member is optional
        // and no handle at all means "no pawn" instead of a refusal. The handle travels as a number,
        // like every other `ref` (a null or negative value is "none").
        int pawnHandle = JsonArgs.GetInt(request, "pawnRef", -1);
        Pawn pawn = pawnHandle < 0 ? null : ResolvePawnByMember(request, "pawnRef");
        LordJob job = null;
        if (pawn != null)
        {
            job = TryBuildJob(jobType, pawn);
        }

        job ??= TryBuildJob(jobType);
        if (job == null)
        {
            throw new BridgeException(
                $"'{jobClass}' takes arguments the bridge cannot build (it has neither a no-argument constructor nor one taking a pawn)");
        }

        Lord lord = new Lord();
        // The pawn comes first: a job's own toil graph walks the lord's pawns while it is built, and one
        // that finds an empty lord throws a null reference deep inside `CreateGraph` (the game itself
        // knows its pawns before it starts the job).
        if (pawn != null)
        {
            lord.AddPawn(pawn);
        }

        try
        {
            lord.SetJob(job, false);
        }
        catch (Exception error)
        {
            // `SetJob` builds the job's own toil graph, and some classes only make sense on a lord the
            // game built itself (a faction, a spot, an escort). The game's message is the refusal.
            throw new BridgeException(
                $"'{jobClass}' could not start on a lord built here: {error.GetType().Name}: {error.Message}");
        }

        map.lordManager?.AddLord(lord);
        return LordAnswer(lord);
    }

    private static LordJob TryBuildJob(Type jobType, params object[] arguments)
    {
        try
        {
            return Activator.CreateInstance(jobType, arguments) as LordJob;
        }
        catch (Exception)
        {
            // A job whose constructor does not take these arguments: the caller decides whether that is a
            // refusal (no other shape fits) or a fallback (the no-argument one does).
            return null;
        }
    }

    /// <summary>
    /// Takes a lord off the map through the game's own `LordManager.RemoveLord` and answers with what is
    /// left. The pawns stay where they are; they simply have no lord afterwards.
    /// </summary>
    public static object LordDismiss(JsonElement request)
    {
        Lord lord = ResolveLord(request);
        var answer = LordAnswer(lord);
        // The manager lives on the map, not on `Find` (verified by the compiler).
        lord.Map?.lordManager?.RemoveLord(lord);
        return answer;
    }

    private static Dictionary<string, object> LordAnswer(Lord lord)
    {
        return new Dictionary<string, object>
        {
            // The lord itself as a handle, so a mod can hand it straight back (`lord_dismiss`,
            // `lord_add_pawn`) instead of digging it out of the manager's list.
            ["handle"] = lord,
            ["job"] = lord?.LordJob?.GetType().Name,
            ["toil"] = lord?.CurLordToil?.GetType().Name,
            ["pawns"] = lord?.ownedPawns?.Count ?? 0,
        };
    }

}
