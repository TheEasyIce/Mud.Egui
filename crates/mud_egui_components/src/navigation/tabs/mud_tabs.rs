//! `MudTabs` — a set of views organised into one or more
//! [`MudTabPanel`]s.
//!
//! Port of `MudTabs.razor.cs` (MudBlazor 8.13.0).
//!
//! # MudBlazor parity
//!
//! | MudBlazor parameter | Rust method | Default |
//! |---------------------|-------------|---------|
//! | `ActivePanelIndex`  | [`TabsState::active_index`] | `0` |
//! | `ActivePanelIndexChanged` | [`MudTabs::on_active_index_changed`] | `None` |
//! | `ActiveTabClass`    | [`MudTabs::active_tab_class`] | `None` |
//! | `AlwaysShowScrollButtons` | [`MudTabs::always_show_scroll_buttons`] | `false` |
//! | `ApplyEffectsToContainer` | [`MudTabs::apply_effects_to_container`] | `false` |
//! | `Border`            | [`MudTabs::border`] | `false` |
//! | `Centered`          | [`MudTabs::centered`] | `false` |
//! | `ChildContent`      | closure to [`MudTabs::show`] | required |
//! | `Color`             | [`MudTabs::color`] | `Default` |
//! | `DisableRipple`     | [`MudTabs::disable_ripple`] | `false` |
//! | `DisableSliderAnimation` | [`MudTabs::disable_slider_animation`] | `false` |
//! | `Elevation`         | [`MudTabs::elevation`] | `0` |
//! | `Header`            | [`MudTabs::header`] | `None` |
//! | `HeaderPosition`    | [`MudTabs::header_position`] | `After` |
//! | `HideSlider`        | [`MudTabs::hide_slider`] | `false` |
//! | `IconColor`         | [`MudTabs::icon_color`] | `Inherit` |
//! | `KeepPanelsAlive`   | [`MudTabs::keep_panels_alive`] | `false` (no-op in egui) |
//! | `MaxHeight`         | [`MudTabs::max_height`] | `None` |
//! | `MinimumTabWidth`   | [`MudTabs::minimum_tab_width`] | `160.0` |
//! | `NextIcon`          | [`MudTabs::next_icon`] | `CHEVRON_RIGHT` |
//! | `OnPreviewInteraction` | [`MudTabs::on_preview_interaction`] | `None` |
//! | `Outlined`          | [`MudTabs::outlined`] | `false` |
//! | `PanelClass`        | [`MudTabs::panel_class`] | `None` |
//! | `Position`          | [`MudTabs::position`] | `Top` |
//! | `PrePanelContent`   | [`MudTabs::pre_panel_content`] | `None` |
//! | `PrevIcon`          | [`MudTabs::prev_icon`] | `CHEVRON_LEFT` |
//! | `RightToLeft`       | [`MudTabs::right_to_left`] | `false` |
//! | `Ripple`            | [`MudTabs::ripple`] | `true` (stored; hover used instead) |
//! | `Rounded`           | [`MudTabs::rounded`] | `false` |
//! | `ScrollIconColor`   | [`MudTabs::scroll_icon_color`] | `Inherit` |
//! | `SliderColor`       | [`MudTabs::slider_color`] | `Inherit` |
//! | `SliderAnimation`   | [`MudTabs::slider_animation`] | `true` |
//! | `TabHeaderClass`    | [`MudTabs::tab_header_class`] | `None` |
//! | `TabPanelClass`     | [`MudTabs::tab_panel_class`] | `None` |
//! | `TabPanelHeader`    | [`MudTabs::tab_panel_header`] | `None` |
//! | `TabPanelHeaderPosition` | [`MudTabs::tab_panel_header_position`] | `After` |
//! | `Class` (base)      | [`MudTabs::classes`] | `None` |
//! | `Style` (base)      | *(use `classes`)* | — |
//! | `Tag` (base)        | **not ported** (no HTML tag) | — |
//! | `UserAttributes`    | **not ported** | — |
//!
//! ### Extensions (not in MudBlazor)
//!
//! | Extension | Description |
//! |-----------|-------------|
//! | [`id_salt`](MudTabs::id_salt) | Stable interaction ids. |
//! | [`TabsState`] | Caller-owned active index. |
//! | [`TabsResponse`] | Rich return value with `action` and `panel_rect`. |
//! | [`TabAction`] | Reports what was clicked this frame. |
//! | [`TabView`] | Read-only snapshot passed to `tab_content`. |
//! | [`position`](MudTabs::position) accepts `Position::Start` / `End` and normalises to `Left` / `Right` under RTL. |
//!
//! ### Deliberately not ported
//!
//! * **`SortDirection` / `SortComparer`** — added in MudBlazor 8.14, not in
//!   8.13.
//! * **`EnableDragAndDrop` / `OnItemDropped`** — also 8.14 additions.
//! * **`KeepPanelsAlive`** — accepted for parity but a no-op: egui panel
//!   bodies are per-frame closures, so state must live on the caller.
//! * **`ShowCloseIcon` / `CloseIcon` / `CloseIconColor` / `OnClose`** — these
//!   belong to `MudDynamicTabs`, which is not ported.
//! * **`AlwaysShowScrollButtons` / `PrevIcon` / `NextIcon` /
//!   `ScrollIconColor`** — accepted for parity; the port uses a native
//!   `ScrollArea` scrollbar instead of icon buttons.
//! * **`Tag` / `UserAttributes`** — no HTML tag.
//! * **`Style`** — no inline CSS strings; use `classes(..)`.
//!
//! # Slider animation
//!
//! When [`slider_animation`](MudTabs::slider_animation) is `true` (default),
//! the coloured underline slides between tab positions using
//! `egui::Context::animate_value_with_time` with a 220 ms ease.
//!
//! # KeepPanelsAlive
//!
//! MudBlazor destroys the inactive panel's `ChildContent` when
//! `KeepPanelsAlive` is `false` (default). In egui, panel content is a
//! per-frame `FnMut` closure, so this parameter is a **no-op**: the active
//! panel's closure runs and every other panel's closure is skipped.
//!
//! # Text and icons
//!
//! Panel titles render through [`MudText`], icons through [`MudIcon`], badges
//! through [`MudBadge`], and the header / panel content areas use the
//! `BsStyle` / `TwStyle` resolvers for their class fields.

use std::any::Any;
use std::sync::Arc;

use egui::{
    pos2, vec2, Color32, CornerRadius, CursorIcon, Frame, Id, Rect, Response, Sense,
    Stroke, StrokeKind, Ui, Vec2,
};
use mud_egui_bootstrap::bs_style::BsStyle;
use mud_egui_core::base::component_base::ComponentBase;
use mud_egui_core::enums::{
    Color, Position, StyleFlavor, TabHeaderPosition, Typo,
};
use mud_egui_icons_material::filled;
use mud_egui_tailwind::tw_style::{TwState, TwStyle};
use mud_egui_theming::models::palettes::palette::Palette;
use mud_egui_theming::theme::MudTheme;

