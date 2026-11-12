//! A container that animates its height to hide or reveal its content.
//!
//! Port of `MudCollapse.razor.cs` (MudBlazor 8.13.0). The public parameter
//! surface mirrors the Blazor component exactly:
//!
//! | MudBlazor parameter | Rust field / method |
//! |---------------------|---------------------|
//! | `ChildContent` | the closure passed to [`MudCollapse::show`] |
//! | `Expanded` | [`MudCollapse::expanded`] |
//! | `ExpandedChanged` | [`CollapseResponse::expanded_changed`] |
//! | `MaxHeight` | [`MudCollapse::max_height`] |
//! | `OnAnimationEnd` | [`CollapseResponse::animation_ended`] |
//! | `Class` | [`MudCollapse::classes`] — resolved through [`flavor`](MudCollapse::flavor) |
//! | `Tag` | [`MudCollapse::tag`] / [`MudCollapse::tag_ref`] |
//! | `Style` | not ported: egui has no inline CSS strings |
//! | `UserAttributes` | not ported: there is no HTML tag |
//!
//! # Rendering paths
//!
//! | Path | Condition | What happens |
//! |------|-----------|--------------|
//! | Collapsed | `!expanded && progress <= 0` | Zero-sized response; the closure is **not called**. |
//! | Settled expanded | `expanded && progress >= 1` | Content drawn plainly, measured, exactly its height reserved. |
//! | Animating | otherwise | Content drawn clipped to `cached × progress`, opacity `= progress`, `cached × progress` reserved. |
//!
//! # Events
//!
//! egui has no event-callback objects, so the two Blazor callbacks are
//! reported as **data about this frame** on [`CollapseResponse`]:
//!
//! - [`expanded_changed`](CollapseResponse::expanded_changed) is `Some(new)`
//!   only when the `expanded` argument differs from the previous frame's. It
//!   is never set on the first frame.
//! - [`animation_ended`](CollapseResponse::animation_ended) is `true` for
//!   exactly one frame, when progress reaches its target. With
//!   `duration(0.0)` no animation is ever in progress, so it never fires.
//!
//! # Persisted state
//!
//! Per collapse, three values live in `egui::Memory` under the widget id:
//!
//! | Key | Type | Purpose |
//! |-----|------|---------|
//! | `"height"` | `f32` | Last measured natural height |
//! | `"prev_expanded"` | `bool` | Previous frame's `expanded` (for the change event) |
//! | `"animating"` | `bool` | Whether we were animating last frame (for the end event) |
//!
//! `id_salt` disambiguates collapses that share a parent, exactly as
//! `MudToggleGroup` does for its ripple keys.
//!
//! # Performance
//!
//! - Nothing is drawn when collapsed; the closure is skipped entirely.
//! - Heights are only re-cached (and a repaint requested) when they change by
//!   more than 0.1 pt, so sub-pixel float noise cannot cause a repaint loop.
//! - Every geometric decision is a pure `#[inline]` function over `f32`.
//! - The theme is fetched once per frame, and only when the collapse is not
//!   collapsed.
//! - No `Vec`, no `String` allocation beyond what the content itself does.

use std::any::Any;
use std::ops::Deref;

use egui::{
    pos2, vec2, Align, Color32, Layout, Rect, Response, Sense, StrokeKind, Ui, UiBuilder, Vec2,
};

use mud_egui_bootstrap::bs_style::BsStyle;
use mud_egui_core::base::component_base::ComponentBase;
use mud_egui_core::enums::StyleFlavor;
use mud_egui_tailwind::tw_style::{TwState, TwStyle};
use mud_egui_theming::theme::MudTheme;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Default animation duration in seconds, matching MudBlazor's CSS transition.
const DEFAULT_DURATION: f32 = 0.2;

/// Height changes smaller than this are ignored, preventing repaint loops from
/// sub-pixel float noise.
const HEIGHT_EPSILON: f32 = 0.1;

/// Largest height the probe `Ui` is allowed to assume.
///
/// It must be finite: an infinite height makes content that stretches to the
/// available height measure as infinite, which poisons parent scroll areas.
const MAX_NATURAL_HEIGHT: f32 = 10_000.0;

