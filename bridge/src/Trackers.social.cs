using System.Collections.Generic;
using System.Text.Json;
using RimWorld;
using Verse;
namespace RustBridge;

/// <summary>
/// The social half of <see cref="Trackers"/>: what one pawn thinks of another, and the thoughts a
/// mood is made of.
/// </summary>
internal static partial class Trackers
{
    /// <summary>
    /// One pawn's opinion of another through the game's own `Pawn_RelationsTracker.OpinionOf`, with the
    /// game's own explanation and the two thresholds it compares the number against.
    /// </summary>
    public static object Opinion(JsonElement request)
    {
        Pawn pawn = Resolve(request);
        Pawn other = ResolvePawnByMember(request, "otherRef");
        if (pawn.relations == null)
        {
            throw new BridgeException($"'{pawn.LabelShort}' has no relations tracker, so no opinion");
        }

        int opinion = pawn.relations.OpinionOf(other);
        return new Dictionary<string, object>
        {
            ["a"] = pawn.LabelShort,
            ["b"] = other.LabelShort,
            ["opinion"] = opinion,
            ["explanation"] = pawn.relations.OpinionExplanation(other),
            ["friendThreshold"] = Pawn_RelationsTracker.FriendOpinionThreshold,
            ["rivalThreshold"] = Pawn_RelationsTracker.RivalOpinionThreshold,
            ["standing"] = opinion >= Pawn_RelationsTracker.FriendOpinionThreshold
                ? "friend"
                : opinion <= Pawn_RelationsTracker.RivalOpinionThreshold ? "rival" : "neutral",
        };
    }

    /// <summary>
    /// Every thought the pawn's mood is made of, through the game's own `ThoughtHandler`, plus the total
    /// offset the game computes from them. A pawn without a mood (an animal, a corpse) is refused.
    /// </summary>
    public static object Thoughts(JsonElement request)
    {
        Pawn pawn = Resolve(request);
        ThoughtHandler handler = pawn.needs?.mood?.thoughts;
        if (handler == null)
        {
            throw new BridgeException($"'{pawn.LabelShort}' has no mood, so no thoughts");
        }

        var all = new List<Thought>();
        handler.GetAllMoodThoughts(all);
        var thoughts = new List<Dictionary<string, object>>();
        foreach (Thought thought in all)
        {
            thoughts.Add(new Dictionary<string, object>
            {
                ["defName"] = thought.def?.defName,
                ["label"] = thought.LabelCap,
                ["moodOffset"] = thought.MoodOffset(),
                ["durationTicks"] = thought.DurationTicks,
            });
        }

        return new Dictionary<string, object>
        {
            ["pawn"] = pawn.LabelShort,
            ["totalMoodOffset"] = handler.TotalMoodOffset(),
            ["count"] = thoughts.Count,
            ["thoughts"] = thoughts,
        };
    }
}
