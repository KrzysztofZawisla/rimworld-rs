using System;
using System.Collections.Generic;
using System.IO;
using System.Runtime.InteropServices;
using System.Text;
using System.Text.Json;
using Verse;

namespace RustBridge;

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

/// <summary>
/// Owns the native Rust library of one mod: loads it, keeps the host callback alive and
/// routes events and host calls between RimWorld and Rust.
/// </summary>
internal sealed class RustHost
{
    public const uint SupportedAbiVersion = 2;

    private static readonly HostCallFn HostCallThunk = HostCall;

    public static RustHost Current { get; private set; }

    private readonly DispatchFn dispatchFn;
    private readonly FreeBufFn freeBufFn;

    public string ModName { get; }
    public int TickInterval = 1;

    /// <summary>
    /// True while the game scribes a save and Rust handles `game_saving`: the game is
    /// mid-serialization, so host operations that change it are refused.
    /// </summary>
    public bool InsideScribe { get; internal set; }

    /// <summary>True while Rust may draw: the map draw callback is running right now.</summary>
    public bool InsideDraw { get; internal set; }

    /// <summary>Whether the mod asked for `on_draw` with `draw_enable(true)`.</summary>
    public bool DrawingEnabled { get; internal set; }

    private long ticksSeen;

    private RustHost(string modName, DispatchFn dispatchFn, FreeBufFn freeBufFn)
    {
        ModName = modName;
        this.dispatchFn = dispatchFn;
        this.freeBufFn = freeBufFn;
    }

    public static RustHost TryCreate(string modName, string rootDir)
    {
        try
        {
            return Create(modName, rootDir);
        }
        catch (Exception ex)
        {
            Log.Error($"[Rust] {modName}: could not load the Rust library: {ex}");
            return null;
        }
    }

    private static RustHost Create(string modName, string rootDir)
    {
        string rustDir = Path.Combine(rootDir, "Rust");
        if (!Directory.Exists(rustDir))
        {
            Log.Error($"[Rust] {modName}: the mod has no 'Rust' folder ({rustDir})");
            return null;
        }

        string[] files = Directory.GetFiles(rustDir, "*.dll");
        if (files.Length != 1)
        {
            Log.Error($"[Rust] {modName}: expected exactly one native library in {rustDir}, found {files.Length}");
            return null;
        }

        string nativePath = files[0];
        IntPtr library = NativeLibrary.Load(nativePath);

        TypeHooks.Install();

        var abiVersion = Marshal.GetDelegateForFunctionPointer<AbiVersionFn>(
            NativeLibrary.GetExport(library, "rwrs_abi_version"));
        uint abi = abiVersion();
        if (abi != SupportedAbiVersion)
        {
            Log.Error($"[Rust] {modName}: native library ABI {abi} does not match the bridge ABI {SupportedAbiVersion}; rebuild the mod");
            NativeLibrary.Free(library);
            return null;
        }

        // Struct layouts must agree exactly: a mismatch corrupts memory silently otherwise.
        var abiLayout = Marshal.GetDelegateForFunctionPointer<AbiLayoutFn>(
            NativeLibrary.GetExport(library, "rwrs_abi_layout"));
        uint sizeOfBuf = (uint)Marshal.SizeOf<NativeBuf>();
        uint offsetOfLength = (uint)Marshal.OffsetOf<NativeBuf>(nameof(NativeBuf.Len)).ToInt64();
        uint sizeOfHostApi = (uint)(2 * IntPtr.Size);
        ulong expectedLayout = sizeOfBuf | ((ulong)offsetOfLength << 16) | ((ulong)sizeOfHostApi << 32);
        ulong layout = abiLayout();
        if (layout != expectedLayout)
        {
            Log.Error($"[Rust] {modName}: ABI layout mismatch (native 0x{layout:x}, bridge 0x{expectedLayout:x}); rebuild the SDK and the bridge together");
            NativeLibrary.Free(library);
            return null;
        }

        var init = Marshal.GetDelegateForFunctionPointer<InitFn>(NativeLibrary.GetExport(library, "rwrs_init"));
        var dispatch = Marshal.GetDelegateForFunctionPointer<DispatchFn>(NativeLibrary.GetExport(library, "rwrs_dispatch"));
        var freeBuf = Marshal.GetDelegateForFunctionPointer<FreeBufFn>(NativeLibrary.GetExport(library, "rwrs_free_buf"));

        IntPtr hostCall = Marshal.GetFunctionPointerForDelegate(HostCallThunk);
        IntPtr hostApi = Marshal.AllocHGlobal(2 * IntPtr.Size);
        bool keepLibrary = false;
        bool keepHostApi = false;
        try
        {
            Marshal.WriteIntPtr(hostApi, IntPtr.Zero);
            Marshal.WriteIntPtr(hostApi + IntPtr.Size, hostCall);

            uint initResult = init(hostApi);
            HostCallBuffers.Flush();
            if (initResult != 0)
            {
                Log.Error($"[Rust] {modName}: rwrs_init failed with code {initResult}");
                return null;
            }

            var host = new RustHost(modName, dispatch, freeBuf);
            Current = host;
            Log.Message($"[Rust] {modName}: loaded {Path.GetFileName(nativePath)}");
            keepLibrary = true;
            keepHostApi = true;
            return host;
        }
        finally
        {
            if (!keepHostApi)
            {
                Marshal.FreeHGlobal(hostApi);
            }

            if (!keepLibrary)
            {
                NativeLibrary.Free(library);
            }
        }
    }

