using System;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Text.Json;
using Verse;

namespace RustBridge;

/// <summary>
/// Harmony support without a compile time dependency: the game does not ship Harmony, the
/// brrainz.harmony mod does, so everything is resolved reflectively at runtime. Patching is
/// "best effort": targets that cannot expose their result fall back to a result-less patch.
/// </summary>
internal static class HarmonyBridge
{
    private static bool attempted;
    private static bool available;
    private static object harmony;
    private static Type harmonyMethodType;
    private static MethodInfo patchMethod;

    /// <summary>The id this bridge patches under: what `Unpatch` uses to take its own patches back.</summary>
    private static string harmonyId;

    private static readonly Dictionary<MethodBase, string> PrefixHandlers = new();
    private static readonly Dictionary<MethodBase, string> PostfixHandlers = new();
    private static readonly Dictionary<MethodBase, bool> ResultWritable = new();

    public static bool TryGetHandler(MethodBase method, bool prefix, out string handler)
    {
        return (prefix ? PrefixHandlers : PostfixHandlers).TryGetValue(method, out handler);
    }

    /// <summary>
    /// Rewrites the calls inside one method: `harmony_redirect` points a call at another method,
    /// `harmony_replace` sends it to a Rust handler. The rest of the method body keeps running.
    /// </summary>
    public static void Rewrite(JsonElement request, bool replacement)
    {
        if (!Ensure())
        {
            throw new BridgeException(
                "Harmony is not available - install the brrainz.harmony mod to rewrite calls");
        }

        MethodInfo target = MethodFrom(request, "target");
        MethodInfo matched = MethodFrom(request, replacement ? "call" : "from");
        var plan = new Redirects.Plan { From = matched };

        if (replacement)
        {
            string handler = JsonArgs.GetString(request, "handler");
            plan.Handler = handler;
            plan.Default = Redirects.DefaultValue(matched);
            Redirects.RegisterHandler(handler, target, matched);
            plan.To = Redirects.Shim(matched, handler, plan.Default);
        }
        else
        {
            plan.To = MethodFrom(request, "to");
            if (plan.To.IsStatic != matched.IsStatic)
            {
                throw new BridgeException(
                    $"cannot redirect {Describe(matched)} to {Describe(plan.To)}: one is static and the other is not, which the rewritten call site cannot take");
            }
        }

        bool alreadyPatched = Redirects.HasPlan(target);
        Redirects.Add(target, plan);
        if (!alreadyPatched)
        {
            ApplyTranspiler(target);
        }

        string destination = replacement ? $"the Rust handler '{plan.Handler}'" : Describe(plan.To);
        Log.Message($"[Rust] calls to {Describe(matched)} inside {Describe(target)} now go to {destination}");
    }

    /// <summary>Reads `{ "typeName": ..., "method": ..., "paramTypes": [...] }`.</summary>
    private static MethodInfo MethodFrom(JsonElement request, string property)
    {
        if (!request.TryGetProperty(property, out JsonElement element) || element.ValueKind != JsonValueKind.Object)
        {
            throw new BridgeException($"the request needs a '{property}' object with typeName, method and optional paramTypes");
        }

        string typeName = JsonArgs.GetString(element, "typeName");
        string methodName = JsonArgs.GetString(element, "method");
        string[] paramTypes = JsonArgs.GetStringArray(element, "paramTypes");
        Type type = JsonValueBridge.ResolveType(typeName)
            ?? throw new BridgeException(
                $"type not found: {typeName}{MethodSelection.SuggestTypes(AppDomain.CurrentDomain.GetAssemblies(), typeName)}");
        return MethodSelection.Select(type, typeName, methodName, paramTypes);
    }

