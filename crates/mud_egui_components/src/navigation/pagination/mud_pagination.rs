//! `MudPagination` — a horizontal row of clickable page numbers with optional
//! navigation controls.
//!
//! Port of `MudPagination.razor.cs` (MudBlazor 8.13.0).
//!
//! # MudBlazor parity
//!
//! | MudBlazor parameter | Rust method | Default |
//! |---------------------|-------------|---------|
//! | `BeforeIcon`        | [`MudPagination::before_icon`] | `NAVIGATE_BEFORE` |
//! | `BoundaryCount`     | [`MudPagination::boundary_count`] | `1` |
//! | `Color`             | [`MudPagination::color`] | `Primary` |
//! | `ControlButtonClicked` | [`PaginationResponse::action`] / [`MudPagination::on_control_clicked`] | — |
//! | `Count`             | [`MudPagination::count`] | `1` |
//! | `Disabled`          | [`MudPagination::disabled`] | `false` |
//! | `DropShadow`        | [`MudPagination::drop_shadow`] | `true` |
//! | `FirstIcon`         | [`MudPagination::first_icon`] | `FIRST_PAGE` |
//! | `LastIcon`          | [`MudPagination::last_icon`] | `LAST_PAGE` |
//! | `MiddleCount`       | [`MudPagination::middle_count`] | `1` |
//! | `NextIcon`          | [`MudPagination::next_icon`] | `NAVIGATE_NEXT` |
//! | `Rectangular`       | [`MudPagination::rectangular`] | `false` |
//! | `RightToLeft`       | [`MudPagination::right_to_left`] | `false` |
//! | `Selected`          | [`PaginationState::selected`] | `1` |
//! | `SelectedChanged`   | [`MudPagination::on_selected_changed`] | — |
//! | `ShowFirstButton`   | [`MudPagination::show_first_button`] | `false` |
//! | `ShowLastButton`    | [`MudPagination::show_last_button`] | `false` |
//! | `ShowNextButton`    | [`MudPagination::show_next_button`] | `true` |
//! | `ShowPageButtons`   | [`MudPagination::show_page_buttons`] | `true` |
//! | `ShowPreviousButton`| [`MudPagination::show_previous_button`] | `true` |
//! | `Size`              | [`MudPagination::size`] | `Medium` |
//! | `Variant`           | [`MudPagination::variant`] | `Text` |
//! | `Class` (base)      | [`MudPagination::classes`] | `None` |
//! | `Style` (base)      | *(use `classes`)* | — |
//! | `Tag` (base)        | **not ported** (no HTML tag) | — |
//! | `UserAttributes`    | **not ported** | — |
//!
//! ### Extensions (not in MudBlazor)
//!
//! | Extension | Description |
//! |-----------|-------------|
//! | [`id_salt`](MudPagination::id_salt) | Stable interaction ids. |
//! | [`PaginationState`] | Caller-owned selection state. |
//! | [`PaginationResponse`] | Rich return value with the resolved action. |
//! | [`paginate_layout`] | Pure layout function, unit-tested. |
//!
//! ### Not ported
//!
//! * **`Tag` / `UserAttributes`** — no HTML tag.
//! * **`Style`** — no inline CSS strings; use `classes(..)`.
//! * **`NavigateToAsync(Page)` / `NavigateToAsync(int)`** — the caller owns
//!   [`PaginationState`], so navigation is a direct field write. See
//!   [`PaginationState`] for the 0-based/1-based rationale.
//!
//! # Variant semantics
//!
//! The **selected** page is always rendered with `Variant::Filled` (solid
//! accent fill). The **other** pages and the **controls** use
//! [`MudPagination::variant`], matching MudBlazor exactly — so `Variant::Text`
//! (the default) gives the familiar "only the selected page is filled"
//! look, while `Variant::Outlined` outlines the non-selected pages, and
//! `Variant::Filled` makes every page solid (a known MudBlazor quirk).
//!
//! # Layout algorithm
//!
//! [`paginate_layout`] returns one [`PaginateItem`] per visible slot. The
//! rules, matching MudBlazor's `PaginationState`:
//!
//! * The first `boundary_count` and last `boundary_count` pages are always
//!   shown.
//! * The selected page and its neighbours within `middle_count` are always
//!   shown.
//! * A gap of one or more hidden pages collapses into a single
//!   [`PaginateItem::Ellipsis`].
//!
//! | `count` | `boundary` | `middle` | `selected` | Result |
//! |---------|------------|----------|------------|--------|
//! | 9 | 1 | 1 | 5 | `1 … 5 … 9` |
//! | 9 | 1 | 3 | 5 | `1 … 4 5 6 … 9` |
//! | 9 | 2 | 1 | 5 | `1 2 … 5 … 8 9` |
//! | 9 | 2 | 3 | 5 | `1 2 … 4 5 6 … 8 9` |
//! | 3 | 1 | 1 | 2 | `1 2 3` (no ellipsis) |
//!
//! # Text and icons
//!
//! Page numbers render through [`MudText`]; control icons and the ellipsis
//! render through [`MudIcon`].

