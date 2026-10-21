using System;
using System.Collections.Generic;
using System.Globalization;
using UnityEngine;
using Verse;

namespace RustBridge;

/// <summary>One setting a Rust mod declares for the mod options window.</summary>
public sealed class SettingField
{
    public string Key;

    public string Label;

    /// <summary>One of <c>bool</c>, <c>int</c>, <c>float</c>, <c>string</c>, <c>token</c>.</summary>
    public string Kind;

    public string Default;

    public float Min;

    public float Max;
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
            if (settings.Get(field.Key) == null && field.Default != null)
            {
                settings.SetSilent(field.Key, field.Default);
            }
        }
    }

    public static void Draw(RustMod mod, Rect rect)
    {
        var listing = new Listing_Standard();
        listing.Begin(rect);
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

        listing.End();
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

    private static int ParseInt(string text, int fallback)
    {
        return int.TryParse(text, NumberStyles.Integer, CultureInfo.InvariantCulture, out int value) ? value : fallback;
    }

    private static float ParseFloat(string text, float fallback)
    {
        return float.TryParse(text, NumberStyles.Float, CultureInfo.InvariantCulture, out float value) ? value : fallback;
    }
}
