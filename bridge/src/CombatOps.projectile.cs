using System;
using System.Collections.Generic;
using System.Text.Json;
using RimWorld;
using Verse;
namespace RustBridge;

/// <summary>
/// The projectile half of <see cref="CombatOps"/>: a mod launches one of the game's own projectile defs,
/// and the game flies it, resolves the hit and applies the damage.
/// </summary>
internal static partial class CombatOps
{
    /// <summary>
    /// Spawns a projectile def at a cell and launches it through the game's own `Projectile.Launch`.
    /// A def whose thing class is not a projectile is refused by name, and so is a cell outside the map.
    /// </summary>
    public static object LaunchProjectile(JsonElement request)
    {
        Map map = MapForCombat(request);
        string defName = JsonArgs.GetString(request, "defName");
        ThingDef def = DefDatabase<ThingDef>.GetNamedSilentFail(defName)
            ?? throw new BridgeException($"unknown ThingDef '{defName}'");
        IntVec3 origin = new IntVec3(JsonArgs.GetInt(request, "x", 0), 0, JsonArgs.GetInt(request, "z", 0));
        if (!origin.InBounds(map))
        {
            throw new BridgeException($"cell {origin.x},{origin.z} is outside map {map.uniqueID}");
        }

        Thing made = ThingMaker.MakeThing(def);
        if (made is not Projectile projectile)
        {
            throw new BridgeException(
                $"'{defName}' is not a projectile (its thing class is {made.GetType().Name})");
        }

        LocalTargetInfo target = CoverTarget(request);
        Thing launcher = JsonArgs.GetInt(request, "launcherRef", -1) >= 0 ? ResolveThing(request, "launcherRef") : null;
        Thing equipment = JsonArgs.GetInt(request, "equipmentRef", -1) >= 0 ? ResolveThing(request, "equipmentRef") : null;
        ProjectileHitFlags flags = ProjectileHitFlags.All;
        string wanted = JsonArgs.GetStringOrNull(request, "hitFlags");
        if (!string.IsNullOrEmpty(wanted)
            && !Enum.TryParse(wanted, ignoreCase: true, out flags))
        {
            throw new BridgeException(
                $"unknown hit flags '{wanted}' (expected one of {string.Join(", ", Enum.GetNames<ProjectileHitFlags>())})");
        }

        GenSpawn.Spawn(projectile, origin, map, WipeMode.Vanish);
        projectile.Launch(launcher, target, target, flags, preventFriendlyFire: false, equipment);
        return new Dictionary<string, object>
        {
            ["handle"] = projectile,
            ["defName"] = def.defName,
            // The countdown and the destination are protected in this build; the game reports the flight
            // through the projectile's own ticks, and a mod reads it through the handle when it needs to.
            ["hitFlags"] = flags.ToString(),
            ["launcher"] = launcher?.LabelShort,
            ["x"] = origin.x,
            ["z"] = origin.z,
        };
    }
}