    /// <summary>
    /// Installs the transpiler that carries out every plan of one target method. Harmony refuses a
    /// DynamicMethod as a patch method, so the bridge hands it the factory that builds one.
    /// </summary>
    private static void ApplyTranspiler(MethodBase target)
    {
        Type codeInstruction = FindType("HarmonyLib.CodeInstruction");
        if (codeInstruction == null)
        {
            throw new BridgeException("this Harmony version has no HarmonyLib.CodeInstruction; call rewriting is disabled");
        }

        Redirects.CodeInstructionType = codeInstruction;
        MethodInfo factory = typeof(Redirects).GetMethod(nameof(Redirects.Factory), BindingFlags.Public | BindingFlags.Static);
        try
        {
            object patchInfo = Activator.CreateInstance(harmonyMethodType, factory);
            patchMethod.Invoke(harmony, new object[] { target, null, null, patchInfo, null });
        }
        catch (Exception error)
        {
            Exception actual = error is TargetInvocationException { InnerException: not null } invocation
                ? invocation.InnerException
                : error;
            throw new BridgeException($"patching {Describe(target)} with a transpiler failed: {actual.Message}");
        }
    }

    public static void Patch(string typeName, string methodName, string[] paramTypes, string kind, string handler,
        int priority = -1, string before = null, string after = null)
    {
        if (!Ensure())
        {
            throw new BridgeException("Harmony is not available - install the brrainz.harmony mod to use harmony_patch");
        }

        bool prefix = string.Equals(kind, "prefix", StringComparison.OrdinalIgnoreCase);
        if (!prefix && !string.Equals(kind, "postfix", StringComparison.OrdinalIgnoreCase))
        {
            throw new BridgeException($"unknown patch kind '{kind}' (expected 'prefix' or 'postfix')");
        }

        MethodBase target = FindTarget(typeName, methodName, paramTypes);
        bool returnsValue = target is MethodInfo info && info.ReturnType != typeof(void);
        bool withResult = returnsValue;

        if (!TryApply(target, prefix, withResult, out Exception error, priority, before, after))
        {
            if (!withResult)
            {
                throw new BridgeException($"patching {Describe(target)} failed: {error.Message}");
            }

            withResult = false;
            if (!TryApply(target, prefix, withResult, out error, priority, before, after))
            {
                throw new BridgeException($"patching {Describe(target)} failed: {error.Message}");
            }

            Log.Warning($"[Rust] {Describe(target)}: this target does not support result access; the handler will not see or set the result");
        }

        if (prefix)
        {
            PrefixHandlers[target] = handler;
        }
        else
        {
            PostfixHandlers[target] = handler;
        }

        ResultWritable[target] = withResult;
        Log.Message($"[Rust] {(prefix ? "prefix" : "postfix")} patch on {Describe(target)} -> handler '{handler}'");
    }

    /// <summary>Whether Harmony can be used at all (one lazy attempt, with a warning when not).</summary>
    public static bool IsAvailable => Ensure();

    /// <summary>
    /// Installs one of the bridge's *own* postfixes: a C# method of this assembly, not a Rust
    /// handler. Used by <see cref="GameEvents"/>, which turns the game's own moments into events.
    ///
    /// Returns the reason instead of throwing, because those events are a convenience: a game update
    /// that moves the method must leave the mod running, just without that event.
    /// </summary>
    public static bool TryInstallPostfix(string typeName, string methodName, string[] paramTypes,
        MethodInfo postfix, out string error)
    {
        return TryInstall(prefix: false, typeName, methodName, paramTypes, postfix, out error);
    }

    /// <summary>
    /// Installs one of the bridge's own **prefixes**. Some events need one: `Frame.CompleteConstruction`
    /// despawns the frame while it works, so only a prefix can still see where the construction happened.
    /// </summary>
    public static bool TryInstallPrefix(string typeName, string methodName, string[] paramTypes,
        MethodInfo prefix, out string error)
    {
        return TryInstall(prefix: true, typeName, methodName, paramTypes, prefix, out error);
    }

