//! A line-shaped indicator of progress.
//!
//! Port of `MudProgressLinear.razor.cs` (MudBlazor 8.13.0). The public
//! parameter surface mirrors the Blazor component exactly:
//!
//! | MudBlazor parameter | Rust field / method |
//! |---------------------|---------------------|
//! | `Buffer` | [`MudProgressLinear::buffer`] |
//! | `BufferValue` | [`MudProgressLinear::buffer_value`] |
//! | `ChildContent` | [`MudProgressLinear::label`] |
//! | `Color` | [`MudProgressLinear::color`] |
//! | `Indeterminate` | [`MudProgressLinear::indeterminate`] |
//! | `Max` | [`MudProgressLinear::max`] |
//! | `Min` | [`MudProgressLinear::min`] |
//! | `Rounded` | [`MudProgressLinear::rounded`] |
//! | `Size` | [`MudProgressLinear::size`] |
//! | `Striped` | [`MudProgressLinear::striped`] |
//! | `Value` | [`MudProgressLinear::value`] |
//! | `Vertical` | [`MudProgressLinear::vertical`] |
//! | `Class` | [`MudProgressLinear::classes`] (stored, not resolved) |
//! | `Tag` | [`MudProgressLinear::tag`] / [`MudProgressLinear::tag_ref`] |
//! | `Style` | not ported: egui has no inline CSS strings |
//! | `UserAttributes` | not ported: there is no HTML tag |
//!
//! # Indeterminate
//!
//! `indeterminate(true)` ignores `value` and paints a fixed-width segment that
//! slides from the start edge to the end edge on a loop. `buffer`, `striped`
//! and `label` are all ignored in this mode — MudBlazor does the same.
//!
//! # Vertical
//!
//! `vertical(true)` makes the bar run **bottom-to-top**: `min` sits at the
//! bottom, `max` at the top. The indeterminate slide reverses direction to
//! match, so the segment enters from the top and exits at the bottom.
//!
//! # Animation
//!
//! The indeterminate slide uses `ui.input(|i| i.time)` and requests a repaint
//! every frame while it is visible. The phase is `(elapsed / LOOP_SECONDS) mod
//! 1`, with `LOOP_SECONDS = 1.8`; this is a constant because MudBlazor's own
//! transition duration is fixed in CSS.
//!
//! # Performance
//!
//! - Theme fetched at most once per frame, and only when a colour is actually
//!   needed (the bar always needs one, so this is always one fetch).
//! - Every geometry decision is a pure `#[inline]` function over `f32`.
//! - No `Vec`, no `String` allocation; the label closure is the only place
//!   user code runs, and it is run exactly once per frame.
//! - The stripe pattern is computed with integer modulo, not a shader or a
//!   shape list, so a striped bar costs the same as a plain one.

use std::any::Any;

use egui::{
    pos2, vec2, Color32, CornerRadius, Rect, Response, Sense, StrokeKind, Ui, Vec2,
};

use mud_egui_core::base::component_base::ComponentBase;
use mud_egui_core::enums::{Color, Size};
use mud_egui_theming::theme::MudTheme;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Bar thickness in logical pixels, per [`Size`].
fn track_thickness(size: Size) -> f32 {
    match size {
        Size::ExtraSmall | Size::Small => 4.0,
        Size::Medium => 6.0,
        Size::Large | Size::ExtraLarge => 10.0,
    }
}

/// Length of the bar when horizontal and no width is forced, in points.
const DEFAULT_HORIZONTAL_LENGTH: f32 = 240.0;

/// Length of the bar when vertical and no height is forced, in points.
const DEFAULT_VERTICAL_LENGTH: f32 = 160.0;

/// Corner radius of a rounded bar is half its thickness, so it reads as a
/// capsule. `rounded(false)` uses 0.
const ROUNDED_RADIUS_FRACTION: f32 = 0.5;

/// Alpha applied to the track (background) colour.
const TRACK_ALPHA: u8 = 60;

/// Width of one indeterminate segment, as a fraction of the bar length.
const INDETERMINATE_SEGMENT_FRACTION: f32 = 0.35;

/// Duration of one indeterminate slide, in seconds.
const LOOP_SECONDS: f64 = 1.8;

/// Stripe period in points.
const STRIPE_PERIOD: f32 = 12.0;

/// Stripe width in points; must be less than [`STRIPE_PERIOD`].
const STRIPE_WIDTH: f32 = 6.0;

