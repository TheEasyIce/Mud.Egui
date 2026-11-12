//! `NavLink` — one `MudNavLink` inside a [`MudNavMenu`](super::MudNavMenu).
//!
//! Port of `MudNavLink.razor.cs` (MudBlazor 8.13.0).
//!
//! # MudBlazor parity
//!
//! | MudBlazor parameter | Rust method | Default |
//! |---------------------|-------------|---------|
//! | `ChildContent`      | [`NavLink::text`] / [`NavLink::child`] | required (one of) |
//! | `Disabled`          | [`NavLink::disabled`] | `false` |
//! | `ForceLoad`         | [`NavLink::force_load`] | `false` |
//! | `Href`              | [`NavLink::new`] / [`NavLink::href`] | `None` |
//! | `Icon`              | [`NavLink::icon`] | `None` |
//! | `IconColor`         | [`NavLink::icon_color`] | `None` (inherit) |
//! | `Match`             | [`NavLink::match_mode`] | `NavLinkMatch::Prefix` |
//! | `OnClick`           | [`NavLink::on_click`] | `None` |
//! | `Ripple`            | [`NavLink::ripple`] | `true` (stored, not rendered) |
//! | `Target`            | [`NavLink::target`] | `None` |
//!
//! ### Not ported
//!
//! * **`ActiveClass`** — MudBlazor applies a CSS class when the link is
//!   active. The Rust port uses `MudNavMenu::color` for the active styling
//!   instead. If you need a caller-driven active flag, use
//!   [`NavLink::child`] and read `is_active` in the closure.
//!
//! # `OnClick` semantics
//!
//! Matches MudBlazor: `on_click` fires **only when `href` is `None`**. When
//! `href` is set, clicking navigates and does not fire `on_click`. If you need
//! a side effect on every click, use [`NavLink::child`] with an interactive
//! child widget, or check `MudNavMenu::show`'s return value.

use egui::Ui;
use mud_egui_core::enums::{Color, NavLinkMatch};

/// One link inside a nav menu.
#[must_use = "NavLink does nothing until you add it to a MudNavMenu"]
pub struct NavLink<'a> {
    // ── MudBlazor parameters ───────────────────────────────────────────────
    /// Navigation URL (`Href`). `None` renders a non-clickable item whose
    /// only interaction is `on_click`.
    pub href: Option<String>,

    /// Text label (`ChildContent` text form). Ignored when [`child`](Self::child)
    /// is set.
    pub text: Option<String>,

    /// Custom label content (`ChildContent` render-fragment form).
    ///
    /// Receives the `is_active` flag so the caller can style active rows
    /// without relying on the component's own active detection.
    pub child: Option<Box<dyn FnMut(&mut Ui, bool) + 'a>>,

    /// Leading icon (`Icon`).
    pub icon: Option<String>,

    /// Icon colour override (`IconColor`). `None` uses the inherited colour.
    pub icon_color: Option<Color>,

    /// Disables the link (`Disabled`).
    pub disabled: bool,

    /// Full-page load instead of client-side routing (`ForceLoad`). The port
    /// opens the absolute URL in the system browser.
    pub force_load: bool,

    /// Browser frame target (`Target`). Only `"_blank"` changes behaviour
    /// (new tab).
    pub target: Option<String>,

    /// Active-detection mode (`Match`).
    pub match_mode: NavLinkMatch,

    /// Accepted for parity (`Ripple`); not rendered by this port.
    pub ripple: bool,

    /// Click callback (`OnClick`). Fires **only when `href` is `None`**,
    /// matching MudBlazor.
    pub on_click: Option<Box<dyn FnMut() + 'a>>,
}

impl<'a> Default for NavLink<'a> {
    fn default() -> Self {
        Self {
            href: None,
            text: None,
            child: None,
            icon: None,
            icon_color: None,
            disabled: false,
            force_load: false,
            target: None,
            match_mode: NavLinkMatch::default(),
            ripple: true,
            on_click: None,
        }
    }
}

impl<'a> NavLink<'a> {
    /// Creates a link with `href` and a text label.
    pub fn new(href: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            href: Some(href.into()),
            text: Some(text.into()),
            ..Default::default()
        }
    }

    /// Creates a text-only link with no `href` (fires `on_click`).
    pub fn label(text: impl Into<String>) -> Self {
        Self {
            text: Some(text.into()),
            ..Default::default()
        }
    }

    // ── Builders ───────────────────────────────────────────────────────────

    /// Sets the navigation URL (`Href`).
    #[inline]
    pub fn href(mut self, href: impl Into<String>) -> Self {
        self.href = Some(href.into());
        self
    }

    /// Sets the text label (`ChildContent` text form).
    #[inline]
    pub fn text(mut self, s: impl Into<String>) -> Self {
        self.text = Some(s.into());
        self
    }

    /// Sets custom label content (`ChildContent` render-fragment form).
    #[inline]
    pub fn child(mut self, f: impl FnMut(&mut Ui, bool) + 'a) -> Self {
        self.child = Some(Box::new(f));
        self
    }

    /// Sets the leading icon (`Icon`).
    #[inline]
    pub fn icon(mut self, svg: impl Into<String>) -> Self {
        self.icon = Some(svg.into());
        self
    }

    /// Sets the icon colour override (`IconColor`).
    #[inline]
    pub fn icon_color(mut self, c: Color) -> Self {
        self.icon_color = Some(c);
        self
    }

    /// Disables the link (`Disabled`).
    #[inline]
    pub fn disabled(mut self, b: bool) -> Self {
        self.disabled = b;
        self
    }

    /// Sets the force-load flag (`ForceLoad`).
    #[inline]
    pub fn force_load(mut self, b: bool) -> Self {
        self.force_load = b;
        self
    }

    /// Sets the browser frame target (`Target`).
    #[inline]
    pub fn target(mut self, s: impl Into<String>) -> Self {
        self.target = Some(s.into());
        self
    }

    /// Sets the active-detection mode (`Match`).
    #[inline]
    pub fn match_mode(mut self, m: NavLinkMatch) -> Self {
        self.match_mode = m;
        self
    }

    /// Sets the ripple flag (`Ripple`). Stored for parity; not rendered.
    #[inline]
    pub fn ripple(mut self, b: bool) -> Self {
        self.ripple = b;
        self
    }

    /// Registers the click callback (`OnClick`).
    #[inline]
    pub fn on_click(mut self, f: impl FnMut() + 'a) -> Self {
        self.on_click = Some(Box::new(f));
        self
    }
}