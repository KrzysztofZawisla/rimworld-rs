using System;
using System.Collections.Generic;
using System.Globalization;
using RimWorld;
using UnityEngine;
using Verse;

namespace RustBridge;

/// <summary>One entry of a <c>choice</c> setting: the stored value and the text the player sees.</summary>
public sealed class SettingOption
{
    public string Value { get; set; }

    public string Label { get; set; }
}

/// <summary>One setting a Rust mod declares for the mod options window.</summary>
public sealed class SettingField
{
    public string Key { get; set; }

    public string Label { get; set; }

    /// <summary>One of <c>bool</c>, <c>int</c>, <c>float</c>, <c>string</c>, <c>token</c>,
    /// <c>action</c>, <c>label</c>, <c>choice</c>.</summary>
    public string Kind { get; set; }

    public string Default { get; set; }

    public float Min { get; set; }

    public float Max { get; set; }

    /// <summary>Entries of a <c>choice</c> setting, in the order they appear in the dropdown.</summary>
    public List<SettingOption> Options { get; set; } = new List<SettingOption>();
}

/// <summary>
/// Draws the mod options window for settings declared from Rust with `settings_define` and
/// reports changes back as `settings_changed` events.
/// </summary>
internal static class SettingsWindow
{
    /// <summary>Stores the declared fields and applies their defaults for keys that are unset.</summary>
    public static void Define(RustMod mod, List<SettingField> fields)
    {
        mod.SetSettingFields(fields);
        RustModSettings settings = mod.Settings;
        if (settings == null)
        {
            return;
        }

        foreach (SettingField field in fields)
        {
            if (field.Kind == "action" || field.Kind == "label")
            {
                continue;
            }
            if (settings.Get(field.Key) == null && field.Default != null)
            {
                settings.SetSilent(field.Key, field.Default);
            }
        }
    }

    public static void Draw(RustMod mod, Rect rect)
    {
        string key = mod.Content?.PackageId ?? mod.Content?.Name ?? "Rust";
        if (!ScrollPositions.TryGetValue(key, out Vector2 scroll))
        {
            scroll = Vector2.zero;
        }
        if (!ContentHeights.TryGetValue(key, out float contentHeight))
        {
            contentHeight = rect.height;
        }

        Rect viewRect = new Rect(0f, 0f, Mathf.Max(0f, rect.width - 16f), contentHeight);
        Widgets.BeginScrollView(rect, ref scroll, viewRect, true);
        var listing = new Listing_Standard();
        listing.Begin(new Rect(0f, 0f, viewRect.width, 10000f));
        DrawBridgeOptions(listing);
        RustModSettings settings = mod.Settings;
        foreach (SettingField field in mod.SettingFields)
        {
            string current = settings?.Get(field.Key) ?? field.Default ?? string.Empty;
            switch (field.Kind)
            {
                case "bool":
                    {
                        bool value = current == "true";
                        bool before = value;
                        listing.CheckboxLabeled(field.Label, ref value, 0f);
                        if (value != before)
                        {
                            Changed(mod, field, value ? "true" : "false");
                        }

                        break;
                    }
                case "int":
                    {
                        int value = ParseInt(current, (int)field.Min);
                        // The number goes into the label: on this build the slider widget draws without its value,
                        // and a setting the player cannot read is a setting they cannot set.
                        float slider =
                            listing.SliderLabeled($"{field.Label}: {value}", value, field.Min, field.Max, 0.6f, null);
                        int rounded = Mathf.RoundToInt(slider);
                        if (rounded != value)
                        {
                            Changed(mod, field, rounded.ToString(CultureInfo.InvariantCulture));
                        }

                        break;
                    }
                case "float":
                    {
                        float value = ParseFloat(current, field.Min);
                        float slider = listing.SliderLabeled(
                            $"{field.Label}: {value:0.###}",
                            value,
                            field.Min,
                            field.Max,
                            0.6f,
                            null);
                        if (Math.Abs(slider - value) > 0.0001f)
                        {
                            Changed(mod, field, slider.ToString("0.###", CultureInfo.InvariantCulture));
                        }

                        break;
                    }
                case "action":
                    {
                        if (Widgets.ButtonText(listing.GetRect(30f), field.Label))
                        {
                            RustHost.Current?.DispatchVoid(
                                "ui_action",
                                new Dictionary<string, object>
                                {
                                    ["handler"] = "settings_action",
                                    ["id"] = field.Key,
                                    ["value"] = "clicked",
                                });
                        }

                        listing.Gap(4f);
                        break;
                    }
                case "label":
                    {
                        listing.Label($"{field.Label}: {LabelStatus(mod, field)}");
                        listing.Gap(4f);
                        break;
                    }
                case "choice":
                    {
                        string selected = current;
                        foreach (SettingOption option in field.Options)
                        {
                            if (option.Value == current)
                            {
                                selected = option.Label;
                                break;
                            }
                        }

                        Rect row = listing.GetRect(30f);
                        float labelWidth = Mathf.Min(160f, Mathf.Round(row.width * 0.45f));
                        Widgets.Label(new Rect(row.x, row.y, labelWidth, row.height), field.Label);
                        Rect buttonRect = new Rect(
                            row.x + labelWidth,
                            row.y,
                            row.width - labelWidth,
                            row.height);
                        if (field.Options.Count > 0 && Widgets.ButtonText(buttonRect, selected))
                        {
                            var menu = new List<FloatMenuOption>();
                            foreach (SettingOption option in field.Options)
                            {
                                SettingOption picked = option;
                                menu.Add(new FloatMenuOption(picked.Label, () => Changed(mod, field, picked.Value)));
                            }

                            Find.WindowStack.Add(new FloatMenu(menu));
                        }

                        listing.Gap(4f);
                        break;
                    }
                case "token":
                    {
                        listing.Label(field.Label);
                        Rect row = listing.GetRect(Text.LineHeight + 6f);
                        Rect valueRect = new Rect(
                            row.x,
                            row.y,
                            Mathf.Max(80f, row.width - 190f),
                            row.height);
                        Widgets.DrawMenuSection(valueRect);
                        Widgets.Label(valueRect.ContractedBy(4f), current);

                        Rect copyRect = new Rect(valueRect.xMax + 4f, row.y, 80f, row.height);
                        if (Widgets.ButtonText(copyRect, "Copy"))
                        {
                            GUIUtility.systemCopyBuffer = current;
                        }

                        Rect rotateRect = new Rect(copyRect.xMax + 4f, row.y, 90f, row.height);
                        if (Widgets.ButtonText(rotateRect, "Rotate"))
                        {
                            RustHost.Current?.DispatchVoid(
                                "ui_action",
                                new Dictionary<string, object>
                                {
                                    ["handler"] = "settings_action",
                                    ["id"] = field.Key,
                                    ["value"] = "rotate",
                                });
                        }

                        listing.Gap(6f);
                        break;
                    }
                default:
                    {
                        string text = listing.TextEntryLabeled(field.Label, current, 1);
                        if (text != current)
                        {
                            Changed(mod, field, text);
                        }

                        break;
                    }
            }
        }

        float measuredHeight = listing.CurHeight;
        listing.End();
        Widgets.EndScrollView();
        if (Mathf.Abs(measuredHeight - contentHeight) > 1f)
        {
            Log.Message($"[Rust] settings '{key}': content {measuredHeight:0} px, view {rect.height:0} px");
            ContentHeights[key] = Mathf.Max(rect.height, measuredHeight);
        }
        ScrollPositions[key] = scroll;
    }