    public bool ShouldDispatchTick()
    {
        ticksSeen++;
        return TickInterval <= 1 || ticksSeen % TickInterval == 0;
    }

    /// <summary>
    /// Dispatches an event and reports handler errors. The payload must be plain data — anything that
    /// carries a game object needs <see cref="DispatchWithPayload"/>.
    /// </summary>
    public void DispatchVoid(string eventName, object payload)
    {
        using JsonDocument document = Dispatch(eventName, payload);
        ReportErrors(document, eventName);
    }

    /// <summary>
    /// Dispatches an event whose payload is plain data (strings, numbers, booleans, dictionaries,
    /// lists).
    ///
    /// A live game object here is a bug: this path serializes with `System.Text.Json`, which walks the
    /// whole object graph — the game's own render and material types contain ref structs and throw —
    /// so a payload that cannot be serialized is refused with a logged error instead of being thrown
    /// into whatever game code called the bridge (a spawn, a gen step, a tick). Use
    /// <see cref="DispatchWithPayload"/> for anything that holds a game object: it turns them into
    /// handles.
    /// </summary>
    public JsonDocument Dispatch(string eventName, object payload)
    {
        byte[] bytes;
        try
        {
            bytes = payload == null ? Array.Empty<byte>() : JsonSerializer.SerializeToUtf8Bytes(payload);
        }
        catch (Exception error)
        {
            Log.Error(
                $"[Rust] the payload of '{eventName}' is not plain data ({error.GetType().Name}: {error.Message}); game objects must travel through DispatchWithPayload");
            return JsonDocument.Parse("{\"ok\":true}");
        }

        return DispatchBytes(eventName, bytes);
    }

    /// <summary>
    /// Dispatches an event whose payload contains live game objects, as a **request** the mod must
    /// answer immediately (gizmos, inspect lines): they travel as handles that stay valid for the
    /// whole handler.
    ///
    /// Refuses while the Rust side is busy (we are inside a host call it made): the answer cannot be
    /// waited for, and dispatching anyway would re-enter the native library on the thread that already
    /// holds the module lock and freeze the game. Use <see cref="SendWithPayload"/> for notifications,
    /// which queue instead.
    /// </summary>
    public JsonDocument DispatchWithPayload(string eventName, object payload)
    {
        if (RefContext.IsDispatching)
        {
            Log.Error(
                $"[Rust] cannot ask the mod for '{eventName}' while it is inside a host call (a request is answered immediately); answering with nothing");
            return JsonDocument.Parse("{\"ok\":true}");
        }

        RefTable refs = new RefTable();
        using var stream = new MemoryStream();
        using (var writer = new Utf8JsonWriter(stream))
        {
            JsonValueBridge.Write(writer, payload, refs);
        }

        return DispatchBytes(eventName, stream.ToArray(), refs);
    }

