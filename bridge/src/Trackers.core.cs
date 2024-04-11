using System;
using System.Collections.Generic;
using System.Linq;
using System.Text.Json;
using RimWorld;
using UnityEngine;
using Verse;
using Verse.AI;
namespace RustBridge;

/// <summary>
/// The `core` half of <see cref="Trackers"/>.
/// </summary>
internal static partial class Trackers
{

    /// <summary>
    /// Hands back a pawn's own tracker as a handle — `skills`, `needs`, `health`, `story`, `guest`, `ideo`,
    /// `relations`, `records`, `training` and the rest of its public members. Everything the tracker exposes
    /// then works through the generic `ref_get`/`ref_set`/`ref_call` route, so one accessor opens a hundred
    /// members instead of one op per member. A name the pawn does not have is refused with its type name.
    /// </summary>
    public static object Tracker(JsonElement request)
    {
        Pawn pawn = Resolve(request);
        string name = JsonArgs.GetString(request, "tracker");
        object tracker = PublicMember(pawn, name, out bool exists);
        if (!exists)
        {
            throw new BridgeException(
                $"'{pawn.LabelShort}' ({pawn.GetType().Name}) has no public member '{name}'");
        }

        if (tracker == null)
        {
            // The member exists but the game has given this pawn nothing there — a humanlike has no training
            // tracker, an animal has no ideoligion. Saying "no such member" would be a lie about the game.
            throw new BridgeException(
                $"'{pawn.LabelShort}' has no '{name}' right now (the member exists on {pawn.GetType().Name}, but the game has not given this pawn one)");
        }

        return tracker;
    }

    /// <summary>
    /// Hands back one of a map's own systems as a handle — `terrainGrid`, `fogGrid`, `weatherManager`,
    /// `areaManager`, `lordManager`, `gameConditionManager`, `mapPawns` and the rest of its public members —
    /// so the same generic route reaches the whole map surface.
    /// </summary>
    public static object MapSystem(JsonElement request)
    {
        int mapId = JsonArgs.GetInt(request, "mapId", -1);
        Map map = Find.Maps?.FirstOrDefault(candidate => candidate.uniqueID == mapId)
            ?? throw new BridgeException($"map {mapId} does not exist");
        string name = JsonArgs.GetString(request, "member");
        object system = PublicMember(map, name, out bool exists);
        if (!exists)
        {
            throw new BridgeException($"map {map.uniqueID} ({map.GetType().Name}) has no public member '{name}'");
        }

        if (system == null)
        {
            throw new BridgeException(
                $"map {map.uniqueID} has no '{name}' right now (the member exists on {map.GetType().Name}, but the game has not given this map one)");
        }

        return system;
    }

    /// <summary>
    /// Hands back one of the planet's own systems as a handle — `grid`, `worldObjects`, `features`,
    /// `worldPawns`, `pathing` and the rest of `Find.World`'s public members — so the generic route reaches
    /// the whole world surface the same way it reaches a map's.
    /// </summary>
    public static object WorldSystem(JsonElement request)
    {
        string name = JsonArgs.GetString(request, "member");
        object system = PublicMember(Find.World, name, out bool exists);
        if (!exists)
        {
            throw new BridgeException($"the world has no public member '{name}'");
        }

        if (system == null)
        {
            throw new BridgeException($"the world has no '{name}' right now (the member exists, but it is empty)");
        }

        return system;
    }

    /// <summary>
    /// Hands back one of the game's own singletons as a handle — `World`, `IdeoManager`, `FactionManager`,
    /// `Storyteller`, `ResearchManager`, `QuestManager` and the rest of `Find`'s public members.
    /// </summary>
    public static object GameSystem(JsonElement request)
    {
        string name = JsonArgs.GetString(request, "member");
        object system = PublicMember(typeof(Find), null, name, out bool exists);
        if (!exists)
        {
            throw new BridgeException($"the game has no public system '{name}'");
        }

        if (system == null)
        {
            throw new BridgeException($"the game has no '{name}' right now (the member exists, but it is empty)");
        }

        return system;
    }
    private static object PublicMember(object instance, string name, out bool exists)
    {
        return PublicMember(instance?.GetType(), instance, name, out exists);
    }

    /// <summary>
    /// A public field or property by name, from an instance or from a static type (when the instance is
    /// null). Both are searched, because the game mixes them freely. `exists` says whether the member is
    /// there at all, which is what makes "the game has no such thing" and "the game has this, but empty" two
    /// different answers instead of one wrong one.
    /// </summary>
    private static object PublicMember(Type type, object instance, string name, out bool exists)
    {
        const System.Reflection.BindingFlags flags =
            System.Reflection.BindingFlags.Public
            | System.Reflection.BindingFlags.Instance
            | System.Reflection.BindingFlags.Static;
        System.Reflection.FieldInfo field = type.GetField(name, flags);
        if (field != null)
        {
            exists = true;
            return field.GetValue(field.IsStatic ? null : instance);
        }

        System.Reflection.PropertyInfo property = type.GetProperty(name, flags);
        exists = property != null;
        return property?.GetValue(property.GetMethod?.IsStatic == true ? null : instance);
    }

    /// <summary>Reads one static member of a type without needing an instance.</summary>
    public static object TypeGet(JsonElement request)
    {
        string name = JsonArgs.GetString(request, "type");
        return GenTypes.GetTypeInAnyAssembly(name, null)
            ?? throw new BridgeException(
                $"the game has no type '{name}' (use the full name, for example 'RimWorld.MoteMaker'; cargo xtask types <text> lists them)");
    }

    /// <summary>`def.label` directly: `LabelCap` throws on the defs that have no plain label.</summary>
    private static string Label(Def def)
    {
        if (def == null)
        {
            return null;
        }

        return string.IsNullOrEmpty(def.label) ? def.defName : def.label;
    }

    /// <summary>Runs one tracker operation, reporting which operation and which pawn failed.</summary>
    private static object Run(string operation, Pawn pawn, Func<object> body)
    {
        try
        {
            return body();
        }
        catch (BridgeException)
        {
            throw;
        }
        catch (Exception error)
        {
            throw new BridgeException($"{operation} failed for '{pawn.LabelShort}': {error.Message}");
        }
    }
    private static Pawn Resolve(JsonElement request)
    {
        int handle = JsonArgs.GetInt(request, "ref", -1);
        object target = RefContext.Current.Get(handle);
        return target switch
        {
            null => throw new BridgeException($"the pawn handle {handle} is not valid"),
            Pawn pawn => pawn,
            _ => throw new BridgeException($"handle {handle} is a {target.GetType().Name}, not a pawn"),
        };
    }
}
