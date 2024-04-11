using System;
using System.Collections.Generic;
using System.Globalization;
using System.Linq;
using System.Text.Json;
using UnityEngine;
using Verse;

namespace RustBridge;

/// <summary>
/// A window declared entirely from Rust: labels, buttons, checkboxes, sliders and text fields, all
/// reported back through the `ui_action` event. The bridge keeps the widget state (what the player
/// typed, where a slider sits) so window updates do not fight the user's input.
/// </summary>
internal sealed class RustWindow : Window
{
    private readonly List<Widget> widgets = new();
    private readonly string windowTitle;
    private readonly string closeHandler;

    public RustWindow(string modName, string title, float width, float height, string closeHandler, IEnumerable<Widget> declared)
    {
        this.closeHandler = closeHandler;
        this.windowTitle = string.IsNullOrEmpty(title) ? modName : title;
        widgets.AddRange(declared);

        doWindowBackground = true;
        closeOnClickedOutside = false;
        absorbInputAroundWindow = true;
        preventCameraMotion = true;
        draggable = true;
        resizeable = false;

        // The game's own title and close button: a window is then always closable, whatever its
        // widgets do, and the title follows the game's look. (Verified: `Verse.Window` has
        // `optionalTitle` and `doCloseX` as public fields.)
        optionalTitle = this.windowTitle;
        doCloseX = true;

        Size = new Vector2(width, height);
    }

    private Vector2 Size { get; }

    private bool fitted;

    public override Vector2 InitialSize => new Vector2(
        Size.x > 0f ? Size.x : 420f,
        Size.y > 0f ? Size.y : WidgetHeight());

    /// <summary>First guess for the height: the widgets, measured with the game's own line height.
    /// The real height is corrected on the first frame (see <see cref="DoWindowContents"/>).</summary>
    private float WidgetHeight()
    {
        float line = Text.LineHeight;
        float height = 34f; // the title area the game draws
        foreach (Widget widget in widgets)
        {
            height += widget.Kind switch
            {
                "label" => line + 6f,
                "separator" => 12f,
                "button" => line + 14f,
                "checkbox" => line + 8f,
                "slider" => line * 2f + 10f,
                "text" => line * 2f + 14f,
                _ => line + 6f,
            } + 4f; // the gap the renderer leaves after every widget
        }

        return Math.Max(140f, height + 20f);
    }

    public override void DoWindowContents(Rect inRect)
    {
        Rect content = inRect.ContractedBy(12f);

        var listing = new Listing_Standard();
        listing.Begin(content);
        foreach (Widget widget in widgets)
        {
            switch (widget.Kind)
            {
                case "label":
                    listing.Label(widget.Text);
                    break;

                case "separator":
                    listing.GapLine(8f);
                    break;

                case "button":
                    if (listing.ButtonText(widget.Text, null, 1f))
                    {
                        Report(widget, "clicked");
                        if (widget.CloseOnClick)
                        {
                            Close(true);
                        }
                    }

                    break;

                case "checkbox":
                    bool checkedNow = widget.Checked;
                    listing.CheckboxLabeled(widget.Text, ref checkedNow);
                    if (checkedNow != widget.Checked)
                    {
                        widget.Checked = checkedNow;
                        Report(widget, checkedNow);
                    }

                    break;

                case "slider":
                    float moved = listing.SliderLabeled(
                        $"{widget.Text}: {widget.Value.ToString("0.##", CultureInfo.InvariantCulture)}",
                        widget.Value,
                        widget.Min,
                        widget.Max,
                        0.6f,
                        null);
                    if (Math.Abs(moved - widget.Value) > 0.0001f)
                    {
                        widget.Value = moved;
                        Report(widget, (double)moved);
                    }

                    break;

                case "text":
                    string typed = listing.TextEntryLabeled(widget.Text, widget.ValueText, 1);
                    if (typed != widget.ValueText)
                    {
                        widget.ValueText = typed;
                        Report(widget, typed);
                    }

                    break;

                default:
                    listing.Label($"unknown widget '{widget.Kind}'");
                    break;
            }

            listing.Gap(4f);
        }

        listing.End();

        // A too-small window silently cuts the last widgets off — in game that hid a declared Close
        // button below the frame. The first frame measures how much room the widgets really need and
        // grows the window to it, so nothing a mod declares is ever out of reach.
        if (!fitted && listing.CurHeight > 0f)
        {
            fitted = true;
            float needed = listing.CurHeight + Margin * 2f + 34f; // title area plus margins
            if (needed > windowRect.height)
            {
                windowRect = new Rect(windowRect.x, windowRect.y, windowRect.width, needed);
            }
        }
    }

