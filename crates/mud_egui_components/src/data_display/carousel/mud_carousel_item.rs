//! `MudCarouselItem` — one slide of a [`MudCarousel`](super::mud_carousel::MudCarousel):
//! arbitrary content with its own transition, colour, and visibility.
//!
//! Port of `MudCarouselItem.razor.cs` (MudBlazor 8.13.0).
//!
//! # MudBlazor parity
//!
//! | MudBlazor parameter | Rust method | Default |
//! |---------------------|-------------|---------|
//! | `ChildContent`      | the content closure of [`MudCarousel::item`](super::mud_carousel::CarouselCtx::item) | required |
//! | `Color`             | [`MudCarouselItem::color`] | [`Color::Default`] |
//! | `CustomTransitionEnter` | [`MudCarouselItem::custom_transition_enter`] | `None` |
//! | `CustomTransitionExit`  | [`MudCarouselItem::custom_transition_exit`] | `None` |
//! | `RightToLeft` (cascading) | [`MudCarouselItem::right_to_left`] | `false` |
//! | `Transition`        | [`MudCarouselItem::transition`] | [`Transition::Slide`] |
//!
//! `Visible` is a **get-only** property in MudBlazor (always `true` for live
//! items). The Rust port exposes it as a builder extension so callers can hide
//! a slide from rendering, navigation, and bullets. See
//! [`MudCarousel::show`](super::mud_carousel::MudCarousel::show).

use egui::Color32;
use mud_egui_core::enums::{Color, Transition};

/// One slide of a carousel, wrapping arbitrary content.
///
/// The carousel only renders the *selected* item, so this is really just a
/// data container that carries per-slide configuration (colour role,
/// transition, visibility, RTL). The actual painting happens inside
/// [`MudCarousel`](super::mud_carousel::MudCarousel).
#[must_use = "MudCarouselItem does nothing until you add it to a MudCarousel"]
pub struct MudCarouselItem {
    /// Theme colour role painted behind the slide. `Default` / `Inherit` paint
    /// nothing (the carousel falls back to `faint_bg_color`).
    pub color: Color,
    /// Custom enter transition (stored for future animation support; not yet
    /// consumed at paint time).
    pub custom_transition_enter: Option<String>,
    /// Custom exit transition (stored for future animation support).
    pub custom_transition_exit: Option<String>,
    /// Effect used when navigating *to* this item. Mirrors `Transition`.
    pub transition: Transition,
    /// Right-to-left content flow. Mirrors cascading `RightToLeft`.
    pub right_to_left: bool,
    /// When `false`, the carousel skips this item during rendering,
    /// navigation, and bullets. Extension: MudBlazor's `Visible` is get-only.
    pub visible: bool,
}

impl Default for MudCarouselItem {
    fn default() -> Self {
        Self {
            color: Color::Default,
            custom_transition_enter: None,
            custom_transition_exit: None,
            transition: Transition::Slide,
            right_to_left: false,
            visible: true,
        }
    }
}

impl MudCarouselItem {
    /// Creates an item with MudBlazor defaults.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the theme colour role painted behind this slide. Mirrors `Color`
    /// (default [`Color::Default`]).
    #[inline]
    pub fn color(mut self, c: Color) -> Self {
        self.color = c;
        self
    }

    /// Sets the enter transition (stored; not yet applied at paint time).
    /// Mirrors `CustomTransitionEnter`.
    #[inline]
    pub fn custom_transition_enter(mut self, s: impl Into<String>) -> Self {
        self.custom_transition_enter = Some(s.into());
        self
    }

    /// Sets the exit transition (stored; not yet applied at paint time).
    /// Mirrors `CustomTransitionExit`.
    #[inline]
    pub fn custom_transition_exit(mut self, s: impl Into<String>) -> Self {
        self.custom_transition_exit = Some(s.into());
        self
    }

    /// Sets the effect used when navigating to this item. Mirrors
    /// `Transition` (default [`Transition::Slide`]).
    #[inline]
    pub fn transition(mut self, t: Transition) -> Self {
        self.transition = t;
        self
    }

    /// Sets right-to-left content flow. Mirrors cascading `RightToLeft`.
    #[inline]
    pub fn right_to_left(mut self, b: bool) -> Self {
        self.right_to_left = b;
        self
    }

    /// Sets whether this slide takes part in rendering, navigation, and
    /// bullets. Extension: MudBlazor's `Visible` is get-only.
    #[inline]
    pub fn visible(mut self, b: bool) -> Self {
        self.visible = b;
        self
    }

    /// Resolves the slide's background colour, if any. Returns `None` for
    /// `Default` / `Inherit`, which paint nothing.
    pub fn resolve_background(
        &self,
        palette: &mud_egui_theming::models::palettes::palette::Palette,
    ) -> Option<Color32> {
        match self.color {
            Color::Default | Color::Inherit => None,
            other => other.resolve(palette).map(Color32::from),
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use mud_egui_theming::theme::MudTheme;

    #[test]
    fn defaults_match_mudblazor() {
        let it = MudCarouselItem::new();
        assert!(matches!(it.color, Color::Default));
        assert!(matches!(it.transition, Transition::Slide));
        assert!(it.visible);
        assert!(!it.right_to_left);
        assert!(it.custom_transition_enter.is_none());
        assert!(it.custom_transition_exit.is_none());
    }

    #[test]
    fn builders_set_every_field() {
        let it = MudCarouselItem::new()
            .color(Color::Secondary)
            .custom_transition_enter("fade-in")
            .custom_transition_exit("fade-out")
            .transition(Transition::Fade)
            .right_to_left(true)
            .visible(false);
        assert!(matches!(it.color, Color::Secondary));
        assert_eq!(it.custom_transition_enter.as_deref(), Some("fade-in"));
        assert_eq!(it.custom_transition_exit.as_deref(), Some("fade-out"));
        assert!(matches!(it.transition, Transition::Fade));
        assert!(it.right_to_left);
        assert!(!it.visible);
    }

    #[test]
    fn default_color_resolves_to_none() {
        let theme = MudTheme::default();
        let palette = theme.current_palette();
        let it = MudCarouselItem::new();
        assert!(it.resolve_background(palette).is_none());
    }

    #[test]
    fn inherit_color_resolves_to_none() {
        let theme = MudTheme::default();
        let palette = theme.current_palette();
        let it = MudCarouselItem::new().color(Color::Inherit);
        assert!(it.resolve_background(palette).is_none());
    }

    #[test]
    fn explicit_color_resolves_to_palette() {
        let theme = MudTheme::default();
        let palette = theme.current_palette();
        let it = MudCarouselItem::new().color(Color::Primary);
        let expected: Color32 = palette.primary.into();
        assert_eq!(it.resolve_background(palette), Some(expected));
    }

    #[test]
    fn visible_extension_toggles_rendering() {
        let it = MudCarouselItem::new().visible(false);
        assert!(!it.visible);
    }
}