use crate::data_display::icons::mud_icon::MudIcon;
use crate::typography::mud_text::MudText;
use crate::MudBadge;

use super::mud_tab_panel::{MudTabPanel, TabView};
use super::tab_action::TabAction;
use super::tab_interaction::TabInteraction;
use super::tabs_state::TabsState;

// =============================================================================
// Constants
// =============================================================================

/// Base rem size passed to the Tailwind resolver.
const REM_SIZE_PT: f32 = 14.0;

/// Height of the tab header row when positioned at the top or bottom.
pub const HORIZONTAL_HEADER_HEIGHT: f32 = 48.0;

/// Height of one tab header row when the header is vertical.
pub const VERTICAL_HEADER_HEIGHT: f32 = 40.0;

/// Thickness of the active tab slider.
pub const SLIDER_THICKNESS: f32 = 2.0;

/// Horizontal padding inside a tab header cell.
pub const TAB_PADDING_X: f32 = 16.0;

/// Gap between an icon and its text.
const ICON_TEXT_GAP: f32 = 8.0;

/// Corner radius when `rounded` is enabled.
const ROUNDED_RADIUS: u8 = 4;

/// Duration of the slider animation, in seconds.
const SLIDER_ANIM_SECS: f32 = 0.22;

/// Default `MinimumTabWidth` (MudBlazor uses `"160px"`).
pub const DEFAULT_MINIMUM_TAB_WIDTH: f32 = 160.0;

/// Vertical gap between the header and the panel content.
const SECTION_GAP: f32 = 12.0;

/// Gap between a tab's icon and its text.
const TAB_ICON_GAP: f32 = 8.0;

/// Horizontal padding inside a tab header cell.
const TAB_CELL_PADDING: f32 = 16.0;

// =============================================================================
// Pure helpers
// =============================================================================

/// Normalises a [`Position`] into the four physical header positions.
#[inline]
pub fn resolve_position(position: Position, right_to_left: bool) -> Position {
    match position.resolve(right_to_left) {
        Position::Bottom => Position::Bottom,
        Position::Left => Position::Left,
        Position::Right => Position::Right,
        _ => Position::Top,
    }
}

/// Whether the header is a horizontal row (top / bottom).
#[inline]
pub fn is_horizontal_header(position: Position) -> bool {
    matches!(position, Position::Top | Position::Bottom)
}

/// Computes each header cell's width for a horizontal header.
pub fn horizontal_cell_widths(natural: &[f32], minimum: f32, available: f32) -> Vec<f32> {
    let minimum = minimum.max(0.0);
    let mut widths: Vec<f32> = natural.iter().map(|w| w.max(minimum)).collect();
    let total: f32 = widths.iter().sum();
    if total < available && !widths.is_empty() && total > 0.0 {
        let scale = available / total;
        for w in &mut widths {
            *w *= scale;
        }
    }
    widths
}

#[inline]
fn resolve_color(c: Color, palette: &Palette, default: Color32) -> Color32 {
    match c {
        Color::Default | Color::Inherit => default,
        other => other.resolve(palette).unwrap_or(default),
    }
}

fn resolve_class_frame(
    class: Option<&str>,
    lookup: &dyn Fn(&str) -> Option<Color32>,
    bs: &mut BsStyle,
    tw: &mut TwStyle,
    flavor: StyleFlavor,
    tw_state: TwState,
) -> Option<Frame> {
    let class = class?.trim();
    if class.is_empty() {
        return None;
    }
    let base = ComponentBase::new().classes(class.split_whitespace());
    let frame = match flavor {
        StyleFlavor::Bootstrap => bs.build_frame(&base, lookup),
        StyleFlavor::Tailwind => tw.build_frame(&base, tw_state, REM_SIZE_PT, lookup),
    };
    Some(frame)
}

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

fn elevation_shadow(elevation: u8) -> egui::epaint::Shadow {
    let e = elevation.min(24) as f32;
    egui::epaint::Shadow {
        offset: [0, (e * 0.4) as i8],
        blur: (e * 1.5) as u8,
        spread: 0,
        color: Color32::from_black_alpha(((e * 3.0) as u8).min(60)),
    }
}

// =============================================================================
// Response
// =============================================================================

/// The result of one [`MudTabs::show`] frame.
#[derive(Debug)]
pub struct TabsResponse {
    pub response: Response,
    pub active_index: usize,
    pub action: TabAction,
    pub panel_rect: Option<Rect>,
}

// =============================================================================
// TabUi
// =============================================================================

/// Collector passed to [`MudTabs::show`]'s closure.
pub struct TabUi<'a> {
    pub(crate) panels: Vec<MudTabPanel<'a>>,
}

impl<'a> TabUi<'a> {
    pub fn panel(&mut self, panel: MudTabPanel<'a>) {
        self.panels.push(panel);
    }
    pub fn text_panel(&mut self, text: impl Into<String>) {
        self.panels.push(MudTabPanel::new(text));
    }
    pub fn len(&self) -> usize {
        self.panels.len()
    }
    pub fn is_empty(&self) -> bool {
        self.panels.is_empty()
    }
}

// =============================================================================
// TabsCfg — per-frame immutable config
// =============================================================================

struct TabsCfg<'p> {
    position: Position,
    horizontal: bool,
    centered: bool,
    minimum_tab_width: f32,
    accent: Color32,
    icon_color: Color32,
    slider_color: Color32,
    text_primary: Color32,
    text_secondary: Color32,
    text_disabled: Color32,
    border_color: Color32,
    radius: CornerRadius,
    outlined: bool,
    border: bool,
    hide_slider: bool,
    slider_animation: bool,
    header_frame: Option<Frame>,
    active_frame: Option<Frame>,
    panel_frame: Option<Frame>,
    base_id: Id,
    palette: &'p Palette,
    header_position: TabHeaderPosition,
    tab_panel_header_position: TabHeaderPosition,
}

// =============================================================================
// MudTabs
// =============================================================================

