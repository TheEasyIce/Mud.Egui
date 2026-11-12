//! A themed slider control.
//!
//! Port of `MudSlider<T>.cs`. Drawn with a custom painter (rather than by
//! wrapping `egui::Slider`) so it can match MudBlazor's look, which
//! `egui::Slider` cannot produce:
//!
//! * a thin rail whose thickness, and a circular thumb whose diameter,
//!   follow [`Size`];
//! * a translucent rail, filled up to the thumb only for `Variant::Filled`;
//! * a hover / press halo around the thumb;
//! * tick marks that sit *on* the rail, with labels centred under them;
//! * a value bubble floating above the thumb, shown only while the slider is
//!   hovered, pressed, or keyboard-focused (it never affects layout).
//!
//! # MudBlazor parity
//!
//! The public parameter surface mirrors `MudSlider<T>` (MudBlazor 8.13.0):
//!
//! | MudBlazor parameter | Rust field / method |
//! |---------------------|---------------------|
//! | `Value` | the bound `&mut f32` passed to [`MudSlider::new`] |
//! | `ValueChanged` | [`Response::changed`] on the returned `Response` |
//! | `Min` | [`MudSlider::new`] `min` argument |
//! | `Max` | [`MudSlider::new`] `max` argument |
//! | `Step` | [`MudSlider::step`] |
//! | `Vertical` | [`MudSlider::vertical`] |
//! | `Disabled` | [`MudSlider::disabled`] |
//! | `Color` | [`MudSlider::color`] |
//! | `Size` | [`MudSlider::size`] |
//! | `Variant` | [`MudSlider::variant`] |
//! | `TickMarks` | [`MudSlider::ticks`] |
//! | `MarkerLabels` | [`MudSlider::tick_labels`] |
//! | `ValueLabel` | [`MudSlider::show_value`] |
//! | `ValueLabelFormat` | [`MudSlider::value_format`] (template string, not a `Func`) |
//! | `ChildContent` (label) | [`MudSlider::label`] (string form) |
//!
//! ## Deliberately not ported
//!
//! * `Range` / `RangeValue` / `RangeValueChanged` — two-thumb range mode is a
//!   separate widget shape and would double the surface; not implemented.
//! * `Immediate` — MudBlazor defaults to firing `ValueChanged` on release;
//!   in an immediate-mode UI every frame is a “release”, so `response.changed()`
//!   fires every time the value moves. This port is effectively
//!   `Immediate = true` always.
//! * `Culture` — Rust’s formatter is not culture-aware; use a
//!   `value_format(..)` template instead.
//! * `LabelAttributes` / `InputAttributes` / `Class` / `Style` /
//!   `Tag` / `UserAttributes` — no CSS strings, no HTML tag in egui.
//!
//! ## Extension
//!
//! * [`MudSlider::length`] — an explicit length along the slider axis, in
//!   points. MudBlazor relies on CSS width for this; `length` is the typed
//!   equivalent and is the only reason a caller would reach for `Style` on
//!   this component.
//!
//! # Rule 3 compliance
//!
//! Every visible string — the label, tick labels, and value-bubble text —
//! renders through [`MudText`]. The rail, fill, ticks, halo, thumb, and
//! bubble *background* are painted directly; no user-facing text is drawn with
//! `painter.text` or `painter.galley`.
//!
//! # Deviation from the pre-refactor bubble
//!
//! The value bubble lives on the slider's own layer (via `ui.painter()` and a
//! child `Ui` for its text) rather than the tooltip layer, because `MudText`
//! needs a `Ui` and egui offers no way to construct one on an arbitrary layer.
//! Bubble z-order is therefore determined by paint order within the slider's
//! `Ui` — the bubble is drawn last, so it is on top of the slider — rather
//! than by the tooltip layer's guaranteed z-priority. For sliders inside a
//! tight clip (e.g. a small `ScrollArea`), the bubble may be clipped where the
//! previous implementation would have escaped. Documented here rather than
//! silently dropped.

use egui::{
    pos2, vec2, Align, Color32, FontId, Id, Key, Layout, Pos2, Rect, Response, Sense, Stroke, Ui,
    UiBuilder, Vec2, Widget,
};

use crate::MudText;
use mud_egui_core::enums::{Color, Size, Typo, Variant};
use mud_egui_theming::theme::{to_egui_family, MudTheme};

/// Minimum number of tick marks [`tick_count`] will accept. Below this
/// there's nothing meaningful to show (zero or one point along the track).
const MIN_TICKS: usize = 2;

/// Maximum number of tick marks [`tick_count`] will accept. Above this the
/// marks would be too dense to read, so a pathological `(min, max, step)`
/// combination is silently skipped rather than flooding the UI with
/// illegible ticks.
const MAX_TICKS: usize = 30;

/// Thickness, in points, of the slider's interactive strip (its height when
/// horizontal, its width when vertical). Must be >= the largest thumb.
const HIT_THICKNESS: f32 = 28.0;

/// Extra height reserved under the strip when tick labels are shown.
const LABEL_ROW_EXTRA: f32 = 12.0;

/// Distance from the rail's centre line to the centre of a tick label.
const TICK_LABEL_OFFSET: f32 = 18.0;

/// How far a tick mark pokes out past each edge of the rail.
const TICK_OVERHANG: f32 = 2.0;

/// Default length of a vertical slider when none is given via
/// [`MudSlider::length`]. (A vertical slider has no "available" length to
/// stretch to, so without a default it collapses to a blob.)
const DEFAULT_VERTICAL_LENGTH: f32 = 160.0;

/// Length used for a horizontal slider when the layout reports no usable
/// available width (e.g. inside an unbounded horizontal scroll area).
const FALLBACK_LENGTH: f32 = 200.0;

/// Opacity of the un-filled rail relative to the slider color.
const TRACK_ALPHA: f32 = 0.38;

