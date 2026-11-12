//! `MudMenu` — a click-to-open dropdown anchored to a trigger.
//!
//! Port of `MudMenu.razor.cs` / `MudMenuItem.razor.cs` /
//! `MudMenuSeparator.razor.cs` (MudBlazor 8.13.0).
//!
//! # MudBlazor parity — `MudMenu`
//!
//! | MudBlazor parameter | Rust method | Default |
//! |---------------------|-------------|---------|
//! | `ActivatorContent`  | the `activator` argument of [`MudMenu::show`] | required |
//! | `AnchorOrigin`      | [`MudMenu::anchor_origin`] | `Bottom` |
//! | `TransformOrigin`   | [`MudMenu::transform_origin`] | `TopLeft` |
//! | `Position`          | [`MudMenu::position`] | `None` (uses anchor/transform) |
//! | `FullWidth`         | [`MudMenu::full_width`] | `false` |
//! | `Icon`              | [`MudMenu::icon`] | `None` |
//! | `Label`             | [`MudMenu::label`] | `None` |
//! | `Dense`             | [`MudMenu::dense`] | `false` |
//! | `Disabled`          | [`MudMenu::disabled`] | `false` |
//! | `MenuClass`         | [`MudMenu::menu_class`] | `None` |
//! | `MenuListClass`     | [`MudMenu::menu_list_class`] | `None` |
//! | `MaxHeight`         | [`MudMenu::max_height`] | `None` |
//! | `LockScroll`        | *(unused in egui — Areas are non-scrolling)* | — |
//! | `OffsetX` / `OffsetY` | [`MudMenu::offset_x`] / [`MudMenu::offset_y`] | `0` |
//! | `RelativeWidth`     | [`MudMenu::relative_width`] | `false` |
//! | `OpenOnClick`       | [`MudMenu::open_on_click`] | `true` |
//! | `OpenOnHover`       | [`MudMenu::open_on_hover`] | `false` |
//! | `CloseOnContentClick` | [`MudMenu::close_on_content_click`] | `true` |
//! | `CloseOnEscapeKey`  | [`MudMenu::close_on_escape_key`] | `true` |
//! | `HoverDelay`        | [`MudMenu::hover_delay_ms`] | `100` (stored; see note) |
//! | `DisableRipple`     | [`MudMenu::disable_ripple`] | `false` |
//! | `Class` (base)      | [`MudMenu::classes`] | `None` |
//! | `Style` (base)      | *(use `classes`)* | — |
//! | `Tag` (base)        | **not ported** (no HTML tag) | — |
//! | `UserAttributes`    | **not ported** | — |
//!
//! # MudBlazor parity — `MudMenuItem`
//!
//! See [`MenuItem`](super::MenuItem) for the full table.
//!
//! ### Extensions
//!
//! | Extension | Description |
//! |-----------|-------------|
//! | [`size_hint`](MudMenu::size_hint) | Initial popover size guess. |
//! | [`MenuAction`](super::MenuAction) | Rich return value reporting open/close/item-activation. |
//!
//! ### Not ported
//!
//! * **`LockScroll`** — egui `Area`s don't participate in scroll containers.
//! * **`DisableRipple`** — the port uses hover highlight, not a ripple.
//! * **`Tag` / `UserAttributes`** — no HTML tag.
//! * **`Style`** — no inline CSS strings; use `classes(..)`.
//!
//! # Text and icons
//!
//! Item text renders through [`MudText`], icons through [`MudIcon`]. The
//! activator is whatever the caller draws — the port does not impose a
//! button style.

use egui::{
    pos2, vec2, Color32, CornerRadius, CursorIcon, Id, Layout, OpenUrl, Rect, Response,
    Sense, Ui, UiBuilder, Vec2,
};
use mud_egui_bootstrap::bs_style::BsStyle;
use mud_egui_core::base::component_base::ComponentBase;
use mud_egui_core::enums::{
    AnchorOrigin, Color, Position, StyleFlavor, TransformOrigin, Typo,
};
use mud_egui_navigation::navigation::NavigationManager;
use mud_egui_tailwind::tw_style::{TwState, TwStyle};
use mud_egui_theming::models::palettes::palette::Palette;
use mud_egui_theming::theme::MudTheme;

