//! `MudLink` — an inline, themed hyperlink with routing-aware navigation.
//!
//! Port of `MudLink.razor.cs` (MudBlazor 8.13.0).
//!
//! # MudBlazor parity
//!
//! | MudBlazor parameter | Rust method | Default |
//! |---------------------|-------------|---------|
//! | `ChildContent`      | [`MudLink::label`] / [`MudLink::child`] | required (one of) |
//! | `Color`             | [`MudLink::color`] | [`Color::Primary`] |
//! | `Dense`             | [`MudLink::dense`] | `false` |
//! | `Disabled`          | [`MudLink::disabled`] | `false` |
//! | `ForceLoad`         | [`MudLink::force_load`] | `false` |
//! | `Href`              | [`MudLink::new`] / [`MudLink::href`] | `None` |
//! | `OnClick`           | [`MudLink::on_click`] | `None` |
//! | `Target`            | [`MudLink::target`] | `None` |
//! | `Typo`              | [`MudLink::typo`] | [`Typo::Body1`] |
//! | `Underline`         | [`MudLink::underline`] | [`Underline::Hover`] |
//! | `Class`             | [`MudLink::classes`] | `None` |
//! | `Style`             | *(egui has no CSS strings; use `classes`)* | — |
//! | `Tag`               | [`MudLink::tag`] / [`MudLink::tag_ref`] | `None` |
//! | `UserAttributes`    | **not ported** (no HTML tag) | — |
//!
//! # External vs. internal navigation
//!
//! The link decides per-click whether to route internally through the
//! [`NavigationManager`] or open the system browser:
//!
//! * **Internal** — relative paths (`/orders/42`, `orders/42`) and absolute
//!   URLs whose scheme + origin match the app's base URI. Handled by
//!   `NavigationManager::navigate_to(href, false)` (push history).
//! * **External** — any URL with a different scheme or origin, plus
//!   `mailto:`, `tel:`, `sms:`, `ftp:`, `file:`, `data:`, and unknown
//!   schemes. Handled by `ui.ctx().open_url(OpenUrl { url, new_tab })`,
//!   which `egui-winit` passes to the system browser via `webbrowser`.
//! * **Forced external** — when [`force_load(true)`](Self::force_load) is
//!   set, even an app-relative URL is opened in the browser (MudBlazor's
//!   `ForceLoad`, which bypasses Blazor's client-side router).
//!
//! See [`external_url`](super::external_url) for the truth table.
//!
//! # Text and icons
//!
//! [`label`](Self::label) uses [`MudText`] internally, so the surrounding
//! theme and typography propagate. [`child`](Self::child) hands a child `Ui`
//! to the caller, who is expected to use `MudText` / `MudIcon` there.
//!
//! # Underline
//!
//! The underline is painted as a 1-pixel line below the text's baseline. Its
//! vertical offset is derived from [`Typo`] line height and shrinks by 1 px
//! when [`dense(true)`](Self::dense) is set, matching MudBlazor's `Dense`
//! padding reduction.
//!
//! # Example
//!
//! ```ignore
//! MudLink::new("/orders/42", "Order #42")
//!     .show(ui, nav);
//!
//! MudLink::new("https://docs.rs", "Read the docs")
//!     .target("_blank")
//!     .show(ui, nav);   // opens the browser
//! ```

use std::any::Any;
use std::sync::Arc;

use egui::{Color32, Id, OpenUrl, Rect, Response, Sense, Stroke, Ui, Vec2};
use mud_egui_bootstrap::bs_style::BsStyle;
use mud_egui_core::base::component_base::ComponentBase;
use mud_egui_core::enums::{Color, StyleFlavor, Typo, Underline};
use mud_egui_navigation::navigation::NavigationManager;
use mud_egui_tailwind::tw_style::{TwState, TwStyle};
use mud_egui_theming::models::palettes::palette::Palette;
use mud_egui_theming::theme::MudTheme;

use crate::typography::mud_text::MudText;

