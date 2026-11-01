using System;
using System.Collections.Generic;
using RimWorld;
using Verse;

namespace RustBridge;

/// <summary>
/// The odyssey half of <see cref="DlcOps"/>: gravships, vacuum and oxygen rooms, orbit sites and the
/// other world features the expansion adds.
/// </summary>
internal static partial class DlcOps
{
    private static Dictionary<string, object> Odyssey()
    {
        List<Thing> grav = ThingsMatching("grav");
        List<Thing> engines = ThingsMatching("gravship engine", "gravengine");
        List<Thing> substructure = ThingsMatching("substructure", "hull");
        List<Thing> scanners = ThingsMatching("scanner", "orbital scanner");
        List<Thing> fishing = ThingsMatching("fishing", "fishing spot");
        List<Thing> landmarks = ThingsMatching("landmark");
        List<Thing> shuttles = ThingsMatching("shuttle");
        List<Thing> mechhives = ThingsMatching("mechhive", "mech hive");
        List<object> orbitSites = WorldObjectsMatching("orbit", "station", "gravship");

        var gravRows = new List<object>();
        foreach (Thing thing in grav)
        {
            var components = new List<object>();
            double fuel = 0;
            double range = 0;
            double launchQuality = 0;
            try
            {
                foreach (ThingComp comp in (thing as ThingWithComps)?.AllComps ?? new List<ThingComp>())
                {
                    string typeName = comp.GetType().Name;
                    if (typeName.IndexOf("Grav", StringComparison.OrdinalIgnoreCase) < 0
                        && typeName.IndexOf("Engine", StringComparison.OrdinalIgnoreCase) < 0
                        && typeName.IndexOf("Substructure", StringComparison.OrdinalIgnoreCase) < 0)
                    {
                        continue;
                    }

                    components.Add(typeName);
                    fuel = Math.Max(fuel, Number(Member(comp, "Fuel", "FuelLevel", "fuel")) ?? 0);
                    range = Math.Max(range, Number(Member(comp, "Range", "MaxRange", "range")) ?? 0);
                    launchQuality = Math.Max(
                        launchQuality,
                        Number(Member(comp, "LaunchQuality", "launchQuality", "Quality")) ?? 0);
                }
            }
            catch
            {
                // A gravship thing with no readable comp still gets a name.
            }

            gravRows.Add(new Dictionary<string, object>
            {
                ["label"] = thing.LabelShort,
                ["defName"] = thing.def?.defName,
                ["fuel"] = fuel,
                ["range"] = range,
                ["launchQuality"] = launchQuality,
                ["components"] = components,
            });
            if (gravRows.Count >= 12)
            {
                break;
            }
        }

        int vacuumRooms = 0;
        int oxygenRooms = 0;
        foreach (Room room in AllRooms())
        {
            if (Flag(Member(room, "Vacuum", "IsVacuum")))
            {
                vacuumRooms++;
            }

            object oxygen = Member(room, "Oxygen", "OxygenLevel", "OxygenPercent");
            if (oxygen != null && Number(oxygen) is double level && level > 0.01)
            {
                oxygenRooms++;
            }
        }

        return new Dictionary<string, object>
        {
            ["gravship"] = gravRows,
            ["gravshipCount"] = grav.Count,
            ["engines"] = engines.Count,
            ["engineLabels"] = Labels(engines, 8),
            ["substructure"] = substructure.Count,
            ["substructureLabels"] = Labels(substructure, 8),
            ["vacuumRooms"] = vacuumRooms,
            ["oxygenRooms"] = oxygenRooms,
            ["orbitSites"] = orbitSites,
            ["orbitSiteCount"] = orbitSites.Count,
            ["scanners"] = scanners.Count,
            ["scannerLabels"] = Labels(scanners, 8),
            ["fishing"] = fishing.Count,
            ["fishingLabels"] = Labels(fishing, 8),
            ["landmarks"] = landmarks.Count,
            ["landmarkLabels"] = Labels(landmarks, 8),
            ["shuttles"] = shuttles.Count,
            ["shuttleLabels"] = Labels(shuttles, 8),
            ["mechhives"] = mechhives.Count,
            ["mechhiveLabels"] = Labels(mechhives, 8),
        };
    }
}