/// Opacity multiplier applied to the color of a disabled slider.
const DISABLED_FACTOR: f32 = 0.5;

/// How far the hover halo extends past the thumb's edge.
const HALO_HOVER_EXTRA: f32 = 8.0;

/// Additional halo growth while the thumb is pressed.
const HALO_PRESS_EXTRA: f32 = 6.0;

/// Opacity of the halo at full strength.
const HALO_ALPHA: f32 = 0.16;

/// Seconds for the halo / bubble to fade in and out.
const ANIM_TIME: f32 = 0.12;

/// Font size, in points, for tick labels and the value bubble.
const LABEL_FONT_SIZE: f32 = 12.0;

/// Padding inside the value bubble, around its text.
const BUBBLE_PAD: Vec2 = Vec2::new(7.0, 3.0);

/// Gap between the halo and the value bubble.
const BUBBLE_GAP: f32 = 2.0;

/// Corner radius of the value bubble.
const BUBBLE_RADIUS: f32 = 4.0;

/// Width of the base of the pointer on the value bubble.
const BUBBLE_ARROW_WIDTH: f32 = 10.0;

/// How far the pointer sticks out from the value bubble toward the thumb.
const BUBBLE_ARROW_HEIGHT: f32 = 5.0;

/// Fills lighter than this relative luminance get dark bubble text; darker
/// fills get white. Set above the WCAG crossover (~0.18) on purpose so
/// mid-tone brand colors (e.g. orange) keep MudBlazor's white text.
const CONTRAST_LUMINANCE_THRESHOLD: f32 = 0.45;

/// A single-value slider.
///
/// Mirrors `MudSlider<T>` (MudBlazor 8.13.0): MudBlazor theming (color,
/// size), an optional label, horizontal or vertical orientation, an optional
/// value bubble, and optional tick marks with labels.
#[must_use = "MudSlider does nothing until you call `.show(ui)` or `ui.add(..)`"]
pub struct MudSlider<'a> {
    /// The bound value, mutated in place as the user drags the slider.
    pub value: &'a mut f32,
    /// Inclusive lower bound of the slider's range.
    pub min: f32,
    /// Inclusive upper bound of the slider's range.
    pub max: f32,
    /// Increment the value snaps to while dragging. `<= 0` means continuous.
    pub step: f32,
    /// MudBlazor display variant. `Variant::Filled` fills the rail up to the
    /// thumb; every other variant draws a uniform, translucent rail (this
    /// matches MudBlazor's default `Text` look).
    pub variant: Variant,
    /// Color applied to the rail, thumb, ticks and value bubble.
    pub color: Color,
    /// Size preset, controlling rail thickness and thumb diameter; see
    /// [`size_metrics`].
    pub size: Size,
    /// Optional text label shown above (horizontal) or beside (vertical)
    /// the slider.
    pub label: Option<&'a str>,
    /// Disables interaction and dims the slider.
    pub disabled: bool,
    /// Renders the slider along the vertical axis instead of horizontal.
    pub vertical: bool,
    /// Shows the current value in a bubble above the thumb (to the right of
    /// it when vertical) while the slider is hovered, pressed, or
    /// keyboard-focused. The bubble is painted over other content and does
    /// not take up layout space.
    pub show_value: bool,
    /// Optional template controlling how the shown value is formatted; see
    /// [`format_with_template`]. Only takes effect when `show_value` is
    /// `true`.
    pub value_format: Option<&'a str>,
    /// Draws evenly-spaced tick marks on the rail.
    pub ticks: bool,
    /// Optional label drawn under each tick mark, left to right, centred on
    /// the tick. If shorter than the tick count, only the first
    /// `labels.len()` ticks get a label. Horizontal sliders only.
    pub tick_labels: Option<&'a [&'a str]>,
    /// Length of the slider along its axis, in points. Defaults to the
    /// available width when horizontal and [`DEFAULT_VERTICAL_LENGTH`] when
    /// vertical.
    pub length: Option<f32>,
}

impl<'a> MudSlider<'a> {
    /// Creates a slider over `[min, max]` bound to `value`, with
    /// MudBlazor's defaults: `Text` variant, `Color::Primary`, `Small`
    /// size, step `1.0`, no label, value bubble hidden, no ticks.
    pub fn new(value: &'a mut f32, min: f32, max: f32) -> Self {
        Self {
            value,
            min,
            max,
            step: 1.0,
            variant: Variant::Text,
            color: Color::Primary,
            size: Size::Small,
            label: None,
            disabled: false,
            vertical: false,
            show_value: false,
            value_format: None,
            ticks: false,
            tick_labels: None,
            length: None,
        }
    }

    /// Sets the drag increment.
    #[inline] pub fn step(mut self, s: f32) -> Self { self.step = s; self }
    /// Sets the display variant.
    #[inline] pub fn variant(mut self, v: Variant) -> Self { self.variant = v; self }
    /// Sets the track/thumb color.
    #[inline] pub fn color(mut self, c: Color) -> Self { self.color = c; self }
    /// Sets the size preset.
    #[inline] pub fn size(mut self, s: Size) -> Self { self.size = s; self }
    /// Sets the label text.
    #[inline] pub fn label(mut self, l: impl Into<&'a str>) -> Self { self.label = Some(l.into()); self }
    /// Disables interaction.
    #[inline] pub fn disabled(mut self, b: bool) -> Self { self.disabled = b; self }
    /// Switches to a vertical layout.
    #[inline] pub fn vertical(mut self, b: bool) -> Self { self.vertical = b; self }
    /// Shows the value bubble on hover / press / keyboard focus.
    #[inline] pub fn show_value(mut self, b: bool) -> Self { self.show_value = b; self }
    /// Sets the value-display template; see [`format_with_template`].
    #[inline] pub fn value_format(mut self, fmt: impl Into<&'a str>) -> Self { self.value_format = Some(fmt.into()); self }
    /// Enables tick marks.
    #[inline] pub fn ticks(mut self, b: bool) -> Self { self.ticks = b; self }
    /// Sets per-tick labels (horizontal sliders only).
    #[inline] pub fn tick_labels(mut self, labels: &'a [&'a str]) -> Self { self.tick_labels = Some(labels); self }
    /// Sets the slider's length along its axis, in points.
    #[inline] pub fn length(mut self, l: f32) -> Self { self.length = Some(l); self }

