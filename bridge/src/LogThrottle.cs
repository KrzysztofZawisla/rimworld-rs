using System;
using System.Collections.Generic;

namespace RustBridge;

/// <summary>
/// Keeps one mod from drowning <c>Player.log</c>: the same line is written at most
/// <c>perWindow</c> times per <c>window</c>, and how many identical lines followed the last written one is
/// reported with the next line of that key.
///
/// A log written from <c>on_tick</c> otherwise emits one line every tick - with the log being the game's
/// diagnostics channel, the file the smoke gate reads and the stream <c>cargo xtask watch-log</c> follows.
/// The throttle is best-effort: its table is bounded, and past the bound it starts over rather than growing
/// for the life of the process.
/// </summary>
internal sealed class LogThrottle
{
    /// <summary>How many distinct lines are remembered; past it the table is cleared.</summary>
    private const int MaxKeys = 256;

    private readonly TimeSpan window;

    private readonly int perWindow;

    private readonly Dictionary<string, Entry> entries = new(StringComparer.Ordinal);

    public LogThrottle(TimeSpan window, int perWindow)
    {
        this.window = window;
        this.perWindow = perWindow < 1 ? 1 : perWindow;
    }

    /// <summary>The throttle the `log` host operation uses: three identical lines per ten seconds.</summary>
    public static LogThrottle Default { get; } = new LogThrottle(TimeSpan.FromSeconds(10), 3);

    /// <summary>
    /// Whether this line may be written now. <paramref name="suppressed"/> answers how many identical
    /// lines were held back since the last written one, so the caller can say so in the line it writes.
    /// </summary>
    public bool ShouldWrite(string key, DateTime now, out int suppressed)
    {
        suppressed = 0;
        if (!entries.TryGetValue(key, out Entry entry))
        {
            Add(key, now);
            return true;
        }

        if (now - entry.WindowStart >= window)
        {
            suppressed = entry.Suppressed;
            Add(key, now);
            return true;
        }

        if (entry.Written < perWindow)
        {
            entry.Written++;
            return true;
        }

        entry.Suppressed++;
        return false;
    }

    private void Add(string key, DateTime now)
    {
        if (entries.Count >= MaxKeys)
        {
            entries.Clear();
        }

        entries[key] = new Entry { WindowStart = now, Written = 1 };
    }

    private sealed class Entry
    {
        public DateTime WindowStart;

        public int Written;

        public int Suppressed;
    }
}
