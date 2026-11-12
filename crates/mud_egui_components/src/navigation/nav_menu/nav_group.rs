//! `NavGroup` — one `MudNavGroup` inside a [`MudNavMenu`](super::MudNavMenu).
//!
//! Port of `MudNavGroup.razor.cs` (MudBlazor 8.13.0).
//!
//! # MudBlazor parity
//!
//! | MudBlazor parameter | Rust method | Default |
//! |---------------------|-------------|---------|
//! | `ChildContent`      | [`NavGroup::link`] / [`NavGroup::group`] | none |
//! | `Disabled`          | [`NavGroup::disabled`] | `false` |
//! | `ExpandIcon`        | [`NavGroup::expand_icon`] | `ARROW_DROP_DOWN` |
//! | `Expanded`          | [`NavGroup::expanded`] (seed) | from `NavMenuState` |
//! | `ExpandedChanged`   | [`NavGroup::on_expanded_changed`] | `None` |
//! | `HideExpandIcon`    | [`NavGroup::hide_expand_icon`] | `false` |
//! | `Icon`              | [`NavGroup::icon`] | `None` |
//! | `IconColor`         | [`NavGroup::icon_color`] | `None` (inherit) |
//! | `MaxHeight`         | [`NavGroup::max_height`] | `None` |
//! | `Ripple`            | [`NavGroup::ripple`] | `true` (stored, not rendered) |
//! | `Title`             | [`NavGroup::new`] / [`NavGroup::title`] | required |
//! | `TitleContent`      | [`NavGroup::title_content`] | `None` |
//!
//! ### Not ported
//!
//! * **`HeaderClass`** — egui has no CSS strings; the group title uses the
//!   theme's typography. Use [`NavGroup::title_content`] for a custom
//!   title renderer.
//! * **`ButtonClassname` / `ButtonTabIndex` / `Classname` /
//!   `ExpandIconClassname` / `IconClassname`** — `protected` members, not
//!   part of the public API.
//!
//! # Expansion state
//!
//! [`NavGroup::expanded`] **seeds** the initial state. After the first frame,
//! the value lives in [`NavMenuState::expanded`](super::NavMenuState) keyed by
//! [`NavGroup::id`]. This matches MudBlazor's `@bind-Expanded` two-way
//! binding: setting `Expanded` initialises, and `ExpandedChanged` fires when
//! the user toggles.
//!
//! Two groups with the same `id` share expansion state. The default `id` is
//! the title, so two groups titled "Settings" will collapse/expand together
//! unless you call [`NavGroup::id`] with distinct keys.

use egui::Ui;
use mud_egui_core::enums::Color;
use mud_egui_icons_material::filled;

use super::nav_entry::NavEntry;
use super::NavLink;

/// One collapsible group inside a nav menu.
#[must_use = "NavGroup does nothing until you add it to a MudNavMenu"]
pub struct NavGroup<'a> {
    // ── Identity ───────────────────────────────────────────────────────────
    /// Stable key used by [`NavMenuState`](super::NavMenuState) for the
    /// expansion state. Defaults to the title. **Extension**: not a
    /// MudBlazor parameter — MudBlazor identifies groups by instance, which
    /// Rust cannot do across frames.
    pub id: String,

    // ── MudBlazor parameters ───────────────────────────────────────────────
    /// Group title (`Title`).
    pub title: Option<String>,

    /// Custom title content (`TitleContent`). Overrides [`title`](Self::title).
    pub title_content: Option<Box<dyn FnMut(&mut Ui) + 'a>>,

    /// Leading icon (`Icon`).
    pub icon: Option<String>,

    /// Icon colour override (`IconColor`).
    pub icon_color: Option<Color>,

    /// Glyph for the expand/collapse chevron (`ExpandIcon`).
    pub expand_icon: Option<String>,

    /// Hides the expand chevron (`HideExpandIcon`).
    pub hide_expand_icon: bool,

    /// Seed for the expansion state (`Expanded`). `None` means "derive from
    /// `NavMenuState`", defaulting to `false` on first render.
    pub expanded: Option<bool>,

    /// Fired when the expansion state changes (`ExpandedChanged`).
    pub on_expanded_changed: Option<Box<dyn FnMut(bool) + 'a>>,

    /// Disables the group header (`Disabled`).
    pub disabled: bool,

    /// Limits the height of the scrollable children area (`MaxHeight`).
    pub max_height: Option<f32>,

    /// Accepted for parity (`Ripple`); not rendered by this port.
    pub ripple: bool,

    // ── Children ───────────────────────────────────────────────────────────
    /// Collected child entries.
    pub(crate) children: Vec<NavEntry<'a>>,
}

