//! `MudAppBar` — the top (or bottom) bar of a [`MudLayout`](super::super::MudLayout).
//!
//! Port of `MudAppBar.razor.cs` (MudBlazor 8.13.0). Pass 1.
//!
//! | MudBlazor parameter | Rust field / method |
//! |---------------------|---------------------|
//! | `Bottom` | [`MudAppBar::bottom`] (default `false`) |
//! | `ChildContent` | `content` argument of [`MudAppBar::show`] |
//! | `Color` | [`MudAppBar::color`] (default `Default`) |
//! | `Contextual` | **not in 8.13.0** — 8.14+ addition |
//! | `Dense` | [`MudAppBar::dense`] (default `false`) |
//! | `Elevation` | [`MudAppBar::elevation`] (default `4`) |
//! | `Fixed` | [`MudAppBar::fixed`] (default `true`) — stored |
//! | `Gutters` | [`MudAppBar::gutters`] (default `true`) |
//! | `ToolBarClass` | [`MudAppBar::toolbar_class`] — stored, not resolved |
//! | `WrapContent` | [`MudAppBar::wrap_content`] (default `false`) |
//! | `Class` (base) | [`MudAppBar::classes`] — stored |
//! | `Style` / `Tag` / `UserAttributes` | not ported |
//!
//! # Composition
//!
//! `MudAppBar` paints its own `egui::Frame` (solid accent fill + elevation
//! shadow) and lays the content out in a horizontal row with 16 pt gutters,
//! an 8 pt gap between children, and a 64 / 48 pt min-height. It does **not**
//! delegate to `MudToolBar`, which would paint a second frame.
//!
//! Every visible string is drawn by the caller through [`MudText`]; the app
//! bar overrides the `Ui`'s `override_text_color` so anything the caller draws
//! reads against the accent fill.
//!
//! # Fixed
//!
//! egui has no free-floating "fixed" chrome inside a scrollable `Ui`. The port
//! stores `fixed` and always paints the bar in place; scroll-away behaviour is
//! the caller's decision.

use egui::{
    vec2, Align, Color32, CornerRadius, InnerResponse, Layout, Rect, Sense, Ui, UiBuilder,
};
use mud_egui_core::base::component_base::ComponentBase;
use mud_egui_core::enums::Color;
use mud_egui_theming::models::palettes::palette::Palette;
use mud_egui_theming::theme::MudTheme;

use crate::layout::layout_shell::LayoutState;

/// Height of a normal app bar, in points.
pub const APP_BAR_HEIGHT: f32 = 64.0;

/// Height of a dense app bar, in points.
pub const DENSE_APP_BAR_HEIGHT: f32 = 48.0;

/// Horizontal padding on each side, in points.
pub const APP_BAR_GUTTER: f32 = 16.0;

/// Gap between children, in points.
pub const APP_BAR_GAP: f32 = 8.0;

/// The top (or bottom) bar of a layout shell.
#[must_use = "MudAppBar does nothing until you call `.show(ui, layout, ..)`"]
pub struct MudAppBar {
    pub bottom: bool,
    pub color: Color,
    pub dense: bool,
    pub elevation: u8,
    pub fixed: bool,
    pub gutters: bool,
    pub toolbar_class: Option<String>,
    pub wrap_content: bool,
    pub base: ComponentBase,
}

impl Default for MudAppBar {
    fn default() -> Self {
        Self {
            bottom: false,
            color: Color::Default,
            dense: false,
            elevation: 4,
            fixed: true,
            gutters: true,
            toolbar_class: None,
            wrap_content: false,
            base: ComponentBase::new(),
        }
    }
}

impl MudAppBar {
    /// Creates an app bar with MudBlazor's default settings.
    pub fn new() -> Self {
        Self::default()
    }

    #[inline] pub fn bottom(mut self, b: bool) -> Self { self.bottom = b; self }
    #[inline] pub fn color(mut self, c: Color) -> Self { self.color = c; self }
    #[inline] pub fn dense(mut self, b: bool) -> Self { self.dense = b; self }
    #[inline] pub fn elevation(mut self, e: u8) -> Self { self.elevation = e; self }
    #[inline] pub fn fixed(mut self, b: bool) -> Self { self.fixed = b; self }
    #[inline] pub fn gutters(mut self, b: bool) -> Self { self.gutters = b; self }
    #[inline] pub fn toolbar_class(mut self, s: impl Into<String>) -> Self { self.toolbar_class = Some(s.into()); self }
    #[inline] pub fn wrap_content(mut self, b: bool) -> Self { self.wrap_content = b; self }
    #[inline] pub fn classes<S: AsRef<str>>(mut self, classes: impl IntoIterator<Item = S>) -> Self { self.base = self.base.classes(classes); self }

