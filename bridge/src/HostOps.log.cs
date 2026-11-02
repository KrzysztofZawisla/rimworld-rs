using System;
using System.Collections;
using System.Collections.Generic;
using System.Diagnostics;
using System.Globalization;
using System.Linq;
using System.Reflection;
using System.Text;
using System.Text.Json;
using RimWorld;
using Verse;
namespace RustBridge;

/// <summary>
/// The `log` half of <see cref="HostOps"/>.
/// </summary>
internal static partial class HostOps
{
    private static void LogMessage(JsonElement request)
    {
        string level = JsonArgs.GetStringOrNull(request, "level") ?? "info";
        string message = JsonArgs.GetString(request, "message");

        // A report line is never throttled: `[selftest]` is what the in-game gate reads, and the bridge's own
        // markers are how a player finds the mod in Player.log. Only a mod's ordinary lines are rate-limited,
        // because a log written from `on_tick` would otherwise emit one line per tick.
        int suppressed = 0;
        if (!IsReport(message)
            && !LogThrottle.Default.ShouldWrite(level + "\u0000" + message, DateTime.UtcNow, out suppressed))
        {
            return;
        }

        if (suppressed > 0)
        {
            message = $"{message} ({suppressed} identical line(s) were suppressed)";
        }

        switch (level)
        {
            case "warn":
                Log.Warning(message);
                break;
            case "error":
                Log.Error(message);
                break;
            default:
                Log.Message(message);
                break;
        }
    }

    /// <summary>A line the gates and the log readers look for, written exactly as the mod sent it.</summary>
    private static bool IsReport(string message)
    {
        return message.StartsWith("[selftest]", StringComparison.Ordinal)
            || message.StartsWith("[Rust]", StringComparison.Ordinal)
            || message.StartsWith("[RustBridge]", StringComparison.Ordinal);
    }
}