    public override void PostClose()
    {
        base.PostClose();
        if (!string.IsNullOrEmpty(closeHandler))
        {
            RustHost.Current?.DispatchVoid(
                "ui_action",
                new Dictionary<string, object> { ["handler"] = closeHandler, ["id"] = string.Empty, ["value"] = "closed" });
        }
    }

    private void Report(Widget widget, object value)
    {
        if (string.IsNullOrEmpty(widget.Handler))
        {
            return;
        }

        RustHost.Current?.DispatchVoid(
            "ui_action",
            new Dictionary<string, object>
            {
                ["handler"] = widget.Handler,
                ["id"] = widget.Id ?? string.Empty,
                ["value"] = value,
            });
    }

    /// <summary>One widget of a Rust-declared window.</summary>
    internal sealed class Widget
    {
        public Widget(string kind)
        {
            Kind = kind;
        }

        public string Kind { get; }

        public string Id { get; set; }

        public string Text { get; set; }

        public string Handler { get; set; }

        public bool CloseOnClick { get; set; }

        public bool Checked { get; set; }

        public float Value { get; set; }

        public float Min { get; set; }

        public float Max { get; set; } = 1f;

        public string ValueText { get; set; } = string.Empty;
    }

    /// <summary>Builds a window from the request of `ui_window_open`.</summary>
    public static RustWindow FromRequest(string modName, JsonElement request)
    {
        if (!request.TryGetProperty("widgets", out JsonElement widgets) || widgets.ValueKind != JsonValueKind.Array)
        {
            throw new BridgeException("ui_window_open needs a 'widgets' array");
        }

        var declared = new List<Widget>();
        foreach (JsonElement entry in widgets.EnumerateArray())
        {
            string kind = JsonArgs.GetStringOrNull(entry, "kind")
                ?? throw new BridgeException("every window widget needs a 'kind'");
            var widget = new Widget(kind)
            {
                Id = JsonArgs.GetStringOrNull(entry, "id"),
                Text = JsonArgs.GetStringOrNull(entry, "text") ?? string.Empty,
                Handler = JsonArgs.GetStringOrNull(entry, "handler"),
                CloseOnClick = JsonArgs.GetBool(entry, "closeOnClick"),
            };

            if (entry.TryGetProperty("checked", out JsonElement checkedValue) && checkedValue.ValueKind is JsonValueKind.True or JsonValueKind.False)
            {
                widget.Checked = checkedValue.GetBoolean();
            }

            if (entry.TryGetProperty("value", out JsonElement value))
            {
                switch (value.ValueKind)
                {
                    case JsonValueKind.Number:
                        widget.Value = value.GetSingle();
                        break;
                    case JsonValueKind.String:
                        widget.ValueText = value.GetString() ?? string.Empty;
                        break;
                }
            }

            if (entry.TryGetProperty("min", out JsonElement min) && min.ValueKind == JsonValueKind.Number)
            {
                widget.Min = min.GetSingle();
            }

            if (entry.TryGetProperty("max", out JsonElement max) && max.ValueKind == JsonValueKind.Number)
            {
                widget.Max = max.GetSingle();
            }

            declared.Add(widget);
        }

        float width = (float)JsonArgs.GetDouble(request, "width", 420);
        float height = (float)JsonArgs.GetDouble(request, "height", 0);
        return new RustWindow(
            modName,
            JsonArgs.GetStringOrNull(request, "title"),
            width,
            height,
            JsonArgs.GetStringOrNull(request, "closeHandler"),
            declared);
    }
}