    private static bool TryInstall(bool prefix, string typeName, string methodName, string[] paramTypes,
        MethodInfo handler, out string error)
    {
        error = null;
        if (!Ensure())
        {
            error = "Harmony is not available";
            return false;
        }

        MethodBase target;
        try
        {
            target = FindTarget(typeName, methodName, paramTypes);
        }
        catch (BridgeException failure)
        {
            error = failure.Message;
            return false;
        }

        try
        {
            object patchInfo = Activator.CreateInstance(harmonyMethodType, handler);
            patchMethod.Invoke(harmony, prefix
                ? new object[] { target, patchInfo, null, null, null }
                : new object[] { target, null, patchInfo, null, null });
        }
        catch (Exception failure)
        {
            Exception actual = failure is TargetInvocationException { InnerException: not null } invocation
                ? invocation.InnerException
                : failure;
            error = actual.Message;
            return false;
        }

        Log.Message($"[Rust] game event {(prefix ? "prefix" : "postfix")} on {Describe(target)}");
        return true;
    }

    public static string Describe(MethodBase method)
    {
        string parameters = string.Join(", ", method.GetParameters().Select(p => p.ParameterType.Name));
        return $"{method.DeclaringType?.FullName}.{method.Name}({parameters})";
    }

    /// <summary>
    /// Takes every patch this bridge put on one target away through the game's own `Harmony.Unpatch` and
    /// answers with what is left on the target. Other mods' patches are left alone: unpatching code the
    /// mod does not own is not this bridge's business.
    /// </summary>
    public static object Unpatch(string typeName, string methodName, string[] paramTypes)
    {
        if (!Ensure())
        {
            throw new BridgeException("Harmony is not available");
        }

        MethodBase target = FindTarget(typeName, methodName, paramTypes);
        MethodInfo unpatch = harmony.GetType()
            .GetMethods(BindingFlags.Public | BindingFlags.Instance)
            .FirstOrDefault(candidate => candidate.Name == "Unpatch"
                && candidate.GetParameters().Length == 3
                && candidate.GetParameters()[0].ParameterType == typeof(MethodBase));
        if (unpatch == null)
        {
            throw new BridgeException("this Harmony version has no Unpatch(MethodBase, HarmonyPatchType, string)");
        }

        Type patchType = FindType("HarmonyLib.HarmonyPatchType");
        if (patchType == null)
        {
            throw new BridgeException("this Harmony version has no HarmonyLib.HarmonyPatchType");
        }

        object everyKind = Enum.Parse(patchType, "All");
        try
        {
            unpatch.Invoke(harmony, new[] { (object)target, everyKind, harmonyId });
        }
        catch (Exception failure)
        {
            Exception actual = failure is TargetInvocationException { InnerException: not null } invocation
                ? invocation.InnerException
                : failure;
            throw new BridgeException($"unpatching {Describe(target)} failed: {actual.Message}");
        }

        PrefixHandlers.Remove(target);
        PostfixHandlers.Remove(target);
        ResultWritable.Remove(target);
        // The redirect registry owns the plans, not Harmony: without this the next harmony_redirect on the target
        // would see a plan, skip installing its transpiler, and report success while nothing rewrites the call.
        Redirects.Forget(target);
        Log.Message($"[Rust] the bridges own patches on {Describe(target)} were taken off");
        return new Dictionary<string, object>
        {
            ["target"] = Describe(target),
            ["harmony"] = harmonyId,
            ["left"] = PatchInfo(target),
        };
    }

    /// <summary>
    /// What this bridge has patched, plus what Harmony itself reports for those targets (kind, owner and
    /// priority), so a mod can see what it has done and what else is running on the same methods.
    /// </summary>
    public static object List()
    {
        var mine = new List<Dictionary<string, object>>();
        foreach (MethodBase target in PrefixHandlers.Keys.Concat(PostfixHandlers.Keys).Distinct())
        {
            mine.Add(new Dictionary<string, object>
            {
                ["target"] = Describe(target),
                ["prefix"] = PrefixHandlers.TryGetValue(target, out string prefix) ? prefix : null,
                ["postfix"] = PostfixHandlers.TryGetValue(target, out string postfix) ? postfix : null,
                ["result"] = ResultWritable.TryGetValue(target, out bool writable) && writable,
            });
        }