/// Root font size in points, passed to the Tailwind resolver.
const REM_SIZE_PT: f32 = 14.0;

// ---------------------------------------------------------------------------
// MudCollapse
// ---------------------------------------------------------------------------

/// A container that animates its height and opacity to hide or reveal content.
///
/// Mirrors `MudCollapse` in MudBlazor 8.13.0.
#[must_use = "MudCollapse does nothing until you call `.show(ui, ..)`"]
pub struct MudCollapse {
    /// Shared component state (utility classes). Resolved into the frame.
    pub base: ComponentBase,
    /// Whether the content is shown. Mirrors MudBlazor `Expanded`.
    pub expanded: bool,
    /// Animation duration in seconds. `0.0` means instant. Mirrors the
    /// MudBlazor CSS transition (no parameter of its own).
    pub duration: f32,
    /// Visible height cap in points. Mirrors MudBlazor `MaxHeight`.
    pub max_height: Option<f32>,
    /// Stable identity for animation state and event detection.
    pub id_salt: Option<egui::Id>,
    /// Utility-class resolver. Mirrors the shared component base behaviour.
    pub flavor: StyleFlavor,
    /// Bootstrap resolver cache.
    pub bs_style: BsStyle,
    /// Tailwind resolver cache.
    pub tw_style: TwStyle,
    /// Tailwind interaction state. Feeds `hover:` / `active:` / `dark:`.
    pub tw_state: TwState,
    /// Arbitrary user data. Mirrors MudBlazor `Tag`.
    pub tag: Option<Box<dyn Any + Send + Sync>>,
}

impl Default for MudCollapse {
    /// Collapsed, 0.2 s animation, no height cap, Bootstrap flavor.
    fn default() -> Self {
        Self {
            base: ComponentBase::new(),
            expanded: false,
            duration: DEFAULT_DURATION,
            max_height: None,
            id_salt: None,
            flavor: StyleFlavor::Bootstrap,
            bs_style: BsStyle::default(),
            tw_style: TwStyle::default(),
            tw_state: TwState::default(),
            tag: None,
        }
    }
}

impl MudCollapse {
    /// Creates a collapsed container with MudBlazor's defaults.
    pub fn new() -> Self {
        Self::default()
    }

    /// Shows or hides the content. Mirrors `Expanded`.
    #[inline]
    pub fn expanded(mut self, b: bool) -> Self {
        self.expanded = b;
        self
    }

    /// Sets the animation duration in seconds (`0.0` is instant).
    ///
    /// Negative or non-finite values fall back to [`DEFAULT_DURATION`].
    #[inline]
    pub fn duration(mut self, seconds: f32) -> Self {
        self.duration = seconds;
        self
    }

    /// Caps the visible height in points. Mirrors `MaxHeight`.
    ///
    /// Negative values become `0`; non-finite values remove the cap.
    #[inline]
    pub fn max_height(mut self, px: f32) -> Self {
        self.max_height = Some(px);
        self
    }

    /// Sets a stable identity. Required when two collapses share a parent.
    #[inline]
    pub fn id_salt(mut self, salt: impl std::hash::Hash + std::fmt::Debug) -> Self {
        self.id_salt = Some(egui::Id::new(salt));
        self
    }

    /// Selects the utility-class resolver.
    #[inline]
    pub fn flavor(mut self, f: StyleFlavor) -> Self {
        self.flavor = f;
        self
    }

    /// Adds utility classes to the frame's [`ComponentBase`].
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

