using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Linq;
using System.Reflection;
using System.Text.Json;
using Verse;

namespace RustBridge;

/// <summary>
/// What the game's assemblies expose and whether the generic route can reach it.
///
/// The SDK's answer to "the game has sixteen thousand types" is the handle route: `type_get` for anything
/// static, `ref_get`/`ref_set`/`ref_call` for instances, `pawn_tracker`/`map_system`/`game_system`/
/// `world_system`/`thing_comp` for the objects a mod usually wants. These three operations make that
/// surface *visible* (`game_types`, `type_members`) and *provable* (`member_scan`, which walks every type
/// of every loaded assembly and checks every declared member with the same lookups the route performs).
/// </summary>
internal static class TypesOps
{
    /// <summary>The name a mod uses to reach a type: the full name, nested types included.</summary>
    private static string NameOf(Type type)
    {
        return type.FullName ?? type.Name;
    }

    /// <summary>
    /// Every type of every loaded assembly, optionally filtered by a name prefix, up to `limit`
    /// (default 500: a page a mod can walk). `total` counts everything that matched, so a mod knows when
    /// it has to ask for the next page.
    /// </summary>
    public static object Types(JsonElement request)
    {
        string prefix = JsonArgs.GetString(request, "prefix");
        int limit = Math.Clamp(JsonArgs.GetInt(request, "limit", 500), 1, 20000);

        var names = new List<string>();
        int total = 0;
        foreach (Type type in GenTypes.AllTypes)
        {
            if (type == null)
            {
                continue;
            }

            string name = NameOf(type);
            if (prefix.Length > 0 && !name.StartsWith(prefix, StringComparison.Ordinal))
            {
                continue;
            }

            total++;
            if (names.Count < limit)
            {
                names.Add(name);
            }
        }

        return new Dictionary<string, object>
        {
            ["types"] = names,
            ["total"] = total,
            ["truncated"] = names.Count < total,
        };
    }

    /// <summary>
    /// One type's declared members — the type's own and its bases', the order the handle route walks — with
    /// what each one is (`field`/`property`/`method`), whether it is static and how it is visible, so a mod
    /// learns the surface the game really exposes instead of guessing names. `limit` (default 200) pages
    /// the answer; `total` says how many there are.
    /// </summary>
    public static object Members(JsonElement request)
    {
        string name = JsonArgs.GetString(request, "type");
        Type type = GenTypes.GetTypeInAnyAssembly(name, null)
            ?? throw new BridgeException(
                $"the game has no type '{name}' (use the full name, for example 'RimWorld.MoteMaker'; `game_types` lists them)");
        int limit = Math.Clamp(JsonArgs.GetInt(request, "limit", 200), 1, 5000);

        var members = new List<Dictionary<string, object>>();
        int total = 0;
        foreach ((Type level, MemberInfo member) in MemberAccess.DeclaredMembers(type))
        {
            total++;
            if (members.Count >= limit)
            {
                continue;
            }

            var row = new Dictionary<string, object>
            {
                ["name"] = member.Name,
                ["declaredOn"] = level.Name,
                ["kind"] = member switch
                {
                    FieldInfo => "field",
                    PropertyInfo => "property",
                    MethodInfo => "method",
                    _ => "member",
                },
                ["static"] = member switch
                {
                    FieldInfo fieldOf => fieldOf.IsStatic,
                    PropertyInfo propertyOf => (propertyOf.GetGetMethod(true) ?? propertyOf.GetSetMethod(true))?.IsStatic ?? false,
                    MethodInfo methodOf => methodOf.IsStatic,
                    _ => false,
                },
                ["visibility"] = Visibility(member),
            };

            if (member is MethodInfo method)
            {
                row["parameters"] = method.GetParameters()
                    .Select(parameter => $"{parameter.ParameterType.Name} {parameter.Name}")
                    .ToList();
                row["returns"] = method.ReturnType.Name;
            }

            if (member is PropertyInfo propertyInfo)
            {
                object accessors = new Dictionary<string, object>
                {
                    ["read"] = propertyInfo.GetGetMethod(true) != null,
                    ["write"] = propertyInfo.GetSetMethod(true) != null,
                };
                row["accessors"] = accessors;
            }

            members.Add(row);
        }

