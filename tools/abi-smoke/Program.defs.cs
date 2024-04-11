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
/// The `defs` half of <see cref="Program"/>.
/// </summary>
internal static partial class Program
{

    /// <summary>
    /// Verifies every def name declared in `bridge/names.json` against the installed game's XML
    /// defs, so a renamed def in a game update fails here instead of inside the game.
    /// </summary>
    /// <summary>
    /// Reads the def names a game installation (and optionally a mod folder) declares and either
    /// prints them or writes a Rust module of constants. This is the single place that parses def
    /// XML: `cargo xtask defs` and `cargo xtask gen-defs` both go through it.
    /// </summary>
    private static int RunDefScan(
        string gameDirectory,
        string extraRoot,
        string outPath,
        string typeFilter,
        string nameFilter,
        bool emit,
        bool countsOnly)
    {
        var roots = new List<string>();
        if (!string.IsNullOrEmpty(gameDirectory))
        {
            string data = Path.Combine(gameDirectory, "Data");
            if (Directory.Exists(data))
            {
                roots.Add(data);
            }
            else
            {
                Console.Error.WriteLine($"no Data folder in {gameDirectory}; is this a RimWorld installation?");
                return 2;
            }
        }

        if (!string.IsNullOrEmpty(extraRoot) && Directory.Exists(extraRoot))
        {
            roots.Add(extraRoot);
        }

        if (roots.Count == 0)
        {
            Console.Error.WriteLine("usage: abi-smoke --emit-defs|--list-defs [--game <RimWorld folder>] [--root <folder>] [--types A,B] [--filter text] [--out <file>]");
            return 2;
        }

        var types = string.IsNullOrEmpty(typeFilter)
            ? null
            : new HashSet<string>(typeFilter.Split(',', StringSplitOptions.RemoveEmptyEntries | StringSplitOptions.TrimEntries), StringComparer.Ordinal);

        // Def classes are grouped case-insensitively: the game's own XML contains both `JobDef` and
        // `jobDef` for the same type (verified in the installed data), and they must land in one
        // module instead of two.
        var found = new SortedDictionary<string, SortedSet<string>>(StringComparer.OrdinalIgnoreCase);
        var spellings = new SortedDictionary<string, SortedSet<string>>(StringComparer.OrdinalIgnoreCase);
        int files = 0;
        foreach (string root in roots)
        {
            foreach (string file in Directory.EnumerateFiles(root, "*.xml", SearchOption.AllDirectories))
            {
                string text;
                try
                {
                    text = File.ReadAllText(file);
                }
                catch (IOException)
                {
                    continue;
                }

                if (!text.Contains("Def", StringComparison.Ordinal) || !text.Contains("<defName>", StringComparison.Ordinal))
                {
                    continue;
                }

                files++;
                XDocument document;
                try
                {
                    document = XDocument.Parse(text);
                }
                catch (Exception error)
                {
                    Console.Error.WriteLine($"  (skipping malformed {file}: {error.Message})");
                    continue;
                }

                foreach (XElement element in document.Descendants())
                {
                    string defClass = element.Name.LocalName;
                    if (!defClass.EndsWith("Def", StringComparison.Ordinal))
                    {
                        continue;
                    }

                    if (types != null && !types.Contains(defClass))
                    {
                        continue;
                    }

                    string value = element.Element("defName")?.Value;
                    if (string.IsNullOrWhiteSpace(value))
                    {
                        continue;
                    }

                    if (!string.IsNullOrEmpty(nameFilter) && !value.Contains(nameFilter, StringComparison.OrdinalIgnoreCase))
                    {
                        continue;
                    }

                    if (!found.TryGetValue(defClass, out SortedSet<string> names))
                    {
                        names = new SortedSet<string>(StringComparer.Ordinal);
                        found[defClass] = names;
                        spellings[defClass] = new SortedSet<string>(StringComparer.Ordinal);
                    }

                    spellings[defClass].Add(defClass);
                    names.Add(value);
                }
            }
        }

        int total = found.Sum(entry => entry.Value.Count);
        Console.WriteLine($"def types : {found.Count}");
        Console.WriteLine($"def names : {total}");
        Console.WriteLine($"xml files : {files}");

        if (!emit)
        {
            if (countsOnly)
            {
                foreach ((string defClass, SortedSet<string> names) in found)
                {
                    Console.WriteLine($"  {defClass}: {names.Count}");
                }

                return 0;
            }

            foreach ((string defClass, SortedSet<string> names) in found)
            {
                Console.WriteLine();
                Console.WriteLine($"{defClass} ({names.Count}):");
                foreach (string name in names)
                {
                    Console.WriteLine($"  {name}");
                }
            }

            return 0;
        }

        if (string.IsNullOrEmpty(outPath))
        {
            Console.Error.WriteLine("--emit-defs needs --out <file>");
            return 2;
        }

        string source = string.IsNullOrEmpty(extraRoot) ? gameDirectory ?? "?" : $"{gameDirectory} + {extraRoot}";
        File.WriteAllText(outPath, BuildDefsModule(found, spellings, source));
        Console.WriteLine($"written   : {Path.GetFullPath(outPath)}");
        return 0;
    }

