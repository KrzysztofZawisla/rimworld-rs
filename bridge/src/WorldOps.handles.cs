using System;
using System.Collections.Generic;
using System.Linq;
using System.Text.Json;
using RimWorld;
using RimWorld.Planet;
using Verse;
namespace RustBridge;

/// <summary>
/// The `handles` half of <see cref="WorldOps"/>.
/// </summary>
internal static partial class WorldOps
{

    /// <summary>
    /// Resolves a handle from a named field — `ref` for the operation's own object, or another name when
    /// the request carries a second one (`pawn` for `caravan_add_pawn`). Reading the wrong field is
    /// invisible until the game answers "handle N is a Caravan, not a pawn", which is exactly the class of
    /// bug this helper's `member` argument exists to prevent.
    /// </summary>
    private static T ResolveHandle<T>(JsonElement request, string member, string what) where T : class
    {
        int handle = JsonArgs.GetInt(request, member, -1);
        object target = RefContext.Current.Get(handle);
        return target switch
        {
            null => throw new BridgeException($"the {what} handle {handle} in '{member}' is not valid"),
            T typed => typed,
            _ => throw new BridgeException(
                $"handle {handle} in '{member}' is a {target.GetType().Name}, not a {what}"),
        };
    }
}
