using System;
using System.Collections.Generic;
using System.Linq;
using System.Text.Json;
using RimWorld;
using UnityEngine;
using Verse;
using Verse.Sound;

namespace RustBridge;

/// <summary>
/// What a mod shows and hears: one-shot sounds, the game's translations for a key (a mod ships its own
/// `Languages/&lt;language&gt;/Keyed/*.xml` and reads it with `ctx.translate`), and drawing a texture at
/// a cell from the draw callback. Every game member reached here is in the bridge contract check.
/// </summary>
internal static class Presentation
{
    public static void SoundPlay(JsonElement request)
    {
        string defName = JsonArgs.GetString(request, "sound");
        SoundDef def = DefDatabase<SoundDef>.GetNamedSilentFail(defName)
            ?? throw new BridgeException($"the sound def '{defName}' does not exist");

        int mapId = JsonArgs.GetInt(request, "mapId", -1);
        SoundInfo info;
        if (mapId >= 0)
        {
            Map map = Current.Game?.Maps?.FirstOrDefault(candidate => candidate.uniqueID == mapId)
                ?? throw new BridgeException($"map {mapId} does not exist");
            var cell = new IntVec3(JsonArgs.GetInt(request, "x", 0), 0, JsonArgs.GetInt(request, "z", 0));
            info = SoundInfo.InMap(new TargetInfo(cell, map), MaintenanceType.None);
        }
        else
        {
            info = SoundInfo.OnCamera(MaintenanceType.None);
        }

        SoundStarter.PlayOneShot(def, info);
    }

    public static object Translate(JsonElement request)
    {
        string key = JsonArgs.GetString(request, "key");
        if (string.IsNullOrEmpty(key))
        {
            throw new BridgeException("translate needs a key (the text a mod ships in Languages/<language>/Keyed/*.xml)");
        }

        return new Dictionary<string, object>
        {
            ["text"] = Translator.TranslateSimple(key) ?? key,
            ["translated"] = Translator.CanTranslate(key),
            ["language"] = LanguageDatabase.activeLanguage?.folderName,
        };
    }

    /// <summary>Draws a texture (the game's or the mod's own, by path) at a cell; only inside `on_draw`.</summary>
    public static void DrawTexture(JsonElement request)
    {
        HostOps.RequireDraw("draw_texture");

        string path = JsonArgs.GetString(request, "texture");
        if (string.IsNullOrEmpty(path))
        {
            throw new BridgeException("draw_texture needs a 'texture' path");
        }

        // The engine cannot resolve every path (the base game's textures are not reachable through
        // ContentFinder on this build), so a miss is reported as a warning instead of a refusal.
        if (ContentFinder<Texture2D>.Get(path, reportFailure: false) == null)
        {
            Log.Warning(
                $"[Rust] no texture found at '{path}' (paths are relative to a mod's Textures folder, without the extension: 'MyMod/Icon'); the frame will draw nothing there");
        }


        if (!request.TryGetProperty("cell", out JsonElement cellElement))
        {
            throw new BridgeException("draw_texture needs a 'cell' handle (create one with ctx.vec3)");
        }

        var cell = (IntVec3)JsonValueBridge.Read(cellElement, typeof(IntVec3), RefContext.Current);
        float size = (float)JsonArgs.GetDouble(request, "size", 1.0);
        float altitude = (float)JsonArgs.GetDouble(request, "altitude", Altitude);

        Material material = MaterialPool.MatFrom(path, ShaderDatabase.MetaOverlay);
        if (material == null)
        {
            throw new BridgeException($"cannot load the texture '{path}' from the mod or the game");
        }

        Vector3 position = cell.ToVector3Shifted();
        position.y = altitude;
        GenDraw.DrawQuad(material, position, Quaternion.identity, size, null);
    }

    /// <summary>Just above the "meta overlay" layer, so an overlay sits on top of buildings.</summary>
    private const float Altitude = 3.7f;