    /// Lays out and draws the slider (plus its optional label), and returns
    /// the slider control's own `Response`. `response.changed()` is `true`
    /// on frames where the bound value changed.
    ///
    /// `self` is destructured up front so `value` (the bound `&mut f32`)
    /// can be passed explicitly into [`render_slider`] instead of
    /// requiring a rendering method to take `&mut self` just to reborrow
    /// one field -- the same shape used by `MudSwitch::show`.
    pub fn show(self, ui: &mut Ui) -> Response {
        let MudSlider {
            value,
            min,
            max,
            step,
            variant,
            color,
            size,
            label,
            disabled,
            vertical,
            show_value,
            value_format,
            ticks,
            tick_labels,
            length,
        } = self;

        let opts = RenderOpts {
            min,
            max,
            step,
            vertical,
            enabled: !disabled && ui.is_enabled(),
            filled: matches!(variant, Variant::Filled),
            color: resolve_color(ui, color),
            metrics: size_metrics(size),
            show_value,
            value_format,
            ticks,
            tick_labels,
            length,
            label,
        };

        if vertical {
            // A vertical slider needs a *horizontal* container so the
            // label can sit beside the vertical track instead of above it.
            ui.horizontal(|ui| {
                if let Some(lbl) = label {
                    MudText::body1(lbl).ui(ui);
                }
                render_slider(ui, value, &opts)
            }).inner
        } else {
            // A horizontal slider needs a *vertical* container so the
            // label can stack above the track.
            ui.vertical(|ui| {
                if let Some(lbl) = label {
                    MudText::body1(lbl).ui(ui);
                }
                render_slider(ui, value, &opts)
            }).inner
        }
    }
}

impl<'a> Widget for MudSlider<'a> {
    /// Delegates to [`MudSlider::show`]; the canonical way to add a
    /// `MudSlider` via `ui.add(..)`.
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui)
    }
}

/// Everything [`render_slider`] needs besides the bound value, resolved from
/// the builder up front.
struct RenderOpts<'a> {
    min: f32,
    max: f32,
    step: f32,
    vertical: bool,
    enabled: bool,
    filled: bool,
    color: Color32,
    metrics: SizeMetrics,
    show_value: bool,
    value_format: Option<&'a str>,
    ticks: bool,
    tick_labels: Option<&'a [&'a str]>,
    length: Option<f32>,
    label: Option<&'a str>,
}

/// Rail thickness and thumb diameter for a [`Size`], in points.
#[derive(Clone, Copy, Debug, PartialEq)]
struct SizeMetrics {
    rail: f32,
    thumb: f32,
}

/// Maps a [`Size`] preset to MudBlazor's rail thickness and thumb diameter.
/// MudBlazor only has three slider sizes, so the extremes fold inwards.
#[inline]
fn size_metrics(size: Size) -> SizeMetrics {
    match size {
        Size::ExtraSmall | Size::Small => SizeMetrics { rail: 2.0, thumb: 16.0 },
        Size::Medium => SizeMetrics { rail: 4.0, thumb: 20.0 },
        Size::Large | Size::ExtraLarge => SizeMetrics { rail: 6.0, thumb: 24.0 },
    }
}

/// The line the thumb travels along, from the `min` end (`start`) to the
/// `max` end (`end`). Works for both orientations: horizontal runs left to
/// right, vertical runs bottom to top.
#[derive(Clone, Copy)]
struct Axis {
    start: Pos2,
    end: Pos2,
}

impl Axis {
    /// The point `t` (0..=1) of the way from `start` to `end`.
    #[inline]
    fn point_at(self, t: f32) -> Pos2 {
        self.start + (self.end - self.start) * t
    }

    /// Projects `p` onto the axis, returning how far along it is (clamped to
    /// 0..=1).
    #[inline]
    fn fraction_of(self, p: Pos2) -> f32 {
        let d = self.end - self.start;
        let len_sq = d.length_sq();
        if len_sq <= f32::EPSILON {
            0.0
        } else {
            ((p - self.start).dot(d) / len_sq).clamp(0.0, 1.0)
        }
    }
}

