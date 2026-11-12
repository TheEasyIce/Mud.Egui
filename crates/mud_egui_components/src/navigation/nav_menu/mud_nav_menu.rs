//! `MudNavMenu` — a vertical list of navigation links with optional
//! collapsible groups.
//!
//! Port of `MudNavMenu.razor.cs` + `MudNavGroup.razor.cs` +
//! `MudNavLink.razor.cs` (MudBlazor 8.13.0).
//!
//! # MudBlazor parity — `MudNavMenu`
//!
//! | MudBlazor parameter | Rust method | Default |
//! |---------------------|-------------|---------|
//! | `Bordered`          | [`MudNavMenu::bordered`] | `false` |
//! | `ChildContent`      | closure to [`MudNavMenu::show`] | required |
//! | `Color`             | [`MudNavMenu::color`] | `Color::Default` |
//! | `Dense`             | [`MudNavMenu::dense`] | `false` |
//! | `Margin`            | [`MudNavMenu::margin`] | `None` |
//! | `Rounded`           | [`MudNavMenu::rounded`] | `false` |
//! | `Class` (base)      | [`MudNavMenu::classes`] | `None` |
//! | `Style` (base)      | *(use `classes`)* | — |
//! | `Tag` (base)        | **not ported** (no HTML tag) | — |
//! | `UserAttributes`    | **not ported** | — |
//!
//! For `MudNavGroup` and `MudNavLink` parity tables, see the module docs for
//! [`NavGroup`] and [`NavLink`].
//!
//! # Cascade
//!
//! The menu cascades to every link and group:
//!
//! | Menu field | Cascade effect |
//! |------------|----------------|
//! | `color`    | Active-link text / icon / border colour (falls back to `palette.primary`) |
//! | `bordered` | Active link gets a left accent border (2 px) |
//! | `rounded`  | With `bordered`, active link gets rounded corners on its fill |
//! | `dense`    | Row height 32 px instead of 40 px |
//! | `margin`   | Extra vertical gap between top-level entries |
//!
//! # Active detection
//!
//! A link is active when [`is_link_active`] returns `true` for the current
//! path ([`NavigationManager::current_relative`]) under the link's
//! [`NavLinkMatch`]:
//!
//! * `Prefix` (default) — segment-aware prefix match. `/orders` matches
//!   `/orders/42` but not `/orders-history`.
//! * `All` — exact path match, ignoring the current path's query/fragment.
//!
//! External URLs (any href with a scheme) never match.
//!
//! # Navigation
//!
//! Clicking a link with an `href`:
//!
//! * **Internal** (relative path or same-origin URL) →
//!   `nav.navigate_to(href, false)`.
//! * **External** (different scheme, or a known browser scheme like
//!   `mailto:` / `tel:`) → `ui.ctx().open_url(OpenUrl { url, new_tab })`,
//!   with `new_tab = (target == "_blank")`.
//! * **`force_load(true)`** → always opens the absolute URL in the browser.
//!
//! Shared with [`MudLink`](crate::navigation::link::MudLink) and
//! [`MudBreadcrumbs`](crate::navigation::breadcrumbs::MudBreadcrumbs) via
//! `is_external_url`.
//!
//! # Text and icons
//!
//! Every label renders through [`MudText`], every glyph through [`MudIcon`].

use std::collections::HashMap;

use egui::{
    pos2, vec2, Color32, CornerRadius, CursorIcon, Id, OpenUrl, Rect, Response, Sense,
    Stroke, StrokeKind, Ui,
};
use mud_egui_bootstrap::bs_style::BsStyle;
use mud_egui_core::base::component_base::ComponentBase;
use mud_egui_core::enums::{Color, NavLinkMatch, StyleFlavor, Typo};
use mud_egui_navigation::navigation::NavigationManager;
use mud_egui_tailwind::tw_style::{TwState, TwStyle};
use mud_egui_theming::models::palettes::palette::Palette;
use mud_egui_theming::theme::MudTheme;

use crate::data_display::icons::mud_icon::MudIcon;
use crate::data_display::link::is_external_url;
use crate::typography::mud_text::MudText;

use super::nav_entry::NavEntry;
use super::nav_group::NavGroup;
use super::nav_link::NavLink;

// =============================================================================
// Constants
// =============================================================================