    /// <summary>
    /// Sends an event that **notifies** the mod and carries live game objects (spawn, despawn, tick).
    ///
    /// A thing can spawn, despawn or tick *inside* a host call — that is what spawning from Rust and
    /// the game's own generation do — and dispatching then would re-enter the native library on the
    /// same thread, which already holds the module lock: the game freezes. So while the Rust side is
    /// busy the event is queued with its handle table and runs right after the current event, exactly
    /// like a nested patch call.
    /// </summary>
    public JsonDocument SendWithPayload(string eventName, object payload)
    {
        RefTable refs = new RefTable();
        byte[] bytes;
        using (var stream = new MemoryStream())
        {
            using (var writer = new Utf8JsonWriter(stream))
            {
                JsonValueBridge.Write(writer, payload, refs);
            }

            bytes = stream.ToArray();
        }

        if (RefContext.IsDispatching)
        {
            QueueNestedEvent(eventName, bytes, refs);
            return JsonDocument.Parse("{\"ok\":true}");
        }

        return DispatchBytes(eventName, bytes, refs);
    }

    public JsonDocument DispatchBytes(string eventName, byte[] payload)
    {
        return DispatchBytes(eventName, payload, null);
    }

    /// <summary>
    /// Dispatches an event whose payload was built with an existing handle table (patch calls and
    /// companion events keep the objects of the payload alive for the handler).
    /// </summary>
    public JsonDocument DispatchBytes(string eventName, byte[] payload, RefTable refs)
    {
        // A host call from inside a Rust handler may trigger a patched method. The Rust side is
        // busy and must not be re-entered, so the patch call is queued and the original method
        // runs unmodified; the queued handlers run right after the current event finishes.
        if (eventName == "patch" && RefContext.IsDispatching)
        {
            return JsonDocument.Parse("{\"ok\":true}");
        }

        JsonDocument document;
        using (RefContext.Enter(refs))
        {
            document = DispatchCore(eventName, payload);
        }

        DrainQueuedEvents();
        return document;
    }

    private JsonDocument DispatchCore(string eventName, byte[] payload)
    {
        byte[] name = Encoding.UTF8.GetBytes(eventName);
        IntPtr namePtr = ToNative(name);
        IntPtr payloadPtr = payload.Length > 0 ? ToNative(payload) : IntPtr.Zero;
        IntPtr slot = Marshal.AllocHGlobal(2 * IntPtr.Size);
        string json;
        try
        {
            uint code = dispatchFn(namePtr, name.Length, payloadPtr, payload.Length, slot);
            if (code != 0)
            {
                json = "{\"ok\":false,\"error\":\"rwrs_dispatch failed\"}";
            }
            else
            {
                var buffer = new NativeBuf
                {
                    Ptr = Marshal.ReadIntPtr(slot),
                    Len = (uint)Marshal.ReadInt32(slot + IntPtr.Size),
                };
                json = CopyAndFree(buffer);
            }
        }
        finally
        {
            Marshal.FreeHGlobal(slot);
            Marshal.FreeHGlobal(namePtr);
            if (payloadPtr != IntPtr.Zero)
            {
                Marshal.FreeHGlobal(payloadPtr);
            }
        }

        HostCallBuffers.Flush();
        return JsonDocument.Parse(json);
    }

    /// <summary>An event that arrived while the Rust side was busy.</summary>
    internal readonly struct QueuedEvent
    {
        public QueuedEvent(string eventName, byte[] payload, RefTable refs)
        {
            EventName = eventName;
            Payload = payload;
            Refs = refs;
        }

        public string EventName { get; }

        public byte[] Payload { get; }

        public RefTable Refs { get; }
    }

    [ThreadStatic] private static List<QueuedEvent> queuedEvents;
    private static bool warnedAboutDeferral;
    [ThreadStatic] private static HashSet<string> deliveredDeferred;

