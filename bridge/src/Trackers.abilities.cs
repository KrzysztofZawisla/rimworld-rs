using System;
using System.Collections.Generic;
using System.Linq;
using System.Text.Json;
using RimWorld;
using Verse;
namespace RustBridge;

/// <summary>
/// The ability half of <see cref="Trackers"/>: casting one ability a pawn already has.
/// </summary>
internal static partial class Trackers
{
    /// <summary>
    /// Casts one of the pawn's abilities through the game's own `Ability.Activate` and answers with what
    /// the game decided. The target is a thing handle, a cell, or nothing at all (a self-cast).
    /// </summary>
    public static object CastAbility(JsonElement request)
    {
        Pawn pawn = Resolve(request);
        string defName = JsonArgs.GetString(request, "ability");
        List<Ability> abilities = pawn.abilities?.AllAbilitiesForReading;
        Ability ability = abilities?.FirstOrDefault(entry => entry.def?.defName == defName);
        if (ability == null)
        {
            string known = abilities == null
                ? "it has no ability tracker"
                : abilities.Count == 0 ? "it has no abilities" : $"it has {string.Join(", ", abilities.Select(entry => entry.def?.defName))}";
            throw new BridgeException($"'{pawn.LabelShort}' has no ability '{defName}': {known}");
        }

        LocalTargetInfo target = Target(request);
        // The game's own verdicts first: an ability that cannot be cast, or cannot be applied to that
        // target, is a named refusal instead of an exception from inside the game's code.
        AcceptanceReport canCast = ability.CanCast;
        if (!canCast.Accepted)
        {
            throw new BridgeException($"'{pawn.LabelShort}' cannot cast '{defName}': {canCast.Reason}");
        }

        if (!ability.CanApplyOn(target))
        {
            throw new BridgeException(
                $"'{defName}' cannot be applied to {Describe(target)}");
        }

        bool cast;
        try
        {
            cast = ability.Activate(target, LocalTargetInfo.Invalid);
        }
        catch (Exception error)
        {
            throw new BridgeException(
                $"the game failed while casting '{defName}' on {Describe(target)} ({error.GetType().Name}): {error.Message}",
                error);
        }

        return new Dictionary<string, object>
        {
            ["ability"] = defName,
            ["cast"] = cast,
            ["cooldownTicksRemaining"] = ability.CooldownTicksRemaining,
            ["target"] = target.Thing?.LabelShort ?? (target.Cell.IsValid ? $"{target.Cell.x},{target.Cell.z}" : "self"),
        };
    }

    private static string Describe(LocalTargetInfo target)
    {
        if (target.Thing != null)
        {
            return $"'{target.Thing.LabelShort}'";
        }

        return target.Cell.IsValid ? $"cell {target.Cell.x},{target.Cell.z}" : "no target";
    }

    private static LocalTargetInfo Target(JsonElement request)
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

        if (JsonArgs.GetInt(request, "x", int.MinValue) != int.MinValue)
        {
            return new LocalTargetInfo(new IntVec3(
                JsonArgs.GetInt(request, "x", 0),
                0,
                JsonArgs.GetInt(request, "z", 0)));
        }

        return LocalTargetInfo.Invalid;
    }
}