        var running = new List<Dictionary<string, object>>();
        if (Ensure())
        {
            foreach (Dictionary<string, object> patch in PatchInfo(null))
            {
                running.Add(patch);
            }
        }

        return new Dictionary<string, object>
        {
            ["mine"] = mine,
            ["running"] = running,
            ["harmony"] = harmonyId,
            ["skippedWhileBusy"] = Redirects.SkippedWhileBusy,
        };
    }

    /// <summary>
    /// Every prefix, postfix, transpiler and finalizer Harmony reports for a target (all of them when the
    /// target is null), with the owner and the priority.
    /// </summary>
    private static List<Dictionary<string, object>> PatchInfo(MethodBase target)
    {
        var patches = new List<Dictionary<string, object>>();
        var targets = new List<MethodBase>();
        if (target != null)
        {
            targets.Add(target);
        }
        else if (CallHarmony("GetAllPatchedMethods") is System.Collections.IEnumerable methods)
        {
            foreach (object method in methods)
            {
                if (method is MethodBase typed)
                {
                    targets.Add(typed);
                }
            }
        }

        foreach (MethodBase method in targets)
        {
            object info = CallHarmony("GetPatchInfo", method);
            if (info == null)
            {
                continue;
            }

            // Harmony 3.0 exposes one property here: `Owners`. The per-kind patch objects (and their
            // priorities) are not public any more, and the game said so (`it offers Owners`).
            if (!(info.GetType().GetProperty("Owners")?.GetValue(info) is System.Collections.IEnumerable owners))
            {
                continue;
            }

            foreach (object owner in owners)
            {
                patches.Add(new Dictionary<string, object>
                {
                    ["target"] = Describe(method),
                    ["kind"] = "Owner",
                    ["owner"] = owner?.ToString(),
                    ["priority"] = null,
                    ["method"] = null,
                });
            }
        }

        return patches;
    }

    /// <summary>
    /// Changes the priority of one patch on a target — found by kind and owner, exactly the way
    /// `harmony_patches` reports it — and answers with the value Harmony kept. The write goes through the
    /// field (or a writable property) Harmony exposes on its own `Patch` object, so a version that keeps
    /// the priority to itself is refused by name instead of pretending the change happened.
    /// </summary>
    public static object SetPriority(
        string typeName,
        string methodName,
        string[] paramTypes,
        string kind,
        string owner,
        int priority)
    {
        if (!Ensure())
        {
            throw new BridgeException("Harmony is not available");
        }

        // The patched method is found the way `harmony_patches` lists it, not by resolving the name again:
        // the game proved the two routes disagree about which MethodBase carries the patches
        // (`FindTarget` + `GetPatchInfo` answered "carries no patches" while the listing showed the patch).
        // Harmony 3.0 keeps the patch objects to itself: `HarmonyLib.Patches` offers `Owners` and nothing
        // else (the game named it), so a priority cannot be written from here — and this op says why
        // instead of pretending.
        string known = string.Join(", ", (CallHarmony("GetAllPatchedMethods") as System.Collections.IEnumerable)
            ?.OfType<MethodBase>()
            .Select(Describe)
            .Take(8) ?? Enumerable.Empty<string>());
        throw new BridgeException(
            $"a patch priority cannot be written: this Harmony's patch info (`HarmonyLib.Patches`) exposes only `Owners`, so the patch objects and their priorities are not public. {Describe(FindTarget(typeName, methodName, paramTypes))} carries patches from: {(known.Length > 0 ? known : "nothing the listing can see")}");
    }

