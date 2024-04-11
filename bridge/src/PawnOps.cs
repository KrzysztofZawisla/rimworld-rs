using System;
using System.Collections.Generic;
using System.Linq;
using System.Reflection;
using System.Text.Json;
using RimWorld;
using Verse;
using Verse.AI;

namespace RustBridge;

/// <summary>
/// The pawn side of the API: everything a mod wants to know about a pawn, plus the actions it
/// commonly takes. Every member reached here is listed in the bridge contract check, so a renamed
/// member fails `cargo xtask test` instead of the running game.
/// </summary>
internal static class PawnOps
{
    public static object Info(JsonElement request)
    {
        Pawn pawn = Resolve(request);
        var info = new Dictionary<string, object>
        {
            ["label"] = pawn.LabelShort,
            ["defName"] = pawn.def?.defName,
            ["kindDefName"] = pawn.kindDef?.defName,
            ["factionDefName"] = pawn.Faction?.def?.defName,
            ["factionName"] = pawn.Faction?.Name,
            ["gender"] = pawn.gender.ToString(),
            ["ageBiologicalYears"] = pawn.ageTracker?.AgeBiologicalYears ?? 0,
            ["isColonist"] = pawn.IsColonist,
            ["isPrisoner"] = pawn.IsPrisonerOfColony,
            ["isSlave"] = pawn.IsSlaveOfColony,
            ["isAnimal"] = pawn.RaceProps?.Animal ?? false,
            ["isMechanoid"] = pawn.RaceProps?.IsMechanoid ?? false,
            ["dead"] = pawn.Dead,
            ["downed"] = pawn.Downed,
            ["drafted"] = pawn.Drafted,
            ["inMentalState"] = pawn.InMentalState,
            ["mentalStateDefName"] = pawn.MentalStateDef?.defName,
            ["currentJobDefName"] = pawn.CurJobDef?.defName,
            ["healthPercent"] = pawn.health?.summaryHealth?.SummaryHealthPercent ?? 0f,
            ["moodPercent"] = pawn.needs?.mood?.CurLevelPercentage ?? -1f,
            ["position"] = Cell(pawn.Position),
            ["mapId"] = pawn.Map?.uniqueID ?? -1,
            ["spawned"] = pawn.Spawned,
        };

        return info;
    }

    public static object Health(JsonElement request)
    {
        Pawn pawn = Resolve(request);
        var hediffs = new List<Dictionary<string, object>>();
        HediffSet set = pawn.health?.hediffSet;
        if (set?.hediffs != null)
        {
            foreach (Hediff hediff in set.hediffs)
            {
                hediffs.Add(new Dictionary<string, object>
                {
                    ["defName"] = hediff.def?.defName,
                    ["label"] = hediff.LabelCap.ToString(),
                    ["severity"] = hediff.Severity,
                    ["part"] = hediff.Part?.Label,
                    ["bleeding"] = hediff.Bleeding,
                });
            }
        }

        return new Dictionary<string, object>
        {
            ["hediffs"] = hediffs,
            ["bleedRate"] = set?.BleedRateTotal ?? 0f,
            ["summaryPercent"] = pawn.health?.summaryHealth?.SummaryHealthPercent ?? 0f,
            ["downed"] = pawn.Downed,
            ["dead"] = pawn.Dead,
        };
    }

    public static object Needs(JsonElement request)
    {
        Pawn pawn = Resolve(request);
        var needs = new List<Dictionary<string, object>>();
        List<Need> all = pawn.needs?.AllNeeds;
        if (all != null)
        {
            foreach (Need need in all)
            {
                needs.Add(new Dictionary<string, object>
                {
                    ["defName"] = need.def?.defName,
                    ["label"] = need.def?.label,
                    ["level"] = need.CurLevel,
                    ["percentage"] = need.CurLevelPercentage,
                });
            }
        }

        return needs;
    }

    public static object Skills(JsonElement request)
    {
        Pawn pawn = Resolve(request);
        var skills = new List<Dictionary<string, object>>();
        List<SkillRecord> records = pawn.skills?.skills;
        if (records != null)
        {
            foreach (SkillRecord skill in records)
            {
                skills.Add(new Dictionary<string, object>
                {
                    ["defName"] = skill.def?.defName,
                    ["label"] = skill.def?.label,
                    ["level"] = skill.Level,
                    ["passion"] = skill.passion.ToString(),
                });
            }
        }

