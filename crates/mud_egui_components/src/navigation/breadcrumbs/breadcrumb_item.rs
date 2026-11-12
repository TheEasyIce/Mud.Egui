//! One breadcrumb entry — the Rust equivalent of MudBlazor's
//! `BreadcrumbItem` record.
//!
//! Port of `BreadcrumbItem.cs` (MudBlazor 8.13.0). The MudBlazor type is a
//! `record` with get-only `Text`, `Href`, `Disabled`, and `Icon`
//! properties. The Rust port uses `pub` fields plus builder methods so the
//! caller can construct items fluently.
//!
//! # Extensions
//!
//! `color` is not a MudBlazor parameter. It lets the caller override the
//! resolved link colour per item — useful for a breadcrumb whose last item
//! highlights the current page.

use mud_egui_core::enums::Color;

/// One breadcrumb item.
///
/// # Example
///
/// ```ignore
/// let items = [
///     BreadcrumbItem::link("Home", "/"),
///     BreadcrumbItem::link("Orders", "/orders"),
///     BreadcrumbItem::new("Order #42"), // no href → not clickable
/// ];
/// ```
#[must_use = "BreadcrumbItem does nothing until you pass it to MudBreadcrumbs"]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BreadcrumbItem {
    /// The caption (`Text`).
    pub text: String,

    /// The URL the item navigates to when clicked (`Href`). `None` renders a
    /// non-clickable item.
    pub href: Option<String>,

    /// When `true`, the item ignores clicks and is drawn disabled
    /// (`Disabled`).
    pub disabled: bool,

    /// Optional leading icon (Material SVG path) rendered before the text
    /// (`Icon`).
    pub icon: Option<String>,

    /// Explicit link colour override. When `None`, the item uses the
    /// resolved defaults described in [`MudBreadcrumbs`](super::MudBreadcrumbs).
    ///
    /// **Extension**: not a MudBlazor parameter.
    pub color: Option<Color>,
}

impl BreadcrumbItem {
    /// Creates a text-only item with no `href`, not disabled, no icon.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            ..Default::default()
        }
    }

    /// Creates a clickable item that navigates to `href`.
    pub fn link(text: impl Into<String>, href: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            href: Some(href.into()),
            ..Default::default()
        }
    }

    /// Sets the navigation URL (`Href`).
    #[inline]
    pub fn href(mut self, href: impl Into<String>) -> Self {
        self.href = Some(href.into());
        self
    }

    /// Marks the item as disabled (`Disabled`).
    #[inline]
    pub fn disabled(mut self, b: bool) -> Self {
        self.disabled = b;
        self
    }

    /// Sets the leading icon (`Icon`).
    #[inline]
    pub fn icon(mut self, svg: impl Into<String>) -> Self {
        self.icon = Some(svg.into());
        self
    }

    /// Sets the link colour override (extension).
    #[inline]
    pub fn color(mut self, c: Color) -> Self {
        self.color = Some(c);
        self
    }

    /// Replaces the caption text.
    #[inline]
    pub fn text(mut self, s: impl Into<String>) -> Self {
        self.text = s.into();
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_has_no_href_and_is_not_disabled() {
        let it = BreadcrumbItem::new("Home");
        assert_eq!(it.text, "Home");
        assert!(it.href.is_none());
        assert!(!it.disabled);
        assert!(it.icon.is_none());
        assert!(it.color.is_none());
    }

    #[test]
    fn link_sets_href() {
        let it = BreadcrumbItem::link("Home", "/");
        assert_eq!(it.text, "Home");
        assert_eq!(it.href.as_deref(), Some("/"));
        assert!(!it.disabled);
    }

    #[test]
    fn builders_set_every_field() {
        let it = BreadcrumbItem::link("Orders", "/orders")
            .text("All Orders")
            .href("/orders/all")
            .disabled(true)
            .icon("M0 0")
            .color(Color::Primary);
        assert_eq!(it.text, "All Orders");
        assert_eq!(it.href.as_deref(), Some("/orders/all"));
        assert!(it.disabled);
        assert_eq!(it.icon.as_deref(), Some("M0 0"));
        assert_eq!(it.color, Some(Color::Primary));
    }
}