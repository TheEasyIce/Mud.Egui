//! `MudDrawer` — a navigation panel docked to an edge of the layout.
//!
//! Port of `MudDrawer.razor.cs` (MudBlazor 8.13.0). Pass 1.
//!
//! | MudBlazor parameter | Rust field / method |
//! |---------------------|---------------------|
//! | `Anchor` | [`MudDrawer::anchor`] (`AnchorOrigin`, default `Start`) |
//! | `Breakpoint` | [`MudDrawer::breakpoint`] (default `Md`) |
//! | `ChildContent` | `content` argument of [`MudDrawer::show`] |
//! | `ClipMode` | [`MudDrawer::clip_mode`] (default `Never`) |
//! | `Color` | [`MudDrawer::color`] (default `Default`) |
//! | `Elevation` | [`MudDrawer::elevation`] (default `1`) |
//! | `Fixed` | [`MudDrawer::fixed`] (default `true`) — stored |
//! | `Height` | [`MudDrawer::height`] (`f32`, default `None`) |
//! | `MiniWidth` | [`MudDrawer::mini_width`] (`f32`, default `None` → 56) |
//! | `Open` | [`DrawerState::open`] |
//! | `OpenChanged` | [`DrawerResponse::opened`] / `closed` |
//! | `OpenMiniOnHover` | [`MudDrawer::open_mini_on_hover`] (default `false`) |
//! | `Overlay` | [`MudDrawer::overlay`] (default `true`) |
//! | `OverlayAutoClose` | [`MudDrawer::overlay_auto_close`] (default `true`) |
//! | `Variant` | [`MudDrawer::variant`] (default `Responsive`) |
//! | `Width` | [`MudDrawer::width`] (`f32`, default `None` → 240) |
//! | `Class` (base) | [`MudDrawer::classes`] — resolved into the content frame |
//! | `Style` / `Tag` / `UserAttributes` | not ported |
//!
//! # Effective variant
//!
//! [`EffectiveVariant`](super::effective_variant::EffectiveVariant) combines
//! `variant` + `breakpoint` + the container width:
//!
//! | Config | Below breakpoint | At / above breakpoint |
//! |--------|------------------|-----------------------|
//! | `Temporary` | Overlay | Overlay |
//! | `Persistent` | Push | Push |
//! | `Responsive` | Overlay | Push |
//! | `Mini` | Overlay | Mini |
//!
//! `Push` and `Mini` reserve their width in the parent `Ui`; `Overlay` paints
//! in a foreground `Area` and does not.
//!
//! # Clip mode
//!
//! | `ClipMode` | Z-order vs app bar | Top offset |
//! |------------|--------------------|------------|
//! | `Never` | drawer above | `0` |
//! | `Docked` | app bar above | `app_bar_height` |
//! | `Always` | app bar above | `app_bar_height` |
//!
//! Whether the drawer *pushes* content is decided by the effective variant,
//! not by `ClipMode`.
//!
//! # `Class` resolution
//!
//! `Class` is a real `MudDrawer` base parameter, so `self.base` is resolved
//! through `BsStyle` (Bootstrap flavour) or `TwStyle` (Tailwind flavour) and
//! the resulting `Frame` wraps the content area. `p-*` therefore pads the
//! drawer, `bg-*` overrides the `color` fill, `rounded-*` rounds the panel —
//! the same behaviour `MudPopover` has. `p-2` is `0.5 rem` = **7 pt** at
//! `REM_SIZE_PT = 14`; `p-1` is `3.5 pt`, `p-0` is `0`.
//!
//! # Row-height correction
//!
//! When the drawer is placed inside `ui.horizontal(..)`, that row's child
//! `Ui` has its `max_rect.height` clamped to `spacing.interact_size.y`
//! (~20 pt), so `ui.available_height()` inside the row is *never* the row's
//! true height. The port therefore sizes a horizontal drawer from
//! [`LayoutState::available_row_height`] (which `MudLayout::show` publishes
//! via `shell_height`) and falls back to `ui.available_height()` when the
//! shell has not run. `ui.allocate_exact_size` grows the row's `min_rect` to
//! match, so the caller's `ui.horizontal(..)` becomes tall enough on its own.

