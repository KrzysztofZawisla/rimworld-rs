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
/// The `types` half of <see cref="Program"/>.
/// </summary>
internal static partial class Program
{

    /// <summary>
    /// Prints the members of one game type with ready-to-paste `harmony_patch` lines, so mod
    /// authors do not have to guess names or parameter types.
    /// </summary>
    /// <summary>
    /// Answers "do we have a wrapper for everything?" per type, with numbers: how many members the type
    /// (and its bases) declares, how many of them the bridge's contract names — i.e. some typed operation
    /// or wrapper reads or writes them — and how many are reachable only through the generic handle route
    /// (`ref_get`/`ref_set`/`ref_call` plus the accessor ops, which reach *every* member).
    /// </summary>
    private static int RunCoverage(string directory, string typeName)
    {
        MetadataReader reader = OpenGameAssembly(directory);
        if (reader == null)
        {
            return 2;
        }

        var provider = new TypeNameProvider(reader);
        var hierarchy = new List<(TypeDefinitionHandle Handle, string Name)>();
        TypeDefinitionHandle? cursor = FindType(reader, typeName);
        while (cursor != null)
        {
            TypeDefinition definition = reader.GetTypeDefinition(cursor.Value);
            hierarchy.Add((cursor.Value, FullTypeName(reader, cursor.Value)));
            cursor = definition.BaseType.IsNil
                ? null
                : FindType(reader, provider.TypeName(definition.BaseType), quiet: true);
        }

        if (hierarchy.Count == 0)
        {
            Console.Error.WriteLine($"no type {typeName}");
            return 1;
        }

        var declared = new SortedSet<string>(StringComparer.Ordinal);
        foreach ((TypeDefinitionHandle handle, string _) in hierarchy)
        {
            TypeDefinition definition = reader.GetTypeDefinition(handle);
            foreach (FieldDefinitionHandle field in definition.GetFields())
            {
                declared.Add(reader.GetString(reader.GetFieldDefinition(field).Name));
            }

            foreach (PropertyDefinitionHandle property in definition.GetProperties())
            {
                declared.Add(reader.GetString(reader.GetPropertyDefinition(property).Name));
            }

            foreach (MethodDefinitionHandle method in definition.GetMethods())
            {
                string name = reader.GetString(reader.GetMethodDefinition(method).Name);
                // Accessors of a property are the property, not a member of their own, and the compiler's
                // own members (backing fields, lambdas, local functions) are not things a mod calls.
                if (name.StartsWith("get_", StringComparison.Ordinal)
                    || name.StartsWith("set_", StringComparison.Ordinal)
                    || name.StartsWith("add_", StringComparison.Ordinal)
                    || name.StartsWith("remove_", StringComparison.Ordinal))
                {
                    continue;
                }

                declared.Add(name);
            }
        }

        // The compiler's own members (backing fields, lambdas, local functions, constructors) are not
        // things a mod reaches for, so they do not count as uncovered surface.
        declared = new SortedSet<string>(
            declared.Where(name => !name.Contains('<') && !name.Contains('>') && !name.StartsWith(".", StringComparison.Ordinal)),
            StringComparer.Ordinal);

        var typed = new SortedSet<string>(StringComparer.Ordinal);
        foreach ((_, string level) in hierarchy)
        {
            foreach ((string type, string member) in ContractRequirements)
            {
                if (type != level || member.Length == 0 || member == ".ctor")
                {
                    continue;
                }

                typed.Add(member.StartsWith("get_", StringComparison.Ordinal) || member.StartsWith("set_", StringComparison.Ordinal)
                    ? member.Substring(4)
                    : member);
            }
        }

        var covered = declared.Where(name => typed.Contains(name)).ToList();
        var genericOnly = declared.Where(name => !typed.Contains(name)).ToList();

        Console.WriteLine($"{hierarchy[0].Name}");
        Console.WriteLine($"  hierarchy    : {string.Join(" -> ", hierarchy.Select(entry => entry.Name))}");
        Console.WriteLine($"  declared     : {declared.Count} member(s), accessors counted with their property");
        Console.WriteLine($"  typed        : {covered.Count} named by the bridge contract (an operation or a wrapper uses them)");
        Console.WriteLine($"  generic only : {genericOnly.Count} reachable through ref_get/ref_set/ref_call and the accessor ops");
        Console.WriteLine($"  typed names  : {string.Join(", ", covered.Take(24))}{(covered.Count > 24 ? ", ..." : string.Empty)}");
        Console.WriteLine($"  generic names: {string.Join(", ", genericOnly.Take(24))}{(genericOnly.Count > 24 ? ", ..." : string.Empty)}");
        return 0;
    }

    /// <summary>
    /// Reduces a type name to what a handler author writes: `Verse.ResearchProjectDef` and
    /// `ResearchProjectDef`, `System.Boolean` and `bool`, `List`1&lt;Verse.Thing&gt;` and `List&lt;Thing&gt;` all
    /// have to compare equal for the binding check to be about names, not about spelling.
    /// </summary>
    private static string SimpleTypeName(string typeName)
    {
        string name = (typeName ?? string.Empty).Trim();

        // `DamageInfo?`, `Nullable<DamageInfo>` and `System.Nullable`1<Verse.DamageInfo>` are one
        // parameter; the short spellings are what the bridge's own patch lines use.
        if (name.EndsWith("?", StringComparison.Ordinal))
        {
            name = name.Substring(0, name.Length - 1);
        }

        name = UnwrapNullable(name, "System.Nullable`1");
        name = UnwrapNullable(name, "Nullable");

        int opening = name.IndexOf('<');
        if (opening >= 0)
        {
            name = name.Substring(0, opening);
        }

        int arity = name.IndexOf('`');
        if (arity >= 0)
        {
            name = name.Substring(0, arity);
        }

        int dot = name.LastIndexOf('.');
        if (dot >= 0)
        {
            name = name.Substring(dot + 1);
        }

        return name switch
        {
            "Boolean" => "bool",
            "Int32" => "int",
            "Single" => "float",
            "Double" => "double",
            "String" => "string",
            _ => name,
        };
    }

    /// <summary>`Nullable&lt;DamageInfo&gt;` and `System.Nullable`1&lt;Verse.DamageInfo&gt;` mean the same parameter.</summary>
    private static string UnwrapNullable(string name, string prefix)
    {
        string head = prefix + "<";
        return name.StartsWith(head, StringComparison.Ordinal) && name.EndsWith(">", StringComparison.Ordinal)
            ? name.Substring(head.Length, name.Length - head.Length - 1)
            : name;
    }
}