    /// Draws the collapse and reports this frame's events.
    ///
    /// The closure is **not called** while the collapse is fully collapsed,
    /// which is what makes a hidden panel free.
    pub fn show<R>(
        mut self,
        ui: &mut Ui,
        add_content: impl FnOnce(&mut Ui) -> R,
    ) -> CollapseResponse<R> {
        let id = ui.id().with(self.id_salt);
        let duration = sane_duration(self.duration);
        let progress = compute_progress(ui.ctx(), id, self.expanded, duration);

        // --- events (pure detection over last frame's persisted state) -----
        let prev_expanded: Option<bool> = ui
            .ctx()
            .memory(|m| m.data.get_temp(id.with("prev_expanded")));
        let expanded_changed = detect_expanded_change(prev_expanded, self.expanded);

        let was_animating: bool = ui
            .ctx()
            .memory(|m| m.data.get_temp(id.with("animating")))
            .unwrap_or(false);
        let animating_now = is_animating(progress, self.expanded);
        let animation_ended = detect_animation_end(was_animating, animating_now);

        ui.ctx().memory_mut(|m| {
            m.data.insert_temp(id.with("prev_expanded"), self.expanded);
            m.data.insert_temp(id.with("animating"), animating_now);
        });

        // --- collapsed fast path -------------------------------------------
        if !self.expanded && progress <= 0.0 {
            ui.ctx()
                .memory_mut(|m| m.data.remove_temp::<f32>(id.with("height")));
            return CollapseResponse {
                response: ui.allocate_response(Vec2::ZERO, Sense::hover()),
                inner: None,
                progress: 0.0,
                expanded_changed,
                animation_ended,
            };
        }

        // --- frame resolution ----------------------------------------------
        let theme = MudTheme::fetch(ui.ctx()).unwrap_or_default();
        let palette = theme.current_palette();
        let lookup = |key: &str| palette.resolve(key).map(Color32::from);
        let frame = match self.flavor {
            StyleFlavor::Bootstrap => self.bs_style.build_frame(&self.base, &lookup),
            StyleFlavor::Tailwind => {
                self.tw_style
                    .build_frame(&self.base, self.tw_state, REM_SIZE_PT, &lookup)
            }
        };

        let max_h = sane_max_height(self.max_height).unwrap_or(f32::INFINITY);
        let margin = frame.inner_margin;
        let pad_x = (margin.left as f32) + (margin.right as f32);
        let pad_y = (margin.top as f32) + (margin.bottom as f32);

        let avail_w = placed_width(ui.available_width());
        let content_w = (avail_w - pad_x).max(1.0);
        let max_content_h = if max_h.is_finite() {
            (max_h - pad_y).max(0.0)
        } else {
            MAX_NATURAL_HEIGHT
        };

        let cached: Option<f32> = ui.ctx().memory(|m| m.data.get_temp(id.with("height")));

        let settled = self.expanded && progress >= 1.0;
        let (clip_content_h, opacity) = if settled {
            (max_content_h, 1.0)
        } else {
            // On the very first expansion the cached height is unknown, so the
            // clip is zero and the content is measured invisibly; the next
            // frame animates with a known height.
            let animatable = cached.map(|c| c.min(max_content_h)).unwrap_or(0.0);
            (animatable * progress, progress)
        };

        // --- draw content (exactly once) -----------------------------------
        let start = ui.cursor().min;
        let content_start = pos2(start.x + margin.left as f32, start.y + margin.top as f32);
        let child_max = Rect::from_min_size(content_start, vec2(content_w, max_content_h));
        let mut child = ui.new_child(
            UiBuilder::new()
                .max_rect(child_max)
                .layout(Layout::top_down(Align::LEFT)),
        );
        let clip_rect = Rect::from_min_size(start, vec2(avail_w, clip_content_h + pad_y));
        child.set_clip_rect(clip_rect.intersect(ui.clip_rect()));
        if opacity < 1.0 {
            child.multiply_opacity(opacity);
        }

        let inner = add_content(&mut child);
        let natural_h = child.min_rect().height();
        let measured = if natural_h.is_finite() {
            natural_h.max(0.0)
        } else {
            0.0
        };

        if needs_height_update(cached, measured) {
            ui.ctx()
                .memory_mut(|m| m.data.insert_temp(id.with("height"), measured));
            ui.ctx().request_repaint();
        }

        // --- reserve the visible height and paint the frame ----------------
        let total_h = (measured + pad_y).min(max_h);
        let visible_h = visible_height(total_h, progress);
        let frame_rect = Rect::from_min_size(start, vec2(avail_w, visible_h));
        let response = ui.allocate_rect(frame_rect, Sense::hover());
        paint_frame_bg(ui, frame_rect, &frame);

        CollapseResponse {
            response,
            inner: Some(inner),
            progress,
            expanded_changed,
            animation_ended,
        }
    }
}

