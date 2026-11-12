//! A temporary placeholder for content while data is loaded.
//!
//! Port of `MudSkeleton.razor.cs` (MudBlazor 8.13.0). The public parameter
//! surface mirrors the Blazor component exactly:
//!
//! | MudBlazor parameter | Rust field / method |
//! |---------------------|---------------------|
//! | `Animation` | [`MudSkeleton::animation`] |
//! | `Height` | [`MudSkeleton::height`] |
//! | `SkeletonType` | [`MudSkeleton::skeleton_type`] |
//! | `Width` | [`MudSkeleton::width`] |
//! | `Class` | [`MudSkeleton::classes`] (stored, not resolved) |
//! | `Tag` | [`MudSkeleton::tag`] / [`MudSkeleton::tag_ref`] |
//! | `Style` / `UserAttributes` | not ported — see below |
//!
//! # Deliberately not ported
//!
//! | MudBlazor | Reason |
//! |-----------|--------|
//! | `Style` | egui has no inline CSS strings. The typed overrides on this builder ([`fill`](MudSkeleton::fill), [`radius`](MudSkeleton::radius)) cover the two properties a caller would routinely set. |
//! | `UserAttributes` | There is no HTML tag to splat attributes onto. |
//! | `Class` resolution | Stored on `base` for parity. The skeleton's only visual properties are its shape (already an enum) and its fill (already an extension), so a utility-class frame would composite a second fill. |
//!
//! # Extensions
//!
//! | Extension | Purpose |
//! |-----------|---------|
//! | [`fill`](MudSkeleton::fill) | Explicit background colour, beats the theme's default skeleton tint. |
//! | [`radius`](MudSkeleton::radius) | Explicit corner radius in points, overrides the shape's default (Text 4 pt, Circle max, Rectangle 0). |
//!
//! Both are typed escape hatches for the two properties `Style` was routinely
//! used for on a skeleton; they are documented as extensions, not as MudBlazor
//! parameters.
//!
//! # Rule 3 compliance
//!
//! `MudSkeleton` paints no text and no glyphs of its own. It is a solid shape;
//! there is nothing for it to route through [`MudText`](crate::MudText) or
//! [`MudIcon`](crate::MudIcon), and it never calls `painter.text` or
//! `painter.galley`.
//!
//! # `Width` / `Height`
//!
//! In MudBlazor both are CSS strings (`"300px"`, `"30%"`). The port takes
//! `Option<f32>` points, the same substitution [`MudStack`](crate::MudStack),
//! `MudCollapse` and `MudCardMedia` already use. Compute a percentage from
//! `ui.available_width()` / `ui.available_height()` before calling the builder:
//!
//! ```ignore
//! MudSkeleton::new()
//!     .width(ui.available_width() * 0.6)
//!     .height(24.0)
//!     .show(ui);
//! ```
//!
//! When `width` / `height` are unset the per-shape defaults apply:
//!
//! | Shape | Width default | Height default |
//! |-------|---------------|----------------|
//! | `Text` | available width | 20 pt (one `Body1` line) |
//! | `Circle` | `height` (forced square) | 40 pt (an avatar) |
//! | `Rectangle` | available width | 100 pt (a thumbnail) |
//!
//! For `Circle`, the smaller of `width` and `height` wins so the shape stays
//! circular, and the corner radius is set to half that diameter.
//!
//! # Animations
//!
//! | Animation | Behaviour |
//! |-----------|-----------|
//! | `Off` | Static fill. No repaints requested. |
//! | `Pulse` (default) | Opacity fades 1.0 → 0.7 → 1.0 in a 1.3 s loop. |
//! | `Wave` | A 200 pt gradient band of lighter fill sweeps left-to-right in a 1.0 s loop. |
//!
//! Both animations are time-driven (`ui.input(|i| i.time)`) and request a
//! repaint every frame while active. All `MudSkeleton`s on a page animate in
//! lockstep, matching MudBlazor's CSS behaviour. The wave band's peak alpha
//! and width are named constants ([`WAVE_PEAK_ALPHA`], [`WAVE_BAND_WIDTH`]) so a
//! theme that wants a subtler or wider wave can adjust them in one place.