/// A set of views organised into one or more panels.
#[must_use = "MudTabs does nothing until you call `.show(ui, state, |t| ..)`"]
pub struct MudTabs<'a> {
    pub position: Position,
    pub centered: bool,
    pub minimum_tab_width: f32,
    pub max_height: Option<f32>,
    pub border: bool,

    pub color: Color,
    pub icon_color: Color,
    pub slider_color: Color,
    pub slider_animation: bool,
    pub hide_slider: bool,
    pub outlined: bool,
    pub rounded: bool,
    pub elevation: u8,
    pub apply_effects_to_container: bool,
    pub ripple: bool,
    pub disable_ripple: bool,

    pub always_show_scroll_buttons: bool,
    pub prev_icon: String,
    pub next_icon: String,
    pub scroll_icon_color: Color,

    pub keep_panels_alive: bool,
    pub right_to_left: bool,

    pub tab_header_class: Option<String>,
    pub panel_class: Option<String>,
    pub tab_panel_class: Option<String>,
    pub active_tab_class: Option<String>,

    pub header: Option<Box<dyn FnMut(&mut Ui) + 'a>>,
    pub header_position: TabHeaderPosition,
    pub tab_panel_header: Option<Box<dyn FnMut(&mut Ui, &TabView<'_>) + 'a>>,
    pub tab_panel_header_position: TabHeaderPosition,
    pub pre_panel_content: Option<Box<dyn FnMut(&mut Ui, &TabView<'_>) + 'a>>,

    pub on_active_index_changed: Option<Box<dyn FnMut(usize) + 'a>>,
    pub on_preview_interaction: Option<Box<dyn FnMut(&mut TabInteraction) + 'a>>,

    pub id_salt: Option<String>,
    pub base: ComponentBase,
    pub flavor: StyleFlavor,
    pub bs_style: BsStyle,
    pub tw_style: TwStyle,
    pub tw_state: TwState,
    tag_value: Option<Arc<dyn Any + Send + Sync>>,
}

impl<'a> Default for MudTabs<'a> {
    fn default() -> Self {
        Self {
            position: Position::Top,
            centered: false,
            minimum_tab_width: DEFAULT_MINIMUM_TAB_WIDTH,
            max_height: None,
            border: false,
            color: Color::Default,
            icon_color: Color::Inherit,
            slider_color: Color::Inherit,
            slider_animation: true,
            hide_slider: false,
            outlined: false,
            rounded: false,
            elevation: 0,
            apply_effects_to_container: false,
            ripple: true,
            disable_ripple: false,
            always_show_scroll_buttons: false,
            prev_icon: filled::CHEVRON_LEFT.to_owned(),
            next_icon: filled::CHEVRON_RIGHT.to_owned(),
            scroll_icon_color: Color::Inherit,
            keep_panels_alive: false,
            right_to_left: false,
            tab_header_class: None,
            panel_class: None,
            tab_panel_class: None,
            active_tab_class: None,
            header: None,
            header_position: TabHeaderPosition::After,
            tab_panel_header: None,
            tab_panel_header_position: TabHeaderPosition::After,
            pre_panel_content: None,
            on_active_index_changed: None,
            on_preview_interaction: None,
            id_salt: None,
            base: ComponentBase::new(),
            flavor: StyleFlavor::default(),
            bs_style: BsStyle::default(),
            tw_style: TwStyle::default(),
            tw_state: TwState::default(),
            tag_value: None,
        }
    }
}

impl<'a> MudTabs<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    // ── Builders ───────────────────────────────────────────────────────────

    #[inline] pub fn position(mut self, p: Position) -> Self { self.position = p; self }
    #[inline] pub fn centered(mut self, b: bool) -> Self { self.centered = b; self }
    #[inline] pub fn minimum_tab_width(mut self, px: f32) -> Self { self.minimum_tab_width = px.max(0.0); self }
    #[inline] pub fn max_height(mut self, px: f32) -> Self { self.max_height = Some(px.max(0.0)); self }
    #[inline] pub fn border(mut self, b: bool) -> Self { self.border = b; self }
    #[inline] pub fn color(mut self, c: Color) -> Self { self.color = c; self }
    #[inline] pub fn icon_color(mut self, c: Color) -> Self { self.icon_color = c; self }
    #[inline] pub fn slider_color(mut self, c: Color) -> Self { self.slider_color = c; self }
    #[inline] pub fn slider_animation(mut self, b: bool) -> Self { self.slider_animation = b; self }
    #[inline] pub fn hide_slider(mut self, b: bool) -> Self { self.hide_slider = b; self }
    #[inline] pub fn outlined(mut self, b: bool) -> Self { self.outlined = b; self }
    #[inline] pub fn rounded(mut self, b: bool) -> Self { self.rounded = b; self }
    #[inline] pub fn elevation(mut self, e: u8) -> Self { self.elevation = e; self }
    #[inline] pub fn apply_effects_to_container(mut self, b: bool) -> Self { self.apply_effects_to_container = b; self }
    #[inline] pub fn ripple(mut self, b: bool) -> Self { self.ripple = b; self }
    #[inline] pub fn disable_ripple(mut self, b: bool) -> Self { self.disable_ripple = b; self }
    #[inline] pub fn always_show_scroll_buttons(mut self, b: bool) -> Self { self.always_show_scroll_buttons = b; self }
    #[inline] pub fn prev_icon(mut self, svg: impl Into<String>) -> Self { self.prev_icon = svg.into(); self }
    #[inline] pub fn next_icon(mut self, svg: impl Into<String>) -> Self { self.next_icon = svg.into(); self }
    #[inline] pub fn scroll_icon_color(mut self, c: Color) -> Self { self.scroll_icon_color = c; self }
    #[inline] pub fn keep_panels_alive(mut self, b: bool) -> Self { self.keep_panels_alive = b; self }
    #[inline] pub fn right_to_left(mut self, b: bool) -> Self { self.right_to_left = b; self }
    #[inline] pub fn tab_header_class(mut self, s: impl Into<String>) -> Self { self.tab_header_class = Some(s.into()); self }
    #[inline] pub fn panel_class(mut self, s: impl Into<String>) -> Self { self.panel_class = Some(s.into()); self }
    #[inline] pub fn tab_panel_class(mut self, s: impl Into<String>) -> Self { self.tab_panel_class = Some(s.into()); self }
    #[inline] pub fn active_tab_class(mut self, s: impl Into<String>) -> Self { self.active_tab_class = Some(s.into()); self }
    #[inline] pub fn header(mut self, f: impl FnMut(&mut Ui) + 'a) -> Self { self.header = Some(Box::new(f)); self }
    #[inline] pub fn header_position(mut self, p: TabHeaderPosition) -> Self { self.header_position = p; self }
    #[inline] pub fn tab_panel_header(mut self, f: impl FnMut(&mut Ui, &TabView<'_>) + 'a) -> Self { self.tab_panel_header = Some(Box::new(f)); self }
    #[inline] pub fn tab_panel_header_position(mut self, p: TabHeaderPosition) -> Self { self.tab_panel_header_position = p; self }
    #[inline] pub fn pre_panel_content(mut self, f: impl FnMut(&mut Ui, &TabView<'_>) + 'a) -> Self { self.pre_panel_content = Some(Box::new(f)); self }
    #[inline] pub fn on_active_index_changed(mut self, f: impl FnMut(usize) + 'a) -> Self { self.on_active_index_changed = Some(Box::new(f)); self }
    #[inline] pub fn on_preview_interaction(mut self, f: impl FnMut(&mut TabInteraction) + 'a) -> Self { self.on_preview_interaction = Some(Box::new(f)); self }
    #[inline] pub fn id_salt(mut self, s: impl Into<String>) -> Self { self.id_salt = Some(s.into()); self }
    #[inline] pub fn classes<S: AsRef<str>>(mut self, classes: impl IntoIterator<Item = S>) -> Self { self.base = self.base.classes(classes); self }
    #[inline] pub fn flavor(mut self, f: StyleFlavor) -> Self { self.flavor = f; self }
    pub fn tag<T: Any + Send + Sync>(mut self, value: T) -> Self { self.tag_value = Some(Arc::new(value)); self }
    pub fn tag_ref<U: Any + Send + Sync>(&self) -> Option<&U> { self.tag_value.as_ref()?.downcast_ref::<U>() }

    // ── Rendering ──────────────────────────────────────────────────────────

    /// Draws the tabs.
    pub fn show<R>(
        mut self,
        ui: &mut Ui,
        state: &mut TabsState,
        add_panels: impl FnOnce(&mut TabUi<'a>) -> R,
    ) -> (TabsResponse, R) {
        // 1. Collect panels.
        let mut collector = TabUi { panels: Vec::new() };
        let user_result = add_panels(&mut collector);
        let mut panels = collector.panels;
        let count = panels.len();

        if count == 0 {
            let response = ui.allocate_response(Vec2::ZERO, Sense::hover());
            return (
                TabsResponse {
                    response,
                    active_index: 0,
                    action: TabAction::None,
                    panel_rect: None,
                },
                user_result,
            );
        }
        state.activate(state.active_index, count);

        // 2. Theme + wrapper frame.
        let theme = MudTheme::fetch(ui.ctx()).unwrap_or_default();
        let palette = theme.current_palette();
        let lookup = |key: &str| palette.resolve(key).map(Color32::from);

        self.tw_state.dark = ui.visuals().dark_mode;
        let mut frame = match self.flavor {
            StyleFlavor::Bootstrap => self.bs_style.build_frame(&self.base, &lookup),
            StyleFlavor::Tailwind => self
                .tw_style
                .build_frame(&self.base, self.tw_state, REM_SIZE_PT, &lookup),
        };
        if self.elevation > 0 && self.apply_effects_to_container {
            frame = frame.shadow(elevation_shadow(self.elevation));
        }

        // 3. Resolve configuration.
        let position = resolve_position(self.position, self.right_to_left);
        let horizontal = is_horizontal_header(position);
        let accent = resolve_color(self.color, palette, palette.primary.into());
        let icon_color = resolve_color(self.icon_color, palette, accent);
        let slider_color = resolve_color(self.slider_color, palette, accent);
        let border_color = palette.divider.into();
        let radius = if self.rounded {
            CornerRadius::same(ROUNDED_RADIUS)
        } else {
            CornerRadius::ZERO
        };

        let header_frame = resolve_class_frame(
            self.tab_header_class.as_deref(),
            &lookup,
            &mut self.bs_style,
            &mut self.tw_style,
            self.flavor,
            self.tw_state,
        );
        let panel_class_effective = self
            .tab_panel_class
            .as_deref()
            .or(self.panel_class.as_deref());
        let panel_frame = resolve_class_frame(
            panel_class_effective,
            &lookup,
            &mut self.bs_style,
            &mut self.tw_style,
            self.flavor,
            self.tw_state,
        );
        let active_frame = resolve_class_frame(
            self.active_tab_class.as_deref(),
            &lookup,
            &mut self.bs_style,
            &mut self.tw_style,
            self.flavor,
            self.tw_state,
        );

        // 4. Move callbacks out of `self`.
        let mut header = self.header.take();
        let mut tab_panel_header = self.tab_panel_header.take();
        let mut pre_panel_content = self.pre_panel_content.take();
        let mut on_active_index_changed = self.on_active_index_changed.take();
        let mut on_preview = self.on_preview_interaction.take();

        let base_id = match self.id_salt.as_deref() {
            Some(s) => ui.scope_id().with(s),
            None => ui.scope_id().with("mud_tabs"),
        };

        let cfg = TabsCfg {
            position,
            horizontal,
            centered: self.centered,
            minimum_tab_width: self.minimum_tab_width,
            accent,
            icon_color,
            slider_color,
            text_primary: palette.text_primary.into(),
            text_secondary: palette.text_secondary.into(),
            text_disabled: palette.text_disabled.into(),
            border_color,
            radius,
            outlined: self.outlined,
            border: self.border,
            hide_slider: self.hide_slider,
            slider_animation: self.slider_animation,
            header_frame,
            active_frame,
            panel_frame,
            base_id,
            palette,
            header_position: self.header_position,
            tab_panel_header_position: self.tab_panel_header_position,
        };

        let max_height = self.max_height;
        let minimum_tab_width = self.minimum_tab_width;

        let mut action = TabAction::None;
        let mut panel_rect_out: Option<Rect> = None;

        // 5. Render body. Each branch calls free functions directly — no
        //    hoisted closures means no overlapping `&mut` borrows.
        let inner = frame.show(ui, |ui| {
            match position {
                Position::Bottom => {
                    panel_rect_out = render_panel_content(
                        ui, &mut panels, state, &cfg,
                    );
                    ui.add_space(SECTION_GAP);
                    render_pre_panel_content(
                        ui, &panels, state, &mut pre_panel_content,
                    );
                    render_header_block(
                        ui, &mut panels, state, &cfg,
                        &mut header, &mut tab_panel_header,
                        &mut on_preview, &mut on_active_index_changed,
                        &mut action,
                    );
                }
                Position::Left => {
                    ui.horizontal_top(|ui| {
                        let total = ui.available_width();
                        let header_w = (total * 0.3).max(minimum_tab_width);
                        ui.allocate_ui_with_layout(
                            vec2(header_w, 0.0),
                            egui::Layout::top_down(egui::Align::Min),
                            |ui| {
                                render_header_block(
                                    ui, &mut panels, state, &cfg,
                                    &mut header, &mut tab_panel_header,
                                    &mut on_preview, &mut on_active_index_changed,
                                    &mut action,
                                );
                            },
                        );
                        ui.allocate_ui_with_layout(
                            vec2((total - header_w).max(0.0), 0.0),
                            egui::Layout::top_down(egui::Align::Min),
                            |ui| {
                                render_pre_panel_content(
                                    ui, &panels, state, &mut pre_panel_content,
                                );
                                panel_rect_out = render_panel_content(
                                    ui, &mut panels, state, &cfg,
                                );
                            },
                        );
                    });
                }
                Position::Right => {
                    ui.horizontal_top(|ui| {
                        let total = ui.available_width();
                        let header_w = (total * 0.3).max(minimum_tab_width);
                        ui.allocate_ui_with_layout(
                            vec2((total - header_w).max(0.0), 0.0),
                            egui::Layout::top_down(egui::Align::Min),
                            |ui| {
                                render_pre_panel_content(
                                    ui, &panels, state, &mut pre_panel_content,
                                );
                                panel_rect_out = render_panel_content(
                                    ui, &mut panels, state, &cfg,
                                );
                            },
                        );
                        ui.allocate_ui_with_layout(
                            vec2(header_w, 0.0),
                            egui::Layout::top_down(egui::Align::Min),
                            |ui| {
                                render_header_block(
                                    ui, &mut panels, state, &cfg,
                                    &mut header, &mut tab_panel_header,
                                    &mut on_preview, &mut on_active_index_changed,
                                    &mut action,
                                );
                            },
                        );
                    });
                }
                _ => {
                    render_header_block(
                        ui, &mut panels, state, &cfg,
                        &mut header, &mut tab_panel_header,
                        &mut on_preview, &mut on_active_index_changed,
                        &mut action,
                    );
                    render_pre_panel_content(
                        ui, &panels, state, &mut pre_panel_content,
                    );
                    if let Some(h) = max_height {
                        egui::ScrollArea::vertical()
                            .max_height(h)
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                panel_rect_out = render_panel_content(
                                    ui, &mut panels, state, &cfg,
                                );
                            });
                    } else {
                        panel_rect_out = render_panel_content(
                            ui, &mut panels, state, &cfg,
                        );
                    }
                }
            }
        });

        (
            TabsResponse {
                response: inner.response,
                active_index: state.active_index,
                action,
                panel_rect: panel_rect_out,
            },
            user_result,
        )
    }
}