/// Base rem size passed to the Tailwind resolver.
const REM_SIZE_PT: f32 = 14.0;

/// Row height for a normal-density link.
const ROW_HEIGHT_NORMAL: f32 = 40.0;

/// Row height for a dense link.
const ROW_HEIGHT_DENSE: f32 = 32.0;

/// Horizontal padding inside a row.
const ROW_PADDING_X: f32 = 12.0;

/// Gap between the leading icon and the label.
const ICON_GAP: f32 = 8.0;

/// Width of the active-link left border (when `bordered`).
const ACTIVE_BORDER_WIDTH: f32 = 2.0;

/// Left indent per nesting level (sub-groups).
const INDENT_PER_DEPTH: f32 = 16.0;

// =============================================================================
// Pure helpers
// =============================================================================

/// Row height for the given density.
#[inline]
pub fn row_height(dense: bool) -> f32 {
    if dense { ROW_HEIGHT_DENSE } else { ROW_HEIGHT_NORMAL }
}

/// Normalises a path for active-comparison: strips the query and fragment,
/// and strips a trailing slash (except for `/` itself).
pub fn normalize_path(path: &str) -> String {
    let path = path.split(['?', '#']).next().unwrap_or(path);
    if path.len() > 1 && path.ends_with('/') {
        path[..path.len() - 1].to_owned()
    } else {
        path.to_owned()
    }
}

/// Returns `true` when `href` should be highlighted for the current path.
///
/// * External URLs (contain `://`, or start with `mailto:` / `tel:` /
///   `sms:` / `ftp:` / `file:` / `data:`) never match.
/// * `Prefix` matches at a segment boundary; `All` matches exactly.
/// * Comparison is ASCII case-insensitive (matching ASP.NET Core's
///   `StringComparison.OrdinalIgnoreCase`).
pub fn is_link_active(href: &str, current_relative: &str, mode: NavLinkMatch) -> bool {
    // External URLs never correspond to the app's current location.
    if href.contains("://") {
        return false;
    }
    for prefix in ["mailto:", "tel:", "sms:", "ftp:", "file:", "data:"] {
        if href.starts_with(prefix) {
            return false;
        }
    }

    let href = normalize_path(href);
    let current = normalize_path(current_relative);

    match mode {
        NavLinkMatch::All => href.eq_ignore_ascii_case(&current),
        NavLinkMatch::Prefix => {
            if href == "/" || href.is_empty() {
                return true;
            }
            if current.len() < href.len() {
                return false;
            }
            if !current[..href.len()].eq_ignore_ascii_case(&href) {
                return false;
            }
            let rest = &current[href.len()..];
            rest.is_empty()
                || rest.starts_with('/')
                || rest.starts_with('?')
                || rest.starts_with('#')
        }
    }
}

/// Resolves the accent colour for active links from the menu's `color`.
#[inline]
pub fn resolve_accent(color: Color, palette: &Palette) -> Color32 {
    match color {
        Color::Default | Color::Inherit => palette.primary.into(),
        other => other.resolve(palette).unwrap_or_else(|| palette.primary.into()),
    }
}

#[inline]
fn with_alpha(c: Color32, a: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), a)
}

/// Whether `target == "_blank"` (case-insensitive).
#[inline]
pub fn target_opens_new_tab(target: Option<&str>) -> bool {
    matches!(target, Some(t) if t.eq_ignore_ascii_case("_blank"))
}

// =============================================================================
// NavMenuState
// =============================================================================

/// Caller-owned state for [`MudNavMenu`].
///
/// Holds the per-group expansion flags, keyed by [`NavGroup::id`]. This is
/// the egui equivalent of MudBlazor's `@bind-Expanded` two-way binding.
///
/// ```ignore
/// let mut state = NavMenuState::new();
///
/// MudNavMenu::new().show(ui, nav, &mut state, |menu| {
///     menu.group(
///         NavGroup::new("Settings")
///             .link(NavLink::new("Profile", "/profile"))
///             .link(NavLink::new("Security", "/security")),
///     );
/// });
/// ```
#[derive(Debug, Clone, Default)]
pub struct NavMenuState {
    /// Group expansion flags, keyed by [`NavGroup::id`].
    pub expanded: HashMap<String, bool>,
}