use std::any::Any;

use egui::{
    epaint::{Mesh, Vertex},
    pos2, vec2, Color32, CornerRadius, Pos2, Response, Sense, Shape, Ui, Vec2,
};

use mud_egui_core::base::component_base::ComponentBase;
use mud_egui_core::enums::{Animation, SkeletonType};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Default height of a `Text` skeleton, in points. Approximately one `Body1`
/// line-height, which is what a text skeleton is a placeholder for.
const DEFAULT_TEXT_HEIGHT: f32 = 20.0;

/// Default diameter of a `Circle` skeleton, in points. Matches the medium
/// `MudAvatar` size, the most common circle-skeleton target.
const DEFAULT_CIRCLE_DIAMETER: f32 = 40.0;

/// Default height of a `Rectangle` skeleton, in points. A typical image
/// thumbnail height.
const DEFAULT_RECTANGLE_HEIGHT: f32 = 100.0;

/// Corner radius of a `Text` skeleton, in points. Matches MudBlazor's
/// `.mud-skeleton-text` rounded corners.
const TEXT_RADIUS: f32 = 4.0;

/// Alpha applied to the skeleton's fill. Low enough to read as a placeholder,
/// high enough to be visible against both light and dark surfaces.
const SKELETON_ALPHA: u8 = 40;

/// Period of the pulse animation, in seconds. Matches MudBlazor's
/// `1.3s` CSS animation.
const PULSE_PERIOD_SECONDS: f32 = 1.3;

/// Lowest opacity the pulse animation reaches. Matches MudBlazor's `0.7` at
/// the halfway point of the loop.
const PULSE_MIN_ALPHA: f32 = 0.7;

/// Period of the wave animation, in seconds. Matches MudBlazor's `1s` CSS
/// animation.
const WAVE_PERIOD_SECONDS: f32 = 1.0;

/// Width of the wave's gradient band, in points. Matches MudBlazor's
/// `200px` pseudo-element.
const WAVE_BAND_WIDTH: f32 = 200.0;

/// Peak alpha of the wave band. The wave is drawn as a three-column vertex-
/// coloured mesh: `0 → WAVE_PEAK_ALPHA → 0` across the band.
const WAVE_PEAK_ALPHA: u8 = 90;

/// Colour of the wave's highlight. A near-white band reads as a "shine" over
/// both light and dark skeleton fills; MudBlazor uses a theme token, but the
/// visual result is the same.
const WAVE_HIGHLIGHT: Color32 = Color32::WHITE;

// ---------------------------------------------------------------------------
// MudSkeleton
// ---------------------------------------------------------------------------

/// A temporary placeholder for content while data is loaded.
///
/// Mirrors `MudSkeleton` in MudBlazor 8.13.0. Build it with the builder
/// methods and draw it with [`show`](Self::show).
#[must_use = "MudSkeleton does nothing until you call `.show(ui)`"]
pub struct MudSkeleton {
    /// Shared component state (utility classes). Stored for parity; not
    /// resolved. Mirrors MudBlazor `Class`.
    pub base: ComponentBase,
    /// Shape of the placeholder. Mirrors MudBlazor `SkeletonType` (default
    /// `Text`).
    pub skeleton_type: SkeletonType,
    /// Animation style. Mirrors MudBlazor `Animation` (default `Pulse`).
    pub animation: Animation,
    /// Explicit width in points; `None` uses the shape's default. Mirrors
    /// MudBlazor `Width` (a CSS string in Blazor, points here).
    pub width: Option<f32>,
    /// Explicit height in points; `None` uses the shape's default. Mirrors
    /// MudBlazor `Height`.
    pub height: Option<f32>,
    /// **Extension**: explicit background colour. Beats the theme's default
    /// skeleton tint.
    pub fill: Option<Color32>,
    /// **Extension**: explicit corner radius in points. Overrides the shape's
    /// default (`Text` 4 pt, `Circle` max, `Rectangle` 0).
    pub radius: Option<f32>,
    /// Arbitrary user data. Mirrors MudBlazor `Tag`.
    pub tag: Option<Box<dyn Any + Send + Sync>>,
}

