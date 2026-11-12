//! `MudBreadcrumbs` — a horizontal, wrapping trail of links.
//!
//! Port of `MudBreadcrumbs.razor.cs` (MudBlazor 8.13.0).
//!
//! # MudBlazor parity
//!
//! | MudBlazor parameter | Rust method | Default |
//! |---------------------|-------------|---------|
//! | `Collapsed` (get-only) | [`BreadcrumbResponse::collapsed`] | derived |
//! | `ExpanderIcon`      | [`MudBreadcrumbs::expander_icon`] | `SETTINGS_ETHERNET` |
//! | `ItemTemplate`      | [`MudBreadcrumbs::item_template`] | `None` |
//! | `Items`             | [`MudBreadcrumbs::new`] / [`MudBreadcrumbs::items`] | required |
//! | `MaxItems`          | [`MudBreadcrumbs::max_items`] | `None` |
//! | `Separator`         | [`MudBreadcrumbs::separator`] | `"/"` |
//! | `SeparatorTemplate` | [`MudBreadcrumbs::separator_template`] | `None` |
//! | `Class` (base)      | [`MudBreadcrumbs::classes`] | `None` |
//! | `Style` (base)      | *(use `classes`)* | — |
//! | `Tag` (base)        | **not ported** (no HTML tag) | — |
//! | `UserAttributes`    | **not ported** | — |
//!
//! ### Extensions (not in MudBlazor)
//!
//! | Extension | Description |
//! |-----------|-------------|
//! | [`separator_icon`](MudBreadcrumbs::separator_icon) | Render an SVG glyph instead of a text separator. |
//! | [`size`](MudBreadcrumbs::size) | Affects icon px and gap. |
//! | [`typo`](MudBreadcrumbs::typo) | Passed to `MudText`. |
//! | [`color`](MudBreadcrumbs::color) | Default link colour override. |
//! | [`mark_last_current`](MudBreadcrumbs::mark_last_current) | Last item uses the "current" style and ignores clicks. |
//! | [`id_salt`](MudBreadcrumbs::id_salt) | Stable interaction ids. |
//! | [`on_navigate`](MudBreadcrumbs::on_navigate) | Fires before navigation, matching `MudLink`. |
//! | [`show_into`](MudBreadcrumbs::show_into) | Writes `collapsed` back into a caller-owned `&mut bool`. |
//!
//! ### Not ported
//!
//! * **`Tag` / `UserAttributes`** — no HTML tag.
//! * **`Style`** — egui has no inline CSS strings; use `classes(..)`.
//!
//! # Collapse semantics
//!
//! `MaxItems` is `Option<u8>`. When `Some(max)` and `items.len() > max as usize`,
//! the trail is collapsed:
//!
//! * the **first** item is always shown,
//! * the **expander icon** is shown next,
//! * the **last `max - 1`** items are shown after the expander.
//!
//! `max == 0` is treated as `max == 1` (only the expander is visible).
//! `max == None` never collapses. This matches MudBlazor’s "one head, one
//! expander, tail" layout.
//!
//! # Navigation
//!
//! Clicking a non-current, non-disabled item with an `href`:
//!
//! * **Internal** (relative path or same-origin URL) →
//!   `nav.navigate_to(href, false)`.
//! * **External** (different scheme, or a known browser scheme like
//!   `mailto:` / `tel:`) → `ui.ctx().open_url(OpenUrl { url, new_tab: false })`.
//!
//! The rule is identical to [`MudLink`](crate::navigation::link::MudLink) and
//! implemented by the shared `is_external_url` helper.
//!
//! # Text and icons
//!
//! Text through [`MudText`], icons through [`MudIcon`] — no raw
//! `egui::Label` / `painter().text`.