use egui::{
    vec2, Color32, CornerRadius, CursorIcon, Id, Order, Rect, Response, Sense, Ui,
    UiBuilder,
};
use mud_egui_bootstrap::bs_style::BsStyle;
use mud_egui_core::base::component_base::ComponentBase;
use mud_egui_core::enums::{
    AnchorOrigin, Breakpoint, Color, DrawerClipMode, DrawerVariant, StyleFlavor,
};
use mud_egui_tailwind::tw_style::{TwState, TwStyle};
use mud_egui_theming::theme::MudTheme;

use crate::layout::layout_shell::LayoutState;

use super::drawer_response::DrawerResponse;
use super::drawer_state::DrawerState;
use super::effective_variant::{resolve, EffectiveVariant};
use super::side::{resolve_side, side_rect, Side};

/// Default drawer width, in points.
pub const DEFAULT_WIDTH: f32 = 240.0;

/// Default mini-drawer width, in points.
pub const DEFAULT_MINI_WIDTH: f32 = 56.0;

/// Default drawer height for top/bottom anchors, in points.
pub const DEFAULT_HEIGHT: f32 = 200.0;

/// Default mini height for top/bottom anchors, in points.
pub const DEFAULT_MINI_HEIGHT: f32 = 40.0;

/// Alpha of the overlay backdrop.
const BACKDROP_ALPHA: u8 = 96;

/// Base rem size passed to the Tailwind resolver.
const REM_SIZE_PT: f32 = 14.0;

/// A navigation panel docked to an edge of the layout.
#[must_use = "MudDrawer does nothing until you call `.show(ui, state, layout, ..)`"]
pub struct MudDrawer {
    pub anchor: AnchorOrigin,
    pub breakpoint: Breakpoint,
    pub clip_mode: DrawerClipMode,
    pub color: Color,
    pub elevation: u8,
    pub fixed: bool,
    pub height: Option<f32>,
    pub mini_width: Option<f32>,
    pub open_mini_on_hover: bool,
    pub overlay: bool,
    pub overlay_auto_close: bool,
    pub variant: DrawerVariant,
    pub width: Option<f32>,
    pub right_to_left: bool,
    pub base: ComponentBase,
}

impl Default for MudDrawer {
    fn default() -> Self {
        Self {
            anchor: AnchorOrigin::Start,
            breakpoint: Breakpoint::Md,
            clip_mode: DrawerClipMode::Never,
            color: Color::Default,
            elevation: 1,
            fixed: true,
            height: None,
            mini_width: None,
            open_mini_on_hover: false,
            overlay: true,
            overlay_auto_close: true,
            variant: DrawerVariant::Responsive,
            width: None,
            right_to_left: false,
            base: ComponentBase::new(),
        }
    }
}

impl MudDrawer {
    /// Creates a drawer with MudBlazor's default settings.
    pub fn new() -> Self {
        Self::default()
    }

    #[inline] pub fn anchor(mut self, a: AnchorOrigin) -> Self { self.anchor = a; self }
    #[inline] pub fn breakpoint(mut self, b: Breakpoint) -> Self { self.breakpoint = b; self }
    #[inline] pub fn clip_mode(mut self, m: DrawerClipMode) -> Self { self.clip_mode = m; self }
    #[inline] pub fn color(mut self, c: Color) -> Self { self.color = c; self }
    #[inline] pub fn elevation(mut self, e: u8) -> Self { self.elevation = e; self }
    #[inline] pub fn fixed(mut self, b: bool) -> Self { self.fixed = b; self }
    #[inline] pub fn height(mut self, px: f32) -> Self { self.height = Some(px); self }
    #[inline] pub fn mini_width(mut self, px: f32) -> Self { self.mini_width = Some(px); self }
    #[inline] pub fn open_mini_on_hover(mut self, b: bool) -> Self { self.open_mini_on_hover = b; self }
    #[inline] pub fn overlay(mut self, b: bool) -> Self { self.overlay = b; self }
    #[inline] pub fn overlay_auto_close(mut self, b: bool) -> Self { self.overlay_auto_close = b; self }
    #[inline] pub fn variant(mut self, v: DrawerVariant) -> Self { self.variant = v; self }
    #[inline] pub fn width(mut self, px: f32) -> Self { self.width = Some(px); self }
    #[inline] pub fn right_to_left(mut self, b: bool) -> Self { self.right_to_left = b; self }
    #[inline] pub fn classes<S: AsRef<str>>(mut self, classes: impl IntoIterator<Item = S>) -> Self { self.base = self.base.classes(classes); self }