    /// <summary>
    /// Calls one of Harmony's own inspection methods by name. The search covers static and instance,
    /// public and not, and the argument list is built from the parameter types (a bool gets `true`), so the
    /// call does not depend on guessing which shape this Harmony ships — the game listed `GetPatchInfo` and
    /// `GetAllPatchedMethods`, and both shapes differ between versions.
    /// </summary>
    private static object CallHarmony(string name, params object[] leading)
    {
        foreach (MethodInfo candidate in harmony.GetType()
                     .GetMethods(BindingFlags.Public | BindingFlags.NonPublic | BindingFlags.Instance | BindingFlags.Static)
                     .Where(candidate => candidate.Name == name))
        {
            ParameterInfo[] parameters = candidate.GetParameters();
            if (parameters.Length < leading.Length)
            {
                continue;
            }

            var args = new object[parameters.Length];
            for (int index = 0; index < parameters.Length; index++)
            {
                args[index] = index < leading.Length
                    ? leading[index]
                    : parameters[index].ParameterType == typeof(bool)
                        ? (object)true
                        : null;
            }

            try
            {
                return candidate.Invoke(candidate.IsStatic ? null : harmony, args);
            }
            catch (TargetInvocationException)
            {
                // Another overload of the same name may fit; keep walking.
            }
            catch (ArgumentException)
            {
            }
        }

        return null;
    }

    private static bool Ensure()
    {
        if (attempted)
        {
            return available;
        }

        attempted = true;
        Type type = FindType("HarmonyLib.Harmony");
        if (type == null)
        {
            Log.Warning("[Rust] Harmony is not loaded (the brrainz.harmony mod is not active); harmony_patch is disabled");
            return false;
        }

        harmonyMethodType = FindType("HarmonyLib.HarmonyMethod");
        if (harmonyMethodType == null)
        {
            Log.Warning("[Rust] HarmonyLib.HarmonyMethod was not found; harmony_patch is disabled");
            return false;
        }

        patchMethod = type.GetMethods(BindingFlags.Public | BindingFlags.Instance)
            .FirstOrDefault(m => m.Name == "Patch"
                && m.GetParameters().Length == 5
                && m.GetParameters()[0].ParameterType == typeof(MethodBase));
        if (patchMethod == null)
        {
            Log.Warning("[Rust] this Harmony version has no Patch(MethodBase, HarmonyMethod x4); harmony_patch is disabled");
            return false;
        }

        string id = "rimworld-rs." + (RustMod.Current?.GetType().Assembly.GetName().Name ?? "mod");
        harmonyId = id;
        try
        {
            harmony = Activator.CreateInstance(type, new object[] { id });
        }
        catch (Exception)
        {
            harmony = Activator.CreateInstance(type);
        }

        available = harmony != null;
        return available;
    }

    /// <summary>
    /// Sets the ordering options a mod may pass: `priority` (higher runs first) and `before`/`after`
    /// (method names of other patches). Older Harmony versions without the member get a warning
    /// instead of a silent no-op.
    /// </summary>
    private static void ApplyOptions(object patchInfo, int priority, string before, string after)
    {
        if (priority >= 0)
        {
            SetMember(patchInfo, "priority", priority);
        }

        if (!string.IsNullOrEmpty(before))
        {
            SetMember(patchInfo, "before", new[] { before });
        }

        if (!string.IsNullOrEmpty(after))
        {
            SetMember(patchInfo, "after", new[] { after });
        }
    }

    private static void SetMember(object target, string member, object value)
    {
        Type type = target.GetType();
        PropertyInfo property = type.GetProperty(member, BindingFlags.Public | BindingFlags.Instance);
        if (property != null)
        {
            property.SetValue(target, value);
            return;
        }

        FieldInfo field = type.GetField(member, BindingFlags.Public | BindingFlags.Instance);
        if (field != null)
        {
            field.SetValue(target, value);
            return;
        }

        Log.Warning($"[Rust] this Harmony version has no '{member}' on HarmonyMethod; that patch option is ignored");
    }

