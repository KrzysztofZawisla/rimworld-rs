using System;
using System.Collections.Generic;
using System.Globalization;
using System.Linq;
using System.Reflection;
using System.Text.Json;

namespace RustBridge;

/// <summary>
/// Reflection helpers for the <c>ref_get</c>, <c>ref_set</c> and <c>ref_call</c> operations.
///
/// Members are looked up level by level, from the concrete type up through its base types: private
/// fields, properties and methods declared on a base class (for example <c>Verse.Thing</c>) are not
/// returned by a lookup on the derived type the game actually instantiated
/// (<c>Verse.ThingWithComps</c>).
/// </summary>
internal static class MemberAccess
{
    private const BindingFlags Declared =
        BindingFlags.Public | BindingFlags.NonPublic | BindingFlags.Instance | BindingFlags.Static | BindingFlags.DeclaredOnly;

    public static object Get(object target, string member)
    {
        Type type = target is Type self ? self : target.GetType();
        object instance = target is Type ? null : target;

        foreach (Type level in Hierarchy(type))
        {
            FieldInfo field = level.GetField(member, Declared);
            if (field != null)
            {
                return field.GetValue(instance);
            }

            PropertyInfo property = FindProperty(level, member);
            if (property != null && property.CanRead && property.GetIndexParameters().Length == 0)
            {
                return property.GetValue(instance, null);
            }

            MethodInfo method = FindMethod(level, member, 0);
            if (method != null)
            {
                return method.Invoke(instance, Array.Empty<object>());
            }
        }

        // A handle to a *type* also answers the members System.Type itself offers (FullName, Name,
        // IsEnum, ...): the members above are the statics of the represented type.
        if (target is Type represented)
        {
            foreach (Type level in Hierarchy(typeof(Type)))
            {
                FieldInfo field = level.GetField(member, Declared);
                if (field != null)
                {
                    return field.GetValue(represented);
                }

                PropertyInfo property = FindProperty(level, member);
                if (property != null && property.CanRead && property.GetIndexParameters().Length == 0)
                {
                    return property.GetValue(represented, null);
                }

                MethodInfo method = FindMethod(level, member, 0);
                if (method != null)
                {
                    return method.Invoke(represented, Array.Empty<object>());
                }
            }
        }

        throw new BridgeException($"{type.Name} has no readable member or method '{member}'{Suggest(type, member)}");
    }

    /// <summary>
    /// One property by name, without letting reflection's ambiguity get in the way: a type that declares
    /// two indexers (`Item[Int32]` and `Item[Int32, Int32]`, `Verse.Area` has them) makes `GetProperty`
    /// throw `AmbiguousMatchException`, while the route only ever wants *a* member of that name.
    /// </summary>
    private static PropertyInfo FindProperty(Type level, string name)
    {
        try
        {
            return level.GetProperty(name, Declared);
        }
        catch (AmbiguousMatchException)
        {
            return level.GetProperties(Declared).FirstOrDefault(candidate => candidate.Name == name);
        }
    }

    public static void Set(object target, string member, JsonElement value, RefTable refs)
    {
        Type type = target is Type self ? self : target.GetType();
        object instance = target is Type ? null : target;

        foreach (Type level in Hierarchy(type))
        {
            FieldInfo field = level.GetField(member, Declared);
            if (field != null)
            {
                field.SetValue(instance, JsonValueBridge.Read(value, field.FieldType, refs));
                return;
            }

            PropertyInfo property = FindProperty(level, member);
            if (property != null && property.CanWrite && property.GetIndexParameters().Length == 0)
            {
                property.SetValue(instance, JsonValueBridge.Read(value, property.PropertyType, refs), null);
                return;
            }
        }

        throw new BridgeException($"{type.Name} has no writable member '{member}'{Suggest(type, member)}");
    }

    public static object Call(object target, string method, object[] args)
    {
        Type type = target is Type self ? self : target.GetType();
        object instance = target is Type ? null : target;

        var candidates = new List<MethodInfo>();
        foreach (Type level in Hierarchy(type))
        {
            foreach (MethodInfo candidate in level.GetMethods(Declared))
            {
                if (candidate.Name == method && CanAccept(candidate, args.Length))
                {
                    candidates.Add(candidate);
                }
            }
        }

        if (candidates.Count == 0)
        {
            throw new BridgeException(
                $"{type.Name} has no method '{method}' taking {args.Length} argument(s){Suggest(type, method)}");
        }

        Exception lastError = null;
        foreach (MethodInfo candidate in candidates)
        {
            try
            {
                return candidate.Invoke(instance, ConvertArguments(candidate, args));
            }
            catch (Exception ex)
            {
                lastError = ex;
            }
        }

        throw new BridgeException($"{type.Name}.{method} failed: {lastError?.Message}");
    }

    /// <summary>The type itself, then every base type (private members are not inherited).</summary>
    private static IEnumerable<Type> Hierarchy(Type type)
    {
        for (Type current = type; current != null; current = current.BaseType)
        {
            yield return current;
        }
    }

