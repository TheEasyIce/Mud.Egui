//! A zero-content control that expands into leftover space in a supporting parent.
//!
//! Port of `MudSpacer.cs`. Includes CSS utility class support via `ComponentBase`.
//!
//! # MudBlazor API parity
//!
//! `MudSpacer` has **no component parameters of its own**: it only inherits the
//! `MudComponentBase` members (`Class`, `Style`, `Tag`, `UserAttributes`) and is
//! used as a flexible filler between siblings, e.g. a toolbar's title on the left
//! and its actions on the right. Accordingly this port does not invent bindings,
//! events, variants or content slots.
//!
//! | MudBlazor        | Rust                       | Notes                                   |
//! |------------------|----------------------------|-----------------------------------------|
//! | `Class`          | [`MudSpacer::classes`]     | Utility classes (Bootstrap / Tailwind). |
//! | `Tag`            | [`MudSpacer::tag`]         | User data, read with [`MudSpacer::tag_ref`]. |
//! | `Style`, `UserAttributes` | — | No CSS strings / HTML attributes in egui. |
//!
//! Extensions that already existed in this port and are kept, because a CSS flex
//! container resolves free space for you while an immediate-mode layout must be
//! told how to: [`MudSpacer::reserve`], [`MudSpacer::fraction`],
//! [`MudSpacer::min_size`]. One extension is new: [`MudSpacer::share`], which
//! reproduces "several spacers split the free space equally".
//!
//! # Architecture
//!
//! All sizing is a pure function ([`MudSpacer::resolve_extent`],
//! [`MudSpacer::equal_share_fraction`]) that is unit tested without a context.
//! `show` only picks the layout axis, calls them and allocates a zero-thickness
//! rectangle along the main axis.
//!
//! # Example
//!
//! ```ignore
//! ui.horizontal(|ui| {
//!     ui.heading("Title");
//!     MudSpacer::new().show(ui);        // pushes the button to the right edge
//!     ui.button("Action");
//! });
//!
//! // Keep 80 pt free for trailing widgets, but never collapse below 8 pt.
//! ui.horizontal(|ui| {
//!     ui.label("left");
//!     ui.add(MudSpacer::new().reserve(80.0).min_size(8.0));
//!     ui.label("right");
//! });
//!
//! // Three equally spaced groups: two spacers sharing the free space.
//! ui.horizontal(|ui| {
//!     ui.label("A");
//!     ui.add(MudSpacer::new().share(0, 2));
//!     ui.label("B");
//!     ui.add(MudSpacer::new().share(1, 2));
//!     ui.label("C");
//! });
//! ```

use std::any::Any;
use std::sync::Arc;

use egui::{vec2, Color32, Frame, Response, Sense, Ui, Vec2, Widget};
use mud_egui_bootstrap::bs_style::BsStyle;
use mud_egui_core::base::component_base::ComponentBase;
use mud_egui_core::enums::StyleFlavor;
use mud_egui_tailwind::tw_style::{TwState, TwStyle};
use mud_egui_theming::theme::MudTheme;

/// Base rem size (in points) passed to the Tailwind resolver.
const REM_SIZE_PT: f32 = 14.0;

// =============================================================================
// Private pure helpers
// =============================================================================

/// Clamps a length to a finite, non-negative value (`NaN` and infinities become `0`).
#[inline]
fn sane_length(v: f32) -> f32 {
    if v.is_finite() {
        v.max(0.0)
    } else {
        0.0
    }
}

/// Clamps a fraction to `0.0..=1.0` (`NaN` becomes `1.0`, "take everything").
#[inline]
fn sane_fraction(f: f32) -> f32 {
    if f.is_nan() {
        1.0
    } else {
        f.clamp(0.0, 1.0)
    }
}

// =============================================================================
// Component
// =============================================================================