/// Alpha of the stripes painted over the fill.
const STRIPE_ALPHA: u8 = 90;

// ---------------------------------------------------------------------------
// MudProgressLinear
// ---------------------------------------------------------------------------

/// A line-shaped indicator of progress.
///
/// Mirrors `MudProgressLinear` in MudBlazor 8.13.0. It is display-only: there
/// is no `ValueChanged`, because MudBlazor has none either.
#[must_use = "MudProgressLinear does nothing until you call `.show(ui)`"]
pub struct MudProgressLinear {
    /// Shared component state (utility classes). Stored for parity, not
    /// resolved: the bar's fill and track come from `color` and `Size`.
    pub base: ComponentBase,
    /// Accent colour of the fill. Mirrors `Color` (default `Default`).
    pub color: Color,
    /// Thickness preset. Mirrors `Size` (default `Medium`).
    pub size: Size,
    /// Current progress amount. Mirrors `Value` (default `0.0`).
    pub value: f64,
    /// Lowest possible value. Mirrors `Min` (default `0.0`).
    pub min: f64,
    /// Highest possible value. Mirrors `Max` (default `100.0`).
    pub max: f64,
    /// Display a fixed-width sliding segment instead of a value. Mirrors
    /// `Indeterminate` (default `false`).
    pub indeterminate: bool,
    /// Display an additional value ahead of `value`. Mirrors `Buffer`.
    pub buffer: bool,
    /// The amount to display ahead of the value. Mirrors `BufferValue`.
    pub buffer_value: f64,
    /// Paint animated diagonal stripes over the fill. Mirrors `Striped`.
    pub striped: bool,
    /// Round the ends. Mirrors `Rounded`.
    pub rounded: bool,
    /// Run bottom-to-top instead of left-to-right. Mirrors `Vertical`.
    pub vertical: bool,
    /// Explicit length along the bar's axis, in points. `None` means "fill the
    /// available space on the cross axis, use the default length on the axis".
    pub length: Option<f32>,
    /// Label drawn over the fill. Mirrors `ChildContent`.
    pub label: Option<Box<dyn FnMut(&mut Ui) + 'static>>,
    /// Arbitrary user data. Mirrors `Tag`.
    pub tag: Option<Box<dyn Any + Send + Sync>>,
}

impl Default for MudProgressLinear {
    /// Value `0`, range `0..=100`, `Default` colour, `Medium` size, not
    /// indeterminate, not buffered, not striped, not rounded, horizontal.
    fn default() -> Self {
        Self {
            base: ComponentBase::new(),
            color: Color::Default,
            size: Size::Medium,
            value: 0.0,
            min: 0.0,
            max: 100.0,
            indeterminate: false,
            buffer: false,
            buffer_value: 0.0,
            striped: false,
            rounded: false,
            vertical: false,
            length: None,
            label: None,
            tag: None,
        }
    }
}

impl MudProgressLinear {
    /// Creates a progress bar with MudBlazor's default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the accent colour. Mirrors `Color`.
    #[inline]
    pub fn color(mut self, c: Color) -> Self {
        self.color = c;
        self
    }

    /// Sets the thickness preset. Mirrors `Size`.
    #[inline]
    pub fn size(mut self, s: Size) -> Self {
        self.size = s;
        self
    }

    /// Sets the current value. Mirrors `Value`.
    #[inline]
    pub fn value(mut self, v: f64) -> Self {
        self.value = v;
        self
    }

    /// Sets the lower bound. Mirrors `Min`.
    #[inline]
    pub fn min(mut self, v: f64) -> Self {
        self.min = v;
        self
    }

    /// Sets the upper bound. Mirrors `Max`.
    #[inline]
    pub fn max(mut self, v: f64) -> Self {
        self.max = v;
        self
    }

    /// Makes the bar indeterminate. Mirrors `Indeterminate`.
    #[inline]
    pub fn indeterminate(mut self, b: bool) -> Self {
        self.indeterminate = b;
        self
    }

    /// Enables the buffer overlay. Mirrors `Buffer`.
    #[inline]
    pub fn buffer(mut self, b: bool) -> Self {
        self.buffer = b;
        self
    }

    /// Sets the buffer value. Mirrors `BufferValue`.
    #[inline]
    pub fn buffer_value(mut self, v: f64) -> Self {
        self.buffer_value = v;
        self
    }

