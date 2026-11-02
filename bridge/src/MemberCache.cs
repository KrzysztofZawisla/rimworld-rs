using System;
using System.Collections.Generic;
using System.Linq;
using System.Reflection;
using System.Runtime.CompilerServices;

namespace RustBridge;

/// <summary>
/// Memoized member resolution for <see cref="MemberAccess"/>.
///
/// Every <c>ref_get</c>, <c>ref_set</c> and <c>ref_call</c> used to walk the type's hierarchy with
/// reflection, and the generated wrappers read through that route on every access - so the same
/// <c>(type, member)</c> pair was resolved again and again. The metadata of a loaded type cannot change,
/// so the first answer is kept: the walk runs once per type and member, not once per call.
///
/// Entries hang off the search-start type through a <see cref="ConditionalWeakTable{TKey,TValue}"/>, so
/// they live exactly as long as the type does. The inner dictionaries are read without a lock because host
/// calls are serialized by the bridge's dispatch lock; the tables themselves are safe for concurrent
/// <c>GetOrCreateValue</c>.
///
/// A type that is asked for many names it does not have (a mod probing with data) stops being cached after
/// <see cref="MaxMembersPerType"/> entries and simply walks again - the cache is a speed-up, not a store.
/// </summary>
internal static class MemberCache
{
    /// <summary>Members declared on one type only; the walk adds each base type itself.</summary>
    internal const BindingFlags Declared =
        BindingFlags.Public | BindingFlags.NonPublic | BindingFlags.Instance | BindingFlags.Static | BindingFlags.DeclaredOnly;

    /// <summary>How many names one type remembers; past it the lookup runs the walk instead.</summary>
    private const int MaxMembersPerType = 512;

    private static readonly ConditionalWeakTable<Type, Dictionary<string, Slot>> Readable = new();
    private static readonly ConditionalWeakTable<Type, Dictionary<string, Slot>> Writable = new();
    private static readonly ConditionalWeakTable<Type, Dictionary<(string Name, int Arguments), List<MethodInfo>>> Callable = new();
    private static readonly ConditionalWeakTable<Type, Dictionary<string, List<MethodInfo>>> Named = new();

    /// <summary>One resolved member: exactly one of the three fields is set.</summary>
    internal sealed class Slot
    {
        public FieldInfo Field;

        public PropertyInfo Property;

        public MethodInfo Method;
    }

    /// <summary>
    /// The member <see cref="MemberAccess.Get"/> reads: the first field, readable property or
    /// parameterless method of that name, level by level through the hierarchy.
    /// </summary>
    public static Slot ReadableSlot(Type start, string member)
    {
        Dictionary<string, Slot> members = Readable.GetOrCreateValue(start);
        if (members.TryGetValue(member, out Slot slot))
        {
            return slot;
        }

        slot = FindReadable(start, member);
        Remember(members, member, slot);
        return slot;
    }

    /// <summary>
    /// The member <see cref="MemberAccess.Set"/> writes: the first field or writable property of that name,
    /// level by level through the hierarchy.
    /// </summary>
    public static Slot WritableSlot(Type start, string member)
    {
        Dictionary<string, Slot> members = Writable.GetOrCreateValue(start);
        if (members.TryGetValue(member, out Slot slot))
        {
            return slot;
        }

        slot = FindWritable(start, member);
        Remember(members, member, slot);
        return slot;
    }

    /// <summary>
    /// Every method of that name that can be called with this many arguments, in the order
    /// <see cref="MemberAccess.Call"/> tries them: the hierarchy first, then each type's own declaration.
    /// </summary>
    public static List<MethodInfo> CallCandidates(Type start, string method, int argumentCount)
    {
        Dictionary<(string Name, int Arguments), List<MethodInfo>> calls = Callable.GetOrCreateValue(start);
        if (calls.TryGetValue((method, argumentCount), out List<MethodInfo> cached))
        {
            return cached;
        }

        var candidates = new List<MethodInfo>();
        foreach (Type level in Hierarchy(start))
        {
            foreach (MethodInfo candidate in level.GetMethods(Declared))
            {
                if (candidate.Name == method && CanAccept(candidate, argumentCount))
                {
                    candidates.Add(candidate);
                }
            }
        }

        if (calls.Count < MaxMembersPerType)
        {
            calls[(method, argumentCount)] = candidates;
        }

        return candidates;
    }

    /// <summary>
    /// Every method of that name on a type, inherited ones included - the candidate list
    /// <see cref="MethodSelection.Select"/> chooses an overload from.
    /// </summary>
    public static List<MethodInfo> MethodsOf(Type type, string name)
    {
        Dictionary<string, List<MethodInfo>> methods = Named.GetOrCreateValue(type);
        if (methods.TryGetValue(name, out List<MethodInfo> cached))
        {
            return cached;
        }

        var found = new List<MethodInfo>();
        foreach (MethodInfo candidate in type.GetMethods(
                     BindingFlags.Public | BindingFlags.NonPublic | BindingFlags.Instance | BindingFlags.Static))
        {
            if (candidate.Name == name)
            {
                found.Add(candidate);
            }
        }

        if (methods.Count < MaxMembersPerType)
        {
            methods[name] = found;
        }

        return found;
    }

    private static void Remember(Dictionary<string, Slot> members, string member, Slot slot)
    {
        if (members.Count < MaxMembersPerType)
        {
            members[member] = slot;
        }
    }

    private static Slot FindReadable(Type start, string member)
    {
        foreach (Type level in Hierarchy(start))
        {
            FieldInfo field = level.GetField(member, Declared);
            if (field != null)
            {
                return new Slot { Field = field };
            }

            PropertyInfo property = FindProperty(level, member);
            if (property != null && property.CanRead && property.GetIndexParameters().Length == 0)
            {
                return new Slot { Property = property };
            }

            MethodInfo method = FindMethod(level, member, 0);
            if (method != null)
            {
                return new Slot { Method = method };
            }
        }

        return null;
    }

    private static Slot FindWritable(Type start, string member)
    {
        foreach (Type level in Hierarchy(start))
        {
            FieldInfo field = level.GetField(member, Declared);
            if (field != null)
            {
                return new Slot { Field = field };
            }

            PropertyInfo property = FindProperty(level, member);
            if (property != null && property.CanWrite && property.GetIndexParameters().Length == 0)
            {
                return new Slot { Property = property };
            }
        }

        return null;
    }

    /// <summary>The type itself, then every base type (private members are not inherited).</summary>
    internal static IEnumerable<Type> Hierarchy(Type type)
    {
        for (Type current = type; current != null; current = current.BaseType)
        {
            yield return current;
        }
    }

    /// <summary>
    /// One property by name, without letting reflection's ambiguity get in the way: a type that declares
    /// two indexers (`Item[Int32]` and `Item[Int32, Int32]`, `Verse.Area` has them) makes `GetProperty`
    /// throw `AmbiguousMatchException`, while the route only ever wants *a* member of that name.
    /// </summary>
    internal static PropertyInfo FindProperty(Type level, string name)
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

    /// <summary>The first method of that name on that level that takes the arguments.</summary>
    internal static MethodInfo FindMethod(Type level, string name, int argCount)
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

    /// <summary>A method matches when it takes exactly the arguments, or can default the rest.</summary>
    internal static bool CanAccept(MethodInfo method, int argumentCount)
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
}