use egui::{
    pos2, vec2, Color32, CursorIcon, OpenUrl, Pos2, Rect, Response, Sense, Stroke, Ui,
};
use mud_egui_bootstrap::bs_style::BsStyle;
use mud_egui_core::base::component_base::ComponentBase;
use mud_egui_core::enums::{Color, Size, StyleFlavor, Typo};
use mud_egui_icons_material::filled;
use mud_egui_navigation::navigation::NavigationManager;
use mud_egui_tailwind::tw_style::{TwState, TwStyle};
use mud_egui_theming::models::palettes::palette::Palette;
use mud_egui_theming::theme::MudTheme;

use crate::data_display::icons::mud_icon::MudIcon;
use crate::data_display::link::is_external_url;
use crate::typography::mud_text::MudText;

use super::breadcrumb_action::BreadcrumbAction;
use super::breadcrumb_item::BreadcrumbItem;

// =============================================================================
// Constants
// =============================================================================

/// Base rem size passed to the Tailwind resolver.
const REM_SIZE_PT: f32 = 14.0;

/// Horizontal gap between an icon and its label.
const ICON_TEXT_GAP: f32 = 4.0;

/// Horizontal gap between the item and the separator.
const ITEM_GAP: f32 = 6.0;

/// Underline stroke width on hover.
const UNDERLINE_THICKNESS: f32 = 1.0;

/// Underline offset from the bottom of the item's rect.
const UNDERLINE_OFFSET: f32 = 2.0;

// =============================================================================
// Pure helpers
// =============================================================================

/// Maps a [`Size`] to a leading-icon side length in pixels.
#[inline]
pub fn icon_size_px(size: Size) -> f32 {
    match size {
        Size::Small => 14.0,
        Size::Medium => 16.0,
        Size::Large => 20.0,
        _ => 16.0,
    }
}

/// Whether the item at `index` should be treated as "current".
///
/// MudBlazor has no explicit current-item flag; `mark_last_current(true)`
/// treats the last item as current so it renders in the secondary colour and
/// ignores clicks — the convention every breadcrumb shows in practice.
#[inline]
pub fn is_current_item(index: usize, len: usize, mark_last_current: bool) -> bool {
    mark_last_current && index + 1 == len
}

/// Which indices are visible after collapse planning.
///
/// See the module docs for the exact semantics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisiblePlan {
    /// Indices shown before the expander.
    pub head: Vec<usize>,
    /// `true` when the expander glyph should be rendered.
    pub expander: bool,
    /// Indices shown after the expander.
    pub tail: Vec<usize>,
    /// `true` when the plan is the collapsed one.
    pub collapsed: bool,
}

/// Computes the [`VisiblePlan`] for `len` items with an optional `max_items`.
///
/// Pure: no `egui::Context`, no allocations beyond the returned `Vec`s.
pub fn plan_visible_indices(len: usize, max_items: Option<u8>) -> VisiblePlan {
    // `max_items == 0` is coerced to 1 so the expander is never the only
    // thing visible.
    let max = max_items
        .map(|m| (m.max(1)) as usize)
        .unwrap_or(usize::MAX);

    if len <= max {
        return VisiblePlan {
            head: (0..len).collect(),
            expander: false,
            tail: Vec::new(),
            collapsed: false,
        };
    }

    // Head: always the first item.
    // Tail: last `max - 1` items, with `max >= 1` so `tail_start >= 1`
    // given `len > max`. That guarantees head and tail are disjoint.
    let head = vec![0];
    let tail_start = len.saturating_sub(max.saturating_sub(1));
    let tail: Vec<usize> = (tail_start..len).collect();

    VisiblePlan {
        head,
        expander: true,
        tail,
        collapsed: true,
    }
}

/// Resolves a per-item colour using `item.color` first, then `default`, with
/// the current-item / disabled fallbacks applied by the caller.
#[inline]
fn resolve_color(opt: Option<Color>, palette: &Palette, fallback: Color32) -> Color32 {
    match opt {
        Some(Color::Default) | Some(Color::Inherit) | None => fallback,
        Some(other) => other.resolve(palette).unwrap_or(fallback),
    }
}