/// A flexible filler that expands into the free space of the current layout.
///
/// In a horizontal layout the spacer grows along x; in a vertical layout it
/// grows along y. Its cross-axis extent is always `0`.
///
/// # Example
///
/// ```ignore
/// ui.horizontal(|ui| {
///     ui.label("Left");
///     ui.add(MudSpacer::new());
///     ui.label("Right");
/// });
/// ```
#[must_use = "MudSpacer does nothing until you call `.show(ui)` or `ui.add(..)`"]
pub struct MudSpacer {
    /// Space (in points) to leave free after the spacer for following widgets.
    reserve: f32,
    /// Share of the remaining space to take, constrained between `0.0` and `1.0`.
    fraction: f32,
    /// Minimum guaranteed extent along the main layout axis.
    min_size: f32,
    /// Arbitrary user data. MudBlazor: `Tag`.
    tag: Option<Arc<dyn Any + Send + Sync>>,
    /// Shared component state (CSS-like utility classes). MudBlazor: `Class`.
    pub base: ComponentBase,
    /// Which utility-class system to resolve against.
    pub flavor: StyleFlavor,
    /// Bootstrap resolver cache.
    pub bs_style: BsStyle,
    /// Tailwind resolver cache.
    pub tw_style: TwStyle,
    /// State fed to the Tailwind resolver.
    pub tw_state: TwState,
}

impl Default for MudSpacer {
    fn default() -> Self {
        Self {
            reserve: 0.0,
            fraction: 1.0,
            min_size: 0.0,
            tag: None,
            base: ComponentBase::new(),
            flavor: StyleFlavor::default(),
            bs_style: BsStyle::default(),
            tw_style: TwStyle::default(),
            tw_state: TwState::default(),
        }
    }
}

impl MudSpacer {
    /// Creates a spacer that takes all the remaining space (`fraction = 1`,
    /// `reserve = 0`, `min_size = 0`).
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Space in points to leave free after the spacer for the widgets that follow.
    /// Negative and non-finite values become `0`.
    #[inline]
    pub fn reserve(mut self, amount: f32) -> Self {
        self.reserve = sane_length(amount);
        self
    }

    /// Share of the remaining space to take, clamped to `0.0..=1.0`
    /// (`NaN` becomes `1.0`).
    #[inline]
    pub fn fraction(mut self, fraction: f32) -> Self {
        self.fraction = sane_fraction(fraction);
        self
    }

    /// Minimum extent along the main layout axis. Negative and non-finite values
    /// become `0`.
    #[inline]
    pub fn min_size(mut self, min_size: f32) -> Self {
        self.min_size = sane_length(min_size);
        self
    }

    /// Makes this spacer the `index`-th (zero based) of `count` spacers that share
    /// the free space equally, like several `flex-grow: 1` siblings.
    ///
    /// It sets [`MudSpacer::fraction`] to [`MudSpacer::equal_share_fraction`].
    /// The split is exact when item spacing and `reserve` are zero and
    /// approximate otherwise.
    ///
    /// ```ignore
    /// ui.add(MudSpacer::new().share(0, 3));
    /// ```
    #[inline]
    pub fn share(mut self, index: usize, count: usize) -> Self {
        self.fraction = Self::equal_share_fraction(index, count);
        self
    }

    /// Selects the utility-class resolver.
    #[inline]
    pub fn flavor(mut self, flavor: StyleFlavor) -> Self {
        self.flavor = flavor;
        self
    }

    /// Adds utility classes.
    #[inline]
    pub fn classes<S: AsRef<str>>(mut self, classes: impl IntoIterator<Item = S>) -> Self {
        self.base = self.base.classes(classes);
        self
    }

    /// Attaches arbitrary user data to the component (MudBlazor: `Tag`).
    ///
    /// ```ignore
    /// let s = MudSpacer::new().tag("toolbar-gap");
    /// assert_eq!(s.tag_ref::<&str>(), Some(&"toolbar-gap"));
    /// ```
    #[inline]
    pub fn tag<T: Any + Send + Sync>(mut self, tag: T) -> Self {
        self.tag = Some(Arc::new(tag));
        self
    }

