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
}
