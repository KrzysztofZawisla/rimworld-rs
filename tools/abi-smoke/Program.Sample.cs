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

internal enum Sample
{
    Alpha,
    Beta,
}

/// <summary>
/// A member that exists on both a base type and its subclass: the cache is keyed by the type the lookup
/// starts at, so each type has to answer with its own member after the other was cached.
/// </summary>
internal class CacheParent
{
    public string Value => "parent";
}

internal sealed class CacheChild : CacheParent
{
    public new string Value => "child";
}
