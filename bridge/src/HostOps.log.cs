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
}