impl egui::Widget for MudCollapse {
    /// Draws an empty collapse. Prefer [`MudCollapse::show`] so you can read
    /// the event fields on [`CollapseResponse`].
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui, |_| ()).response
    }
}

// ---------------------------------------------------------------------------
// CollapseResponse
// ---------------------------------------------------------------------------

/// What [`MudCollapse::show`] did this frame.
pub struct CollapseResponse<R> {
    /// The reserved area. `CollapseResponse` dereferences to it, so
    /// `out.hovered()` works.
    pub response: Response,
    /// The closure's return value; `None` while fully collapsed.
    pub inner: Option<R>,
    /// `0.0` collapsed … `1.0` expanded.
    pub progress: f32,
    /// `Some(new)` on the frame `expanded` changed; never set on the first
    /// frame. Mirrors MudBlazor `ExpandedChanged`.
    pub expanded_changed: Option<bool>,
    /// `true` for exactly one frame when the animation reaches its target.
    /// Mirrors MudBlazor `OnAnimationEnd`. Never `true` with `duration(0.0)`.
    pub animation_ended: bool,
}

impl<R> CollapseResponse<R> {
    /// `true` when fully expanded.
    #[inline]
    pub fn is_fully_expanded(&self) -> bool {
        progress_is_fully_expanded(self.progress)
    }

    /// `true` when fully collapsed.
    #[inline]
    pub fn is_fully_collapsed(&self) -> bool {
        progress_is_fully_collapsed(self.progress)
    }
}

impl<R> Deref for CollapseResponse<R> {
    type Target = Response;

    fn deref(&self) -> &Self::Target {
        &self.response
    }
}

// ---------------------------------------------------------------------------
// Pure helpers (no `egui::Context` required)
// ---------------------------------------------------------------------------

/// `true` when `progress` has reached the fully-expanded end.
#[inline]
fn progress_is_fully_expanded(progress: f32) -> bool {
    progress >= 1.0
}

/// `true` when `progress` has reached the fully-collapsed end.
#[inline]
fn progress_is_fully_collapsed(progress: f32) -> bool {
    progress <= 0.0
}

/// Repairs a duration: negative or non-finite falls back to the default.
#[inline]
fn sane_duration(seconds: f32) -> f32 {
    if seconds.is_finite() && seconds >= 0.0 {
        seconds
    } else {
        DEFAULT_DURATION
    }
}

/// Repairs a non-negative pixel length: `NaN` and negatives become `0`.
#[inline]
fn sane_length(px: f32) -> f32 {
    if px.is_finite() {
        px.max(0.0)
    } else {
        0.0
    }
}

/// Repairs a height cap: non-finite means "no cap"; negatives become `0`.
#[inline]
fn sane_max_height(h: Option<f32>) -> Option<f32> {
    h.filter(|px| px.is_finite()).map(sane_length)
}

/// Whether the animation is still in flight.
#[inline]
fn is_animating(progress: f32, expanded: bool) -> bool {
    if expanded {
        progress < 1.0
    } else {
        progress > 0.0
    }
}

/// Whether the animation reached its target on this frame.
#[inline]
fn detect_animation_end(was_animating: bool, now_animating: bool) -> bool {
    was_animating && !now_animating
}

/// `Some(new)` when `expanded` changed; `None` on the first frame and on
/// steady frames.
#[inline]
fn detect_expanded_change(previous: Option<bool>, current: bool) -> Option<bool> {
    previous.filter(|p| *p != current).map(|_| current)
}

/// The visible height of the animation, clamped to `0..=1` of `natural`.
#[inline]
fn visible_height(natural: f32, progress: f32) -> f32 {
    let n = if natural.is_finite() {
        natural.max(0.0)
    } else {
        0.0
    };
    n * progress.clamp(0.0, 1.0)
}

/// Whether the cached height must be replaced by `measured`.
#[inline]
fn needs_height_update(cached: Option<f32>, measured: f32) -> bool {
    match cached {
        None => true,
        Some(c) => (c - measured).abs() > HEIGHT_EPSILON,
    }
}