    /// <summary>
    /// Indexes the types an assembly defines: full name (nested ones as `Outer+Nested`, the way the
    /// game's XML writes them) to the full name of the base type.
    /// </summary>
    private static bool CollectTypes(string assemblyPath, Dictionary<string, string> types, out int added)
    {
        added = 0;
        try
        {
            using FileStream stream = File.OpenRead(assemblyPath);
            using var reader = new PEReader(stream);
            MetadataReader metadata = reader.GetMetadataReader();
            foreach (TypeDefinitionHandle handle in metadata.TypeDefinitions)
            {
                TypeDefinition type = metadata.GetTypeDefinition(handle);
                string name = TypeFullName(metadata, type);
                if (!types.ContainsKey(name))
                {
                    added++;
                }

                types[name] = type.BaseType.IsNil ? null : TypeReferenceName(metadata, type.BaseType);
            }
        }
        catch (Exception)
        {
            return false;
        }

        return true;
    }
    private static string TypeFullName(MetadataReader metadata, TypeDefinition type)
    {
        var name = new StringBuilder(metadata.GetString(type.Name));
        string ns = metadata.GetString(type.Namespace);
        TypeDefinitionHandle declaring = type.GetDeclaringType();
        while (!declaring.IsNil)
        {
            TypeDefinition outer = metadata.GetTypeDefinition(declaring);
            name.Insert(0, '+').Insert(0, metadata.GetString(outer.Name));
            if (ns.Length == 0)
            {
                ns = metadata.GetString(outer.Namespace);
            }

            declaring = outer.GetDeclaringType();
        }

        return ns.Length == 0 ? name.ToString() : ns + "." + name;
    }
    private static string TypeReferenceName(MetadataReader metadata, EntityHandle handle)
    {
        switch (handle.Kind)
        {
            case HandleKind.TypeDefinition:
                return TypeFullName(metadata, metadata.GetTypeDefinition((TypeDefinitionHandle)handle));
            case HandleKind.TypeReference:
                TypeReference reference = metadata.GetTypeReference((TypeReferenceHandle)handle);
                string name = metadata.GetString(reference.Name);
                string ns = metadata.GetString(reference.Namespace);
                return ns.Length == 0 ? name : ns + "." + name;
            default:
                return null;
        }
    }

    /// <summary>Walks the base types the index knows until it reaches <paramref name="required"/>.</summary>
    private static bool DerivesFrom(Dictionary<string, string> types, string name, string required)
    {
        for (int depth = 0; depth < 60 && name != null; depth++)
        {
            if (string.Equals(name, required, StringComparison.Ordinal))
            {
                return true;
            }

            if (!types.TryGetValue(name, out string baseName))
            {
                return false;
            }

            name = baseName;
        }

        return false;
    }

