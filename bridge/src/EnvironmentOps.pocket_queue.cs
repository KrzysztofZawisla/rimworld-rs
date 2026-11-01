using System;
using System.Collections.Generic;
using System.Text.Json;
using RimWorld;
using RimWorld.Planet;
using Verse;

namespace RustBridge;

/// <summary>
/// The queued half of <see cref="EnvironmentOps"/>: a pocket map is generated on the game's own
/// long-event thread, and the caller polls a token instead of blocking the tick that asked for it.
/// </summary>
internal static partial class EnvironmentOps
{
    private sealed class PocketJob
    {
        public int Token;
        public string State = "queued";
        public int MapId = -1;
        public string Error;
    }

    private static readonly Dictionary<int, PocketJob> PocketJobs = new();
    private static readonly object PocketJobsLock = new();
    private static int nextPocketJob;

    /// <summary>
    /// Queues one pocket-map generation and answers with its token at once. The map itself is made by
    /// `LongEventHandler`, exactly like the game's own maps; poll `map_generation_status` for it.
    /// </summary>
    public static object QueuePocketMap(JsonElement request)
    {
        (int size, MapGeneratorDef generator, PocketMapParent parent) = PreparePocketMap(request);
        int token;
        lock (PocketJobsLock)
        {
            token = ++nextPocketJob;
            PocketJobs[token] = new PocketJob { Token = token };
        }

        LongEventHandler.QueueLongEvent(
            () => RunPocketJob(token, size, generator, parent),
            "GeneratingMap",
            doAsynchronously: true,
            exceptionHandler: error => FailPocketJob(token, error),
            showExtraUIInfo: false,
            forceHideUI: false,
            callback: null);
        return new Dictionary<string, object> { ["token"] = token, ["state"] = "queued" };
    }

    /// <summary>Answers the state of one queued job: `queued`, `running`, `done` or `failed`.</summary>
    public static object PocketStatus(JsonElement request)
    {
        int token = JsonArgs.GetInt(request, "token", -1);
        lock (PocketJobsLock)
        {
            if (!PocketJobs.TryGetValue(token, out PocketJob job))
            {
                throw new BridgeException($"there is no pocket-map job {token}");
            }

            return new Dictionary<string, object>
            {
                ["token"] = job.Token,
                ["state"] = job.State,
                ["mapId"] = job.MapId,
                ["error"] = job.Error,
            };
        }
    }

    private static void RunPocketJob(int token, int size, MapGeneratorDef generator, PocketMapParent parent)
    {
        try
        {
            Map map = GeneratePocketMapNow(size, generator, parent);
            lock (PocketJobsLock)
            {
                PocketJobs[token].State = "done";
                PocketJobs[token].MapId = map.uniqueID;
            }
        }
        catch (Exception error)
        {
            FailPocketJob(token, error);
        }
    }

    private static void FailPocketJob(int token, Exception error)
    {
        lock (PocketJobsLock)
        {
            if (PocketJobs.TryGetValue(token, out PocketJob job))
            {
                job.State = "failed";
                job.Error = $"{error.GetType().Name}: {error.Message}";
            }
        }
    }
}