use egui::{
    pos2, vec2, Color32, CornerRadius, CursorIcon, Id, Rect, Response, Sense, Stroke,
    StrokeKind, Ui,
};
use mud_egui_bootstrap::bs_style::BsStyle;
use mud_egui_core::base::component_base::ComponentBase;
use mud_egui_core::enums::{Color, PageControl, Size, StyleFlavor, Variant};
use mud_egui_icons_material::filled;
use mud_egui_tailwind::tw_style::{TwState, TwStyle};
use mud_egui_theming::models::palettes::palette::Palette;
use mud_egui_theming::theme::MudTheme;

use crate::data_display::icons::mud_icon::MudIcon;
use crate::typography::mud_text::MudText;

use super::paginate_item::PaginateItem;
use super::pagination_action::PaginationAction;
use super::pagination_state::PaginationState;

// =============================================================================
// Constants
// =============================================================================

/// Base rem size passed to the Tailwind resolver.
const REM_SIZE_PT: f32 = 14.0;

/// Gap between buttons.
const BUTTON_GAP: f32 = 4.0;

/// Corner radius when `rectangular` is `true`.
const RECTANGULAR_RADIUS: u8 = 4;

/// Shadow blur for the drop-shadow on page buttons.
const DROP_SHADOW_BLUR: u8 = 6;

// =============================================================================
// Pure helpers
// =============================================================================

/// Side length of each button for a given [`Size`].
///
/// | Size | Pixels |
/// |------|--------|
/// | `Small` | 32 |
/// | `Medium` | 40 |
/// | `Large` | 48 |
#[inline]
pub fn button_size(size: Size) -> f32 {
    match size {
        Size::Small => 32.0,
        Size::Large => 48.0,
        _ => 40.0,
    }
}

/// Icon side length inside a control button.
#[inline]
pub fn icon_size(size: Size) -> f32 {
    match size {
        Size::Small => 16.0,
        Size::Large => 24.0,
        _ => 20.0,
    }
}

/// Font size of a page number label.
#[inline]
pub fn font_size(size: Size) -> f32 {
    match size {
        Size::Small => 12.0,
        Size::Large => 16.0,
        _ => 14.0,
    }
}

/// Resolves the accent colour from the pagination's `Color`.
#[inline]
pub fn resolve_color(color: Color, palette: &Palette) -> Color32 {
    match color {
        Color::Default | Color::Inherit => palette.primary.into(),
        other => other
            .resolve(palette)
            .unwrap_or_else(|| palette.primary.into()),
    }
}

/// Resolves the effective [`Variant`] of a button.
///
/// Selected page → `Filled` (always). Non-selected → `base`.
#[inline]
pub fn resolve_variant(selected: bool, base: Variant) -> Variant {
    if selected { Variant::Filled } else { base }
}

/// Computes the visible [`PaginateItem`] sequence.
///
/// See the module docs for the exact rules.
pub(crate) fn paginate_layout(
    count: u32,
    boundary_count: u32,
    middle_count: u32,
    selected: u32,
) -> Vec<PaginateItem> {
    let count = count.max(1);
    let selected = selected.clamp(1, count);
    // Guarantee at least one middle page so `selected` is never hidden.
    let middle_count = middle_count.max(1);

    let half = middle_count / 2;
    let middle_start = selected.saturating_sub(half).max(1);
    let middle_end = (middle_start + middle_count - 1).min(count);

    let mut items = Vec::new();
    let mut prev_was_hidden = false;

    for page in 1..=count {
        let in_head = page <= boundary_count;
        let in_tail = page > count.saturating_sub(boundary_count);
        let in_middle = page >= middle_start && page <= middle_end;

        if in_head || in_tail || in_middle {
            if prev_was_hidden {
                items.push(PaginateItem::Ellipsis);
                prev_was_hidden = false;
            }
            items.push(PaginateItem::Page(page));
        } else {
            prev_was_hidden = true;
        }
    }

    items
}

/// Multiplies the alpha channel of `c` by `alpha`.
///
/// * `alpha >= 1.0` → returns `c` unchanged (identity).
/// * `alpha <= 0.0` → returns [`Color32::TRANSPARENT`].
/// * Otherwise the result is `c`'s RGB with `c.a() * alpha` as its new alpha.
///
/// This matches the semantics of `MudChip::with_alpha` so all components
/// interpret the same call the same way. Callers pass a **multiplier**
/// (`0.0..=1.0`), not a raw `u8` alpha value.
#[inline]
fn with_alpha(c: Color32, alpha: f32) -> Color32 {
    if alpha >= 1.0 {
        return c;
    }
    if alpha <= 0.0 {
        return Color32::TRANSPARENT;
    }
    Color32::from_rgba_unmultiplied(
        c.r(),
        c.g(),
        c.b(),
        (c.a() as f32 * alpha).round() as u8,
    )
}