/// Sanitizes the available width.
#[inline]
fn placed_width(available: f32) -> f32 {
    if available.is_finite() {
        available.max(0.0)
    } else {
        0.0
    }
}

/// Reads (or advances) the animation progress for `expanded`.
///
/// With `duration == 0`, returns the target directly without touching
/// `egui::Memory`.
fn compute_progress(
    ctx: &egui::Context,
    id: egui::Id,
    expanded: bool,
    duration: f32,
) -> f32 {
    if duration <= 0.0 {
        if expanded {
            1.0
        } else {
            0.0
        }
    } else {
        ctx.animate_bool_with_time(id, expanded, duration)
    }
}

/// Paints the frame's shadow, fill and stroke over `rect`.
///
/// `Frame::shadow` and `Frame::stroke` are plain values in egui 0.27+, not
/// `Option`s. Their disabled sentinels are `Shadow::NONE` (zero alpha) and
/// `Stroke::NONE` (zero width), so both are detected by value rather than by an
/// `Option` match. Guarding on alpha and width — rather than `PartialEq` —
/// keeps the check correct even if egui ever changes those derives.
fn paint_frame_bg(ui: &Ui, rect: Rect, frame: &egui::Frame) {
    if frame.shadow.color.a() > 0 {
        ui.painter()
            .add(frame.shadow.as_shape(rect, frame.corner_radius));
    }
    if frame.fill != Color32::TRANSPARENT {
        ui.painter()
            .rect_filled(rect, frame.corner_radius, frame.fill);
    }
    if frame.stroke.width > 0.0 {
        ui.painter()
            .rect_stroke(rect, frame.corner_radius, frame.stroke, StrokeKind::Inside);
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // --- sanitizers -------------------------------------------------------

    #[test]
    fn sane_duration_keeps_valid_values() {
        assert_eq!(sane_duration(0.0), 0.0);
        assert_eq!(sane_duration(0.25), 0.25);
        assert_eq!(sane_duration(2.0), 2.0);
    }

    #[test]
    fn sane_duration_repairs_bad_values() {
        assert_eq!(sane_duration(-1.0), DEFAULT_DURATION);
        assert_eq!(sane_duration(f32::NAN), DEFAULT_DURATION);
        assert_eq!(sane_duration(f32::INFINITY), DEFAULT_DURATION);
        assert_eq!(sane_duration(f32::NEG_INFINITY), DEFAULT_DURATION);
    }

    #[test]
    fn sane_length_repairs_bad_values() {
        assert_eq!(sane_length(100.0), 100.0);
        assert_eq!(sane_length(-5.0), 0.0);
        assert_eq!(sane_length(f32::NAN), 0.0);
        assert_eq!(sane_length(f32::INFINITY), 0.0);
    }

    #[test]
    fn sane_max_height_handles_every_shape() {
        assert_eq!(sane_max_height(None), None);
        assert_eq!(sane_max_height(Some(120.0)), Some(120.0));
        assert_eq!(sane_max_height(Some(0.0)), Some(0.0));
        assert_eq!(sane_max_height(Some(-10.0)), Some(0.0));
        assert_eq!(sane_max_height(Some(f32::NAN)), None);
        assert_eq!(sane_max_height(Some(f32::INFINITY)), None);
    }

    #[test]
    fn placed_width_sanitizes() {
        assert_eq!(placed_width(400.0), 400.0);
        assert_eq!(placed_width(-10.0), 0.0);
        assert_eq!(placed_width(f32::NAN), 0.0);
        assert_eq!(placed_width(f32::INFINITY), 0.0);
    }

    // --- animation state --------------------------------------------------

    #[test]
    fn is_animating_expanding() {
        assert!(is_animating(0.0, true));
        assert!(is_animating(0.5, true));
        assert!(!is_animating(1.0, true));
    }

    #[test]
    fn is_animating_collapsing() {
        assert!(is_animating(1.0, false));
        assert!(is_animating(0.5, false));
        assert!(!is_animating(0.0, false));
    }

    #[test]
    fn detect_animation_end_fires_once() {
        assert!(!detect_animation_end(false, true));
        assert!(!detect_animation_end(false, false));
        assert!(detect_animation_end(true, false));
        assert!(!detect_animation_end(true, true));
    }

    #[test]
    fn detect_expanded_change_skips_first_frame() {
        assert_eq!(detect_expanded_change(None, true), None);
        assert_eq!(detect_expanded_change(None, false), None);
    }

    #[test]
    fn detect_expanded_change_reports_only_on_change() {
        assert_eq!(detect_expanded_change(Some(false), true), Some(true));
        assert_eq!(detect_expanded_change(Some(true), false), Some(false));
        assert_eq!(detect_expanded_change(Some(true), true), None);
        assert_eq!(detect_expanded_change(Some(false), false), None);
    }

    // --- geometry ---------------------------------------------------------

    #[test]
    fn visible_height_scales_linearly() {
        assert_eq!(visible_height(100.0, 0.0), 0.0);
        assert_eq!(visible_height(100.0, 0.5), 50.0);
        assert_eq!(visible_height(100.0, 1.0), 100.0);
    }

    #[test]
    fn visible_height_clamps_and_sanitizes() {
        assert_eq!(visible_height(100.0, -0.5), 0.0);
        assert_eq!(visible_height(100.0, 1.5), 100.0);
        assert_eq!(visible_height(f32::NAN, 0.5), 0.0);
        assert_eq!(visible_height(-10.0, 1.0), 0.0);
    }

    #[test]
    fn needs_height_update_uses_epsilon() {
        assert!(needs_height_update(None, 100.0));
        assert!(needs_height_update(Some(100.0), 100.5));
        assert!(needs_height_update(Some(100.0), 99.5));
        assert!(!needs_height_update(Some(100.0), 100.05));
        assert!(!needs_height_update(Some(100.0), 100.0));
    }

    // --- progress thresholds ---------------------------------------------

    #[test]
    fn progress_thresholds() {
        assert!(progress_is_fully_expanded(1.0));
        assert!(progress_is_fully_expanded(1.5));
        assert!(!progress_is_fully_expanded(0.5));
        assert!(!progress_is_fully_expanded(0.0));

        assert!(progress_is_fully_collapsed(0.0));
        assert!(progress_is_fully_collapsed(-0.5));
        assert!(!progress_is_fully_collapsed(0.5));
        assert!(!progress_is_fully_collapsed(1.0));
    }

    // --- builders ---------------------------------------------------------

    #[test]
    fn defaults_match_mudblazor() {
        let c = MudCollapse::new();
        assert!(!c.expanded);
        assert_eq!(c.duration, DEFAULT_DURATION);
        assert!(c.max_height.is_none());
        assert!(c.id_salt.is_none());
        assert!(matches!(c.flavor, StyleFlavor::Bootstrap));
        assert!(c.tag.is_none());
        assert!(c.base.class_names().is_empty());
    }

    #[test]
    fn builders_set_every_field() {
        let c = MudCollapse::new()
            .expanded(true)
            .duration(0.4)
            .max_height(120.0)
            .id_salt("faq-1")
            .flavor(StyleFlavor::Tailwind)
            .classes(["p-2"])
            .tag("faq");
        assert!(c.expanded);
        assert_eq!(c.duration, 0.4);
        assert_eq!(c.max_height, Some(120.0));
        assert!(c.id_salt.is_some());
        assert!(matches!(c.flavor, StyleFlavor::Tailwind));
        assert!(!c.base.class_names().is_empty());
        assert_eq!(c.tag_ref::<&str>(), Some(&"faq"));
        assert!(c.tag_ref::<u32>().is_none());
    }

    #[test]
    fn id_salt_is_stable_across_calls() {
        let a = MudCollapse::new().id_salt("x");
        let b = MudCollapse::new().id_salt("x");
        assert_eq!(a.id_salt, b.id_salt);
        let c = MudCollapse::new().id_salt("y");
        assert_ne!(a.id_salt, c.id_salt);
    }

    #[test]
    fn tag_round_trips_by_type() {
        let c = MudCollapse::new().tag("advanced-options");
        assert_eq!(c.tag_ref::<&str>(), Some(&"advanced-options"));
        assert!(c.tag_ref::<u32>().is_none());
    }
}