//! `MudTabPanel` — one panel in a [`MudTabs`](super::MudTabs).
//!
//! Port of `MudTabPanel.razor.cs` (MudBlazor 8.13.0).
//!
//! # MudBlazor parity
//!
//! | MudBlazor parameter | Rust method | Default |
//! |---------------------|-------------|---------|
//! | `BadgeColor`        | [`MudTabPanel::badge_color`] | `Primary` |
//! | `BadgeData`         | [`MudTabPanel::badge_data`] | `None` |
//! | `BadgeDot`          | [`MudTabPanel::badge_dot`] | `false` |
//! | `BadgeIcon`         | [`MudTabPanel::badge_icon`] | `None` |
//! | `ChildContent`      | [`MudTabPanel::content`] | `None` |
//! | `Disabled`          | [`MudTabPanel::disabled`] | `false` |
//! | `ID`                | [`MudTabPanel::id`] | `None` |
//! | `Icon`              | [`MudTabPanel::icon`] | `None` |
//! | `IconColor`         | [`MudTabPanel::icon_color`] | `Inherit` |
//! | `OnClick`           | [`MudTabPanel::on_click`] | `None` |
//! | `ShowCloseIcon`     | [`MudTabPanel::show_close_icon`] | `true` (stored; only `MudDynamicTabs` renders it) |
//! | `TabContent`        | [`MudTabPanel::tab_content`] | `None` |
//! | `TabWrapperContent` | [`MudTabPanel::tab_wrapper_content`] | `None` |
//! | `Text`              | [`MudTabPanel::new`] / [`MudTabPanel::text`] | `None` |
//! | `ToolTip`           | [`MudTabPanel::tooltip`] | `None` |
//! | `Visible`           | [`MudTabPanel::visible`] | `true` |
//!
//! ### Not ported
//!
//! * **`PanelRef`** — an `ElementReference` to the underlying HTML element.
//!   egui has no DOM; the panel's rect is returned in
//!   [`TabsResponse`](super::TabsResponse)'s `panel_rect` if you need it.
//!
//! # Text and icons
//!
//! `text` renders through [`MudText`], `icon` through [`MudIcon`], and the
//! badge through [`MudBadge`]. Custom `tab_content` closures receive a child
//! `Ui` and are expected to use `MudText` / `MudIcon` themselves, matching
//! every other component in the library.
//!
//! # `TabContent` vs. `Text`
//!
//! MudBlazor's `Text` is ignored when `TabContent` is set. The port matches
//! this: if `tab_content` is `Some`, the default text+icon+badge renderer is
//! replaced entirely.
//!
//! # `TabWrapperContent`
//!
//! MudBlazor types this as `RenderFragment<RenderFragment>`. The Rust port
//! models it as `FnMut(&mut Ui, &TabView, &mut dyn FnMut(&mut Ui))` — the
//! third argument is the "render inner" callback. Calling it renders the
//! default header content into the wrapper's `Ui`.

use egui::Ui;
use mud_egui_core::enums::Color;