// =============================================================================
// Response
// =============================================================================

/// The result of one [`MudPagination::show`] frame.
#[derive(Debug)]
pub struct PaginationResponse {
    /// The wrapper's own `egui::Response`.
    pub response: Response,
    /// The page state **after** the click was applied (if any).
    pub selected: u32,
    /// What the user clicked this frame.
    pub action: PaginationAction,
}

// =============================================================================
// MudPagination
// =============================================================================

/// A horizontal row of clickable page numbers with optional navigation
/// controls.
#[must_use = "MudPagination does nothing until you call `.show(ui, state)`"]
pub struct MudPagination<'a> {
    // ── Icons ──────────────────────────────────────────────────────────────
    /// Icon for the "go to first page" control (`FirstIcon`).
    pub first_icon: String,
    /// Icon for the "previous page" control (`BeforeIcon`).
    pub before_icon: String,
    /// Icon for the "next page" control (`NextIcon`).
    pub next_icon: String,
    /// Icon for the "go to last page" control (`LastIcon`).
    pub last_icon: String,

    // ── Layout ─────────────────────────────────────────────────────────────
    /// Number of page numbers always shown at each end (`BoundaryCount`).
    pub boundary_count: u32,
    /// Number of page numbers shown around the selected page (`MiddleCount`).
    pub middle_count: u32,
    /// Square buttons instead of round (`Rectangular`).
    pub rectangular: bool,
    /// Drop shadow behind each button (`DropShadow`).
    pub drop_shadow: bool,
    /// Renders the row right-to-left (`RightToLeft`).
    pub right_to_left: bool,

    // ── Visibility ─────────────────────────────────────────────────────────
    /// Show the "first page" control (`ShowFirstButton`, default `false`).
    pub show_first_button: bool,
    /// Show the "last page" control (`ShowLastButton`, default `false`).
    pub show_last_button: bool,
    /// Show the "next page" control (`ShowNextButton`, default `true`).
    pub show_next_button: bool,
    /// Show the "previous page" control (`ShowPreviousButton`, default `true`).
    pub show_previous_button: bool,
    /// Show the numeric page buttons (`ShowPageButtons`, default `true`).
    pub show_page_buttons: bool,

    // ── Behaviour ──────────────────────────────────────────────────────────
    /// Total number of pages (`Count`, default `1`).
    pub count: u32,
    /// Prevents every button from responding (`Disabled`, default `false`).
    pub disabled: bool,

    // ── Appearance ─────────────────────────────────────────────────────────
    /// Accent colour (`Color`, default `Primary`).
    pub color: Color,
    /// Button size (`Size`, default `Medium`).
    pub size: Size,
    /// Base variant for non-selected buttons (`Variant`, default `Text`).
    pub variant: Variant,

    // ── Extensions ─────────────────────────────────────────────────────────
    /// Stable interaction id salt.
    pub id_salt: Option<String>,
    /// Fires after the selection changes.
    pub on_selected_changed: Option<Box<dyn FnMut(u32) + 'a>>,
    /// Fires when a control button is clicked.
    pub on_control_clicked: Option<Box<dyn FnMut(PageControl) + 'a>>,

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

impl<'a> Default for MudPagination<'a> {
    fn default() -> Self {
        Self {
            first_icon: filled::FIRST_PAGE.to_owned(),
            before_icon: filled::NAVIGATE_BEFORE.to_owned(),
            next_icon: filled::NAVIGATE_NEXT.to_owned(),
            last_icon: filled::LAST_PAGE.to_owned(),
            boundary_count: 1,
            middle_count: 1,
            rectangular: false,
            drop_shadow: true,
            right_to_left: false,
            show_first_button: false,
            show_last_button: false,
            show_next_button: true,
            show_previous_button: true,
            show_page_buttons: true,
            count: 1,
            disabled: false,
            color: Color::Primary,
            size: Size::Medium,
            variant: Variant::Text,
            id_salt: None,
            on_selected_changed: None,
            on_control_clicked: None,
            base: ComponentBase::new(),
            flavor: StyleFlavor::default(),
            bs_style: BsStyle::default(),
            tw_style: TwStyle::default(),
            tw_state: TwState::default(),
        }
    }
}

impl<'a> MudPagination<'a> {
    /// Creates a pagination bar with MudBlazor defaults and the given page
    /// count.
    pub fn new(count: u32) -> Self {
        Self {
            count: count.max(1),
            ..Default::default()
        }
    }

