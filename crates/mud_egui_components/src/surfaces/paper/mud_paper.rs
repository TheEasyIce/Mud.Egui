//! A themed surface for grouping other components.
//!
//! Port of `MudPaper.razor.cs` (MudBlazor 8.13.0). The public parameter surface
//! mirrors the Blazor component exactly:
//!
//! | MudBlazor parameter | Rust field / method |
//! |---------------------|---------------------|
//! | `ChildContent` | the closure passed to [`MudPaper::show`] |
//! | `Elevation` | [`MudPaper::elevation`] (default `1`) |
//! | `Height` | [`MudPaper::height`] |
//! | `MaxHeight` | [`MudPaper::max_height`] |
//! | `MaxWidth` | [`MudPaper::max_width`] |
//! | `MinHeight` | [`MudPaper::min_height`] |
//! | `MinWidth` | [`MudPaper::min_width`] |
//! | `Outlined` | [`MudPaper::outlined`] |
//! | `Square` | [`MudPaper::square`] |
//! | `Width` | [`MudPaper::width`] |
//! | `Class` | [`MudPaper::classes`] — resolved through [`flavor`](MudPaper::flavor) |
//! | `Tag` | [`MudPaper::tag`] / [`MudPaper::tag_ref`] |
//! | `Style` | not ported: egui has no inline CSS strings |
//! | `UserAttributes` | not ported: there is no HTML tag |
//!
//! Four **extensions** exist that are not MudBlazor parameters; they are marked
//! as such on their builders: [`transparent`](MudPaper::transparent),
//! [`fill`](MudPaper::fill), [`stroke`](MudPaper::stroke) and
//! [`corner_radius`](MudPaper::corner_radius). `transparent` is load-bearing for
//! `MudSplitPanel`; the other three are typed escape hatches for one-off
//! surfaces.
//!
//! # Elevation vs. outlined
//!
//! MudBlazor disables elevation when a component is outlined. That is enforced
//! structurally here: [`paper_shadow`] returns `Shadow::NONE` whenever
//! `outlined` is set, regardless of the elevation value.
//!
//! | `outlined` | `elevation` | Result |
//! |------------|-------------|--------|
//! | `false` | `0` | Flat surface, no shadow |
//! | `false` | `1..=24` | Surface with a drop shadow |
//! | `true` | any | 1 pt outline, **shadow suppressed** |
//!
//! # Resolution order
//!
//! Every visual property has a single, unit-tested precedence:
//!
//! | Property | Highest → lowest |
//! |----------|------------------|
//! | Fill | `transparent` → `fill` → `bg-*` class → theme surface |
//! | Corner radius | `square` → `corner_radius` → `rounded-*` class → theme radius |
//! | Outline | none unless `outlined`; then `stroke` → themed border colour |
//! | Shadow | none if `transparent` or `outlined`; else the elevation ramp |
//!
//! # Per-frame model
//!
//! `MudPaper` is stateless between frames **except** for `tw_state`, which
//! remembers the previous frame's hover/active flags so Tailwind `hover:` and
//! `active:` variants resolve correctly. That is why [`show`](MudPaper::show)
//! takes `&mut self`: it writes the flags back after drawing. Store the paper on
//! a struct field to get correct variant behaviour; a one-shot
//! `ui.add(MudPaper::new())` re-resolves classes every frame and cannot carry
//! state.
//!
//! # Performance
//!
//! - The class resolver (`BsStyle` / `TwStyle`) is cached on the instance, so a
//!   stored paper re-parses nothing between frames.
//! - Every precedence decision is a pure `#[inline]` function over `Copy` values.
//! - Bounds are applied before the content closure runs, so the content sees the
//!   final width/height through `ui.available_width()`.
//! - The theme is fetched once per frame; the palette lookup closure is shared
//!   by the frame builder and the outline colour.

use std::any::Any;

use egui::epaint::Shadow;
use egui::{
    Color32, CornerRadius, Frame, InnerResponse, Response, Stroke, Ui,
};