/// One panel in a tabs component.
#[must_use = "MudTabPanel does nothing until you add it to a MudTabs"]
pub struct MudTabPanel<'a> {
    /// Panel title (`Text`).
    pub text: Option<String>,
    /// Optional icon shown before the title (`Icon`).
    pub icon: Option<String>,
    /// Per-panel icon colour override (`IconColor`).
    pub icon_color: Option<Color>,
    /// Tooltip displayed on hover over the header (`ToolTip`).
    pub tooltip: Option<String>,
    /// Stable id used for activation (`ID`).
    pub id: Option<String>,

    /// Badge text shown on the header (`BadgeData`). Rendered via `MudBadge`.
    pub badge_data: Option<String>,
    /// When `true`, the badge is rendered as a dot (`BadgeDot`).
    pub badge_dot: bool,
    /// Optional icon inside the badge (`BadgeIcon`).
    pub badge_icon: Option<String>,
    /// Badge colour (`BadgeColor`, default `Primary`).
    pub badge_color: Color,

    /// Prevents interaction (`Disabled`).
    pub disabled: bool,
    /// Hides the panel (`Visible`, default `true`).
    pub visible: bool,
    /// Stored for parity (`ShowCloseIcon`). Only `MudDynamicTabs` renders it.
    pub show_close_icon: bool,

    /// Panel body (`ChildContent`). Runs only when the panel is active.
    pub content: Option<Box<dyn FnMut(&mut Ui) + 'a>>,
    /// Custom header content (`TabContent`). Replaces text + icon + badge.
    pub tab_content: Option<Box<dyn FnMut(&mut Ui, &TabView<'_>) + 'a>>,
    /// Header wrapper (`TabWrapperContent`).
    pub tab_wrapper_content:
        Option<Box<dyn FnMut(&mut Ui, &TabView<'_>, &mut dyn FnMut(&mut Ui)) + 'a>>,

    /// Fires when the header is clicked (`OnClick`).
    pub on_click: Option<Box<dyn FnMut() + 'a>>,
}

impl<'a> Default for MudTabPanel<'a> {
    fn default() -> Self {
        Self {
            text: None,
            icon: None,
            icon_color: None,
            tooltip: None,
            id: None,
            badge_data: None,
            badge_dot: false,
            badge_icon: None,
            badge_color: Color::Primary,
            disabled: false,
            visible: true,
            show_close_icon: true,
            content: None,
            tab_content: None,
            tab_wrapper_content: None,
            on_click: None,
        }
    }
}

impl<'a> MudTabPanel<'a> {
    /// Creates a panel with the given text and no content.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: Some(text.into()),
            ..Default::default()
        }
    }

    /// Creates a panel with content and no text.
    pub fn content_only(f: impl FnMut(&mut Ui) + 'a) -> Self {
        Self {
            content: Some(Box::new(f)),
            ..Default::default()
        }
    }

    // ── Builders ───────────────────────────────────────────────────────────

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
    pub fn tooltip(mut self, s: impl Into<String>) -> Self {
        self.tooltip = Some(s.into());
        self
    }

    #[inline]
    pub fn id(mut self, s: impl Into<String>) -> Self {
        self.id = Some(s.into());
        self
    }

    #[inline]
    pub fn badge_data(mut self, v: impl std::fmt::Display) -> Self {
        self.badge_data = Some(format!("{v}"));
        self
    }

    #[inline]
    pub fn badge_dot(mut self, b: bool) -> Self {
        self.badge_dot = b;
        self
    }

    #[inline]
    pub fn badge_icon(mut self, svg: impl Into<String>) -> Self {
        self.badge_icon = Some(svg.into());
        self
    }

    #[inline]
    pub fn badge_color(mut self, c: Color) -> Self {
        self.badge_color = c;
        self
    }

    #[inline]
    pub fn disabled(mut self, b: bool) -> Self {
        self.disabled = b;
        self
    }

    #[inline]
    pub fn visible(mut self, b: bool) -> Self {
        self.visible = b;
        self
    }

    #[inline]
    pub fn show_close_icon(mut self, b: bool) -> Self {
        self.show_close_icon = b;
        self
    }

    #[inline]
    pub fn content(mut self, f: impl FnMut(&mut Ui) + 'a) -> Self {
        self.content = Some(Box::new(f));
        self
    }

    #[inline]
    pub fn tab_content(mut self, f: impl FnMut(&mut Ui, &TabView<'_>) + 'a) -> Self {
        self.tab_content = Some(Box::new(f));
        self
    }

    #[inline]
    pub fn tab_wrapper_content(
        mut self,
        f: impl FnMut(&mut Ui, &TabView<'_>, &mut dyn FnMut(&mut Ui)) + 'a,
    ) -> Self {
        self.tab_wrapper_content = Some(Box::new(f));
        self
    }

    #[inline]
    pub fn on_click(mut self, f: impl FnMut() + 'a) -> Self {
        self.on_click = Some(Box::new(f));
        self
    }

    /// Builds the read-only [`TabView`] for this panel.
    pub(crate) fn view(&self, index: usize, is_active: bool) -> TabView<'_> {
        TabView {
            index,
            text: self.text.as_deref(),
            icon: self.icon.as_deref(),
            icon_color: self.icon_color.unwrap_or(Color::Inherit),
            disabled: self.disabled,
            visible: self.visible,
            is_active,
            has_badge: self.badge_data.is_some()
                || self.badge_dot
                || self.badge_icon.is_some(),
            badge_data: self.badge_data.as_deref(),
            badge_dot: self.badge_dot,
            badge_icon: self.badge_icon.as_deref(),
            badge_color: self.badge_color,
            tooltip: self.tooltip.as_deref(),
        }
    }
}