    /// <summary>Renders the scanned names as Rust modules of constants.</summary>
    private static string BuildDefsModule(
        SortedDictionary<string, SortedSet<string>> found,
        SortedDictionary<string, SortedSet<string>> spellings,
        string source)
    {
        var text = new System.Text.StringBuilder();
        text.AppendLine("//! Def names declared by this game installation, generated by `cargo xtask gen-defs`.");
        text.AppendLine("//!");
        text.AppendLine($"//! Source: {source}");
        text.AppendLine("//!");
        text.AppendLine("//! Do not edit by hand and do not commit: regenerate after the game, its DLCs or the");
        text.AppendLine("//! def-providing mods change. Using these constants instead of strings turns a renamed or");
        text.AppendLine("//! missing def into a compile error.");
        text.AppendLine("//!");
        text.AppendLine("//! Most constants stay unused in any one mod, and def names contain characters that are not");
        text.AppendLine("//! valid Rust identifiers, so the two lints this file would otherwise trigger are allowed.");
        text.AppendLine("#![allow(dead_code, non_snake_case)]");
        text.AppendLine();

        var seenModules = new HashSet<string>(StringComparer.Ordinal);
        var seenConstants = new HashSet<string>(StringComparer.Ordinal);
        foreach ((string defClass, SortedSet<string> names) in found)
        {
            string element = spellings.TryGetValue(defClass, out SortedSet<string> variants) && variants.Count > 1
                ? string.Join(" / ", variants)
                : defClass;
            string module = RustModule(ToSnakeCase(defClass));
            if (!seenModules.Add(module))
            {
                // Defensive only: case variants are already merged above.
                int suffix = 2;
                while (!seenModules.Add($"{module}_{suffix}"))
                {
                    suffix++;
                }

                module = $"{module}_{suffix}";
            }

            text.AppendLine($"/// `{element}` ({names.Count} names).");
            text.AppendLine($"pub mod {module} {{");
            foreach (string name in names)
            {
                string constant = ToConstant(name);
                while (!seenConstants.Add($"{module}.{constant}"))
                {
                    constant += "_ALT";
                }

                text.AppendLine($"    pub const {constant}: &str = \"{EscapeRust(name)}\";");
            }

            text.AppendLine("}");
            text.AppendLine();
        }

        return text.ToString();
    }

    /// <summary>`ThingDef` -> `thing_def`, `RoomPart_BarricadeDef` -> `room_part_barricade_def`.</summary>
    private static string ToSnakeCase(string name)
    {
        var text = new System.Text.StringBuilder();
        for (int index = 0; index < name.Length; index++)
        {
            char current = name[index];
            bool boundary = index > 0
                && char.IsUpper(current)
                && (!char.IsUpper(name[index - 1]) || (index + 1 < name.Length && char.IsLower(name[index + 1])));
            if (boundary && text.Length > 0 && text[text.Length - 1] != '_')
            {
                text.Append('_');
            }

            if (current == '_')
            {
                // never emit two underscores in a row, and never a trailing one
                if (text.Length == 0 || text[text.Length - 1] == '_')
                {
                    continue;
                }
            }

            text.Append(char.ToLowerInvariant(current));
        }

        return text.ToString().TrimEnd('_');
    }

    /// <summary>`WoodLog` -> `WOOD_LOG`, `Apparel_CowboyHat` -> `APPAREL_COWBOY_HAT`, `2x4` -> `_2X4`.</summary>
    private static string ToConstant(string name)
    {
        var text = new System.Text.StringBuilder();
        bool lastWasLower = false;
        foreach (char current in name)
        {
            if (!char.IsLetterOrDigit(current))
            {
                text.Append('_');
                lastWasLower = false;
                continue;
            }

            if (char.IsUpper(current) && lastWasLower)
            {
                text.Append('_');
            }

            text.Append(char.ToUpperInvariant(current));
            lastWasLower = char.IsLower(current) || char.IsDigit(current);
        }

        string constant = text.ToString().Trim('_');
        if (constant.Length == 0 || char.IsDigit(constant[0]))
        {
            constant = "_" + constant;
        }

        // `SELF`, `CRATE` and `SUPER` are uppercase Rust keywords.
        return constant is "SELF" or "CRATE" or "SUPER" ? constant + "_" : constant;
    }
    /// <summary>
    /// A def name as a Rust string literal: the value is data, so a quote, a backslash or a control character in
    /// the game's XML cannot end the literal and turn the rest of the name into Rust source. The name is also the
    /// only source of the constant identifier, and <see cref="ToConstant"/> sanitizes that one separately.
    /// </summary>
    private static string EscapeRust(string value)
    {
        var escaped = new System.Text.StringBuilder(value.Length);
        foreach (char character in value)
        {
            switch (character)
            {
                case '"':
                    escaped.Append("\\\"");
                    break;
                case '\\':
                    escaped.Append("\\\\");
                    break;
                case '\n':
                    escaped.Append("\\n");
                    break;
                case '\r':
                    escaped.Append("\\r");
                    break;
                case '\t':
                    escaped.Append("\\t");
                    break;
                default:
                    if (char.IsControl(character))
                    {
                        escaped.Append("\\u{").Append(((int)character).ToString("x")).Append('}');
                    }
                    else
                    {
                        escaped.Append(character);
                    }

                    break;
            }
        }

        return escaped.ToString();
    }