use crate::data_display::icons::mud_icon::MudIcon;
use crate::data_display::link::is_external_url;
use crate::feedback::popover::{MudPopover, PopoverResponse};
use crate::typography::mud_text::MudText;

use super::menu_action::MenuAction;
use super::menu_entry::MenuEntry;
use super::menu_item::MenuItem;
use super::menu_state::MenuState;
use super::menu_ui::MenuUi;

// =============================================================================
// Constants
// =============================================================================

/// Base rem size passed to the Tailwind resolver.
const REM_SIZE_PT: f32 = 14.0;

/// Height of one menu item row.
pub const ITEM_HEIGHT: f32 = 36.0;

/// Horizontal padding inside an item row.
pub const ITEM_PADDING_X: f32 = 16.0;

/// Gap between an item's icon and its text.
const ITEM_ICON_GAP: f32 = 12.0;

/// Height of a separator.
pub const SEPARATOR_HEIGHT: f32 = 1.0;

/// Vertical margin around a separator.
const SEPARATOR_MARGIN_Y: f32 = 4.0;

/// Corner radius of the popover's paper (matches `MudPopover`).
const PAPER_RADIUS: u8 = 4;

/// Default popover size hint.
const DEFAULT_MENU_SIZE_HINT: Vec2 = Vec2::new(200.0, 120.0);

// =============================================================================
// Pure helpers
// =============================================================================

/// Height of an item row for a given `dense` flag.
#[inline]
pub fn item_height(dense: bool) -> f32 {
    if dense { ITEM_HEIGHT - 8.0 } else { ITEM_HEIGHT }
}

/// Normalises a [`Position`] into an [`AnchorOrigin`] + [`TransformOrigin`]
/// pair. `Position::Start` / `End` are resolved against `right_to_left`.
pub fn position_to_origins(
    position: Position,
    right_to_left: bool,
) -> (AnchorOrigin, TransformOrigin) {
    match position.resolve(right_to_left) {
        Position::Top => (AnchorOrigin::Top, TransformOrigin::BottomLeft),
        Position::Left => (AnchorOrigin::Left, TransformOrigin::CenterRight),
        Position::Right => (AnchorOrigin::Right, TransformOrigin::CenterLeft),
        _ => (AnchorOrigin::Bottom, TransformOrigin::TopLeft),
    }
}

/// Resolves the icon colour for an item (override or secondary text).
#[inline]
pub fn resolve_icon_color(c: Option<Color>, palette: &Palette, fallback: Color32) -> Color32 {
    match c {
        Some(Color::Default) | Some(Color::Inherit) | None => fallback,
        Some(other) => other.resolve(palette).unwrap_or(fallback),
    }
}

/// Applies `offset_x` / `offset_y` to an `AnchorOrigin`, returning the
/// effective popover offset magnitude along the anchor's axis.
///
/// `MudPopover` takes a single `offset` magnitude; for vertical anchors we
/// use `offset_y`, for horizontal we use `offset_x`. When both are set the
/// larger magnitude wins (documented limitation).
#[inline]
pub fn apply_offsets(anchor: AnchorOrigin, offset_x: f32, offset_y: f32) -> (AnchorOrigin, f32) {
    match anchor {
        AnchorOrigin::Top | AnchorOrigin::Bottom => (anchor, offset_y.abs().max(offset_x.abs())),
        _ => (anchor, offset_x.abs().max(offset_y.abs())),
    }
}

// =============================================================================
// Response
// =============================================================================

/// The result of one [`MudMenu::show`] frame.
#[derive(Debug)]
pub struct MenuResponse {
    /// The activator's response.
    pub activator: Response,
    /// The menu action this frame.
    pub action: MenuAction,
    /// The popover response, when the menu was open.
    pub popover: Option<PopoverResponse>,
}

// =============================================================================
// MudMenu
// =============================================================================

