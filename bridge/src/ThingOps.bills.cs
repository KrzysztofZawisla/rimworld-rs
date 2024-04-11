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
/// The `bills` half of <see cref="ThingOps"/>.
/// </summary>
internal static partial class ThingOps
{

    /// <summary>
    /// Sets what a production bill does: whether it is paused, how many it wants and how it repeats,
    /// through the game's own bill fields, and answers with the bill afterwards. A bill that has none of
    /// those (a plain `Bill`) is refused with the reason.
    /// </summary>
    public static object BillSet(JsonElement request)
    {
        Thing thing = ResolveThing(request);
        BillStack stack = (thing as RimWorld.IBillGiver)?.BillStack
            ?? throw new BridgeException(
                $"'{thing.LabelShort}' cannot hold bills (a {thing.GetType().Name} is not a work giver)");
        int id = JsonArgs.GetInt(request, "id", -1);
        if (id < 0 || id >= stack.Bills.Count)
        {
            throw new BridgeException($"'{thing.LabelShort}' has no bill {id} (it has {stack.Bills.Count})");
        }

        Bill bill = stack.Bills[id];
        if (!(bill is Bill_Production production))
        {
            throw new BridgeException(
                $"bill {id} of '{thing.LabelShort}' is a {bill.GetType().Name}, which has nothing to set");
        }

        if (request.TryGetProperty("paused", out JsonElement pausedElement))
        {
            production.paused = pausedElement.GetBoolean();
        }

        if (request.TryGetProperty("count", out JsonElement countElement))
        {
            production.targetCount = Math.Max(1, countElement.GetInt32());
        }

        string repeatName = JsonArgs.GetStringOrNull(request, "repeatMode");
        if (repeatName != null)
        {
            production.repeatMode = DefDatabase<BillRepeatModeDef>.GetNamedSilentFail(repeatName)
                ?? throw new BridgeException($"there is no bill repeat mode '{repeatName}'");
        }

        return new Dictionary<string, object>
        {
            ["id"] = id,
            ["recipe"] = bill.recipe?.defName,
            ["paused"] = production.paused,
            ["count"] = production.targetCount,
            ["repeatMode"] = production.repeatMode?.defName,
        };
    }

    /// <summary>
    /// Restricts a bill to one pawn through the game's own `SetPawnRestriction`, or opens it to anyone
    /// through `SetAnyPawnRestriction` when no pawn is given, and answers with what the bill reports.
    /// </summary>
    public static object BillSetPawn(JsonElement request)
    {
        Thing thing = ResolveThing(request);
        Bill bill = RequireBill(thing, request);
        // The pawn is optional and travels as a handle number (or null when there is none).
        int pawnHandle = JsonArgs.GetInt(request, "pawnRef", -1);
        Pawn pawn = null;
        if (pawnHandle >= 0)
        {
            pawn = RefContext.Current.Get(pawnHandle) as Pawn
                ?? throw new BridgeException($"handle {pawnHandle} in 'pawnRef' is no pawn");
        }

        if (pawn == null)
        {
            bill.SetAnyPawnRestriction();
        }
        else
        {
            bill.SetPawnRestriction(pawn);
        }

        return new Dictionary<string, object>
        {
            ["id"] = JsonArgs.GetInt(request, "id", -1),
            ["recipe"] = bill.recipe?.defName,
            ["pawn"] = bill.PawnRestriction?.LabelShort,
        };
    }
    private static Bill RequireBill(Thing thing, JsonElement request)
    {
        BillStack stack = (thing as RimWorld.IBillGiver)?.BillStack
            ?? throw new BridgeException(
                $"'{thing.LabelShort}' cannot hold bills (a {thing.GetType().Name} is not a work giver)");
        int id = JsonArgs.GetInt(request, "id", -1);
        if (id < 0 || id >= stack.Bills.Count)
        {
            throw new BridgeException($"'{thing.LabelShort}' has no bill {id} (it has {stack.Bills.Count})");
        }

        return stack.Bills[id];
    }