/// A read-only snapshot of a panel, passed to `tab_content` /
/// `tab_wrapper_content` closures.
#[derive(Debug)]
pub struct TabView<'s> {
    /// 0-based index of the panel.
    pub index: usize,
    /// `MudTabPanel::text`.
    pub text: Option<&'s str>,
    /// `MudTabPanel::icon`.
    pub icon: Option<&'s str>,
    /// Resolved icon colour.
    pub icon_color: Color,
    /// `MudTabPanel::disabled`.
    pub disabled: bool,
    /// `MudTabPanel::visible`.
    pub visible: bool,
    /// Whether this panel is the active one.
    pub is_active: bool,
    /// Whether a badge is configured.
    pub has_badge: bool,
    /// `MudTabPanel::badge_data`.
    pub badge_data: Option<&'s str>,
    /// `MudTabPanel::badge_dot`.
    pub badge_dot: bool,
    /// `MudTabPanel::badge_icon`.
    pub badge_icon: Option<&'s str>,
    /// `MudTabPanel::badge_color`.
    pub badge_color: Color,
    /// `MudTabPanel::tooltip`.
    pub tooltip: Option<&'s str>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_sets_text_only() {
        let p = MudTabPanel::new("Account");
        assert_eq!(p.text.as_deref(), Some("Account"));
        assert!(p.content.is_none());
        assert!(p.tab_content.is_none());
        assert!(p.visible);
        assert!(!p.disabled);
        assert!(matches!(p.badge_color, Color::Primary));
        assert!(p.show_close_icon);
    }

    #[test]
    fn content_only_has_no_text() {
        let p = MudTabPanel::content_only(|_| {});
        assert!(p.text.is_none());
        assert!(p.content.is_some());
    }

    #[test]
    fn builders_set_every_field() {
        let p = MudTabPanel::new("T")
            .text("T2")
            .icon("M0 0")
            .icon_color(Color::Error)
            .tooltip("tip")
            .id("panel-1")
            .badge_data(3)
            .badge_dot(true)
            .badge_icon("B")
            .badge_color(Color::Warning)
            .disabled(true)
            .visible(false)
            .show_close_icon(false)
            .content(|_| {})
            .tab_content(|_, _| {})
            .tab_wrapper_content(|ui, _, inner| inner(ui))
            .on_click(|| {});

        assert_eq!(p.text.as_deref(), Some("T2"));
        assert_eq!(p.icon.as_deref(), Some("M0 0"));
        assert_eq!(p.icon_color, Some(Color::Error));
        assert_eq!(p.tooltip.as_deref(), Some("tip"));
        assert_eq!(p.id.as_deref(), Some("panel-1"));
        assert_eq!(p.badge_data.as_deref(), Some("3"));
        assert!(p.badge_dot);
        assert_eq!(p.badge_icon.as_deref(), Some("B"));
        assert_eq!(p.badge_color, Color::Warning);
        assert!(p.disabled);
        assert!(!p.visible);
        assert!(!p.show_close_icon);
        assert!(p.content.is_some());
        assert!(p.tab_content.is_some());
        assert!(p.tab_wrapper_content.is_some());
        assert!(p.on_click.is_some());
    }

    #[test]
    fn view_reflects_panel_fields() {
        let p = MudTabPanel::new("T")
            .badge_data("3")
            .badge_color(Color::Error);
        let v = p.view(2, true);
        assert_eq!(v.index, 2);
        assert_eq!(v.text, Some("T"));
        assert!(v.is_active);
        assert!(v.has_badge);
        assert_eq!(v.badge_data, Some("3"));
        assert_eq!(v.badge_color, Color::Error);
    }

    #[test]
    fn has_badge_is_false_when_unconfigured() {
        let p = MudTabPanel::new("T");
        assert!(!p.view(0, false).has_badge);
    }
}