// =============================================================================
// Free rendering functions
// =============================================================================

/// Renders the active panel's body.
fn render_panel_content<'a>(
    ui: &mut Ui,
    panels: &mut [MudTabPanel<'a>],
    state: &TabsState,
    cfg: &TabsCfg<'_>,
) -> Option<Rect> {
    let active = state.active_index;
    if active >= panels.len() {
        return None;
    }

    // `take()` any field the panel needs *mutably* so we can later create an
    // immutable `&panel`-borrow without conflict. Panels are per-frame, so
    // taking is safe and cheap.
    let mut content_opt = panels[active].content.take();
    let text_opt = panels[active].text.take();
    let text_primary = cfg.text_primary;

    let mut rendered_rect: Option<Rect> = None;

    let inner = |ui: &mut Ui| {
        let scope = ui.scope(|ui| {
            if let Some(content) = content_opt.as_mut() {
                content(ui);
            } else if let Some(t) = text_opt.as_deref() {
                ui.add(
                    MudText::new(t.to_owned())
                        .typo(Typo::Body1)
                        .custom_color(text_primary),
                );
            }
        });
        rendered_rect = Some(scope.response.rect);
    };

    if let Some(f) = cfg.panel_frame.as_ref() {
        let f = f.clone();
        f.show(ui, inner);
    } else {
        ui.scope(inner);
    }

    rendered_rect
}