// =============================================================================
// Response
// =============================================================================

/// The result of one [`MudBreadcrumbs::show`] frame.
#[derive(Debug)]
pub struct BreadcrumbResponse {
    /// The breadcrumb wrapper's own `egui::Response`.
    pub response: Response,

    /// `true` when the trail was rendered in collapsed form this frame.
    pub collapsed: bool,

    /// The navigation outcome for this frame (if any).
    pub action: BreadcrumbAction,
}

// =============================================================================
// MudBreadcrumbs
// =============================================================================

/// A horizontal, wrapping trail of links.
#[must_use = "MudBreadcrumbs does nothing until you call `.show(ui, nav)`"]
pub struct MudBreadcrumbs<'a> {
    /// The list of items to display (`Items`).
    pub items: &'a [BreadcrumbItem],

    /// Icon shown when the trail is collapsed (`ExpanderIcon`).
    pub expander_icon: String,

    /// Maximum number of items shown before collapsing (`MaxItems`).
    pub max_items: Option<u8>,

    /// Text separator between items (`Separator`). Ignored when
    /// `separator_icon` or `separator_template` is set.
    pub separator: String,

    /// When set, renders this SVG glyph as the separator instead of
    /// `separator` (extension).
    pub separator_icon: Option<String>,

    /// Custom separator renderer. Overrides both `separator` and
    /// `separator_icon` (`SeparatorTemplate`).
    pub separator_template: Option<Box<dyn FnMut(&mut Ui) + 'a>>,

    /// Custom item renderer. Replaces the default icon + text content but
    /// keeps the interaction wrapper (`ItemTemplate`).
    pub item_template: Option<Box<dyn FnMut(&mut Ui, &BreadcrumbItem, bool) + 'a>>,

    // ── Extensions ─────────────────────────────────────────────────────────
    /// Affects icon px and gap between items. Default `Medium`.
    pub size: Size,

    /// Typography passed to `MudText`. Default `Body2`.
    pub typo: Typo,

    /// Default link colour. `None` uses `TextPrimary` for links,
    /// `TextSecondary` for the current item, `TextDisabled` for disabled.
    pub color: Option<Color>,

    /// When `true`, the last item is treated as current (no click, secondary
    /// colour) unless it has an explicit `.color(..)`.
    pub mark_last_current: bool,

    /// Stable interaction id salt. When `None`, the salt is derived from the
    /// wrapper's `ui.scope_id()`.
    pub id_salt: Option<String>,

    /// Fires after an item's `href` is resolved but before the navigation is
    /// dispatched.
    pub on_navigate: Option<Box<dyn FnMut(usize, &BreadcrumbItem) + 'a>>,

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
}

impl<'a> Default for MudBreadcrumbs<'a> {
    fn default() -> Self {
        Self {
            items: &[],
            expander_icon: filled::SETTINGS_ETHERNET.to_owned(),
            max_items: None,
            separator: "/".to_owned(),
            separator_icon: None,
            separator_template: None,
            item_template: None,
            size: Size::Medium,
            typo: Typo::Body2,
            color: None,
            mark_last_current: false,
            id_salt: None,
            on_navigate: None,
            base: ComponentBase::new(),
            flavor: StyleFlavor::default(),
            bs_style: BsStyle::default(),
            tw_style: TwStyle::default(),
            tw_state: TwState::default(),
        }
    }
}

impl<'a> MudBreadcrumbs<'a> {
    /// Creates a breadcrumb with the given item slice.
    pub fn new(items: &'a [BreadcrumbItem]) -> Self {
        Self {
            items,
            ..Default::default()
        }
    }

    /// Creates an empty breadcrumb (renders nothing).
    pub fn empty() -> Self {
        Self::default()
    }

    /// Replaces the item slice.
    #[inline]
    pub fn items(mut self, items: &'a [BreadcrumbItem]) -> Self {
        self.items = items;
        self
    }