impl<'a> Default for NavGroup<'a> {
    fn default() -> Self {
        Self {
            id: String::new(),
            title: None,
            title_content: None,
            icon: None,
            icon_color: None,
            expand_icon: None,
            hide_expand_icon: false,
            expanded: None,
            on_expanded_changed: None,
            disabled: false,
            max_height: None,
            ripple: true,
            children: Vec::new(),
        }
    }
}

impl<'a> NavGroup<'a> {
    /// Creates a group with the given title. The title is also used as the
    /// default `id`.
    pub fn new(title: impl Into<String>) -> Self {
        let title = title.into();
        Self {
            id: title.clone(),
            title: Some(title),
            ..Default::default()
        }
    }

    // ── Builders ───────────────────────────────────────────────────────────

    /// Sets a stable id different from the title.
    #[inline]
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = id.into();
        self
    }

    #[inline]
    pub fn title(mut self, s: impl Into<String>) -> Self {
        self.title = Some(s.into());
        self
    }

    #[inline]
    pub fn title_content(mut self, f: impl FnMut(&mut Ui) + 'a) -> Self {
        self.title_content = Some(Box::new(f));
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
    pub fn expand_icon(mut self, svg: impl Into<String>) -> Self {
        self.expand_icon = Some(svg.into());
        self
    }

    #[inline]
    pub fn hide_expand_icon(mut self, b: bool) -> Self {
        self.hide_expand_icon = b;
        self
    }

    #[inline]
    pub fn expanded(mut self, b: bool) -> Self {
        self.expanded = Some(b);
        self
    }

    #[inline]
    pub fn on_expanded_changed(mut self, f: impl FnMut(bool) + 'a) -> Self {
        self.on_expanded_changed = Some(Box::new(f));
        self
    }

    #[inline]
    pub fn disabled(mut self, b: bool) -> Self {
        self.disabled = b;
        self
    }

    #[inline]
    pub fn max_height(mut self, px: f32) -> Self {
        self.max_height = Some(px.max(0.0));
        self
    }

    #[inline]
    pub fn ripple(mut self, b: bool) -> Self {
        self.ripple = b;
        self
    }

    // ── Children ───────────────────────────────────────────────────────────

    /// Adds one child link.
    #[inline]
    pub fn link(mut self, link: NavLink<'a>) -> Self {
        self.children.push(NavEntry::Link(link));
        self
    }

    /// Adds a text link (`MudNavLink` convenience).
    #[inline]
    pub fn text_link(self, href: impl Into<String>, text: impl Into<String>) -> Self {
        self.link(NavLink::new(href, text))
    }

    /// Adds one child group (nested).
    #[inline]
    pub fn group(mut self, group: NavGroup<'a>) -> Self {
        self.children.push(NavEntry::Group(group));
        self
    }

    /// Adds a nested group with the given title and no children yet.
    #[inline]
    pub fn text_group(self, title: impl Into<String>) -> Self {
        self.group(NavGroup::new(title))
    }

    /// Resolved expand glyph: the explicit `expand_icon`, or
    /// `filled::ARROW_DROP_DOWN`.
    #[inline]
    pub fn resolve_expand_icon(&self) -> &str {
        self.expand_icon
            .as_deref()
            .unwrap_or(filled::ARROW_DROP_DOWN)
    }

    /// Whether the group has any children (used to hide the expand chevron
    /// even when `hide_expand_icon` is `false`).
    #[inline]
    pub fn has_children(&self) -> bool {
        !self.children.is_empty()
    }
}

// =============================================================================
// Convenience trait used only to satisfy `#[derive(Default)]`-friendly
// construction. Not exposed publicly.
// =============================================================================

/// Marker trait re-exported for the `Default` impl of [`NavLink`].
///
/// Not part of the public API; used only to keep `NavLink::default()`
/// building without hard-coding the `NavLinkMatch` default in two places.
pub trait NavLinkMatchDefault {
    const VALUE: mud_egui_core::enums::NavLinkMatch;
}

/// Implements [`NavLinkMatchDefault`] for `()`.
pub struct MudNavLinkMatchDefault;

impl NavLinkMatchDefault for MudNavLinkMatchDefault {
    const VALUE: mud_egui_core::enums::NavLinkMatch =
        mud_egui_core::enums::NavLinkMatch::Prefix;
}