    // ------------------------------------------------------------------ sustained sound and music

    private static readonly List<Sustainer> Sustainers = new();

    /// <summary>Starts a sustained sound and keeps it alive until `sound_end` or the sound ends.</summary>
    public static object SoundSustain(JsonElement request)
    {
        string defName = JsonArgs.GetString(request, "sound");
        SoundDef def = DefDatabase<SoundDef>.GetNamedSilentFail(defName)
            ?? throw new BridgeException($"the sound def '{defName}' does not exist");

        int mapId = JsonArgs.GetInt(request, "mapId", -1);
        SoundInfo info;
        if (mapId >= 0)
        {
            Map map = Current.Game?.Maps?.FirstOrDefault(candidate => candidate.uniqueID == mapId)
                ?? throw new BridgeException($"map {mapId} does not exist");
            var cell = new IntVec3(JsonArgs.GetInt(request, "x", 0), 0, JsonArgs.GetInt(request, "z", 0));
            info = SoundInfo.InMap(new TargetInfo(cell, map), MaintenanceType.PerTick);
        }
        else
        {
            info = SoundInfo.OnCamera(MaintenanceType.PerTick);
        }

        var sustainer = new Sustainer(def, info);
        Sustainers.Add(sustainer);
        return sustainer;
    }

    /// <summary>Stops a sustained sound early.</summary>
    public static void SoundEnd(JsonElement request)
    {
        object target = RefContext.Current.Get(JsonArgs.GetInt(request, "ref", -1));
        if (target is not Sustainer sustainer)
        {
            throw new BridgeException("sound_end takes the handle sound_sustain returned");
        }

        sustainer.End();
        Sustainers.Remove(sustainer);
    }

    /// <summary>Keeps the sustained sounds going; the game expects this every frame or tick.</summary>
    public static void MaintainSounds()
    {
        for (int index = Sustainers.Count - 1; index >= 0; index--)
        {
            Sustainer sustainer = Sustainers[index];
            if (sustainer.Ended)
            {
                Sustainers.RemoveAt(index);
                continue;
            }

            sustainer.Maintain();
        }
    }

    /// <summary>Forces a music track, the way a mod's own soundtrack does.</summary>
    public static object MusicPlay(JsonElement request)
    {
        string name = JsonArgs.GetString(request, "song");
        SongDef song = DefDatabase<SongDef>.GetNamedSilentFail(name)
            ?? throw new BridgeException($"the song def '{name}' does not exist");
        MusicManagerPlay music = Find.MusicManagerPlay;
        if (music == null)
        {
            throw new BridgeException("there is no music manager (no game is running)");
        }

        music.ForcePlaySong(song, ignorePrefsVolume: false);
        return new Dictionary<string, object> { ["song"] = music.CurrentSong?.defName };
    }

    /// <summary>Shows one of the game's motes at a cell (motes are ThingDefs).</summary>
    public static object MoteSpawn(JsonElement request)
    {
        string name = JsonArgs.GetString(request, "mote");
        ThingDef def = DefDatabase<ThingDef>.GetNamedSilentFail(name)
            ?? throw new BridgeException($"the mote def '{name}' does not exist");
        int mapId = JsonArgs.GetInt(request, "mapId", -1);
        Map map = Current.Game?.Maps?.FirstOrDefault(candidate => candidate.uniqueID == mapId)
            ?? throw new BridgeException($"map {mapId} does not exist");
        var cell = new IntVec3(JsonArgs.GetInt(request, "x", 0), 0, JsonArgs.GetInt(request, "z", 0));
        float scale = (float)JsonArgs.GetDouble(request, "scale", 1.0);
        Mote mote = MoteMaker.MakeStaticMote(cell, map, def, scale);
        if (mote == null)
        {
            throw new BridgeException($"the game refused to spawn a mote of '{def.defName}'");
        }

        return mote;
    }
}