    // ── MudBlazor builders ─────────────────────────────────────────────────

    /// Sets `ExpanderIcon`.
    #[inline]
    pub fn expander_icon(mut self, svg: impl Into<String>) -> Self {
        self.expander_icon = svg.into();
        self
    }

    /// Sets `MaxItems`. `None` disables collapsing.
    #[inline]
    pub fn max_items(mut self, n: u8) -> Self {
        self.max_items = Some(n);
        self
    }

    /// Sets `Separator` (text).
    #[inline]
    pub fn separator(mut self, s: impl Into<String>) -> Self {
        self.separator = s.into();
        self
    }

    /// Sets a custom `SeparatorTemplate`.
    #[inline]
    pub fn separator_template(mut self, f: impl FnMut(&mut Ui) + 'a) -> Self {
        self.separator_template = Some(Box::new(f));
        self
    }

    /// Sets a custom `ItemTemplate`. Receives the `Ui`, the item, and
    /// `is_current`.
    #[inline]
    pub fn item_template(
        mut self,
        f: impl FnMut(&mut Ui, &BreadcrumbItem, bool) + 'a,
    ) -> Self {
        self.item_template = Some(Box::new(f));
        self
    }

    // ── Extensions ─────────────────────────────────────────────────────────

    /// Uses an SVG glyph as the separator instead of `separator` text.
    #[inline]
    pub fn separator_icon(mut self, svg: impl Into<String>) -> Self {
        self.separator_icon = Some(svg.into());
        self
    }

    /// Sets the icon/label size preset.
    #[inline]
    pub fn size(mut self, s: Size) -> Self {
        self.size = s;
        self
    }

    /// Sets the typography passed to `MudText`.
    #[inline]
    pub fn typo(mut self, t: Typo) -> Self {
        self.typo = t;
        self
    }

    /// Sets the default link colour.
    #[inline]
    pub fn color(mut self, c: Color) -> Self {
        self.color = Some(c);
        self
    }

    /// Marks the last item as current.
    #[inline]
    pub fn mark_last_current(mut self, b: bool) -> Self {
        self.mark_last_current = b;
        self
    }

    /// Sets a stable interaction id salt.
    #[inline]
    pub fn id_salt(mut self, s: impl Into<String>) -> Self {
        self.id_salt = Some(s.into());
        self
    }

    /// Registers an `on_navigate` callback fired before dispatching the
    /// navigation for a clicked item.
    #[inline]
    pub fn on_navigate(
        mut self,
        f: impl FnMut(usize, &BreadcrumbItem) + 'a,
    ) -> Self {
        self.on_navigate = Some(Box::new(f));
        self
    }

