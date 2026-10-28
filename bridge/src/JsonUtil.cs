using System;
using System.Collections.Generic;
using System.Globalization;
using System.Reflection;
using System.Runtime.CompilerServices;
using System.Text.Json;
using System.Threading;

namespace RustBridge;

/// <summary>Raised for protocol level problems; the message is reported back to Rust.</summary>
internal sealed class BridgeException : Exception
{
    public BridgeException()
    {
    }

    public BridgeException(string message) : base(message)
    {
    }

    public BridgeException(string message, Exception innerException) : base(message, innerException)
    {
    }
}

/// <summary>
/// Registry of live managed objects that Rust refers to by integer handle. A table lives for
/// the duration of one dispatch (event or patch call) and its handles become invalid afterwards.
/// </summary>
internal sealed class RefTable
{
    private sealed class ReferenceComparer : IEqualityComparer<object>
    {
        public static readonly ReferenceComparer Instance = new();

        public new bool Equals(object a, object b) => ReferenceEquals(a, b);

        public int GetHashCode(object value) => RuntimeHelpers.GetHashCode(value);
    }

    private static int nextId;
    private readonly Dictionary<object, int> ids = new(ReferenceComparer.Instance);
    private readonly Dictionary<int, object> items = new();

    public int Add(object value)
    {
        if (ids.TryGetValue(value, out int existing))
        {
            return existing;
        }

        // The id is unique for the process, not just for this table: a handle kept past its
        // dispatch must not resolve to an unrelated object of the next table.
        int id = Interlocked.Increment(ref nextId);
        items.Add(id, value);
        ids[value] = id;
        return id;
    }

    public object Get(int id)
    {
        if (!items.TryGetValue(id, out object value))
        {
            throw new BridgeException($"object reference {id} is not valid anymore");
        }

        return value;
    }

    public void Clear()
    {
        items.Clear();
        ids.Clear();
    }
}

/// <summary>
/// Thread local scope that owns the <see cref="RefTable"/> for the currently executing dispatch.
/// Nested dispatches without a handle table keep the outer table alive; a nested dispatch that
/// brings its own table is refused, because its $ref numbers belong to that table. A patch call
/// that a host call triggers from inside a Rust handler is not dispatched while the Rust side is
/// busy: it is queued instead (see <see cref="RustHost.QueueNestedPatch"/>).
/// </summary>
internal static class RefContext
{
    [ThreadStatic] private static RefTable current;
    [ThreadStatic] private static int depth;

    /// <summary>True while a dispatch is running on this thread.</summary>
    public static bool IsDispatching => depth > 0;

    public static RefTable Current => current ??= new RefTable();

    /// <summary>Enters a fresh dispatch; the handles of the previous one become invalid.</summary>
    public static IDisposable Enter()
    {
        return Enter(null);
    }

    /// <summary>Enters a dispatch, optionally reusing the table of a queued patch call.</summary>
    public static IDisposable Enter(RefTable table)
    {
        if (depth > 0)
        {
            if (table != null)
            {
                // The payload of the nested event was built against its own table, so resolving its
                // $ref numbers against the outer table would reach the wrong objects. Queue it.
                throw new BridgeException(
                    "a handle-carrying dispatch cannot start while another dispatch is running; queue the event instead");
            }

            depth++;
            return new Scope();
        }

        depth = 1;
        current = table ?? new RefTable();
        return new Scope();
    }

    private sealed class Scope : IDisposable
    {
        public void Dispose()
        {
            if (--depth == 0)
            {
                current = null; // handles are invalid from here on
            }
        }
    }
}

/// <summary>Converts between CLR values and the JSON wire format shared with the Rust SDK.</summary>
internal static class JsonValueBridge
{
    /// <summary>
    /// Optional hook, installed by the bridge, that renders game types which behave like strings
    /// (for example <c>Verse.TaggedString</c>, returned by most labels and descriptions) as JSON
    /// strings instead of object handles. Keep this layer free of game references.
    /// </summary>
    public static Func<object, string> StringLikeConverter = null;

    /// <summary>Optional hook that converts a JSON string into such a game type on the way back.</summary>
    public static Func<string, Type, object> StringLikeParser = null;

    /// <summary>
    /// Optional hook that builds a value type out of an inline JSON object, for example <c>Verse.IntVec3</c> from
    /// <c>{"x":1,"y":0,"z":2}</c>. Without it an inline object can only be a handle, and a handle cannot be
    /// assigned to a value type - which is exactly what a patch handler does when it rewrites a struct argument
    /// (or a struct member) in the game.
    /// </summary>
    public static Func<JsonElement, Type, object> StructParser = null;

