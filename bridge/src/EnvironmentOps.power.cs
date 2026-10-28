using System;
using System.Collections.Generic;
using System.Linq;
using System.Text.Json;
using RimWorld;
using Verse;

namespace RustBridge;

/// <summary>
/// The power half of <see cref="EnvironmentOps"/>: a map's nets, one thing's power components and the
/// switch a flickable building carries.
/// </summary>
internal static partial class EnvironmentOps
{
    public static object Power(JsonElement request)
    {
        Map map = ResolveMap(request);
        var nets = new List<Dictionary<string, object>>();
        float stored = 0;
        float gain = 0;
        bool hasPower = false;

        List<PowerNet> all = map.powerNetManager?.AllNetsListForReading;
        if (all != null)
        {
            foreach (PowerNet net in all)
            {
                if (net == null || net.Map?.uniqueID != map.uniqueID)
                {
                    continue;
                }

                float netStored = net.CurrentStoredEnergy();
                float netGain = net.CurrentEnergyGainRate();
                bool active = net.HasActivePowerSource;
                stored += netStored;
                gain += netGain;
                hasPower |= active;
                nets.Add(new Dictionary<string, object>
                {
                    ["stored"] = netStored,
                    ["gain"] = netGain,
                    ["hasActivePowerSource"] = active,
                    ["traders"] = net.powerComps?.Count ?? 0,
                    ["batteries"] = net.batteryComps?.Count ?? 0,
                });
            }
        }

        return new Dictionary<string, object>
        {
            ["nets"] = nets,
            ["storedTotal"] = stored,
            ["gainTotal"] = gain,
            ["hasPower"] = hasPower,
        };
    }

    public static object ThingPower(JsonElement request)
    {
        Thing thing = ThingOps.ResolveThing(request);
        List<ThingComp> comps = (thing as ThingWithComps)?.AllComps;
        var trader = comps?.OfType<CompPowerTrader>().FirstOrDefault();
        var battery = comps?.OfType<CompPowerBattery>().FirstOrDefault();
        var flickable = comps?.OfType<CompFlickable>().FirstOrDefault();
        if (trader == null && battery == null && flickable == null)
        {
            throw new BridgeException(
                $"'{thing.LabelShort}' has no power component (CompPowerTrader, CompPowerBattery or CompFlickable)");
        }

        return new Dictionary<string, object>
        {
            ["label"] = thing.LabelShort,
            ["powerOn"] = trader?.PowerOn,
            ["powerOutput"] = trader?.PowerOutput,
            ["storedEnergy"] = battery?.StoredEnergy,
            ["storedEnergyPct"] = battery?.StoredEnergyPct,
            ["storedEnergyCanAccept"] = battery?.AmountCanAccept,
            ["flickable"] = flickable?.SwitchIsOn,
        };
    }

    public static object SetFlickable(JsonElement request)
    {
        Thing thing = ThingOps.ResolveThing(request);
        bool on = JsonArgs.GetBool(request, "on");
        var flickable = (thing as ThingWithComps)?.AllComps?.OfType<CompFlickable>().FirstOrDefault();
        if (flickable == null)
        {
            throw new BridgeException(
                $"'{thing.LabelShort}' has no switch: a flickable building needs the game's CompFlickable (lamps, benches, turrets, ...)");
        }

        bool before = flickable.SwitchIsOn;
        if (before != on)
        {
            flickable.SwitchIsOn = on;
        }

        return new Dictionary<string, object>
        {
            ["label"] = thing.LabelShort,
            ["before"] = before,
            ["after"] = flickable.SwitchIsOn,
        };
    }
}
