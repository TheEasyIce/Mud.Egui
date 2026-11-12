//! `MudMainContent` — the page body area of a [`MudLayout`](super::MudLayout).
//!
//! Port of `MudMainContent.razor.cs` (MudBlazor 8.13.0). MudBlazor's
//! `MudMainContent` exposes only `ChildContent`; its job is to occupy the
//! rectangle the app bar and drawers leave free.
//!
//! # `Class` resolution
//!
//! `Class` is a real `MudMainContent` base parameter, so `self.base` is
//! resolved through `BsStyle` (Bootstrap) or `TwStyle` (Tailwind) and applied
//! to the content area. **All** `bg-*` classes work, including
//! `bg-background` — the token-to-key converter turns `background` into
//! `MudPaletteBackground`, exactly the same key `bg-body` aliases to.
//!
//! # Why we paint the fill manually
//!
//! `egui::Frame::show` allocates only the *content's* natural size, not the
//! parent's `max_rect`. If we relied on it for the fill, `bg-background`
//! would paint a thin strip at the top of the main area (the height of
//! `MudDivider` + whatever `body(ui)` draws), leaving the rest of the rect
//! uncovered. The port therefore:
//!
//! 1. Paints `frame.fill` and `frame.stroke` directly at the target `rect`
//!    with `ui.painter()`, and
//! 2. Pads the content child by `frame.inner_margin` so `p-*` still works.
//!
//! This is the same reason `MudDrawer` paints its panel with `rect_filled`
//! rather than delegating to `Frame::show`.
//!
//! # Row-height correction
//!
//! When the widget is placed inside `ui.horizontal(..)`, that row's child
//! `Ui` has its `max_rect.height` clamped to `spacing.interact_size.y`
//! (~20 pt), so `ui.available_rect_before_wrap().height()` is *never* the
//! row's true height. The port therefore sizes itself from
//! [`LayoutState::available_row_height`] (which `MudLayout::show` publishes
//! via `shell_height`) and falls back to the natural available height when
//! the shell has not run.

use egui::{
    pos2, vec2, Align, Color32, CornerRadius, InnerResponse, Layout, Rect, Response,
    Sense, Stroke, StrokeKind, Ui, UiBuilder,
};
use mud_egui_bootstrap::bs_style::BsStyle;
use mud_egui_core::base::component_base::ComponentBase;
use mud_egui_core::enums::StyleFlavor;
use mud_egui_tailwind::tw_style::{TwState, TwStyle};
use mud_egui_theming::theme::MudTheme;

use super::layout_state::LayoutState;

/// Base rem size passed to the Tailwind resolver.
const REM_SIZE_PT: f32 = 14.0;

/// The page body area of a [`MudLayout`](super::MudLayout).
#[must_use = "MudMainContent does nothing until you call `.show(ui, layout, ..)`"]
pub struct MudMainContent {
    /// Shared component state (utility classes). Resolved into the content
    /// fill / stroke / padding.
    pub base: ComponentBase,
    /// Bootstrap vs Tailwind resolver. Not a MudBlazor parameter — mirrors
    /// `MudDrawer` / `MudPopover`'s flavour selector.
    pub flavor: StyleFlavor,
}

impl Default for MudMainContent {
    fn default() -> Self {
        Self { base: ComponentBase::new(), flavor: StyleFlavor::default() }
    }
}

impl MudMainContent {
    /// Creates the main content area.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the classes. Resolved through `flavor` — `bg-background`,
    /// `bg-body`, `bg-primary`, `p-4`, `rounded-lg`, … all work.
    pub fn classes<S: AsRef<str>>(mut self, classes: impl IntoIterator<Item = S>) -> Self {
        self.base = self.base.classes(classes);
        self
    }

    /// Sets the resolver flavour (Bootstrap by default).
    #[inline]
    pub fn flavor(mut self, flavor: StyleFlavor) -> Self {
        self.flavor = flavor;
        self
    }