/// A click-to-open dropdown anchored to a trigger.
///
/// **Not generic over `'a`.** All fields are owned / `Copy`. The item
/// closures are introduced per-call via the `add_items` argument of
/// [`show`](Self::show), which carries its own lifetime.
#[must_use = "MudMenu does nothing until you call `.show(ui, state, ..)`"]
pub struct MudMenu {
    // ── Activator fallback (MudBlazor: Icon / Label) ───────────────────────
    /// Default activator glyph. Stored for parity; the caller always supplies
    /// an explicit activator closure to `show`.
    pub icon: Option<String>,
    /// Default activator label. Same caveat.
    pub label: Option<String>,

    // ── Placement ──────────────────────────────────────────────────────────
    /// Attach edge (`AnchorOrigin`). Ignored when `position` is set.
    pub anchor_origin: Option<AnchorOrigin>,
    /// Transform origin (`TransformOrigin`). Ignored when `position` is set.
    pub transform_origin: Option<TransformOrigin>,
    /// Coarse placement helper (`Position`). When set, overrides
    /// `anchor_origin` and `transform_origin`.
    pub position: Option<Position>,
    /// RTL flag for `Position::Start` / `End`.
    pub right_to_left: bool,

    // ── Sizing ─────────────────────────────────────────────────────────────
    /// Match the activator's width (`FullWidth`).
    pub full_width: bool,
    /// Match the activator's width as a *minimum* (`RelativeWidth`).
    pub relative_width: bool,
    /// Max height for the item list.
    pub max_height: Option<f32>,
    /// Explicit popover size guess.
    pub size_hint: Option<Vec2>,

    // ── Behaviour ──────────────────────────────────────────────────────────
    /// Compact item heights (`Dense`).
    pub dense: bool,
    /// Disables the activator and all items (`Disabled`).
    pub disabled: bool,
    /// Opens on activator click (`OpenOnClick`, default `true`).
    pub open_on_click: bool,
    /// Opens on activator hover (`OpenOnHover`, default `false`).
    pub open_on_hover: bool,
    /// Hover delay in ms (`HoverDelay`, default `100`). Stored for parity —
    /// the port opens on the first hover frame.
    pub hover_delay_ms: u32,
    /// Closes on item click (`CloseOnContentClick`, default `true`).
    pub close_on_content_click: bool,
    /// Closes on Escape (`CloseOnEscapeKey`, default `true`).
    pub close_on_escape_key: bool,
    /// Stored for parity (`DisableRipple`); not rendered.
    pub disable_ripple: bool,
    /// Horizontal offset (`OffsetX`).
    pub offset_x: f32,
    /// Vertical offset (`OffsetY`).
    pub offset_y: f32,

    // ── CSS classes ────────────────────────────────────────────────────────
    /// Classes on the popover paper (`MenuClass`).
    pub menu_class: Option<String>,
    /// Classes on the item list container (`MenuListClass`).
    pub menu_list_class: Option<String>,

    // ── Utilities ──────────────────────────────────────────────────────────
    /// Utility classes on the wrapper.
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

impl Default for MudMenu {
    fn default() -> Self {
        Self {
            icon: None,
            label: None,
            anchor_origin: None,
            transform_origin: None,
            position: None,
            right_to_left: false,
            full_width: false,
            relative_width: false,
            max_height: None,
            size_hint: None,
            dense: false,
            disabled: false,
            open_on_click: true,
            open_on_hover: false,
            hover_delay_ms: 100,
            close_on_content_click: true,
            close_on_escape_key: true,
            disable_ripple: false,
            offset_x: 0.0,
            offset_y: 0.0,
            menu_class: None,
            menu_list_class: None,
            base: ComponentBase::new(),
            flavor: StyleFlavor::default(),
            bs_style: BsStyle::default(),
            tw_style: TwStyle::default(),
            tw_state: TwState::default(),
        }
    }
}

impl MudMenu {
    /// Creates a menu with MudBlazor defaults.
    pub fn new() -> Self {
        Self::default()
    }

    // ── Builders ───────────────────────────────────────────────────────────