        return new Dictionary<string, object>
        {
            ["type"] = NameOf(type),
            ["members"] = members,
            ["total"] = total,
            ["truncated"] = members.Count < total,
        };
    }

    /// <summary>
    /// Walks **every** type of every loaded assembly and checks **every** declared member with the same
    /// lookups `ref_get`/`ref_set`/`ref_call` perform, plus the game's own type resolver that `type_get`
    /// uses. The answer is the audit trail: how many types and members there are, how many the route can
    /// address, and the first few it cannot (a member it cannot address is a bug in the bridge, not in the
    /// mod that would have guessed the name).
    /// </summary>
    public static object Scan(JsonElement request)
    {
        int limit = Math.Clamp(JsonArgs.GetInt(request, "limit", 16), 1, 200);
        var failures = new List<string>();

        int types = 0;
        int unnamed = 0;
        int fields = 0;
        int properties = 0;
        int methods = 0;
        int addressable = 0;

        var stopwatch = Stopwatch.StartNew();
        foreach (Type type in GenTypes.AllTypes)
        {
            if (type == null)
            {
                continue;
            }

            types++;
            string name = type.FullName;
            if (name == null)
            {
                // An open generic definition has no name a mod could resolve (`List`1`), so it is counted
                // apart instead of being called unreachable.
                unnamed++;
            }
            else if (!ReferenceEquals(GenTypes.GetTypeInAnyAssembly(name, null), type))
            {
                if (failures.Count < limit)
                {
                    failures.Add($"{name}: the type resolver the `type_get` route uses does not find it");
                }
            }

            foreach ((Type level, MemberInfo member) in MemberAccess.DeclaredMembers(type))
            {
                switch (member)
                {
                    case FieldInfo:
                        fields++;
                        break;
                    case PropertyInfo:
                        properties++;
                        break;
                    case MethodInfo:
                        methods++;
                        break;
                }

                if (MemberAccess.CanAddress(level, member))
                {
                    addressable++;
                }
                else if (failures.Count < limit)
                {
                    failures.Add($"{NameOf(type)}.{member.Name}: the member lookup refuses it");
                }
            }
        }

        stopwatch.Stop();
        int members = fields + properties + methods;
        return new Dictionary<string, object>
        {
            ["types"] = types,
            ["unnamed"] = unnamed,
            ["fields"] = fields,
            ["properties"] = properties,
            ["methods"] = methods,
            ["members"] = members,
            ["addressable"] = addressable,
            ["missing"] = members - addressable,
            ["failures"] = failures,
            ["millis"] = stopwatch.ElapsedMilliseconds,
        };
    }

    private static string Visibility(MemberInfo member)
    {
        FieldAttributes? fieldAttributes = (member as FieldInfo)?.Attributes;
        MethodAttributes? methodAttributes = member switch
        {
            MethodInfo method => method.Attributes,
            PropertyInfo property => (property.GetGetMethod(true) ?? property.GetSetMethod(true))?.Attributes,
            _ => null,
        };

        if (fieldAttributes is FieldAttributes fieldAccess)
        {
            return Access(fieldAccess);
        }

        return methodAttributes is MethodAttributes methodAccess ? Access(methodAccess) : "unknown";
    }

    private static string Access(FieldAttributes attributes)
    {
        return (attributes & FieldAttributes.FieldAccessMask) switch
        {
            FieldAttributes.Public => "public",
            FieldAttributes.Assembly => "internal",
            FieldAttributes.Family => "protected",
            FieldAttributes.FamORAssem => "protected internal",
            FieldAttributes.FamANDAssem => "private protected",
            _ => "private",
        };
    }

    private static string Access(MethodAttributes attributes)
    {
        return (attributes & MethodAttributes.MemberAccessMask) switch
        {
            MethodAttributes.Public => "public",
            MethodAttributes.Assembly => "internal",
            MethodAttributes.Family => "protected",
            MethodAttributes.FamORAssem => "protected internal",
            MethodAttributes.FamANDAssem => "private protected",
            _ => "private",
        };
    }
}