        return skills;
    }

    public static object Inventory(JsonElement request)
    {
        Pawn pawn = Resolve(request);
        var items = new List<object>();
        ThingOwner owner = pawn.inventory?.innerContainer;
        if (owner != null)
        {
            foreach (Thing thing in owner)
            {
                items.Add(thing);
            }
        }

        return items;
    }

    public static object WorkPriorities(JsonElement request)
    {
        Pawn pawn = Resolve(request);
        var work = new List<Dictionary<string, object>>();
        foreach (WorkTypeDef def in DefDatabase<WorkTypeDef>.AllDefsListForReading)
        {
            int priority = -1;
            try
            {
                priority = pawn.workSettings?.GetPriority(def) ?? -1;
            }
            catch (Exception)
            {
                priority = -1; // pawns without work settings (animals, mechs) report -1
            }

            work.Add(new Dictionary<string, object>
            {
                ["defName"] = def.defName,
                ["label"] = def.label,
                ["priority"] = priority,
                // Always-active work types (Firefighter, Patient, ...) are kept at priority 3 by the
                // game: SetPriority ignores them and GetPriority reports 3 whatever the priority map
                // says (verified in game: a write to Firefighter reads back as 3).
                ["locked"] = def.alwaysStartActive,
            });
        }

        return work;
    }

    public static void SetWorkPriority(JsonElement request)
    {
        Pawn pawn = Resolve(request);
        string defName = JsonArgs.GetString(request, "workType");
        WorkTypeDef def = DefDatabase<WorkTypeDef>.GetNamedSilentFail(defName)
            ?? throw new BridgeException($"unknown WorkTypeDef '{defName}'");
        if (pawn.workSettings == null)
        {
            throw new BridgeException($"{pawn.LabelShort} has no work settings (animals and mechs do not work)");
        }

        if (def.alwaysStartActive)
        {
            throw new BridgeException(
                $"'{defName}' is always active in this game, so it has no assignable priority "
                + "(see WorkSnapshot::locked)");
        }

        int priority = JsonArgs.GetInt(request, "priority", 3);
        pawn.workSettings.EnableAndInitializeIfNotAlreadyInitialized();
        pawn.workSettings.SetPriority(def, priority);
        int afterSetter = pawn.workSettings.GetPriority(def);

        // The priority map behind the setter is private, so it is reached through reflection: a pawn
        // whose work settings the game has not initialized keeps the requested value there.
        int storedBefore = -1;
        int storedAfter = -1;
        bool haveMap = false;
        FieldInfo field = typeof(Pawn_WorkSettings).GetField(
            "priorities",
            BindingFlags.Instance | BindingFlags.NonPublic);
        if (field?.GetValue(pawn.workSettings) is DefMap<WorkTypeDef, int> priorities)
        {
            haveMap = true;
            storedBefore = priorities[def];
            if (afterSetter != priority)
            {
                priorities[def] = priority;
                pawn.workSettings.Notify_UseWorkPrioritiesChanged();
                storedAfter = priorities[def];
            }
        }

        int after = pawn.workSettings.GetPriority(def);
        if (after != priority)
        {
            throw new BridgeException(
                $"{pawn.LabelShort}: '{defName}' kept priority {after} (asked {priority}); "
                + $"setter read back {afterSetter}, the priority map "
                + (haveMap ? $"held {storedBefore} and {storedAfter} after a direct write" : "is not initialized")
                + $", initialized={pawn.workSettings.Initialized}, everWork={pawn.workSettings.EverWork}");
        }
    }

    public static object Relations(JsonElement request)
    {
        Pawn pawn = Resolve(request);
        var relations = new List<Dictionary<string, object>>();
        List<DirectPawnRelation> direct = pawn.relations?.DirectRelations;
        if (direct != null)
        {
            foreach (DirectPawnRelation relation in direct)
            {
                relations.Add(new Dictionary<string, object>
                {
                    ["other"] = relation.otherPawn,
                    ["label"] = relation.otherPawn?.LabelShort,
                    ["kind"] = relation.def?.defName,
                    ["kindLabel"] = relation.def?.label,
                });
            }
        }