    /// Draws the drawer and publishes its measurement into `layout`.
    pub fn show<R>(
        mut self,
        ui: &mut Ui,
        state: &mut DrawerState,
        layout: &mut LayoutState,
        content: impl FnOnce(&mut Ui) -> R,
    ) -> DrawerResponse {
        let was_open = state.open;
        let screen_width = ui.ctx().input(|i| i.content_rect().width());
        let effective = resolve(self.variant, self.breakpoint, screen_width);

        let side = resolve_side(self.anchor, self.right_to_left);
        let is_horizontal = side.is_horizontal();

        let mini_visible = matches!(effective, EffectiveVariant::Mini);
        let collapsed_mini = mini_visible && !state.open && !state.hover_expanded;
        let open_now = state.open || state.hover_expanded;

        let full_extent = if is_horizontal {
            self.width.unwrap_or(DEFAULT_WIDTH)
        } else {
            self.height.unwrap_or(DEFAULT_HEIGHT)
        };
        let mini_extent = if is_horizontal {
            self.mini_width.unwrap_or(DEFAULT_MINI_WIDTH)
        } else {
            DEFAULT_MINI_HEIGHT
        };

        let visible = match effective {
            EffectiveVariant::Overlay => open_now,
            EffectiveVariant::Push => open_now,
            EffectiveVariant::Mini => true,
        };

        let extent = if !visible {
            0.0
        } else if collapsed_mini {
            mini_extent
        } else {
            full_extent
        };

        // ── Overlay path ───────────────────────────────────────────────────
        if matches!(effective, EffectiveVariant::Overlay) {
            let mut backdrop_clicked = false;
            if state.open && self.overlay && self.overlay_auto_close {
                backdrop_clicked = paint_backdrop(ui, state);
            }
            if backdrop_clicked {
                state.close();
            }
            let mut action = DrawerResponse {
                response: ui.allocate_response(vec2(0.0, 0.0), Sense::hover()),
                opened: false,
                closed: false,
                visible: false,
                reserved_extent: 0.0,
            };
            if state.open {
                let screen = ui.ctx().input(|i| i.content_rect());
                let (rect, response) =
                    self.paint_overlay(ui, screen, side, extent, layout, content);
                layout.any_overlay_open = true;
                action = DrawerResponse {
                    response,
                    opened: !was_open,
                    closed: false,
                    visible: true,
                    reserved_extent: 0.0,
                };
                let _ = rect;
            }
            return finish(action, state, was_open);
        }

        // ── Inline path (Push / Mini) ──────────────────────────────────────
        let mut action = DrawerResponse {
            response: ui.allocate_response(vec2(0.0, 0.0), Sense::hover()),
            opened: false,
            closed: false,
            visible: false,
            reserved_extent: 0.0,
        };

        if !visible {
            return finish(action, state, was_open);
        }

        // Reserve the strip. `ui.horizontal(..)` clamps its child
        // `max_rect.height` to `spacing.interact_size.y` (~20 pt), so
        // `ui.available_height()` inside the row is never the row's true
        // height. When the shell published its height, use
        // `layout.available_row_height()` — that is the drawer's real
        // vertical extent. `allocate_exact_size` grows the row's `min_rect`
        // to match, so the caller's `ui.horizontal(..)` becomes tall enough
        // on its own.
        let (rect, response) = if is_horizontal {
            let h = ui
                .available_height()
                .max(layout.available_row_height());
            ui.allocate_exact_size(vec2(extent, h), Sense::hover())
        } else {
            let w = ui.available_width();
            ui.allocate_exact_size(vec2(w, extent), Sense::hover())
        };

        if mini_visible && self.open_mini_on_hover {
            state.hover_expanded = response.hovered();
        }

        self.paint_inline(ui, rect, side, content);

        match side {
            Side::Start => layout.start_drawer_width = extent,
            Side::End => layout.end_drawer_width = extent,
            Side::Top => layout.top_drawer_height = extent,
            Side::Bottom => layout.bottom_drawer_height = extent,
        }

        action = DrawerResponse {
            response,
            opened: !was_open,
            closed: false,
            visible: true,
            reserved_extent: extent,
        };
        finish(action, state, was_open)
    }