impl NavMenuState {
    /// Creates empty state (all groups collapsed on first render unless the
    /// group sets `.expanded(true)`).
    pub fn new() -> Self {
        Self::default()
    }

    /// Current expansion state for `id`, or `None` if it has never been
    /// seeded.
    pub fn is_expanded(&self, id: &str) -> Option<bool> {
        self.expanded.get(id).copied()
    }

    /// Seeds `id` to `initial` if it has not been set yet.
    pub fn ensure_expanded(&mut self, id: &str, initial: bool) {
        self.expanded.entry(id.to_owned()).or_insert(initial);
    }

    /// Flips `id`, seeding it to `!current_default` first when absent.
    pub fn toggle_expanded(&mut self, id: &str) -> bool {
        let entry = self.expanded.entry(id.to_owned()).or_insert(false);
        *entry = !*entry;
        *entry
    }

    /// Explicitly sets `id`.
    pub fn set_expanded(&mut self, id: &str, value: bool) {
        self.expanded.insert(id.to_owned(), value);
    }

    /// Removes every expansion entry.
    pub fn clear(&mut self) {
        self.expanded.clear();
    }
}

// =============================================================================
// NavMenuUi
// =============================================================================

/// Collector passed to [`MudNavMenu::show`]'s closure.
pub struct NavMenuUi<'a> {
    pub(crate) entries: Vec<NavEntry<'a>>,
}

impl<'a> NavMenuUi<'a> {
    /// Adds a link.
    pub fn link(&mut self, link: NavLink<'a>) {
        self.entries.push(NavEntry::Link(link));
    }

    /// Adds a text link.
    pub fn text_link(&mut self, href: impl Into<String>, text: impl Into<String>) {
        self.entries.push(NavEntry::Link(NavLink::new(href, text)));
    }

    /// Adds a group.
    pub fn group(&mut self, group: NavGroup<'a>) {
        self.entries.push(NavEntry::Group(group));
    }

    /// Number of collected entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether nothing has been collected.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

// =============================================================================
// MudNavMenu
// =============================================================================

/// A vertical list of navigation links with support for groups.
#[must_use = "MudNavMenu does nothing until you call `.show(ui, nav, state, |menu| ..)`"]
pub struct MudNavMenu {
    // ── MudBlazor parameters ───────────────────────────────────────────────
    /// Draws a left accent border on the active link (`Bordered`).
    pub bordered: bool,

    /// Rounds the active link's fill corners (`Rounded`, `bordered` only).
    pub rounded: bool,

    /// Compact row heights (`Dense`).
    pub dense: bool,

    /// Active-link accent colour (`Color`).
    pub color: Color,

    /// Extra vertical gap between top-level entries (`Margin`).
    pub margin: Option<f32>,

    // ── Utilities ──────────────────────────────────────────────────────────
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
}

impl Default for MudNavMenu {
    fn default() -> Self {
        Self {
            bordered: false,
            rounded: false,
            dense: false,
            color: Color::Default,
            margin: None,
            base: ComponentBase::new(),
            flavor: StyleFlavor::default(),
            bs_style: BsStyle::default(),
            tw_style: TwStyle::default(),
            tw_state: TwState::default(),
        }
    }
}

impl MudNavMenu {
    /// Creates a nav menu with MudBlazor defaults.
    pub fn new() -> Self {
        Self::default()
    }

    // ── Builders ───────────────────────────────────────────────────────────

    /// Draws a left accent border on the active link (`Bordered`).
    #[inline]
    pub fn bordered(mut self, b: bool) -> Self {
        self.bordered = b;
        self
    }

    /// Rounds the active-link fill corners (`Rounded`).
    #[inline]
    pub fn rounded(mut self, b: bool) -> Self {
        self.rounded = b;
        self
    }

    /// Uses compact row heights (`Dense`).
    #[inline]
    pub fn dense(mut self, b: bool) -> Self {
        self.dense = b;
        self
    }

    /// Sets the active-link accent colour (`Color`).
    #[inline]
    pub fn color(mut self, c: Color) -> Self {
        self.color = c;
        self
    }