    /// Fills the remaining area with `content` and reserves it in the parent.
    pub fn show<R>(
        self,
        ui: &mut Ui,
        layout: &LayoutState,
        content: impl FnOnce(&mut Ui) -> R,
    ) -> InnerResponse<R> {
        let avail = ui.available_rect_before_wrap();
        let target_h = avail.height().max(layout.available_row_height());
        let rect = Rect::from_min_size(avail.min, vec2(avail.width(), target_h));

        // Resolve `self.base` into a Frame — this is what makes `bg-*` /
        // `p-*` / `rounded-*` actually contribute.
        let theme = MudTheme::fetch(ui.ctx()).unwrap_or_default();
        let palette = theme.current_palette();
        let lookup = |key: &str| palette.resolve(key).map(Color32::from);
        let frame = match self.flavor {
            StyleFlavor::Bootstrap => {
                let mut bs = BsStyle::default();
                bs.build_frame(&self.base, &lookup)
            }
            StyleFlavor::Tailwind => {
                let mut state = TwState::default();
                state.dark = ui.visuals().dark_mode;
                let mut tw = TwStyle::default();
                tw.build_frame(&self.base, state, REM_SIZE_PT, &lookup)
            }
        };

        // 1. Paint the resolved fill and stroke **at the full target rect**.
        //
        // `Frame::show` allocates only the content's natural size, so relying
        // on it would paint a thin strip at the top instead of filling the
        // main area. We paint manually here, then use the frame's
        // `inner_margin` to pad the content below.
        if ui.is_rect_visible(rect) {
            if frame.fill != Color32::TRANSPARENT {
                ui.painter().rect_filled(rect, frame.corner_radius, frame.fill);
            }
            if frame.stroke != Stroke::NONE {
                ui.painter().rect_stroke(
                    rect,
                    frame.corner_radius,
                    frame.stroke,
                    StrokeKind::Inside,
                );
            }
        }

        // 2. Content child, inset by `frame.inner_margin`.
        let inner_rect = Rect::from_min_max(
            rect.min + vec2(frame.inner_margin.left as f32, frame.inner_margin.top as f32),
            rect.max - vec2(frame.inner_margin.right as f32, frame.inner_margin.bottom as f32),
        );
        let mut child = ui.new_child(
            UiBuilder::new()
                .max_rect(inner_rect)
                .layout(Layout::top_down(Align::Min)),
        );
        // egui applies `item_spacing` *before* the first allocation in a
        // fresh child `Ui`. Zero it so the content starts flush with the
        // drawer's right edge; padding comes from `frame.inner_margin` above.
        child.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);

        let inner = content(&mut child);
        let response: Response = ui.allocate_rect(rect, Sense::hover());
        InnerResponse { inner, response }
    }
}

/// The rectangle that `MudMainContent` would occupy in the **implicit**
/// composition model: `avail` minus the insets in `layout`.
///
/// Pure and public so a caller can preview the geometry without a `Ui`, or
/// implement their own implicit layout. Not used by
/// [`MudMainContent::show`] — see the module docs.
pub fn compute_main_rect(avail: Rect, layout: &LayoutState) -> Rect {
    let left = avail.left() + layout.start_drawer_width;
    let right = avail.right() - layout.end_drawer_width;
    let top = avail.top() + layout.top_inset();
    let bottom = avail.bottom() - layout.bottom_inset();
    Rect::from_min_max(pos2(left.min(right), top.min(bottom)), pos2(right, bottom))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn avail() -> Rect {
        Rect::from_min_size(pos2(0.0, 0.0), egui::vec2(1000.0, 800.0))
    }

    #[test]
    fn default_state_leaves_the_rect_unchanged() {
        let r = compute_main_rect(avail(), &LayoutState::default());
        assert_eq!(r, avail());
    }

    #[test]
    fn top_app_bar_shrinks_the_top() {
        let s = LayoutState { app_bar_height: 64.0, ..Default::default() };
        let r = compute_main_rect(avail(), &s);
        assert_eq!(r.top(), 64.0);
        assert_eq!(r.bottom(), 800.0);
    }

    #[test]
    fn bottom_app_bar_shrinks_the_bottom() {
        let s = LayoutState {
            app_bar_height: 64.0,
            app_bar_at_bottom: true,
            ..Default::default()
        };
        let r = compute_main_rect(avail(), &s);
        assert_eq!(r.top(), 0.0);
        assert_eq!(r.bottom(), 800.0 - 64.0);
    }

    #[test]
    fn drawers_shrink_both_sides() {
        let s = LayoutState {
            start_drawer_width: 240.0,
            end_drawer_width: 180.0,
            ..Default::default()
        };
        let r = compute_main_rect(avail(), &s);
        assert_eq!(r.left(), 240.0);
        assert_eq!(r.right(), 1000.0 - 180.0);
    }

    #[test]
    fn everything_composes() {
        let s = LayoutState {
            app_bar_height: 64.0,
            start_drawer_width: 240.0,
            end_drawer_width: 180.0,
            ..Default::default()
        };
        let r = compute_main_rect(avail(), &s);
        assert_eq!(r.min, pos2(240.0, 64.0));
        assert_eq!(r.max, pos2(820.0, 800.0));
    }

    #[test]
    fn oversized_insets_do_not_invert_the_rect() {
        let s = LayoutState {
            start_drawer_width: 2000.0,
            end_drawer_width: 2000.0,
            app_bar_height: 2000.0,
            ..Default::default()
        };
        let r = compute_main_rect(avail(), &s);
        assert!(r.width() >= 0.0);
        assert!(r.height() >= 0.0);
    }

    #[test]
    fn classes_accumulate_across_calls() {
        let m = MudMainContent::new()
            .classes(["bg-background"])
            .classes(["p-4"]);
        assert_eq!(m.base.class_names().len(), 2);
    }

    // `CornerRadius` and `Stroke` are re-exported for the `frame.stroke`
    // field access above to resolve.
    #[allow(dead_code)]
    fn _type_check() -> (CornerRadius, Stroke) {
        (CornerRadius::ZERO, Stroke::NONE)
    }
}