/// Allocates, interacts with, and paints the slider, returning its
/// `Response`.
///
/// A free function (rather than a `MudSlider` method) taking `value: &mut
/// f32` explicitly, so nothing needs `&mut self` just to reborrow one field.
fn render_slider(ui: &mut Ui, value: &mut f32, o: &RenderOpts<'_>) -> Response {
    let SizeMetrics { rail, thumb } = o.metrics;
    let thumb_r = thumb / 2.0;
    let valid_range = o.max > o.min;

    let ticks_n = if o.ticks { tick_count(o.min, o.max, o.step) } else { None };
    let labels = if o.vertical { None } else { o.tick_labels.filter(|l| !l.is_empty()) };
    let has_labels = ticks_n.is_some() && labels.is_some();

    // --- Allocation ------------------------------------------------------
    // Allocate exactly the slider's length (never more), so the value bubble
    // and thumb can't push the parent container wider than its available
    // space.
    let main = if o.vertical {
        o.length.unwrap_or(DEFAULT_VERTICAL_LENGTH)
    } else {
        o.length.unwrap_or_else(|| usable_or(ui.available_width(), FALLBACK_LENGTH))
    }
        .max(thumb * 2.0);

    let size = if o.vertical {
        vec2(HIT_THICKNESS, main)
    } else {
        vec2(main, HIT_THICKNESS + if has_labels { LABEL_ROW_EXTRA } else { 0.0 })
    };
    let sense = if o.enabled { Sense::click_and_drag() } else { Sense::hover() };
    let (rect, mut response) = ui.allocate_exact_size(size, sense);

    let axis = if o.vertical {
        let cx = rect.center().x;
        Axis {
            start: pos2(cx, rect.bottom() - thumb_r),
            end: pos2(cx, rect.top() + thumb_r),
        }
    } else {
        let cy = rect.top() + HIT_THICKNESS / 2.0;
        Axis {
            start: pos2(rect.left() + thumb_r, cy),
            end: pos2(rect.right() - thumb_r, cy),
        }
    };

    // --- Interaction -----------------------------------------------------
    let pressed = o.enabled && (response.is_pointer_button_down_on() || response.dragged());
    let kb_id = response.id.with("kb_focus");
    let mut changed = false;

    if o.enabled && valid_range {
        if pressed {
            if let Some(pos) = response.interact_pointer_pos() {
                let new = fraction_to_value(axis.fraction_of(pos), o.min, o.max, o.step);
                if new != *value {
                    *value = new;
                    changed = true;
                }
            }
        }
        if response.has_focus() {
            if let Some(new) = keyboard_adjust(ui, *value, o.min, o.max, o.step) {
                ui.data_mut(|d| d.insert_temp(kb_id, true));
                if new != *value {
                    *value = new;
                    changed = true;
                }
            }
        }
    }
    if o.enabled {
        if response.drag_started() || response.clicked() {
            response.request_focus();
        }
        // Remember whether focus came from the keyboard, so the halo/bubble
        // don't stay stuck on after a mouse click.
        if pressed {
            ui.data_mut(|d| d.insert_temp(kb_id, false));
        } else if response.gained_focus() {
            ui.data_mut(|d| d.insert_temp(kb_id, true));
        }
    }
    if changed {
        response.mark_changed();
    }

    // Clamp for display only; the caller's value is left alone unless the
    // user actually moves the slider.
    let shown = if valid_range { value.clamp(o.min, o.max) } else { o.min };
    // `WidgetInfo::slider` takes three arguments:
    // `(enabled: bool, value: f64, label: impl ToString)`.
    response.widget_info(|| {
        egui::WidgetInfo::slider(o.enabled, f64::from(shown), o.label.unwrap_or(""))
    });

    let kb_focus = response.has_focus() && ui.data(|d| d.get_temp::<bool>(kb_id)).unwrap_or(false);
    let active = o.enabled && (response.hovered() || pressed || kb_focus);
    let active_t = ui.ctx().animate_bool_with_time(response.id.with("active"), active, ANIM_TIME);
    let press_t = ui.ctx().animate_bool_with_time(response.id.with("press"), pressed, ANIM_TIME);

    let response = if o.enabled {
        response.on_hover_cursor(egui::CursorIcon::PointingHand)
    } else {
        response
    };

    // --- Painting --------------------------------------------------------
    if !ui.is_rect_visible(rect) {
        return response;
    }

    // `Painter` is cheap to clone (it holds an Arc to the context) and doing
    // so releases the immutable borrow of `ui`, so the label/tick/bubble text
    // can open child `Ui`s further down. Drawing to the clone still targets
    // the same layer, in the same submission order.
    let painter = ui.painter().clone();

    let base = if o.enabled { o.color } else { o.color.gamma_multiply(DISABLED_FACTOR) };
    let frac = if valid_range { (shown - o.min) / (o.max - o.min) } else { 0.0 };
    let thumb_pos = axis.point_at(frac);
    let half = rail / 2.0;

    // Rail: translucent across its full length, plus a solid fill up to the
    // thumb for `Variant::Filled`.
    let (rail_rect, fill_rect) = if o.vertical {
        let cx = axis.start.x;
        (
            Rect::from_min_max(pos2(cx - half, rect.top()), pos2(cx + half, rect.bottom())),
            Rect::from_min_max(pos2(cx - half, thumb_pos.y), pos2(cx + half, rect.bottom())),
        )
    } else {
        let cy = axis.start.y;
        (
            Rect::from_min_max(pos2(rect.left(), cy - half), pos2(rect.right(), cy + half)),
            Rect::from_min_max(pos2(rect.left(), cy - half), pos2(thumb_pos.x, cy + half)),
        )
    };
    painter.rect_filled(rail_rect, half, base.gamma_multiply(TRACK_ALPHA));
    if o.filled {
        painter.rect_filled(fill_rect, half, base);
    }

    // The MudTheme is fetched once, shared by every `MudText` below and by
    // the measurement that positions tick labels.
    let theme = MudTheme::fetch(ui.ctx()).unwrap_or_default();
    let font = label_font(&theme);

    // Tick marks sit on the rail, poking out a couple of points either side.
    if let Some(count) = ticks_n {
        let tick_half = half + TICK_OVERHANG;
        let perp = if o.vertical { vec2(1.0, 0.0) } else { vec2(0.0, 1.0) };
        for i in 0..count {
            let Some(t) = tick_fraction(i, o.step, o.min, o.max) else { break };
            let p = axis.point_at(t);
            painter.line_segment(
                [p - perp * tick_half, p + perp * tick_half],
                Stroke::new(1.0, base),
            );
        }

        // Labels are centred *on* their tick, a fixed distance below the
        // rail. Each label is a `MudText` inside a child `Ui` sized to the
        // text, so Rule 3 is satisfied and the theme's typography applies.
        if let Some(labels) = labels {
            let text_color = if o.enabled {
                ui.visuals().text_color()
            } else {
                ui.visuals().weak_text_color()
            };
            for (i, text) in labels.iter().enumerate().take(count) {
                let Some(t) = tick_fraction(i, o.step, o.min, o.max) else { break };
                let p = axis.point_at(t);
                let anchor = pos2(p.x, axis.start.y + TICK_LABEL_OFFSET);

                // Measure with the same FontId the `MudText` will use, so the
                // child `Ui`'s rect is exactly the text's size and the label
                // reads as centred on `anchor`.
                let size = painter
                    .layout_no_wrap((*text).to_string(), font.clone(), text_color)
                    .size();
                let label_rect = Rect::from_center_size(anchor, size);

                let mut child = ui.new_child(
                    UiBuilder::new()
                        .max_rect(label_rect)
                        .layout(Layout::left_to_right(Align::Center)),
                );
                child.add(
                    MudText::new(*text)
                        .typo(Typo::Body2)
                        .font_size(LABEL_FONT_SIZE)
                        .custom_color(text_color)
                        .inline(true),
                );
            }
        }
    }

    // Halo (hover / press), then the thumb on top of it.
    let halo_extra = HALO_HOVER_EXTRA * active_t + HALO_PRESS_EXTRA * press_t;
    if active_t > 0.0 {
        painter.circle_filled(
            thumb_pos,
            thumb_r + halo_extra,
            base.gamma_multiply(HALO_ALPHA * active_t),
        );
    }
    painter.circle_filled(thumb_pos, thumb_r, base);

    // Value bubble: only while active, drawn last so it reads as "on top" of
    // the slider's own shapes. Its text goes through `MudText` (Rule 3); its
    // background and pointer are painted directly. See the module docs for
    // why this is not on the tooltip layer.
    if o.show_value && active_t > 0.0 {
        draw_value_bubble(
            ui,
            &format_value(shown, o.step, o.value_format),
            &theme,
            &font,
            &BubbleSpec {
                anchor: thumb_pos,
                clearance: thumb_r + halo_extra,
                vertical: o.vertical,
                fill: o.color,
                opacity: active_t,
            },
        );
    }

    response
}