use mud_egui_bootstrap::bs_style::BsStyle;
use mud_egui_core::base::component_base::ComponentBase;
use mud_egui_core::enums::StyleFlavor;
use mud_egui_tailwind::tw_style::{TwState, TwStyle};
use mud_egui_theming::theme::MudTheme;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Highest elevation accepted; values above this are clamped.
const MAX_ELEVATION: u8 = 24;

/// Width of the outlined paper's border stroke, in logical pixels.
const OUTLINE_STROKE_WIDTH: f32 = 1.0;

/// Root font size in points, passed to the Tailwind resolver.
const REM_SIZE_PT: f32 = 14.0;

/// Fallback theme radius, used when `MudTheme` cannot be fetched.
const FALLBACK_THEME_RADIUS: f32 = 4.0;

// ---------------------------------------------------------------------------
// MudPaper
// ---------------------------------------------------------------------------

/// A themed surface for grouping other components.
///
/// Mirrors `MudPaper` in MudBlazor 8.13.0. Build it with the builder methods and
/// draw it with [`show`](Self::show); store it on a field so the Tailwind
/// variant state persists between frames.
#[must_use = "MudPaper does nothing until you call `.show(ui, ..)`"]
pub struct MudPaper {
    /// Shared component state (utility classes). Resolved into the frame.
    pub base: ComponentBase,
    /// Drop-shadow depth, clamped to [`MAX_ELEVATION`]. Mirrors MudBlazor
    /// `Elevation` (default `1`). Ignored when `outlined` or `transparent`.
    pub elevation: u8,
    /// Draw a 1 pt border and suppress the shadow. Mirrors MudBlazor
    /// `Outlined`.
    pub outlined: bool,
    /// Force zero corner radius. Mirrors MudBlazor `Square`.
    pub square: bool,
    /// Explicit width in points. Mirrors MudBlazor `Width` (a CSS string in
    /// Blazor, points here).
    pub width: Option<f32>,
    /// Explicit height in points. Mirrors MudBlazor `Height`.
    pub height: Option<f32>,
    /// Lower width bound. Mirrors MudBlazor `MinWidth`.
    pub min_width: Option<f32>,
    /// Lower height bound. Mirrors MudBlazor `MinHeight`.
    pub min_height: Option<f32>,
    /// Upper width bound. Mirrors MudBlazor `MaxWidth`.
    pub max_width: Option<f32>,
    /// Upper height bound. Mirrors MudBlazor `MaxHeight`.
    pub max_height: Option<f32>,
    /// Extension: suppress fill, shadow and outline.
    pub transparent: bool,
    /// Extension: explicit background, beats `bg-*` utilities.
    pub fill: Option<Color32>,
    /// Extension: explicit outline, drawn only when `outlined`.
    pub stroke: Option<Stroke>,
    /// Extension: explicit corner radius; `square` wins over it.
    pub corner_radius: Option<CornerRadius>,
    /// Utility-class resolver.
    pub flavor: StyleFlavor,
    /// Bootstrap resolver cache.
    pub bs_style: BsStyle,
    /// Tailwind resolver cache.
    pub tw_style: TwStyle,
    /// Tailwind interaction state, refreshed after each frame.
    pub tw_state: TwState,
    /// Arbitrary user data. Mirrors MudBlazor `Tag`.
    pub tag: Option<Box<dyn Any + Send + Sync>>,
}

impl Default for MudPaper {
    /// Elevation `1`, rounded, not outlined — MudBlazor's defaults.
    fn default() -> Self {
        Self {
            base: ComponentBase::new(),
            elevation: 1,
            outlined: false,
            square: false,
            width: None,
            height: None,
            min_width: None,
            min_height: None,
            max_width: None,
            max_height: None,
            transparent: false,
            fill: None,
            stroke: None,
            corner_radius: None,
            flavor: StyleFlavor::Bootstrap,
            bs_style: BsStyle::default(),
            tw_style: TwStyle::default(),
            tw_state: TwState::default(),
            tag: None,
        }
    }
}

impl MudPaper {
    /// Creates a surface with MudBlazor's default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the drop-shadow depth (clamped to 24). Mirrors `Elevation`.
    #[inline]
    pub fn elevation(mut self, e: u8) -> Self {
        self.elevation = e;
        self
    }