    // ── Icons ──────────────────────────────────────────────────────────────

    #[inline]
    pub fn first_icon(mut self, svg: impl Into<String>) -> Self {
        self.first_icon = svg.into();
        self
    }

    #[inline]
    pub fn before_icon(mut self, svg: impl Into<String>) -> Self {
        self.before_icon = svg.into();
        self
    }

    #[inline]
    pub fn next_icon(mut self, svg: impl Into<String>) -> Self {
        self.next_icon = svg.into();
        self
    }

    #[inline]
    pub fn last_icon(mut self, svg: impl Into<String>) -> Self {
        self.last_icon = svg.into();
        self
    }

    // ── Layout ─────────────────────────────────────────────────────────────

    #[inline]
    pub fn boundary_count(mut self, n: u32) -> Self {
        self.boundary_count = n;
        self
    }

    #[inline]
    pub fn middle_count(mut self, n: u32) -> Self {
        self.middle_count = n;
        self
    }

    #[inline]
    pub fn rectangular(mut self, b: bool) -> Self {
        self.rectangular = b;
        self
    }

    #[inline]
    pub fn drop_shadow(mut self, b: bool) -> Self {
        self.drop_shadow = b;
        self
    }

    #[inline]
    pub fn right_to_left(mut self, b: bool) -> Self {
        self.right_to_left = b;
        self
    }

    // ── Visibility ─────────────────────────────────────────────────────────

    #[inline]
    pub fn show_first_button(mut self, b: bool) -> Self {
        self.show_first_button = b;
        self
    }

    #[inline]
    pub fn show_last_button(mut self, b: bool) -> Self {
        self.show_last_button = b;
        self
    }

    #[inline]
    pub fn show_next_button(mut self, b: bool) -> Self {
        self.show_next_button = b;
        self
    }

    #[inline]
    pub fn show_previous_button(mut self, b: bool) -> Self {
        self.show_previous_button = b;
        self
    }

    #[inline]
    pub fn show_page_buttons(mut self, b: bool) -> Self {
        self.show_page_buttons = b;
        self
    }

    // ── Behaviour ──────────────────────────────────────────────────────────

    #[inline]
    pub fn count(mut self, n: u32) -> Self {
        self.count = n.max(1);
        self
    }

    #[inline]
    pub fn disabled(mut self, b: bool) -> Self {
        self.disabled = b;
        self
    }

    // ── Appearance ─────────────────────────────────────────────────────────

    #[inline]
    pub fn color(mut self, c: Color) -> Self {
        self.color = c;
        self
    }

    #[inline]
    pub fn size(mut self, s: Size) -> Self {
        self.size = s;
        self
    }

    #[inline]
    pub fn variant(mut self, v: Variant) -> Self {
        self.variant = v;
        self
    }

    // ── Extensions ─────────────────────────────────────────────────────────

    #[inline]
    pub fn id_salt(mut self, s: impl Into<String>) -> Self {
        self.id_salt = Some(s.into());
        self
    }

    #[inline]
    pub fn on_selected_changed(mut self, f: impl FnMut(u32) + 'a) -> Self {
        self.on_selected_changed = Some(Box::new(f));
        self
    }

    #[inline]
    pub fn on_control_clicked(mut self, f: impl FnMut(PageControl) + 'a) -> Self {
        self.on_control_clicked = Some(Box::new(f));
        self
    }

    // ── Utilities ──────────────────────────────────────────────────────────

    #[inline]
    pub fn classes<S: AsRef<str>>(mut self, classes: impl IntoIterator<Item = S>) -> Self {
        self.base = self.base.classes(classes);
        self
    }

    #[inline]
    pub fn flavor(mut self, f: StyleFlavor) -> Self {
        self.flavor = f;
        self
    }

    // ── Rendering ──────────────────────────────────────────────────────────