    /// <summary>
    /// A Rust module identifier from a def class name: the snake-case form is kept when it already is one - the
    /// callers' `defs::thing_def::STEEL` paths depend on it - and anything else (a dot, a dash, a leading digit)
    /// is mapped to `_`, so an unusual XML element name cannot produce `pub mod x._def`.
    /// </summary>
    private static string RustModule(string snakeCase)
    {
        var identifier = new System.Text.StringBuilder(snakeCase.Length);
        foreach (char character in snakeCase)
        {
            identifier.Append(char.IsAsciiLetterOrDigit(character) || character == '_' ? character : '_');
        }

        if (identifier.Length == 0 || char.IsAsciiDigit(identifier[0]))
        {
            identifier.Insert(0, '_');
        }

        return identifier.ToString();
    }

    private static int RunDefNamesCheck(string gameDirectory, string namesPath)
    {
        if (gameDirectory == null || !Directory.Exists(gameDirectory))
        {
            Console.Error.WriteLine("usage: abi-smoke --defs --game <RimWorld folder> [--names bridge/names.json]");
            return 2;
        }

        if (!File.Exists(namesPath))
        {
            Console.Error.WriteLine($"names file not found: {Path.GetFullPath(namesPath)}");
            return 2;
        }

        using JsonDocument namesDocument = JsonDocument.Parse(File.ReadAllText(namesPath));
        var required = new Dictionary<string, List<string>>(StringComparer.Ordinal);
        foreach (JsonProperty classEntry in namesDocument.RootElement.GetProperty("defsByClass").EnumerateObject())
        {
            var list = new List<string>();
            foreach (JsonProperty nameEntry in classEntry.Value.EnumerateObject())
            {
                list.Add(nameEntry.Value.GetString());
            }

            required[classEntry.Name] = list;
        }

        // Collect the def names the game actually declares, for the classes names.json uses.
        var found = new Dictionary<string, SortedSet<string>>(StringComparer.Ordinal);
        string dataFolder = Path.Combine(gameDirectory, "Data");
        if (!Directory.Exists(dataFolder))
        {
            Console.Error.WriteLine($"no Data folder in {gameDirectory}; is this a RimWorld installation?");
            return 2;
        }

        int scanned = 0;
        foreach (string file in Directory.EnumerateFiles(dataFolder, "*.xml", SearchOption.AllDirectories))
        {
            string folder = Path.GetDirectoryName(file) ?? string.Empty;
            if (!folder.Split(Path.DirectorySeparatorChar).Contains("Defs", StringComparer.OrdinalIgnoreCase))
            {
                continue;
            }

            string text = File.ReadAllText(file);
            bool relevant = false;
            foreach (string defClass in required.Keys)
            {
                if (text.Contains("<" + defClass, StringComparison.Ordinal))
                {
                    relevant = true;
                    break;
                }
            }

            if (!relevant)
            {
                continue;
            }

            scanned++;
            XDocument document;
            try
            {
                document = XDocument.Parse(text);
            }
            catch (Exception error)
            {
                Console.Error.WriteLine($"  (skipping malformed {file}: {error.Message})");
                continue;
            }

            foreach (XElement element in document.Descendants())
            {
                string defClass = element.Name.LocalName;
                if (!required.ContainsKey(defClass))
                {
                    continue;
                }

                string defName = element.Element("defName")?.Value;
                if (string.IsNullOrEmpty(defName))
                {
                    continue;
                }

                if (!found.TryGetValue(defClass, out SortedSet<string> declaredNames))
                {
                    declaredNames = new SortedSet<string>(StringComparer.Ordinal);
                    found[defClass] = declaredNames;
                }

                declaredNames.Add(defName);
            }
        }

        Console.WriteLine($"game          : {gameDirectory}");
        Console.WriteLine($"names file    : {Path.GetFullPath(namesPath)}");
        Console.WriteLine($"def files read: {scanned}");

        int total = 0;
        int failures = 0;
        foreach (KeyValuePair<string, List<string>> classEntry in required)
        {
            SortedSet<string> gameNames = found.TryGetValue(classEntry.Key, out SortedSet<string> set)
                ? set
                : new SortedSet<string>(StringComparer.Ordinal);
            Console.WriteLine($"{classEntry.Key} ({gameNames.Count} in game): {string.Join(", ", gameNames)}");
            foreach (string name in classEntry.Value)
            {
                total++;
                if (gameNames.Contains(name))
                {
                    Console.WriteLine($"ok    {classEntry.Key} '{name}'");
                }
                else
                {
                    Console.WriteLine($"FAIL  {classEntry.Key} '{name}' is not declared by the game");
                    failures++;
                }
            }
        }

        Console.WriteLine();
        Console.WriteLine($"{total - failures}/{total} declared def names exist in the game data");
        return failures == 0 ? 0 : 1;
    }
}
