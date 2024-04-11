using System.Text.Json;
using RimWorld;
using Verse;
using Verse.AI.Group;

namespace RustBridge;

/// <summary>
/// The handle resolvers of <see cref="CombatOps"/>: every operation receives handles and has to turn them into
/// the game object the work needs, with an error that names what the handle actually was. They live here so the
/// operations themselves stay in one file that fits a reader's attention.
/// </summary>
internal static partial class CombatOps
{
    private static Lord ResolveLord(JsonElement request)
    {
        int handle = JsonArgs.GetInt(request, "ref", -1);
        object target = RefContext.Current.Get(handle);
        if (target is Lord lord)
        {
            return lord;
        }

        throw new BridgeException(target == null
            ? $"the lord handle {handle} is not valid"
            : $"handle {handle} is a {target.GetType().Name}, not a lord");
    }

    private static Pawn ResolvePawnByMember(JsonElement request, string member)
    {
        int handle = JsonArgs.GetInt(request, member, -1);
        object target = RefContext.Current.Get(handle);
        if (target is Pawn pawn)
        {
            return pawn;
        }

        throw new BridgeException(target == null
            ? $"the pawn handle {handle} in '{member}' is not valid"
            : $"handle {handle} in '{member}' is a {target.GetType().Name}, not a pawn");
    }

    private static Pawn ResolvePawn(JsonElement request)
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

    private static ThingWithComps ResolveItem(JsonElement request, string member)
    {
        int handle = JsonArgs.GetInt(request, member, -1);
        object target = RefContext.Current.Get(handle);
        return target switch
        {
            null => throw new BridgeException($"the handle {member} = {handle} is not valid"),
            ThingWithComps thing => thing,
            _ => throw new BridgeException(
                $"handle {handle} is a {target.GetType().Name}, not a thing with comps (equipment needs one)"),
        };
    }

    private static Thing ResolveThing(JsonElement request, string member)
    {
        int handle = JsonArgs.GetInt(request, member, -1);
        object target = RefContext.Current.Get(handle);
        return target switch
        {
            null => throw new BridgeException($"the handle {member} = {handle} is not valid"),
            Thing thing => thing,
            _ => throw new BridgeException($"handle {handle} is a {target.GetType().Name}, not a thing"),
        };
    }
}