    /// <summary>
    /// How many deferred events one dispatch may hold. A single map-generated dispatch runs a whole
    /// suite of host calls, so this must be generous: reaching it used to drop events silently, which
    /// lost `incident_fired` while its sibling `letter_shown` from the same call arrived.
    /// </summary>
    private const int MaxQueuedEvents = 4096;
    [ThreadStatic] private static HashSet<string> droppedDeferred;

    /// <summary>
    /// Queues a patch call that a host call triggered from inside a Rust handler. The handler runs
    /// after the current event, without the ability to change the call that already happened.
    /// </summary>
    internal static void QueueNestedPatch(byte[] payload, RefTable refs)
    {
        QueueNestedEvent("patch", payload, refs);
    }

    /// <summary>Queues any event that cannot run right now because the Rust side is busy.</summary>
    internal static void QueueNestedEvent(string eventName, byte[] payload, RefTable refs)
    {
        queuedEvents ??= new List<QueuedEvent>();
        if (queuedEvents.Count >= MaxQueuedEvents)
        {
            droppedDeferred ??= new HashSet<string>(StringComparer.Ordinal);
            if (droppedDeferred.Add(eventName))
            {
                Log.Warning(
                    $"[Rust] the queue of deferred events is full ({MaxQueuedEvents}); dropping '{eventName}' — an event is lost, so the mod never sees it");
            }

            return;
        }

        queuedEvents.Add(new QueuedEvent(eventName, payload, refs));
        if (!warnedAboutDeferral)
        {
            warnedAboutDeferral = true;
            Log.Warning(
                $"[Rust] a host call from inside a handler triggered {DescribeDeferred(eventName, payload)}; it is queued and runs after the current event finishes, so it cannot change the call that triggered it");
        }
    }

    /// <summary>Names what was deferred, for the one-time warning.</summary>
    private static string DescribeDeferred(string eventName, byte[] payload)
    {
        string details = null;
        try
        {
            if (payload != null && payload.Length > 0)
            {
                using JsonDocument document = JsonDocument.Parse(payload);
                JsonElement root = document.RootElement;
                if (root.ValueKind == JsonValueKind.Object)
                {
                    if (root.TryGetProperty("target", out JsonElement target) && target.ValueKind == JsonValueKind.String)
                    {
                        details = target.GetString();
                    }
                    else if (root.TryGetProperty("name", out JsonElement name) && name.ValueKind == JsonValueKind.String)
                    {
                        details = $"'{name.GetString()}' from a companion";
                    }
                    else if (root.TryGetProperty("handler", out JsonElement handler) && handler.ValueKind == JsonValueKind.String)
                    {
                        details = $"handler '{handler.GetString()}'";
                    }
                }
            }
        }
        catch (JsonException)
        {
            details = null;
        }

        return details == null ? $"'{eventName}'" : $"'{eventName}' ({details})";
    }

    private void DrainQueuedEvents()
    {
        if (queuedEvents == null || queuedEvents.Count == 0)
        {
            return;
        }

        int guard = 0;
        while (queuedEvents.Count > 0 && guard++ < MaxQueuedEvents)
        {
            QueuedEvent queued = queuedEvents[0];
            queuedEvents.RemoveAt(0);
            using (RefContext.Enter(queued.Refs))
            {
                using JsonDocument document = DispatchCore(queued.EventName, queued.Payload);
                ReportErrors(document, queued.EventName + " (deferred)");
                AnnounceDelivery(queued.EventName);
            }
        }

        if (queuedEvents.Count > 0)
        {
            Log.Warning(
                $"[Rust] the queue of deferred events still holds {queuedEvents.Count} event(s) after draining {MaxQueuedEvents}; they are dropped — an event is lost, so the mod never sees it");
        }

        queuedEvents.Clear();
    }

    /// <summary>Confirms once per event name that a deferred event reached the mod, so a missing
    /// event can be told apart from an event that never arrived.</summary>
    private static void AnnounceDelivery(string eventName)
    {
        deliveredDeferred ??= new HashSet<string>(StringComparer.Ordinal);
        if (deliveredDeferred.Add(eventName))
        {
            Log.Message(
                $"[Rust] the queued event '{eventName}' reached the mod (it is delivered after the call that raised it)");
        }
    }