    /// Draws a 1 pt border and suppresses the shadow. Mirrors `Outlined`.
    #[inline]
    pub fn outlined(mut self, b: bool) -> Self {
        self.outlined = b;
        self
    }

    /// Forces zero corner radius. Mirrors `Square`.
    #[inline]
    pub fn square(mut self, b: bool) -> Self {
        self.square = b;
        self
    }

    /// Sets an explicit width in points. Mirrors `Width`.
    #[inline]
    pub fn width(mut self, px: f32) -> Self {
        self.width = Some(px);
        self
    }

    /// Sets an explicit height in points. Mirrors `Height`.
    #[inline]
    pub fn height(mut self, px: f32) -> Self {
        self.height = Some(px);
        self
    }

    /// Sets a lower width bound. Mirrors `MinWidth`.
    #[inline]
    pub fn min_width(mut self, px: f32) -> Self {
        self.min_width = Some(px);
        self
    }

    /// Sets a lower height bound. Mirrors `MinHeight`.
    #[inline]
    pub fn min_height(mut self, px: f32) -> Self {
        self.min_height = Some(px);
        self
    }

    /// Sets an upper width bound. Mirrors `MaxWidth`.
    #[inline]
    pub fn max_width(mut self, px: f32) -> Self {
        self.max_width = Some(px);
        self
    }

    /// Sets an upper height bound. Mirrors `MaxHeight`.
    #[inline]
    pub fn max_height(mut self, px: f32) -> Self {
        self.max_height = Some(px);
        self
    }

    /// **Extension** (not a MudBlazor parameter): suppresses fill, shadow and
    /// outline. Used by `MudSplitPanel`.
    #[inline]
    pub fn transparent(mut self, b: bool) -> Self {
        self.transparent = b;
        self
    }

    /// **Extension**: explicit background, beats `bg-*` utilities.
    #[inline]
    pub fn fill(mut self, c: Color32) -> Self {
        self.fill = Some(c);
        self
    }

    /// **Extension**: explicit outline, drawn only when `outlined` is set.
    #[inline]
    pub fn stroke(mut self, s: Stroke) -> Self {
        self.stroke = Some(s);
        self
    }

    /// **Extension**: explicit corner radius; `square` wins over it.
    #[inline]
    pub fn corner_radius(mut self, r: CornerRadius) -> Self {
        self.corner_radius = Some(r);
        self
    }

    /// Selects the utility-class resolver.
    #[inline]
    pub fn flavor(mut self, f: StyleFlavor) -> Self {
        self.flavor = f;
        self
    }

    /// Adds utility classes to the surface's [`ComponentBase`].
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

    /// Draws the surface and runs `add_contents` inside it.
    ///
    /// Takes `&mut self` because the Tailwind hover/active flags are written
    /// back after drawing; store the paper on a field so they persist between
    /// frames.
    pub fn show<R>(
        &mut self,
        ui: &mut Ui,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> InnerResponse<R> {
        let theme = MudTheme::fetch(ui.ctx()).unwrap_or_default();
        let palette = theme.current_palette();
        let lookup = |key: &str| palette.resolve(key).map(Color32::from);
        let border_color = ui.visuals().widgets.noninteractive.bg_stroke.color;

        // Refresh the flags Tailwind's `dark:` / `hover:` / `active:` variants
        // read, before the frame is built.
        self.tw_state.dark = ui.visuals().dark_mode;

        let mut frame = match self.flavor {
            StyleFlavor::Bootstrap => self.bs_style.build_frame(&self.base, &lookup),
            StyleFlavor::Tailwind => self
                .tw_style
                .build_frame(&self.base, self.tw_state, REM_SIZE_PT, &lookup),
        };

        // --- visual precedence --------------------------------------------
        if self.transparent {
            frame = frame
                .fill(Color32::TRANSPARENT)
                .stroke(Stroke::NONE)
                .shadow(Shadow::NONE);
        } else {
            if let Some(fill) = self.fill {
                frame = frame.fill(fill);
            }
            if let Some(stroke) = paper_stroke(self.outlined, self.stroke, border_color) {
                frame = frame.stroke(stroke);
            } else {
                frame = frame.stroke(Stroke::NONE);
            }
            frame = frame.shadow(paper_shadow(self.elevation, self.outlined));
            frame = apply_radius(
                frame,
                self.square,
                self.corner_radius,
                theme_radius(&theme),
            );
        }

        // --- layout bounds -------------------------------------------------
        let (min_w, max_w) = ordered_bounds(self.min_width, self.max_width);
        let (min_h, max_h) = ordered_bounds(self.min_height, self.max_height);

        let out = frame.show(ui, |ui| {
            apply_axis_bounds(ui, Axis::Width, self.width, min_w, max_w);
            apply_axis_bounds(ui, Axis::Height, self.height, min_h, max_h);
            add_contents(ui)
        });

        // Persist hover/active for the next frame's Tailwind variants.
        self.tw_state.hovered = out.response.hovered();
        self.tw_state.active = out.response.is_pointer_button_down_on();

        out
    }
}

impl egui::Widget for MudPaper {
    /// Draws an empty surface. Prefer [`MudPaper::show`] so the Tailwind state
    /// survives between frames.
    fn ui(mut self, ui: &mut Ui) -> Response {
        self.show(ui, |_| ()).response
    }
}

// ---------------------------------------------------------------------------
// Pure helpers (no `egui::Context` required)
// ---------------------------------------------------------------------------

/// Which axis a bound applies to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Axis {
    Width,
    Height,
}

