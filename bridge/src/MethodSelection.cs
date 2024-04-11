using System;
using System.Collections.Generic;
using System.Linq;
using System.Reflection;

namespace RustBridge;

/// <summary>
/// Finds a method by name and optional parameter type names and describes what was available when
/// the lookup fails. Shared by the Harmony bridge and the companion assembly bridge so both give
/// the same errors for the same mistakes.
/// </summary>
internal static class MethodSelection
{
    /// <summary>
    /// Resolves one method. When several overloads share the name, `paramTypes` picks one; without
    /// it the number of arguments the caller passes is used, and only a name that stays ambiguous
    /// after that asks for explicit `paramTypes`.
    /// </summary>
    public static MethodInfo Select(Type type, string typeName, string methodName, string[] paramTypes, int argumentCount = -1)
    {
        var candidates = new List<MethodInfo>();
        foreach (MethodInfo candidate in type.GetMethods(
                     BindingFlags.Public | BindingFlags.NonPublic | BindingFlags.Instance | BindingFlags.Static))
        {
            if (candidate.Name == methodName)
            {
                candidates.Add(candidate);
            }
        }

        if (candidates.Count == 0)
        {
            throw new BridgeException($"method not found: {typeName}.{methodName}{SuggestMethods(type, methodName)}");
        }

        if (paramTypes == null || paramTypes.Length == 0)
        {
            List<MethodInfo> usable = candidates;
            if (argumentCount >= 0)
            {
                usable = candidates.Where(candidate => CanAccept(candidate, argumentCount)).ToList();
                if (usable.Count == 1)
                {
                    return usable[0];
                }
            }

            if (usable.Count > 1)
            {
                string how = argumentCount >= 0
                    ? $"{usable.Count} overloads that take {argumentCount} argument(s)"
                    : $"{usable.Count} overloads";
                throw new BridgeException(
                    $"{typeName}.{methodName} has {how}; pass paramTypes. Candidates: "
                    + string.Join(" | ", usable.Select(Describe)));
            }

            if (usable.Count == 0)
            {
                throw new BridgeException(
                    $"{typeName}.{methodName} cannot take {argumentCount} argument(s){DescribeOverloads(candidates)}");
            }

            return candidates[0];
        }

        foreach (MethodInfo candidate in candidates)
        {
            ParameterInfo[] parameters = candidate.GetParameters();
            if (parameters.Length != paramTypes.Length)
            {
                continue;
            }

            bool matches = true;
            for (int i = 0; i < parameters.Length; i++)
            {
                if (!ParameterMatches(parameters[i].ParameterType, paramTypes[i]))
                {
                    matches = false;
                    break;
                }
            }

            if (matches)
            {
                return candidate;
            }
        }

        throw new BridgeException(
            $"{typeName}.{methodName} has no overload matching ({string.Join(", ", paramTypes)}){DescribeOverloads(candidates)}");
    }

    /// <summary>True when the method can be called with this many arguments (defaults fill the rest).</summary>
    private static bool CanAccept(MethodInfo method, int argumentCount)
    {
        ParameterInfo[] parameters = method.GetParameters();
        if (parameters.Length < argumentCount)
        {
            return false;
        }

        for (int index = argumentCount; index < parameters.Length; index++)
        {
            if (!parameters[index].HasDefaultValue)
            {
                return false;
            }
        }

        return true;
    }

    /// <summary>Full signature of a method, used in log lines and error messages.</summary>
    public static string Describe(MethodBase method)
    {
        string parameters = string.Join(", ", method.GetParameters().Select(p => p.ParameterType.Name));
        return $"{method.DeclaringType?.FullName}.{method.Name}({parameters})";
    }

    /// <summary>Lists overloads so the caller can copy the right parameter types.</summary>
    public static string DescribeOverloads(IEnumerable<MethodInfo> candidates)
    {
        var descriptions = candidates
            .Take(8)
            .Select(method => $"{method.Name}({string.Join(", ", method.GetParameters().Select(parameter => parameter.ParameterType.Name))})")
            .ToList();
        return descriptions.Count == 0 ? string.Empty : " | candidates: " + string.Join(" | ", descriptions);
    }