    // ── Painting helpers ───────────────────────────────────────────────────

    /// Resolves `self.base` into an `egui::Frame`. `Class` is a real
    /// `MudDrawer` base parameter; the frame contributes padding (via
    /// `p-*`), fill (via `bg-*`), and corner radius (via `rounded-*`),
    /// exactly like `MudPopover`.
    fn resolve_content_frame(&self, ui: &Ui) -> egui::Frame {
        let theme = MudTheme::fetch(ui.ctx()).unwrap_or_default();
        let palette = theme.current_palette();
        let lookup = |key: &str| palette.resolve(key).map(Color32::from);
        match StyleFlavor::default() {
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
        }
    }

    fn paint_overlay<R>(
        &self,
        ui: &mut Ui,
        screen: Rect,
        side: Side,
        extent: f32,
        layout: &LayoutState,
        content: impl FnOnce(&mut Ui) -> R,
    ) -> (Rect, Response) {
        let top_offset = if self.clip_mode.starts_above_app_bar() {
            0.0
        } else {
            layout.app_bar_height
        };
        let rect = side_rect(screen, side, extent, top_offset);
        let frame = self.resolve_content_frame(ui);
        let area = egui::Area::new(Id::new("mud_drawer_overlay"))
            .order(Order::Foreground)
            .fixed_pos(rect.min)
            .interactable(true);
        let inner = area.show(ui.ctx(), |ui| {
            let (r, resp) = ui.allocate_exact_size(rect.size(), Sense::hover());
            let panel_rect = Rect::from_min_size(rect.min, r.size());
            self.paint_panel(ui, panel_rect);
            let mut child = ui.new_child(
                UiBuilder::new()
                    .max_rect(panel_rect)
                    .layout(egui::Layout::top_down(egui::Align::Min)),
            );
            child.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);
            // `Class`-resolved padding applies on the overlay path too — the
            // user's `p-*` should not vanish on narrow screens.
            frame.show(&mut child, content).inner;
            resp
        });
        (rect, inner.inner)
    }

    fn paint_inline<R>(
        &self,
        ui: &mut Ui,
        rect: Rect,
        side: Side,
        content: impl FnOnce(&mut Ui) -> R,
    ) {
        self.paint_panel(ui, rect);
        let mut child = ui.new_child(
            UiBuilder::new()
                .max_rect(rect)
                .layout(egui::Layout::top_down(egui::Align::Min)),
        );
        // egui applies `item_spacing` *before* the first allocation in a fresh
        // child `Ui`, so leaving inherited spacing in place would push every
        // child (including the header hairline) right by `item_spacing.x`.
        // Zero it here; padding comes from the resolved content frame below.
        child.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);
        // `Class`-resolved padding / fill / radius from `self.base`. Without
        // this, `.classes(["p-2"])` on a `MudDrawer` would be silently
        // ignored — a Rule 1 violation.
        let frame = self.resolve_content_frame(ui);
        frame.show(&mut child, content).inner;
        if ui.rect_contains_pointer(rect) {
            ui.ctx().set_cursor_icon(CursorIcon::Default);
        }
        let _ = side;
    }

    fn paint_panel(&self, ui: &Ui, rect: Rect) {
        let theme = MudTheme::fetch(ui.ctx()).unwrap_or_default();
        let palette = theme.current_palette();
        // Explicit `Color32` annotation: `.into()` is ambiguous because both
        // `MudColor: Into<Color32>` and `Color32: Into<Color32>` (identity)
        // exist.
        let fill: Color32 = match self.color {
            Color::Default | Color::Inherit => palette.surface.into(),
            other => other
                .resolve(palette)
                .map(Color32::from)
                .unwrap_or_else(|| palette.surface.into()),
        };
        let e = self.elevation.min(24) as f32;
        if e > 0.0 {
            for (offset, alpha) in [(3.0_f32, 16u8), (2.0, 24), (1.0, 32)] {
                ui.painter().rect_filled(
                    rect.translate(vec2(offset, 0.0)),
                    CornerRadius::ZERO,
                    Color32::from_black_alpha(alpha),
                );
            }
        }
        ui.painter().rect_filled(rect, CornerRadius::ZERO, fill);
    }
}