    #[inline] pub fn icon(mut self, svg: impl Into<String>) -> Self { self.icon = Some(svg.into()); self }
    #[inline] pub fn label(mut self, s: impl Into<String>) -> Self { self.label = Some(s.into()); self }
    #[inline] pub fn anchor_origin(mut self, a: AnchorOrigin) -> Self { self.anchor_origin = Some(a); self }
    #[inline] pub fn transform_origin(mut self, t: TransformOrigin) -> Self { self.transform_origin = Some(t); self }
    #[inline] pub fn position(mut self, p: Position) -> Self { self.position = Some(p); self }
    #[inline] pub fn right_to_left(mut self, b: bool) -> Self { self.right_to_left = b; self }
    #[inline] pub fn full_width(mut self, b: bool) -> Self { self.full_width = b; self }
    #[inline] pub fn relative_width(mut self, b: bool) -> Self { self.relative_width = b; self }
    #[inline] pub fn max_height(mut self, px: f32) -> Self { self.max_height = Some(px); self }
    #[inline] pub fn size_hint(mut self, v: Vec2) -> Self { self.size_hint = Some(v); self }
    #[inline] pub fn dense(mut self, b: bool) -> Self { self.dense = b; self }
    #[inline] pub fn disabled(mut self, b: bool) -> Self { self.disabled = b; self }
    #[inline] pub fn open_on_click(mut self, b: bool) -> Self { self.open_on_click = b; self }
    #[inline] pub fn open_on_hover(mut self, b: bool) -> Self { self.open_on_hover = b; self }
    #[inline] pub fn hover_delay_ms(mut self, ms: u32) -> Self { self.hover_delay_ms = ms; self }
    #[inline] pub fn close_on_content_click(mut self, b: bool) -> Self { self.close_on_content_click = b; self }
    #[inline] pub fn close_on_escape_key(mut self, b: bool) -> Self { self.close_on_escape_key = b; self }
    #[inline] pub fn disable_ripple(mut self, b: bool) -> Self { self.disable_ripple = b; self }
    #[inline] pub fn offset_x(mut self, px: f32) -> Self { self.offset_x = px; self }
    #[inline] pub fn offset_y(mut self, px: f32) -> Self { self.offset_y = px; self }
    #[inline] pub fn menu_class(mut self, s: impl Into<String>) -> Self { self.menu_class = Some(s.into()); self }
    #[inline] pub fn menu_list_class(mut self, s: impl Into<String>) -> Self { self.menu_list_class = Some(s.into()); self }
    #[inline] pub fn classes<S: AsRef<str>>(mut self, classes: impl IntoIterator<Item = S>) -> Self { self.base = self.base.classes(classes); self }
    #[inline] pub fn flavor(mut self, f: StyleFlavor) -> Self { self.flavor = f; self }

    // ── Rendering ──────────────────────────────────────────────────────────