    private static readonly Dictionary<string, Vector2> ScrollPositions = new();
    private static readonly Dictionary<string, float> ContentHeights = new();

    /// <summary>
    /// The bridge's own options, above the ones Rust declared. They write silently: Rust never declared
    /// these keys, so a `settings_changed` event about one would be a lie about the mod's own settings.
    /// </summary>
    private static void DrawBridgeOptions(Listing_Standard listing)
    {
        bool verbose = BridgeOptions.VerboseErrors;
        bool before = verbose;
        listing.CheckboxLabeled(
            "Log a stack trace when a bridge call fails",
            ref verbose,
            "Writes the full exception into Player.log when a call from Rust into the game fails.");
        if (verbose != before)
        {
            BridgeOptions.SetVerboseErrors(verbose);
        }

        listing.Gap(6f);
    }

    /// <summary>
    /// Stores the value without rewriting the config file: RimWorld writes mod settings when the
    /// options window closes, so a dragged slider does not hit the disk on every frame.
    /// </summary>
    private static void Changed(RustMod mod, SettingField field, string value)
    {
        mod.Settings?.SetSilent(field.Key, value);
        RustHost.Current?.DispatchVoid(
            "settings_changed",
            new Dictionary<string, object> { ["key"] = field.Key, ["value"] = value });
    }

    private sealed class LabelCacheEntry
    {
        public string Value { get; set; }
        public float At { get; set; }
    }

    private static readonly Dictionary<string, LabelCacheEntry> LabelCache = new();

    /// <summary>
    /// Labels are read live from Rust, throttled so a 60 FPS settings window does not flood the
    /// bridge. The mod answers the `settings_label` companion event with a short status string.
    /// </summary>
    private static string LabelStatus(RustMod mod, SettingField field)
    {
        string source = mod.Content?.Name ?? "Rust";
        string key = source + ":" + field.Key;
        float now = Time.realtimeSinceStartup;
        if (LabelCache.TryGetValue(key, out LabelCacheEntry cached) && now - cached.At < 0.25f)
        {
            return cached.Value;
        }

        string value = Companion.Request(
            source,
            "settings_label",
            new Dictionary<string, object> { ["id"] = field.Key });
        LabelCache[key] = new LabelCacheEntry { Value = value, At = now };
        return value;
    }

    private static int ParseInt(string text, int fallback)
    {
        return int.TryParse(text, NumberStyles.Integer, CultureInfo.InvariantCulture, out int value) ? value : fallback;
    }

    private static float ParseFloat(string text, float fallback)
    {
        return float.TryParse(text, NumberStyles.Float, CultureInfo.InvariantCulture, out float value) ? value : fallback;
    }
}