        return relations;
    }

    public static object Abilities(JsonElement request)
    {
        Pawn pawn = Resolve(request);
        var abilities = new List<Dictionary<string, object>>();
        List<Ability> all = pawn.abilities?.AllAbilitiesForReading;
        if (all != null)
        {
            foreach (Ability ability in all)
            {
                abilities.Add(new Dictionary<string, object>
                {
                    ["defName"] = ability.def?.defName,
                    ["label"] = ability.def?.label,
                });
            }
        }

        return abilities;
    }

    public static void Damage(JsonElement request)
    {
        Pawn pawn = Resolve(request);
        float amount = (float)JsonArgs.GetDouble(request, "amount", 1);
        string defName = JsonArgs.GetStringOrNull(request, "damageDef");

        DamageDef def = defName == null
            ? DefDatabase<DamageDef>.GetNamedSilentFail(GameNames.Def("DamageDef", "blunt", "blunt"))
                ?? throw new BridgeException("the game has no blunt DamageDef (declared in names.json)")
            : DefDatabase<DamageDef>.GetNamedSilentFail(defName)
                ?? throw new BridgeException($"unknown DamageDef '{defName}'");

        ConstructorInfo constructor = typeof(DamageInfo).GetConstructors()
            .OrderByDescending(candidate => candidate.GetParameters().Length)
            .First();
        ParameterInfo[] parameters = constructor.GetParameters();
        var arguments = new object[parameters.Length];
        for (int index = 0; index < parameters.Length; index++)
        {
            arguments[index] = parameters[index].HasDefaultValue ? parameters[index].DefaultValue : Type.Missing;
        }

        arguments[0] = def;
        arguments[1] = amount;
        object info = constructor.Invoke(arguments);
        pawn.TakeDamage((DamageInfo)info);
    }

    public static void Kill(JsonElement request)
    {
        Pawn pawn = Resolve(request);
        if (pawn.Dead)
        {
            throw new BridgeException($"{pawn.LabelShort} is already dead");
        }

        pawn.Kill(null, null);
    }

    public static void Teleport(JsonElement request)
    {
        Pawn pawn = Resolve(request);
        IntVec3 cell = new IntVec3(
            JsonArgs.GetInt(request, "x", 0),
            JsonArgs.GetInt(request, "y", 0),
            JsonArgs.GetInt(request, "z", 0));

        Map map = pawn.Map;
        if (map != null && !cell.InBounds(map))
        {
            throw new BridgeException($"{cell} is outside map {map.uniqueID}");
        }

        if (pawn.Spawned)
        {
            pawn.Position = cell;
            pawn.Notify_Teleported(true, false);
        }
        else if (map != null)
        {
            GenSpawn.Spawn(pawn, cell, map);
        }
        else
        {
            throw new BridgeException($"{pawn.LabelShort} is not on a map; spawn it instead");
        }
    }

    public static object MakeJob(JsonElement request)
    {
        string defName = JsonArgs.GetString(request, "defName");
        JobDef def = DefDatabase<JobDef>.GetNamedSilentFail(defName)
            ?? throw new BridgeException($"unknown JobDef '{defName}'");

        Job job;
        if (request.TryGetProperty("cell", out JsonElement cell) && cell.ValueKind == JsonValueKind.Array)
        {
            // `cell` is [x, y, z]; two elements mean a flat cell.
            JsonElement[] parts = cell.EnumerateArray().ToArray();
            var target = new IntVec3(
                parts.Length > 0 ? parts[0].GetInt32() : 0,
                parts.Length > 2 ? parts[1].GetInt32() : 0,
                parts.Length > 2 ? parts[2].GetInt32() : parts.Length > 1 ? parts[1].GetInt32() : 0);
            job = new Job(def, new LocalTargetInfo(target));
        }
        else if (request.TryGetProperty("ref", out JsonElement handle) && handle.ValueKind == JsonValueKind.Number)
        {
            object target = RefContext.Current.Get(handle.GetInt32())
                ?? throw new BridgeException($"the job target handle {handle.GetInt32()} is not valid");
            job = new Job(def, new LocalTargetInfo((Thing)target));
        }
        else
        {
            job = new Job(def);
        }