    /// Draws the menu.
    ///
    /// `'a` belongs to the item list, not to `MudMenu` — it is introduced on
    /// this method and constrained by `add_items`'s parameter type. The
    /// struct itself carries no lifetime.
    pub fn show<'a, R>(
        mut self,
        ui: &mut Ui,
        state: &mut MenuState,
        nav: Option<&mut NavigationManager>,
        activator: impl FnOnce(&mut Ui) -> Response,
        add_items: impl FnOnce(&mut MenuUi<'a>) -> R,
    ) -> (MenuResponse, R) {
        // 1. Render the activator.
        let activator_resp = activator(ui);
        let mut action = MenuAction::None;

        if !self.disabled {
            if self.open_on_click && activator_resp.clicked() {
                let was_open = state.is_open();
                state.toggle();
                action = if was_open {
                    MenuAction::Closed
                } else {
                    MenuAction::Opened
                };
            } else if self.open_on_hover && activator_resp.hovered() && !state.is_open() {
                state.open();
                action = MenuAction::Opened;
            }
        }

        // 2. Collect items.
        let mut collector = MenuUi { entries: Vec::new() };
        let user_result = add_items(&mut collector);
        let mut entries = collector.entries;

        // 3. Placement.
        let (anchor_origin, transform_origin) = match self.position {
            Some(p) => position_to_origins(p, self.right_to_left),
            None => (
                self.anchor_origin.unwrap_or(AnchorOrigin::Bottom),
                self.transform_origin.unwrap_or(TransformOrigin::TopLeft),
            ),
        };
        let (anchor_origin, offset) = apply_offsets(anchor_origin, self.offset_x, self.offset_y);

        // 4. Size hint.
        let size_hint = self.size_hint.unwrap_or_else(|| {
            let h = entries
                .iter()
                .map(|e| match e {
                    MenuEntry::Item(_) => item_height(self.dense),
                    MenuEntry::Separator(_) => SEPARATOR_HEIGHT + 2.0 * SEPARATOR_MARGIN_Y,
                })
                .sum::<f32>();
            vec2(DEFAULT_MENU_SIZE_HINT.x, h.max(40.0))
        });

        // 5. Build popover.
        let popover = MudPopover::new()
            .anchor_origin(anchor_origin)
            .transform_origin(transform_origin)
            .offset(offset)
            .right_to_left(self.right_to_left)
            .match_anchor_width(self.full_width || self.relative_width)
            .size_hint(size_hint)
            .dismiss_on_escape(self.close_on_escape_key)
            .paper(true)
            .elevation(8);

        // 6. Pre-compute everything OUTSIDE the popover closure. This is the
        //    key change: no `&mut self` and no `&mut entries` survives into
        //    the closure besides `inner_action` and `nav_local`, both declared
        //    locally.
        let theme = MudTheme::fetch(ui.ctx()).unwrap_or_default();
        let palette = theme.current_palette();
        let text_primary: Color32 = palette.text_primary.into();
        let text_secondary: Color32 = palette.text_secondary.into();
        let text_disabled: Color32 = palette.text_disabled.into();
        let hover_fill = ui.visuals().widgets.hovered.weak_bg_fill;
        let divider: Color32 = palette.divider.into();
        let menu_base_id = state.popover.id.with("items");
        let dense = self.dense;
        let disabled = self.disabled;
        let close_on_click = self.close_on_content_click;
        let max_height = self.max_height;
        let menu_list_class = self.menu_list_class.clone();
        let activator_rect = activator_resp.rect;

        // Pre-compute the list frame before the popover closure runs, so the
        // closure does not capture `&mut self.bs_style` / `&mut self.tw_style`.
        let list_frame = menu_list_class.as_deref().and_then(|c| {
            resolve_list_frame(
                c,
                &palette,
                &mut self.bs_style,
                &mut self.tw_style,
                self.flavor,
                self.tw_state,
            )
        });

        // 7. Render.
        let mut inner_action = MenuAction::None;
        let mut nav_local = nav;
        let (pop_resp, _) = popover.show(
            ui,
            &mut state.popover,
            activator_rect,
            |pop_ui| {
                let mut render_items =
                    |ui: &mut Ui, entries: &mut Vec<MenuEntry<'a>>| {
                        ui.spacing_mut().item_spacing.y = 0.0;
                        for (i, entry) in entries.iter_mut().enumerate() {
                            match entry {
                                MenuEntry::Item(item) => {
                                    render_item(
                                        ui,
                                        item,
                                        &menu_base_id.with(("item", i)),
                                        dense,
                                        disabled,
                                        text_primary,
                                        text_secondary,
                                        text_disabled,
                                        hover_fill,
                                        &palette,
                                        &mut nav_local,
                                        &mut inner_action,
                                    );
                                }
                                MenuEntry::Separator(_) => {
                                    ui.add_space(SEPARATOR_MARGIN_Y);
                                    let sep_rect = ui
                                        .allocate_exact_size(
                                            vec2(ui.available_width(), SEPARATOR_HEIGHT),
                                            Sense::hover(),
                                        )
                                        .0;
                                    ui.painter().rect_filled(sep_rect, 0.0, divider);
                                    ui.add_space(SEPARATOR_MARGIN_Y);
                                }
                            }
                        }
                    };

                let mut list_closure = |ui: &mut Ui| {
                    render_items(ui, &mut entries);
                };

                // `list_closure` is a FnMut, so it is borrowed (`|ui| list_closure(ui)`)
                // in each arm rather than moved. Only one arm runs, so the
                // shared mutable borrow is fine.
                match (max_height, list_frame) {
                    (Some(mh), Some(lf)) => {
                        egui::ScrollArea::vertical()
                            .max_height(mh)
                            .auto_shrink([false, false])
                            .show(pop_ui, |ui| lf.show(ui, |ui| list_closure(ui)));
                    }
                    (Some(mh), None) => {
                        egui::ScrollArea::vertical()
                            .max_height(mh)
                            .auto_shrink([false, false])
                            .show(pop_ui, |ui| list_closure(ui));
                    }
                    (None, Some(lf)) => {
                        lf.show(pop_ui, |ui| list_closure(ui));
                    }
                    (None, None) => {
                        list_closure(pop_ui);
                    }
                }
            },
        );