    private static bool TryApply(MethodBase target, bool prefix, bool withResult, out Exception error,
        int priority = -1, string before = null, string after = null)
    {
        error = null;
        MethodInfo trampoline = SelectTrampoline(prefix, withResult);
        try
        {
            object patchInfo = Activator.CreateInstance(harmonyMethodType, trampoline);
            ApplyOptions(patchInfo, priority, before, after);
            object[] arguments = prefix
                ? new object[] { target, patchInfo, null, null, null }
                : new object[] { target, null, patchInfo, null, null };
            patchMethod.Invoke(harmony, arguments);
            return true;
        }
        catch (Exception ex)
        {
            error = ex is TargetInvocationException { InnerException: not null } invocation ? invocation.InnerException : ex;
            return false;
        }
    }

    private static MethodInfo SelectTrampoline(bool prefix, bool withResult)
    {
        string name = prefix
            ? (withResult ? nameof(RustPatches.PrefixWithResult) : nameof(RustPatches.PrefixNoResult))
            : (withResult ? nameof(RustPatches.PostfixWithResult) : nameof(RustPatches.PostfixNoResult));
        return typeof(RustPatches).GetMethod(name, BindingFlags.Public | BindingFlags.Static);
    }

    private static Type FindType(string fullName)
    {
        Type type = Type.GetType(fullName);
        if (type != null)
        {
            return type;
        }

        foreach (Assembly assembly in AppDomain.CurrentDomain.GetAssemblies())
        {
            try
            {
                type = assembly.GetType(fullName, throwOnError: false);
            }
            catch (Exception)
            {
                continue;
            }

            if (type != null)
            {
                return type;
            }
        }

        return null;
    }

    private static MethodBase FindTarget(string typeName, string methodName, string[] paramTypes)
    {
        // Constructors carry the metadata name `.ctor`, and this layer resolves members by their declared name
        // only, so the lookup ends in "method not found" and reads like a typo. Say what it is instead.
        if (methodName.StartsWith(".ctor", StringComparison.Ordinal) || methodName.StartsWith(".cctor", StringComparison.Ordinal))
        {
            throw new BridgeException(
                $"{methodName} is a constructor: harmony_patch resolves members by their declared name, so a constructor cannot be named (patch a member of the type, or listen to an event instead)");
        }

        Type type = JsonValueBridge.ResolveType(typeName);
        if (type == null)
        {
            throw new BridgeException(
                $"type not found: {typeName}{MethodSelection.SuggestTypes(AppDomain.CurrentDomain.GetAssemblies(), typeName)}");
        }

        return MethodSelection.Select(type, typeName, methodName, paramTypes);
    }
}

/// <summary>
/// Every trampoline the bridge registers with Harmony. The magic parameter names are injected by
/// Harmony; <c>__args</c> carries (and writes back) the original arguments, <c>__result</c> the
/// return value where the target and the Harmony version allow it.
/// </summary>
public static class RustPatches
{
    public static bool PrefixWithResult(MethodBase __originalMethod, object __instance, object[] __args, ref object __result)
    {
        return PatchDispatch.RunPrefix(__originalMethod, __instance, __args, resultAvailable: true, ref __result);
    }

    public static bool PrefixNoResult(MethodBase __originalMethod, object __instance, object[] __args)
    {
        object unused = null;
        return PatchDispatch.RunPrefix(__originalMethod, __instance, __args, resultAvailable: false, ref unused);
    }

    public static void PostfixWithResult(MethodBase __originalMethod, object __instance, object[] __args, ref object __result)
    {
        PatchDispatch.RunPostfix(__originalMethod, __instance, __args, resultAvailable: true, ref __result);
    }

    public static void PostfixNoResult(MethodBase __originalMethod, object __instance, object[] __args)
    {
        object unused = null;
        PatchDispatch.RunPostfix(__originalMethod, __instance, __args, resultAvailable: false, ref unused);
    }
}