    /// Adds utility classes to the wrapper.
    #[inline]
    pub fn classes<S: AsRef<str>>(
        mut self,
        classes: impl IntoIterator<Item = S>,
    ) -> Self {
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

    /// Draws the breadcrumb and returns its [`BreadcrumbResponse`].
    pub fn show(mut self, ui: &mut Ui, nav: &mut NavigationManager) -> BreadcrumbResponse {
        self.show_impl(ui, Some(nav), None)
    }

    /// Alias for [`show`](Self::show), kept for backward compatibility.
    pub fn show_and_navigate(
        self,
        ui: &mut Ui,
        nav: &mut NavigationManager,
    ) -> BreadcrumbResponse {
        self.show(ui, nav)
    }

    /// Draws the breadcrumb and writes `collapsed` back into the caller's
    /// `&mut bool`.
    pub fn show_into(
        self,
        ui: &mut Ui,
        nav: &mut NavigationManager,
        collapsed: &mut bool,
    ) -> BreadcrumbResponse {
        let resp = self.show_impl(ui, Some(nav), Some(collapsed));
        resp
    }

    fn show_impl(
        mut self,
        ui: &mut Ui,
        nav: Option<&mut NavigationManager>,
        mut collapsed_out: Option<&mut bool>,
    ) -> BreadcrumbResponse {
        // 1. Theme + frame.
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

        // 2. Collapse plan.
        let plan = plan_visible_indices(self.items.len(), self.max_items);
        if let Some(out) = collapsed_out.as_deref_mut() {
            *out = plan.collapsed;
        }

        // 3. Stable id salt.
        let scope_id = ui.scope_id();
        let base_id = match self.id_salt.as_deref() {
            Some(s) => scope_id.with(s),
            None => scope_id.with("mud_breadcrumbs"),
        };

        // 4. Prepare everything the closure needs to borrow.
        let nav_ptr: Option<&mut NavigationManager> = nav;
        let mut action = BreadcrumbAction::None;
        let items = self.items;
        let size = self.size;
        let typo = self.typo;
        let default_color = self.color;
        let mark_last = self.mark_last_current;
        let expander_icon = self.expander_icon.clone();
        let separator = self.separator.clone();
        let separator_icon = self.separator_icon.clone();
        let mut separator_template = self.separator_template.take();
        let mut item_template = self.item_template.take();
        let mut on_navigate = self.on_navigate.take();

        let inner = frame.show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                let mut nav = nav_ptr;

                // 4a. Head.
                for &idx in &plan.head {
                    render_one(
                        ui,
                        items,
                        idx,
                        mark_last,
                        size,
                        typo,
                        default_color,
                        &base_id,
                        &mut item_template,
                        &mut on_navigate,
                        &mut action,
                        &mut nav,
                        palette,
                    );
                    render_separator(
                        ui,
                        size,
                        &separator,
                        separator_icon.as_deref(),
                        &mut separator_template,
                        palette,
                    );
                }

                // 4b. Expander.
                if plan.expander {
                    ui.add(
                        MudIcon::new(expander_icon.clone())
                            .size_px(icon_size_px(size))
                            .custom_color(palette.text_secondary.into()),
                    );
                    render_separator(
                        ui,
                        size,
                        &separator,
                        separator_icon.as_deref(),
                        &mut separator_template,
                        palette,
                    );
                }

                // 4c. Tail.
                let tail_len = plan.tail.len();
                for (i, &idx) in plan.tail.iter().enumerate() {
                    render_one(
                        ui,
                        items,
                        idx,
                        mark_last,
                        size,
                        typo,
                        default_color,
                        &base_id,
                        &mut item_template,
                        &mut on_navigate,
                        &mut action,
                        &mut nav,
                        palette,
                    );
                    if i + 1 < tail_len {
                        render_separator(
                            ui,
                            size,
                            &separator,
                            separator_icon.as_deref(),
                            &mut separator_template,
                            palette,
                        );
                    }
                }
            })
                .response
        });

        BreadcrumbResponse {
            response: inner.inner,
            collapsed: plan.collapsed,
            action,
        }
    }
}

// =============================================================================
// Render helpers
// =============================================================================