/// The outline stroke for an outlined paper, or `None` when not outlined.
#[inline]
fn paper_stroke(outlined: bool, custom: Option<Stroke>, color: Color32) -> Option<Stroke> {
    if !outlined {
        return None;
    }
    Some(custom.unwrap_or(Stroke::new(OUTLINE_STROKE_WIDTH, color)))
}

/// The drop shadow for a paper.
///
/// Returns `Shadow::NONE` when the paper is outlined (MudBlazor disables
/// elevation when `Outlined` is used), when it is transparent, or at elevation
/// `0`.
fn paper_shadow(elevation: u8, outlined: bool) -> Shadow {
    if outlined || elevation == 0 {
        return Shadow::NONE;
    }
    elevation_shadow(elevation)
}

/// The elevation ramp, clamped to [`MAX_ELEVATION`].
fn elevation_shadow(elevation: u8) -> Shadow {
    let e = elevation.min(MAX_ELEVATION) as f32;
    Shadow {
        offset: [0, (e * 0.5) as i8],
        blur: (e * 1.5) as u8,
        spread: 0,
        color: Color32::from_black_alpha(((e * 6.0) as u8).min(96)),
    }
}

/// Applies `square`, the explicit override, or the theme radius, in that order.
///
/// The theme radius is only used when neither `square` nor an explicit override
/// is set **and** the class resolver left the radius at zero. This is a
/// heuristic: an explicit `rounded-none` class is overridden by the theme
/// default. Use [`MudPaper::square`] to force square corners unambiguously.
fn apply_radius(
    frame: Frame,
    square: bool,
    explicit: Option<CornerRadius>,
    theme_radius: f32,
) -> Frame {
    if square {
        return frame.corner_radius(CornerRadius::same(0));
    }
    if let Some(r) = explicit {
        return frame.corner_radius(r);
    }
    if radius_is_zero(frame.corner_radius) {
        frame.corner_radius(CornerRadius::same(radius_u8(theme_radius)))
    } else {
        frame
    }
}

/// The theme's default border radius, or [`FALLBACK_THEME_RADIUS`].
#[inline]
fn theme_radius(theme: &MudTheme) -> f32 {
    let r = theme.layout.default_border_radius;
    if r.is_finite() && r >= 0.0 {
        r
    } else {
        FALLBACK_THEME_RADIUS
    }
}

/// `true` when every corner of `r` is zero.
#[inline]
fn radius_is_zero(r: CornerRadius) -> bool {
    r.nw == 0 && r.ne == 0 && r.sw == 0 && r.se == 0
}

/// Clamps a radius to the `u8` payload `CornerRadius` expects.
#[inline]
fn radius_u8(px: f32) -> u8 {
    if px.is_finite() {
        px.clamp(0.0, 255.0) as u8
    } else {
        0
    }
}

