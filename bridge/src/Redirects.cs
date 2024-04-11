using System;
using System.Collections;
using System.Collections.Generic;
using System.Linq;
using System.Reflection;
using System.Reflection.Emit;
using System.Text.Json;
using Verse;

namespace RustBridge;

/// <summary>
/// Rewrites calls inside one method (a Harmony transpiler): <c>harmony_redirect</c> points a call
/// at another method, <c>harmony_replace</c> sends it to a Rust handler.
///
/// Everything Harmony-specific is reached through reflection, so the bridge keeps working without
/// the Harmony mod. Harmony refuses a <c>DynamicMethod</c> as a patch method and asks for a factory
/// instead, so <see cref="Factory"/> (static, returns <c>MethodInfo</c>, takes the original
/// <c>MethodBase</c>) builds the transpiler of one target on demand.
/// </summary>
internal static class Redirects
{
    /// <summary>How many replaced calls were skipped because Rust was already running.</summary>
    private static int skippedWhileBusy;

    /// <summary>Targets whose skip has already been logged (one log per target, the rest are counted).</summary>
    private static readonly HashSet<MethodBase> SkippedLogged = new();

    /// <summary>Readable through `harmony_patches`, so a skipped call is never invisible.</summary>
    public static int SkippedWhileBusy => skippedWhileBusy;

    /// <summary>
    /// Handlers whose answer does not matter: when Rust is already running, they are skipped without an
    /// error (the default value is fine), because the mod declared that the answer is not needed. The
    /// count still rises, so nothing disappears silently.
    /// </summary>
    private static readonly HashSet<string> OptionalHandlers = new();

    /// <summary>Declares that a redirect handler's answer may be dropped when Rust is busy.</summary>
    public static void MarkOptional(string handler)
    {
        if (!string.IsNullOrEmpty(handler))
        {
            OptionalHandlers.Add(handler);
            Log.Message($"[Rust] the redirect handler '{handler}' is optional: when Rust is busy its answer is skipped quietly");
        }
    }

    /// <summary>What a matched call becomes.</summary>
    internal sealed class Plan
    {
        /// <summary>The method whose calls are rewritten.</summary>
        public MethodInfo From;

        /// <summary>The replacement, when the call is redirected to another method.</summary>
        public MethodInfo To;

        /// <summary>The Rust handler, when the call goes into Rust instead.</summary>
        public string Handler;

        /// <summary>Value returned when Rust is busy or the handler answers nothing.</summary>
        public object Default;
    }

    /// <summary>Plans per patched (target) method.</summary>
    private static readonly Dictionary<MethodBase, List<Plan>> Plans = new();

    /// <summary>Key of each patched target, used as the constant the generated transpiler passes on.</summary>
    private static readonly Dictionary<MethodBase, string> Keys = new();

    private static readonly Dictionary<string, MethodBase> Targets = new();

    /// <summary>Shims per handler, so a replaced call can be made callable from IL.</summary>
    private static readonly Dictionary<string, MethodInfo> Shims = new();

    /// <summary>Harmony's <c>CodeInstruction</c>; the bridge sets it once Harmony is found.</summary>
    public static Type CodeInstructionType { get; set; }

    public static void Add(MethodBase target, Plan plan)
    {
        if (!Plans.TryGetValue(target, out List<Plan> plans))
        {
            plans = new List<Plan>();
            Plans[target] = plans;
            string key = $"RustRedirect{Keys.Count}";
            Keys[target] = key;
            Targets[key] = target;
        }

        plans.RemoveAll(existing => existing.From == plan.From);
        plans.Add(plan);
    }

    public static bool HasPlan(MethodBase target)
    {
        return Plans.ContainsKey(target);
    }

    /// <summary>
    /// Harmony's factory: called with the method being patched, it returns the transpiler that
    /// rewrites that method's calls. A dynamic method cannot be passed as a patch directly.
    /// </summary>
    public static MethodInfo Factory(MethodBase original)
    {
        if (CodeInstructionType == null)
        {
            throw new BridgeException("Harmony is not available (the brrainz.harmony mod is not active)");
        }

        if (original == null || !Keys.TryGetValue(original, out string key))
        {
            throw new BridgeException($"no call rewriting is registered for {original}");
        }

        Type enumerable = typeof(IEnumerable<>).MakeGenericType(CodeInstructionType);
        var transpiler = new DynamicMethod(
            $"RimWorldRustTranspiler_{key}",
            enumerable,
            new[] { enumerable },
            typeof(Redirects),
            skipVisibility: true);

        ILGenerator il = transpiler.GetILGenerator();
        il.Emit(OpCodes.Ldarg_0);
        il.Emit(OpCodes.Ldstr, key);
        il.Emit(OpCodes.Call, typeof(Redirects).GetMethod(nameof(Run), BindingFlags.Public | BindingFlags.Static));
        il.Emit(OpCodes.Castclass, typeof(List<>).MakeGenericType(CodeInstructionType));
        il.Emit(OpCodes.Ret);
        return transpiler;
    }

