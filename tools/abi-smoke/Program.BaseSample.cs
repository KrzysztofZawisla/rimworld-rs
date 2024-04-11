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

internal partial class BaseSample
{
    private int secret = 3;

    public string Label { get; set; } = "base";

    public Sample Mode { get; set; } = Sample.Alpha;

    protected string Describe() => $"secret={secret}";

    public string Describe(string value) => $"value={value}";

    public string Describe(string value, int repeat) => $"{value}x{repeat}";

    public string Pick(int value) => "int";

    public string Pick(string value) => "string";

    public string Take(System.Collections.Generic.List<int> values) => $"count={values.Count}";

    public int Bump(ref int value) => ++value;

    public string InstanceSource() => "source";

    public string InstanceReplacement() => "replacement";

    public string Unrelated() => "unrelated";

    public int WithDefault(int value = 42) => value;
}