    public static void Write(Utf8JsonWriter writer, object value, RefTable refs)
    {
        if (value != null && StringLikeConverter != null && StringLikeConverter(value) is string stringLike)
        {
            writer.WriteStringValue(stringLike);
            return;
        }

        switch (value)
        {
            case null:
                writer.WriteNullValue();
                return;
            case bool b:
                writer.WriteBooleanValue(b);
                return;
            case string s:
                writer.WriteStringValue(s);
                return;
            case char c:
                writer.WriteStringValue(c.ToString());
                return;
            case byte x:
                writer.WriteNumberValue(x);
                return;
            case sbyte x:
                writer.WriteNumberValue(x);
                return;
            case short x:
                writer.WriteNumberValue(x);
                return;
            case ushort x:
                writer.WriteNumberValue(x);
                return;
            case int x:
                writer.WriteNumberValue(x);
                return;
            case uint x:
                writer.WriteNumberValue(x);
                return;
            case long x:
                writer.WriteNumberValue(x);
                return;
            case ulong x:
                writer.WriteNumberValue(x);
                return;
            case float x:
                writer.WriteNumberValue(x);
                return;
            case double x:
                writer.WriteNumberValue(x);
                return;
            case decimal x:
                writer.WriteNumberValue(x);
                return;
            case Enum e:
                writer.WriteStringValue(e.ToString());
                return;
            case System.Collections.IDictionary dictionary:
                // Structured results (game_info, def_info, map info, ...) are dictionaries and lists;
                // they must become real JSON, not object handles.
                writer.WriteStartObject();
                foreach (System.Collections.DictionaryEntry entry in dictionary)
                {
                    writer.WritePropertyName(Convert.ToString(entry.Key, CultureInfo.InvariantCulture) ?? string.Empty);
                    Write(writer, entry.Value, refs);
                }

                writer.WriteEndObject();
                return;
            case System.Collections.IEnumerable enumerable:
                writer.WriteStartArray();
                foreach (object item in enumerable)
                {
                    Write(writer, item, refs);
                }

                writer.WriteEndArray();
                return;
        }

        writer.WriteStartObject();
        writer.WriteNumber("$ref", refs.Add(value));
        writer.WriteString("$type", value.GetType().FullName);
        string text = Describe(value);
        if (text != null)
        {
            writer.WriteString("$str", text);
        }

        writer.WriteEndObject();
    }

    public static object Read(JsonElement element, Type target, RefTable refs)
    {
        Type nullable = Nullable.GetUnderlyingType(target);
        if (element.ValueKind == JsonValueKind.Null)
        {
            if (target.IsValueType && nullable == null)
            {
                throw new BridgeException($"null cannot be assigned to {target.Name}");
            }

            return null;
        }

        if (nullable != null)
        {
            target = nullable;
        }

        switch (element.ValueKind)
        {
            case JsonValueKind.True:
            case JsonValueKind.False:
                return Convert.ChangeType(element.GetBoolean(), target, CultureInfo.InvariantCulture);
            case JsonValueKind.Number:
                if (target == typeof(object))
                {
                    return element.TryGetInt64(out long asLong) ? asLong : element.GetDouble();
                }

                if (target.IsEnum)
                {
                    return Enum.ToObject(target, element.TryGetInt64(out long raw) ? raw : (long)element.GetDouble());
                }

                return Convert.ChangeType(
                    element.TryGetInt64(out long integer) ? integer : element.GetDouble(),
                    target,
                    CultureInfo.InvariantCulture);
            case JsonValueKind.String:
                return ReadString(element.GetString(), target);
            case JsonValueKind.Object:
                if (element.TryGetProperty("$ref", out JsonElement reference))
                {
                    int id = reference.GetInt32();
                    object target0 = refs.Get(id);
                    if (target != typeof(object) && !target.IsInstanceOfType(target0))
                    {
                        throw new BridgeException(
                            $"object reference {id} holds {target0.GetType().Name}, not {target.Name}");
                    }

                    return target0;
                }

                if (StructParser != null && StructParser(element, target) is object parsed)
                {
                    return parsed;
                }

                throw new BridgeException(
                    $"an inline JSON object cannot become {target.Name}: only a handle, a string-like value or a struct the bridge builds can arrive inline");
            case JsonValueKind.Array:
                if (target.IsArray)
                {
                    Type elementType = target.GetElementType();
                    var values = new List<object>();
                    foreach (JsonElement item in element.EnumerateArray())
                    {
                        values.Add(Read(item, elementType, refs));
                    }

                    Array array = Array.CreateInstance(elementType, values.Count);
                    for (int i = 0; i < values.Count; i++)
                    {
                        array.SetValue(values[i], i);
                    }

                    return array;
                }

                throw new BridgeException($"a JSON array cannot be converted to {target.Name} yet");
            default:
                throw new BridgeException($"unsupported JSON value kind {element.ValueKind}");
        }
    }