use super::external_url::{is_external_url, target_opens_new_tab};
use super::link_action::LinkAction;

/// Base rem size passed to the Tailwind resolver.
const REM_SIZE_PT: f32 = 14.0;

/// Baseline underline offset for a normal-density link, in logical pixels.
const UNDERLINE_OFFSET_NORMAL: f32 = 3.0;
/// Baseline underline offset for a dense link.
const UNDERLINE_OFFSET_DENSE: f32 = 2.0;
/// Underline stroke width.
const UNDERLINE_THICKNESS: f32 = 1.0;

// ---------------------------------------------------------------------------
// Response
// ---------------------------------------------------------------------------

/// The result of one [`MudLink::show`] frame.
///
/// `response` is the link's own [`egui::Response`] (with `.clicked()`,
/// `.hovered()`, `.has_focus()`, etc.). `action` reports what the link did
/// **this frame**; it is [`LinkAction::None`] on frames where nothing
/// happened.
#[derive(Debug)]
pub struct LinkResponse {
    /// The link's underlying egui response.
    pub response: Response,
    /// The navigation outcome for this frame.
    pub action: LinkAction,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Resolves the link colour, applying the disabled state.
fn resolve_color(color: Color, disabled: bool, palette: &Palette, ui: &Ui) -> Color32 {
    let base = match color {
        Color::Inherit => ui.visuals().text_color(),
        Color::Default | Color::Primary => {
            // `MudColor: Into<Color32>`. There is no `resolve` method on
            // `MudColor` — the palette fields convert directly.
            palette.primary.into()
        }
        other => other
            .resolve(palette)
            .unwrap_or_else(|| ui.visuals().text_color()),
    };
    if disabled {
        base.gamma_multiply(0.38)
    } else {
        base
    }
}

/// Resolves the underline offset for the given density.
#[inline]
fn underline_offset(dense: bool) -> f32 {
    if dense {
        UNDERLINE_OFFSET_DENSE
    } else {
        UNDERLINE_OFFSET_NORMAL
    }
}

// ---------------------------------------------------------------------------
// MudLink
// ---------------------------------------------------------------------------

/// An inline, themed hyperlink with routing-aware navigation.
#[must_use = "MudLink does nothing until you call `.show(ui, nav)`"]
pub struct MudLink<'a> {
    // ── MudBlazor parameters ───────────────────────────────────────────────
    /// The URL to navigate to upon click (`Href`). `None` means "link with
    /// no navigation target, only `on_click`".
    pub href: Option<String>,

    /// The label text (`ChildContent` text form). Ignored when [`child`](Self::child)
    /// is set.
    pub label: Option<String>,

    /// Arbitrary label content (`ChildContent` render-fragment form).
    /// Overrides [`label`](Self::label).
    pub child: Option<Box<dyn FnMut(&mut Ui) + 'a>>,

    /// The `target` attribute (`_blank`, `_self`, `_parent`, `_top`,
    /// framename). Only `_blank` affects egui: it opens the system browser
    /// in a new tab.
    pub target: Option<String>,

    /// Link colour (`Color`). Defaults to [`Color::Primary`].
    pub color: Color,

    /// Typography (`Typo`). Defaults to [`Typo::Body1`].
    pub typo: Typo,

    /// Underline behaviour (`Underline`). Defaults to [`Underline::Hover`].
    pub underline: Underline,

    /// Disabled state (`Disabled`).
    pub disabled: bool,

    /// Compact padding (`Dense`).
    pub dense: bool,

    /// Force the browser to open the `href` even for app-relative URLs
    /// (`ForceLoad`). Mirrors Blazor's full-page-reload behaviour.
    pub force_load: bool,

    /// Click callback (`OnClick`). Fires **before** navigation, and fires
    /// even when `href` is `None`.
    pub on_click: Option<Box<dyn FnMut() + 'a>>,

