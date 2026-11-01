using System;
using System.Collections.Generic;
using System.Linq;
using System.Text.Json;
using RimWorld;
using RimWorld.Planet;
using Verse;
namespace RustBridge;

/// <summary>
/// The `trade` half of <see cref="WorldOps"/>.
/// </summary>
internal static partial class WorldOps
{

    /// <summary>What a trader is: its kind, its name, what it trades in — and how much it offers.</summary>
    public static object Trader(JsonElement request)
    {
        Pawn pawn = ResolvePawn(request);
        if (pawn is not ITrader trader || trader.TraderKind == null)
        {
            throw new BridgeException(
                $"'{pawn.LabelShort}' is not a trader (its TraderKind is empty; a caravan or a settlement trader has one)");
        }

        int goods = 0;
        foreach (Thing good in trader.Goods ?? Enumerable.Empty<Thing>())
        {
            if (good != null)
            {
                goods++;
            }

            if (goods >= 512)
            {
                break;
            }
        }

        return new Dictionary<string, object>
        {
            ["label"] = pawn.LabelShort,
            ["traderKind"] = trader.TraderKind.defName,
            ["traderKindLabel"] = trader.TraderKind.LabelCap.ToString(),
            ["traderName"] = trader.TraderName,
            ["currency"] = trader.TradeCurrency.ToString(),
            ["canTradeNow"] = trader.CanTradeNow,
            ["factionDefName"] = trader.Faction?.def?.defName,
            ["goods"] = goods,
        };
    }

    /// <summary>What a trader has on offer, up to a limit.</summary>
    public static object TraderGoods(JsonElement request)
    {
        Pawn pawn = ResolvePawn(request);
        if (pawn is not ITrader trader || trader.TraderKind == null)
        {
            throw new BridgeException($"'{pawn.LabelShort}' is not a trader (its TraderKind is empty)");
        }

        int limit = Math.Min(Math.Max(JsonArgs.GetInt(request, "limit", 128), 1), 512);
        var goods = new List<Dictionary<string, object>>();
        foreach (Thing good in trader.Goods ?? Enumerable.Empty<Thing>())
        {
            if (good == null)
            {
                continue;
            }

            goods.Add(new Dictionary<string, object>
            {
                ["handle"] = good,
                ["defName"] = good.def?.defName,
                ["label"] = good.LabelShort,
                ["stackCount"] = good.stackCount,
                ["marketValue"] = good.MarketValue,
            });
            if (goods.Count >= limit)
            {
                break;
            }
        }

        return goods;
    }

    /// <summary>
    /// The game's own price formula for one thing, with a neutral trader: a live trade applies its own
    /// factors (negotiator skill, faction goodwill, the trader's kind), which is what a market preview
    /// wants to see without them.
    /// </summary>
    public static object Prices(JsonElement request)
    {
        Thing thing = ThingOps.ResolveThing(request);
        return new Dictionary<string, object>
        {
            ["label"] = thing.LabelShort,
            ["defName"] = thing.def?.defName,
            ["marketValue"] = thing.MarketValue,
            ["playerBuys"] = TradeUtility.GetPricePlayerBuy(thing, 1f, 1f, 0f, 0f),
            ["playerSells"] = TradeUtility.GetPricePlayerSell(
                thing, 1f, 1f, 0f, 0f, 0f, 0f, TradeCurrency.Silver),
        };
    }

    /// <summary>
    /// Executes one line of trade through the game's own `TradeSession` and `TradeDeal.TryExecute`: a
    /// positive count sells colony goods of a def to the trader, a negative one buys from it. The game's
    /// own `Tradeable.CanAdjustTo` decides whether the direction and the amount are legal, so its refusal
    /// is reported in its words; the price, the silver and the transfer are the game's.
    /// </summary>
    public static object ExecuteTrade(JsonElement request)
    {
        ITrader trader = TraderFromRequest(request);
        if (!trader.CanTradeNow)
        {
            throw new BridgeException($"'{trader.TraderName}' cannot trade now");
        }

        Pawn negotiator = ResolveHandle<Pawn>(request, "pawnRef", "pawn");
        string defName = JsonArgs.GetString(request, "thingDef");
        int count = JsonArgs.GetInt(request, "count", 0);
        if (count == 0)
        {
            throw new BridgeException("the count must not be zero: positive sells to the trader, negative buys from it");
        }

        try
        {
            TradeSession.SetupWith(trader, negotiator, giftMode: false);
        }
        catch (Exception error)
        {
            throw new BridgeException(
                $"the game failed to open a trade session with '{trader.TraderName}' ({error.GetType().Name}): {error.Message}",
                error);
        }

        try
        {
            TradeDeal deal = TradeSession.deal;
            if (deal == null)
            {
                throw new BridgeException("the game opened no trade deal for that trader");
            }

            Tradeable tradeable;
            try
            {
                tradeable = deal.AllTradeables?.FirstOrDefault(entry =>
                    entry?.ThingDef?.defName == defName
                    && (count > 0 ? entry.FirstThingColony != null : entry.FirstThingTrader != null));
            }
            catch (Exception error)
            {
                throw new BridgeException(
                    $"the game failed to list the deal's tradeables ({error.GetType().Name}): {error.Message}", error);
            }

            if (tradeable == null)
            {
                throw new BridgeException(
                    $"the deal has no '{defName}' {(count > 0 ? "to sell" : "to buy")} (the trader may not trade it)");
            }

            // The public sign is the caller's: a positive count sells colony goods. The game's own
            // transfer sign is the opposite, so the tradeable is adjusted by the negated count.
            int transfer = -count;
            AcceptanceReport can = tradeable.CanAdjustTo(transfer);
            if (!can.Accepted)
            {
                throw new BridgeException($"the game does not allow moving {count} of '{defName}': {can.Reason}");
            }

            tradeable.AdjustTo(transfer);
            bool executed;
            bool traded;
            try
            {
                executed = deal.TryExecute(out traded);
            }
            catch (Exception error)
            {
                throw new BridgeException(
                    $"the game failed while executing the deal ({error.GetType().Name}): {error.Message}", error);
            }

            return new Dictionary<string, object>
            {
                ["trader"] = trader.TraderName,
                ["defName"] = defName,
                ["count"] = count,
                ["executed"] = executed,
                ["traded"] = traded,
                ["canTradeNow"] = trader.CanTradeNow,
            };
        }
        finally
        {
            // Closing a session that never opened must not replace the real error with its own.
            try
            {
                TradeSession.Close();
            }
            catch (Exception error)
            {
                Log.Warning($"[Rust] closing the trade session failed: {error.GetType().Name}: {error.Message}");
            }
        }
    }

    private static ITrader TraderFromRequest(JsonElement request)
    {
        int handle = JsonArgs.GetInt(request, "traderRef", -1);
        object target = RefContext.Current.Get(handle);
        return target switch
        {
            null => throw new BridgeException($"the trader handle {handle} is not valid"),
            ITrader trader => trader,
            _ => throw new BridgeException($"handle {handle} is a {target.GetType().Name}, not a trader"),
        };
    }
}