    /// <summary>
    /// Accepts full names, simple names, arity-free names and short generic forms such as
    /// "Nullable&lt;Rot4&gt;" or "Predicate&lt;IntVec3&gt;" for one parameter type. Comparison is
    /// case-insensitive and understands the C# keywords people write in Rust code
    /// (`"string"` for `String`, `"int"` for `Int32`, `"List<int>"` for `List&lt;Int32&gt;`).
    /// </summary>
    public static bool ParameterMatches(Type parameter, string expected)
    {
        if (Matches(parameter, expected))
        {
            return true;
        }

        return parameter.IsByRef && parameter.GetElementType() is Type element && Matches(element, expected);
    }

    /// <summary>C# keywords and their CLR names.</summary>
    private static readonly Dictionary<string, string> KeywordAliases = new(StringComparer.OrdinalIgnoreCase)
    {
        ["bool"] = "Boolean",
        ["byte"] = "Byte",
        ["sbyte"] = "SByte",
        ["char"] = "Char",
        ["short"] = "Int16",
        ["ushort"] = "UInt16",
        ["int"] = "Int32",
        ["uint"] = "UInt32",
        ["long"] = "Int64",
        ["ulong"] = "UInt64",
        ["float"] = "Single",
        ["double"] = "Double",
        ["decimal"] = "Decimal",
        ["string"] = "String",
        ["object"] = "Object",
        ["void"] = "Void",
    };

    private static bool Matches(Type type, string expected)
    {
        string wanted = Alias(expected);

        string simple = type.Name;
        int arity = simple.IndexOf('`');
        string withoutArity = arity >= 0 ? simple.Substring(0, arity) : simple;
        if (string.Equals(type.FullName, wanted, StringComparison.OrdinalIgnoreCase)
            || string.Equals(simple, wanted, StringComparison.OrdinalIgnoreCase)
            || string.Equals(withoutArity, wanted, StringComparison.OrdinalIgnoreCase))
        {
            return true;
        }

        int open = wanted.IndexOf('<');
        if (open < 0 || !wanted.EndsWith(">", StringComparison.Ordinal))
        {
            return false;
        }

        if (!type.IsGenericType
            || !string.Equals(withoutArity, Alias(wanted.Substring(0, open)), StringComparison.OrdinalIgnoreCase))
        {
            return false;
        }

        string[] arguments = wanted.Substring(open + 1, wanted.Length - open - 2).Split(',');
        Type[] actual = type.GetGenericArguments();
        if (actual.Length != arguments.Length)
        {
            return false;
        }

        for (int index = 0; index < actual.Length; index++)
        {
            if (!Matches(actual[index], arguments[index].Trim()))
            {
                return false;
            }
        }

        return true;
    }

    private static string Alias(string name)
    {
        return KeywordAliases.TryGetValue(name, out string alias) ? alias : name;
    }

    /// <summary>Suggests type names from the given assemblies that look like the one that was not found.</summary>
    public static string SuggestTypes(IEnumerable<Assembly> assemblies, string wanted)
    {
        var matches = new List<string>();
        foreach (Assembly assembly in assemblies)
        {
            Type[] types;
            try
            {
                types = assembly.GetTypes();
            }
            catch (ReflectionTypeLoadException error)
            {
                types = error.Types.Where(type => type != null).ToArray();
            }
            catch (Exception)
            {
                continue;
            }

            foreach (Type type in types)
            {
                if (type == null)
                {
                    continue;
                }

                if (type.Name.Contains(wanted, StringComparison.OrdinalIgnoreCase)
                    || wanted.Contains(type.Name, StringComparison.OrdinalIgnoreCase))
                {
                    matches.Add(type.FullName ?? type.Name);
                    if (matches.Count >= 5)
                    {
                        break;
                    }
                }
            }

            if (matches.Count >= 5)
            {
                break;
            }
        }

        return matches.Count == 0 ? string.Empty : " (did you mean: " + string.Join(", ", matches) + "?)";
    }

    /// <summary>Suggests method names on the type that look like the one that was not found.</summary>
    public static string SuggestMethods(Type type, string wanted)
    {
        var matches = new List<string>();
        foreach (MethodInfo method in type.GetMethods(
                     BindingFlags.Public | BindingFlags.NonPublic | BindingFlags.Instance | BindingFlags.Static))
        {
            if (method.Name.Contains(wanted, StringComparison.OrdinalIgnoreCase)
                || wanted.Contains(method.Name, StringComparison.OrdinalIgnoreCase))
            {
                if (!matches.Contains(method.Name))
                {
                    matches.Add(method.Name);
                }

                if (matches.Count >= 5)
                {
                    break;
                }
            }
        }

        return matches.Count == 0 ? string.Empty : " (did you mean: " + string.Join(", ", matches) + "?)";
    }
}