// ---------------------------------------------------------------------------
// Free helpers
// ---------------------------------------------------------------------------

fn paint_backdrop(ui: &mut Ui, state: &mut DrawerState) -> bool {
    let screen = ui.ctx().input(|i| i.content_rect());
    let area = egui::Area::new(Id::new("mud_drawer_backdrop"))
        .order(Order::Middle)
        .fixed_pos(screen.min)
        .interactable(true);
    let inner = area.show(ui.ctx(), |ui| {
        let resp = ui.allocate_rect(screen, Sense::click());
        ui.painter().rect_filled(
            screen,
            CornerRadius::ZERO,
            Color32::from_black_alpha(BACKDROP_ALPHA),
        );
        resp
    });
    let clicked = inner.inner.clicked();
    let _ = state;
    clicked
}

fn finish(mut action: DrawerResponse, state: &mut DrawerState, was_open: bool) -> DrawerResponse {
    action.closed = was_open && !state.open;
    action
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_mudblazor() {
        let d = MudDrawer::new();
        assert_eq!(d.anchor, AnchorOrigin::Start);
        assert_eq!(d.breakpoint, Breakpoint::Md);
        assert_eq!(d.clip_mode, DrawerClipMode::Never);
        assert!(matches!(d.color, Color::Default));
        assert_eq!(d.elevation, 1);
        assert!(d.fixed);
        assert!(d.height.is_none());
        assert!(d.mini_width.is_none());
        assert!(!d.open_mini_on_hover);
        assert!(d.overlay);
        assert!(d.overlay_auto_close);
        assert_eq!(d.variant, DrawerVariant::Responsive);
        assert!(d.width.is_none());
        assert!(!d.right_to_left);
        assert!(d.base.class_names().is_empty());
    }

    #[test]
    fn builders_set_every_field() {
        let d = MudDrawer::new()
            .anchor(AnchorOrigin::End)
            .breakpoint(Breakpoint::Lg)
            .clip_mode(DrawerClipMode::Always)
            .color(Color::Secondary)
            .elevation(4)
            .fixed(false)
            .height(120.0)
            .mini_width(48.0)
            .open_mini_on_hover(true)
            .overlay(false)
            .overlay_auto_close(false)
            .variant(DrawerVariant::Mini)
            .width(300.0)
            .right_to_left(true)
            .classes(["p-2"]);
        assert_eq!(d.anchor, AnchorOrigin::End);
        assert_eq!(d.breakpoint, Breakpoint::Lg);
        assert_eq!(d.clip_mode, DrawerClipMode::Always);
        assert!(matches!(d.color, Color::Secondary));
        assert_eq!(d.elevation, 4);
        assert!(!d.fixed);
        assert_eq!(d.height, Some(120.0));
        assert_eq!(d.mini_width, Some(48.0));
        assert!(d.open_mini_on_hover);
        assert!(!d.overlay && !d.overlay_auto_close);
        assert_eq!(d.variant, DrawerVariant::Mini);
        assert_eq!(d.width, Some(300.0));
        assert!(d.right_to_left);
        assert!(d.base.has_class("p-2"));
    }

    #[test]
    fn default_extents_match_mudblazor() {
        assert_eq!(DEFAULT_WIDTH, 240.0);
        assert_eq!(DEFAULT_MINI_WIDTH, 56.0);
        assert_eq!(DEFAULT_HEIGHT, 200.0);
        assert_eq!(DEFAULT_MINI_HEIGHT, 40.0);
    }

    #[test]
    fn class_names_are_stored_and_accumulate() {
        let d = MudDrawer::new()
            .classes(["p-2"])
            .classes(["bg-primary", "rounded-lg"]);
        assert_eq!(d.base.class_names().len(), 3);
    }
}