    /// Draws the pagination bar.
    ///
    /// `state` is mutated in place: the click, when it resolves to a page
    /// change, updates `state.selected`. The response reports what was
    /// clicked through [`PaginationAction`].
    pub fn show(mut self, ui: &mut Ui, state: &mut PaginationState) -> PaginationResponse {
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

        // 2. Resolve layout.
        let count = self.count.max(1);
        let selected = state.selected.clamp(1, count);
        state.selected = selected;

        let items = if self.show_page_buttons {
            paginate_layout(count, self.boundary_count, self.middle_count, selected)
        } else {
            Vec::new()
        };

        let accent = resolve_color(self.color, &palette);
        let button_px = button_size(self.size);
        let icon_px = icon_size(self.size);
        let font_px = font_size(self.size);
        let radius = if self.rectangular {
            CornerRadius::same(RECTANGULAR_RADIUS)
        } else {
            CornerRadius::same((button_px * 0.5) as u8)
        };

        let scope_id = ui.scope_id();
        let base_id = match self.id_salt.as_deref() {
            Some(s) => scope_id.with(s),
            None => scope_id.with("mud_pagination"),
        };

        let mut action = PaginationAction::None;
        let mut new_selected = state.selected;

        // 3. Render row.
        let layout = if self.right_to_left {
            egui::Layout::right_to_left(egui::Align::Center)
        } else {
            egui::Layout::left_to_right(egui::Align::Center)
        };

        let inner = frame.show(ui, |ui| {
            ui.with_layout(layout, |ui| {
                ui.spacing_mut().item_spacing.x = BUTTON_GAP;

                let disabled = self.disabled;

                // 3a. First control.
                if self.show_first_button {
                    let clicked = render_control(
                        ui,
                        &self.first_icon,
                        icon_px,
                        button_px,
                        radius,
                        accent,
                        palette,
                        self.variant,
                        self.drop_shadow,
                        disabled,
                    );
                    if clicked {
                        action = PaginationAction::Control(PageControl::First);
                        new_selected = 1;
                    }
                }

                // 3b. Previous control.
                if self.show_previous_button {
                    let clicked = render_control(
                        ui,
                        &self.before_icon,
                        icon_px,
                        button_px,
                        radius,
                        accent,
                        palette,
                        self.variant,
                        self.drop_shadow,
                        disabled,
                    );
                    if clicked {
                        action = PaginationAction::Control(PageControl::Previous);
                        new_selected = selected.saturating_sub(1).max(1);
                    }
                }

                // 3c. Page buttons + ellipses.
                for item in items.iter() {
                    match item {
                        PaginateItem::Page(n) => {
                            let is_selected = *n == selected;
                            let clicked = render_page_button(
                                ui,
                                *n,
                                is_selected,
                                font_px,
                                button_px,
                                radius,
                                accent,
                                palette,
                                self.variant,
                                self.drop_shadow,
                                disabled,
                            );
                            if clicked && !is_selected {
                                action = PaginationAction::Page(*n);
                                new_selected = *n;
                            }
                        }
                        PaginateItem::Ellipsis => {
                            render_ellipsis(ui, icon_px, button_px, palette);
                        }
                    }
                }

                // 3d. Next control.
                if self.show_next_button {
                    let clicked = render_control(
                        ui,
                        &self.next_icon,
                        icon_px,
                        button_px,
                        radius,
                        accent,
                        palette,
                        self.variant,
                        self.drop_shadow,
                        disabled,
                    );
                    if clicked {
                        action = PaginationAction::Control(PageControl::Next);
                        new_selected = (selected + 1).min(count);
                    }
                }

                // 3e. Last control.
                if self.show_last_button {
                    let clicked = render_control(
                        ui,
                        &self.last_icon,
                        icon_px,
                        button_px,
                        radius,
                        accent,
                        palette,
                        self.variant,
                        self.drop_shadow,
                        disabled,
                    );
                    if clicked {
                        action = PaginationAction::Control(PageControl::Last);
                        new_selected = count;
                    }
                }
            })
                .response
        });

        // 4. Apply selection change + dispatch callbacks.
        let changed = new_selected != state.selected;
        state.selected = new_selected;

        if changed {
            if let Some(cb) = self.on_selected_changed.as_mut() {
                cb(state.selected);
            }
        }
        if let PaginationAction::Control(control) = action {
            if let Some(cb) = self.on_control_clicked.as_mut() {
                cb(control);
            }
        }

        PaginationResponse {
            response: inner.inner,
            selected: state.selected,
            action,
        }
    }
}

// =============================================================================
// Render helpers
// =============================================================================

/// Paints a control button (first / previous / next / last).
///
/// Returns `true` when the button was clicked this frame.
#[allow(clippy::too_many_arguments)]
fn render_control(
    ui: &mut Ui,
    icon_svg: &str,
    icon_px: f32,
    button_px: f32,
    radius: CornerRadius,
    accent: Color32,
    palette: &Palette,
    variant: Variant,
    drop_shadow: bool,
    disabled: bool,
) -> bool {
    let sense = if disabled { Sense::hover() } else { Sense::click() };
    let (rect, response) = ui.allocate_exact_size(vec2(button_px, button_px), sense);
    let hovered = response.hovered() && !disabled;

    paint_button_bg(
        ui,
        rect,
        radius,
        accent,
        variant,
        hovered,
        disabled,
        drop_shadow,
        palette,
    );

    // Icon colour: accent for Text/Outlined, contrast for Filled.
    let icon_color = if disabled {
        palette.text_disabled.into()
    } else {
        match variant {
            Variant::Filled => palette.primary_contrast_text.into(),
            _ => accent,
        }
    };
    let icon_rect = Rect::from_center_size(rect.center(), vec2(icon_px, icon_px));
    ui.put(
        icon_rect,
        MudIcon::new(icon_svg.to_owned())
            .size_px(icon_px)
            .custom_color(icon_color),
    );

    if hovered {
        ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
    }

    response.clicked() && !disabled
}

