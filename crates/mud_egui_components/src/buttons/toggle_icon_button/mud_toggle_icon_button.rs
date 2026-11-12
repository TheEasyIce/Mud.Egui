//! An icon button that switches between two looks when clicked.
//!
//! Port of `MudToggleIconButton.cs`. Composes an icon button and an optional
//! label horizontally. The toggle state is hoisted out of the widget (following
//! egui's immediate-mode paradigm) and passed in as a mutable reference.

use egui::{Response, Ui, Widget};

use crate::buttons::button::mud_button::MudButton;
use mud_egui_core::base::component_base::ComponentBase;
use crate::typography::mud_text::MudText;
use mud_egui_core::enums::{Color, Size, Variant};

/// The resolved look of a [`MudToggleIconButton`] for its current toggle state.
///
/// Produced by [`MudToggleIconButton::resolve_visual_state`] and used to render
/// the inner button. Immutable and cheap to construct.
struct ToggleVisualState {
    icon: String,
    color: Color,
    size: Size,
    variant: Variant,
    title: Option<String>,
    icon_color: Option<Color>,
}

/// An icon button that toggles between two visual states.
///
/// ```ignore
/// let mut is_favorite = false;
/// ui.add(
///     MudToggleIconButton::new(&mut is_favorite, filled::FAVORITE_BORDER, filled::FAVORITE)
///         .color(Color::Default)
///         .toggled_color(Color::Error)
///         .title("Add to favorites")
/// );
/// ```
#[must_use = "MudToggleIconButton does nothing until you call `.show(ui)` or `ui.add(..)`"]
pub struct MudToggleIconButton<'a> {
    /// Mutable reference to the toggle state. Flipped on click.
    pub toggled: &'a mut bool,

    /// The icon of the normal state.
    pub icon: String,
    /// The icon of the toggled state. `None` keeps `icon`.
    pub toggled_icon: Option<String>,

    /// The palette role of the normal state.
    pub color: Color,
    /// The palette role of the toggled state. `None` keeps `color`.
    pub toggled_color: Option<Color>,

    /// The size token of the normal state.
    pub size: Size,
    /// The size token of the toggled state. `None` keeps `size`.
    pub toggled_size: Option<Size>,

    /// The surface variant of the normal state.
    pub variant: Variant,
    /// The surface variant of the toggled state. `None` keeps `variant`.
    pub toggled_variant: Option<Variant>,

    /// The tooltip / accessible name of the normal state.
    pub title: Option<String>,
    /// The tooltip of the toggled state. `None` or empty keeps `title`.
    pub toggled_title: Option<String>,

    /// Explicit icon color of the normal state.
    pub icon_color: Option<Color>,
    /// Explicit icon color of the toggled state.
    pub toggled_icon_color: Option<Color>,

    /// If `true`, ignores input and renders faded.
    pub disabled: bool,

    /// Optional text shown beside the button.
    pub label: Option<String>,

    /// Shared component state (utility classes).
    pub base: ComponentBase,
}

impl<'a> MudToggleIconButton<'a> {
    /// Creates a new toggle icon button.
    ///
    /// `toggled` is a mutable reference to the state; the widget flips it on click.
    pub fn new(
        toggled: &'a mut bool,
        icon: impl Into<String>,
        toggled_icon: impl Into<String>,
    ) -> Self {
        Self {
            toggled,
            icon: icon.into(),
            toggled_icon: Some(toggled_icon.into()),
            color: Color::Default,
            toggled_color: None,
            size: Size::Medium,
            toggled_size: None,
            variant: Variant::Text,
            toggled_variant: None,
            title: None,
            toggled_title: None,
            icon_color: None,
            toggled_icon_color: None,
            disabled: false,
            label: None,
            base: ComponentBase::new(),
        }
    }

    // =========================================================================
    // Builder methods
    // =========================================================================

    #[inline]
    pub fn toggled_icon(mut self, icon: impl Into<String>) -> Self {
        self.toggled_icon = Some(icon.into());
        self
    }

    #[inline]
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    #[inline]
    pub fn toggled_color(mut self, color: Color) -> Self {
        self.toggled_color = Some(color);
        self
    }