/// Where and how to draw the value bubble.
struct BubbleSpec {
    /// Centre of the thumb.
    anchor: Pos2,
    /// Distance from `anchor` to the thumb/halo edge the bubble must clear.
    clearance: f32,
    /// Vertical sliders put the bubble to the right of the thumb; horizontal
    /// sliders put it above.
    vertical: bool,
    fill: Color32,
    /// 0..=1 fade-in factor.
    opacity: f32,
}

/// Paints the value bubble and renders its text through [`MudText`].
///
/// The bubble background and pointer are painted with `ui.painter()`; the
/// text is a `MudText` inside a child `Ui` positioned at the text rect. See
/// the module docs for why this is on the slider's own layer rather than the
/// tooltip layer.
fn draw_value_bubble(
    ui: &mut Ui,
    text: &str,
    theme: &MudTheme,
    font: &FontId,
    spec: &BubbleSpec,
) {
    let _ = theme;
    let text_color = contrast_text(spec.fill).gamma_multiply(spec.opacity);
    let text_size = ui
        .painter()
        .layout_no_wrap(text.to_string(), font.clone(), text_color)
        .size();
    let bubble_size = text_size + BUBBLE_PAD * 2.0;

    // The pointer's tip lands just outside the halo; the body sits one
    // pointer-height further out.
    let bubble_min = if spec.vertical {
        pos2(
            spec.anchor.x + spec.clearance + BUBBLE_GAP + BUBBLE_ARROW_HEIGHT,
            spec.anchor.y - bubble_size.y / 2.0,
        )
    } else {
        pos2(
            spec.anchor.x - bubble_size.x / 2.0,
            spec.anchor.y - spec.clearance - BUBBLE_GAP - BUBBLE_ARROW_HEIGHT - bubble_size.y,
        )
    };
    let bubble_rect = Rect::from_min_size(bubble_min, bubble_size);
    let fill = spec.fill.gamma_multiply(spec.opacity);
    ui.painter().rect_filled(bubble_rect, BUBBLE_RADIUS, fill);

    // Pointer, centred on the edge facing the thumb (bottom edge when
    // horizontal, left edge when vertical). Its base starts 1pt inside the
    // body so no anti-aliasing seam shows between the two. Points are in
    // clockwise order, which egui's convex polygon fill expects.
    let half_w = BUBBLE_ARROW_WIDTH / 2.0;
    let pointer = if spec.vertical {
        let cy = bubble_rect.center().y;
        let x = bubble_rect.left() + 1.0;
        vec![
            pos2(x, cy - half_w),
            pos2(x, cy + half_w),
            pos2(bubble_rect.left() - BUBBLE_ARROW_HEIGHT, cy),
        ]
    } else {
        let cx = bubble_rect.center().x;
        let y = bubble_rect.bottom() - 1.0;
        vec![
            pos2(cx - half_w, y),
            pos2(cx + half_w, y),
            pos2(cx, bubble_rect.bottom() + BUBBLE_ARROW_HEIGHT),
        ]
    };
    ui.painter().add(egui::Shape::convex_polygon(pointer, fill, Stroke::NONE));

    // Bubble text through `MudText`, in a child `Ui` sized to the text.
    let text_rect = Rect::from_min_size(bubble_rect.min + BUBBLE_PAD, text_size);
    let mut child = ui.new_child(
        UiBuilder::new()
            .max_rect(text_rect)
            .layout(Layout::left_to_right(Align::Center)),
    );
    child.add(
        MudText::new(text.to_string())
            .typo(Typo::Body2)
            .font_size(LABEL_FONT_SIZE)
            .custom_color(text_color)
            .inline(true),
    );
}

/// The FontId that tick labels and the value bubble are rendered with.
///
/// Derived from `MudTheme::typography.body2.font_family` so the render path
/// and any pre-measurement use the same font family (which matters when a
/// theme overrides the default family).
fn label_font(theme: &MudTheme) -> FontId {
    FontId::new(
        LABEL_FONT_SIZE,
        to_egui_family(theme.typography.body2.font_family),
    )
}