    /// <summary>
    /// The transpiler body: walks the instructions of the patched method and replaces every call
    /// that matches a plan of that target. Public and object-typed, because Harmony's own types are
    /// not available to the compiler here.
    /// </summary>
    public static object Run(object instructions, string key)
    {
        var result = (IList)Activator.CreateInstance(typeof(List<>).MakeGenericType(CodeInstructionType));
        if (instructions is IEnumerable sequence)
        {
            foreach (object instruction in sequence)
            {
                result.Add(instruction);
            }
        }

        if (!Targets.TryGetValue(key, out MethodBase original)
            || !Plans.TryGetValue(original, out List<Plan> plans)
            || plans.Count == 0)
        {
            return result;
        }

        var rewrites = new List<InstructionRewrite>();
        foreach (Plan plan in plans)
        {
            if (plan.To == null && plan.Handler != null)
            {
                plan.To = Shims.TryGetValue(plan.Handler, out MethodInfo shim) ? shim : null;
            }

            rewrites.Add(new InstructionRewrite { From = plan.From, To = plan.To, Handler = plan.Handler });
        }

        int replacements = InstructionRewriting.Apply(
            result,
            CodeInstructionType,
            rewrites,
            message => Log.Error("[Rust] " + message));

        if (replacements < 0)
        {
            Plans.Clear();
            return result;
        }

        if (replacements > 0)
        {
            Log.Message($"[Rust] transpiler rewrote {replacements} call(s) inside {HarmonyBridge.Describe(original)}");
        }

        return result;
    }

    /// <summary>Creates the callable shim of a handler: same signature as the replaced method.</summary>
    public static MethodInfo Shim(MethodInfo matched, string handler, object defaultValue)
    {
        if (Shims.TryGetValue(handler, out MethodInfo existing))
        {
            // A shim is built for one signature, so a second replaced call under the same handler name is only
            // valid when it has that same shape. Handing out the first shim for a different one rewrites a call
            // site to a method that cannot take its arguments - a mismatch the compiler never sees - so the name
            // stays bound to the shape it was built from.
            if (SameSignature(existing, matched))
            {
                return existing;
            }

            throw new BridgeException(
                $"the handler '{handler}' is already bound to a replaced call of a different shape: one handler name can only replace calls with the same signature, so give {HarmonyBridge.Describe(matched)} a handler name of its own");
        }

        ParameterInfo[] parameters = matched.GetParameters();
        if (!matched.IsStatic && (matched.DeclaringType == null || matched.DeclaringType.IsValueType))
        {
            throw new BridgeException(
                $"cannot replace the instance call {HarmonyBridge.Describe(matched)}: only static methods and reference-type instances are supported");
        }

        var parameterTypes = new List<Type>();
        if (!matched.IsStatic)
        {
            parameterTypes.Add(matched.DeclaringType);
        }

        foreach (ParameterInfo parameter in parameters)
        {
            parameterTypes.Add(parameter.ParameterType);
        }

        var shim = new DynamicMethod(
            $"RustRedirect_{handler}",
            matched.ReturnType,
            parameterTypes.ToArray(),
            typeof(Redirects),
            skipVisibility: true);

        ILGenerator il = shim.GetILGenerator();
        LocalBuilder arguments = il.DeclareLocal(typeof(object[]));
        LocalBuilder result = il.DeclareLocal(typeof(object));

        il.Emit(OpCodes.Ldc_I4, parameters.Length);
        il.Emit(OpCodes.Newarr, typeof(object));
        il.Emit(OpCodes.Stloc, arguments);

        int offset = matched.IsStatic ? 0 : 1;
        for (int index = 0; index < parameters.Length; index++)
        {
            Type parameterType = parameters[index].ParameterType;
            if (parameterType.IsByRef || parameterType.IsPointer)
            {
                throw new BridgeException(
                    $"cannot replace {HarmonyBridge.Describe(matched)}: 'ref', 'out' and pointer parameters are not supported");
            }

            il.Emit(OpCodes.Ldloc, arguments);
            il.Emit(OpCodes.Ldc_I4, index);
            il.Emit(OpCodes.Ldarg, index + offset);
            if (parameterType.IsValueType || parameterType.IsGenericParameter)
            {
                il.Emit(OpCodes.Box, parameterType);
            }

            il.Emit(OpCodes.Stelem_Ref);
        }

        il.Emit(OpCodes.Ldstr, handler);
        if (matched.IsStatic)
        {
            il.Emit(OpCodes.Ldnull);
        }
        else
        {
            il.Emit(OpCodes.Ldarg_0);
        }

        il.Emit(OpCodes.Ldloc, arguments);
        il.Emit(OpCodes.Call, typeof(Redirects).GetMethod(nameof(Invoke), BindingFlags.Public | BindingFlags.Static));
        il.Emit(OpCodes.Stloc, result);

        if (matched.ReturnType == typeof(void))
        {
            il.Emit(OpCodes.Ret);
        }
        else
        {
            il.Emit(OpCodes.Ldloc, result);
            if (matched.ReturnType.IsValueType || matched.ReturnType.IsGenericParameter)
            {
                il.Emit(OpCodes.Unbox_Any, matched.ReturnType);
            }
            else
            {
                il.Emit(OpCodes.Castclass, matched.ReturnType);
            }

            il.Emit(OpCodes.Ret);
        }

        Shims[handler] = shim;
        return shim;
    }

