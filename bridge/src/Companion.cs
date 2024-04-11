using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Text.Json;
using Verse;

namespace RustBridge;

/// <summary>
/// The bridge between a Rust mod and its optional companion C# assembly: a second DLL in the
/// mod's <c>Assemblies</c> folder that may define managed game types (ThingComp, JobDriver,
/// ITab, Window, ...) which a native library cannot create.
///
/// Rust calls into the companion with the <c>companion_call</c> host operation; the companion
/// calls into Rust with <see cref="Send"/> and <see cref="Request"/>. Only types of this mod's own
/// assemblies are visible: other mods are never searched.
/// </summary>
public static class Companion
{
    /// <summary>Reports an event to the Rust mod and discards the answer.</summary>
    public static void Send(string source, string name, object payload = null)
    {
        Request(source, name, payload);
    }

    /// <summary>
    /// Reports an event to the Rust mod and returns its answer as text (empty when there is none).
    /// Live game objects inside <paramref name="payload"/> travel as handles, so Rust can read
    /// them with `ref_get` / `ref_call` while the event is being handled.
    /// </summary>
    public static string Request(string source, string name, object payload = null)
    {
        RustHost host = RustHost.Current;
        if (host == null)
        {
            Log.Error($"[Rust] companion event '{source}/{name}' arrived before the Rust library was ready");
            return string.Empty;
        }

        if (RefContext.IsDispatching)
        {
            // Rust is busy with another event: queue this one with the handles it needs.
            var queuedRefs = new RefTable();
            RustHost.QueueNestedEvent("companion_event", BuildPayload(source, name, payload, queuedRefs), queuedRefs);
            return string.Empty;
        }

        var refs = new RefTable();
        byte[] body = BuildPayload(source, name, payload, refs);
        using JsonDocument document = host.DispatchBytes("companion_event", body, refs);
        JsonElement root = document.RootElement;
        if (!(root.TryGetProperty("ok", out JsonElement ok) && ok.ValueKind == JsonValueKind.True))
        {
            string error = root.TryGetProperty("error", out JsonElement message) ? message.GetString() : "unknown error";
            Log.Error($"[Rust] companion event '{source}/{name}' failed: {error}");
            return string.Empty;
        }

        return root.TryGetProperty("result", out JsonElement result) && result.ValueKind == JsonValueKind.String
            ? result.GetString()
            : string.Empty;
    }

    /// <summary>Calls one member of the companion from Rust (the `companion_call` host operation).</summary>
    internal static object Invoke(JsonElement request)
    {
        string typeName = JsonArgs.GetString(request, "type");
        string memberName = JsonArgs.GetString(request, "member");
        Type type = ResolveType(typeName);

        var paramTypes = new List<string>();
        if (request.TryGetProperty("paramTypes", out JsonElement declared) && declared.ValueKind == JsonValueKind.Array)
        {
            foreach (JsonElement entry in declared.EnumerateArray())
            {
                if (entry.ValueKind == JsonValueKind.String)
                {
                    paramTypes.Add(entry.GetString());
                }
            }
        }

        object[] args = JsonArgs.ReadArray(request, "args", RefContext.Current);
        MethodInfo method;
        try
        {
            // The argument count picks the overload when the caller did not name parameter types.
            method = MethodSelection.Select(type, typeName, memberName, paramTypes.ToArray(), args.Length);
        }
        catch (BridgeException)
        {
            // Fields and properties of a companion type are reachable the same way.
            int handle = JsonArgs.GetInt(request, "ref", -1);
            if (handle >= 0)
            {
                return MemberAccess.Call(RefContext.Current.Get(handle), memberName, args);
            }

            object staticValue = type.GetField(memberName, BindingFlags.Public | BindingFlags.NonPublic | BindingFlags.Static)?.GetValue(null);
            if (staticValue != null)
            {
                return staticValue;
            }

            throw;
        }

        object instance = null;
        if (!method.IsStatic)
        {
            int handle = JsonArgs.GetInt(request, "ref", -1);
            if (handle < 0)
            {
                throw new BridgeException($"companion member {MethodSelection.Describe(method)} is not static; pass the object as 'ref'");
            }

            instance = RefContext.Current.Get(handle)
                ?? throw new BridgeException($"companion call needs a valid 'ref', got handle {handle}");
        }

        object[] converted = MemberAccess.ConvertArguments(method, args);
        try
        {
            return method.Invoke(instance, converted);
        }
        catch (TargetInvocationException error) when (error.InnerException != null)
        {
            throw new BridgeException($"companion member {MethodSelection.Describe(method)} threw: {error.InnerException.Message}");
        }
    }

    /// <summary>
    /// Finds a type in this mod's own assemblies: the bridge, the compiled Rust companion and
    /// everything else RimWorld loaded from this mod's Assemblies folder.
    /// </summary>
    internal static Type ResolveType(string typeName)
    {
        List<Assembly> assemblies = OwnAssemblies();
        foreach (Assembly assembly in assemblies)
        {
            Type type = assembly.GetType(typeName, throwOnError: false);
            if (type != null)
            {
                return type;
            }
        }

        // Shorthand: a unique type whose simple or full name ends with the requested name.
        var matches = new List<Type>();
        foreach (Assembly assembly in assemblies)
        {
            foreach (Type candidate in SafeTypes(assembly))
            {
                if (candidate.Name == typeName
                    || (candidate.FullName != null && candidate.FullName.EndsWith("." + typeName, StringComparison.Ordinal)))
                {
                    matches.Add(candidate);
                }
            }
        }

        if (matches.Count == 1)
        {
            return matches[0];
        }

        string suggestion = matches.Count > 1
            ? " (ambiguous: " + string.Join(", ", matches.Take(5).Select(type => type.FullName)) + ")"
            : MethodSelection.SuggestTypes(assemblies, typeName);
        throw new BridgeException($"companion type not found: {typeName}{suggestion}");
    }

    /// <summary>The assemblies that belong to this mod, bridge first.</summary>
    private static List<Assembly> OwnAssemblies()
    {
        var seen = new HashSet<Assembly>();
        var assemblies = new List<Assembly>();
        Assembly bridge = typeof(Companion).Assembly;
        seen.Add(bridge);
        assemblies.Add(bridge);

        List<Assembly> loaded = RustMod.Current?.Content?.assemblies?.loadedAssemblies;
        if (loaded != null)
        {
            foreach (Assembly assembly in loaded)
            {
                if (assembly != null && seen.Add(assembly))
                {
                    assemblies.Add(assembly);
                }
            }
        }

        return assemblies;
    }

    private static IEnumerable<Type> SafeTypes(Assembly assembly)
    {
        try
        {
            return assembly.GetTypes();
        }
        catch (ReflectionTypeLoadException error)
        {
            return error.Types.Where(type => type != null);
        }
        catch (Exception)
        {
            return Array.Empty<Type>();
        }
    }

    private static byte[] BuildPayload(string source, string name, object payload, RefTable refs)
    {
        using var stream = new MemoryStream();
        using (var writer = new Utf8JsonWriter(stream))
        {
            writer.WriteStartObject();
            writer.WriteString("source", source);
            writer.WriteString("name", name);
            writer.WritePropertyName("payload");
            JsonValueBridge.Write(writer, payload, refs);
            writer.WriteEndObject();
        }

        return stream.ToArray();
    }
}