/// Arrow keys step by `step`, PageUp/PageDown by ten steps, Home/End jump
/// to the ends. Returns the new value if any of them were pressed this
/// frame. Only call while the slider has keyboard focus.
fn keyboard_adjust(ui: &Ui, current: f32, min: f32, max: f32, step: f32) -> Option<f32> {
    let unit = if step > 0.0 { step } else { (max - min) / 100.0 };
    ui.input(|i| {
        if i.key_pressed(Key::ArrowRight) || i.key_pressed(Key::ArrowUp) {
            Some(current + unit)
        } else if i.key_pressed(Key::ArrowLeft) || i.key_pressed(Key::ArrowDown) {
            Some(current - unit)
        } else if i.key_pressed(Key::PageUp) {
            Some(current + 10.0 * unit)
        } else if i.key_pressed(Key::PageDown) {
            Some(current - 10.0 * unit)
        } else if i.key_pressed(Key::Home) {
            Some(min)
        } else if i.key_pressed(Key::End) {
            Some(max)
        } else {
            None
        }
    })
        .map(|v| snap_to_step(v, min, max, step))
}

/// Returns `v` if it is a finite, sane length; otherwise `fallback`.
#[inline]
fn usable_or(v: f32, fallback: f32) -> f32 {
    if v.is_finite() && v > 0.0 && v < 1.0e5 { v } else { fallback }
}

/// How far along `[min, max]` the value is, as 0..=1. A degenerate range
/// (`max <= min`) maps everything to `0.0`.
#[inline]
fn value_to_fraction(value: f32, min: f32, max: f32) -> f32 {
    if max > min {
        ((value - min) / (max - min)).clamp(0.0, 1.0)
    } else {
        0.0
    }
}

/// Snaps `v` to the nearest `min + n * step`, then clamps into `[min, max]`.
/// A non-positive `step` means continuous: only the clamp applies.
#[inline]
fn snap_to_step(v: f32, min: f32, max: f32, step: f32) -> f32 {
    let v = if step > 0.0 && step.is_finite() {
        min + ((v - min) / step).round() * step
    } else {
        v
    };
    v.clamp(min, max)
}

/// The slider value at fraction `t` (0..=1) along `[min, max]`, snapped to
/// `step`.
#[inline]
fn fraction_to_value(t: f32, min: f32, max: f32, step: f32) -> f32 {
    snap_to_step(min + t.clamp(0.0, 1.0) * (max - min), min, max, step)
}

/// Number of decimal places needed to show multiples of `step` exactly
/// (`1.0` -> 0, `0.25` -> 2). Continuous sliders (`step <= 0`) show 2.
fn step_decimals(step: f32) -> usize {
    if !(step.is_finite() && step > 0.0) {
        return 2;
    }
    let step = f64::from(step);
    for d in 0..=6_usize {
        let scaled = step * 10f64.powi(d as i32);
        if (scaled - scaled.round()).abs() < 1e-4 {
            return d;
        }
    }
    6
}

/// The text shown in the value bubble: `value`, rounded to the step's
/// precision (so `0.1_f32 * 3` shows `0.3`, not `0.30000001192...`), run
/// through `template` if there is one.
fn format_value(value: f32, step: f32, template: Option<&str>) -> String {
    let decimals = step_decimals(step);
    let p = 10f64.powi(decimals as i32);
    let v = (f64::from(value) * p).round() / p;
    match template {
        Some(t) => format_with_template(t, v),
        None => format!("{v:.decimals$}"),
    }
}

/// Renders `value` into `template`, substituting the first `{...}`
/// placeholder found with the formatted number and leaving the rest of the
/// template untouched -- e.g. `"{:.1} px"` with `value = 3.0` renders
/// `"3.0 px"`, and a bare `"{}"` renders the value with Rust's default
/// numeric formatting.
///
/// Rust's `format!` requires a compile-time string literal, so a
/// caller-supplied template can't be passed to it directly; this supports the
/// common `{}` / `{:.N}` placeholder forms by hand instead.
///
/// Malformed templates are repaired rather than rejected, matching the
/// rest of the library's render-path error handling: a template with no
/// `{...}` placeholder is returned as-is (ignoring `value`); an
/// unparseable precision spec falls back to default formatting; only the
/// first placeholder is substituted if the template contains more than
/// one.
fn format_with_template(template: &str, value: f64) -> String {
    let Some(open) = template.find('{') else {
        return template.to_string();
    };
    let Some(close_rel) = template[open..].find('}') else {
        return template.to_string();
    };
    let close = open + close_rel;
    let spec = &template[open + 1..close];

    let formatted = match spec.strip_prefix(":.").and_then(|digits| digits.parse::<usize>().ok()) {
        Some(precision) => format!("{value:.precision$}"),
        None => format!("{value}"),
    };

    let mut out = String::with_capacity(template.len() + formatted.len());
    out.push_str(&template[..open]);
    out.push_str(&formatted);
    out.push_str(&template[close + 1..]);
    out
}

/// Computes how many tick marks to draw for a given `(min, max, step)`, or
/// `None` if the combination is invalid (`step` not positive, or `max` not
/// greater than `min`) or would produce a degenerate (`< 2`) or illegibly
/// dense (`> 30`) tick count. Pure, so the range-validation logic is
/// unit-testable without a live `egui::Context`.
fn tick_count(min: f32, max: f32, step: f32) -> Option<usize> {
    if step <= 0.0 || max <= min {
        return None;
    }
    let count = ((max - min) / step).round() as usize + 1;
    (MIN_TICKS..=MAX_TICKS).contains(&count).then_some(count)
}

/// Position of tick `i` along the rail, as 0..=1, or `None` if it would fall
/// past `max` (which happens when the range isn't an exact multiple of
/// `step`, since [`tick_count`] rounds).
fn tick_fraction(i: usize, step: f32, min: f32, max: f32) -> Option<f32> {
    if max <= min {
        return None;
    }
    let t = i as f32 * step / (max - min);
    (t <= 1.0 + 1e-4).then(|| t.min(1.0))
}