impl Default for MudSkeleton {
    /// A `Text` skeleton with the `Pulse` animation and shape defaults.
    fn default() -> Self {
        Self {
            base: ComponentBase::new(),
            skeleton_type: SkeletonType::Text,
            animation: Animation::Pulse,
            width: None,
            height: None,
            fill: None,
            radius: None,
            tag: None,
        }
    }
}

impl MudSkeleton {
    /// Creates a skeleton with MudBlazor's default settings (Text shape, Pulse
    /// animation, no explicit size).
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the shape. Mirrors MudBlazor `SkeletonType`.
    #[inline]
    pub fn skeleton_type(mut self, kind: SkeletonType) -> Self {
        self.skeleton_type = kind;
        self
    }

    /// Sets the animation style. Mirrors MudBlazor `Animation`.
    #[inline]
    pub fn animation(mut self, animation: Animation) -> Self {
        self.animation = animation;
        self
    }

    /// Sets an explicit width in points. Mirrors MudBlazor `Width`.
    #[inline]
    pub fn width(mut self, px: f32) -> Self {
        self.width = Some(px);
        self
    }

    /// Sets an explicit height in points. Mirrors MudBlazor `Height`.
    #[inline]
    pub fn height(mut self, px: f32) -> Self {
        self.height = Some(px);
        self
    }

    /// **Extension**: explicit background colour.
    #[inline]
    pub fn fill(mut self, color: Color32) -> Self {
        self.fill = Some(color);
        self
    }

    /// **Extension**: explicit corner radius in points.
    #[inline]
    pub fn radius(mut self, px: f32) -> Self {
        self.radius = Some(px);
        self
    }

    /// Adds utility classes. Stored for parity; no frame is painted from them.
    /// Mirrors MudBlazor `Class`.
    pub fn classes<S: AsRef<str>>(mut self, classes: impl IntoIterator<Item = S>) -> Self {
        self.base = self.base.classes(classes);
        self
    }

    /// Attaches arbitrary user data. Mirrors MudBlazor `Tag`.
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

    /// Draws the skeleton and returns its `Response`.
    pub fn show(self, ui: &mut Ui) -> Response {
        let available = ui.available_size();
        let (w, h) = skeleton_dimensions(
            self.skeleton_type,
            self.width,
            self.height,
            available,
        );

        let (rect, response) =
            ui.allocate_exact_size(vec2(w, h), Sense::hover());
        if !ui.is_rect_visible(rect) {
            return response;
        }

        // Resolve the base fill.
        let base = self
            .fill
            .unwrap_or_else(|| ui.visuals().widgets.noninteractive.bg_fill);
        let base = with_alpha(base, SKELETON_ALPHA);

        let corner = CornerRadius::same(radius_u8(skeleton_radius(
            self.skeleton_type,
            self.radius,
            (w, h),
        )));

        let now = ui.input(|i| i.time);
        let alpha_mul = if matches!(self.animation, Animation::Pulse) {
            pulse_alpha(now)
        } else {
            1.0
        };

        ui.painter()
            .rect_filled(rect, corner, base.gamma_multiply(alpha_mul));

        if matches!(self.animation, Animation::Wave) {
            paint_wave(ui, rect, base, now);
        }

        if self.animation.is_animated() {
            ui.ctx().request_repaint();
        }

        response
    }
}

impl egui::Widget for MudSkeleton {
    /// Delegates to [`MudSkeleton::show`]; the canonical way to add a
    /// skeleton via `ui.add(..)`.
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui)
    }
}

// ---------------------------------------------------------------------------
// Pure helpers (no `egui::Context` required)
// ---------------------------------------------------------------------------