/// Swaps an inverted `(min, max)` pair so the lower bound is never above the
/// upper bound.
#[inline]
fn ordered_bounds(min: Option<f32>, max: Option<f32>) -> (Option<f32>, Option<f32>) {
    match (min, max) {
        (Some(lo), Some(hi)) if lo > hi => (Some(hi), Some(lo)),
        other => other,
    }
}

/// Clamps `value` into `[min, max]`.
#[inline]
fn clamp_to_bounds(value: f32, min: Option<f32>, max: Option<f32>) -> f32 {
    let mut v = value;
    if let Some(lo) = min {
        v = v.max(lo);
    }
    if let Some(hi) = max {
        v = v.min(hi);
    }
    v
}

/// Applies the explicit size and the resolved bounds to one axis of `ui`.
fn apply_axis_bounds(
    ui: &mut Ui,
    axis: Axis,
    explicit: Option<f32>,
    min: Option<f32>,
    max: Option<f32>,
) {
    if let Some(lo) = min {
        match axis {
            Axis::Width => ui.set_min_width(lo),
            Axis::Height => ui.set_min_height(lo),
        }
    }
    if let Some(hi) = max {
        match axis {
            Axis::Width => ui.set_max_width(hi),
            Axis::Height => ui.set_max_height(hi),
        }
    }
    if let Some(px) = explicit {
        let clamped = clamp_to_bounds(px, min, max);
        match axis {
            Axis::Width => ui.set_width(clamped),
            Axis::Height => ui.set_height(clamped),
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn red() -> Color32 {
        Color32::from_rgb(200, 30, 30)
    }

    // --- outline / shadow -------------------------------------------------

    #[test]
    fn outlined_paper_has_no_shadow() {
        assert_eq!(paper_shadow(8, true), Shadow::NONE);
        assert_eq!(paper_shadow(1, true), Shadow::NONE);
    }

    #[test]
    fn flat_paper_has_no_shadow() {
        assert_eq!(paper_shadow(0, false), Shadow::NONE);
    }

    #[test]
    fn elevated_paper_gets_a_scaled_shadow() {
        let s = paper_shadow(4, false);
        assert!(s.blur > 0);
        assert!(s.color.a() > 0);
    }

    #[test]
    fn elevation_clamps_at_max() {
        assert_eq!(paper_shadow(40, false).blur, paper_shadow(MAX_ELEVATION, false).blur);
    }

    #[test]
    fn paper_stroke_only_when_outlined() {
        assert!(paper_stroke(false, None, red()).is_none());
        assert!(paper_stroke(false, Some(Stroke::new(2.0, red())), red()).is_none());
    }

    #[test]
    fn paper_stroke_uses_custom_or_fallback() {
        let custom = Stroke::new(2.0, red());
        assert_eq!(paper_stroke(true, Some(custom), Color32::GRAY), Some(custom));

        let fallback = paper_stroke(true, None, red()).expect("outlined must stroke");
        assert_eq!(fallback.width, OUTLINE_STROKE_WIDTH);
        assert_eq!(fallback.color, red());
    }

    // --- radius -----------------------------------------------------------

    #[test]
    fn square_wins_over_everything() {
        let frame = apply_radius(Frame::new(), true, Some(CornerRadius::same(12)), 4.0);
        assert_eq!(frame.corner_radius, CornerRadius::same(0));
    }

    #[test]
    fn explicit_radius_beats_theme() {
        let frame = apply_radius(Frame::new(), false, Some(CornerRadius::same(12)), 4.0);
        assert_eq!(frame.corner_radius, CornerRadius::same(12));
    }

    #[test]
    fn theme_radius_fills_a_zero_radius() {
        let frame = apply_radius(Frame::new(), false, None, 4.0);
        assert_eq!(frame.corner_radius, CornerRadius::same(4));
    }

    #[test]
    fn class_radius_is_not_overwritten() {
        let seeded = Frame::new().corner_radius(CornerRadius::same(9));
        let frame = apply_radius(seeded, false, None, 4.0);
        assert_eq!(frame.corner_radius, CornerRadius::same(9));
    }

    #[test]
    fn radius_helpers_sanitize() {
        assert_eq!(radius_u8(4.0), 4);
        assert_eq!(radius_u8(-1.0), 0);
        assert_eq!(radius_u8(300.0), 255);
        assert_eq!(radius_u8(f32::NAN), 0);
        assert!(radius_is_zero(CornerRadius::same(0)));
        assert!(!radius_is_zero(CornerRadius::same(4)));
    }

    #[test]
    fn theme_radius_repairs_bad_values() {
        assert_eq!(theme_radius(&MudTheme::default()), 4.0);
    }

    // --- bounds -----------------------------------------------------------

    #[test]
    fn ordered_bounds_passes_through_ordered_pairs() {
        assert_eq!(ordered_bounds(None, None), (None, None));
        assert_eq!(ordered_bounds(Some(10.0), None), (Some(10.0), None));
        assert_eq!(ordered_bounds(None, Some(20.0)), (None, Some(20.0)));
        assert_eq!(ordered_bounds(Some(10.0), Some(20.0)), (Some(10.0), Some(20.0)));
    }

    #[test]
    fn ordered_bounds_swaps_inverted_pairs() {
        assert_eq!(ordered_bounds(Some(20.0), Some(10.0)), (Some(10.0), Some(20.0)));
    }

    #[test]
    fn clamp_to_bounds_respects_both_ends() {
        assert_eq!(clamp_to_bounds(50.0, Some(10.0), Some(20.0)), 20.0);
        assert_eq!(clamp_to_bounds(5.0, Some(10.0), Some(20.0)), 10.0);
        assert_eq!(clamp_to_bounds(15.0, Some(10.0), Some(20.0)), 15.0);
        assert_eq!(clamp_to_bounds(15.0, None, Some(20.0)), 15.0);
        assert_eq!(clamp_to_bounds(15.0, Some(10.0), None), 15.0);
    }

    // --- builders ---------------------------------------------------------

    #[test]
    fn defaults_match_mudblazor() {
        let p = MudPaper::new();
        assert_eq!(p.elevation, 1);
        assert!(!p.outlined);
        assert!(!p.square);
        assert!(p.width.is_none() && p.height.is_none());
        assert!(p.min_width.is_none() && p.min_height.is_none());
        assert!(p.max_width.is_none() && p.max_height.is_none());
        assert!(!p.transparent);
        assert!(p.fill.is_none() && p.stroke.is_none() && p.corner_radius.is_none());
        assert!(matches!(p.flavor, StyleFlavor::Bootstrap));
        assert!(p.tag.is_none());
        assert!(p.base.class_names().is_empty());
    }

    #[test]
    fn builders_set_every_field() {
        let p = MudPaper::new()
            .elevation(6)
            .outlined(true)
            .square(true)
            .width(320.0)
            .height(180.0)
            .min_width(100.0)
            .min_height(50.0)
            .max_width(600.0)
            .max_height(400.0)
            .transparent(true)
            .fill(red())
            .stroke(Stroke::new(2.0, red()))
            .corner_radius(CornerRadius::same(8))
            .flavor(StyleFlavor::Tailwind)
            .classes(["p-4"])
            .tag("card");

        assert_eq!(p.elevation, 6);
        assert!(p.outlined && p.square && p.transparent);
        assert_eq!(p.width, Some(320.0));
        assert_eq!(p.height, Some(180.0));
        assert_eq!(p.min_width, Some(100.0));
        assert_eq!(p.min_height, Some(50.0));
        assert_eq!(p.max_width, Some(600.0));
        assert_eq!(p.max_height, Some(400.0));
        assert_eq!(p.fill, Some(red()));
        assert_eq!(p.stroke.map(|s| s.width), Some(2.0));
        assert_eq!(p.corner_radius, Some(CornerRadius::same(8)));
        assert!(matches!(p.flavor, StyleFlavor::Tailwind));
        assert!(!p.base.class_names().is_empty());
        assert_eq!(p.tag_ref::<&str>(), Some(&"card"));
        assert!(p.tag_ref::<u32>().is_none());
    }

    #[test]
    fn tag_round_trips_by_type() {
        let p = MudPaper::new().tag("panel");
        assert_eq!(p.tag_ref::<&str>(), Some(&"panel"));
        assert!(p.tag_ref::<u32>().is_none());
    }
}