        // 8. Close on item activation.
        if inner_action.activated() && close_on_click {
            state.close();
        }

        // 9. Resolve the final action. `matches!(&inner_action, …)` borrows
        //    instead of moving, so `inner_action` is still available for the
        //    `if` branch below.
        let final_action = if !matches!(&inner_action, MenuAction::None) {
            inner_action
        } else {
            action
        };

        (
            MenuResponse {
                activator: activator_resp,
                action: final_action,
                popover: Some(pop_resp),
            },
            user_result,
        )
    }
}

/// Renders one clickable item and reports (via `action`) whether it was
/// activated.
#[allow(clippy::too_many_arguments)]
fn render_item<'a>(
    ui: &mut Ui,
    item: &mut MenuItem<'a>,
    id: &Id,
    dense: bool,
    menu_disabled: bool,
    text_primary: Color32,
    text_secondary: Color32,
    text_disabled: Color32,
    hover_fill: Color32,
    palette: &Palette,
    nav: &mut Option<&mut NavigationManager>,
    action: &mut MenuAction,
) {
    let effective_disabled = menu_disabled || item.disabled;
    let height = item_height(dense);

    let (rect, resp) = ui.allocate_exact_size(
        vec2(ui.available_width(), height),
        if effective_disabled {
            Sense::hover()
        } else {
            Sense::click()
        },
    );
    let _ = id;

    let hovered = resp.hovered() && !effective_disabled;
    if hovered {
        ui.painter()
            .rect_filled(rect, CornerRadius::same(PAPER_RADIUS), hover_fill);
        ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
    }

    let text_color = if effective_disabled {
        text_disabled
    } else {
        text_primary
    };

    // Take mutable fields out so `&item` can be read for icon/text while the
    // child closure mutably uses its own buffer.
    let mut child_opt = item.child.take();
    let mut on_click_opt = item.on_click.take();
    let text_opt = item.text.clone();
    let icon_opt = item.icon.clone();
    let icon_color = resolve_icon_color(item.icon_color, palette, text_secondary);

    // Layout: padding + [icon] + [text OR child].
    let inner_rect = Rect::from_min_max(
        pos2(rect.left() + ITEM_PADDING_X, rect.top()),
        pos2(rect.right() - ITEM_PADDING_X, rect.bottom()),
    );
    let mut cursor_x = inner_rect.left();
    let center_y = rect.center().y;
    let icon_px = height * 0.5;

    if let Some(icon) = icon_opt {
        let icon_rect = Rect::from_center_size(
            pos2(cursor_x + icon_px * 0.5, center_y),
            vec2(icon_px, icon_px),
        );
        ui.put(
            icon_rect,
            MudIcon::new(icon).size_px(icon_px).custom_color(icon_color),
        );
        cursor_x += icon_px + ITEM_ICON_GAP;
    }

    let content_rect = Rect::from_min_max(pos2(cursor_x, inner_rect.top()), inner_rect.max);
    if let Some(child) = child_opt.as_mut() {
        let mut child_ui = ui.new_child(
            UiBuilder::new()
                .max_rect(content_rect)
                .layout(Layout::left_to_right(egui::Align::Center)),
        );
        child_ui.set_clip_rect(content_rect);
        child(&mut child_ui);
    } else if let Some(text) = text_opt.as_deref() {
        let mut text_ui = ui.new_child(
            UiBuilder::new()
                .max_rect(content_rect)
                .layout(Layout::left_to_right(egui::Align::Center)),
        );
        text_ui.set_clip_rect(content_rect);
        text_ui.add(
            MudText::new(text.to_owned())
                .typo(Typo::Body2)
                .custom_color(text_color),
        );
    }

    let label = text_opt.unwrap_or_default();
    let enabled = !effective_disabled;
    resp.clone()
        .widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, enabled, label.as_str()));

    if !resp.clicked() || effective_disabled {
        return;
    }

    if let Some(cb) = on_click_opt.as_mut() {
        cb();
    }

    let href = item.href.clone();
    if let Some(href) = href {
        match nav.as_deref_mut() {
            Some(manager) => {
                let new_tab = matches!(item.target.as_deref(), Some("_blank"));
                if item.force_load {
                    let url = manager.to_absolute(&href);
                    ui.ctx().open_url(OpenUrl { url, new_tab });
                } else if is_external_url(&href, manager.base_uri()) {
                    ui.ctx().open_url(OpenUrl {
                        url: href.clone(),
                        new_tab,
                    });
                } else {
                    let _ = manager.navigate_to(&href, false);
                }
            }
            None => {
                ui.ctx().open_url(OpenUrl {
                    url: href.clone(),
                    new_tab: matches!(item.target.as_deref(), Some("_blank")),
                });
            }
        }
    }

    *action = MenuAction::ItemActivated {
        label: item.text.clone(),
        href: item.href.clone(),
    };
}