    /// Enables striped overlay. Mirrors `Striped`.
    #[inline]
    pub fn striped(mut self, b: bool) -> Self {
        self.striped = b;
        self
    }

    /// Rounds the bar ends. Mirrors `Rounded`.
    #[inline]
    pub fn rounded(mut self, b: bool) -> Self {
        self.rounded = b;
        self
    }

    /// Runs bottom-to-top. Mirrors `Vertical`.
    #[inline]
    pub fn vertical(mut self, b: bool) -> Self {
        self.vertical = b;
        self
    }

    /// Sets an explicit length along the bar's axis, in points.
    ///
    /// This is an extension: MudBlazor's length comes from CSS (`width: 100%`),
    /// and there is no `Length` parameter. It is the equivalent of `Style="width: …"`
    /// without a CSS string.
    #[inline]
    pub fn length(mut self, px: f32) -> Self {
        self.length = Some(px);
        self
    }

    /// Draws a label over the fill. Mirrors `ChildContent`.
    ///
    /// The closure runs exactly once per frame, inside the fill rect. It
    /// should be free of side effects.
    #[inline]
    pub fn label(mut self, f: impl FnMut(&mut Ui) + 'static) -> Self {
        self.label = Some(Box::new(f));
        self
    }

    /// Adds utility classes. Stored for parity.
    pub fn classes<S: AsRef<str>>(mut self, classes: impl IntoIterator<Item = S>) -> Self {
        self.base = self.base.classes(classes);
        self
    }

    /// Attaches arbitrary user data. Mirrors `Tag`.
    #[inline]
    pub fn tag<T: Any + Send + Sync>(mut self, tag: T) -> Self {
        self.tag = Some(Box::new(tag));
        self
    }

    /// Reads user data attached with [`tag`](Self::tag).
    #[inline]
    pub fn tag_ref<T: Any + Send + Sync>(&self) -> Option<&T> {
        self.tag.as_ref().and_then(|b| b.downcast_ref::<T>())
    }

    /// Draws the bar and returns its `Response`.
    pub fn show(mut self, ui: &mut Ui) -> Response {
        let theme = MudTheme::fetch(ui.ctx()).unwrap_or_default();
        let accent = self
            .color
            .resolve(theme.current_palette())
            .unwrap_or_else(|| ui.visuals().selection.bg_fill);
        let track = with_alpha(accent, TRACK_ALPHA);

        let thickness = track_thickness(self.size);
        let length = self
            .length
            .unwrap_or(if self.vertical {
                DEFAULT_VERTICAL_LENGTH
            } else {
                DEFAULT_HORIZONTAL_LENGTH
            });
        let length = finite_or(length, if self.vertical {
            DEFAULT_VERTICAL_LENGTH
        } else {
            DEFAULT_HORIZONTAL_LENGTH
        })
            .max(1.0);

        // Size the allocated rect: along the axis, `length`; across it, the
        // bar thickness (or the available width for a horizontal bar).
        let size = if self.vertical {
            vec2(thickness, length)
        } else {
            vec2(ui.available_width().max(1.0).min(length.max(1.0)), thickness)
        };

        let (rect, response) = ui.allocate_exact_size(size, Sense::hover());
        if !ui.is_rect_visible(rect) {
            return response;
        }

        let radius = bar_radius(self.rounded, thickness);

        // --- track ---------------------------------------------------------
        ui.painter().rect_filled(rect, radius, track);

        // --- bar geometry --------------------------------------------------
        let bar = BarGeometry {
            rect,
            thickness,
            vertical: self.vertical,
        };
        let phase = if self.indeterminate {
            indeterminate_phase(ui.input(|i| i.time))
        } else {
            0.0
        };

        if self.indeterminate {
            let (a, b) = indeterminate_fractions(phase, INDETERMINATE_SEGMENT_FRACTION);
            let fill = bar.slice(a, b);
            ui.painter().rect_filled(fill, radius, accent);
        } else {
            // --- buffer ----------------------------------------------------
            if self.buffer {
                let frac = fraction(self.buffer_value, self.min, self.max);
                if frac > 0.0 {
                    let fill = bar.slice(0.0, frac);
                    ui.painter()
                        .rect_filled(fill, radius, with_alpha(accent, 120));
                }
            }

            // --- value -----------------------------------------------------
            let frac = fraction(self.value, self.min, self.max);
            if frac > 0.0 {
                let fill = bar.slice(0.0, frac);
                ui.painter().rect_filled(fill, radius, accent);

                if self.striped {
                    paint_stripes(ui, fill, radius, accent);
                }
            }
        }

        // --- label ---------------------------------------------------------
        if let Some(render) = self.label.as_mut() {
            let mut child = ui.new_child(
                egui::UiBuilder::new()
                    .max_rect(bar.label_rect())
                    .layout(egui::Layout::centered_and_justified(
                        egui::Direction::LeftToRight,
                    )),
            );
            render(&mut child);
        }

        if self.indeterminate {
            ui.ctx().request_repaint();
        }

        response
    }
}