    #[inline]
    pub fn size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }

    #[inline]
    pub fn toggled_size(mut self, size: Size) -> Self {
        self.toggled_size = Some(size);
        self
    }

    #[inline]
    pub fn variant(mut self, variant: Variant) -> Self {
        self.variant = variant;
        self
    }

    #[inline]
    pub fn toggled_variant(mut self, variant: Variant) -> Self {
        self.toggled_variant = Some(variant);
        self
    }

    #[inline]
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    #[inline]
    pub fn toggled_title(mut self, title: impl Into<String>) -> Self {
        self.toggled_title = Some(title.into());
        self
    }

    #[inline]
    pub fn icon_color(mut self, color: Color) -> Self {
        self.icon_color = Some(color);
        self
    }

    #[inline]
    pub fn toggled_icon_color(mut self, color: Color) -> Self {
        self.toggled_icon_color = Some(color);
        self
    }

    #[inline]
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    #[inline]
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn classes<S: AsRef<str>>(mut self, classes: impl IntoIterator<Item = S>) -> Self {
        self.base = self.base.classes(classes);
        self
    }

    // =========================================================================
    // Internal helpers
    // =========================================================================

    /// Resolves the look for the current state.
    ///
    /// Toggled values override the normal ones when set.
    fn resolve_visual_state(&self) -> ToggleVisualState {
        let is_toggled = *self.toggled;

        let title = if is_toggled && self.toggled_title.as_ref().is_some_and(|t| !t.is_empty()) {
            self.toggled_title.clone()
        } else {
            self.title.clone()
        };

        let icon_color = if is_toggled && self.toggled_icon_color.is_some() {
            self.toggled_icon_color
        } else {
            self.icon_color
        };

        let icon = if is_toggled && self.toggled_icon.as_ref().is_some_and(|i| !i.is_empty()) {
            self.toggled_icon.clone().unwrap()
        } else {
            self.icon.clone()
        };

        ToggleVisualState {
            icon,
            color: if is_toggled { self.toggled_color.unwrap_or(self.color) } else { self.color },
            size: if is_toggled { self.toggled_size.unwrap_or(self.size) } else { self.size },
            variant: if is_toggled { self.toggled_variant.unwrap_or(self.variant) } else { self.variant },
            title,
            icon_color,
        }
    }

    // =========================================================================
    // Rendering
    // =========================================================================

    /// Renders the button and flips `toggled` on click.
    pub fn show(self, ui: &mut Ui) -> Response {
        let state = self.resolve_visual_state();

        // Build the underlying MudButton.
        let mut button = MudButton::new("")
            .start_icon(state.icon)
            .variant(state.variant)
            .color(state.color)
            .size(state.size)
            .disabled(self.disabled);

        // Apply explicit icon color override if set.
        if let Some(ic) = state.icon_color {
            button = button.icon_color(ic);
        }

        button = button.classes(self.base.class_names().iter().map(|s| s.as_str()));

        // Render horizontally (button + optional label).
        let response = ui.horizontal(|ui| {
            // Apply 4px spacing like the C# MudStack.
            ui.spacing_mut().item_spacing.x = 4.0;

            let btn_resp = ui.add(button);

            if let Some(lbl) = &self.label {
                // Vertically center the label with the button.
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    MudText::body1(lbl).ui(ui);
                });
            }

            btn_resp
        }).inner;

        // Handle toggle click.
        if response.clicked() {
            *self.toggled = !*self.toggled;
        }

        // Attach tooltip if a title was provided.
        if let Some(t) = state.title {
            response.on_hover_text(t)
        } else {
            response
        }
    }
}

impl<'a> Widget for MudToggleIconButton<'a> {
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

    #[test]
    fn resolve_visual_state_uses_normal_when_untoggled() {
        let mut toggled = false;
        let btn = MudToggleIconButton::new(&mut toggled, "A", "B")
            .color(Color::Primary)
            .size(Size::Small);

        let state = btn.resolve_visual_state();
        assert_eq!(state.icon, "A");
        assert!(matches!(state.color, Color::Primary));
        assert!(matches!(state.size, Size::Small));
    }

    #[test]
    fn resolve_visual_state_uses_toggled_when_toggled() {
        let mut toggled = true;
        let btn = MudToggleIconButton::new(&mut toggled, "A", "B")
            .color(Color::Primary)
            .toggled_color(Color::Error);

        let state = btn.resolve_visual_state();
        assert_eq!(state.icon, "B");
        assert!(matches!(state.color, Color::Error));
    }

    #[test]
    fn toggled_icon_falls_back_to_normal_if_empty() {
        let mut toggled = true;
        let btn = MudToggleIconButton::new(&mut toggled, "A", "")
            .color(Color::Primary);

        let state = btn.resolve_visual_state();
        assert_eq!(state.icon, "A");
    }
}