    /// Sets the extra vertical gap between top-level entries (`Margin`).
    #[inline]
    pub fn margin(mut self, px: f32) -> Self {
        self.margin = Some(px.max(0.0));
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

    // ── Rendering ──────────────────────────────────────────────────────────

    /// Draws the nav menu.
    ///
    /// Returns `(wrapper_response, user_result)`. The user result is whatever
    /// the `add_items` closure returns.
    ///
    /// `state` is mutated in place: group expansion flags persist across
    /// frames. `nav` is used for active detection (read) and navigation
    /// (write).
    pub fn show<'a, R>(
        mut self,
        ui: &mut Ui,
        nav: &mut NavigationManager,
        state: &mut NavMenuState,
        add_items: impl FnOnce(&mut NavMenuUi<'a>) -> R,
    ) -> (Response, R) {
        // 1. Collect entries.
        let mut collector = NavMenuUi {
            entries: Vec::new(),
        };
        let user_result = add_items(&mut collector);
        let mut entries = collector.entries;

        // 2. Theme + wrapper frame.
        let theme = MudTheme::fetch(ui.ctx()).unwrap_or_default();
        let palette = theme.current_palette();
        let lookup = |key: &str| palette.resolve(key).map(Color32::from);

        self.tw_state.dark = ui.visuals().dark_mode;
        let frame = match self.flavor {
            StyleFlavor::Bootstrap => self.bs_style.build_frame(&self.base, &lookup),
            StyleFlavor::Tailwind => self
                .tw_style
                .build_frame(&self.base, self.tw_state, REM_SIZE_PT, &lookup),
        };

        // 3. Resolve config.
        let accent = resolve_accent(self.color, palette);
        let row_h = row_height(self.dense);
        let cfg = NavCfg {
            accent,
            text_primary: palette.text_primary.into(),
            text_secondary: palette.text_secondary.into(),
            text_disabled: palette.text_disabled.into(),
            hover_fill: ui.visuals().widgets.hovered.weak_bg_fill,
            active_fill: with_alpha(accent, 20),
            row_h,
            bordered: self.bordered,
            rounded: self.rounded,
            margin: self.margin,
            palette,
        };

        // 4. Render.
        let scope_id = ui.scope_id();
        let base_id = scope_id.with("mud_nav_menu");

        let inner = frame.show(ui, |ui| {
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 0.0;
                let n = entries.len();
                for (i, entry) in entries.iter_mut().enumerate() {
                    render_entry(ui, nav, state, entry, &cfg, &base_id, 0, i);
                    // Margin between top-level entries.
                    if cfg.margin.is_some() && i + 1 < n {
                        ui.add_space(cfg.margin.unwrap_or(0.0));
                    }
                }
            })
                .response
        });

        (inner.inner, user_result)
    }
}

// =============================================================================
// Render configuration
// =============================================================================

/// Immutable per-frame rendering config.
struct NavCfg<'p> {
    accent: Color32,
    text_primary: Color32,
    text_secondary: Color32,
    text_disabled: Color32,
    hover_fill: Color32,
    active_fill: Color32,
    row_h: f32,
    bordered: bool,
    rounded: bool,
    margin: Option<f32>,
    palette: &'p Palette,
}

// =============================================================================
// Rendering
// =============================================================================

fn render_entry<'a>(
    ui: &mut Ui,
    nav: &mut NavigationManager,
    state: &mut NavMenuState,
    entry: &mut NavEntry<'a>,
    cfg: &NavCfg<'_>,
    base_id: &Id,
    depth: usize,
    index: usize,
) {
    match entry {
        NavEntry::Link(link) => {
            render_link(ui, nav, link, cfg, base_id, depth, index);
        }
        NavEntry::Group(group) => {
            render_group(ui, nav, state, group, cfg, base_id, depth, index);
        }
    }
}