/// Paints a page-number button.
///
/// Returns `true` when the button was clicked this frame.
#[allow(clippy::too_many_arguments)]
fn render_page_button(
    ui: &mut Ui,
    page: u32,
    is_selected: bool,
    font_px: f32,
    button_px: f32,
    radius: CornerRadius,
    accent: Color32,
    palette: &Palette,
    base_variant: Variant,
    drop_shadow: bool,
    disabled: bool,
) -> bool {
    let sense = if disabled { Sense::hover() } else { Sense::click() };
    let (rect, response) = ui.allocate_exact_size(vec2(button_px, button_px), sense);
    let hovered = response.hovered() && !disabled;

    let effective_variant = resolve_variant(is_selected, base_variant);
    paint_button_bg(
        ui,
        rect,
        radius,
        accent,
        effective_variant,
        hovered,
        disabled,
        drop_shadow && is_selected, // shadow only on the selected page
        palette,
    );

    // Text colour: contrast for Filled, accent for Text/Outlined.
    let text_color = if disabled {
        palette.text_disabled.into()
    } else {
        match effective_variant {
            Variant::Filled => palette.primary_contrast_text.into(),
            _ => accent,
        }
    };

    let text_rect = Rect::from_center_size(rect.center(), vec2(button_px, font_px * 2.0));
    let mut text_ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(text_rect)
            .layout(egui::Layout::centered_and_justified(
                egui::Direction::TopDown,
            )),
    );
    text_ui.add(
        MudText::new(page.to_string())
            .font_size(font_px)
            .custom_color(text_color)
            .inline(true)
            .no_wrap(true),
    );

    if hovered {
        ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
    }

    response.clicked() && !disabled
}

/// Paints the ellipsis between two page groups.
fn render_ellipsis(ui: &mut Ui, icon_px: f32, button_px: f32, palette: &Palette) {
    let (rect, _) = ui.allocate_exact_size(vec2(button_px, button_px), Sense::hover());
    let icon_rect = Rect::from_center_size(rect.center(), vec2(icon_px, icon_px));
    ui.put(
        icon_rect,
        MudIcon::new(filled::MORE_HORIZ.to_owned())
            .size_px(icon_px)
            .custom_color(palette.text_secondary.into()),
    );
}

