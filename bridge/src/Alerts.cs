using System;
using System.Collections.Generic;
using System.Linq;
using System.Reflection;
using RimWorld;
using Verse;

namespace RustBridge;

/// <summary>
/// The alert a Rust mod shows in the game's alert column.
///
/// RimWorld discovers this class by scanning the loaded assemblies (`AlertsReadout` collects every
/// `Alert` subclass) and instantiates it once, so a mod does not have to ship a managed alert type of
/// its own. The state is *pushed* from Rust — `ctx.alert_set(label, explanation, priority)` and
/// `ctx.alert_clear()` — and <see cref="GetReport"/> hands the game whatever was pushed last; the
/// bridge assembly is per mod, so one mod's alert cannot see another's state.
/// </summary>
public class RustAlert : Alert
{
    private static bool active;
    private static string label;
    private static string explanation;
    private static AlertPriority priority = AlertPriority.Medium;

    public RustAlert()
    {
        defaultLabel = "Rust mod alert";
        defaultExplanation = "A Rust mod reports a problem; read its message in the mod's own description.";
        defaultPriority = AlertPriority.Medium;
    }

    public override AlertReport GetReport()
    {
        return active ? AlertReport.Active : AlertReport.Inactive;
    }

    public override AlertPriority Priority => priority;

    public override string GetLabel() => label ?? defaultLabel;

    public override TaggedString GetExplanation() => explanation ?? defaultExplanation;

    /// <summary>Shows the alert with these texts; the priority names are the game's (`Medium`, `High`, `Critical`).</summary>
    internal static void Set(string newLabel, string newExplanation, string newPriority)
    {
        label = string.IsNullOrEmpty(newLabel) ? null : newLabel;
        explanation = string.IsNullOrEmpty(newExplanation) ? null : newExplanation;
        priority = ParsePriority(newPriority);
        active = true;
        Register();
    }

    /// <summary>
    /// Makes sure the game's alert column is watching this class. `AlertsReadout` caches the alert
    /// types of every loaded assembly *once* (`allAlertTypesCached`) when the readout is first built,
    /// which can happen before a mod's assembly is scanned, so the instance has to be added by hand.
    /// `AllAlerts` is internal to the game, hence the reflection.
    /// </summary>
    private static void Register()
    {
        try
        {
            AlertsReadout readout = Find.Alerts;
            FieldInfo field = typeof(AlertsReadout).GetField(
                "AllAlerts",
                BindingFlags.Instance | BindingFlags.Public | BindingFlags.NonPublic);
            if (readout == null || field?.GetValue(readout) is not List<Alert> alerts)
            {
                return;
            }

            if (!alerts.Any(alert => alert is RustAlert))
            {
                alerts.Add(new RustAlert());
                Log.Message("[Rust] the mod's alert joined the game's alert column");
            }
        }
        catch (Exception error)
        {
            Log.Warning($"[Rust] could not register the mod's alert with the game's alert column: {error.Message}");
        }
    }

    /// <summary>Hides the alert again.</summary>
    internal static void Clear() => active = false;

    /// <summary>Whether the mod is currently asking for the alert; read by `alerts_list`.</summary>
    internal static bool IsActive => active;

    private static AlertPriority ParsePriority(string name)
    {
        if (!string.IsNullOrEmpty(name) && Enum.TryParse(name, ignoreCase: true, out AlertPriority parsed))
        {
            return parsed;
        }

        return AlertPriority.Medium;
    }
}
