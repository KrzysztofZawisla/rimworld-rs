//! Protocol smoke test: loads a compiled Rust mod library, replaces the game side of the bridge
//! with a mock host and drives the full event surface. It proves that the ABI, the JSON protocol
//! and the dispatch routing work without launching RimWorld.
//!
//! Modes:
//!   dotnet run --project tools/abi-smoke
//!       marshaling self-test (the bridge's JSON layer, compiled into this tool)
//!   dotnet run --project tools/abi-smoke -- <path to a Rust cdylib> [--fixture]
//!       ABI smoke test; --fixture additionally checks patch outcomes and panic recovery
//!   dotnet run --project tools/abi-smoke -- --contract <folder with Assembly-CSharp.dll>
//!       verifies the game still exposes the API the bridge compiles against

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

[StructLayout(LayoutKind.Sequential)]
internal struct NativeBuf
{
    public IntPtr Ptr;
    public uint Len;
}

[UnmanagedFunctionPointer(CallingConvention.Cdecl)]
internal delegate uint AbiVersionFn();

[UnmanagedFunctionPointer(CallingConvention.Cdecl)]
internal delegate ulong AbiLayoutFn();

[UnmanagedFunctionPointer(CallingConvention.Cdecl)]
internal delegate uint InitFn(IntPtr hostApi);

[UnmanagedFunctionPointer(CallingConvention.Cdecl)]
internal delegate uint DispatchFn(IntPtr eventPtr, int eventLen, IntPtr payloadPtr, int payloadLen, IntPtr responseOut);

[UnmanagedFunctionPointer(CallingConvention.Cdecl)]
internal delegate void FreeBufFn(IntPtr ptr, uint len);

[UnmanagedFunctionPointer(CallingConvention.Cdecl)]
internal delegate uint HostCallFn(IntPtr userData, IntPtr requestPtr, int requestLen, IntPtr responseOut);

internal static partial class Program
{
    private const uint ExpectedAbiVersion = 2;

    private static readonly List<string> Log = new();
    private static readonly List<string> Errors = new();
    private static readonly List<string> Operations = new();
    private static readonly HostCallFn HostCallThunk = HostCall;


















    /// <summary>In-memory stand-in for the bridge's save data.</summary>
    private static readonly Dictionary<string, string> SaveStore = new();

    /// <summary>In-memory stand-in for the bridge's mod settings.</summary>
    private static readonly Dictionary<string, string> SettingStore = new();

    /// <summary>The defs the mock host pretends to know, matching the `def_names` answer.</summary>
    private static readonly HashSet<string> KnownDefs = new(StringComparer.Ordinal) { "Steel", "Silver", "WoodLog", "Plant_Potato" };

    /// <summary>Mirrors the bridge: drawing only works while the draw callback runs.</summary>
    private static bool InsideDraw;

    /// <summary>Mirrors the bridge: game-changing operations are refused while a save is written.</summary>
    private static bool InsideScribe;

    /// <summary>Fake objects handed out as `{"$ref": n}` handles, by index.</summary>
    private static readonly List<Dictionary<string, object>> MockObjects = new();

    private static int MockMap = -1;
    private static int MockColonist = -1;

    /// <summary>The mock map's air temperature: `map_push_heat` moves it, so the self-test can read it back.</summary>
    private static float MockCellTemperature = 21f;


    /// <summary>The mock storage's priority, so setting it reads back.</summary>
    private static string MockStoragePriority = "Normal";

    /// <summary>The mock cell's roof and gas, so setting them reads back through `map_cell_info`.</summary>
    private static string MockRoofDefName = null;

    private static readonly Dictionary<string, int> MockGas = new();




    /// <summary>How many cells the last zone operation left the mock's stockpile with.</summary>
    private static int MockZoneCells = 9;

    private static float MockPlantGrowth = 0.5f;


    private const int KnownMapId = 1;


    /// <summary>Handlers of the dialogs the mod asked for; a click would send a `ui_action` event.</summary>
    private static readonly List<string> Dialogs = new();







































































    private static string StaticSource() => "source";

    private static string StaticReplacement() => "replacement";






















    /// <summary>Kept alive for the process so the metadata reader stays valid.</summary>
    private static FileStream openAssemblyStream;
    private static PEReader openAssemblyReader;

































}
