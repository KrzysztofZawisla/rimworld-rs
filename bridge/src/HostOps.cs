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

/// <summary>Implements every host operation the Rust SDK can invoke.</summary>
internal static partial class HostOps
{


    // ---------------------------------------------------------------- instrumentation

    private static readonly Dictionary<string, long> OpCalls = new(StringComparer.Ordinal);
    private static readonly Dictionary<string, long> OpMicroseconds = new(StringComparer.Ordinal);
    private static readonly DateTime Started = DateTime.UtcNow;







































































}
