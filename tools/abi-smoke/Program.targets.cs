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
/// The `targets` half of <see cref="Program"/>.
/// </summary>
internal static partial class Program
{
    private static int RunTargets(string directory, string typeName, string filter)
    {
        MetadataReader reader = OpenGameAssembly(directory);
        if (reader == null)
        {
            return 2;
        }

        TypeDefinitionHandle? resolved = FindType(reader, typeName);
        if (resolved == null)
        {
            return 1;
        }

        TypeDefinition type = reader.GetTypeDefinition(resolved.Value);
        var provider = new TypeNameProvider(reader);
        string fullName = FullTypeName(reader, resolved.Value);
        string baseName = type.BaseType.IsNil ? null : provider.TypeName(type.BaseType);
        string kind = (type.Attributes & TypeAttributes.Interface) != 0 ? "interface"
            : baseName == "System.Enum" ? "enum"
            : baseName == "System.ValueType" ? "struct"
            : baseName == "System.MulticastDelegate" ? "delegate"
            : "class";
        Console.WriteLine($"{fullName} : {kind}{(baseName != null && baseName != "System.Object" ? " : " + baseName : string.Empty)}");
        var interfaces = type.GetInterfaceImplementations()
            .Select(handle => provider.TypeName(reader.GetInterfaceImplementation(handle).Interface))
            .ToList();
        if (interfaces.Count > 0)
        {
            Console.WriteLine($"interfaces : {string.Join(", ", interfaces)}");
        }

        bool matches(string name) => filter == null || name.Contains(filter, StringComparison.OrdinalIgnoreCase);
        bool snippets = filter != null;

        Console.WriteLine();
        Console.WriteLine("methods (call through harmony_patch or ref_call):");
        foreach (MethodDefinitionHandle handle in type.GetMethods())
        {
            MethodDefinition method = reader.GetMethodDefinition(handle);
            string name = reader.GetString(method.Name);
            if (!matches(name))
            {
                continue;
            }

            MethodSignature<string> signature = method.DecodeSignature(provider, null);
            string[] parameterTypes = signature.ParameterTypes.ToArray();
            string[] parameterNames = new string[parameterTypes.Length];
            foreach (ParameterHandle parameterHandle in method.GetParameters())
            {
                Parameter parameter = reader.GetParameter(parameterHandle);
                int sequence = parameter.SequenceNumber;
                if (sequence >= 1 && sequence <= parameterTypes.Length && !parameter.Name.IsNil)
                {
                    parameterNames[sequence - 1] = reader.GetString(parameter.Name);
                }
            }

            var parameters = new List<string>();
            for (int index = 0; index < parameterTypes.Length; index++)
            {
                string parameterName = parameterNames[index] ?? "p" + index;
                parameters.Add($"{parameterTypes[index]} {parameterName}");
            }

            Console.WriteLine($"  {signature.ReturnType} {name}({string.Join(", ", parameters)}){DescribeMethodFlags(method.Attributes)}");

            bool patchable = (method.Attributes & MethodAttributes.Abstract) == 0 && signature.GenericParameterCount == 0;
            if (snippets && patchable)
            {
                string paramTypes = parameterTypes.Length == 0
                    ? "&[]"
                    : "&[\"" + string.Join("\", \"", parameterTypes.Select(ShortTypeName)) + "\"]";
                Console.WriteLine(
                    $"      ctx.harmony_patch_with_params(\"{fullName}\", \"{name}\", {paramTypes}, PatchKind::Prefix, \"handler\")?;");
            }
        }

        Console.WriteLine();
        Console.WriteLine("properties (ref_get / ref_set):");
        foreach (PropertyDefinitionHandle handle in type.GetProperties())
        {
            PropertyDefinition property = reader.GetPropertyDefinition(handle);
            string name = reader.GetString(property.Name);
            if (!matches(name))
            {
                continue;
            }

            var signature = property.DecodeSignature(provider, null);
            PropertyAccessors accessors = property.GetAccessors();
            string access = (accessors.Getter.IsNil ? string.Empty : "get; ")
                + (accessors.Setter.IsNil ? string.Empty : "set; ");
            Console.WriteLine($"  {signature.ReturnType} {name} {{ {access}}}");
        }

        Console.WriteLine();
        Console.WriteLine("fields (ref_get / ref_set):");
        foreach (FieldDefinitionHandle handle in type.GetFields())
        {
            FieldDefinition field = reader.GetFieldDefinition(handle);
            string name = reader.GetString(field.Name);
            if (!matches(name))
            {
                continue;
            }

            string fieldType = field.DecodeSignature(provider, null);
            Console.WriteLine($"  {fieldType} {name}{DescribeFieldFlags(field.Attributes)}");
        }

        if (!snippets)
        {
            Console.WriteLine();
            Console.WriteLine("add --members <substring> to print ready-to-paste harmony_patch lines");
        }

        return 0;
    }
    private static string DescribeMethodFlags(MethodAttributes attributes)
    {
        var flags = new List<string>();
        if ((attributes & MethodAttributes.Static) != 0)
        {
            flags.Add("static");
        }

        if ((attributes & MethodAttributes.Abstract) != 0)
        {
            flags.Add("abstract");
        }

        if ((attributes & MethodAttributes.Virtual) != 0)
        {
            flags.Add("virtual");
        }

        // The accessibility is what decides whether the bridge can call a member *directly* (a private or
        // protected one is only reachable through `ref_call`, which reflects): say it plainly instead of
        // leaving the reader to wonder what an unmarked method can do.
        switch (attributes & MethodAttributes.MemberAccessMask)
        {
            case MethodAttributes.Public:
                break;
            case MethodAttributes.Assembly:
                flags.Add("internal");
                break;
            case MethodAttributes.Family:
                flags.Add("protected");
                break;
            case MethodAttributes.FamORAssem:
                flags.Add("protected internal");
                break;
            case MethodAttributes.FamANDAssem:
                flags.Add("private protected");
                break;
            default:
                flags.Add("private");
                break;
        }

        return flags.Count == 0 ? string.Empty : " [" + string.Join(", ", flags) + "]";
    }
    private static string DescribeFieldFlags(FieldAttributes attributes)
    {
        var flags = new List<string>();
        if ((attributes & FieldAttributes.Static) != 0)
        {
            flags.Add("static");
        }

        if ((attributes & FieldAttributes.Literal) != 0)
        {
            flags.Add("const");
        }

        if ((attributes & FieldAttributes.InitOnly) != 0)
        {
            flags.Add("readonly");
        }

        return flags.Count == 0 ? string.Empty : " [" + string.Join(", ", flags) + "]";
    }
}