        if (request.TryGetProperty("count", out JsonElement count) && count.ValueKind == JsonValueKind.Number)
        {
            job.count = count.GetInt32();
        }

        return job;
    }

    public static object TakeJob(JsonElement request)
    {
        Pawn pawn = Resolve(request);
        object job = RefContext.Current.Get(JsonArgs.GetInt(request, "job", -1))
            ?? throw new BridgeException($"the job handle {JsonArgs.GetInt(request, "job", -1)} is not valid");
        if (pawn.jobs == null)
        {
            throw new BridgeException($"{pawn.LabelShort} cannot take jobs");
        }

        bool queue = JsonArgs.GetBool(request, "queue");
        return pawn.jobs.TryTakeOrderedJob((Job)job, null, queue);
    }

    public static object Spawn(JsonElement request)
    {
        string kindName = JsonArgs.GetString(request, "kindDef");
        PawnKindDef kind = DefDatabase<PawnKindDef>.GetNamedSilentFail(kindName)
            ?? throw new BridgeException($"unknown PawnKindDef '{kindName}'");

        Faction faction = null;
        string factionName = JsonArgs.GetStringOrNull(request, "factionDef");
        if (factionName != null)
        {
            faction = Find.FactionManager.AllFactions
                .FirstOrDefault(candidate => candidate.def?.defName == factionName || candidate.Name == factionName)
                ?? throw new BridgeException($"unknown faction '{factionName}'");
        }

        int mapId = JsonArgs.GetInt(request, "mapId", -1);
        Map map = Find.Maps.FirstOrDefault(candidate => candidate.uniqueID == mapId)
            ?? throw new BridgeException($"map {mapId} does not exist");

        var cell = new IntVec3(JsonArgs.GetInt(request, "x", 0), 0, JsonArgs.GetInt(request, "z", 0));
        if (!cell.InBounds(map))
        {
            throw new BridgeException($"{cell} is outside map {mapId}");
        }

        Pawn pawn = PawnGenerator.GeneratePawn(kind, faction, null);
        GenSpawn.Spawn(pawn, cell, map, WipeMode.Vanish);
        return pawn;
    }

    // ------------------------------------------------------------------ genes (Biotech)

    /// <summary>The genes, xenogenes and xenotype of a pawn.</summary>
    public static object Genes(JsonElement request)
    {
        Pawn pawn = Resolve(request);
        Pawn_GeneTracker genes = GenesOf(pawn);
        var list = new List<Dictionary<string, object>>();
        foreach (Gene gene in genes.GenesListForReading)
        {
            list.Add(new Dictionary<string, object>
            {
                ["defName"] = gene.def?.defName,
                ["label"] = gene.def?.LabelCap.ToString(),
                ["xenogene"] = genes.IsXenogene(gene),
            });
        }

        return new Dictionary<string, object>
        {
            ["xenotype"] = genes.Xenotype?.defName,
            ["xenotypeLabel"] = genes.XenotypeLabel,
            ["genes"] = list,
        };
    }

    public static object AddGene(JsonElement request)
    {
        Pawn pawn = Resolve(request);
        Pawn_GeneTracker genes = GenesOf(pawn);
        GeneDef def = GeneDefNamed(JsonArgs.GetString(request, "gene"));
        bool xenogene = JsonArgs.GetBool(request, "xenogene");
        genes.AddGene(def, xenogene);

        return new Dictionary<string, object>
        {
            ["gene"] = def.defName,
            ["xenogene"] = xenogene,
            ["has"] = genes.HasActiveGene(def),
        };
    }

    public static void RemoveGene(JsonElement request)
    {
        Pawn pawn = Resolve(request);
        Pawn_GeneTracker genes = GenesOf(pawn);
        GeneDef def = GeneDefNamed(JsonArgs.GetString(request, "gene"));
        Gene gene = genes.GetGene(def)
            ?? throw new BridgeException($"'{pawn.LabelShort}' does not have the gene '{def.defName}'");
        genes.RemoveGene(gene);
    }