    /// Returns the attached user data if it exists and has type `T`.
    #[inline]
    pub fn tag_ref<T: Any>(&self) -> Option<&T> {
        self.tag.as_deref().and_then(|t| t.downcast_ref::<T>())
    }

    // =========================================================================
    // Sizing Math
    // =========================================================================

    /// Fraction that makes the `index`-th of `count` consecutive spacers take an
    /// equal part of the free space.
    ///
    /// Each spacer takes a fraction of what is *left*, so the k-th of n takes
    /// `1 / (n - k)`. `count == 0` and `index >= count` return `1.0`.
    ///
    /// ```ignore
    /// assert_eq!(MudSpacer::equal_share_fraction(0, 2), 0.5);
    /// assert_eq!(MudSpacer::equal_share_fraction(1, 2), 1.0);
    /// ```
    #[inline]
    pub fn equal_share_fraction(index: usize, count: usize) -> f32 {
        if count == 0 || index >= count {
            1.0
        } else {
            1.0 / (count - index) as f32
        }
    }

    /// Computes the spacer's extent along the main axis.
    ///
    /// `extent = max(min_size, (available - item_spacing - reserve).max(0) * fraction)`
    ///
    /// If `available` is not finite (for example in the scroll direction of a
    /// `ScrollArea`) there is no free space to share and `min_size` is returned.
    /// Every input is sanitized, so the result is always finite and `>= 0`.
    ///
    /// ```ignore
    /// assert_eq!(MudSpacer::resolve_extent(200.0, 8.0, 12.0, 1.0, 0.0), 180.0);
    /// assert_eq!(MudSpacer::resolve_extent(f32::INFINITY, 8.0, 0.0, 1.0, 5.0), 5.0);
    /// ```
    #[inline]
    pub fn resolve_extent(
        available: f32,
        item_spacing: f32,
        reserve: f32,
        fraction: f32,
        min_size: f32,
    ) -> f32 {
        let min_size = sane_length(min_size);

        if !available.is_finite() {
            return min_size;
        }

        let remaining = (available - sane_length(item_spacing) - sane_length(reserve)).max(0.0);
        (remaining * sane_fraction(fraction)).max(min_size)
    }

    #[inline]
    fn plan_size(&self, horizontal: bool, avail: Vec2, spacing: Vec2) -> Vec2 {
        if horizontal {
            vec2(
                Self::resolve_extent(avail.x, spacing.x, self.reserve, self.fraction, self.min_size),
                0.0,
            )
        } else {
            vec2(
                0.0,
                Self::resolve_extent(avail.y, spacing.y, self.reserve, self.fraction, self.min_size),
            )
        }
    }

    // =========================================================================
    // Rendering
    // =========================================================================

    /// Allocates the spacer and returns its `Response` (hover sense only).
    pub fn show(mut self, ui: &mut Ui) -> Response {
        let theme = MudTheme::fetch(ui.ctx()).unwrap_or_default();
        let palette = theme.current_palette();

        // Resolve CSS utility frame
        let lookup = |key: &str| palette.resolve(key).map(Color32::from);
        self.tw_state.dark = ui.visuals().dark_mode;

        let frame: Frame = match self.flavor {
            StyleFlavor::Bootstrap => self.bs_style.build_frame(&self.base, &lookup),
            StyleFlavor::Tailwind => {
                self.tw_style
                    .build_frame(&self.base, self.tw_state, REM_SIZE_PT, &lookup)
            }
        };

        frame
            .show(ui, |ui| {
                let horizontal = ui.layout().main_dir.is_horizontal();
                let size =
                    self.plan_size(horizontal, ui.available_size(), ui.spacing().item_spacing);
                ui.allocate_response(size, Sense::hover())
            })
            .inner
    }
}