fn render_link<'a>(
    ui: &mut Ui,
    nav: &mut NavigationManager,
    link: &mut NavLink<'a>,
    cfg: &NavCfg<'_>,
    base_id: &Id,
    depth: usize,
    index: usize,
) {
    let is_active = match link.href.as_deref() {
        Some(href) if !link.disabled => {
            is_link_active(href, nav.current_relative(), link.match_mode)
        }
        _ => false,
    };

    let row_w = ui.available_width();
    let indent = depth as f32 * INDENT_PER_DEPTH;
    let sense = if link.disabled {
        Sense::hover()
    } else {
        Sense::click()
    };

    let (rect, response) =
        ui.allocate_exact_size(vec2(row_w, cfg.row_h), sense);

    // Resolve colour for the row's text and icon.
    let text_color = if link.disabled {
        cfg.text_disabled
    } else if is_active {
        cfg.accent
    } else {
        cfg.text_primary
    };

    // Background fill.
    let fill = if link.disabled {
        Color32::TRANSPARENT
    } else if is_active {
        cfg.active_fill
    } else if response.hovered() {
        cfg.hover_fill
    } else {
        Color32::TRANSPARENT
    };
    let radius = if cfg.rounded && cfg.bordered {
        CornerRadius::same(4)
    } else {
        CornerRadius::ZERO
    };
    if fill != Color32::TRANSPARENT {
        ui.painter().rect_filled(rect, radius, fill);
    }

    // Active border.
    if cfg.bordered && is_active && !link.disabled {
        let border_rect = Rect::from_min_size(
            pos2(rect.left(), rect.top()),
            vec2(ACTIVE_BORDER_WIDTH, rect.height()),
        );
        ui.painter().rect_filled(border_rect, CornerRadius::ZERO, cfg.accent);
    }

    // Cursor + focus ring.
    if response.hovered() && !link.disabled {
        ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
    }
    if response.has_focus() && !link.disabled {
        ui.painter().rect_stroke(
            rect,
            radius,
            Stroke::new(2.0, with_alpha(cfg.accent, 100)),
            StrokeKind::Outside,
        );
    }

    // Layout: indent + icon + text.
    let mut cursor_x = rect.left() + indent + ROW_PADDING_X;
    let center_y = rect.center().y;

    if let Some(icon) = link.icon.as_deref() {
        let icon_px = cfg.row_h * 0.5;
        let icon_color = if link.disabled {
            cfg.text_disabled
        } else {
            link.icon_color
                .and_then(|c| c.resolve(cfg.palette).map(Color32::from))
                .unwrap_or(text_color)
        };
        let icon_rect = Rect::from_center_size(
            pos2(cursor_x + icon_px * 0.5, center_y),
            vec2(icon_px, icon_px),
        );
        ui.put(
            icon_rect,
            MudIcon::new(icon.to_owned())
                .size_px(icon_px)
                .custom_color(icon_color),
        );
        cursor_x += icon_px + ICON_GAP;
    }

    // Label.
    let text_rect = Rect::from_min_max(
        pos2(cursor_x, rect.top()),
        pos2(rect.right() - ROW_PADDING_X, rect.bottom()),
    );
    if let Some(child) = link.child.as_mut() {
        let mut child_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(text_rect)
                .layout(egui::Layout::left_to_right(egui::Align::Center)),
        );
        child_ui.set_clip_rect(rect);
        child(&mut child_ui, is_active);
    } else if let Some(text) = link.text.as_deref() {
        let mut text_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(text_rect)
                .layout(egui::Layout::left_to_right(egui::Align::Center)),
        );
        text_ui.set_clip_rect(rect);
        text_ui.add(
            MudText::new(text.to_owned())
                .typo(Typo::Body1)
                .custom_color(text_color),
        );
    }

    // Accessibility.
    let label = link.text.clone().unwrap_or_default();
    let enabled = !link.disabled;
    response.clone().widget_info(|| {
        egui::WidgetInfo::labeled(egui::Role::Link, enabled, label.as_str())
    });

    // Click handling.
    if response.clicked() && !link.disabled {
        if let Some(href) = link.href.clone() {
            handle_navigation(ui, nav, &href, link.force_load, link.target.as_deref());
        } else if let Some(cb) = link.on_click.as_mut() {
            cb();
        }
    }

    let _ = base_id;
    let _ = index;
}