/// Renders the `pre_panel_content` fragment (between header and panel).
fn render_pre_panel_content<'a>(
    ui: &mut Ui,
    panels: &[MudTabPanel<'a>],
    state: &TabsState,
    pre_panel_content: &mut Option<Box<dyn FnMut(&mut Ui, &TabView<'_>) + 'a>>,
) {
    if let Some(content) = pre_panel_content.as_mut() {
        let active = state.active_index;
        if active < panels.len() {
            let view = panels[active].view(active, true);
            content(ui, &view);
        }
    }
}

/// Renders the header row (with before / after fragments, slider, border).
#[allow(clippy::too_many_arguments)]
fn render_header_block<'a>(
    ui: &mut Ui,
    panels: &mut [MudTabPanel<'a>],
    state: &mut TabsState,
    cfg: &TabsCfg<'_>,
    header: &mut Option<Box<dyn FnMut(&mut Ui) + 'a>>,
    tab_panel_header: &mut Option<Box<dyn FnMut(&mut Ui, &TabView<'_>) + 'a>>,
    on_preview: &mut Option<Box<dyn FnMut(&mut TabInteraction) + 'a>>,
    on_active_index_changed: &mut Option<Box<dyn FnMut(usize) + 'a>>,
    action: &mut TabAction,
) -> Rect {
    let mut any_rect = Rect::NOTHING;

    // Header "before" content.
    if cfg.header_position.is_before() {
        if let Some(h) = header.as_mut() {
            let scope = ui.scope(|ui| h(ui));
            any_rect = any_rect.union(scope.response.rect);
        }
    }

    // Visible panel indices.
    let visible: Vec<usize> = panels
        .iter()
        .enumerate()
        .filter(|(_, p)| p.visible)
        .map(|(i, _)| i)
        .collect();
    if visible.is_empty() {
        return any_rect;
    }

    // Measure cell widths (horizontal only).
    let available = ui.available_width();
    let cell_widths: Vec<f32> = if cfg.horizontal {
        let natural: Vec<f32> = visible
            .iter()
            .map(|&i| measure_header_width(ui, &panels[i]))
            .collect();
        horizontal_cell_widths(&natural, cfg.minimum_tab_width, available)
    } else {
        vec![cfg.minimum_tab_width; visible.len()]
    };
    let total_w: f32 = cell_widths.iter().sum();

    // Lay out the header. The two branches are exclusive; each borrows
    // `tab_rects` mutably in its own scope.
    let mut tab_rects: Vec<(usize, Rect)> = Vec::with_capacity(visible.len());

    if cfg.horizontal && total_w > available {
        egui::ScrollArea::horizontal()
            .auto_shrink([false, true])
            .show(ui, |ui| {
                render_header_layout(
                    ui, panels, state, cfg, &visible, &cell_widths, available,
                    tab_panel_header, on_preview, on_active_index_changed,
                    action, &mut tab_rects,
                );
            });
    } else {
        render_header_layout(
            ui, panels, state, cfg, &visible, &cell_widths, available,
            tab_panel_header, on_preview, on_active_index_changed,
            action, &mut tab_rects,
        );
    }

    for (_, r) in &tab_rects {
        any_rect = any_rect.union(*r);
    }

    // Border between header and panel.
    if cfg.border {
        if let Some(&(_, last)) = tab_rects.last() {
            let y = match cfg.position {
                Position::Bottom => last.top(),
                _ => last.bottom(),
            };
            ui.painter().line_segment(
                [pos2(any_rect.left(), y), pos2(any_rect.right(), y)],
                Stroke::new(1.0, cfg.border_color),
            );
        }
    }

    // Outlined border around the header.
    if cfg.outlined {
        ui.painter().rect_stroke(
            any_rect,
            cfg.radius,
            Stroke::new(1.0, cfg.border_color),
            StrokeKind::Inside,
        );
    }

    // Slider.
    if !cfg.hide_slider {
        if let Some((_, active_rect)) = tab_rects
            .iter()
            .find(|(i, _)| *i == state.active_index)
            .copied()
        {
            paint_slider(
                ui,
                active_rect,
                cfg.position,
                cfg.slider_color,
                cfg.slider_animation,
                &cfg.base_id,
            );
        }
    }

    // Header "after" content.
    if cfg.header_position.is_after() {
        if let Some(h) = header.as_mut() {
            let scope = ui.scope(|ui| h(ui));
            any_rect = any_rect.union(scope.response.rect);
        }
    }

    any_rect
}

/// Lays out the tab header cells and returns their rects via `tab_rects`.
#[allow(clippy::too_many_arguments)]
fn render_header_layout<'a>(
    ui: &mut Ui,
    panels: &mut [MudTabPanel<'a>],
    state: &mut TabsState,
    cfg: &TabsCfg<'_>,
    visible: &[usize],
    cell_widths: &[f32],
    available: f32,
    tab_panel_header: &mut Option<Box<dyn FnMut(&mut Ui, &TabView<'_>) + 'a>>,
    on_preview: &mut Option<Box<dyn FnMut(&mut TabInteraction) + 'a>>,
    on_active_index_changed: &mut Option<Box<dyn FnMut(usize) + 'a>>,
    action: &mut TabAction,
    tab_rects: &mut Vec<(usize, Rect)>,
) {
    if cfg.horizontal {
        let row_height = HORIZONTAL_HEADER_HEIGHT;
        let total_w: f32 = cell_widths.iter().sum();
        let start_x = if cfg.centered && total_w < available {
            ui.cursor().min.x + (available - total_w) * 0.5
        } else {
            ui.cursor().min.x
        };
        let start_y = ui.cursor().min.y;
        let mut x = start_x;
        for (k, &idx) in visible.iter().enumerate() {
            let w = cell_widths[k];
            let rect = Rect::from_min_size(pos2(x, start_y), vec2(w, row_height));
            let clicked = render_single_tab(
                ui, rect, idx, &mut panels[idx], state.active_index == idx,
                cfg, tab_panel_header,
            );
            if clicked {
                activate_tab(idx, state, on_preview, on_active_index_changed, action);
            }
            tab_rects.push((idx, rect));
            x += w;
        }
        ui.allocate_rect(
            Rect::from_min_size(pos2(start_x, start_y), vec2(total_w, row_height)),
            Sense::hover(),
        );
    } else {
        let row_height = VERTICAL_HEADER_HEIGHT;
        let start = ui.cursor().min;
        for (k, &idx) in visible.iter().enumerate() {
            let w = cell_widths[k];
            let rect = Rect::from_min_size(
                pos2(start.x, start.y + k as f32 * row_height),
                vec2(w, row_height),
            );
            let clicked = render_single_tab(
                ui, rect, idx, &mut panels[idx], state.active_index == idx,
                cfg, tab_panel_header,
            );
            if clicked {
                activate_tab(idx, state, on_preview, on_active_index_changed, action);
            }
            tab_rects.push((idx, rect));
        }
        if let Some(&(_, last)) = tab_rects.last() {
            ui.allocate_rect(
                Rect::from_min_size(start, vec2(cell_widths[0], last.bottom() - start.y)),
                Sense::hover(),
            );
        }
    }
}

/// Renders one tab header cell and returns `true` if it was clicked.
#[allow(clippy::too_many_arguments)]
fn render_single_tab<'a>(
    ui: &mut Ui,
    rect: Rect,
    index: usize,
    panel: &mut MudTabPanel<'a>,
    is_active: bool,
    cfg: &TabsCfg<'_>,
    tab_panel_header: &mut Option<Box<dyn FnMut(&mut Ui, &TabView<'_>) + 'a>>,
) -> bool {
    let disabled = panel.disabled;

    let sense = if disabled { Sense::hover() } else { Sense::click() };
    let resp = ui.interact(rect, cfg.base_id.with(("tab", index)), sense);
    let hovered = resp.hovered() && !disabled;

    // Background fill.
    let fill = if is_active {
        cfg.active_frame
            .as_ref()
            .and_then(|f| if f.fill.a() > 0 { Some(f.fill) } else { None })
            .unwrap_or(Color32::TRANSPARENT)
    } else if hovered {
        with_alpha(cfg.text_primary, 0.06)
    } else {
        Color32::TRANSPARENT
    };
    if fill != Color32::TRANSPARENT {
        ui.painter().rect_filled(rect, CornerRadius::ZERO, fill);
    }
    if hovered {
        ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
    }

    // Take any `FnMut` fields we need to call, so we can later create an
    // immutable `&panel`-borrow for `view` without conflicting.
    let mut tab_content_opt = panel.tab_content.take();
    let mut tab_wrapper_opt = panel.tab_wrapper_content.take();
    let mut on_click_opt = panel.on_click.take();

    let view = panel.view(index, is_active);

    let content_rect = rect.shrink2(vec2(TAB_CELL_PADDING, 0.0));

    // "Before" panel-header fragment.
    if cfg.tab_panel_header_position.is_before() {
        if let Some(f) = tab_panel_header.as_mut() {
            let mut child = ui.new_child(
                egui::UiBuilder::new()
                    .max_rect(content_rect)
                    .layout(egui::Layout::left_to_right(egui::Align::Center)),
            );
            f(&mut child, &view);
        }
    }

    // Custom tab content or default text + icon + badge.
    if let Some(tab_content) = tab_content_opt.as_mut() {
        let mut child = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(content_rect)
                .layout(egui::Layout::left_to_right(egui::Align::Center)),
        );
        child.set_clip_rect(rect);
        tab_content(&mut child, &view);
    } else {
        let text_color = if disabled {
            cfg.text_disabled
        } else if is_active {
            cfg.icon_color
        } else {
            cfg.text_secondary
        };

        let mut child = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(content_rect)
                .layout(egui::Layout::left_to_right(egui::Align::Center)),
        );
        child.set_clip_rect(rect);

        if let Some(wrapper) = tab_wrapper_opt.as_mut() {
            let mut inner_box: Box<dyn FnMut(&mut Ui)> = Box::new(|ui: &mut Ui| {
                render_default_tab_content(ui, disabled, &view, text_color, cfg.palette);
            });
            wrapper(&mut child, &view, inner_box.as_mut());
        } else {
            render_default_tab_content(&mut child, disabled, &view, text_color, cfg.palette);
        }
    }

    // "After" panel-header fragment.
    if cfg.tab_panel_header_position.is_after() {
        if let Some(f) = tab_panel_header.as_mut() {
            let mut child = ui.new_child(
                egui::UiBuilder::new()
                    .max_rect(content_rect)
                    .layout(egui::Layout::left_to_right(egui::Align::Center)),
            );
            f(&mut child, &view);
        }
    }

    // Tooltip.
    if let Some(tip) = view.tooltip {
        resp.clone().on_hover_text(tip.to_owned());
    }

    // Accessibility.
    let label = view.text.unwrap_or("").to_owned();
    let enabled = !disabled;
    resp.clone().widget_info(|| {
        egui::WidgetInfo::selected(egui::Role::Tab, enabled, is_active, label.as_str())
    });

    // Fire the panel's own `on_click` regardless of active state.
    if resp.clicked() && !disabled {
        if let Some(cb) = on_click_opt.as_mut() {
            cb();
        }
        return true;
    }
    false
}

/// Renders the default tab header content: icon + title + badge.
fn render_default_tab_content(
    ui: &mut Ui,
    disabled: bool,
    view: &TabView<'_>,
    text_color: Color32,
    palette: &Palette,
) {
    if let Some(icon) = view.icon {
        let px = 18.0;
        let icon_color = if disabled {
            text_color
        } else {
            view.icon_color
                .resolve(palette)
                .map(Color32::from)
                .unwrap_or(text_color)
        };
        ui.add(
            MudIcon::new(icon.to_owned())
                .size_px(px)
                .custom_color(icon_color),
        );
        ui.add_space(TAB_ICON_GAP);
    }
    if let Some(text) = view.text {
        ui.add(
            MudText::new(text.to_owned())
                .typo(Typo::Body2)
                .bold(view.is_active)
                .custom_color(text_color),
        );
    }
    if view.has_badge {
        ui.add_space(6.0);
        let mut badge = MudBadge::new()
            .color(view.badge_color)
            .dot(view.badge_dot);
        if let Some(data) = view.badge_data {
            badge = badge.content(data);
        }
        if let Some(icon) = view.badge_icon {
            badge = badge.icon(icon);
        }
        badge.show(ui, |ui| {
            ui.allocate_space(Vec2::ZERO);
        });
    }
}

fn activate_tab<'a>(
    target: usize,
    state: &mut TabsState,
    on_preview: &mut Option<Box<dyn FnMut(&mut TabInteraction) + 'a>>,
    on_active_index_changed: &mut Option<Box<dyn FnMut(usize) + 'a>>,
    action: &mut TabAction,
) {
    let from = state.active_index;
    if from == target {
        return;
    }

    if let Some(cb) = on_preview.as_mut() {
        let mut interaction = TabInteraction::new(from, target);
        cb(&mut interaction);
        if interaction.cancel {
            *action = TabAction::Vetoed { target };
            return;
        }
    }

    state.active_index = target;
    if let Some(cb) = on_active_index_changed.as_mut() {
        cb(target);
    }
    *action = TabAction::Activated { from, to: target };
}