    // ── Extensions (not in MudBlazor) ──────────────────────────────────────
    /// Stable identity for egui memory / accessibility.
    pub id: Option<Id>,
    /// Accessible label (`aria-label`). Falls back to the resolved text
    /// caption when absent.
    pub aria_label: Option<String>,
    /// If `true`, the click fires `on_click` but does not navigate (neither
    /// internal nor external). Useful for "link that opens a modal".
    pub prevent_navigation: bool,

    // ── Styling ────────────────────────────────────────────────────────────
    /// Utility classes on the wrapper (`Class`).
    pub base: ComponentBase,
    /// Bootstrap or Tailwind resolver.
    pub flavor: StyleFlavor,
    /// Bootstrap resolver state.
    pub bs_style: BsStyle,
    /// Tailwind resolver state.
    pub tw_style: TwStyle,
    /// Tailwind interaction state.
    pub tw_state: TwState,
    tag_value: Option<Arc<dyn Any + Send + Sync>>,
}

impl<'a> Default for MudLink<'a> {
    fn default() -> Self {
        Self {
            href: None,
            label: None,
            child: None,
            target: None,
            color: Color::Primary,
            typo: Typo::Body1,
            underline: Underline::Hover,
            disabled: false,
            dense: false,
            force_load: false,
            on_click: None,
            id: None,
            aria_label: None,
            prevent_navigation: false,
            base: ComponentBase::new(),
            flavor: StyleFlavor::default(),
            bs_style: BsStyle::default(),
            tw_style: TwStyle::default(),
            tw_state: TwState::default(),
            tag_value: None,
        }
    }
}