    public static object SetXenotype(JsonElement request)
    {
        Pawn pawn = Resolve(request);
        Pawn_GeneTracker genes = GenesOf(pawn);
        string name = JsonArgs.GetString(request, "xenotype");
        XenotypeDef def = DefDatabase<XenotypeDef>.GetNamedSilentFail(name)
            ?? throw new BridgeException($"the xenotype def '{name}' does not exist");
        genes.SetXenotype(def);

        return new Dictionary<string, object>
        {
            ["xenotype"] = genes.Xenotype?.defName,
            ["label"] = genes.XenotypeLabel,
        };
    }

    // ------------------------------------------------------------------ thoughts and needs

    /// <summary>Adds a memory (a `ThoughtDef`) to a pawn; memories are what the mood is made of.</summary>
    public static object AddThought(JsonElement request)
    {
        Pawn pawn = Resolve(request);
        ThoughtDef def = ThoughtDefNamed(JsonArgs.GetString(request, "thought"));
        MemoryThoughtHandler memories = MemoriesOf(pawn);
        memories.TryGainMemory(def, null, null);

        return new Dictionary<string, object>
        {
            ["thought"] = def.defName,
            ["has"] = memories.GetFirstMemoryOfDef(def) != null,
        };
    }

    public static void RemoveThought(JsonElement request)
    {
        Pawn pawn = Resolve(request);
        ThoughtDef def = ThoughtDefNamed(JsonArgs.GetString(request, "thought"));
        MemoryThoughtHandler memories = MemoriesOf(pawn);
        Thought_Memory memory = memories.GetFirstMemoryOfDef(def)
            ?? throw new BridgeException($"'{pawn.LabelShort}' has no memory of '{def.defName}'");
        memories.RemoveMemory(memory);
    }

    /// <summary>Sets a need directly (mood is the `Mood` need), clamped to the need's own range.</summary>
    public static object SetNeed(JsonElement request)
    {
        Pawn pawn = Resolve(request);
        string name = JsonArgs.GetString(request, "need");
        NeedDef def = DefDatabase<NeedDef>.GetNamedSilentFail(name)
            ?? throw new BridgeException($"the need def '{name}' does not exist");
        Need need = pawn.needs?.AllNeeds?.FirstOrDefault(candidate => candidate.def == def)
            ?? throw new BridgeException($"'{pawn.LabelShort}' has no need '{def.defName}'");
        double wanted = JsonArgs.GetDouble(request, "level", need.CurLevel);
        need.CurLevel = (float)Math.Max(0.0, Math.Min(1.0, wanted));

        return new Dictionary<string, object>
        {
            ["need"] = def.defName,
            ["level"] = need.CurLevel,
            ["percent"] = need.CurLevelPercentage,
        };
    }

    private static Pawn_GeneTracker GenesOf(Pawn pawn)
    {
        return pawn.genes
            ?? throw new BridgeException($"'{pawn.LabelShort}' has no genes (is the Biotech expansion active?)");
    }

    private static GeneDef GeneDefNamed(string name)
    {
        return DefDatabase<GeneDef>.GetNamedSilentFail(name)
            ?? throw new BridgeException($"the gene def '{name}' does not exist");
    }

    private static ThoughtDef ThoughtDefNamed(string name)
    {
        return DefDatabase<ThoughtDef>.GetNamedSilentFail(name)
            ?? throw new BridgeException($"the thought def '{name}' does not exist");
    }

    private static MemoryThoughtHandler MemoriesOf(Pawn pawn)
    {
        return pawn.needs?.mood?.thoughts?.memories
            ?? throw new BridgeException($"'{pawn.LabelShort}' has no mood, so it cannot hold memories");
    }

    private static Pawn Resolve(JsonElement request)
    {
        int handle = JsonArgs.GetInt(request, "ref", -1);
        object target = RefContext.Current.Get(handle);
        return target switch
        {
            null => throw new BridgeException($"the pawn handle {handle} is not valid"),
            Pawn pawn => pawn,
            _ => throw new BridgeException($"handle {handle} is a {target.GetType().Name}, not a pawn"),
        };
    }

    private static int[] Cell(IntVec3 cell)
    {
        return new[] { cell.x, cell.y, cell.z };
    }
}
