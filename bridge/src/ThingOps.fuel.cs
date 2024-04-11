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
/// The `fuel` half of <see cref="ThingOps"/>.
/// </summary>
internal static partial class ThingOps
{

    /// <summary>
    /// Fills a refuelable thing by making the fuel itself: the mod names a def and a count, the bridge makes
    /// that many units and hands them to the game's own `Refuel`, and the answer says what the thing holds
    /// afterwards plus how much the game actually added.
    /// </summary>
    public static object Refuel(JsonElement request)
    {
        Thing thing = ResolveThing(request);
        CompRefuelable refuelable = RequireRefuelable(thing);
        string defName = JsonArgs.GetString(request, "fuel");
        ThingDef fuelDef = DefDatabase<ThingDef>.GetNamedSilentFail(defName)
            ?? throw new BridgeException($"there is no thing def '{defName}'");
        int count = Math.Max(1, JsonArgs.GetInt(request, "count", 1));
        var fuel = new List<Thing>();
        for (int index = 0; index < count; index++)
        {
            Thing unit = ThingMaker.MakeThing(fuelDef, GenStuff.DefaultStuffFor(fuelDef));
            unit.stackCount = 1;
            fuel.Add(unit);
        }

        float before = refuelable.Fuel;
        refuelable.Refuel(fuel);
        Dictionary<string, object> row = FuelRow(thing, refuelable);
        row["given"] = count;
        row["gained"] = refuelable.Fuel - before;
        return row;
    }

    /// <summary>
    /// Sets the fuel level a refuelable thing aims for through the game's own `TargetFuelLevel` setter —
    /// the number the game's own refuel job fills up to — and answers with what the thing reports.
    /// </summary>
    public static object SetFuelTarget(JsonElement request)
    {
        Thing thing = ResolveThing(request);
        CompRefuelable refuelable = RequireRefuelable(thing);
        double target = JsonArgs.GetDouble(request, "target", 0.0);
        refuelable.TargetFuelLevel = (float)Math.Max(0.0, target);
        return FuelRow(thing, refuelable);
    }

    /// <summary>
    /// Burns fuel through the game's own `ConsumeFuel` and answers with what is left, so a burn the game
    /// clamped is visible instead of assumed.
    /// </summary>
    public static object ConsumeFuel(JsonElement request)
    {
        Thing thing = ResolveThing(request);
        CompRefuelable refuelable = RequireRefuelable(thing);
        float before = refuelable.Fuel;
        refuelable.ConsumeFuel((float)Math.Max(0.0, JsonArgs.GetDouble(request, "amount", 0.0)));
        Dictionary<string, object> row = FuelRow(thing, refuelable);
        row["burned"] = before - refuelable.Fuel;
        return row;
    }
    private static CompRefuelable RequireRefuelable(Thing thing)
    {
        CompRefuelable refuelable = (thing as ThingWithComps)?.GetComp<CompRefuelable>();
        if (refuelable == null)
        {
            throw new BridgeException(
                $"'{thing.LabelShort}' ({thing.GetType().Name}) burns no fuel (it has no CompRefuelable)");
        }

        return refuelable;
    }
    private static Dictionary<string, object> FuelRow(Thing thing, CompRefuelable refuelable)
    {
        return new Dictionary<string, object>
        {
            ["label"] = thing.LabelShort,
            ["target"] = refuelable.TargetFuelLevel,
            ["fuel"] = refuelable.Fuel,
            ["fuelPercentOfTarget"] = refuelable.FuelPercentOfTarget,
            ["hasFuel"] = refuelable.HasFuel,
        };
    }
}