fn resolve_list_frame(
    class: &str,
    palette: &Palette,
    bs: &mut BsStyle,
    tw: &mut TwStyle,
    flavor: StyleFlavor,
    tw_state: TwState,
) -> Option<egui::Frame> {
    let class = class.trim();
    if class.is_empty() {
        return None;
    }
    let base = ComponentBase::new().classes(class.split_whitespace());
    let lookup = |key: &str| palette.resolve(key).map(Color32::from);
    Some(match flavor {
        StyleFlavor::Bootstrap => bs.build_frame(&base, &lookup),
        StyleFlavor::Tailwind => tw.build_frame(&base, tw_state, REM_SIZE_PT, &lookup),
    })
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ── item_height ────────────────────────────────────────────────────────

    #[test]
    fn item_height_normal_and_dense() {
        assert_eq!(item_height(false), ITEM_HEIGHT);
        assert_eq!(item_height(true), ITEM_HEIGHT - 8.0);
    }

    // ── position_to_origins ───────────────────────────────────────────────

    #[test]
    fn position_bottom_is_default() {
        let (a, t) = position_to_origins(Position::Bottom, false);
        assert_eq!(a, AnchorOrigin::Bottom);
        assert_eq!(t, TransformOrigin::TopLeft);
    }

    #[test]
    fn position_top_flips() {
        let (a, t) = position_to_origins(Position::Top, false);
        assert_eq!(a, AnchorOrigin::Top);
        assert_eq!(t, TransformOrigin::BottomLeft);
    }

    #[test]
    fn position_left_and_right() {
        let (a, t) = position_to_origins(Position::Left, false);
        assert_eq!(a, AnchorOrigin::Left);
        assert_eq!(t, TransformOrigin::CenterRight);

        let (a, t) = position_to_origins(Position::Right, false);
        assert_eq!(a, AnchorOrigin::Right);
        assert_eq!(t, TransformOrigin::CenterLeft);
    }

    #[test]
    fn position_start_resolves_under_rtl() {
        let (a, _) = position_to_origins(Position::Start, false);
        assert_eq!(a, AnchorOrigin::Left);
        let (a, _) = position_to_origins(Position::Start, true);
        assert_eq!(a, AnchorOrigin::Right);
    }

    // ── resolve_icon_color ─────────────────────────────────────────────────

    #[test]
    fn resolve_icon_color_fallback_and_explicit() {
        let theme = MudTheme::default();
        let palette = theme.current_palette();
        let fallback = Color32::from_rgb(1, 2, 3);
        assert_eq!(resolve_icon_color(None, palette, fallback), fallback);
        assert_eq!(
            resolve_icon_color(Some(Color::Default), palette, fallback),
            fallback
        );
        let error: Color32 = palette.error.into();
        assert_eq!(
            resolve_icon_color(Some(Color::Error), palette, fallback),
            error
        );
    }

    // ── apply_offsets ──────────────────────────────────────────────────────

    #[test]
    fn apply_offsets_picks_anchor_axis() {
        let (a, o) = apply_offsets(AnchorOrigin::Bottom, 4.0, 12.0);
        assert_eq!(a, AnchorOrigin::Bottom);
        assert_eq!(o, 12.0);
        let (a, o) = apply_offsets(AnchorOrigin::Right, 12.0, 4.0);
        assert_eq!(a, AnchorOrigin::Right);
        assert_eq!(o, 12.0);
    }

    // ── Defaults ───────────────────────────────────────────────────────────

    #[test]
    fn defaults_match_mudblazor() {
        let m: MudMenu = MudMenu::new();
        assert!(m.icon.is_none());
        assert!(m.label.is_none());
        assert!(m.anchor_origin.is_none());
        assert!(m.transform_origin.is_none());
        assert!(m.position.is_none());
        assert!(!m.right_to_left);
        assert!(!m.full_width);
        assert!(!m.relative_width);
        assert!(m.max_height.is_none());
        assert!(m.size_hint.is_none());
        assert!(!m.dense);
        assert!(!m.disabled);
        assert!(m.open_on_click);
        assert!(!m.open_on_hover);
        assert_eq!(m.hover_delay_ms, 100);
        assert!(m.close_on_content_click);
        assert!(m.close_on_escape_key);
        assert!(!m.disable_ripple);
        assert_eq!(m.offset_x, 0.0);
        assert_eq!(m.offset_y, 0.0);
        assert!(m.menu_class.is_none());
        assert!(m.menu_list_class.is_none());
    }

    // ── Builders ───────────────────────────────────────────────────────────

    #[test]
    fn builders_set_every_field() {
        let m = MudMenu::new()
            .icon("M0 0")
            .label("Open")
            .anchor_origin(AnchorOrigin::Top)
            .transform_origin(TransformOrigin::BottomLeft)
            .position(Position::Top)
            .right_to_left(true)
            .full_width(true)
            .relative_width(true)
            .max_height(300.0)
            .size_hint(vec2(240.0, 180.0))
            .dense(true)
            .disabled(true)
            .open_on_click(false)
            .open_on_hover(true)
            .hover_delay_ms(200)
            .close_on_content_click(false)
            .close_on_escape_key(false)
            .disable_ripple(true)
            .offset_x(4.0)
            .offset_y(8.0)
            .menu_class("menu-a")
            .menu_list_class("menu-b")
            .classes(["wrap"]);

        assert_eq!(m.icon.as_deref(), Some("M0 0"));
        assert_eq!(m.label.as_deref(), Some("Open"));
        assert_eq!(m.anchor_origin, Some(AnchorOrigin::Top));
        assert_eq!(m.transform_origin, Some(TransformOrigin::BottomLeft));
        assert_eq!(m.position, Some(Position::Top));
        assert!(m.right_to_left);
        assert!(m.full_width && m.relative_width);
        assert_eq!(m.max_height, Some(300.0));
        assert_eq!(m.size_hint, Some(vec2(240.0, 180.0)));
        assert!(m.dense && m.disabled);
        assert!(!m.open_on_click);
        assert!(m.open_on_hover);
        assert_eq!(m.hover_delay_ms, 200);
        assert!(!m.close_on_content_click);
        assert!(!m.close_on_escape_key);
        assert!(m.disable_ripple);
        assert_eq!(m.offset_x, 4.0);
        assert_eq!(m.offset_y, 8.0);
        assert_eq!(m.menu_class.as_deref(), Some("menu-a"));
        assert_eq!(m.menu_list_class.as_deref(), Some("menu-b"));
        assert!(m.base.has_class("wrap"));
    }

    // ── MenuAction helpers ─────────────────────────────────────────────────

    #[test]
    fn action_helpers() {
        assert!(MenuAction::Opened.opened());
        assert!(MenuAction::Closed.closed());
        assert!(MenuAction::ItemActivated {
            label: None,
            href: None
        }
            .activated());
    }
}