fn render_group<'a>(
    ui: &mut Ui,
    nav: &mut NavigationManager,
    state: &mut NavMenuState,
    group: &mut NavGroup<'a>,
    cfg: &NavCfg<'_>,
    base_id: &Id,
    depth: usize,
    index: usize,
) {
    // 1. Seed expansion state.
    let seed = group.expanded.unwrap_or(false);
    state.ensure_expanded(&group.id, seed);
    let expanded = state.is_expanded(&group.id).unwrap_or(seed);

    // 2. Header row (title + expand chevron).
    let row_w = ui.available_width();
    let indent = depth as f32 * INDENT_PER_DEPTH;
    let sense = if group.disabled {
        Sense::hover()
    } else {
        Sense::click()
    };
    let (header_rect, header_resp) =
        ui.allocate_exact_size(vec2(row_w, cfg.row_h), sense);

    // Background on hover (no active state for groups — matches MudBlazor).
    if header_resp.hovered() && !group.disabled {
        ui.painter().rect_filled(
            header_rect,
            CornerRadius::ZERO,
            cfg.hover_fill,
        );
    }

    // Cursor.
    if header_resp.hovered() && !group.disabled && group.has_children() {
        ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
    }

    // Layout.
    let mut cursor_x = header_rect.left() + indent + ROW_PADDING_X;
    let center_y = header_rect.center().y;

    // Expand chevron.
    if !group.hide_expand_icon && group.has_children() {
        let chevron_px = cfg.row_h * 0.5;
        let chevron_rect = Rect::from_center_size(
            pos2(cursor_x + chevron_px * 0.5, center_y),
            vec2(chevron_px, chevron_px),
        );
        // MudBlazor defaults to `ARROW_DROP_DOWN`; the port rotates it 90°
        // when collapsed to keep the same glyph. Rotation is cheap thanks to
        // MudIcon's transform cache.
        let rotation = if expanded { 0.0 } else { -90.0 };
        let chevron_color = if group.disabled {
            cfg.text_disabled
        } else {
            cfg.text_secondary
        };
        ui.put(
            chevron_rect,
            MudIcon::new(group.resolve_expand_icon().to_owned())
                .size_px(chevron_px)
                .rotation(rotation)
                .custom_color(chevron_color),
        );
        cursor_x += chevron_px + ICON_GAP;
    } else {
        // Reserve the same indent so titles align regardless of chevron.
        if group.has_children() {
            cursor_x += cfg.row_h * 0.5 + ICON_GAP;
        }
    }

    // Leading icon.
    if let Some(icon) = group.icon.as_deref() {
        let icon_px = cfg.row_h * 0.5;
        let icon_color = if group.disabled {
            cfg.text_disabled
        } else {
            group
                .icon_color
                .and_then(|c| c.resolve(cfg.palette).map(Color32::from))
                .unwrap_or(cfg.text_primary)
        };
        let icon_rect = Rect::from_center_size(
            pos2(cursor_x + icon_px * 0.5, center_y),
            vec2(icon_px, icon_px),
        );
        ui.put(
            icon_rect,
            MudIcon::new(icon.to_owned())
                .size_px(icon_px)
                .custom_color(icon_color),
        );
        cursor_x += icon_px + ICON_GAP;
    }

    // Title.
    let title_rect = Rect::from_min_max(
        pos2(cursor_x, header_rect.top()),
        pos2(header_rect.right() - ROW_PADDING_X, header_rect.bottom()),
    );
    if let Some(title_content) = group.title_content.as_mut() {
        let mut child_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(title_rect)
                .layout(egui::Layout::left_to_right(egui::Align::Center)),
        );
        child_ui.set_clip_rect(header_rect);
        title_content(&mut child_ui);
    } else if let Some(title) = group.title.as_deref() {
        let mut title_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(title_rect)
                .layout(egui::Layout::left_to_right(egui::Align::Center)),
        );
        title_ui.set_clip_rect(header_rect);
        title_ui.add(
            MudText::new(title.to_owned())
                .typo(Typo::Subtitle2)
                .custom_color(if group.disabled {
                    cfg.text_disabled
                } else {
                    cfg.text_primary
                })
                .bold(true),
        );
    }

    // Accessibility.
    let title_label = group.title.clone().unwrap_or_default();
    let group_enabled = !group.disabled;
    header_resp.clone().widget_info(|| {
        egui::WidgetInfo::labeled(egui::Role::Button, group_enabled, title_label.as_str())
    });

    // Header click toggles expansion.
    if header_resp.clicked() && !group.disabled && group.has_children() {
        let now_expanded = state.toggle_expanded(&group.id);
        if let Some(cb) = group.on_expanded_changed.as_mut() {
            cb(now_expanded);
        }
    }

    // 3. Children, when expanded.
    if expanded && !group.children.is_empty() {
        let render_children = |ui: &mut Ui| {
            let n = group.children.len();
            for (i, child) in group.children.iter_mut().enumerate() {
                render_entry(
                    ui,
                    nav,
                    state,
                    child,
                    cfg,
                    base_id,
                    depth + 1,
                    i,
                );
                if cfg.margin.is_some() && i + 1 < n {
                    ui.add_space(cfg.margin.unwrap_or(0.0));
                }
            }
        };
        if let Some(max_h) = group.max_height {
            egui::ScrollArea::vertical()
                .max_height(max_h)
                .auto_shrink([false, false])
                .show(ui, render_children);
        } else {
            ui.scope(render_children);
        }
    }

    let _ = index;
}