/// Resolves the final `(width, height)` of a skeleton.
///
/// `Circle` forces a square: `height` wins over `width`, and if neither is
/// set the [`DEFAULT_CIRCLE_DIAMETER`] is used. Non-finite or negative
/// dimensions are repaired to the shape's default.
fn skeleton_dimensions(
    kind: SkeletonType,
    width: Option<f32>,
    height: Option<f32>,
    available: Vec2,
) -> (f32, f32) {
    let avail_w = sane_positive(available.x, 0.0);
    let width = width.map(|v| sane_positive(v, 0.0));
    let height = height.map(|v| sane_positive(v, 0.0));

    match kind {
        SkeletonType::Circle => {
            let d = match (width, height) {
                (Some(a), Some(b)) => a.min(b),
                (Some(a), None) => a,
                (None, Some(b)) => b,
                (None, None) => DEFAULT_CIRCLE_DIAMETER,
            };
            (d, d)
        }
        SkeletonType::Text => (
            width.unwrap_or(avail_w),
            height.unwrap_or(DEFAULT_TEXT_HEIGHT),
        ),
        SkeletonType::Rectangle => (
            width.unwrap_or(avail_w),
            height.unwrap_or(DEFAULT_RECTANGLE_HEIGHT),
        ),
    }
}

/// Resolves the final corner radius in points.
///
/// An explicit `override_radius` always wins. Otherwise: `Text` gets
/// [`TEXT_RADIUS`], `Circle` gets half the smaller side (so it is a perfect
/// circle), and `Rectangle` gets `0`.
fn skeleton_radius(kind: SkeletonType, override_radius: Option<f32>, size: (f32, f32)) -> f32 {
    if let Some(r) = override_radius {
        return sane_positive(r, 0.0);
    }
    match kind {
        SkeletonType::Text => TEXT_RADIUS,
        SkeletonType::Circle => (size.0.min(size.1)) * 0.5,
        SkeletonType::Rectangle => 0.0,
    }
}

/// Returns `v` if it is finite and strictly positive; otherwise `fallback`.
#[inline]
fn sane_positive(v: f32, fallback: f32) -> f32 {
    if v.is_finite() && v > 0.0 {
        v
    } else {
        fallback
    }
}

/// Clamps a pixel value into `u8`, as `CornerRadius` expects.
#[inline]
fn radius_u8(px: f32) -> u8 {
    if px.is_finite() {
        px.clamp(0.0, 255.0) as u8
    } else {
        0
    }
}

/// Replaces an alpha channel, keeping the (unmultiplied) RGB.
#[inline]
fn with_alpha(c: Color32, alpha: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), alpha)
}

/// Opacity multiplier for the pulse animation at time `t`, in `[PULSE_MIN_ALPHA, 1.0]`.
///
/// `1.0` at the start and end of the loop, `PULSE_MIN_ALPHA` at the halfway
/// point. Non-finite time yields `1.0` (no pulse), which is what a frame that
/// has not yet started should look like.
fn pulse_alpha(t: f64) -> f32 {
    if !t.is_finite() {
        return 1.0;
    }
    let phase = ((t / PULSE_PERIOD_SECONDS as f64).rem_euclid(1.0)) as f32;
    let cos = (phase * std::f32::consts::TAU).cos();
    let mid = (1.0 + cos) * 0.5; // 1.0 at phase 0, 0.0 at phase 0.5
    PULSE_MIN_ALPHA + (1.0 - PULSE_MIN_ALPHA) * mid
}

/// Wave phase at time `t`, in `[0.0, 1.0)`.
fn wave_phase(t: f64) -> f32 {
    if !t.is_finite() {
        return 0.0;
    }
    (t / WAVE_PERIOD_SECONDS as f64).rem_euclid(1.0) as f32
}

/// The left, centre, and right x-coordinates of the wave band at `phase`.
///
/// The band is `band_width` wide. At phase `0.0` it starts fully off the left
/// edge; at phase `1.0` it has completely exited the right edge. The three
/// coordinates are clamped to `[0.0, rect_width]` so the resulting mesh never
/// extends outside the skeleton.
fn wave_band_x(phase: f32, rect_width: f32, band_width: f32) -> (f32, f32, f32) {
    let w = rect_width.max(0.0);
    let bw = band_width.max(0.0);
    let phase = phase.clamp(0.0, 1.0);
    // Start with the band's left edge at -bw; end with it at w.
    let left = phase * (w + bw) - bw;
    let center = left + bw * 0.5;
    let right = left + bw;
    (
        left.clamp(0.0, w),
        center.clamp(0.0, w),
        right.clamp(0.0, w),
    )
}