    /// <summary>
    /// Moves a bill up or down its stack through the game's own `BillStack.Reorder` and answers with the
    /// order afterwards, so a move that did nothing is visible.
    /// </summary>
    public static object BillMove(JsonElement request)
    {
        Thing thing = ResolveThing(request);
        BillStack stack = (thing as RimWorld.IBillGiver)?.BillStack
            ?? throw new BridgeException(
                $"'{thing.LabelShort}' cannot hold bills (a {thing.GetType().Name} is not a work giver)");
        int id = JsonArgs.GetInt(request, "id", -1);
        if (id < 0 || id >= stack.Bills.Count)
        {
            throw new BridgeException($"'{thing.LabelShort}' has no bill {id} (it has {stack.Bills.Count})");
        }

        int offset = JsonArgs.GetInt(request, "offset", 0);
        Bill bill = stack.Bills[id];
        int before = stack.Bills.IndexOf(bill);
        stack.Reorder(bill, offset);
        int after = stack.Bills.IndexOf(bill);
        if (after == before)
        {
            throw new BridgeException(
                $"the game kept bill {id} at {before} (an offset of {offset} moves nothing there)");
        }

        return new Dictionary<string, object>
        {
            ["id"] = id,
            ["before"] = before,
            ["after"] = after,
            ["order"] = stack.Bills.Select(entry => entry.recipe?.defName).ToList(),
        };
    }

    /// <summary>The bills of anything that gives work: workbenches, stoves, butcher tables.</summary>
    public static object Bills(JsonElement request)
    {
        BillStack stack = BillsOf(ResolveThing(request));
        var bills = new List<Dictionary<string, object>>();
        for (int index = 0; index < stack.Bills.Count; index++)
        {
            Bill bill = stack.Bills[index];
            bills.Add(new Dictionary<string, object>
            {
                ["id"] = index,
                ["recipe"] = bill.recipe?.defName,
                ["label"] = bill.recipe?.LabelCap.ToString(),
                ["paused"] = (bill as Bill_Production)?.paused ?? false,
                ["count"] = (bill as Bill_Production)?.targetCount ?? 0,
                ["repeatMode"] = (bill as Bill_Production)?.repeatMode?.defName,
                ["handle"] = bill,
            });
        }

        return bills;
    }

    /// <summary>Queues a recipe: `count` repetitions, or forever when the count is 0.</summary>
    public static object AddBill(JsonElement request)
    {
        Thing thing = ResolveThing(request);
        BillStack stack = BillsOf(thing);
        string name = JsonArgs.GetString(request, "recipe");
        RecipeDef recipe = DefDatabase<RecipeDef>.GetNamedSilentFail(name)
            ?? throw new BridgeException($"the recipe def '{name}' does not exist");
        int count = JsonArgs.GetInt(request, "count", 1);

        var bill = new Bill_Production(recipe, null);
        if (count <= 0)
        {
            bill.repeatMode = BillRepeatModeDefOf.Forever;
        }
        else
        {
            bill.repeatMode = BillRepeatModeDefOf.RepeatCount;
            bill.repeatCount = count;
        }

        stack.AddBill(bill);
        return new Dictionary<string, object>
        {
            ["id"] = stack.Bills.Count - 1,
            ["recipe"] = recipe.defName,
            ["count"] = count,
            ["bills"] = stack.Count,
        };
    }

    /// <summary>Removes the bill at an index; without one it removes the last bill.</summary>
    public static void RemoveBill(JsonElement request)
    {
        Thing thing = ResolveThing(request);
        BillStack stack = BillsOf(thing);
        int index = JsonArgs.GetInt(request, "index", -1);
        if (index < 0)
        {
            // Without an index (or with the -1 the SDK sends for "the last one") remove the last bill.
            index = stack.Count - 1;
        }

        if (index < 0 || index >= stack.Count)
        {
            throw new BridgeException($"bill {index} does not exist ('{thing.LabelShort}' has {stack.Count} bill(s))");
        }

        stack.Delete(stack[index]);
    }
    public static object ClearBills(JsonElement request)
    {
        BillStack stack = BillsOf(ResolveThing(request));
        int removed = stack.Count;
        stack.Clear();
        return new Dictionary<string, object> { ["removed"] = removed };
    }
    private static BillStack BillsOf(Thing thing)
    {
        return (thing as RimWorld.IBillGiver)?.BillStack
            ?? throw new BridgeException($"'{thing.LabelShort}' cannot hold bills (a {thing.GetType().Name} is not a work giver)");
    }
}