fn paint_slider(
    ui: &Ui,
    active_rect: Rect,
    position: Position,
    color: Color32,
    animate: bool,
    base_id: &Id,
) {
    let target_x = active_rect.left();
    let target_w = active_rect.width();
    let target_y = active_rect.top();
    let target_h = active_rect.height();

    let (x, w, y, h) = if animate {
        let ctx = ui.ctx();
        let anim_x = ctx.animate_value_with_time(
            base_id.with("slider_x"),
            target_x,
            SLIDER_ANIM_SECS,
        );
        let anim_w = ctx.animate_value_with_time(
            base_id.with("slider_w"),
            target_w,
            SLIDER_ANIM_SECS,
        );
        let anim_y = ctx.animate_value_with_time(
            base_id.with("slider_y"),
            target_y,
            SLIDER_ANIM_SECS,
        );
        let anim_h = ctx.animate_value_with_time(
            base_id.with("slider_h"),
            target_h,
            SLIDER_ANIM_SECS,
        );
        (anim_x, anim_w, anim_y, anim_h)
    } else {
        (target_x, target_w, target_y, target_h)
    };

    let slider_rect = match position {
        Position::Bottom => Rect::from_min_size(pos2(x, y), vec2(w, SLIDER_THICKNESS)),
        Position::Left => {
            Rect::from_min_size(pos2(x + w - SLIDER_THICKNESS, y), vec2(SLIDER_THICKNESS, h))
        }
        Position::Right => Rect::from_min_size(pos2(x, y), vec2(SLIDER_THICKNESS, h)),
        _ => {
            Rect::from_min_size(pos2(x, y + h - SLIDER_THICKNESS), vec2(w, SLIDER_THICKNESS))
        }
    };
    ui.painter().rect_filled(slider_rect, CornerRadius::ZERO, color);
}