impl Widget for MudSpacer {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui)
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{Pos2, Rect};

    const EPS: f32 = 0.01;

    /// Runs `f` inside a real egui frame on a headless 400x300 context.
    fn run_in_ui<R>(f: impl FnOnce(&mut Ui) -> R) -> R {
        let ctx = egui::Context::default();
        let input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(400.0, 300.0))),
            ..Default::default()
        };
        let mut f = Some(f);
        let mut out = None;
        let _ = ctx.run_ui(input, |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                if let Some(f) = f.take() {
                    out = Some(f(ui));
                }
            });
        });
        out.expect("the frame closure must run exactly once")
    }

    // ---- sanitizers ---------------------------------------------------------

    #[test]
    fn sane_length_clamps_everything_bad_to_zero() {
        assert_eq!(sane_length(-1.0), 0.0);
        assert_eq!(sane_length(f32::NAN), 0.0);
        assert_eq!(sane_length(f32::INFINITY), 0.0);
        assert_eq!(sane_length(f32::NEG_INFINITY), 0.0);
        assert_eq!(sane_length(7.5), 7.5);
    }

    #[test]
    fn sane_fraction_clamps_and_defaults_nan_to_one() {
        assert_eq!(sane_fraction(f32::NAN), 1.0);
        assert_eq!(sane_fraction(-0.5), 0.0);
        assert_eq!(sane_fraction(2.0), 1.0);
        assert_eq!(sane_fraction(0.25), 0.25);
    }

    // ---- resolve_extent -----------------------------------------------------

    #[test]
    fn takes_all_remaining_space_by_default() {
        assert!((MudSpacer::resolve_extent(200.0, 8.0, 0.0, 1.0, 0.0) - 192.0).abs() < EPS);
    }

    #[test]
    fn reserve_is_left_free() {
        assert!((MudSpacer::resolve_extent(200.0, 8.0, 12.0, 1.0, 0.0) - 180.0).abs() < EPS);
    }

    #[test]
    fn fraction_scales_the_remaining_space() {
        assert!((MudSpacer::resolve_extent(208.0, 8.0, 0.0, 0.5, 0.0) - 100.0).abs() < EPS);
        assert_eq!(MudSpacer::resolve_extent(208.0, 8.0, 0.0, 0.0, 0.0), 0.0);
    }

    #[test]
    fn min_size_is_a_floor() {
        assert_eq!(MudSpacer::resolve_extent(5.0, 8.0, 0.0, 1.0, 20.0), 20.0);
        assert_eq!(MudSpacer::resolve_extent(208.0, 8.0, 0.0, 0.0, 20.0), 20.0);
    }

    #[test]
    fn unbounded_space_yields_min_size() {
        assert_eq!(MudSpacer::resolve_extent(f32::INFINITY, 8.0, 0.0, 1.0, 5.0), 5.0);
        assert_eq!(MudSpacer::resolve_extent(f32::NAN, 8.0, 0.0, 1.0, 0.0), 0.0);
    }

    #[test]
    fn negative_availability_never_goes_negative() {
        assert_eq!(MudSpacer::resolve_extent(-50.0, 8.0, 0.0, 1.0, 0.0), 0.0);
        assert_eq!(MudSpacer::resolve_extent(10.0, 8.0, 100.0, 1.0, 0.0), 0.0);
    }

    #[test]
    fn bad_inputs_are_sanitized() {
        let v = MudSpacer::resolve_extent(200.0, f32::NAN, -5.0, f32::NAN, f32::NAN);
        assert!(v.is_finite() && v >= 0.0);
        assert!((v - 200.0).abs() < EPS);
    }

    // ---- equal shares -------------------------------------------------------

    #[test]
    fn equal_share_fractions() {
        assert!((MudSpacer::equal_share_fraction(0, 3) - 1.0 / 3.0).abs() < EPS);
        assert!((MudSpacer::equal_share_fraction(1, 3) - 0.5).abs() < EPS);
        assert_eq!(MudSpacer::equal_share_fraction(2, 3), 1.0);
        assert_eq!(MudSpacer::equal_share_fraction(0, 1), 1.0);
    }

    #[test]
    fn equal_share_edge_cases() {
        assert_eq!(MudSpacer::equal_share_fraction(0, 0), 1.0);
        assert_eq!(MudSpacer::equal_share_fraction(5, 2), 1.0);
    }

    #[test]
    fn equal_shares_split_free_space_equally_without_spacing() {
        // 3 spacers, 300 pt free, no item spacing or reserve.
        let mut left = 300.0;
        let mut parts = Vec::new();
        for i in 0..3 {
            let e = MudSpacer::resolve_extent(left, 0.0, 0.0, MudSpacer::equal_share_fraction(i, 3), 0.0);
            parts.push(e);
            left -= e;
        }
        for p in parts {
            assert!((p - 100.0).abs() < EPS);
        }
    }

    // ---- builder ------------------------------------------------------------

    #[test]
    fn defaults_take_everything() {
        let s = MudSpacer::new();
        assert_eq!(s.reserve, 0.0);
        assert_eq!(s.fraction, 1.0);
        assert_eq!(s.min_size, 0.0);
        assert!(s.tag_ref::<u32>().is_none());
    }

    #[test]
    fn builder_sanitizes_inputs() {
        let s = MudSpacer::new()
            .reserve(f32::NAN)
            .fraction(9.0)
            .min_size(-3.0);
        assert_eq!(s.reserve, 0.0);
        assert_eq!(s.fraction, 1.0);
        assert_eq!(s.min_size, 0.0);
    }

    #[test]
    fn share_sets_the_fraction() {
        assert!((MudSpacer::new().share(0, 2).fraction - 0.5).abs() < EPS);
    }

    #[test]
    fn tag_round_trips_by_type() {
        let s = MudSpacer::new().tag(7_u8);
        assert_eq!(s.tag_ref::<u8>(), Some(&7));
        assert_eq!(s.tag_ref::<String>(), None);
    }

    // ---- plan_size ----------------------------------------------------------

    #[test]
    fn plan_size_grows_along_the_main_axis_only() {
        let s = MudSpacer::new();
        let h = s.plan_size(true, vec2(100.0, 50.0), vec2(8.0, 4.0));
        assert!((h.x - 92.0).abs() < EPS);
        assert_eq!(h.y, 0.0);

        let v = s.plan_size(false, vec2(100.0, 50.0), vec2(8.0, 4.0));
        assert_eq!(v.x, 0.0);
        assert!((v.y - 46.0).abs() < EPS);
    }

    // ---- rendering (headless frames) ---------------------------------------

    #[test]
    fn horizontal_spacer_fills_the_row() {
        let r = run_in_ui(|ui| ui.horizontal(|ui| MudSpacer::new().show(ui)).inner);
        assert!(r.rect.width() > 100.0);
    }

    #[test]
    fn horizontal_spacer_honors_reserve() {
        let (full, reserved) = run_in_ui(|ui| {
            let a = ui.horizontal(|ui| MudSpacer::new().show(ui)).inner.rect.width();
            let b = ui
                .horizontal(|ui| MudSpacer::new().reserve(50.0).show(ui))
                .inner
                .rect
                .width();
            (a, b)
        });
        assert!((full - reserved - 50.0).abs() < 1.0);
    }

    #[test]
    fn vertical_spacer_fills_the_column() {
        let r = run_in_ui(|ui| ui.vertical(|ui| MudSpacer::new().show(ui)).inner);
        assert!(r.rect.height() > 100.0);
    }

    #[test]
    fn half_fraction_takes_about_half() {
        let (full, half) = run_in_ui(|ui| {
            let a = ui.horizontal(|ui| MudSpacer::new().show(ui)).inner.rect.width();
            let b = ui
                .horizontal(|ui| MudSpacer::new().fraction(0.5).show(ui))
                .inner
                .rect
                .width();
            (a, b)
        });
        assert!((half - full * 0.5).abs() < 5.0);
    }

    #[test]
    fn widget_trait_renders_like_show() {
        let r = run_in_ui(|ui| ui.horizontal(|ui| ui.add(MudSpacer::new())).inner);
        assert!(r.rect.width() > 0.0);
    }
}