/// Resolves a `Color` to a concrete `Color32`.
///
/// `Default`/`Inherit` resolve straight to the UI's ambient text color
/// without ever touching the theme -- MudBlazor's slider has no semantic
/// meaning for "default" beyond "whatever text looks like here".
/// Any other `Color` is resolved against the current theme's palette,
/// falling back to the UI's selection fill color if the palette has no
/// mapping for it.
fn resolve_color(ui: &Ui, color: Color) -> Color32 {
    if is_ambient_color(color) {
        ui.visuals().text_color()
    } else {
        let theme = MudTheme::fetch(ui.ctx()).unwrap_or_default();
        let palette = theme.current_palette();
        color.resolve(palette).unwrap_or_else(|| ui.visuals().selection.bg_fill)
    }
}

/// Returns `true` for the color variants that resolve straight to the
/// ambient text color rather than through the theme palette. Pure and
/// decoupled from `&Ui` so it's unit-testable; the actual ambient-color
/// lookup stays in [`resolve_color`], which needs a live `Ui`.
#[inline]
fn is_ambient_color(color: Color) -> bool {
    matches!(color, Color::Default | Color::Inherit)
}

/// Black or white, whichever reads better on `bg` (see
/// [`CONTRAST_LUMINANCE_THRESHOLD`]).
fn contrast_text(bg: Color32) -> Color32 {
    fn linear(c: u8) -> f32 {
        let c = f32::from(c) / 255.0;
        if c <= 0.03928 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) }
    }
    let lum = 0.2126 * linear(bg.r()) + 0.7152 * linear(bg.g()) + 0.0722 * linear(bg.b());
    if lum > CONTRAST_LUMINANCE_THRESHOLD { Color32::BLACK } else { Color32::WHITE }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f32 = 1e-4;

    fn approx(a: f32, b: f32) -> bool {
        (a - b).abs() < EPS
    }

    // --- size_metrics ---------------------------------------------------------

    #[test]
    fn size_metrics_for_each_size() {
        let small = size_metrics(Size::Small);
        assert!(approx(small.rail, 2.0) && approx(small.thumb, 16.0));
        let medium = size_metrics(Size::Medium);
        assert!(approx(medium.rail, 4.0) && approx(medium.thumb, 20.0));
        let large = size_metrics(Size::Large);
        assert!(approx(large.rail, 6.0) && approx(large.thumb, 24.0));
    }

    #[test]
    fn size_metrics_extremes_fold_inwards() {
        assert_eq!(size_metrics(Size::ExtraSmall), size_metrics(Size::Small));
        assert_eq!(size_metrics(Size::ExtraLarge), size_metrics(Size::Large));
    }

    #[test]
    fn largest_thumb_fits_in_the_hit_strip() {
        assert!(size_metrics(Size::Large).thumb <= HIT_THICKNESS);
    }

    // --- value <-> fraction ---------------------------------------------------

    #[test]
    fn value_to_fraction_maps_and_clamps() {
        assert!(approx(value_to_fraction(25.0, 0.0, 100.0), 0.25));
        assert!(approx(value_to_fraction(-5.0, 0.0, 100.0), 0.0));
        assert!(approx(value_to_fraction(500.0, 0.0, 100.0), 1.0));
    }

    #[test]
    fn value_to_fraction_degenerate_range_is_zero() {
        assert!(approx(value_to_fraction(3.0, 5.0, 5.0), 0.0));
        assert!(approx(value_to_fraction(3.0, 5.0, 1.0), 0.0));
    }

    #[test]
    fn fraction_to_value_snaps_to_step() {
        assert!(approx(fraction_to_value(0.263, 0.0, 100.0, 10.0), 30.0));
        assert!(approx(fraction_to_value(0.0, 0.0, 100.0, 10.0), 0.0));
        assert!(approx(fraction_to_value(1.0, 0.0, 100.0, 10.0), 100.0));
    }

    #[test]
    fn fraction_to_value_never_exceeds_max() {
        // Range isn't a multiple of the step: 10 -> snaps to 9, not 12.
        assert!(approx(fraction_to_value(1.0, 0.0, 10.0, 3.0), 9.0));
    }

    #[test]
    fn snap_to_step_is_relative_to_min() {
        assert!(approx(snap_to_step(7.0, 5.0, 20.0, 5.0), 5.0));
        assert!(approx(snap_to_step(8.0, 5.0, 20.0, 5.0), 10.0));
    }

    #[test]
    fn non_positive_step_is_continuous() {
        assert!(approx(fraction_to_value(0.5, 0.0, 1.0, 0.0), 0.5));
        assert!(approx(snap_to_step(2.0, 0.0, 1.0, -1.0), 1.0));
    }

    // --- axis -----------------------------------------------------------------

    #[test]
    fn horizontal_axis_maps_points_and_fractions() {
        let axis = Axis { start: pos2(10.0, 5.0), end: pos2(110.0, 5.0) };
        let p = axis.point_at(0.5);
        assert!(approx(p.x, 60.0) && approx(p.y, 5.0));
        assert!(approx(axis.fraction_of(pos2(35.0, 99.0)), 0.25));
        assert!(approx(axis.fraction_of(pos2(-50.0, 5.0)), 0.0));
        assert!(approx(axis.fraction_of(pos2(500.0, 5.0)), 1.0));
    }

    #[test]
    fn vertical_axis_runs_bottom_to_top() {
        let axis = Axis { start: pos2(5.0, 100.0), end: pos2(5.0, 0.0) };
        assert!(approx(axis.fraction_of(pos2(5.0, 75.0)), 0.25));
        assert!(approx(axis.point_at(1.0).y, 0.0));
    }

    #[test]
    fn zero_length_axis_does_not_divide_by_zero() {
        let axis = Axis { start: pos2(1.0, 1.0), end: pos2(1.0, 1.0) };
        assert!(approx(axis.fraction_of(pos2(9.0, 9.0)), 0.0));
    }

    // --- format_with_template -------------------------------------------------

    #[test]
    fn bare_placeholder_uses_default_formatting() {
        assert_eq!(format_with_template("{}", 3.0), "3");
    }

    #[test]
    fn precision_spec_formats_fixed_decimals() {
        assert_eq!(format_with_template("{:.2}", 3.0), "3.00");
        assert_eq!(format_with_template("{:.0}", 3.7), "4");
    }

    #[test]
    fn surrounding_literal_text_is_preserved() {
        assert_eq!(format_with_template("value: {:.1} units", 2.5), "value: 2.5 units");
    }

    #[test]
    fn template_with_no_placeholder_is_returned_unchanged() {
        assert_eq!(format_with_template("plain text", 42.0), "plain text");
    }

    #[test]
    fn unmatched_brace_is_returned_unchanged() {
        assert_eq!(format_with_template("value: {", 1.0), "value: {");
    }

    #[test]
    fn unparseable_precision_spec_falls_back_to_default() {
        assert_eq!(format_with_template("{:.abc}", 5.0), "5");
    }

    #[test]
    fn only_the_first_placeholder_is_substituted() {
        assert_eq!(format_with_template("{} and {}", 1.0), "1 and {}");
    }

    // --- format_value ---------------------------------------------------------

    #[test]
    fn step_decimals_matches_the_step() {
        assert_eq!(step_decimals(1.0), 0);
        assert_eq!(step_decimals(10.0), 0);
        assert_eq!(step_decimals(0.1), 1);
        assert_eq!(step_decimals(0.25), 2);
    }

    #[test]
    fn continuous_step_shows_two_decimals() {
        assert_eq!(step_decimals(0.0), 2);
        assert_eq!(step_decimals(-1.0), 2);
    }

    #[test]
    fn format_value_without_template_follows_step_precision() {
        assert_eq!(format_value(53.0, 1.0, None), "53");
        assert_eq!(format_value(2.5, 0.5, None), "2.5");
    }

    #[test]
    fn format_value_hides_f32_noise() {
        // 0.1_f32 * 3 is 0.30000001192...; the bubble must show 0.3.
        assert_eq!(format_value(0.1_f32 * 3.0, 0.1, None), "0.3");
    }

    #[test]
    fn format_value_applies_template() {
        assert_eq!(format_value(17.5, 0.5, Some("${:.2}")), "$17.50");
    }

    // --- tick_count -----------------------------------------------------------

    #[test]
    fn tick_count_rejects_non_positive_step() {
        assert_eq!(tick_count(0.0, 10.0, 0.0), None);
        assert_eq!(tick_count(0.0, 10.0, -1.0), None);
    }

    #[test]
    fn tick_count_rejects_max_not_greater_than_min() {
        assert_eq!(tick_count(5.0, 5.0, 1.0), None);
        assert_eq!(tick_count(5.0, 4.0, 1.0), None);
    }

    #[test]
    fn tick_count_rejects_degenerate_range() {
        // (0.4 - 0.0) / 1.0 rounds to 0 -> count 1, below MIN_TICKS.
        assert_eq!(tick_count(0.0, 0.4, 1.0), None);
    }

    #[test]
    fn tick_count_rejects_excessively_dense_range() {
        assert_eq!(tick_count(0.0, 1000.0, 1.0), None);
    }

    #[test]
    fn tick_count_accepts_a_normal_range() {
        assert_eq!(tick_count(0.0, 10.0, 1.0), Some(11));
    }

    #[test]
    fn tick_count_boundaries_are_inclusive() {
        assert_eq!(tick_count(0.0, 29.0, 1.0), Some(30));
        assert_eq!(tick_count(0.0, 30.0, 1.0), None);
    }

    // --- tick_fraction --------------------------------------------------------

    #[test]
    fn tick_fraction_spaces_ticks_evenly() {
        assert!(approx(tick_fraction(0, 10.0, 0.0, 100.0).unwrap(), 0.0));
        assert!(approx(tick_fraction(5, 10.0, 0.0, 100.0).unwrap(), 0.5));
        assert!(approx(tick_fraction(10, 10.0, 0.0, 100.0).unwrap(), 1.0));
    }

    #[test]
    fn tick_fraction_drops_ticks_past_max() {
        // Range 10, step 4: tick_count rounds up to 4 ticks (0, 4, 8, 12);
        // the one at 12 is past the end and must not be drawn.
        assert!(tick_fraction(3, 4.0, 0.0, 10.0).is_none());
        assert!(tick_fraction(2, 4.0, 0.0, 10.0).is_some());
    }

    #[test]
    fn tick_fraction_tolerates_float_error_at_the_end() {
        // 3 * 0.1 / 0.3 is slightly above 1.0 in f32; it must still count.
        assert!(approx(tick_fraction(3, 0.1, 0.0, 0.3).unwrap(), 1.0));
    }

    #[test]
    fn tick_fraction_rejects_degenerate_range() {
        assert!(tick_fraction(0, 1.0, 5.0, 5.0).is_none());
    }

    // --- color mapping --------------------------------------------------------

    #[test]
    fn default_and_inherit_are_ambient() {
        assert!(is_ambient_color(Color::Default));
        assert!(is_ambient_color(Color::Inherit));
    }

    #[test]
    fn other_colors_are_not_ambient() {
        assert!(!is_ambient_color(Color::Primary));
    }

    #[test]
    fn contrast_text_is_dark_on_light_and_white_on_brand_colors() {
        assert_eq!(contrast_text(Color32::WHITE), Color32::BLACK);
        assert_eq!(contrast_text(Color32::from_rgb(222, 110, 80)), Color32::WHITE);
        assert_eq!(contrast_text(Color32::from_rgb(124, 124, 255)), Color32::WHITE);
    }
}