    private string CopyAndFree(NativeBuf buffer)
    {
        if (buffer.Ptr == IntPtr.Zero)
        {
            if (buffer.Len != 0)
            {
                Log.Error($"[Rust] the native response buffer has length {buffer.Len} but no pointer");
                return "{\"ok\":false,\"error\":\"native response buffer has no pointer\"}";
            }

            return "{\"ok\":true}";
        }

        try
        {
            if (buffer.Len == 0)
            {
                return "{\"ok\":true}";
            }

            if (buffer.Len > int.MaxValue)
            {
                Log.Error(
                    $"[Rust] the native response buffer is {buffer.Len} bytes, larger than the bridge can copy");
                return "{\"ok\":false,\"error\":\"native response buffer too large\"}";
            }

            var bytes = new byte[(int)buffer.Len];
            Marshal.Copy(buffer.Ptr, bytes, 0, bytes.Length);
            return Encoding.UTF8.GetString(bytes);
        }
        finally
        {
            freeBufFn(buffer.Ptr, buffer.Len);
        }
    }

    private static IntPtr ToNative(byte[] bytes)
    {
        IntPtr memory = Marshal.AllocHGlobal(bytes.Length);
        Marshal.Copy(bytes, 0, memory, bytes.Length);
        return memory;
    }

    private static void ReportErrors(JsonDocument document, string eventName)
    {
        if (document == null || document.RootElement.ValueKind != JsonValueKind.Object)
        {
            return;
        }

        if (document.RootElement.TryGetProperty("ok", out JsonElement ok) && ok.ValueKind == JsonValueKind.False)
        {
            string error = document.RootElement.TryGetProperty("error", out JsonElement message)
                ? message.GetString()
                : "unknown error";
            Log.Error($"[Rust] error while handling '{eventName}': {error}");
        }
    }

    private static uint HostCall(IntPtr userData, IntPtr requestPtr, int requestLen, IntPtr responseOut)
    {
        byte[] response;
        try
        {
            string request = requestLen > 0
                ? Encoding.UTF8.GetString(Read(requestPtr, requestLen))
                : "{}";
            response = Encoding.UTF8.GetBytes(HostOps.Handle(request));
        }
        catch (Exception ex)
        {
            string escaped = JsonSerializer.Serialize(ex.Message);
            response = Encoding.UTF8.GetBytes($"{{\"ok\":false,\"error\":{escaped}}}");
        }

        try
        {
            HostCallBuffers.Publish(responseOut, response);
            return 0;
        }
        catch (Exception error)
        {
            Log.Error($"[Rust] could not publish a host response: {error}");
            return 1;
        }
    }

    private static byte[] Read(IntPtr pointer, int length)
    {
        var bytes = new byte[length];
        Marshal.Copy(pointer, bytes, 0, length);
        return bytes;
    }
}

/// <summary>
/// Host call responses are allocated by the bridge and are only guaranteed to stay alive until
/// the next host call or the end of the current dispatch. The previous buffer is therefore
/// released as soon as a new one is produced, and <see cref="Flush"/> releases the last one.
/// </summary>
internal static class HostCallBuffers
{
    [ThreadStatic] private static IntPtr latest;

    public static void Publish(IntPtr responseOut, byte[] response)
    {
        Flush();
        IntPtr memory = Marshal.AllocHGlobal(response.Length);
        try
        {
            Marshal.Copy(response, 0, memory, response.Length);
            Marshal.WriteIntPtr(responseOut, memory);
            Marshal.WriteInt32(responseOut + IntPtr.Size, response.Length);
            latest = memory;
        }
        catch
        {
            Marshal.FreeHGlobal(memory);
            throw;
        }
    }

    public static void Flush()
    {
        if (latest != IntPtr.Zero)
        {
            Marshal.FreeHGlobal(latest);
            latest = IntPtr.Zero;
        }
    }
}