#[allow(clippy::too_many_arguments)]
fn render_one<'a>(
    ui: &mut Ui,
    items: &'a [BreadcrumbItem],
    index: usize,
    mark_last: bool,
    size: Size,
    typo: Typo,
    default_color: Option<Color>,
    base_id: &egui::Id,
    item_template: &mut Option<Box<dyn FnMut(&mut Ui, &BreadcrumbItem, bool) + 'a>>,
    on_navigate: &mut Option<Box<dyn FnMut(usize, &BreadcrumbItem) + 'a>>,
    action: &mut BreadcrumbAction,
    nav: &mut Option<&mut NavigationManager>,
    palette: &Palette,
) {
    let Some(item) = items.get(index) else {
        return;
    };
    let is_current = is_current_item(index, items.len(), mark_last);
    let interactive = !is_current && !item.disabled && item.href.is_some();

    // Resolve the base colour.
    let color: Color32 = if item.disabled {
        palette.text_disabled.into()
    } else if is_current && item.color.is_none() {
        palette.text_secondary.into()
    } else {
        resolve_color(
            item.color.or(default_color),
            palette,
            palette.text_primary.into(),
        )
    };

    // Render content.
    let scope = ui.scope(|ui| {
        if let Some(template) = item_template.as_mut() {
            template(ui, item, is_current);
        } else {
            if let Some(icon) = item.icon.as_deref() {
                ui.add(
                    MudIcon::new(icon.to_owned())
                        .size_px(icon_size_px(size))
                        .custom_color(color),
                );
                ui.add_space(ICON_TEXT_GAP);
            }
            ui.add(
                MudText::new(item.text.clone())
                    .typo(typo)
                    .custom_color(color),
            );
        }
    });

    let item_rect = scope.response.rect;

    // Interaction overlay.
    let sense = if interactive { Sense::click() } else { Sense::hover() };
    let body_resp = ui.interact(
        item_rect,
        base_id.with(("bc_item", index)),
        sense,
    );

    // Hover affordance for interactive items.
    if interactive && body_resp.hovered() {
        ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
        let underline_y = item_rect.bottom() - UNDERLINE_OFFSET;
        if underline_y > item_rect.top() {
            ui.painter().line_segment(
                [
                    pos2(item_rect.left(), underline_y),
                    pos2(item_rect.right(), underline_y),
                ],
                Stroke::new(UNDERLINE_THICKNESS, color),
            );
        }
    }

    // Accessibility.
    let label = item.text.clone();
    body_resp.clone().widget_info(|| {
        egui::WidgetInfo::labeled(
            egui::Role::Link,
            !item.disabled && !is_current,
            label.as_str(),
        )
    });

    // Handle click.
    if body_resp.clicked() && interactive {
        let href = item.href.clone().unwrap_or_default();

        if let Some(cb) = on_navigate.as_mut() {
            cb(index, item);
        }

        match nav.as_deref_mut() {
            Some(manager) => {
                if is_external_url(&href, manager.base_uri()) {
                    ui.ctx().open_url(OpenUrl {
                        url: href.clone(),
                        new_tab: false,
                    });
                    *action = BreadcrumbAction::External {
                        index,
                        url: href,
                    };
                } else {
                    match manager.navigate_to(&href, false) {
                        Ok(true) => {
                            *action = BreadcrumbAction::Internal {
                                index,
                                uri: href,
                            };
                        }
                        Ok(false) => {
                            *action = BreadcrumbAction::Vetoed {
                                index,
                                uri: href,
                            };
                        }
                        Err(e) => {
                            *action = BreadcrumbAction::Error {
                                index,
                                uri: href,
                                message: e.to_string(),
                            };
                        }
                    }
                }
            }
            None => {
                // No router: treat as external.
                ui.ctx().open_url(OpenUrl {
                    url: href.clone(),
                    new_tab: false,
                });
                *action = BreadcrumbAction::External {
                    index,
                    url: href,
                };
            }
        }
    }

    ui.add_space(ITEM_GAP);
}

#[allow(clippy::too_many_arguments)]
fn render_separator<'a>(
    ui: &mut Ui,
    size: Size,
    separator: &str,
    separator_icon: Option<&str>,
    separator_template: &mut Option<Box<dyn FnMut(&mut Ui) + 'a>>,
    palette: &Palette,
) {
    if let Some(template) = separator_template.as_mut() {
        template(ui);
    } else if let Some(icon) = separator_icon {
        ui.add(
            MudIcon::new(icon.to_owned())
                .size_px(icon_size_px(size))
                .custom_color(palette.text_secondary.into()),
        );
    } else {
        ui.add(
            MudText::new(separator.to_owned())
                .typo(Typo::Body2)
                .custom_color(palette.text_secondary.into()),
        );
    }
    ui.add_space(ITEM_GAP);
}

// =============================================================================
// Widget impl
// =============================================================================