    public static Type ResolveType(string name)
    {
        if (string.IsNullOrEmpty(name))
        {
            return null;
        }

        Type found = Type.GetType(name);
        if (found != null)
        {
            return found;
        }

        foreach (Assembly assembly in AppDomain.CurrentDomain.GetAssemblies())
        {
            try
            {
                found = assembly.GetType(name);
            }
            catch (Exception)
            {
                continue;
            }

            if (found != null)
            {
                return found;
            }
        }

        return null;
    }

    private static object ReadString(string text, Type target)
    {
        if (target == typeof(string) || target == typeof(object))
        {
            return text;
        }

        if (StringLikeParser != null && StringLikeParser(text, target) is object stringLike)
        {
            return stringLike;
        }

        if (target == typeof(char))
        {
            if (string.IsNullOrEmpty(text))
            {
                throw new BridgeException("an empty string cannot be a char");
            }

            return text[0];
        }

        if (target == typeof(Type))
        {
            return ResolveType(text) ?? throw new BridgeException($"type not found: {text}");
        }

        if (target == typeof(Guid))
        {
            return Guid.Parse(text);
        }

        if (target.IsEnum)
        {
            return Enum.Parse(target, text, ignoreCase: true);
        }

        return Convert.ChangeType(text, target, CultureInfo.InvariantCulture);
    }

    private static string Describe(object value)
    {
        try
        {
            return Convert.ToString(value, CultureInfo.InvariantCulture);
        }
        catch (Exception)
        {
            return null;
        }
    }
}

/// <summary>Small helpers for reading request properties.</summary>
internal static class JsonArgs
{
    public static string GetString(JsonElement request, string name)
    {
        if (!request.TryGetProperty(name, out JsonElement value) || value.ValueKind != JsonValueKind.String)
        {
            throw new BridgeException($"missing string field '{name}'");
        }

        return value.GetString();
    }

    public static string GetStringOrNull(JsonElement request, string name)
    {
        return request.TryGetProperty(name, out JsonElement value) && value.ValueKind == JsonValueKind.String
            ? value.GetString()
            : null;
    }

    public static int GetInt(JsonElement request, string name, int fallback)
    {
        if (request.TryGetProperty(name, out JsonElement value) && value.ValueKind == JsonValueKind.Number)
        {
            return value.GetInt32();
        }

        return fallback;
    }

    public static double GetDouble(JsonElement request, string name, double fallback)
    {
        if (request.TryGetProperty(name, out JsonElement value) && value.ValueKind == JsonValueKind.Number)
        {
            return value.GetDouble();
        }

        return fallback;
    }

    public static bool GetBool(JsonElement request, string name)
    {
        return request.TryGetProperty(name, out JsonElement value) && value.ValueKind == JsonValueKind.True;
    }

    public static bool TryGet(JsonElement request, string name, out JsonElement value)
    {
        return request.TryGetProperty(name, out value);
    }

    public static string[] GetStringArray(JsonElement request, string name)
    {
        if (!request.TryGetProperty(name, out JsonElement value) || value.ValueKind != JsonValueKind.Array)
        {
            return null;
        }

        var result = new List<string>();
        foreach (JsonElement item in value.EnumerateArray())
        {
            result.Add(item.GetString());
        }

        return result.ToArray();
    }

    public static object[] ReadArray(JsonElement request, string name, RefTable refs)
    {
        if (!request.TryGetProperty(name, out JsonElement value) || value.ValueKind != JsonValueKind.Array)
        {
            return Array.Empty<object>();
        }

        var result = new List<object>();
        foreach (JsonElement item in value.EnumerateArray())
        {
            result.Add(JsonValueBridge.Read(item, typeof(object), refs));
        }

        return result.ToArray();
    }
}
