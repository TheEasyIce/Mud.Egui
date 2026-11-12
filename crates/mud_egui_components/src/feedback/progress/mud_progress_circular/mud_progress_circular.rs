//! A circle-shaped indicator of progress.
//!
//! Port of `MudProgressCircular.razor.cs` (MudBlazor 8.13.0). The public
//! parameter surface mirrors the Blazor component exactly:
//!
//! | MudBlazor parameter | Rust field / method |
//! |---------------------|---------------------|
//! | `ChildContent` | [`MudProgressCircular::label`] |
//! | `Color` | [`MudProgressCircular::color`] |
//! | `Indeterminate` | [`MudProgressCircular::indeterminate`] |
//! | `Max` | [`MudProgressCircular::max`] |
//! | `Min` | [`MudProgressCircular::min`] |
//! | `Rounded` | [`MudProgressCircular::rounded`] |
//! | `Size` | [`MudProgressCircular::size`] |
//! | `StrokeWidth` | [`MudProgressCircular::stroke_width`] |
//! | `Value` | [`MudProgressCircular::value`] |
//! | `Class` | [`MudProgressCircular::classes`] (stored, not resolved) |
//! | `Tag` | [`MudProgressCircular::tag`] / [`MudProgressCircular::tag_ref`] |
//! | `Style` | not ported: egui has no inline CSS strings |
//! | `UserAttributes` | not ported: there is no HTML tag |
//!
//! # Sizes
//!
//! MudBlazor's SCSS gives three concrete diameters; `ExtraSmall` folds into
//! `Small` and `ExtraLarge` folds into `Large`, exactly as the stylesheet does.
//!
//! | [`Size`] | Diameter |
//! |----------|----------|
//! | `ExtraSmall` / `Small` | 24 pt |
//! | `Medium` | 40 pt |
//! | `Large` / `ExtraLarge` | 56 pt |
//!
//! # Indeterminate
//!
//! The arc rotates a full turn every 1.4 s and its sweep breathes between 30°
//! and 288°. That is a time-based approximation of MudBlazor's CSS
//! `rotate` + `dash` keyframes; the CSS `ease-in-out` timing is replaced by a
//! sinusoidal sweep, which reads the same at a glance and is one line of
//! arithmetic. The bar requests a repaint every frame while indeterminate.
//!
//! # Rounded
//!
//! `rounded(true)` is MudBlazor's `stroke-linecap: round`: a filled circle of
//! radius `stroke_width / 2` is painted at each end of the arc. The `0` and
//! `2π` ends of a determinate arc are therefore never flat.
//!
//! # Performance
//!
//! - Theme fetched once per frame; one `Color::resolve` call.
//! - Every geometric decision is a pure `#[inline]` function over `f32`.
//! - The only allocation is the arc's `Vec<Pos2>`, bounded by
//!   [`ARC_SEGMENTS`]. It is built once per frame and dropped.
//! - The label closure runs exactly once per frame in a child `Ui`.
//! - A determinate circle never requests a repaint.

use std::any::Any;
use std::f32::consts::{FRAC_PI_2, FRAC_PI_6, PI, TAU};

use egui::{
    pos2, Color32, Pos2, Response, Sense, Shape, Stroke, Ui, Vec2,
};

use mud_egui_core::base::component_base::ComponentBase;
use mud_egui_core::enums::{Color, Size};
use mud_egui_theming::theme::MudTheme;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Number of line segments used to approximate one arc.
///
/// 64 is enough that the arc reads as a circle at every [`Size`], including
/// `Large` (56 pt), without a visible polygonal edge.
const ARC_SEGMENTS: usize = 64;

/// Duration of one indeterminate rotation, in seconds. Matches MudBlazor's
/// `1.4s linear infinite`.
const LOOP_SECONDS: f64 = 1.4;

/// Smallest sweep of the indeterminate arc, in radians (30°).
const MIN_SWEEP: f32 = FRAC_PI_6;