/// Paints the wave band over `rect`.
///
/// A three-column vertex-coloured mesh: alpha `0` at the band edges, peak at
/// the centre. The band's three x-coordinates come from [`wave_band_x`], so a
/// band that has partially exited the rect is clipped to it by construction
/// (no `clip_rect` needed).
fn paint_wave(ui: &Ui, rect: Rect, base: Color32, now: f64) {
    let phase = wave_phase(now);
    let (left, center, right) = wave_band_x(phase, rect.width(), WAVE_BAND_WIDTH);
    if left >= right {
        return;
    }

    let _ = base; // reserved for future per-wave tinting
    let peak = with_alpha(WAVE_HIGHLIGHT, WAVE_PEAK_ALPHA);
    let zero = with_alpha(WAVE_HIGHLIGHT, 0);

    let top = rect.top();
    let bottom = rect.bottom();
    let left_abs = rect.left() + left;
    let center_abs = rect.left() + center;
    let right_abs = rect.left() + right;

    let mut mesh = Mesh::default();
    mesh.vertices.push(Vertex {
        pos: pos2(left_abs, top),
        uv: Pos2::ZERO,
        color: zero,
    });
    mesh.vertices.push(Vertex {
        pos: pos2(left_abs, bottom),
        uv: Pos2::ZERO,
        color: zero,
    });
    mesh.vertices.push(Vertex {
        pos: pos2(center_abs, top),
        uv: Pos2::ZERO,
        color: peak,
    });
    mesh.vertices.push(Vertex {
        pos: pos2(center_abs, bottom),
        uv: Pos2::ZERO,
        color: peak,
    });
    mesh.vertices.push(Vertex {
        pos: pos2(right_abs, top),
        uv: Pos2::ZERO,
        color: zero,
    });
    mesh.vertices.push(Vertex {
        pos: pos2(right_abs, bottom),
        uv: Pos2::ZERO,
        color: zero,
    });
    // Two quads: 0-1-2 / 1-2-3 covers the left half, 2-3-4 / 3-4-5 the right.
    mesh.indices.extend_from_slice(&[0, 1, 2, 1, 2, 3, 2, 3, 4, 3, 4, 5]);

    ui.painter().add(Shape::mesh(mesh));
}