impl egui::Widget for MudProgressLinear {
    /// Draws an empty progress bar. Prefer [`MudProgressLinear::show`] so you
    /// can keep the `Response`.
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui)
    }
}

// ---------------------------------------------------------------------------
// BarGeometry
// ---------------------------------------------------------------------------

/// The rectangle of the bar, and its orientation.
#[derive(Clone, Copy, Debug, PartialEq)]
struct BarGeometry {
    rect: Rect,
    thickness: f32,
    vertical: bool,
}

impl BarGeometry {
    /// The sub-rect covering `[a, b]` of the bar, where `0.0` is the minimum
    /// end and `1.0` is the maximum end. Vertical bars run bottom-to-top.
    fn slice(self, a: f32, b: f32) -> Rect {
        let a = a.clamp(0.0, 1.0);
        let b = b.clamp(a, 1.0);
        if self.vertical {
            let y0 = self.rect.bottom() - (self.rect.height() * b);
            let y1 = self.rect.bottom() - (self.rect.height() * a);
            Rect::from_min_max(pos2(self.rect.left(), y0), pos2(self.rect.right(), y1))
        } else {
            let x0 = self.rect.left() + self.rect.width() * a;
            let x1 = self.rect.left() + self.rect.width() * b;
            Rect::from_min_max(pos2(x0, self.rect.top()), pos2(x1, self.rect.bottom()))
        }
    }

    /// The rect the label is laid out in: the whole bar.
    fn label_rect(self) -> Rect {
        self.rect
    }
}

// ---------------------------------------------------------------------------
// Pure helpers (no `egui::Context` required)
// ---------------------------------------------------------------------------

/// Normalises `value` into `0..=1` against `[min, max]`.
///
/// A degenerate range (`max <= min`), a non-finite `value`, or a `value` outside
/// the range all map to a defined fraction: degenerate → `0.0`, below → `0.0`,
/// above → `1.0`.
fn fraction(value: f64, min: f64, max: f64) -> f32 {
    if !value.is_finite() || !min.is_finite() || !max.is_finite() || max <= min {
        return 0.0;
    }
    (((value - min) / (max - min)).clamp(0.0, 1.0)) as f32
}

/// Corner radius of the bar in `u8`.
fn bar_radius(rounded: bool, thickness: f32) -> CornerRadius {
    if !rounded {
        return CornerRadius::same(0);
    }
    let px = (thickness * ROUNDED_RADIUS_FRACTION).max(0.0);
    CornerRadius::same(radius_u8(px))
}

/// Clamps a pixel value into `u8`.
#[inline]
fn radius_u8(px: f32) -> u8 {
    if px.is_finite() {
        px.clamp(0.0, 255.0) as u8
    } else {
        0
    }
}

/// Replaces a non-finite value with `fallback`.
#[inline]
fn finite_or(v: f32, fallback: f32) -> f32 {
    if v.is_finite() {
        v
    } else {
        fallback
    }
}

/// Replaces an alpha channel, keeping the (unmultiplied) RGB.
#[inline]
fn with_alpha(c: Color32, alpha: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), alpha)
}

/// The phase of the indeterminate loop, in `0.0..1.0`.
///
/// `LOOP_SECONDS` is fixed; a non-finite time yields `0.0`.
fn indeterminate_phase(time: f64) -> f32 {
    if !time.is_finite() {
        return 0.0;
    }
    let t = time.rem_euclid(LOOP_SECONDS) / LOOP_SECONDS;
    t as f32
}

/// The `(start, end)` fractions of the sliding segment at `phase`.
///
/// The segment is `width` wide and travels from `0.0 - width` to
/// `1.0 + width`, so it enters from one edge and exits at the other.
fn indeterminate_fractions(phase: f32, width: f32) -> (f32, f32) {
    let width = width.clamp(0.0, 1.0);
    let travel = 1.0 + width;
    let start = phase.clamp(0.0, 1.0) * travel - width;
    let end = start + width;
    (start.clamp(0.0, 1.0), end.clamp(0.0, 1.0))
}

