//! Data model for one menu entry.
//!
//! Port of `MudMenuItem.razor.cs` and `MudMenuSeparator.razor.cs`
//! (MudBlazor 8.13.0).

use egui::Ui;
use mud_egui_core::enums::Color;

/// One clickable menu item.
#[must_use = "MenuItem does nothing until you add it to a MudMenu"]
pub struct MenuItem<'a> {
    /// Item label (`Text`).
    pub text: Option<String>,
    /// Leading icon (`Icon`).
    pub icon: Option<String>,
    /// Icon colour override (`IconColor`).
    pub icon_color: Option<Color>,
    /// Navigation URL (`Href`).
    pub href: Option<String>,
    /// Browser frame target (`Target`).
    pub target: Option<String>,
    /// Full-page load instead of client-side routing (`ForceLoad`).
    pub force_load: bool,
    /// Disables this item (`Disabled`).
    pub disabled: bool,
    /// Renders arbitrary custom content in place of text + icon
    /// (`ChildContent`).
    pub child: Option<Box<dyn FnMut(&mut Ui) + 'a>>,
    /// Click callback (`OnClick`).
    pub on_click: Option<Box<dyn FnMut() + 'a>>,
    /// Stable id for accessibility and interaction (`Id`).
    pub id: Option<String>,
}

impl<'a> Default for MenuItem<'a> {
    fn default() -> Self {
        Self {
            text: None,
            icon: None,
            icon_color: None,
            href: None,
            target: None,
            force_load: false,
            disabled: false,
            child: None,
            on_click: None,
            id: None,
        }
    }
}

impl<'a> MenuItem<'a> {
    /// Creates an item with the given text.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: Some(text.into()),
            ..Default::default()
        }
    }

    /// Creates an item that navigates to `href`.
    pub fn link(text: impl Into<String>, href: impl Into<String>) -> Self {
        Self {
            text: Some(text.into()),
            href: Some(href.into()),
            ..Default::default()
        }
    }

    #[inline]
    pub fn text(mut self, s: impl Into<String>) -> Self {
        self.text = Some(s.into());
        self
    }

    #[inline]
    pub fn icon(mut self, svg: impl Into<String>) -> Self {
        self.icon = Some(svg.into());
        self
    }

    #[inline]
    pub fn icon_color(mut self, c: Color) -> Self {
        self.icon_color = Some(c);
        self
    }

    #[inline]
    pub fn href(mut self, s: impl Into<String>) -> Self {
        self.href = Some(s.into());
        self
    }

    #[inline]
    pub fn target(mut self, s: impl Into<String>) -> Self {
        self.target = Some(s.into());
        self
    }

    #[inline]
    pub fn force_load(mut self, b: bool) -> Self {
        self.force_load = b;
        self
    }

    #[inline]
    pub fn disabled(mut self, b: bool) -> Self {
        self.disabled = b;
        self
    }

    #[inline]
    pub fn child(mut self, f: impl FnMut(&mut Ui) + 'a) -> Self {
        self.child = Some(Box::new(f));
        self
    }

    #[inline]
    pub fn on_click(mut self, f: impl FnMut() + 'a) -> Self {
        self.on_click = Some(Box::new(f));
        self
    }

    #[inline]
    pub fn id(mut self, s: impl Into<String>) -> Self {
        self.id = Some(s.into());
        self
    }
}

/// A non-clickable divider between menu items.
#[must_use = "MenuSeparator does nothing until you add it to a MudMenu"]
#[derive(Debug, Clone, Default)]
pub struct MenuSeparator;

impl MenuSeparator {
    /// Creates a separator.
    pub fn new() -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_sets_text() {
        let it = MenuItem::new("Open");
        assert_eq!(it.text.as_deref(), Some("Open"));
        assert!(it.href.is_none());
        assert!(!it.disabled);
    }

    #[test]
    fn link_sets_href() {
        let it = MenuItem::link("Docs", "/docs");
        assert_eq!(it.text.as_deref(), Some("Docs"));
        assert_eq!(it.href.as_deref(), Some("/docs"));
    }

    #[test]
    fn builders_set_every_field() {
        let it = MenuItem::new("X")
            .text("Y")
            .icon("M0 0")
            .icon_color(Color::Error)
            .href("/y")
            .target("_blank")
            .force_load(true)
            .disabled(true)
            .child(|_| {})
            .on_click(|| {})
            .id("item-y");
        assert_eq!(it.text.as_deref(), Some("Y"));
        assert_eq!(it.icon.as_deref(), Some("M0 0"));
        assert_eq!(it.icon_color, Some(Color::Error));
        assert_eq!(it.href.as_deref(), Some("/y"));
        assert_eq!(it.target.as_deref(), Some("_blank"));
        assert!(it.force_load && it.disabled);
        assert!(it.child.is_some() && it.on_click.is_some());
        assert_eq!(it.id.as_deref(), Some("item-y"));
    }

    #[test]
    fn separator_constructs() {
        let _ = MenuSeparator::new();
        let _ = MenuSeparator;
    }
}