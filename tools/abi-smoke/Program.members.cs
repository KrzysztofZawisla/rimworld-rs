using System;
using System.Collections.Generic;
using System.Collections.Immutable;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Reflection.Emit;
using System.Reflection.Metadata;
using System.Reflection.PortableExecutable;
using System.Runtime.InteropServices;
using System.Text;
using System.Text.Json;
using System.Text.RegularExpressions;
using System.Xml.Linq;
namespace RustBridge.Smoke;

/// <summary>
/// The `members` half of <see cref="Program"/>.
/// </summary>
internal static partial class Program
{
    private static MetadataReader OpenGameAssembly(string directory)
    {
        string path = Path.Combine(directory, "Assembly-CSharp.dll");
        if (!File.Exists(path))
        {
            Console.Error.WriteLine($"no Assembly-CSharp.dll in {Path.GetFullPath(directory)}");
            return null;
        }

        openAssemblyStream?.Dispose();
        openAssemblyReader?.Dispose();
        openAssemblyStream = File.OpenRead(path);
        openAssemblyReader = new PEReader(openAssemblyStream);
        if (!openAssemblyReader.HasMetadata)
        {
            Console.Error.WriteLine($"{path} has no managed metadata");
            return null;
        }

        Console.Error.WriteLine($"game assembly : {path}");
        return openAssemblyReader.GetMetadataReader();
    }

    /// <summary>Full CLR name of a type definition (nested types keep their simple name).</summary>
    internal static string FullTypeName(MetadataReader reader, TypeDefinitionHandle handle)
    {
        TypeDefinition type = reader.GetTypeDefinition(handle);
        string name = reader.GetString(type.Name);
        string namespaceName = reader.GetString(type.Namespace);
        return string.IsNullOrEmpty(namespaceName) ? name : namespaceName + "." + name;
    }
    private static TypeDefinitionHandle? FindType(MetadataReader reader, string name, bool quiet = false)
    {
        var exact = new List<TypeDefinitionHandle>();
        var simple = new List<TypeDefinitionHandle>();
        foreach (TypeDefinitionHandle handle in reader.TypeDefinitions)
        {
            string fullName = FullTypeName(reader, handle);
            if (fullName == name)
            {
                exact.Add(handle);
                continue;
            }

            if (reader.GetString(reader.GetTypeDefinition(handle).Name) == name)
            {
                simple.Add(handle);
            }
        }

        if (exact.Count == 1)
        {
            return exact[0];
        }

        if (exact.Count == 0 && simple.Count == 1)
        {
            return simple[0];
        }

        List<TypeDefinitionHandle> candidates = exact.Count > 0 ? exact : simple;
        if (candidates.Count == 0)
        {
            if (!quiet)
            {
                Console.Error.WriteLine($"type not found: {name} (try --list-types <prefix>)");
            }

            return null;
        }

        Console.Error.WriteLine($"'{name}' is ambiguous; candidates:");
        foreach (TypeDefinitionHandle candidate in candidates.Take(20))
        {
            Console.Error.WriteLine($"  {FullTypeName(reader, candidate)}");
        }

        return null;
    }
    private static int RunListTypes(string directory, string prefix)
    {
        MetadataReader reader = OpenGameAssembly(directory);
        if (reader == null)
        {
            return 2;
        }

        var names = new List<string>();
        foreach (TypeDefinitionHandle handle in reader.TypeDefinitions)
        {
            string fullName = FullTypeName(reader, handle);
            if (prefix == null || fullName.StartsWith(prefix, StringComparison.Ordinal))
            {
                names.Add(fullName);
            }
        }

        names.Sort(StringComparer.Ordinal);
        Console.WriteLine($"types matching '{prefix ?? "*"}': {names.Count}");
        foreach (string name in names.Take(400))
        {
            Console.WriteLine($"  {name}");
        }

        if (names.Count > 400)
        {
            Console.WriteLine($"  ... and {names.Count - 400} more; narrow the prefix");
        }

        return 0;
    }