    /// <summary>
    /// Lists near matches, so a typo or a wrong casing ("destroyed" instead of "Destroyed") is
    /// visible in the error instead of requiring a decompiler.
    /// </summary>
    private static string Suggest(Type type, string member)
    {
        bool matches(string name) =>
            name.Contains(member, StringComparison.OrdinalIgnoreCase)
            || member.Contains(name, StringComparison.OrdinalIgnoreCase);

        var suggestions = new List<string>();
        foreach (Type level in Hierarchy(type))
        {
            foreach (FieldInfo field in level.GetFields(Declared))
            {
                if (matches(field.Name))
                {
                    suggestions.Add(field.Name);
                }
            }

            foreach (PropertyInfo property in level.GetProperties(Declared))
            {
                if (matches(property.Name))
                {
                    suggestions.Add(property.Name);
                }
            }

            foreach (MethodInfo method in level.GetMethods(Declared))
            {
                if (matches(method.Name))
                {
                    suggestions.Add(method.Name + "()");
                }
            }

            if (suggestions.Count >= 8)
            {
                break;
            }
        }

        return suggestions.Count == 0
            ? string.Empty
            : " (did you mean: " + string.Join(", ", suggestions.Take(8)) + "?)";
    }

    /// <summary>A method matches when it takes exactly the arguments, or can default the rest.</summary>
    private static bool CanAccept(MethodInfo method, int argumentCount)
    {
        ParameterInfo[] parameters = method.GetParameters();
        if (parameters.Length < argumentCount)
        {
            return false;
        }

        for (int i = argumentCount; i < parameters.Length; i++)
        {
            // A `ref`/`out` parameter has no default to fall back on: the caller has to name the slot.
            if (!parameters[i].HasDefaultValue || parameters[i].ParameterType.IsByRef)
            {
                return false;
            }
        }

        return true;
    }

    /// <summary>
    /// The declared members of one type, in the order the lookup walks them (the type itself, then every
    /// base type). Exposed so the bridge can *show* a type's surface (`type_members`) and *prove* the
    /// generic route reaches all of it (`member_scan`) instead of claiming it.
    /// </summary>
    public static IEnumerable<(Type Level, MemberInfo Member)> DeclaredMembers(Type type)
    {
        foreach (Type level in Hierarchy(type))
        {
            foreach (FieldInfo field in level.GetFields(Declared))
            {
                yield return (level, field);
            }

            foreach (PropertyInfo property in level.GetProperties(Declared))
            {
                yield return (level, property);
            }

            foreach (MethodInfo method in level.GetMethods(Declared))
            {
                yield return (level, method);
            }
        }
    }

    /// <summary>
    /// Whether <see cref="Get"/>, <see cref="Set"/> or <see cref="Call"/> can address one declared member
    /// by name — the very lookup those operations perform, so a member this refuses is a member the route
    /// cannot reach.
    /// </summary>
    public static bool CanAddress(Type level, MemberInfo member)
    {
        switch (member)
        {
            case FieldInfo field:
                return level.GetField(field.Name, Declared) != null;
            case PropertyInfo property:
                return FindProperty(level, property.Name) != null;
            case MethodInfo method:
                // The exact-arity call is the one a mod can always make (defaults only make fewer
                // arguments possible, they are not needed).
                return FindMethod(level, method.Name, method.GetParameters().Length) != null;
            default:
                return false;
        }
    }

    private static MethodInfo FindMethod(Type level, string name, int argCount)
    {
        foreach (MethodInfo candidate in level.GetMethods(Declared))
        {
            if (candidate.Name == name && CanAccept(candidate, argCount))
            {
                return candidate;
            }
        }

        return null;
    }

    internal static object[] ConvertArguments(MethodInfo method, object[] args)
    {
        ParameterInfo[] parameters = method.GetParameters();
        var converted = new object[parameters.Length];
        for (int i = 0; i < parameters.Length; i++)
        {
            if (i >= args.Length)
            {
                converted[i] = Type.Missing; // reflection fills in the parameter default
                continue;
            }

            converted[i] = ConvertArgument(args[i], parameters[i].ParameterType, method.Name, i);
        }

        return converted;
    }

    /// <summary>
    /// Converts one argument, including enums from strings or numbers. A `ref`/`out` parameter takes
    /// the value behind the reference: the boxed argument goes in, reflection copies the method's own
    /// write-back into the argument array, and nobody reads it — so `MakeSolidThing(ref bool)` and the
    /// rest of the game's by-ref API answer like any other method.
    /// </summary>
    internal static object ConvertArgument(object arg, Type target, string methodName, int index)
    {
        if (target.IsByRef)
        {
            target = target.GetElementType();
        }

        if (arg == null)
        {
            if (target.IsValueType && Nullable.GetUnderlyingType(target) == null)
            {
                throw new BridgeException($"argument {index} of {methodName} cannot be null");
            }

            return null;
        }

        if (target.IsInstanceOfType(arg))
        {
            return arg;
        }

        if (target.IsEnum)
        {
            if (arg is string text)
            {
                return Enum.Parse(target, text, ignoreCase: true);
            }

            if (arg is IConvertible)
            {
                return Enum.ToObject(target, Convert.ToInt64(arg, CultureInfo.InvariantCulture));
            }
        }

        if (arg is IConvertible && typeof(IConvertible).IsAssignableFrom(target))
        {
            return Convert.ChangeType(arg, target, CultureInfo.InvariantCulture);
        }

        throw new BridgeException($"argument {index} of {methodName} cannot take a {arg.GetType().Name}");
    }
}
