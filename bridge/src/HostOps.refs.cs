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
/// The `refs` half of <see cref="HostOps"/>.
/// </summary>
internal static partial class HostOps
{
    private static object RefGet(JsonElement request)
    {
        object target = RefContext.Current.Get(JsonArgs.GetInt(request, "ref", -1));
        return MemberAccess.Get(target, JsonArgs.GetString(request, "member"));
    }
    private static void RefSet(JsonElement request)
    {
        object target = RefContext.Current.Get(JsonArgs.GetInt(request, "ref", -1));
        if (!JsonArgs.TryGet(request, "value", out JsonElement value))
        {
            throw new BridgeException("missing 'value'");
        }

        MemberAccess.Set(target, JsonArgs.GetString(request, "member"), value, RefContext.Current);
    }
    private static object RefCall(JsonElement request)
    {
        object target = RefContext.Current.Get(JsonArgs.GetInt(request, "ref", -1));
        return MemberAccess.Call(
            target,
            JsonArgs.GetString(request, "method"),
            JsonArgs.ReadArray(request, "args", RefContext.Current));
    }
}