/// Dispatches navigation for a clicked link, honouring `force_load` and
/// `target`.
fn handle_navigation(
    ui: &Ui,
    nav: &mut NavigationManager,
    href: &str,
    force_load: bool,
    target: Option<&str>,
) {
    let new_tab = target_opens_new_tab(target);

    // ForceLoad: always open in the system browser at the absolute URL.
    if force_load {
        let url = nav.to_absolute(href);
        ui.ctx().open_url(OpenUrl { url, new_tab });
        return;
    }

    // External: open in the system browser.
    if is_external_url(href, nav.base_uri()) {
        ui.ctx().open_url(OpenUrl {
            url: href.to_owned(),
            new_tab,
        });
        return;
    }

    // Internal: route through the manager.
    let _ = nav.navigate_to(href, false);
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use mud_egui_core::enums::Color;

    // ── row_height ─────────────────────────────────────────────────────────

    #[test]
    fn row_height_matches_density() {
        assert_eq!(row_height(false), ROW_HEIGHT_NORMAL);
        assert_eq!(row_height(true), ROW_HEIGHT_DENSE);
    }

    // ── normalize_path ─────────────────────────────────────────────────────

    #[test]
    fn normalize_path_strips_query_and_fragment() {
        assert_eq!(normalize_path("/a?x=1"), "/a");
        assert_eq!(normalize_path("/a#top"), "/a");
        assert_eq!(normalize_path("/a?x=1#top"), "/a");
    }

    #[test]
    fn normalize_path_strips_trailing_slash() {
        assert_eq!(normalize_path("/a/"), "/a");
        assert_eq!(normalize_path("/"), "/");
        assert_eq!(normalize_path(""), "");
    }

    #[test]
    fn normalize_path_leaves_interior_alone() {
        assert_eq!(normalize_path("/a/b/c"), "/a/b/c");
    }

    // ── is_link_active ─────────────────────────────────────────────────────

    #[test]
    fn prefix_matches_exact() {
        assert!(is_link_active("/orders", "/orders", NavLinkMatch::Prefix));
    }

    #[test]
    fn prefix_matches_segment_boundary() {
        assert!(is_link_active(
            "/orders",
            "/orders/42",
            NavLinkMatch::Prefix
        ));
    }

    #[test]
    fn prefix_rejects_substring_collision() {
        assert!(!is_link_active(
            "/orders",
            "/orders-history",
            NavLinkMatch::Prefix
        ));
    }

    #[test]
    fn prefix_root_matches_everything() {
        assert!(is_link_active("/", "/anything", NavLinkMatch::Prefix));
        assert!(is_link_active("/", "/", NavLinkMatch::Prefix));
    }

    #[test]
    fn prefix_is_case_insensitive() {
        assert!(is_link_active("/Orders", "/orders/42", NavLinkMatch::Prefix));
    }

    #[test]
    fn all_requires_exact_match() {
        assert!(is_link_active("/orders", "/orders", NavLinkMatch::All));
        assert!(!is_link_active("/orders", "/orders/42", NavLinkMatch::All));
    }

    #[test]
    fn all_ignores_current_query() {
        assert!(is_link_active("/orders", "/orders?page=2", NavLinkMatch::All));
    }

    #[test]
    fn all_ignores_href_query() {
        assert!(is_link_active("/orders?x=1", "/orders", NavLinkMatch::All));
    }

    #[test]
    fn external_urls_never_match() {
        for href in [
            "https://example.com/orders",
            "mailto:a@b.com",
            "tel:+15551234",
            "ftp://files/x",
            "file:///etc/hosts",
            "data:image/png;base64,xxx",
        ] {
            assert!(
                !is_link_active(href, "/orders", NavLinkMatch::Prefix),
                "{href} must not match"
            );
            assert!(
                !is_link_active(href, "/orders", NavLinkMatch::All),
                "{href} must not match (All)"
            );
        }
    }

    #[test]
    fn prefix_shorter_current_returns_false() {
        assert!(!is_link_active("/orders/42", "/orders", NavLinkMatch::Prefix));
    }

    // ── resolve_accent ─────────────────────────────────────────────────────

    #[test]
    fn resolve_accent_default_uses_primary() {
        let theme = MudTheme::default();
        let palette = theme.current_palette();
        let primary: Color32 = palette.primary.into();
        assert_eq!(resolve_accent(Color::Default, palette), primary);
        assert_eq!(resolve_accent(Color::Inherit, palette), primary);
    }

    #[test]
    fn resolve_accent_explicit_uses_palette() {
        let theme = MudTheme::default();
        let palette = theme.current_palette();
        let error: Color32 = palette.error.into();
        assert_eq!(resolve_accent(Color::Error, palette), error);
    }

    // ── target_opens_new_tab ──────────────────────────────────────────────

    #[test]
    fn target_blank_opens_new_tab() {
        assert!(target_opens_new_tab(Some("_blank")));
        assert!(target_opens_new_tab(Some("_BLANK")));
    }

    #[test]
    fn other_targets_do_not_open_new_tab() {
        assert!(!target_opens_new_tab(Some("_self")));
        assert!(!target_opens_new_tab(Some("_parent")));
        assert!(!target_opens_new_tab(Some("_top")));
        assert!(!target_opens_new_tab(Some("frame")));
        assert!(!target_opens_new_tab(None));
    }

    // ── NavMenuState ───────────────────────────────────────────────────────

    #[test]
    fn state_starts_empty() {
        let s = NavMenuState::new();
        assert!(s.expanded.is_empty());
    }

    #[test]
    fn ensure_expanded_seeds_once() {
        let mut s = NavMenuState::new();
        assert_eq!(s.is_expanded("g"), None);
        s.ensure_expanded("g", true);
        assert_eq!(s.is_expanded("g"), Some(true));
        s.ensure_expanded("g", false);
        assert_eq!(s.is_expanded("g"), Some(true), "seed must not overwrite");
    }

    #[test]
    fn toggle_expanded_flips() {
        let mut s = NavMenuState::new();
        assert!(s.toggle_expanded("g"));
        assert_eq!(s.is_expanded("g"), Some(true));
        assert!(!s.toggle_expanded("g"));
        assert_eq!(s.is_expanded("g"), Some(false));
    }

    #[test]
    fn set_expanded_sets_value() {
        let mut s = NavMenuState::new();
        s.set_expanded("g", true);
        assert_eq!(s.is_expanded("g"), Some(true));
        s.set_expanded("g", false);
        assert_eq!(s.is_expanded("g"), Some(false));
    }

    #[test]
    fn clear_empties_state() {
        let mut s = NavMenuState::new();
        s.set_expanded("a", true);
        s.set_expanded("b", true);
        s.clear();
        assert!(s.expanded.is_empty());
    }

    // ── Defaults ───────────────────────────────────────────────────────────

    #[test]
    fn menu_defaults_match_mudblazor() {
        let m = MudNavMenu::new();
        assert!(!m.bordered);
        assert!(!m.rounded);
        assert!(!m.dense);
        assert!(matches!(m.color, Color::Default));
        assert!(m.margin.is_none());
    }

    #[test]
    fn menu_builders_set_every_field() {
        let m = MudNavMenu::new()
            .bordered(true)
            .rounded(true)
            .dense(true)
            .color(Color::Error)
            .margin(8.0)
            .classes(["my-class"]);
        assert!(m.bordered);
        assert!(m.rounded);
        assert!(m.dense);
        assert!(matches!(m.color, Color::Error));
        assert_eq!(m.margin, Some(8.0));
        assert!(m.base.has_class("my-class"));
    }

    #[test]
    fn margin_clamps_negative() {
        let m = MudNavMenu::new().margin(-5.0);
        assert_eq!(m.margin, Some(0.0));
    }
}