    /// <summary>
    /// Prints the members of one game type with ready-to-paste `harmony_patch` lines, so mod
    /// authors do not have to guess names or parameter types.
    /// </summary>
    /// <summary>
    /// Prints the members of the given types as JSON — the machine-readable half of `--targets`, for tooling
    /// that writes code (`cargo xtask gen-subsystems`). Every member carries its kind (field, property,
    /// method), how many arguments a method takes, which type in the hierarchy declares it, whether it is
    /// static and how visible it is, so the caller never parses a layout meant for a human: a method that
    /// needs arguments cannot be mistaken for a property, which is the mistake this exists to prevent.
    /// </summary>
    private static int RunMembersJson(string directory, List<string> typeNames)
    {
        MetadataReader reader = OpenGameAssembly(directory);
        if (reader == null)
        {
            return 2;
        }

        var provider = new TypeNameProvider(reader);
        var types = new List<string>();
        foreach (string typeName in typeNames)
        {
            TypeDefinitionHandle? start = FindType(reader, typeName, quiet: true);
            if (start == null)
            {
                Console.Error.WriteLine($"skip: this installation has no type {typeName}");
                continue;
            }

            var members = new List<string>();
            TypeDefinitionHandle? level = start;
            while (level != null)
            {
                TypeDefinition type = reader.GetTypeDefinition(level.Value);
                string declaredIn = FullTypeName(reader, level.Value);

                foreach (FieldDefinitionHandle handle in type.GetFields())
                {
                    FieldDefinition field = reader.GetFieldDefinition(handle);
                    string name = reader.GetString(field.Name);
                    if (SkipMemberName(name))
                    {
                        continue;
                    }

                    bool isStatic = (field.Attributes & System.Reflection.FieldAttributes.Static) != 0;
                    members.Add(MemberJson(
                        name,
                        "field",
                        declaredIn,
                        isStatic,
                        Access(field.Attributes),
                        -1,
                        field.DecodeSignature(provider, null)));
                }

                foreach (PropertyDefinitionHandle handle in type.GetProperties())
                {
                    PropertyDefinition property = reader.GetPropertyDefinition(handle);
                    string name = reader.GetString(property.Name);
                    if (SkipMemberName(name))
                    {
                        continue;
                    }

                    MethodDefinitionHandle getter = property.GetAccessors().Getter;
                    MethodDefinition? accessor = getter.IsNil ? null : reader.GetMethodDefinition(getter);

                    bool isStatic = accessor is MethodDefinition method
                        && (method.Attributes & System.Reflection.MethodAttributes.Static) != 0;
                    members.Add(MemberJson(
                        name,
                        "property",
                        declaredIn,
                        isStatic,
                        accessor is MethodDefinition accessorMethod ? Access(accessorMethod.Attributes) : "private",
                        -1,
                        property.DecodeSignature(provider, null).ReturnType));
                }

                foreach (MethodDefinitionHandle handle in type.GetMethods())
                {
                    MethodDefinition method = reader.GetMethodDefinition(handle);
                    string name = reader.GetString(method.Name);
                    if (SkipMemberName(name))
                    {
                        continue;
                    }

                    MethodSignature<string> signature = method.DecodeSignature(provider, null);
                    members.Add(MemberJson(
                        name,
                        "method",
                        declaredIn,
                        (method.Attributes & System.Reflection.MethodAttributes.Static) != 0,
                        Access(method.Attributes),
                        signature.ParameterTypes.Length,
                        signature.ReturnType));
                }

                level = type.BaseType.IsNil ? null : FindType(reader, provider.TypeName(type.BaseType), quiet: true);
            }

            types.Add("{\"type\":\"" + Escape(typeName) + "\",\"members\":[" + string.Join(",", members) + "]}");
        }

        Console.WriteLine("[" + string.Join(",", types) + "]");
        return 0;
    }

    /// <summary>One member as JSON, with every fact the code generator needs and nothing it would parse.</summary>
    private static string MemberJson(
        string name,
        string kind,
        string declaredIn,
        bool isStatic,
        string visibility,
        int arity,
        string type)
    {
        string json = "{\"name\":\"" + Escape(name) + "\""
            + ",\"kind\":\"" + kind + "\""
            + ",\"declaredIn\":\"" + Escape(declaredIn) + "\""
            + ",\"static\":" + (isStatic ? "true" : "false")
            + ",\"visibility\":\"" + visibility + "\""
            + ",\"type\":\"" + Escape(type) + "\"";
        if (arity >= 0)
        {
            json += ",\"arity\":" + arity;
        }

        return json + "}";
    }
    private static bool SkipMemberName(string name)
    {
        return name.Length == 0
            || name == ".ctor"
            || name == ".cctor"
            || name.IndexOf('<') >= 0
            || name.StartsWith("get_", StringComparison.Ordinal)
            || name.StartsWith("set_", StringComparison.Ordinal)
            || name.StartsWith("add_", StringComparison.Ordinal)
            || name.StartsWith("remove_", StringComparison.Ordinal);
    }
    private static string Escape(string text)
    {
        return text.Replace("\\", "\\\\").Replace("\"", "\\\"");
    }
    private static string Access(System.Reflection.FieldAttributes attributes)
    {
        return (attributes & System.Reflection.FieldAttributes.FieldAccessMask) switch
        {
            System.Reflection.FieldAttributes.Public => "public",
            System.Reflection.FieldAttributes.Assembly => "internal",
            System.Reflection.FieldAttributes.Family => "protected",
            System.Reflection.FieldAttributes.FamORAssem => "protected internal",
            System.Reflection.FieldAttributes.FamANDAssem => "private protected",
            _ => "private",
        };
    }
    private static string Access(System.Reflection.MethodAttributes attributes)
    {
        return (attributes & System.Reflection.MethodAttributes.MemberAccessMask) switch
        {
            System.Reflection.MethodAttributes.Public => "public",
            System.Reflection.MethodAttributes.Assembly => "internal",
            System.Reflection.MethodAttributes.Family => "protected",
            System.Reflection.MethodAttributes.FamORAssem => "protected internal",
            System.Reflection.MethodAttributes.FamANDAssem => "private protected",
            _ => "private",
        };
    }

    /// <summary>
    /// Turns "ref System.Collections.Generic.List`1&lt;Verse.Thing&gt;" into "List&lt;Thing&gt;", which is
    /// exactly the form the bridge's parameter matcher accepts.
    /// </summary>
    private static string ShortTypeName(string typeName)
    {
        var builder = new StringBuilder();
        var token = new StringBuilder();

        void FlushToken()
        {
            if (token.Length == 0)
            {
                return;
            }

            string text = token.ToString();
            token.Clear();
            int lastDot = text.LastIndexOf('.');
            string shortName = lastDot >= 0 ? text.Substring(lastDot + 1) : text;
            int arity = shortName.IndexOf('`');
            if (arity >= 0)
            {
                shortName = shortName.Substring(0, arity);
            }

            builder.Append(shortName);
        }

        foreach (char character in typeName)
        {
            if (char.IsLetterOrDigit(character) || character == '_' || character == '.' || character == '`')
            {
                token.Append(character);
            }
            else
            {
                FlushToken();
                builder.Append(character);
            }
        }

        FlushToken();
        string result = builder.ToString();
        return result.StartsWith("ref ", StringComparison.Ordinal) ? result.Substring(4) : result;
    }
}
