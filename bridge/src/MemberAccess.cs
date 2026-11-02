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
/// (<c>Verse.ThingWithComps</c>). <see cref="MemberCache"/> remembers what the walk found, so a member is
/// resolved once per type instead of once per access.
/// </summary>
internal static class MemberAccess
{
    public static object Get(object target, string member)
    {
        Type type = target is Type self ? self : target.GetType();
        object instance = target is Type ? null : target;

        MemberCache.Slot slot = MemberCache.ReadableSlot(type, member);
        if (slot != null)
        {
            return Read(slot, instance);
        }

        // A handle to a *type* also answers the members System.Type itself offers (FullName, Name,
        // IsEnum, ...): the members above are the statics of the represented type.
        if (target is Type represented)
        {
            MemberCache.Slot own = MemberCache.ReadableSlot(typeof(Type), member);
            if (own != null)
            {
                return Read(own, represented);
            }
        }

        throw new BridgeException($"{type.Name} has no readable member or method '{member}'{Suggest(type, member)}");
    }

    private static object Read(MemberCache.Slot slot, object instance)
    {
        if (slot.Field != null)
        {
            return slot.Field.GetValue(instance);
        }

        if (slot.Property != null)
        {
            return slot.Property.GetValue(instance, null);
        }

        return slot.Method.Invoke(instance, Array.Empty<object>());
    }

    public static void Set(object target, string member, JsonElement value, RefTable refs)
    {
        Type type = target is Type self ? self : target.GetType();
        object instance = target is Type ? null : target;

        MemberCache.Slot slot = MemberCache.WritableSlot(type, member);
        if (slot == null)
        {
            throw new BridgeException($"{type.Name} has no writable member '{member}'{Suggest(type, member)}");
        }

        if (slot.Field != null)
        {
            slot.Field.SetValue(instance, JsonValueBridge.Read(value, slot.Field.FieldType, refs));
            return;
        }

        slot.Property.SetValue(instance, JsonValueBridge.Read(value, slot.Property.PropertyType, refs), null);
    }

    public static object Call(object target, string method, object[] args)
    {
        Type type = target is Type self ? self : target.GetType();
        object instance = target is Type ? null : target;

        List<MethodInfo> candidates = MemberCache.CallCandidates(type, method, args.Length);
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
        foreach (Type level in MemberCache.Hierarchy(type))
        {
            foreach (FieldInfo field in level.GetFields(MemberCache.Declared))
            {
                if (matches(field.Name))
                {
                    suggestions.Add(field.Name);
                }
            }

            foreach (PropertyInfo property in level.GetProperties(MemberCache.Declared))
            {
                if (matches(property.Name))
                {
                    suggestions.Add(property.Name);
                }
            }

            foreach (MethodInfo method in level.GetMethods(MemberCache.Declared))
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

    /// <summary>
    /// The declared members of one type, in the order the lookup walks them (the type itself, then every
    /// base type). Exposed so the bridge can *show* a type's surface (`type_members`) and *prove* the
    /// generic route reaches all of it (`member_scan`) instead of claiming it.
    /// </summary>
    public static IEnumerable<(Type Level, MemberInfo Member)> DeclaredMembers(Type type)
    {
        foreach (Type level in MemberCache.Hierarchy(type))
        {
            foreach (FieldInfo field in level.GetFields(MemberCache.Declared))
            {
                yield return (level, field);
            }

            foreach (PropertyInfo property in level.GetProperties(MemberCache.Declared))
            {
                yield return (level, property);
            }

            foreach (MethodInfo method in level.GetMethods(MemberCache.Declared))
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
                return level.GetField(field.Name, MemberCache.Declared) != null;
            case PropertyInfo property:
                return MemberCache.FindProperty(level, property.Name) != null;
            case MethodInfo method:
                // The exact-arity call is the one a mod can always make (defaults only make fewer
                // arguments possible, they are not needed).
                return MemberCache.FindMethod(level, method.Name, method.GetParameters().Length) != null;
            default:
                return false;
        }
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
