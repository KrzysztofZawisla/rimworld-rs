using System;
using System.Collections.Generic;
using System.Linq;
using System.Text.Json;
using RimWorld;
using RimWorld.Planet;
using Verse;
namespace RustBridge;

/// <summary>
/// The `game` half of <see cref="WorldOps"/>.
/// </summary>
internal static partial class WorldOps
{

    /// <summary>
    /// Pauses or unpauses through the game's own `TickManager.TogglePaused` — the same call the player's
    /// spacebar makes — and answers with the speed and paused state the game reports afterwards.
    /// </summary>
    public static object TogglePause(JsonElement request)
    {
        TickManager ticks = Find.TickManager;
        if (ticks == null)
        {
            throw new BridgeException("there is no tick manager (no game is running)");
        }

        ticks.TogglePaused();
        return new Dictionary<string, object>
        {
            ["speed"] = ticks.CurTimeSpeed.ToString(),
            ["paused"] = ticks.Paused,
        };
    }

    /// <summary>
    /// Sets the game's own time speed through `TickManager.CurTimeSpeed` — the same values the player's
    /// buttons use — and answers with what the game reports. The name is matched against the game's own enum,
    /// and a name it does not have is refused **with the values it does**.
    /// </summary>
    public static object SetSpeed(JsonElement request)
    {
        string name = JsonArgs.GetString(request, "speed");
        if (!Enum.TryParse(name, ignoreCase: true, out TimeSpeed speed))
        {
            string known = string.Join(", ", Enum.GetNames(typeof(TimeSpeed)));
            throw new BridgeException($"there is no game speed '{name}' (the game has: {known})");
        }

        Find.TickManager.CurTimeSpeed = speed;
        return new Dictionary<string, object>
        {
            ["speed"] = Find.TickManager.CurTimeSpeed.ToString(),
            ["paused"] = Find.TickManager.Paused,
        };
    }

    /// <summary>
    /// Saves the game to a file name through the game's own `GameDataSaveLoader.SaveGame`, and answers with
    /// where the game put it. A failure inside the game is reported as the game reports it.
    /// </summary>
    public static object SaveGame(JsonElement request)
    {
        string file = JsonArgs.GetString(request, "file");
        if (file.Length == 0)
        {
            throw new BridgeException("saving needs a file name");
        }

        // Found in game: saving while the map is still generating hangs the whole game (no error, no log —
        // the process just stops). So the op refuses unless the game is actually playing.
        if (Current.ProgramState != ProgramState.Playing || Current.Game == null)
        {
            throw new BridgeException(
                $"the game is not playing yet ({Current.ProgramState}), so a save now would hang it — try again from a running colony");
        }

        GameDataSaveLoader.SaveGame(file);
        return new Dictionary<string, object>
        {
            ["file"] = file,
            ["saved"] = true,
        };
    }
    public static void FireIncident(JsonElement request)
    {
        string defName = JsonArgs.GetString(request, "defName");
        IncidentDef def = DefDatabase<IncidentDef>.GetNamedSilentFail(defName)
            ?? throw new BridgeException($"unknown IncidentDef '{defName}'");

        Map map = Find.AnyPlayerHomeMap ?? Find.CurrentMap;
        if (map == null)
        {
            throw new BridgeException("there is no player map to fire an incident on");
        }

        var parms = new IncidentParms { target = map };
        bool fired = false;
        if (Find.Storyteller == null)
        {
            // No storyteller yet (this happens during map generation): the incident goes straight
            // through its worker, and the reason `incident_fired` stays silent is now in the log.
            Log.Warning($"[Rust] firing '{defName}' without a storyteller (there is none yet), so no incident_fired event fires");
        }
        else
        {
            try
            {
                fired = Find.Storyteller.TryFire(new FiringIncident(def, null, parms));
            }
            catch (Exception error)
            {
                Log.Warning($"[Rust] the storyteller failed to fire '{defName}': {error.Message}");
            }
        }

        if (!fired && def.Worker != null)
        {
            // A storyteller refuses incidents by its own rules (points, cooldowns, target tags) —
            // in game, AmbrosiaSprout is one of them. A mod that asks for an incident explicitly gets
            // it anyway, straight through the incident worker.
            fired = def.Worker.TryExecute(parms);
        }

        if (!fired)
        {
            throw new BridgeException($"'{defName}' could not start (the storyteller refused and its worker did too)");
        }
    }
}