    /// <summary>The value a replaced call returns when Rust does not answer.</summary>
    public static object DefaultValue(MethodInfo matched)
    {
        if (matched.ReturnType == typeof(void))
        {
            return null;
        }

        return matched.ReturnType.IsValueType ? Activator.CreateInstance(matched.ReturnType) : null;
    }

    /// <summary>Called by every generated shim: runs the Rust handler and converts its answer.</summary>
    public static object Invoke(string handler, object instance, object[] args)
    {
        if (!Handlers.TryGetValue(handler, out HandlerEntry entry))
        {
            Log.Error($"[Rust] no redirect handler '{handler}' is registered");
            return null;
        }

        MethodInfo matched = entry.Matched;
        if (RefContext.IsDispatching)
        {
            // Calling into Rust now would deadlock the native module lock the running call holds, so the
            // handler is skipped and the game gets the default value. The first skip of each target is
            // logged with its stack; the rest are counted, and the count is readable through
            // `harmony_patches`, so nothing is hidden and the log is not flooded either.
            skippedWhileBusy++;
            if (!OptionalHandlers.Contains(handler) && SkippedLogged.Add(matched))
            {
                Log.Error(
                    $"[Rust] the replaced call {HarmonyBridge.Describe(matched)} happened while Rust was already running; the handler is skipped and the default value is used (further skips of this target are counted in harmony_patches, not logged)");
            }

            return DefaultValue(matched);
        }

        using (RefContext.Enter())
        {
            byte[] payload = Payload(handler, entry, instance, args, RefContext.Current);
            using JsonDocument document = RustHost.Current.DispatchBytes("redirect", payload);
            JsonElement root = document.RootElement;
            if (!(root.TryGetProperty("ok", out JsonElement ok) && ok.ValueKind == JsonValueKind.True))
            {
                string error = root.TryGetProperty("error", out JsonElement message) ? message.GetString() : "unknown error";
                Log.Error($"[Rust] the handler of {HarmonyBridge.Describe(matched)} failed: {error}");
                return DefaultValue(matched);
            }

            if (!root.TryGetProperty("result", out JsonElement value) || value.ValueKind == JsonValueKind.Null)
            {
                return DefaultValue(matched);
            }

            try
            {
                return JsonValueBridge.Read(value, matched.ReturnType, RefContext.Current);
            }
            catch (Exception error)
            {
                Log.Error($"[Rust] the handler of {HarmonyBridge.Describe(matched)} returned an unusable value: {error.Message}");
                return DefaultValue(matched);
            }
        }
    }

    private static byte[] Payload(string handler, HandlerEntry entry, object instance, object[] args, RefTable refs)
    {
        using var stream = new System.IO.MemoryStream();
        using (var writer = new Utf8JsonWriter(stream))
        {
            writer.WriteStartObject();
            writer.WriteString("handler", handler);
            writer.WriteString("target", HarmonyBridge.Describe(entry.Target));
            writer.WriteString("call", HarmonyBridge.Describe(entry.Matched));
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
            writer.WriteEndObject();
        }

        return stream.ToArray();
    }

    /// <summary>What a handler is bound to, so the shim can describe and convert the call.</summary>
    internal sealed class HandlerEntry
    {
        public MethodBase Target;
        public MethodInfo Matched;
    }

    private static readonly Dictionary<string, HandlerEntry> Handlers = new();

    public static void RegisterHandler(string handler, MethodBase target, MethodInfo matched)
    {
        Handlers[handler] = new HandlerEntry { Target = target, Matched = matched };
    }

    /// <summary>
    /// Whether a cached shim can carry a replaced method: the return type and every parameter, including the
    /// instance a non-static replaced call is entered with.
    /// </summary>
    private static bool SameSignature(MethodInfo shim, MethodInfo matched)
    {
        ParameterInfo[] expected = shim.GetParameters();
        ParameterInfo[] actual = matched.GetParameters();
        int offset = matched.IsStatic ? 0 : 1;
        if (expected.Length != actual.Length + offset || shim.ReturnType != matched.ReturnType)
        {
            return false;
        }

        for (int index = 0; index < actual.Length; index++)
        {
            if (expected[index + offset].ParameterType != actual[index].ParameterType)
            {
                return false;
            }
        }

        return true;
    }

    /// <summary>
    /// Drops everything registered for one target. Harmony's unpatch takes this bridge's transpiler off a method,
    /// so a registry that keeps the target would answer <c>HasPlan</c> with true, make the next redirect skip
    /// installing the transpiler, and log a success while nothing rewrites the call.
    /// </summary>
    public static void Forget(MethodBase target)
    {
        Plans.Remove(target);
        if (Keys.TryGetValue(target, out string key))
        {
            Keys.Remove(target);
            Targets.Remove(key);
        }
    }
}