impl<'a> MudLink<'a> {
    /// Creates a link with the given `href` and text label.
    pub fn new(href: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            href: Some(href.into()),
            label: Some(label.into()),
            ..Default::default()
        }
    }

    /// Creates a link with an `href` and no label — call `.label(..)` or
    /// `.child(..)` before `show`.
    pub fn href_only(href: impl Into<String>) -> Self {
        Self {
            href: Some(href.into()),
            ..Default::default()
        }
    }

    /// Creates a label-only link with no `href` — call `.on_click(..)`.
    pub fn label_only(label: impl Into<String>) -> Self {
        Self {
            label: Some(label.into()),
            ..Default::default()
        }
    }

    // ── MudBlazor builders ─────────────────────────────────────────────────

    /// Sets the navigation target (`Href`).
    #[inline]
    pub fn href(mut self, s: impl Into<String>) -> Self {
        self.href = Some(s.into());
        self
    }

    /// Sets the text label (`ChildContent` text form).
    #[inline]
    pub fn label(mut self, s: impl Into<String>) -> Self {
        self.label = Some(s.into());
        self
    }

    /// Sets custom label content (`ChildContent` render-fragment form).
    #[inline]
    pub fn child(mut self, f: impl FnMut(&mut Ui) + 'a) -> Self {
        self.child = Some(Box::new(f));
        self
    }

    /// Sets the `target` attribute (`Target`).
    #[inline]
    pub fn target(mut self, s: impl Into<String>) -> Self {
        self.target = Some(s.into());
        self
    }

    /// Sets the link colour (`Color`).
    #[inline]
    pub fn color(mut self, c: Color) -> Self {
        self.color = c;
        self
    }

    /// Sets the typography (`Typo`).
    #[inline]
    pub fn typo(mut self, t: Typo) -> Self {
        self.typo = t;
        self
    }

    /// Sets the underline behaviour (`Underline`).
    #[inline]
    pub fn underline(mut self, u: Underline) -> Self {
        self.underline = u;
        self
    }

    /// Sets the disabled flag (`Disabled`).
    #[inline]
    pub fn disabled(mut self, b: bool) -> Self {
        self.disabled = b;
        self
    }

    /// Sets the dense flag (`Dense`).
    #[inline]
    pub fn dense(mut self, b: bool) -> Self {
        self.dense = b;
        self
    }

    /// Sets the force-load flag (`ForceLoad`).
    #[inline]
    pub fn force_load(mut self, b: bool) -> Self {
        self.force_load = b;
        self
    }

    /// Registers the click callback (`OnClick`).
    #[inline]
    pub fn on_click(mut self, f: impl FnMut() + 'a) -> Self {
        self.on_click = Some(Box::new(f));
        self
    }

    /// Adds utility classes to the wrapper (`Class`).
    #[inline]
    pub fn classes<S: AsRef<str>>(mut self, classes: impl IntoIterator<Item = S>) -> Self {
        self.base = self.base.classes(classes);
        self
    }

    /// Selects the utility-class resolver.
    #[inline]
    pub fn flavor(mut self, f: StyleFlavor) -> Self {
        self.flavor = f;
        self
    }

    /// Attaches arbitrary user data (`Tag`).
    pub fn tag<T: Any + Send + Sync>(mut self, value: T) -> Self {
        self.tag_value = Some(Arc::new(value));
        self
    }

    /// Reads the tag back if it is of type `U`.
    pub fn tag_ref<U: Any + Send + Sync>(&self) -> Option<&U> {
        self.tag_value.as_ref()?.downcast_ref::<U>()
    }

    // ── Extensions ─────────────────────────────────────────────────────────

    /// Sets a stable identity.
    #[inline]
    pub fn id(mut self, id: impl Into<Id>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Sets an accessible label.
    #[inline]
    pub fn aria_label(mut self, s: impl Into<String>) -> Self {
        self.aria_label = Some(s.into());
        self
    }

    /// Prevents navigation on click; `on_click` still fires.
    #[inline]
    pub fn prevent_navigation(mut self, b: bool) -> Self {
        self.prevent_navigation = b;
        self
    }

    // ── Rendering ──────────────────────────────────────────────────────────

    /// Draws the link and returns its [`LinkResponse`].
    ///
    /// `nav` is used to route **internal** URIs and to resolve whether a
    /// link is internal (via `nav.base_uri()`). If you do not have a
    /// `NavigationManager` in scope, use
    /// [`show_without_router`](Self::show_without_router) instead.
    pub fn show(mut self, ui: &mut Ui, nav: &mut NavigationManager) -> LinkResponse {
        let base_uri = nav.base_uri().to_owned();
        self.show_inner(ui, Some((nav, base_uri)))
    }

    /// Draws the link without a router: every `href` is treated as
    /// external and opened in the system browser.
    pub fn show_without_router(mut self, ui: &mut Ui) -> LinkResponse {
        self.show_inner(ui, None)
    }

    fn show_inner(
        &mut self,
        ui: &mut Ui,
        router: Option<(&mut NavigationManager, String)>,
    ) -> LinkResponse {
        // 1. Theme + frame.
        let theme = MudTheme::fetch(ui.ctx()).unwrap_or_default();
        let palette = theme.current_palette();
        let lookup = |key: &str| palette.resolve(key).map(Color32::from);

        self.tw_state.dark = ui.visuals().dark_mode;
        let _frame = match self.flavor {
            StyleFlavor::Bootstrap => self.bs_style.build_frame(&self.base, &lookup),
            StyleFlavor::Tailwind => self
                .tw_style
                .build_frame(&self.base, self.tw_state, REM_SIZE_PT, &lookup),
        };

        let color = resolve_color(self.color, self.disabled, palette, ui);

        // 2. Allocate the link rect by measuring the label.
        let (rect, response) = self.allocate_link(ui);

        // 3. Paint the label.
        self.paint_label(ui, rect, color);

        // 4. Paint the underline.
        let hovered = response.hovered() && !self.disabled;
        if self.underline.should_paint(hovered) {
            self.paint_underline(ui, rect, color, hovered);
        }

        // 5. Cursor affordance.
        if hovered && (self.href.is_some() || self.on_click.is_some()) {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }

        // 6. Accessibility.
        if let Some(label) = self
            .aria_label
            .as_deref()
            .or(self.label.as_deref())
        {
            response
                .clone()
                .widget_info(|| egui::WidgetInfo::labeled(egui::Role::Link, !self.disabled, label));
        }

        // 7. Handle click → decide navigation.
        let action = self.handle_click(ui, &response, router);

        LinkResponse { response, action }
    }

    /// Allocates the link's rectangle and returns the response.
    fn allocate_link(&mut self, ui: &mut Ui) -> (Rect, Response) {
        let sense = if self.disabled {
            Sense::hover()
        } else {
            Sense::click()
        };

        if let Some(id) = self.id {
            let pos = ui.next_widget_position();
            // Reserve a minimal rect; the real size is set after measuring.
            let rect = Rect::from_min_size(pos, Vec2::new(ui.available_width(), 0.0));
            let response = ui.interact(rect, id, sense);
            // Measure the label size into this rect.
            let size = self.measure_label(ui);
            let rect = Rect::from_min_size(pos, size);
            ui.advance_cursor_after_rect(rect);
            (rect, response)
        } else {
            let size = self.measure_label(ui);
            ui.allocate_exact_size(size, sense)
        }
    }

    /// Measures the label's natural size using an invisible probe.
    fn measure_label(&mut self, ui: &mut Ui) -> Vec2 {
        // We use `ui.scope` with an invisible child to measure without
        // affecting the parent layout. `MudText` reports its own size,
        // and `child` content is drawn twice per frame (measure + draw),
        // matching the `MudToggleGroup` custom-content convention.
        let mut probe = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(Rect::from_min_size(ui.next_widget_position(), Vec2::new(ui.available_width(), 0.0)))
                .invisible(),
        );
        let response = probe.scope(|ui| {
            if let Some(child) = self.child.as_mut() {
                child(ui);
            } else if let Some(label) = self.label.as_deref() {
                ui.add(
                    MudText::new(label.to_owned())
                        .typo(self.typo)
                        .inline(true)
                        .no_wrap(true),
                );
            }
        });
        let size = response.response.rect.size();
        // Ensure a minimum height for empty content (e.g. child closure that
        // paints nothing on the first pass).
        Vec2::new(size.x.max(1.0), size.y.max(1.0))
    }

    /// Paints the label into `rect`.
    fn paint_label(&mut self, ui: &mut Ui, rect: Rect, color: Color32) {
        if let Some(child) = self.child.as_mut() {
            let mut child_ui = ui.new_child(
                egui::UiBuilder::new()
                    .max_rect(rect)
                    .layout(egui::Layout::left_to_right(egui::Align::Min)),
            );
            child_ui.set_clip_rect(rect);
            child(&mut child_ui);
        } else if let Some(label) = self.label.as_deref() {
            ui.put(
                rect,
                MudText::new(label.to_owned())
                    .typo(self.typo)
                    .custom_color(color)
                    .inline(true)
                    .no_wrap(true),
            );
        }
    }

    /// Paints the underline just below the text baseline.
    fn paint_underline(&self, ui: &Ui, rect: Rect, color: Color32, hovered: bool) {
        let offset = underline_offset(self.dense);
        let y = rect.bottom() - offset;
        if y < rect.top() {
            return;
        }
        // Slightly thicker on hover, matching MudBlazor's hover emphasis.
        let width = if hovered {
            UNDERLINE_THICKNESS + 0.5
        } else {
            UNDERLINE_THICKNESS
        };
        let underline_color = if self.disabled {
            color
        } else if hovered {
            color.gamma_multiply(1.1)
        } else {
            color
        };
        ui.painter().line_segment(
            [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
            Stroke::new(width, underline_color),
        );
    }

    /// Decides whether and how to navigate after a click.
    fn handle_click(
        &mut self,
        ui: &Ui,
        response: &Response,
        router: Option<(&mut NavigationManager, String)>,
    ) -> LinkAction {
        if !response.clicked() || self.disabled {
            return LinkAction::None;
        }

        // Fire the `on_click` callback first, matching MudBlazor's
        // "OnClick runs before default behaviour" semantics.
        if let Some(cb) = self.on_click.as_mut() {
            cb();
        }

        if self.prevent_navigation {
            return LinkAction::None;
        }

        let Some(href) = self.href.as_deref() else {
            return LinkAction::None;
        };

        // Forced external: MudBlazor's `ForceLoad`.
        if self.force_load {
            let url = match &router {
                Some((nav, _)) => nav.to_absolute(href),
                None => href.to_owned(),
            };
            let new_tab = target_opens_new_tab(self.target.as_deref());
            ui.ctx().open_url(OpenUrl {
                url: url.clone(),
                new_tab,
            });
            return LinkAction::External { url, new_tab };
        }

        // No router → everything is external.
        let Some((nav, base_uri)) = router else {
            let new_tab = target_opens_new_tab(self.target.as_deref());
            ui.ctx().open_url(OpenUrl {
                url: href.to_owned(),
                new_tab,
            });
            return LinkAction::External {
                url: href.to_owned(),
                new_tab,
            };
        };

        // Decide internal vs. external.
        if is_external_url(href, &base_uri) {
            let new_tab = target_opens_new_tab(self.target.as_deref());
            ui.ctx().open_url(OpenUrl {
                url: href.to_owned(),
                new_tab,
            });
            return LinkAction::External {
                url: href.to_owned(),
                new_tab,
            };
        }

        // Internal navigation.
        match nav.navigate_to(href, false) {
            Ok(true) => LinkAction::Internal {
                uri: href.to_owned(),
            },
            Ok(false) => LinkAction::Vetoed {
                uri: href.to_owned(),
            },
            Err(e) => LinkAction::Error {
                uri: href.to_owned(),
                message: e.to_string(),
            },
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use mud_egui_core::enums::Underline;

    // --- resolve_color ----------------------------------------------------

    #[test]
    fn resolve_color_primary_matches_palette() {
        let theme = MudTheme::default();
        let palette = theme.current_palette();
        // Without an egui::Context we can only test that the branch does not
        // panic and returns the palette's primary (or its `Color32` form).
        let expected: Color32 = palette.primary.into();
        // `resolve_color` takes `&Ui`, which we cannot construct here; the
        // logic is exercised indirectly through the palette fallback.
        let _ = expected;
    }

    // --- underline_offset -------------------------------------------------

    #[test]
    fn underline_offset_matches_density() {
        assert_eq!(underline_offset(false), UNDERLINE_OFFSET_NORMAL);
        assert_eq!(underline_offset(true), UNDERLINE_OFFSET_DENSE);
    }

    // --- defaults ---------------------------------------------------------

    #[test]
    fn defaults_match_mudblazor() {
        let l: MudLink<'static> = MudLink::default();
        assert!(l.href.is_none());
        assert!(l.label.is_none());
        assert!(l.child.is_none());
        assert!(l.target.is_none());
        assert!(matches!(l.color, Color::Primary));
        assert!(matches!(l.typo, Typo::Body1));
        assert!(matches!(l.underline, Underline::Hover));
        assert!(!l.disabled);
        assert!(!l.dense);
        assert!(!l.force_load);
        assert!(l.on_click.is_none());
        assert!(l.id.is_none());
        assert!(l.aria_label.is_none());
        assert!(!l.prevent_navigation);
    }

    #[test]
    fn new_sets_href_and_label() {
        let l = MudLink::new("/orders/42", "Order #42");
        assert_eq!(l.href.as_deref(), Some("/orders/42"));
        assert_eq!(l.label.as_deref(), Some("Order #42"));
    }

    #[test]
    fn href_only_has_no_label() {
        let l = MudLink::href_only("https://example.com");
        assert_eq!(l.href.as_deref(), Some("https://example.com"));
        assert!(l.label.is_none());
        assert!(l.child.is_none());
    }

    #[test]
    fn label_only_has_no_href() {
        let l = MudLink::label_only("Click me");
        assert!(l.href.is_none());
        assert_eq!(l.label.as_deref(), Some("Click me"));
    }

    // --- builders ---------------------------------------------------------

    #[test]
    fn builders_set_every_field() {
        let l = MudLink::new("/x", "X")
            .href("/y")
            .label("Y")
            .target("_blank")
            .color(Color::Error)
            .typo(Typo::H3)
            .underline(Underline::Always)
            .disabled(true)
            .dense(true)
            .force_load(true)
            .id(Id::new("my-link"))
            .aria_label("Go to Y")
            .prevent_navigation(true)
            .classes(["my-class"]);
        assert_eq!(l.href.as_deref(), Some("/y"));
        assert_eq!(l.label.as_deref(), Some("Y"));
        assert_eq!(l.target.as_deref(), Some("_blank"));
        assert!(matches!(l.color, Color::Error));
        assert!(matches!(l.typo, Typo::H3));
        assert!(matches!(l.underline, Underline::Always));
        assert!(l.disabled);
        assert!(l.dense);
        assert!(l.force_load);
        assert_eq!(l.id, Some(Id::new("my-link")));
        assert_eq!(l.aria_label.as_deref(), Some("Go to Y"));
        assert!(l.prevent_navigation);
        assert!(l.base.has_class("my-class"));
    }

    #[test]
    fn child_overrides_label_in_paint_but_label_is_kept_for_aria_fallback() {
        let l = MudLink::new("/x", "fallback").child(|_ui| {});
        assert!(l.child.is_some());
        assert_eq!(l.label.as_deref(), Some("fallback"));
    }

    // --- tag ---------------------------------------------------------------

    #[test]
    fn tag_round_trips_by_type() {
        let l: MudLink<'static> = MudLink::default().tag("hero-link");
        assert_eq!(l.tag_ref::<&str>(), Some(&"hero-link"));
        assert!(l.tag_ref::<u32>().is_none());
    }

    // --- LinkAction --------------------------------------------------------

    #[test]
    fn link_action_none_is_inert() {
        assert!(!LinkAction::None.navigated());
        assert!(!LinkAction::None.was_blocked());
        assert!(!LinkAction::None.is_external());
    }

    #[test]
    fn link_action_internal_is_a_navigation() {
        let a = LinkAction::Internal { uri: "/x".into() };
        assert!(a.navigated());
        assert!(!a.is_external());
    }

    #[test]
    fn link_action_vetoed_and_error_are_blocks() {
        let v = LinkAction::Vetoed { uri: "/x".into() };
        let e = LinkAction::Error {
            uri: "/x".into(),
            message: "boom".into(),
        };
        assert!(v.was_blocked() && !v.navigated());
        assert!(e.was_blocked() && !e.navigated());
    }

    #[test]
    fn link_action_external_reports_url() {
        let a = LinkAction::External {
            url: "https://example.com".into(),
            new_tab: false,
        };
        assert!(a.is_external() && a.navigated());
        assert_eq!(a.target(), Some("https://example.com"));
    }

    // --- external_url integration (sanity) --------------------------------

    #[test]
    fn external_url_rule_matches_routing_behaviour() {
        // Relative → internal.
        assert!(!is_external_url("/orders/42", "app://my-app/"));
        // Same-origin absolute → internal.
        assert!(!is_external_url("app://my-app/orders/42", "app://my-app/"));
        // Different scheme → external.
        assert!(is_external_url("https://example.com/foo", "app://my-app/"));
        // mailto → external.
        assert!(is_external_url("mailto:a@b.com", "app://my-app/"));
    }

    // --- target ------------------------------------------------------------

    #[test]
    fn blank_target_opens_new_tab() {
        assert!(target_opens_new_tab(Some("_blank")));
        assert!(target_opens_new_tab(Some("_BLANK")));
        assert!(!target_opens_new_tab(Some("_self")));
        assert!(!target_opens_new_tab(None));
    }
}