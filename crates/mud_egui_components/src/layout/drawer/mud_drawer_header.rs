//! `MudDrawerHeader` — the fixed header at the top of a
//! [`MudDrawer`](super::MudDrawer).
//!
//! Port of `MudDrawerHeader.razor.cs` (MudBlazor 8.13.0). Parameters:
//! `ChildContent`, `Dense` (64 → 48 pt), and the base members.
//!
//! The header reserves its height, centres its content vertically, and paints
//! a 1 pt bottom hairline — matching `.mud-drawer-header`.

use egui::{vec2, Align, InnerResponse, Layout, Rect, Sense, Stroke, Ui, UiBuilder};
use mud_egui_core::base::component_base::ComponentBase;

/// Height of a normal drawer header, in points.
pub const DRAWER_HEADER_HEIGHT: f32 = 64.0;

/// Height of a dense drawer header, in points.
pub const DENSE_DRAWER_HEADER_HEIGHT: f32 = 48.0;

/// Horizontal padding, in points.
pub const DRAWER_HEADER_PADDING: f32 = 16.0;

/// The fixed header at the top of a drawer.
#[must_use = "MudDrawerHeader does nothing until you call `.show(ui, ..)`"]
pub struct MudDrawerHeader {
    /// Compact height (48 instead of 64).
    pub dense: bool,
    /// Shared component state. Stored for parity.
    pub base: ComponentBase,
}

impl Default for MudDrawerHeader {
    fn default() -> Self {
        Self { dense: false, base: ComponentBase::new() }
    }
}

impl MudDrawerHeader {
    /// Creates a drawer header with MudBlazor's default settings.
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn dense(mut self, b: bool) -> Self {
        self.dense = b;
        self
    }

    #[inline]
    pub fn classes<S: AsRef<str>>(mut self, classes: impl IntoIterator<Item = S>) -> Self {
        self.base = self.base.classes(classes);
        self
    }

    /// Draws the header and returns the content closure's value.
    pub fn show<R>(self, ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        let height = if self.dense {
            DENSE_DRAWER_HEADER_HEIGHT
        } else {
            DRAWER_HEADER_HEIGHT
        };
        let width = ui.available_width();
        let (rect, response) = ui.allocate_exact_size(vec2(width, height), Sense::hover());

        // Bottom hairline.
        let stroke_color = ui.visuals().widgets.noninteractive.bg_stroke.color;
        ui.painter().line_segment(
            [
                egui::pos2(rect.left(), rect.bottom() - 0.5),
                egui::pos2(rect.right(), rect.bottom() - 0.5),
            ],
            Stroke::new(1.0, stroke_color),
        );

        let inner_rect = Rect::from_min_max(
            rect.min + vec2(DRAWER_HEADER_PADDING, 0.0),
            rect.max - vec2(DRAWER_HEADER_PADDING, 0.0),
        );
        let mut child = ui.new_child(
            UiBuilder::new()
                .max_rect(inner_rect)
                .layout(Layout::left_to_right(Align::Center)),
        );
        let inner = content(&mut child);
        let _ = &self.base;
        InnerResponse { inner, response }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_not_dense() {
        assert!(!MudDrawerHeader::new().dense);
    }

    #[test]
    fn dense_setter_stores() {
        assert!(MudDrawerHeader::new().dense(true).dense);
    }

    #[test]
    fn heights_match_mudblazor() {
        assert_eq!(DRAWER_HEADER_HEIGHT, 64.0);
        assert_eq!(DENSE_DRAWER_HEADER_HEIGHT, 48.0);
    }
}