using System.Collections.Generic;
using System.Linq;
using System.Text.Json;
using RimWorld;
using Verse;
namespace RustBridge;

/// <summary>
/// The ranged half of <see cref="CombatOps"/>: starting a cast with one of a pawn's own verbs, so a mod
/// does not have to reimplement aiming, warm-up or the projectile the game fires.
/// </summary>
internal static partial class CombatOps
{
    /// <summary>
    /// Starts a cast with one of the pawn's own verbs through `Verb.TryStartCastOn` and answers with
    /// what the game decided. The verb is picked by its label when one is given, otherwise the first
    /// verb that is not a melee attack. The pawn's own verbs and its equipment's verbs are one list —
    /// a gun lives on the equipment tracker, not in `VerbTracker` — and a pawn with no such verb is
    /// refused with the verbs it has.
    /// </summary>
    public static object RangedAttack(JsonElement request)
    {
        Pawn pawn = ResolvePawn(request);
        string wanted = JsonArgs.GetStringOrNull(request, "verb");
        List<Verb> verbs = (pawn.VerbTracker?.AllVerbs ?? new List<Verb>())
            .Concat(pawn.equipment?.AllEquipmentVerbs ?? new List<Verb>())
            .Distinct()
            .ToList();
        Verb verb = string.IsNullOrEmpty(wanted)
            ? verbs.FirstOrDefault(candidate => candidate.verbProps?.IsMeleeAttack == false)
            : verbs.FirstOrDefault(candidate => candidate.verbProps?.label == wanted);
        if (verb == null)
        {
            string known = verbs.Count == 0
                ? "it has no verbs"
                : $"it has {string.Join(", ", verbs.Select(candidate => candidate.verbProps?.label ?? candidate.GetType().Name))}";
            throw new BridgeException(
                string.IsNullOrEmpty(wanted)
                    ? $"'{pawn.LabelShort}' has no ranged verb: {known}"
                    : $"'{pawn.LabelShort}' has no verb '{wanted}': {known}");
        }

        LocalTargetInfo target = TargetOf(request);
        bool started = verb.TryStartCastOn(target, surpriseAttack: false, canHitNonTargetPawns: true, preventFriendlyFire: false, nonInterruptingSelfCast: false);
        return new Dictionary<string, object>
        {
            ["pawn"] = pawn.LabelShort,
            ["verb"] = verb.verbProps?.label,
            ["range"] = verb.verbProps?.range ?? 0f,
            ["started"] = started,
            ["target"] = target.Thing?.LabelShort ?? (target.Cell.IsValid ? $"{target.Cell.x},{target.Cell.z}" : "none"),
        };
    }

    private static LocalTargetInfo TargetOf(JsonElement request)
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
            JsonArgs.GetInt(request, "x", 0),
            0,
            JsonArgs.GetInt(request, "z", 0)));
    }
}
