using System;
using System.Collections.Generic;
using System.Linq;
using RimWorld;
using Verse;

namespace RustBridge;

/// <summary>
/// The game scans behind the dashboards: quests, world objects, things, rooms and the player's
/// colonists, each filtered by a keyword and capped so one snapshot stays small.
/// </summary>
internal static partial class DlcOps
{
    private static List<object> QuestsMatching(params string[] words)
    {
        var rows = new List<object>();
        try
        {
            foreach (Quest quest in Find.QuestManager?.QuestsListForReading ?? new List<Quest>())
            {
                string name = Text(Member(quest, "name", "Name")) ?? "";
                if (!ContainsAny(name, words))
                {
                    continue;
                }

                rows.Add(new Dictionary<string, object>
                {
                    ["name"] = name,
                    ["id"] = quest.id,
                });
                if (rows.Count >= 12)
                {
                    break;
                }
            }
        }
        catch
        {
            // The quest manager is not available before a game is loaded.
        }

        return rows;
    }

    private static List<object> WorldObjectsMatching(params string[] words)
    {
        var rows = new List<object>();
        try
        {
            object world = StaticMember(typeof(Find), "World");
            object objects = Member(world, "WorldObjects", "AllWorldObjects");
            foreach (object item in Enumerate(objects))
            {
                string name = Text(Member(item, "Label", "label", "Name", "name")) ?? "";
                string defName = Text(Member(Member(item, "def"), "defName")) ?? "";
                if (!ContainsAny(name, words) && !ContainsAny(defName, words))
                {
                    continue;
                }

                rows.Add(new Dictionary<string, object>
                {
                    ["label"] = name.Length > 0 ? name : defName,
                    ["defName"] = defName,
                });
                if (rows.Count >= 12)
                {
                    break;
                }
            }
        }
        catch
        {
            // World objects are optional on the dashboard.
        }

        return rows;
    }

    private static List<Thing> ThingsMatching(params string[] words)
    {
        var rows = new List<Thing>();
        foreach (Thing thing in AllThings())
        {
            string defName = thing.def?.defName ?? "";
            string label = Label(thing.def);
            if (ContainsAny(defName, words) || ContainsAny(label, words))
            {
                rows.Add(thing);
                if (rows.Count >= 32)
                {
                    break;
                }
            }
        }

        return rows;
    }

    private static List<string> Labels(List<Thing> things, int limit)
    {
        return things.Take(limit).Select(thing => thing.LabelShort).ToList();
    }

    private static List<Thing> AllThings()
    {
        var rows = new List<Thing>();
        try
        {
            foreach (Map map in Find.Maps ?? new List<Map>())
            {
                foreach (Thing thing in map?.listerThings?.AllThings ?? new List<Thing>())
                {
                    rows.Add(thing);
                }
            }
        }
        catch
        {
            // A missing map list means there is nothing to scan.
        }

        return rows;
    }

    private static List<Room> AllRooms()
    {
        var rows = new List<Room>();
        try
        {
            foreach (Map map in Find.Maps ?? new List<Map>())
            {
                foreach (Room room in map?.regionGrid?.AllRooms ?? new List<Room>())
                {
                    rows.Add(room);
                }
            }
        }
        catch
        {
            // Room data is optional.
        }

        return rows;
    }

    private static List<Pawn> PlayerColonists()
    {
        var rows = new List<Pawn>();
        try
        {
            foreach (Map map in Find.Maps ?? new List<Map>())
            {
                foreach (Pawn pawn in map?.mapPawns?.FreeColonistsSpawned ?? new List<Pawn>())
                {
                    if (pawn != null && !rows.Contains(pawn))
                    {
                        rows.Add(pawn);
                    }
                }
            }
        }
        catch
        {
            // The dashboard is allowed to be empty in the main menu.
        }

        return rows;
    }
}