// Import Rect here so it is in scope for `paint_wave`; it comes from egui
// directly (the `use egui::{...}` block above intentionally omits it to keep
// the surface minimal, but the pure helper signature needs the type).
use egui::Rect;

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f32 = 1e-3;

    fn approx(a: f32, b: f32) -> bool {
        (a - b).abs() < EPS
    }

    // --- dimensions -------------------------------------------------------

    #[test]
    fn text_defaults() {
        let (w, h) = skeleton_dimensions(
            SkeletonType::Text,
            None,
            None,
            vec2(800.0, 600.0),
        );
        assert!(approx(w, 800.0));
        assert!(approx(h, DEFAULT_TEXT_HEIGHT));
    }

    #[test]
    fn text_respects_explicit_size() {
        let (w, h) = skeleton_dimensions(
            SkeletonType::Text,
            Some(120.0),
            Some(24.0),
            vec2(800.0, 600.0),
        );
        assert!(approx(w, 120.0));
        assert!(approx(h, 24.0));
    }

    #[test]
    fn circle_defaults_to_a_square() {
        let (w, h) = skeleton_dimensions(
            SkeletonType::Circle,
            None,
            None,
            vec2(800.0, 600.0),
        );
        assert!(approx(w, DEFAULT_CIRCLE_DIAMETER));
        assert!(approx(h, DEFAULT_CIRCLE_DIAMETER));
        assert!(approx(w, h));
    }

    #[test]
    fn circle_uses_the_smaller_side_when_both_are_set() {
        let (w, h) = skeleton_dimensions(
            SkeletonType::Circle,
            Some(60.0),
            Some(40.0),
            vec2(800.0, 600.0),
        );
        assert!(approx(w, 40.0));
        assert!(approx(h, 40.0));
    }

    #[test]
    fn circle_uses_the_one_set_side() {
        let (w, h) = skeleton_dimensions(
            SkeletonType::Circle,
            Some(50.0),
            None,
            vec2(800.0, 600.0),
        );
        assert!(approx(w, 50.0) && approx(h, 50.0));

        let (w, h) = skeleton_dimensions(
            SkeletonType::Circle,
            None,
            Some(30.0),
            vec2(800.0, 600.0),
        );
        assert!(approx(w, 30.0) && approx(h, 30.0));
    }

    #[test]
    fn rectangle_defaults() {
        let (w, h) = skeleton_dimensions(
            SkeletonType::Rectangle,
            None,
            None,
            vec2(800.0, 600.0),
        );
        assert!(approx(w, 800.0));
        assert!(approx(h, DEFAULT_RECTANGLE_HEIGHT));
    }

    #[test]
    fn non_finite_dimensions_fall_back_to_defaults() {
        let (w, h) = skeleton_dimensions(
            SkeletonType::Text,
            Some(f32::NAN),
            Some(f32::INFINITY),
            vec2(800.0, 600.0),
        );
        assert!(approx(w, 800.0));
        assert!(approx(h, DEFAULT_TEXT_HEIGHT));
    }

    #[test]
    fn negative_dimensions_fall_back_to_defaults() {
        let (w, h) = skeleton_dimensions(
            SkeletonType::Text,
            Some(-10.0),
            Some(-5.0),
            vec2(800.0, 600.0),
        );
        assert!(approx(w, 800.0));
        assert!(approx(h, DEFAULT_TEXT_HEIGHT));
    }

    // --- radius -----------------------------------------------------------

    #[test]
    fn radius_defaults_per_shape() {
        assert!(approx(
            skeleton_radius(SkeletonType::Text, None, (100.0, 20.0)),
            TEXT_RADIUS
        ));
        assert!(approx(
            skeleton_radius(SkeletonType::Circle, None, (40.0, 40.0)),
            20.0
        ));
        assert!(approx(
            skeleton_radius(SkeletonType::Circle, None, (60.0, 40.0)),
            20.0
        ));
        assert!(approx(
            skeleton_radius(SkeletonType::Rectangle, None, (100.0, 40.0)),
            0.0
        ));
    }

    #[test]
    fn radius_override_wins() {
        assert!(approx(
            skeleton_radius(SkeletonType::Rectangle, Some(8.0), (100.0, 40.0)),
            8.0
        ));
        assert!(approx(
            skeleton_radius(SkeletonType::Text, Some(0.0), (100.0, 20.0)),
            0.0
        ));
    }

    #[test]
    fn radius_override_is_sanitized() {
        assert!(approx(
            skeleton_radius(SkeletonType::Text, Some(f32::NAN), (100.0, 20.0)),
            0.0
        ));
        assert!(approx(
            skeleton_radius(SkeletonType::Text, Some(-4.0), (100.0, 20.0)),
            0.0
        ));
    }

    // --- pulse ------------------------------------------------------------

    #[test]
    fn pulse_starts_and_ends_at_full_alpha() {
        assert!(approx(pulse_alpha(0.0), 1.0));
        assert!(approx(pulse_alpha(PULSE_PERIOD_SECONDS as f64), 1.0));
        assert!(approx(pulse_alpha(PULSE_PERIOD_SECONDS as f64 * 2.0), 1.0));
    }

    #[test]
    fn pulse_reaches_minimum_at_half_period() {
        let mid = PULSE_PERIOD_SECONDS as f64 * 0.5;
        assert!(approx(pulse_alpha(mid), PULSE_MIN_ALPHA));
    }

    #[test]
    fn pulse_stays_within_band() {
        for step in 0..=20 {
            let t = step as f64 * PULSE_PERIOD_SECONDS as f64 / 20.0;
            let a = pulse_alpha(t);
            assert!(
                a >= PULSE_MIN_ALPHA - EPS && a <= 1.0 + EPS,
                "pulse alpha out of band at t={t}: {a}"
            );
        }
    }

    #[test]
    fn pulse_repairs_non_finite_time() {
        assert!(approx(pulse_alpha(f64::NAN), 1.0));
        assert!(approx(pulse_alpha(f64::INFINITY), 1.0));
    }

    // --- wave -------------------------------------------------------------

    #[test]
    fn wave_phase_is_periodic() {
        assert!(approx(wave_phase(0.0), 0.0));
        assert!(approx(wave_phase(WAVE_PERIOD_SECONDS as f64), 0.0));
        assert!(approx(
            wave_phase(WAVE_PERIOD_SECONDS as f64 * 0.5),
            0.5
        ));
    }

    #[test]
    fn wave_phase_repairs_non_finite() {
        assert!(approx(wave_phase(f64::NAN), 0.0));
        assert!(approx(wave_phase(f64::INFINITY), 0.0));
    }

    #[test]
    fn wave_band_is_fully_off_left_at_phase_zero() {
        let (l, _c, r) = wave_band_x(0.0, 400.0, WAVE_BAND_WIDTH);
        assert!(approx(l, 0.0));
        assert!(approx(r, 0.0));
    }

    #[test]
    fn wave_band_is_fully_off_right_at_phase_one() {
        let (l, _c, r) = wave_band_x(1.0, 400.0, WAVE_BAND_WIDTH);
        assert!(approx(l, 400.0));
        assert!(approx(r, 400.0));
    }

    #[test]
    fn wave_band_is_centred_at_phase_half() {
        let (l, c, r) = wave_band_x(0.5, 400.0, WAVE_BAND_WIDTH);
        assert!(approx(c, 200.0));
        assert!(approx(l, 200.0 - WAVE_BAND_WIDTH * 0.5));
        assert!(approx(r, 200.0 + WAVE_BAND_WIDTH * 0.5));
    }

    #[test]
    fn wave_band_never_extends_outside_the_rect() {
        for step in 0..=20 {
            let phase = step as f32 / 20.0;
            let (l, c, r) = wave_band_x(phase, 400.0, WAVE_BAND_WIDTH);
            assert!(l >= 0.0 && l <= 400.0);
            assert!(c >= 0.0 && c <= 400.0);
            assert!(r >= 0.0 && r <= 400.0);
            assert!(l <= c && c <= r);
        }
    }

    // --- builders ---------------------------------------------------------

    #[test]
    fn defaults_match_mudblazor() {
        let s = MudSkeleton::new();
        assert!(matches!(s.skeleton_type, SkeletonType::Text));
        assert!(matches!(s.animation, Animation::Pulse));
        assert!(s.width.is_none());
        assert!(s.height.is_none());
        assert!(s.fill.is_none());
        assert!(s.radius.is_none());
        assert!(s.tag.is_none());
        assert!(s.base.class_names().is_empty());
    }

    #[test]
    fn builders_set_every_field() {
        let s = MudSkeleton::new()
            .skeleton_type(SkeletonType::Circle)
            .animation(Animation::Wave)
            .width(48.0)
            .height(48.0)
            .fill(Color32::RED)
            .radius(6.0)
            .classes(["mb-2"])
            .tag("avatar-loading");

        assert!(matches!(s.skeleton_type, SkeletonType::Circle));
        assert!(matches!(s.animation, Animation::Wave));
        assert_eq!(s.width, Some(48.0));
        assert_eq!(s.height, Some(48.0));
        assert_eq!(s.fill, Some(Color32::RED));
        assert_eq!(s.radius, Some(6.0));
        assert!(!s.base.class_names().is_empty());
        assert_eq!(s.tag_ref::<&str>(), Some(&"avatar-loading"));
        assert!(s.tag_ref::<u32>().is_none());
    }

    #[test]
    fn tag_round_trips_by_type() {
        let s = MudSkeleton::new().tag("loading");
        assert_eq!(s.tag_ref::<&str>(), Some(&"loading"));
        assert!(s.tag_ref::<u32>().is_none());
    }
}