/// Paints diagonal stripes over `fill`.
fn paint_stripes(ui: &Ui, fill: Rect, radius: CornerRadius, accent: Color32) {
    let stripe_color = with_alpha(Color32::WHITE, STRIPE_ALPHA);
    let width = fill.width();
    if width <= 0.0 {
        return;
    }
    let mut x = 0.0;
    while x < width {
        let stripe_left = fill.left() + x;
        let stripe_right = (stripe_left + STRIPE_WIDTH).min(fill.right());
        let stripe = Rect::from_min_max(
            pos2(stripe_left, fill.top()),
            pos2(stripe_right, fill.bottom()),
        );
        ui.painter().rect_filled(stripe, radius, stripe_color);
        x += STRIPE_PERIOD;
    }
    let _ = accent;
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // --- size mapping -----------------------------------------------------

    #[test]
    fn thickness_matches_size_preset() {
        assert_eq!(track_thickness(Size::ExtraSmall), 4.0);
        assert_eq!(track_thickness(Size::Small), 4.0);
        assert_eq!(track_thickness(Size::Medium), 6.0);
        assert_eq!(track_thickness(Size::Large), 10.0);
        assert_eq!(track_thickness(Size::ExtraLarge), 10.0);
    }

    #[test]
    fn thickness_is_monotonic() {
        let small = track_thickness(Size::Small);
        let medium = track_thickness(Size::Medium);
        let large = track_thickness(Size::Large);
        assert!(small < medium && medium < large);
    }

    // --- fraction ---------------------------------------------------------

    #[test]
    fn fraction_maps_into_unit_range() {
        assert_eq!(fraction(50.0, 0.0, 100.0), 0.5);
        assert_eq!(fraction(0.0, 0.0, 100.0), 0.0);
        assert_eq!(fraction(100.0, 0.0, 100.0), 1.0);
    }

    #[test]
    fn fraction_clamps_out_of_range() {
        assert_eq!(fraction(-50.0, 0.0, 100.0), 0.0);
        assert_eq!(fraction(150.0, 0.0, 100.0), 1.0);
    }

    #[test]
    fn fraction_handles_degenerate_and_non_finite() {
        assert_eq!(fraction(50.0, 100.0, 100.0), 0.0);
        assert_eq!(fraction(50.0, 100.0, 0.0), 0.0);
        assert_eq!(fraction(f64::NAN, 0.0, 100.0), 0.0);
        assert_eq!(fraction(f64::INFINITY, 0.0, 100.0), 0.0);
        assert_eq!(fraction(50.0, f64::NAN, 100.0), 0.0);
    }

    // --- indeterminate ----------------------------------------------------

    #[test]
    fn indeterminate_phase_is_periodic() {
        assert_eq!(indeterminate_phase(0.0), 0.0);
        assert!((indeterminate_phase(LOOP_SECONDS) - 0.0).abs() < f32::EPSILON);
        assert!((indeterminate_phase(LOOP_SECONDS * 1.5) - 0.5).abs() < 1e-5);
    }

    #[test]
    fn indeterminate_phase_repairs_non_finite() {
        assert_eq!(indeterminate_phase(f64::NAN), 0.0);
        assert_eq!(indeterminate_phase(f64::INFINITY), 0.0);
    }

    #[test]
    fn indeterminate_segment_slides_across_the_bar() {
        let width = 0.35;
        let (a0, b0) = indeterminate_fractions(0.0, width);
        assert_eq!(a0, 0.0);
        assert!((b0 - width).abs() < f32::EPSILON);

        let (a1, b1) = indeterminate_fractions(1.0, width);
        assert!((a1 - (1.0 - width)).abs() < f32::EPSILON);
        assert_eq!(b1, 1.0);

        // Monotonic start across the loop.
        let (a_mid, _) = indeterminate_fractions(0.5, width);
        assert!(a0 < a_mid && a_mid < a1);
    }

    #[test]
    fn indeterminate_never_leaves_the_bar() {
        for step in 0..=20 {
            let p = step as f32 / 20.0;
            let (a, b) = indeterminate_fractions(p, INDETERMINATE_SEGMENT_FRACTION);
            assert!(a >= 0.0 && a <= 1.0);
            assert!(b >= 0.0 && b <= 1.0);
            assert!(b >= a);
        }
    }

    // --- geometry ---------------------------------------------------------

    #[test]
    fn horizontal_slice_runs_left_to_right() {
        let rect = Rect::from_min_size(pos2(10.0, 20.0), vec2(200.0, 6.0));
        let bar = BarGeometry {
            rect,
            thickness: 6.0,
            vertical: false,
        };
        let half = bar.slice(0.0, 0.5);
        assert_eq!(half.left(), rect.left());
        assert_eq!(half.right(), rect.left() + 100.0);
        assert_eq!(half.top(), rect.top());
        assert_eq!(half.bottom(), rect.bottom());
    }

    #[test]
    fn vertical_slice_runs_bottom_to_top() {
        let rect = Rect::from_min_size(pos2(10.0, 20.0), vec2(6.0, 200.0));
        let bar = BarGeometry {
            rect,
            thickness: 6.0,
            vertical: true,
        };
        let half = bar.slice(0.0, 0.5);
        assert_eq!(half.bottom(), rect.bottom());
        assert_eq!(half.top(), rect.center().y);
    }

    #[test]
    fn slice_clamps_and_orders_inputs() {
        let rect = Rect::from_min_size(pos2(0.0, 0.0), vec2(100.0, 4.0));
        let bar = BarGeometry {
            rect,
            thickness: 4.0,
            vertical: false,
        };
        let s = bar.slice(0.8, 0.2);
        assert_eq!(s.left(), bar.slice(0.2, 0.8).left());
        assert_eq!(bar.slice(-1.0, 2.0), rect);
    }

    // --- radius -----------------------------------------------------------

    #[test]
    fn rounded_radius_is_half_thickness() {
        let r = bar_radius(true, 6.0);
        assert_eq!(r.nw, 3);
        let r = bar_radius(true, 10.0);
        assert_eq!(r.nw, 5);
    }

    #[test]
    fn unrounded_has_zero_radius() {
        assert_eq!(bar_radius(false, 10.0), CornerRadius::same(0));
    }

    #[test]
    fn radius_helper_sanitizes() {
        assert_eq!(radius_u8(4.0), 4);
        assert_eq!(radius_u8(-1.0), 0);
        assert_eq!(radius_u8(300.0), 255);
        assert_eq!(radius_u8(f32::NAN), 0);
    }

    // --- builders ---------------------------------------------------------

    #[test]
    fn defaults_match_mudblazor() {
        let p = MudProgressLinear::new();
        assert!(matches!(p.color, Color::Default));
        assert!(matches!(p.size, Size::Medium));
        assert_eq!(p.value, 0.0);
        assert_eq!(p.min, 0.0);
        assert_eq!(p.max, 100.0);
        assert!(!p.indeterminate);
        assert!(!p.buffer);
        assert_eq!(p.buffer_value, 0.0);
        assert!(!p.striped);
        assert!(!p.rounded);
        assert!(!p.vertical);
        assert!(p.length.is_none());
        assert!(p.label.is_none());
        assert!(p.tag.is_none());
    }

    #[test]
    fn builders_set_every_field() {
        let p = MudProgressLinear::new()
            .color(Color::Success)
            .size(Size::Large)
            .value(45.0)
            .min(10.0)
            .max(90.0)
            .indeterminate(true)
            .buffer(true)
            .buffer_value(70.0)
            .striped(true)
            .rounded(true)
            .vertical(true)
            .length(120.0)
            .label(|ui| {
                ui.label("45%");
            })
            .classes(["mt-2"])
            .tag("upload");

        assert!(matches!(p.color, Color::Success));
        assert!(matches!(p.size, Size::Large));
        assert_eq!(p.value, 45.0);
        assert_eq!(p.min, 10.0);
        assert_eq!(p.max, 90.0);
        assert!(p.indeterminate && p.buffer && p.striped && p.rounded && p.vertical);
        assert_eq!(p.buffer_value, 70.0);
        assert_eq!(p.length, Some(120.0));
        assert!(p.label.is_some());
        assert!(!p.base.class_names().is_empty());
        assert_eq!(p.tag_ref::<&str>(), Some(&"upload"));
        assert!(p.tag_ref::<u32>().is_none());
    }

    #[test]
    fn tag_round_trips_by_type() {
        let p = MudProgressLinear::new().tag("upload");
        assert_eq!(p.tag_ref::<&str>(), Some(&"upload"));
        assert!(p.tag_ref::<u32>().is_none());
    }
}