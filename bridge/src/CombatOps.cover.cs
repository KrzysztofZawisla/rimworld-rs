using System.Collections.Generic;
using System.Linq;
using System.Text.Json;
using RimWorld;
using Verse;
namespace RustBridge;

/// <summary>
/// The cover half of <see cref="CombatOps"/>: the game's own answer to "how much does this cell protect
/// the target from that shooter", and which things do the protecting.
/// </summary>
internal static partial class CombatOps
{
    /// <summary>
    /// How much cover the target has against a shooter at a cell, through the game's own
    /// `CoverUtility.CalculateOverallBlockChance`, plus every cover giver the game counted.
    /// </summary>
    public static object Cover(JsonElement request)
    {
        Map map = MapForCombat(request);
        IntVec3 shooter = new IntVec3(JsonArgs.GetInt(request, "x", 0), 0, JsonArgs.GetInt(request, "z", 0));
        LocalTargetInfo target = CoverTarget(request);
        float chance = CoverUtility.CalculateOverallBlockChance(target, shooter, map);
        var givers = new List<Dictionary<string, object>>();
        foreach (CoverInfo info in CoverUtility.CalculateCoverGiverSet(target, shooter, map))
        {
            givers.Add(new Dictionary<string, object>
            {
                ["thing"] = info.Thing,
                ["defName"] = info.Thing?.def?.defName,
                ["label"] = info.Thing?.LabelShort,
                ["blockChance"] = info.BlockChance,
            });
        }

        return new Dictionary<string, object>
        {
            ["blockChance"] = chance,
            ["count"] = givers.Count,
            ["givers"] = givers,
            ["shooter"] = new Dictionary<string, object> { ["x"] = shooter.x, ["z"] = shooter.z },
        };
    }

    private static Map MapForCombat(JsonElement request)
    {
        int mapId = JsonArgs.GetInt(request, "mapId", -1);
        Map map = Find.Maps?.FirstOrDefault(candidate => candidate.uniqueID == mapId);
        return map ?? throw new BridgeException($"map {mapId} does not exist");
    }

    private static LocalTargetInfo CoverTarget(JsonElement request)
    {
        int handle = JsonArgs.GetInt(request, "targetRef", -1);
        if (handle >= 0)
        {
            object target = RefContext.Current.Get(handle);
            if (target is not Thing thing)
            {
                throw new BridgeException(target == null
                    ? $"the target handle {handle} is not valid"
                    : $"handle {handle} is a {target.GetType().Name}, not a thing");
            }

            return new LocalTargetInfo(thing);
        }

        return new LocalTargetInfo(new IntVec3(
            JsonArgs.GetInt(request, "targetX", 0),
            0,
            JsonArgs.GetInt(request, "targetZ", 0)));
    }
}