    /// Draws the bar and publishes its height into `layout`.
    pub fn show<R>(
        self,
        ui: &mut Ui,
        layout: &mut LayoutState,
        content: impl FnOnce(&mut Ui) -> R,
    ) -> InnerResponse<R> {
        let theme = MudTheme::fetch(ui.ctx()).unwrap_or_default();
        let palette = theme.current_palette();
        let (fill, ink) = resolve_surface(self.color, palette);

        let height = if self.dense { DENSE_APP_BAR_HEIGHT } else { APP_BAR_HEIGHT };
        let gutter = if self.gutters { APP_BAR_GUTTER } else { 0.0 };
        let width = ui.available_width();
        let (rect, response) = ui.allocate_exact_size(vec2(width, height), Sense::hover());

        if ui.is_rect_visible(rect) {
            paint_surface(ui, rect, fill, self.elevation, self.bottom);
        }

        let inner_rect = Rect::from_min_max(
            rect.min + vec2(gutter, 0.0),
            rect.max - vec2(gutter, 0.0),
        );
        let mut child = ui.new_child(
            UiBuilder::new()
                .max_rect(inner_rect)
                .layout(if self.wrap_content {
                    Layout::left_to_right(Align::Center).with_main_wrap(true)
                } else {
                    Layout::left_to_right(Align::Center)
                }),
        );
        // The shell zeroes `item_spacing`; the app bar's own gap is 8 pt.
        child.spacing_mut().item_spacing.x = APP_BAR_GAP;
        child.visuals_mut().override_text_color = Some(ink);
        let inner = content(&mut child);

        layout.app_bar_height = height;
        layout.app_bar_at_bottom = self.bottom;

        let _ = (self.fixed, &self.base, &self.toolbar_class);
        InnerResponse { inner, response }
    }
}

/// Resolves the app bar's fill and the ink that reads against it.
fn resolve_surface(color: Color, palette: &Palette) -> (Color32, Color32) {
    let fill: Color32 = match color {
        Color::Default | Color::Inherit => palette.surface.into(),
        other => other
            .resolve(palette)
            .map(Color32::from)
            .unwrap_or_else(|| palette.surface.into()),
    };
    let ink = if is_light(fill) { Color32::BLACK } else { Color32::WHITE };
    (fill, ink)
}

/// `true` when `c`'s RGB channels sum above the library's 384 threshold.
#[inline]
fn is_light(c: Color32) -> bool {
    (c.r() as u32 + c.g() as u32 + c.b() as u32) > 384
}

/// Paints the fill and a soft elevation shadow.
fn paint_surface(ui: &Ui, rect: Rect, fill: Color32, elevation: u8, bottom: bool) {
    let e = elevation.min(24) as f32;
    if e > 0.0 {
        let dy = if bottom { -1.0 } else { 1.0 };
        for (offset, alpha) in [(3.0_f32, 18u8), (2.0, 28), (1.0, 38)] {
            ui.painter().rect_filled(
                rect.translate(vec2(0.0, dy * offset)),
                CornerRadius::ZERO,
                Color32::from_black_alpha(alpha),
            );
        }
    }
    ui.painter().rect_filled(rect, CornerRadius::ZERO, fill);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_mudblazor() {
        let b = MudAppBar::new();
        assert!(!b.bottom);
        assert!(matches!(b.color, Color::Default));
        assert!(!b.dense);
        assert_eq!(b.elevation, 4);
        assert!(b.fixed);
        assert!(b.gutters);
        assert!(b.toolbar_class.is_none());
        assert!(!b.wrap_content);
    }

    #[test]
    fn builders_set_every_field() {
        let b = MudAppBar::new()
            .bottom(true)
            .color(Color::Primary)
            .dense(true)
            .elevation(8)
            .fixed(false)
            .gutters(false)
            .toolbar_class("my-bar")
            .wrap_content(true)
            .classes(["sticky"]);
        assert!(b.bottom);
        assert!(matches!(b.color, Color::Primary));
        assert!(b.dense);
        assert_eq!(b.elevation, 8);
        assert!(!b.fixed && !b.gutters);
        assert_eq!(b.toolbar_class.as_deref(), Some("my-bar"));
        assert!(b.wrap_content);
        assert!(b.base.has_class("sticky"));
    }

    #[test]
    fn height_constants_match_mudblazor() {
        assert_eq!(APP_BAR_HEIGHT, 64.0);
        assert_eq!(DENSE_APP_BAR_HEIGHT, 48.0);
    }

    #[test]
    fn contrast_threshold_is_384() {
        assert!(is_light(Color32::from_rgb(255, 255, 255)));
        assert!(!is_light(Color32::from_rgb(0, 0, 0)));
        assert!(!is_light(Color32::from_rgb(128, 128, 128)));
        assert!(is_light(Color32::from_rgb(129, 128, 128)));
    }
}