/// Largest sweep of the indeterminate arc, in radians (288°).
const MAX_SWEEP: f32 = PI * 1.6;

/// Alpha applied to the track (background) ring.
const TRACK_ALPHA: u8 = 60;

/// Lowest stroke width accepted, in points.
const MIN_STROKE: f32 = 1.0;

// ---------------------------------------------------------------------------
// MudProgressCircular
// ---------------------------------------------------------------------------

/// A circle-shaped indicator of progress.
///
/// Mirrors `MudProgressCircular` in MudBlazor 8.13.0. It is display-only: the
/// Blazor component has no `ValueChanged`, and neither does this port.
#[must_use = "MudProgressCircular does nothing until you call `.show(ui)`"]
pub struct MudProgressCircular {
    /// Shared component state (utility classes). Stored for parity, not
    /// resolved: the ring's colour comes from `color` and its geometry from
    /// `size` and `stroke_width`.
    pub base: ComponentBase,
    /// Accent colour of the arc. Mirrors `Color` (default `Default`).
    pub color: Color,
    /// Diameter preset. Mirrors `Size` (default `Medium`).
    pub size: Size,
    /// Ring thickness in points. Mirrors `StrokeWidth` (default `3`).
    pub stroke_width: u8,
    /// Current progress amount. Mirrors `Value` (default `0.0`).
    pub value: f64,
    /// Lowest possible value. Mirrors `Min` (default `0.0`).
    pub min: f64,
    /// Highest possible value. Mirrors `Max` (default `100.0`).
    pub max: f64,
    /// Display a rotating, breathing arc instead of a value. Mirrors
    /// `Indeterminate` (default `false`).
    pub indeterminate: bool,
    /// Round the arc ends. Mirrors `Rounded` (default `false`).
    pub rounded: bool,
    /// Label drawn over the circle. Mirrors `ChildContent`.
    pub label: Option<Box<dyn FnMut(&mut Ui) + 'static>>,
    /// Arbitrary user data. Mirrors `Tag`.
    pub tag: Option<Box<dyn Any + Send + Sync>>,
}

impl Default for MudProgressCircular {
    /// Value `0`, range `0..=100`, `Default` colour, `Medium` size, stroke
    /// width `3`, not indeterminate, not rounded.
    fn default() -> Self {
        Self {
            base: ComponentBase::new(),
            color: Color::Default,
            size: Size::Medium,
            stroke_width: 3,
            value: 0.0,
            min: 0.0,
            max: 100.0,
            indeterminate: false,
            rounded: false,
            label: None,
            tag: None,
        }
    }
}

impl MudProgressCircular {
    /// Creates a circular progress indicator with MudBlazor's default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the accent colour. Mirrors `Color`.
    #[inline]
    pub fn color(mut self, c: Color) -> Self {
        self.color = c;
        self
    }

    /// Sets the diameter preset. Mirrors `Size`.
    #[inline]
    pub fn size(mut self, s: Size) -> Self {
        self.size = s;
        self
    }

