using System;
using System.Globalization;
using System.IO;
using System.Reflection;
using System.Text.Json;
using Verse;

namespace RustBridge;

/// <summary>
/// What a Harmony trampoline does once Harmony calls it: hand the original method, its arguments and its current
/// result to the Rust side, apply what came back (skipped original, changed arguments, a result) and stay out of
/// the way when Rust is already busy dispatching.
///
/// It is a file of its own because <c>HarmonyBridge.cs</c> holds the registration side of the same feature, and
/// one file holding both outgrew the layout rule.
/// </summary>
internal static class PatchDispatch
{
    private readonly struct Outcome
    {
        public Outcome(bool skip, bool resultSet, object result)
        {
            Skip = skip;
            ResultSet = resultSet;
            Result = result;
        }

        public bool Skip { get; }

        public bool ResultSet { get; }

        public object Result { get; }
    }

    public static bool RunPrefix(MethodBase method, object instance, object[] args, bool resultAvailable, ref object result)
    {
        if (RustHost.Current == null || !HarmonyBridge.TryGetHandler(method, prefix: true, out string handler))
        {
            return true;
        }

        Outcome outcome = Run("prefix", handler, method, instance, args, resultAvailable, resultAvailable ? result : null);
        if (outcome.ResultSet)
        {
            result = outcome.Result;
        }

        return !outcome.Skip;
    }

    public static void RunPostfix(MethodBase method, object instance, object[] args, bool resultAvailable, ref object result)
    {
        if (RustHost.Current == null || !HarmonyBridge.TryGetHandler(method, prefix: false, out string handler))
        {
            return;
        }

        Outcome outcome = Run("postfix", handler, method, instance, args, resultAvailable, resultAvailable ? result : null);
        if (outcome.ResultSet)
        {
            result = outcome.Result;
        }
    }

    private static Outcome Run(
        string kind,
        string handler,
        MethodBase method,
        object instance,
        object[] args,
        bool resultAvailable,
        object currentResult)
    {
        if (RefContext.IsDispatching)
        {
            // The Rust side is busy: this patched method was triggered by a host call from inside
            // a handler. Queue the call and let the original run unmodified.
            var table = new RefTable();
            RustHost.QueueNestedPatch(
                BuildPayload(kind, handler, method, instance, args, currentResult, table),
                table);
            return default;
        }

        using (RefContext.Enter())
        {
            byte[] payload = BuildPayload(kind, handler, method, instance, args, currentResult, RefContext.Current);
            using JsonDocument document = RustHost.Current.DispatchBytes("patch", payload);
            JsonElement root = document.RootElement;
            if (!(root.TryGetProperty("ok", out JsonElement ok) && ok.ValueKind == JsonValueKind.True))
            {
                string error = root.TryGetProperty("error", out JsonElement message) ? message.GetString() : "unknown error";
                Log.Error($"[Rust] patch handler '{handler}' failed: {error}");
                return default;
            }

            ApplyArguments(method, args, root);
            bool skip = root.TryGetProperty("skipOriginal", out JsonElement skipElement)
                && skipElement.ValueKind == JsonValueKind.True;

            object result = null;
            bool resultSet = false;
            if (root.TryGetProperty("result", out JsonElement resultElement) && resultElement.ValueKind != JsonValueKind.Null)
            {
                if (resultAvailable)
                {
                    Type returnType = method is MethodInfo info ? info.ReturnType : typeof(object);
                    result = JsonValueBridge.Read(resultElement, returnType, RefContext.Current);
                    resultSet = true;
                }
                else
                {
                    Log.Error($"[Rust] handler '{handler}' tried to set the result of {HarmonyBridge.Describe(method)} but this patch has no result access");
                }
            }

            return new Outcome(skip, resultSet, result);
        }
    }

    private static byte[] BuildPayload(
        string kind,
        string handler,
        MethodBase method,
        object instance,
        object[] args,
        object currentResult,
        RefTable refs)
    {
        using var stream = new MemoryStream();
        using (var writer = new Utf8JsonWriter(stream))
        {
            writer.WriteStartObject();
            writer.WriteString("kind", kind);
            writer.WriteString("handler", handler);
            writer.WriteString("target", HarmonyBridge.Describe(method));
            writer.WritePropertyName("instance");
            JsonValueBridge.Write(writer, instance, refs);
            writer.WritePropertyName("args");
            writer.WriteStartArray();
            if (args != null)
            {
                foreach (object arg in args)
                {
                    JsonValueBridge.Write(writer, arg, refs);
                }
            }

            writer.WriteEndArray();
            if (currentResult != null)
            {
                writer.WritePropertyName("result");
                JsonValueBridge.Write(writer, currentResult, refs);
            }

            writer.WriteEndObject();
        }

        return stream.ToArray();
    }

    private static void ApplyArguments(MethodBase method, object[] args, JsonElement response)
    {
        if (args == null || args.Length == 0)
        {
            return;
        }

        if (!response.TryGetProperty("args", out JsonElement overrides) || overrides.ValueKind != JsonValueKind.Object)
        {
            return;
        }

        ParameterInfo[] parameters = method.GetParameters();
        foreach (JsonProperty entry in overrides.EnumerateObject())
        {
            if (!int.TryParse(entry.Name, NumberStyles.Integer, CultureInfo.InvariantCulture, out int index))
            {
                continue;
            }

            if (index < 0 || index >= args.Length || index >= parameters.Length)
            {
                continue;
            }

            args[index] = JsonValueBridge.Read(entry.Value, parameters[index].ParameterType, RefContext.Current);
        }
    }
}
