using System;
using System.Collections;
using System.Linq;
using System.Reflection;
using RimWorld;
using Verse;

namespace RustBridge;

/// <summary>
/// The reflection helpers <see cref="DlcOps"/> reads the game with. A member that is missing, renamed
/// or of an unexpected type answers null, so an absent expansion or a newer game build degrades into an
/// empty section instead of an exception.
/// </summary>
internal static partial class DlcOps
{
    private static Type TypeNamed(string name)
    {
        Type type = typeof(Def).Assembly.GetType("RimWorld." + name, throwOnError: false)
            ?? typeof(Def).Assembly.GetType("Verse." + name, throwOnError: false)
            ?? typeof(Def).Assembly.GetType(name, throwOnError: false);
        if (type != null)
        {
            return type;
        }

        foreach (Assembly assembly in AppDomain.CurrentDomain.GetAssemblies())
        {
            try
            {
                type = assembly.GetType("RimWorld." + name, throwOnError: false)
                    ?? assembly.GetType("Verse." + name, throwOnError: false)
                    ?? assembly.GetType(name, throwOnError: false);
                if (type != null)
                {
                    return type;
                }
            }
            catch
            {
                // Dynamic assemblies may refuse GetType.
            }
        }

        return null;
    }

    private static object StaticMember(Type type, string name)
    {
        if (type == null || string.IsNullOrEmpty(name))
        {
            return null;
        }

        try
        {
            return type.GetProperty(name, BindingFlags.Public | BindingFlags.Static)?.GetValue(null)
                ?? type.GetField(name, BindingFlags.Public | BindingFlags.Static)?.GetValue(null);
        }
        catch
        {
            return null;
        }
    }

    private static object Member(object target, params string[] names)
    {
        if (target == null || names == null)
        {
            return null;
        }

        Type type = target.GetType();
        foreach (string name in names)
        {
            if (string.IsNullOrEmpty(name))
            {
                continue;
            }

            try
            {
                object value = type.GetProperty(name, BindingFlags.Public | BindingFlags.Instance)?.GetValue(target)
                    ?? type.GetField(name, BindingFlags.Public | BindingFlags.Instance)?.GetValue(target);
                if (value != null)
                {
                    return value;
                }
            }
            catch
            {
                // Try the next spelling or the next member.
            }
        }

        return null;
    }

    private static object InvokeMember(object target, string name)
    {
        if (target == null || string.IsNullOrEmpty(name))
        {
            return null;
        }

        try
        {
            return target.GetType().GetMethod(name, BindingFlags.Public | BindingFlags.Instance)
                ?.Invoke(target, null);
        }
        catch
        {
            return null;
        }
    }

    private static IEnumerable Enumerate(object value)
    {
        if (value is IEnumerable enumerable)
        {
            return enumerable;
        }

        return Array.Empty<object>();
    }

    private static bool ContainsAny(string value, params string[] words)
    {
        if (string.IsNullOrEmpty(value))
        {
            return false;
        }

        return words.Any(word => !string.IsNullOrEmpty(word)
            && value.IndexOf(word, StringComparison.OrdinalIgnoreCase) >= 0);
    }

    private static string Label(Def def)
    {
        if (def == null)
        {
            return null;
        }

        string label = def.label;
        if (!string.IsNullOrEmpty(label))
        {
            return label;
        }

        return def.defName;
    }

    private static string Text(object value)
    {
        if (value == null)
        {
            return null;
        }

        if (value is string text)
        {
            return text;
        }

        try
        {
            return Convert.ToString(value, System.Globalization.CultureInfo.InvariantCulture);
        }
        catch
        {
            return null;
        }
    }

    private static bool Flag(object value)
    {
        if (value is bool flag)
        {
            return flag;
        }

        try
        {
            return Convert.ToBoolean(value, System.Globalization.CultureInfo.InvariantCulture);
        }
        catch
        {
            return false;
        }
    }

    private static double? Number(object value)
    {
        if (value == null)
        {
            return null;
        }

        try
        {
            return Convert.ToDouble(value, System.Globalization.CultureInfo.InvariantCulture);
        }
        catch
        {
            return null;
        }
    }
}