/// Paints the background (and optional stroke / shadow) of a button.
#[allow(clippy::too_many_arguments)]
fn paint_button_bg(
    ui: &Ui,
    rect: Rect,
    radius: CornerRadius,
    accent: Color32,
    variant: Variant,
    hovered: bool,
    disabled: bool,
    drop_shadow: bool,
    palette: &Palette,
) {
    if drop_shadow && !disabled {
        let shadow_color = Color32::from_black_alpha((DROP_SHADOW_BLUR * 4).min(40));
        let shadow_rect = rect.translate(vec2(0.0, 1.0));
        ui.painter().rect_filled(shadow_rect, radius, shadow_color);
    }

    match variant {
        Variant::Filled => {
            let fill = if disabled {
                with_alpha(palette.action_disabled.into(), 0.12)
            } else if hovered {
                with_alpha(accent, 0.92)
            } else {
                accent
            };
            ui.painter().rect_filled(rect, radius, fill);
        }
        Variant::Outlined => {
            let stroke_color = if disabled {
                with_alpha(accent, 0.38)
            } else {
                accent
            };
            if hovered && !disabled {
                ui.painter()
                    .rect_filled(rect, radius, with_alpha(accent, 0.08));
            }
            ui.painter().rect_stroke(
                rect,
                radius,
                Stroke::new(1.0, stroke_color),
                StrokeKind::Inside,
            );
        }
        Variant::Text => {
            if hovered && !disabled {
                ui.painter()
                    .rect_filled(rect, radius, with_alpha(accent, 0.08));
            }
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // --- button_size ------------------------------------------------------

    #[test]
    fn button_size_maps_sizes() {
        assert_eq!(button_size(Size::Small), 32.0);
        assert_eq!(button_size(Size::Medium), 40.0);
        assert_eq!(button_size(Size::Large), 48.0);
        assert_eq!(button_size(Size::ExtraSmall), 40.0);
        assert_eq!(button_size(Size::ExtraLarge), 40.0);
    }

    #[test]
    fn icon_size_maps_sizes() {
        assert_eq!(icon_size(Size::Small), 16.0);
        assert_eq!(icon_size(Size::Medium), 20.0);
        assert_eq!(icon_size(Size::Large), 24.0);
    }

    #[test]
    fn font_size_maps_sizes() {
        assert_eq!(font_size(Size::Small), 12.0);
        assert_eq!(font_size(Size::Medium), 14.0);
        assert_eq!(font_size(Size::Large), 16.0);
    }

    // --- resolve_color ----------------------------------------------------

    #[test]
    fn resolve_color_default_and_inherit_use_primary() {
        let theme = MudTheme::default();
        let palette = theme.current_palette();
        let primary: Color32 = palette.primary.into();
        assert_eq!(resolve_color(Color::Default, palette), primary);
        assert_eq!(resolve_color(Color::Inherit, palette), primary);
    }

    #[test]
    fn resolve_color_explicit_uses_palette() {
        let theme = MudTheme::default();
        let palette = theme.current_palette();
        let error: Color32 = palette.error.into();
        assert_eq!(resolve_color(Color::Error, palette), error);
    }

    // --- resolve_variant --------------------------------------------------

    #[test]
    fn selected_always_uses_filled() {
        for base in [Variant::Text, Variant::Filled, Variant::Outlined] {
            assert_eq!(resolve_variant(true, base), Variant::Filled);
        }
    }

    #[test]
    fn unselected_uses_base() {
        assert_eq!(resolve_variant(false, Variant::Text), Variant::Text);
        assert_eq!(resolve_variant(false, Variant::Outlined), Variant::Outlined);
        assert_eq!(resolve_variant(false, Variant::Filled), Variant::Filled);
    }

    // --- with_alpha -------------------------------------------------------

    #[test]
    fn with_alpha_full_returns_same() {
        let c = Color32::from_rgb(100, 150, 200);
        assert_eq!(with_alpha(c, 1.0), c);
        assert_eq!(with_alpha(c, 1.5), c);
    }

    #[test]
    fn with_alpha_zero_returns_transparent() {
        let c = Color32::from_rgb(100, 150, 200);
        assert_eq!(with_alpha(c, 0.0), Color32::TRANSPARENT);
        assert_eq!(with_alpha(c, -1.0), Color32::TRANSPARENT);
    }

    #[test]
    fn with_alpha_scales_alpha_channel() {
        let c = Color32::from_rgba_unmultiplied(100, 150, 200, 200);
        let half = with_alpha(c, 0.5);
        assert_eq!(half.r(), 100);
        assert_eq!(half.g(), 150);
        assert_eq!(half.b(), 200);
        assert_eq!(half.a(), 100);
    }

    // --- paginate_layout --------------------------------------------------

    #[test]
    fn layout_shows_all_when_small() {
        let items = paginate_layout(5, 1, 1, 3);
        assert_eq!(
            items,
            vec![
                PaginateItem::Page(1),
                PaginateItem::Page(2),
                PaginateItem::Page(3),
                PaginateItem::Page(4),
                PaginateItem::Page(5),
            ]
        );
    }

    #[test]
    fn layout_standard_boundary1_middle1() {
        let items = paginate_layout(9, 1, 1, 5);
        assert_eq!(
            items,
            vec![
                PaginateItem::Page(1),
                PaginateItem::Ellipsis,
                PaginateItem::Page(5),
                PaginateItem::Ellipsis,
                PaginateItem::Page(9),
            ]
        );
    }

    #[test]
    fn layout_boundary1_middle3() {
        let items = paginate_layout(9, 1, 3, 5);
        assert_eq!(
            items,
            vec![
                PaginateItem::Page(1),
                PaginateItem::Ellipsis,
                PaginateItem::Page(4),
                PaginateItem::Page(5),
                PaginateItem::Page(6),
                PaginateItem::Ellipsis,
                PaginateItem::Page(9),
            ]
        );
    }

    #[test]
    fn layout_boundary2_middle1() {
        let items = paginate_layout(9, 2, 1, 5);
        assert_eq!(
            items,
            vec![
                PaginateItem::Page(1),
                PaginateItem::Page(2),
                PaginateItem::Ellipsis,
                PaginateItem::Page(5),
                PaginateItem::Ellipsis,
                PaginateItem::Page(8),
                PaginateItem::Page(9),
            ]
        );
    }

    #[test]
    fn layout_boundary2_middle3() {
        let items = paginate_layout(9, 2, 3, 5);
        assert_eq!(
            items,
            vec![
                PaginateItem::Page(1),
                PaginateItem::Page(2),
                PaginateItem::Ellipsis,
                PaginateItem::Page(4),
                PaginateItem::Page(5),
                PaginateItem::Page(6),
                PaginateItem::Ellipsis,
                PaginateItem::Page(8),
                PaginateItem::Page(9),
            ]
        );
    }

    #[test]
    fn layout_count_one() {
        assert_eq!(paginate_layout(1, 1, 1, 1), vec![PaginateItem::Page(1)]);
    }

    #[test]
    fn layout_count_zero_treated_as_one() {
        assert_eq!(paginate_layout(0, 1, 1, 1), vec![PaginateItem::Page(1)]);
    }

    #[test]
    fn layout_selected_clamped_high() {
        assert_eq!(
            paginate_layout(5, 1, 1, 99),
            vec![
                PaginateItem::Page(1),
                PaginateItem::Ellipsis,
                PaginateItem::Page(4),
                PaginateItem::Page(5),
            ]
        );
    }

    #[test]
    fn layout_selected_clamped_low() {
        let items = paginate_layout(5, 1, 1, 0);
        assert_eq!(items[0], PaginateItem::Page(1));
        assert_eq!(items[1], PaginateItem::Page(2));
    }

    #[test]
    fn layout_no_ellipsis_when_gap_is_zero() {
        let items = paginate_layout(5, 2, 1, 3);
        assert!(items.iter().all(|i| matches!(i, PaginateItem::Page(_))));
    }

    #[test]
    fn layout_middle_zero_treated_as_one() {
        let items = paginate_layout(5, 1, 0, 3);
        assert!(items.iter().any(|i| matches!(i, PaginateItem::Page(3))));
    }

    #[test]
    fn layout_ellipsis_only_when_gap_present() {
        let items = paginate_layout(9, 0, 1, 5);
        assert_eq!(items.first(), Some(&PaginateItem::Ellipsis));
        assert!(items.contains(&PaginateItem::Page(5)));
    }

    // --- Defaults ---------------------------------------------------------

    #[test]
    fn defaults_match_mudblazor() {
        let p = MudPagination::new(1);
        assert_eq!(p.first_icon, filled::FIRST_PAGE);
        assert_eq!(p.before_icon, filled::NAVIGATE_BEFORE);
        assert_eq!(p.next_icon, filled::NAVIGATE_NEXT);
        assert_eq!(p.last_icon, filled::LAST_PAGE);
        assert_eq!(p.boundary_count, 1);
        assert_eq!(p.middle_count, 1);
        assert!(!p.rectangular);
        assert!(p.drop_shadow);
        assert!(!p.right_to_left);
        assert!(!p.show_first_button);
        assert!(!p.show_last_button);
        assert!(p.show_next_button);
        assert!(p.show_previous_button);
        assert!(p.show_page_buttons);
        assert_eq!(p.count, 1);
        assert!(!p.disabled);
        assert!(matches!(p.color, Color::Primary));
        assert_eq!(p.size, Size::Medium);
        assert_eq!(p.variant, Variant::Text);
    }

    #[test]
    fn new_clamps_count_to_one() {
        assert_eq!(MudPagination::new(0).count, 1);
    }

    // --- Builders ---------------------------------------------------------

    #[test]
    fn builders_set_every_field() {
        let p = MudPagination::new(10)
            .first_icon("F")
            .before_icon("B")
            .next_icon("N")
            .last_icon("L")
            .boundary_count(2)
            .middle_count(3)
            .rectangular(true)
            .drop_shadow(false)
            .right_to_left(true)
            .show_first_button(true)
            .show_last_button(true)
            .show_next_button(false)
            .show_previous_button(false)
            .show_page_buttons(false)
            .count(20)
            .disabled(true)
            .color(Color::Error)
            .size(Size::Large)
            .variant(Variant::Outlined)
            .id_salt("pager")
            .on_selected_changed(|_| {})
            .on_control_clicked(|_| {})
            .classes(["my-pager"]);

        assert_eq!(p.first_icon, "F");
        assert_eq!(p.before_icon, "B");
        assert_eq!(p.next_icon, "N");
        assert_eq!(p.last_icon, "L");
        assert_eq!(p.boundary_count, 2);
        assert_eq!(p.middle_count, 3);
        assert!(p.rectangular);
        assert!(!p.drop_shadow);
        assert!(p.right_to_left);
        assert!(p.show_first_button);
        assert!(p.show_last_button);
        assert!(!p.show_next_button);
        assert!(!p.show_previous_button);
        assert!(!p.show_page_buttons);
        assert_eq!(p.count, 20);
        assert!(p.disabled);
        assert!(matches!(p.color, Color::Error));
        assert_eq!(p.size, Size::Large);
        assert_eq!(p.variant, Variant::Outlined);
        assert_eq!(p.id_salt.as_deref(), Some("pager"));
        assert!(p.on_selected_changed.is_some());
        assert!(p.on_control_clicked.is_some());
        assert!(p.base.has_class("my-pager"));
    }

    #[test]
    fn count_clamps_to_one() {
        let p = MudPagination::new(5).count(0);
        assert_eq!(p.count, 1);
    }
}