    /// Sets the ring thickness in points. Mirrors `StrokeWidth`.
    ///
    /// The value is clamped to `1..=diameter / 2` at paint time so the ring
    /// can never invert.
    #[inline]
    pub fn stroke_width(mut self, w: u8) -> Self {
        self.stroke_width = w;
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

    /// Makes the ring indeterminate. Mirrors `Indeterminate`.
    #[inline]
    pub fn indeterminate(mut self, b: bool) -> Self {
        self.indeterminate = b;
        self
    }

    /// Rounds the arc ends. Mirrors `Rounded`.
    #[inline]
    pub fn rounded(mut self, b: bool) -> Self {
        self.rounded = b;
        self
    }

    /// Draws a label over the circle. Mirrors `ChildContent`.
    ///
    /// The closure runs exactly once per frame in a child `Ui` sized to the
    /// circle and centred both ways.
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

    /// Draws the ring and returns its `Response`.
    pub fn show(mut self, ui: &mut Ui) -> Response {
        let theme = MudTheme::fetch(ui.ctx()).unwrap_or_default();
        let accent = self
            .color
            .resolve(theme.current_palette())
            .unwrap_or_else(|| ui.visuals().selection.bg_fill);
        let track = with_alpha(accent, TRACK_ALPHA);

        let diameter = diameter(self.size);
        let stroke = effective_stroke_width(self.stroke_width, diameter);
        let radius = ((diameter - stroke) * 0.5).max(0.0);

        let size = Vec2::splat(diameter);
        let (rect, response) = ui.allocate_exact_size(size, Sense::hover());
        if !ui.is_rect_visible(rect) {
            return response;
        }

        let center = rect.center();
        ui.painter()
            .circle_stroke(center, radius, Stroke::new(stroke, track));

        let (start, sweep) = if self.indeterminate {
            let time = ui.input(|i| i.time);
            let phase = indeterminate_phase(time);
            (rotation_for(phase), sweep_for(phase))
        } else {
            let frac = fraction(self.value, self.min, self.max);
            (START_ANGLE, frac * TAU)
        };

        if sweep > 0.0 {
            paint_arc(ui, center, radius, start, sweep, stroke, accent, self.rounded);
        }

        if let Some(render) = self.label.as_mut() {
            let mut child = ui.new_child(
                egui::UiBuilder::new()
                    .max_rect(rect)
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

impl egui::Widget for MudProgressCircular {
    /// Draws an empty circle. Prefer [`MudProgressCircular::show`] so you can
    /// keep the `Response`.
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui)
    }
}

// ---------------------------------------------------------------------------
// Pure helpers (no `egui::Context` required)
// ---------------------------------------------------------------------------

/// Arc start angle, in radians: 12 o'clock (the SCSS rotates the SVG −90°).
const START_ANGLE: f32 = -FRAC_PI_2;

/// Diameter in points for a [`Size`].
#[inline]
fn diameter(size: Size) -> f32 {
    match size {
        Size::ExtraSmall | Size::Small => 24.0,
        Size::Medium => 40.0,
        Size::Large | Size::ExtraLarge => 56.0,
    }
}

/// Stroke width clamped to `1..=diameter / 2`.
#[inline]
fn effective_stroke_width(stroke_width: u8, diameter: f32) -> f32 {
    let max = (diameter * 0.5).max(MIN_STROKE);
    (stroke_width as f32).clamp(MIN_STROKE, max)
}

/// Normalises `value` into `0..=1` against `[min, max]`.
fn fraction(value: f64, min: f64, max: f64) -> f32 {
    if !value.is_finite() || !min.is_finite() || !max.is_finite() || max <= min {
        return 0.0;
    }
    (((value - min) / (max - min)).clamp(0.0, 1.0)) as f32
}

/// The indeterminate loop phase, in `0.0..1.0`.
fn indeterminate_phase(time: f64) -> f32 {
    if !time.is_finite() {
        return 0.0;
    }
    (time.rem_euclid(LOOP_SECONDS) / LOOP_SECONDS) as f32
}

/// Rotation of the indeterminate arc, in radians.
///
/// One full turn per [`LOOP_SECONDS`], starting at [`START_ANGLE`].
fn rotation_for(phase: f32) -> f32 {
    START_ANGLE + phase.clamp(0.0, 1.0) * TAU
}

/// Sweep of the indeterminate arc, in radians.
///
/// `MIN_SWEEP` at phase `0` and `1`, `MAX_SWEEP` at phase `0.5`. The curve is
/// `0.5 − 0.5·cos(2π·phase)`, so the sweep eases in and out once per loop.
fn sweep_for(phase: f32) -> f32 {
    let p = phase.clamp(0.0, 1.0);
    let t = 0.5 - 0.5 * (p * TAU).cos();
    MIN_SWEEP + (MAX_SWEEP - MIN_SWEEP) * t
}

/// Replaces an alpha channel, keeping the (unmultiplied) RGB.
#[inline]
fn with_alpha(c: Color32, alpha: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), alpha)
}

/// `n + 1` points along an arc, from `start` through `sweep` radians.
///
/// The result is a polyline; painting it with a round-capped stroke gives the
/// same silhouette as an SVG arc.
fn arc_points(center: Pos2, radius: f32, start: f32, sweep: f32, n: usize) -> Vec<Pos2> {
    let n = n.max(1);
    let mut points = Vec::with_capacity(n + 1);
    for i in 0..=n {
        let t = i as f32 / n as f32;
        let angle = start + sweep * t;
        points.push(pos2(
            center.x + radius * angle.cos(),
            center.y + radius * angle.sin(),
        ));
    }
    points
}

/// Paints the arc and, when `rounded`, a filled cap at each end.
fn paint_arc(
    ui: &Ui,
    center: Pos2,
    radius: f32,
    start: f32,
    sweep: f32,
    stroke_width: f32,
    color: Color32,
    rounded: bool,
) {
    let points = arc_points(center, radius, start, sweep, ARC_SEGMENTS);
    if points.len() < 2 {
        return;
    }
    ui.painter()
        .add(Shape::line(points.clone(), Stroke::new(stroke_width, color)));

    if rounded {
        let cap = stroke_width * 0.5;
        let last = points.len() - 1;
        ui.painter().circle_filled(points[0], cap, color);
        ui.painter().circle_filled(points[last], cap, color);
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // --- size mapping -----------------------------------------------------

    #[test]
    fn diameter_matches_mudblazor_scss() {
        assert_eq!(diameter(Size::ExtraSmall), 24.0);
        assert_eq!(diameter(Size::Small), 24.0);
        assert_eq!(diameter(Size::Medium), 40.0);
        assert_eq!(diameter(Size::Large), 56.0);
        assert_eq!(diameter(Size::ExtraLarge), 56.0);
    }

    #[test]
    fn diameter_is_monotonic() {
        assert!(diameter(Size::Small) < diameter(Size::Medium));
        assert!(diameter(Size::Medium) < diameter(Size::Large));
    }

    // --- stroke width -----------------------------------------------------

    #[test]
    fn stroke_width_clamps_between_one_and_half_the_diameter() {
        assert_eq!(effective_stroke_width(3, 40.0), 3.0);
        assert_eq!(effective_stroke_width(0, 40.0), MIN_STROKE);
        assert_eq!(effective_stroke_width(200, 40.0), 20.0);
        assert_eq!(effective_stroke_width(3, 24.0), 3.0);
        assert_eq!(effective_stroke_width(20, 24.0), 12.0);
    }

    #[test]
    fn degenerate_diameter_still_yields_a_positive_stroke() {
        assert_eq!(effective_stroke_width(3, 0.0), MIN_STROKE);
        assert_eq!(effective_stroke_width(3, f32::NAN), MIN_STROKE);
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
        assert_eq!(indeterminate_phase(LOOP_SECONDS), 0.0);
        assert!((indeterminate_phase(LOOP_SECONDS * 0.5) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn indeterminate_phase_repairs_non_finite() {
        assert_eq!(indeterminate_phase(f64::NAN), 0.0);
        assert_eq!(indeterminate_phase(f64::INFINITY), 0.0);
    }

    #[test]
    fn rotation_completes_one_turn_per_loop() {
        let a = rotation_for(0.0);
        let b = rotation_for(1.0);
        assert!((a - START_ANGLE).abs() < 1e-6);
        assert!((b - (START_ANGLE + TAU)).abs() < 1e-6);
    }

    #[test]
    fn sweep_breathes_between_min_and_max() {
        let at_zero = sweep_for(0.0);
        let at_half = sweep_for(0.5);
        let at_one = sweep_for(1.0);
        assert!((at_zero - MIN_SWEEP).abs() < 1e-6);
        assert!((at_half - MAX_SWEEP).abs() < 1e-6);
        assert!((at_one - MIN_SWEEP).abs() < 1e-6);
        assert!(at_half > at_zero);
    }

    #[test]
    fn sweep_never_leaves_its_band() {
        for step in 0..=20 {
            let p = step as f32 / 20.0;
            let s = sweep_for(p);
            assert!(s >= MIN_SWEEP - 1e-5 && s <= MAX_SWEEP + 1e-5);
        }
    }

    // --- arc geometry -----------------------------------------------------

    #[test]
    fn arc_points_start_at_twelve_o_clock() {
        let center = pos2(50.0, 50.0);
        let points = arc_points(center, 10.0, START_ANGLE, 0.1, 4);
        // cos(-π/2) = 0, sin(-π/2) = -1, so the first point is above the centre.
        assert!((points[0].x - center.x).abs() < 1e-4);
        assert!(points[0].y < center.y);
    }

    #[test]
    fn arc_points_run_clockwise() {
        let center = pos2(50.0, 50.0);
        let points = arc_points(center, 10.0, START_ANGLE, PI * 0.5, 8);
        // A quarter turn from 12 o'clock ends at 3 o'clock (to the right).
        let last = *points.last().unwrap();
        assert!(last.x > center.x);
        assert!((last.y - center.y).abs() < 1e-4);
    }

    #[test]
    fn arc_points_count_matches_segments_plus_one() {
        for n in 1..=8 {
            let pts = arc_points(pos2(0.0, 0.0), 1.0, 0.0, 1.0, n);
            assert_eq!(pts.len(), n + 1);
        }
    }

    #[test]
    fn arc_points_are_all_on_the_circle() {
        let center = pos2(20.0, 30.0);
        let radius = 12.0;
        for p in arc_points(center, radius, START_ANGLE, TAU, 16) {
            let d = ((p.x - center.x).powi(2) + (p.y - center.y).powi(2)).sqrt();
            assert!((d - radius).abs() < 1e-4);
        }
    }

    #[test]
    fn full_circle_arc_closes_on_itself() {
        let pts = arc_points(pos2(0.0, 0.0), 5.0, START_ANGLE, TAU, 16);
        let first = pts[0];
        let last = *pts.last().unwrap();
        assert!((first.x - last.x).abs() < 1e-4);
        assert!((first.y - last.y).abs() < 1e-4);
    }

    // --- builders ---------------------------------------------------------

    #[test]
    fn defaults_match_mudblazor() {
        let p = MudProgressCircular::new();
        assert!(matches!(p.color, Color::Default));
        assert!(matches!(p.size, Size::Medium));
        assert_eq!(p.stroke_width, 3);
        assert_eq!(p.value, 0.0);
        assert_eq!(p.min, 0.0);
        assert_eq!(p.max, 100.0);
        assert!(!p.indeterminate);
        assert!(!p.rounded);
        assert!(p.label.is_none());
        assert!(p.tag.is_none());
    }

    #[test]
    fn builders_set_every_field() {
        let p = MudProgressCircular::new()
            .color(Color::Info)
            .size(Size::Large)
            .stroke_width(5)
            .value(72.0)
            .min(10.0)
            .max(90.0)
            .indeterminate(true)
            .rounded(true)
            .label(|ui| {
                ui.label("72%");
            })
            .classes(["ml-2"])
            .tag("spinner");

        assert!(matches!(p.color, Color::Info));
        assert!(matches!(p.size, Size::Large));
        assert_eq!(p.stroke_width, 5);
        assert_eq!(p.value, 72.0);
        assert_eq!(p.min, 10.0);
        assert_eq!(p.max, 90.0);
        assert!(p.indeterminate && p.rounded);
        assert!(p.label.is_some());
        assert!(!p.base.class_names().is_empty());
        assert_eq!(p.tag_ref::<&str>(), Some(&"spinner"));
        assert!(p.tag_ref::<u32>().is_none());
    }

    #[test]
    fn tag_round_trips_by_type() {
        let p = MudProgressCircular::new().tag("spinner");
        assert_eq!(p.tag_ref::<&str>(), Some(&"spinner"));
        assert!(p.tag_ref::<u32>().is_none());
    }
}