impl<'a> egui::Widget for MudBreadcrumbs<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        // `Widget::ui` has no `NavigationManager`; treat every link as
        // external-only. Use `MudBreadcrumbs::show(ui, nav)` when you have
        // a router.
        self.show_impl(ui, None, None).response
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // --- icon_size_px -----------------------------------------------------

    #[test]
    fn icon_size_px_maps_sizes() {
        assert_eq!(icon_size_px(Size::Small), 14.0);
        assert_eq!(icon_size_px(Size::Medium), 16.0);
        assert_eq!(icon_size_px(Size::Large), 20.0);
        assert_eq!(icon_size_px(Size::ExtraSmall), 16.0);
        assert_eq!(icon_size_px(Size::ExtraLarge), 16.0);
    }

    // --- is_current_item --------------------------------------------------

    #[test]
    fn is_current_only_for_last_when_marked() {
        assert!(!is_current_item(0, 3, true));
        assert!(!is_current_item(1, 3, true));
        assert!(is_current_item(2, 3, true));
    }

    #[test]
    fn is_current_never_when_not_marked() {
        for i in 0..3 {
            assert!(!is_current_item(i, 3, false));
        }
    }

    #[test]
    fn is_current_empty_slice() {
        assert!(!is_current_item(0, 0, true));
    }

    // --- plan_visible_indices --------------------------------------------

    #[test]
    fn plan_uncollapsed_when_len_fits() {
        let p = plan_visible_indices(3, Some(5));
        assert!(!p.collapsed);
        assert!(!p.expander);
        assert_eq!(p.head, vec![0, 1, 2]);
        assert!(p.tail.is_empty());
    }

    #[test]
    fn plan_uncollapsed_when_no_max() {
        let p = plan_visible_indices(100, None);
        assert!(!p.collapsed);
        assert!(!p.expander);
        assert_eq!(p.head.len(), 100);
    }

    #[test]
    fn plan_uncollapsed_when_len_equals_max() {
        let p = plan_visible_indices(5, Some(5));
        assert!(!p.collapsed);
        assert!(!p.expander);
        assert_eq!(p.head, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn plan_collapses_when_len_exceeds_max() {
        let p = plan_visible_indices(10, Some(3));
        assert!(p.collapsed);
        assert!(p.expander);
        assert_eq!(p.head, vec![0]);
        // Tail: last (3 - 1) = 2 items → indices 8, 9.
        assert_eq!(p.tail, vec![8, 9]);
    }

    #[test]
    fn plan_tail_start_does_not_overlap_head() {
        // len = max + 1 → tail_start == 2, head = [0], no overlap.
        let p = plan_visible_indices(4, Some(3));
        assert_eq!(p.head, vec![0]);
        assert_eq!(p.tail, vec![3]);
    }

    #[test]
    fn plan_max_one_collapses_to_head_and_expander() {
        let p = plan_visible_indices(10, Some(1));
        assert!(p.collapsed);
        assert!(p.expander);
        assert_eq!(p.head, vec![0]);
        assert!(p.tail.is_empty());
    }

    #[test]
    fn plan_max_zero_is_treated_as_one() {
        let p = plan_visible_indices(3, Some(0));
        assert!(p.collapsed);
        assert!(p.expander);
        assert_eq!(p.head, vec![0]);
    }

    #[test]
    fn plan_empty_slice_is_not_collapsed() {
        let p = plan_visible_indices(0, Some(3));
        assert!(!p.collapsed);
        assert!(!p.expander);
        assert!(p.head.is_empty());
        assert!(p.tail.is_empty());
    }

    // --- resolve_color ----------------------------------------------------

    #[test]
    fn resolve_color_falls_back_for_none_and_default_and_inherit() {
        let theme = MudTheme::default();
        let palette = theme.current_palette();
        let fallback = Color32::from_rgb(1, 2, 3);
        assert_eq!(resolve_color(None, palette, fallback), fallback);
        assert_eq!(
            resolve_color(Some(Color::Default), palette, fallback),
            fallback
        );
        assert_eq!(
            resolve_color(Some(Color::Inherit), palette, fallback),
            fallback
        );
    }

    #[test]
    fn resolve_color_uses_palette_for_explicit_colors() {
        let theme = MudTheme::default();
        let palette = theme.current_palette();
        let primary: Color32 = palette.primary.into();
        let fallback = Color32::from_rgb(99, 0, 0);
        assert_eq!(
            resolve_color(Some(Color::Primary), palette, fallback),
            primary
        );
    }

    // --- defaults ---------------------------------------------------------

    #[test]
    fn defaults_match_mudblazor() {
        let bc = MudBreadcrumbs::empty();
        assert!(bc.items.is_empty());
        assert_eq!(bc.expander_icon, filled::SETTINGS_ETHERNET);
        assert!(bc.max_items.is_none());
        assert_eq!(bc.separator, "/");
        assert!(bc.separator_icon.is_none());
        assert!(bc.separator_template.is_none());
        assert!(bc.item_template.is_none());
        assert_eq!(bc.size, Size::Medium);
        assert_eq!(bc.typo, Typo::Body2);
        assert!(bc.color.is_none());
        assert!(!bc.mark_last_current);
        assert!(bc.id_salt.is_none());
        assert!(bc.on_navigate.is_none());
    }

    #[test]
    fn new_sets_items_slice() {
        let items = vec![BreadcrumbItem::new("A")];
        let bc = MudBreadcrumbs::new(&items);
        assert_eq!(bc.items.len(), 1);
    }

    // --- builders ---------------------------------------------------------

    #[test]
    fn builders_set_every_field() {
        let items = vec![BreadcrumbItem::link("Home", "/")];
        let bc = MudBreadcrumbs::new(&items)
            .expander_icon("EXP")
            .max_items(3)
            .separator(">")
            .separator_icon("SEP")
            .separator_template(|_ui| {})
            .item_template(|_ui, _item, _cur| {})
            .size(Size::Small)
            .typo(Typo::Caption)
            .color(Color::Primary)
            .mark_last_current(true)
            .id_salt("bc")
            .on_navigate(|_i, _item| {})
            .classes(["my-class"]);

        assert_eq!(bc.expander_icon, "EXP");
        assert_eq!(bc.max_items, Some(3));
        assert_eq!(bc.separator, ">");
        assert_eq!(bc.separator_icon.as_deref(), Some("SEP"));
        assert!(bc.separator_template.is_some());
        assert!(bc.item_template.is_some());
        assert_eq!(bc.size, Size::Small);
        assert_eq!(bc.typo, Typo::Caption);
        assert_eq!(bc.color, Some(Color::Primary));
        assert!(bc.mark_last_current);
        assert_eq!(bc.id_salt.as_deref(), Some("bc"));
        assert!(bc.on_navigate.is_some());
        assert!(bc.base.has_class("my-class"));
    }

    #[test]
    fn items_builder_replaces_slice() {
        let a = vec![BreadcrumbItem::new("A")];
        let b = vec![BreadcrumbItem::new("B"), BreadcrumbItem::new("C")];
        let bc = MudBreadcrumbs::new(&a).items(&b);
        assert_eq!(bc.items.len(), 2);
        assert_eq!(bc.items[0].text, "B");
    }

    // --- BreadcrumbAction methods ----------------------------------------

    #[test]
    fn action_methods_are_consistent() {
        let i = BreadcrumbAction::Internal {
            index: 1,
            uri: "/x".into(),
        };
        assert!(i.navigated() && !i.is_external() && !i.was_blocked());
        assert_eq!(i.target(), Some("/x"));
        assert_eq!(i.index(), Some(1));

        let e = BreadcrumbAction::External {
            index: 2,
            url: "https://docs.rs".into(),
        };
        assert!(e.is_external());
        assert_eq!(e.target(), Some("https://docs.rs"));
    }
}