fn measure_header_width(ui: &mut Ui, panel: &MudTabPanel<'_>) -> f32 {
    let text_w = panel
        .text
        .as_deref()
        .map(|t| {
            let galley = ui.painter().layout_no_wrap(
                t.to_owned(),
                egui::FontId::proportional(14.0),
                Color32::WHITE,
            );
            galley.size().x
        })
        .unwrap_or(0.0);
    let icon_w = if panel.icon.is_some() {
        18.0 + TAB_ICON_GAP
    } else {
        0.0
    };
    let badge_w = if panel.badge_data.is_some() || panel.badge_dot || panel.badge_icon.is_some() {
        6.0 + 20.0
    } else {
        0.0
    };

    let mut width = text_w + icon_w + badge_w + 2.0 * TAB_CELL_PADDING;

    if panel.tab_content.is_some() {
        width = width.max(DEFAULT_MINIMUM_TAB_WIDTH);
    }
    if panel.tab_wrapper_content.is_some() {
        width = width.max(DEFAULT_MINIMUM_TAB_WIDTH);
    }

    width
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_position_maps_physical_variants() {
        assert!(matches!(resolve_position(Position::Top, false), Position::Top));
        assert!(matches!(resolve_position(Position::Bottom, false), Position::Bottom));
        assert!(matches!(resolve_position(Position::Left, false), Position::Left));
        assert!(matches!(resolve_position(Position::Right, false), Position::Right));
    }

    #[test]
    fn resolve_position_normalises_start_end() {
        assert!(matches!(resolve_position(Position::Start, false), Position::Left));
        assert!(matches!(resolve_position(Position::Start, true), Position::Right));
        assert!(matches!(resolve_position(Position::End, false), Position::Right));
        assert!(matches!(resolve_position(Position::End, true), Position::Left));
    }

    #[test]
    fn resolve_position_maps_center_to_top() {
        assert!(matches!(resolve_position(Position::Center, false), Position::Top));
    }

    #[test]
    fn horizontal_header_for_top_and_bottom() {
        assert!(is_horizontal_header(Position::Top));
        assert!(is_horizontal_header(Position::Bottom));
        assert!(!is_horizontal_header(Position::Left));
        assert!(!is_horizontal_header(Position::Right));
    }

    #[test]
    fn cell_widths_clamp_to_minimum() {
        let w = horizontal_cell_widths(&[50.0, 200.0, 100.0], 160.0, 1000.0);
        assert_eq!(w, vec![160.0, 200.0, 160.0]);
    }

    #[test]
    fn cell_widths_grow_to_fill_available() {
        let w = horizontal_cell_widths(&[100.0, 100.0], 50.0, 500.0);
        assert_eq!(w, vec![250.0, 250.0]);
    }

    #[test]
    fn cell_widths_do_not_shrink_when_overflowing() {
        let w = horizontal_cell_widths(&[200.0, 200.0, 200.0], 100.0, 300.0);
        assert_eq!(w, vec![200.0, 200.0, 200.0]);
    }

    #[test]
    fn cell_widths_empty_input() {
        assert!(horizontal_cell_widths(&[], 100.0, 500.0).is_empty());
    }

    #[test]
    fn defaults_match_mudblazor() {
        let t: MudTabs<'static> = MudTabs::new();
        assert!(matches!(t.position, Position::Top));
        assert!(!t.centered);
        assert_eq!(t.minimum_tab_width, DEFAULT_MINIMUM_TAB_WIDTH);
        assert!(t.max_height.is_none());
        assert!(!t.border);
        assert!(matches!(t.color, Color::Default));
        assert!(matches!(t.icon_color, Color::Inherit));
        assert!(matches!(t.slider_color, Color::Inherit));
        assert!(t.slider_animation);
        assert!(!t.hide_slider);
        assert!(!t.outlined);
        assert!(!t.rounded);
        assert_eq!(t.elevation, 0);
        assert!(!t.apply_effects_to_container);
        assert!(t.ripple);
        assert!(!t.disable_ripple);
        assert!(!t.always_show_scroll_buttons);
        assert_eq!(t.prev_icon, filled::CHEVRON_LEFT);
        assert_eq!(t.next_icon, filled::CHEVRON_RIGHT);
        assert!(matches!(t.scroll_icon_color, Color::Inherit));
        assert!(!t.keep_panels_alive);
        assert!(!t.right_to_left);
        assert!(t.tab_header_class.is_none());
        assert!(t.panel_class.is_none());
        assert!(t.tab_panel_class.is_none());
        assert!(t.active_tab_class.is_none());
        assert!(t.header.is_none());
        assert_eq!(t.header_position, TabHeaderPosition::After);
        assert!(t.tab_panel_header.is_none());
        assert_eq!(t.tab_panel_header_position, TabHeaderPosition::After);
        assert!(t.pre_panel_content.is_none());
        assert!(t.on_active_index_changed.is_none());
        assert!(t.on_preview_interaction.is_none());
        assert!(t.id_salt.is_none());
    }

    #[test]
    fn builders_set_every_field() {
        let t: MudTabs<'static> = MudTabs::new()
            .position(Position::Bottom)
            .centered(true)
            .minimum_tab_width(200.0)
            .max_height(500.0)
            .border(true)
            .color(Color::Secondary)
            .icon_color(Color::Error)
            .slider_color(Color::Success)
            .slider_animation(false)
            .hide_slider(true)
            .outlined(true)
            .rounded(true)
            .elevation(4)
            .apply_effects_to_container(true)
            .ripple(false)
            .disable_ripple(true)
            .always_show_scroll_buttons(true)
            .prev_icon("P")
            .next_icon("N")
            .scroll_icon_color(Color::Warning)
            .keep_panels_alive(true)
            .right_to_left(true)
            .tab_header_class("th")
            .panel_class("pc")
            .tab_panel_class("tpc")
            .active_tab_class("atc")
            .header(|_| {})
            .header_position(TabHeaderPosition::Before)
            .tab_panel_header(|_, _| {})
            .tab_panel_header_position(TabHeaderPosition::None)
            .pre_panel_content(|_, _| {})
            .on_active_index_changed(|_| {})
            .on_preview_interaction(|_| {})
            .id_salt("tabs")
            .classes(["my-tabs"]);

        assert!(matches!(t.position, Position::Bottom));
        assert!(t.centered);
        assert_eq!(t.minimum_tab_width, 200.0);
        assert_eq!(t.max_height, Some(500.0));
        assert!(t.border);
        assert!(matches!(t.color, Color::Secondary));
        assert!(matches!(t.icon_color, Color::Error));
        assert!(matches!(t.slider_color, Color::Success));
        assert!(!t.slider_animation);
        assert!(t.hide_slider);
        assert!(t.outlined && t.rounded);
        assert_eq!(t.elevation, 4);
        assert!(t.apply_effects_to_container);
        assert!(!t.ripple && t.disable_ripple);
        assert!(t.always_show_scroll_buttons);
        assert_eq!(t.prev_icon, "P");
        assert_eq!(t.next_icon, "N");
        assert!(matches!(t.scroll_icon_color, Color::Warning));
        assert!(t.keep_panels_alive);
        assert!(t.right_to_left);
        assert_eq!(t.tab_header_class.as_deref(), Some("th"));
        assert_eq!(t.panel_class.as_deref(), Some("pc"));
        assert_eq!(t.tab_panel_class.as_deref(), Some("tpc"));
        assert_eq!(t.active_tab_class.as_deref(), Some("atc"));
        assert!(t.header.is_some());
        assert_eq!(t.header_position, TabHeaderPosition::Before);
        assert!(t.tab_panel_header.is_some());
        assert_eq!(t.tab_panel_header_position, TabHeaderPosition::None);
        assert!(t.pre_panel_content.is_some());
        assert!(t.on_active_index_changed.is_some());
        assert!(t.on_preview_interaction.is_some());
        assert_eq!(t.id_salt.as_deref(), Some("tabs"));
        assert!(t.base.has_class("my-tabs"));
    }

    #[test]
    fn minimum_tab_width_clamps_negative() {
        let t = MudTabs::new().minimum_tab_width(-50.0);
        assert_eq!(t.minimum_tab_width, 0.0);
    }

    #[test]
    fn collector_preserves_order() {
        let mut c: TabUi<'static> = TabUi { panels: Vec::new() };
        c.text_panel("A");
        c.panel(MudTabPanel::new("B"));
        c.text_panel("C");
        assert_eq!(c.len(), 3);
        assert_eq!(c.panels[0].text.as_deref(), Some("A"));
        assert_eq!(c.panels[1].text.as_deref(), Some("B"));
        assert_eq!(c.panels[2].text.as_deref(), Some("C"));
    }

    #[test]
    fn action_helpers_agree() {
        assert!(TabAction::Activated { from: 0, to: 1 }.changed_active_index());
        assert!(TabAction::Vetoed { target: 2 }.was_vetoed());
        assert_eq!(TabAction::Activated { from: 0, to: 1 }.target_index(), Some(1));
        assert_eq!(TabAction::Vetoed { target: 2 }.target_index(), Some(2));
    }
}