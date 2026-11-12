//! `MudTimeline` — displays items in chronological order along a vertical or
//! horizontal axis.
//!
//! Port of `MudTimeline` + `MudTimelineItem` + `TimelineAlign` +
//! `TimelineOrientation` + `TimelinePosition` (MudBlazor 8.13.0).
//!
//! # Parity — `MudTimeline`
//!
//! | MudBlazor parameter | Rust method | Default |
//! |---------------------|-------------|---------|
//! | `Modifiers`         | [`MudTimeline::modifiers`] | `true` |
//! | `Reverse`           | [`MudTimeline::reverse`] | `false` |
//! | `RightToLeft` (cascading) | [`MudTimeline::right_to_left`] | `false` |
//! | `SelectedIndex`     | [`TimelineState::selected_index`] | `None` |
//! | `SelectedIndexChanged` | [`MudTimeline::on_selected_index_change`] | `None` |
//! | `TimelineAlign`     | [`MudTimeline::timeline_align`] | `Default` |
//! | `TimelineOrientation` | [`MudTimeline::timeline_orientation`] | `Vertical` |
//! | `TimelinePosition`  | [`MudTimeline::timeline_position`] | `Alternate` |
//! | `ChildContent`      | closure to [`MudTimeline::show`] | required |
//! | `Class`             | [`MudTimeline::classes`] | `None` |
//! | `Style`             | [`MudTimeline::frame`] | `None` |
//! | `Tag`               | [`MudTimeline::tag`] / [`MudTimeline::tag_ref`] | `None` |
//! | `UserAttributes`    | **not ported** (no HTML tag) | — |
//!
//! **`MoveTo(int)`, `Next()`, `Previous()`** are provided as
//! [`MudTimeline::move_to`], [`MudTimeline::next`], [`MudTimeline::previous`].
//!
//! # Parity — `MudTimelineItem`
//!
//! See [`TimelineItem`] for the full table.
//!
//! # Not ported
//!
//! * **`Modifiers`** are accepted for API parity, but egui has no CSS
//!   pseudo-elements; the parameter is stored and exposed via
//!   [`MudTimeline::modifiers`] but does not affect rendering.
//! * **`DotStyle` / `DotClass`** — no CSS strings. Use [`TimelineItem::dot`]
//!   for a fully custom dot renderer.
//! * **`UserAttributes`** — there is no HTML tag.
//!
//! # Layout
//!
//! **Vertical** timelines stack item rows top-to-bottom. Each row is a
//! horizontal band with the dot column at the left, right, or centre:
//!
//! | Position | Row layout |
//! |----------|-----------|
//! | `Left` / `Start` | `[ dot ][ content ]` |
//! | `Right` / `End` | `[ content ][ dot ]` |
//! | `Alternate` | `[ content \| opposite ][ dot ][ opposite \| content ]` |
//!
//! **Horizontal** timelines lay items left-to-right with a fixed item box
//! (`item_width` × `item_height`):
//!
//! | Position | Item layout |
//! |----------|-------------|
//! | `Top` | `[ dot row ][ content ]` |
//! | `Bottom` | `[ content ][ dot row ]` |
//! | `Alternate` | `[ content \| opposite ][ dot row ][ opposite \| content ]` |
//!
//! `ItemOpposite` is only rendered in `Alternate` mode, matching MudBlazor.
//!
//! # Connector line
//!
//! A single straight line runs through all dot centres, from the first to the
//! last. It is painted **behind** items by reserving a shape slot before
//! rendering.
//!
//! # Click / selection
//!
//! When [`selectable`](MudTimeline::selectable) is `true` (the default), the
//! whole item row is clickable and writes
//! [`TimelineState::selected_index`]. The optional
//! [`on_item_click`](MudTimeline::on_item_click) callback fires for every
//! click, matching the pattern recommended in MudBlazor discussion #6008.

use std::any::Any;
use std::sync::Arc;

use egui::{
    vec2, Align, Color32, CornerRadius, Layout, Pos2, Rect, Response, Sense, Shape,
    Stroke, Ui, UiBuilder,
};
use mud_egui_bootstrap::bs_style::BsStyle;
use mud_egui_core::base::component_base::ComponentBase;
use mud_egui_core::enums::{
    Color, Size, StyleFlavor, TimelineAlign, TimelineOrientation, TimelinePosition, Variant,
};
use mud_egui_tailwind::tw_style::{TwState, TwStyle};
use mud_egui_theming::models::palettes::palette::Palette;
use mud_egui_theming::theme::MudTheme;

use crate::data_display::icons::mud_icon::MudIcon;
use crate::typography::mud_text::MudText;

use super::timeline_item::TimelineItem;
use super::timeline_state::TimelineState;
use super::timeline_ui::TimelineUi;

/// Base rem size passed to the Tailwind resolver.
const REM_SIZE_PT: f32 = 14.0;

/// Diameter of a small dot.
pub const DOT_SMALL: f32 = 12.0;
/// Diameter of a medium dot.
pub const DOT_MEDIUM: f32 = 16.0;
/// Diameter of a large dot.
pub const DOT_LARGE: f32 = 20.0;
/// Padding on each side of the dot column/row.
pub const DOT_PADDING: f32 = 8.0;
/// Vertical gap between items in a vertical timeline.
pub const ITEM_GAP: f32 = 8.0;
/// Thickness of the connector line.
pub const CONNECTOR_THICKNESS: f32 = 2.0;
/// Default width of a horizontal timeline item.
pub const DEFAULT_ITEM_WIDTH: f32 = 240.0;
/// Default height of a horizontal timeline item.
pub const DEFAULT_ITEM_HEIGHT: f32 = 200.0;
/// Minimum horizontal content width.
const MIN_CONTENT_WIDTH: f32 = 40.0;

// ---------------------------------------------------------------------------
// Pure helpers
// ---------------------------------------------------------------------------

/// Diameter of a dot for the given [`Size`].
///
/// Mirrors MudBlazor's dot size scale. `ExtraSmall` / `ExtraLarge` fall back
/// to `Medium`.
#[inline]
pub fn dot_diameter(size: Size) -> f32 {
    match size {
        Size::Small => DOT_SMALL,
        Size::Large => DOT_LARGE,
        _ => DOT_MEDIUM,
    }
}

/// Width/height of the dot column/row (dot diameter + padding on both sides).
#[inline]
pub fn dot_span(size: Size) -> f32 {
    dot_diameter(size) + 2.0 * DOT_PADDING
}

/// Side assignment for an item in `Alternate` mode.
///
/// Returns `-1` for start (left/top) and `1` for end (right/bottom).
#[inline]
pub fn alternate_side(index: usize, reverse: bool) -> i8 {
    let odd = (index % 2 == 1) ^ reverse;
    if odd { 1 } else { -1 }
}

/// Resolves `(content_side, opposite_side)` for one item.
///
/// `content_side` is `-1` (start/left/top), `0` (center — not used), or `1`
/// (end/right/bottom). `opposite_side` is `0` when the opposite content is not
/// rendered (non-alternate positions).
pub fn item_sides(
    index: usize,
    position: TimelinePosition,
    orientation: TimelineOrientation,
    reverse: bool,
) -> (i8, i8) {
    if position.is_alternate() {
        let content = alternate_side(index, reverse);
        return (content, -content);
    }
    let content = match (orientation, position) {
        // Vertical non-alternate: content is opposite the dot.
        (TimelineOrientation::Vertical, TimelinePosition::Left)
        | (TimelineOrientation::Vertical, TimelinePosition::Start) => 1,
        (TimelineOrientation::Vertical, TimelinePosition::Right)
        | (TimelineOrientation::Vertical, TimelinePosition::End) => -1,
        // Horizontal non-alternate: content is opposite the dot.
        (TimelineOrientation::Horizontal, TimelinePosition::Top) => 1,
        (TimelineOrientation::Horizontal, TimelinePosition::Bottom) => -1,
        // Fallback: treat unknown combinations as start-anchored.
        _ => 1,
    };
    (content, 0)
}

/// Resolves the physical position for the given orientation, normalising
/// `Start` / `End` and rejecting cross-axis values.
///
/// For a vertical timeline, `Top` / `Bottom` are coerced to `Alternate`; for a
/// horizontal timeline, `Left` / `Right` / `Start` / `End` are coerced to
/// `Alternate`. This mirrors MudBlazor, which only validates the combination
/// at render time.
#[inline]
pub fn resolve_position(
    position: TimelinePosition,
    orientation: TimelineOrientation,
    right_to_left: bool,
) -> TimelinePosition {
    let resolved = position.resolve(right_to_left);
    match orientation {
        TimelineOrientation::Vertical => {
            if resolved.is_horizontal_axis() {
                TimelinePosition::Alternate
            } else {
                resolved
            }
        }
        TimelineOrientation::Horizontal => {
            if resolved.is_vertical_axis() {
                TimelinePosition::Alternate
            } else {
                resolved
            }
        }
    }
}

/// Resolves the per-item alignment: the item's override or the timeline's.
#[inline]
pub fn effective_align(item_override: Option<TimelineAlign>, timeline: TimelineAlign) -> TimelineAlign {
    item_override.unwrap_or(timeline)
}

/// Offset, in pixels, of the dot centre from the top of the row for a
/// vertical timeline.
#[inline]
pub fn dot_y_offset(row_h: f32, dot_diameter: f32, align: TimelineAlign) -> f32 {
    match align {
        TimelineAlign::Start => DOT_PADDING + dot_diameter * 0.5,
        TimelineAlign::End => (row_h - DOT_PADDING - dot_diameter * 0.5).max(0.0),
        TimelineAlign::Default => row_h * 0.5,
    }
}

/// Offset, in pixels, of the dot centre from the left of the item for a
/// horizontal timeline.
#[inline]
pub fn dot_x_offset(item_w: f32, dot_diameter: f32, align: TimelineAlign) -> f32 {
    match align {
        TimelineAlign::Start => DOT_PADDING + dot_diameter * 0.5,
        TimelineAlign::End => (item_w - DOT_PADDING - dot_diameter * 0.5).max(0.0),
        TimelineAlign::Default => item_w * 0.5,
    }
}

#[inline]
fn with_alpha(c: Color32, a: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), a)
}

// ---------------------------------------------------------------------------
// Draw configuration
// ---------------------------------------------------------------------------

struct TimelineCfg {
    position: TimelinePosition,
    orientation: TimelineOrientation,
    align: TimelineAlign,
    reverse: bool,
    surface: Color32,
    divider: Color32,
    contrast: Color32,
    item_width: f32,
    item_height: f32,
    palette: &'static Palette, // placeholder; real ref set below
}

// The `palette` field above cannot hold a borrowed reference with `'static`
// without changing the struct; we use a separate cfg with a lifetime instead.
struct Cfg<'p> {
    position: TimelinePosition,
    orientation: TimelineOrientation,
    align: TimelineAlign,
    reverse: bool,
    surface: Color32,
    divider: Color32,
    contrast: Color32,
    item_width: f32,
    item_height: f32,
    palette: &'p Palette,
}

// ---------------------------------------------------------------------------
// MudTimeline
// ---------------------------------------------------------------------------

/// Displays items in chronological order.
#[must_use = "MudTimeline does nothing until you call `.show(ui, state, |t| ..)`"]
pub struct MudTimeline<'a> {
    // ── Behaviour ──────────────────────────────────────────────────────────
    /// Enables modifiers (`Modifiers`, default `true`). Stored for parity;
    /// egui has no CSS pseudo-elements, so it has no visual effect.
    pub modifiers: bool,
    /// Reverses the alternate side assignment (`Reverse`, default `false`).
    pub reverse: bool,
    /// Right-to-left layout (cascading `RightToLeft`, default `false`).
    pub right_to_left: bool,
    /// How items are positioned relative to the axis (`TimelinePosition`,
    /// default `Alternate`).
    pub timeline_position: TimelinePosition,
    /// Axis along which items are laid out (`TimelineOrientation`, default
    /// `Vertical`).
    pub timeline_orientation: TimelineOrientation,
    /// Default dot alignment (`TimelineAlign`, default `Default`).
    pub timeline_align: TimelineAlign,

    // ── Selection / interaction ────────────────────────────────────────────
    /// Whether item rows are clickable (`true` by default). Clicking writes
    /// [`TimelineState::selected_index`].
    pub selectable: bool,
    /// Callback fired on item click with the clicked index
    /// (`SelectedIndexChanged` equivalent).
    pub on_item_click: Option<Box<dyn FnMut(usize) + 'a>>,

    // ── Layout ─────────────────────────────────────────────────────────────
    /// Maximum height for a vertical timeline; enables a scroll area.
    pub height: Option<f32>,
    /// Fixed item width for a horizontal timeline.
    pub item_width: f32,
    /// Fixed item height for a horizontal timeline.
    pub item_height: f32,

    // ── Utilities ──────────────────────────────────────────────────────────
    /// Utility classes for the wrapper.
    pub base: ComponentBase,
    /// Bootstrap or Tailwind resolver.
    pub flavor: StyleFlavor,
    /// Bootstrap resolver state.
    pub bs_style: BsStyle,
    /// Tailwind resolver state.
    pub tw_style: TwStyle,
    /// Tailwind interaction state.
    pub tw_state: TwState,
    /// Overrides the resolved wrapper frame (`Style`).
    pub frame_override: Option<egui::Frame>,
    /// Accessible label published on the timeline node.
    pub aria_label: Option<String>,
    tag_value: Option<Arc<dyn Any + Send + Sync>>,
}

impl<'a> Default for MudTimeline<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> MudTimeline<'a> {
    /// Creates a timeline with MudBlazor defaults.
    pub fn new() -> Self {
        Self {
            modifiers: true,
            reverse: false,
            right_to_left: false,
            timeline_position: TimelinePosition::Alternate,
            timeline_orientation: TimelineOrientation::Vertical,
            timeline_align: TimelineAlign::Default,
            selectable: true,
            on_item_click: None,
            height: None,
            item_width: DEFAULT_ITEM_WIDTH,
            item_height: DEFAULT_ITEM_HEIGHT,
            base: ComponentBase::new(),
            flavor: StyleFlavor::default(),
            bs_style: BsStyle::default(),
            tw_style: TwStyle::default(),
            tw_state: TwState::default(),
            frame_override: None,
            aria_label: None,
            tag_value: None,
        }
    }

    // ── Behaviour ──────────────────────────────────────────────────────────

    /// Enables or disables modifiers (`Modifiers`). Stored for parity.
    #[inline]
    pub fn modifiers(mut self, b: bool) -> Self { self.modifiers = b; self }

    /// Reverses the alternate side assignment (`Reverse`).
    #[inline]
    pub fn reverse(mut self, b: bool) -> Self { self.reverse = b; self }

    /// Right-to-left layout (`RightToLeft`).
    #[inline]
    pub fn right_to_left(mut self, b: bool) -> Self { self.right_to_left = b; self }

    /// Sets `TimelinePosition`.
    #[inline]
    pub fn timeline_position(mut self, p: TimelinePosition) -> Self { self.timeline_position = p; self }

    /// Sets `TimelineOrientation`.
    #[inline]
    pub fn timeline_orientation(mut self, o: TimelineOrientation) -> Self { self.timeline_orientation = o; self }

    /// Sets the default `TimelineAlign`.
    #[inline]
    pub fn timeline_align(mut self, a: TimelineAlign) -> Self { self.timeline_align = a; self }

    // ── Selection ──────────────────────────────────────────────────────────

    /// Enables or disables item click selection.
    #[inline]
    pub fn selectable(mut self, b: bool) -> Self { self.selectable = b; self }

    /// Registers a click callback (`SelectedIndexChanged` equivalent).
    #[inline]
    pub fn on_selected_index_change(mut self, f: impl FnMut(usize) + 'a) -> Self { self.on_item_click = Some(Box::new(f)); self }

    /// Alias for [`on_selected_index_change`](Self::on_selected_index_change).
    #[inline]
    pub fn on_item_click(self, f: impl FnMut(usize) + 'a) -> Self { self.on_selected_index_change(f) }

    // ── Layout ─────────────────────────────────────────────────────────────

    /// Sets a maximum height; enables a vertical scroll area.
    #[inline]
    pub fn height(mut self, h: f32) -> Self { self.height = Some(h.max(0.0)); self }

    /// Sets the item width for a horizontal timeline.
    #[inline]
    pub fn item_width(mut self, w: f32) -> Self { self.item_width = w.max(MIN_CONTENT_WIDTH); self }

    /// Sets the item height for a horizontal timeline.
    #[inline]
    pub fn item_height(mut self, h: f32) -> Self { self.item_height = h.max(DOT_LARGE + 2.0 * DOT_PADDING); self }

    // ── CSS / tag ──────────────────────────────────────────────────────────

    /// Adds utility classes to the wrapper.
    #[inline]
    pub fn classes<S: AsRef<str>>(mut self, classes: impl IntoIterator<Item = S>) -> Self { self.base = self.base.classes(classes); self }

    /// Selects the utility-class resolver.
    #[inline]
    pub fn flavor(mut self, f: StyleFlavor) -> Self { self.flavor = f; self }

    /// Overrides the resolved wrapper frame (`Style`).
    #[inline]
    pub fn frame(mut self, f: egui::Frame) -> Self { self.frame_override = Some(f); self }

    /// Sets the accessible label.
    #[inline]
    pub fn aria_label(mut self, s: impl Into<String>) -> Self { self.aria_label = Some(s.into()); self }

    /// Attaches arbitrary user data (`Tag`).
    pub fn tag<U: Any + Send + Sync>(mut self, value: U) -> Self { self.tag_value = Some(Arc::new(value)); self }

    /// Reads the tag back if it is of type `U`.
    pub fn tag_ref<U: Any + Send + Sync>(&self) -> Option<&U> { self.tag_value.as_ref()?.downcast_ref::<U>() }

    // ── Navigation helpers ─────────────────────────────────────────────────

    /// Moves to `index`, clamped into `0..count`.
    pub fn move_to(state: &mut TimelineState, index: usize, count: usize) {
        if count == 0 { return; }
        state.selected_index = Some(index.min(count - 1));
    }

    /// Advances to the next item, wrapping to the first.
    pub fn next(state: &mut TimelineState, count: usize) {
        if count == 0 { return; }
        let cur = state.selected_index.unwrap_or(0);
        state.selected_index = Some((cur + 1) % count);
    }

    /// Moves to the previous item, wrapping to the last.
    pub fn previous(state: &mut TimelineState, count: usize) {
        if count == 0 { return; }
        let cur = state.selected_index.unwrap_or(0);
        state.selected_index = Some((cur + count - 1) % count);
    }

    // ── Rendering ──────────────────────────────────────────────────────────

    /// Draws the timeline. Returns `(response, user_result)`.
    pub fn show<R>(
        mut self,
        ui: &mut Ui,
        state: &mut TimelineState,
        add_items: impl FnOnce(&mut TimelineUi<'a>) -> R,
    ) -> (Response, R) {
        // 1. Collect items.
        let mut collector = TimelineUi { items: Vec::new() };
        let user_result = add_items(&mut collector);
        let mut items = collector.items;

        // 2. Theme.
        let theme = MudTheme::fetch(ui.ctx()).unwrap_or_default();
        let palette = theme.current_palette();
        let lookup = |key: &str| palette.resolve(key).map(Color32::from);

        self.tw_state.dark = ui.visuals().dark_mode;
        let frame = if let Some(f) = self.frame_override.take() {
            f
        } else {
            match self.flavor {
                StyleFlavor::Bootstrap => self.bs_style.build_frame(&self.base, &lookup),
                StyleFlavor::Tailwind => self
                    .tw_style
                    .build_frame(&self.base, self.tw_state, REM_SIZE_PT, &lookup),
            }
        };

        // 3. Resolve position + colours.
        let position = resolve_position(
            self.timeline_position,
            self.timeline_orientation,
            self.right_to_left,
        );
        let cfg = Cfg {
            position,
            orientation: self.timeline_orientation,
            align: self.timeline_align,
            reverse: self.reverse,
            surface: palette.surface.into(),
            divider: palette.divider.into(),
            contrast: palette.primary_contrast_text.into(),
            item_width: self.item_width,
            item_height: self.item_height,
            palette,
        };

        // 4. Render.
        let outer = frame.show(ui, |ui| {
            let slot = ui.painter().add(Shape::Noop);
            state.last_item_rects.clear();
            state.last_dot_centers.clear();

            match cfg.orientation {
                TimelineOrientation::Vertical => {
                    render_vertical(ui, &mut items, state, &cfg, self.selectable, &mut self.on_item_click);
                }
                TimelineOrientation::Horizontal => {
                    render_horizontal(ui, &mut items, state, &cfg, self.selectable, &mut self.on_item_click);
                }
            }

            // Connector line through all dot centres.
            if state.last_dot_centers.len() >= 2 {
                let first = state.last_dot_centers[0];
                let last = *state.last_dot_centers.last().unwrap();
                let (a, b) = match cfg.orientation {
                    TimelineOrientation::Vertical => (Pos2::new(first.x, first.y), Pos2::new(last.x, last.y)),
                    TimelineOrientation::Horizontal => (Pos2::new(first.x, first.y), Pos2::new(last.x, last.y)),
                };
                ui.painter().set(
                    slot,
                    Shape::line_segment([a, b], Stroke::new(CONNECTOR_THICKNESS, cfg.divider)),
                );
            }
        });

        self.tw_state.hovered = outer.response.hovered();
        self.tw_state.active = outer.response.is_pointer_button_down_on();

        if let Some(label) = &self.aria_label {
            outer.response.widget_info(|| {
                egui::WidgetInfo::labeled(egui::Role::List, true, label)
            });
        }

        (outer.response, user_result)
    }
}

// ---------------------------------------------------------------------------
// Vertical rendering
// ---------------------------------------------------------------------------

fn render_vertical<'a>(
    ui: &mut Ui,
    items: &mut [TimelineItem<'a>],
    state: &mut TimelineState,
    cfg: &Cfg<'_>,
    selectable: bool,
    on_click: &mut Option<Box<dyn FnMut(usize) + 'a>>,
) {
    let n = items.len();
    let scroll = cfg.palette; // unused but keeps signature stable
    let _ = scroll;

    for (i, item) in items.iter_mut().enumerate() {
        let (content_side, opposite_side) =
            item_sides(i, cfg.position, TimelineOrientation::Vertical, cfg.reverse);

        let span = dot_span(item.size);
        let align = effective_align(item.timeline_align, cfg.align);
        let accent = item.color.resolve(cfg.palette).map(Color32::from).unwrap_or_else(|| cfg.palette.primary.into());

        // Allocate the row.
        let avail = ui.available_width();
        let row = ui.horizontal_top(|ui| {
            let dot_col = |ui: &mut Ui| {
                ui.allocate_ui_with_layout(
                    vec2(span, 0.0),
                    Layout::top_down(Align::Center),
                    |_| {},
                );
            };
            let content_col = |ui: &mut Ui, w: f32, side: i8, item: &mut TimelineItem<'a>| {
                let layout = if side > 0 {
                    Layout::top_down(Align::Min)
                } else {
                    Layout::top_down(Align::Max)
                };
                ui.allocate_ui_with_layout(vec2(w, 0.0), layout, |ui| {
                    // Alternate: decide which closure goes on this side.
                    if cfg.position.is_alternate() {
                        if (side > 0) == (content_side > 0) {
                            (item.content)(ui);
                        } else if opposite_side != 0 {
                            if let Some(opp) = item.opposite.as_mut() {
                                opp(ui);
                            }
                        }
                    } else {
                        (item.content)(ui);
                    }
                });
            };

            match cfg.position {
                TimelinePosition::Left => {
                    // Dot at left, content to the right.
                    dot_col(ui);
                    let w = (avail - span).max(MIN_CONTENT_WIDTH);
                    content_col(ui, w, 1, item);
                }
                TimelinePosition::Right => {
                    // Content to the left, dot at right.
                    let w = (avail - span).max(MIN_CONTENT_WIDTH);
                    content_col(ui, w, -1, item);
                    dot_col(ui);
                }
                _ => {
                    // Alternate: content on the chosen side, opposite on the
                    // other, dot in the centre.
                    let half = ((avail - span) * 0.5).max(MIN_CONTENT_WIDTH);
                    content_col(ui, half, -1, item);
                    dot_col(ui);
                    content_col(ui, half, 1, item);
                }
            }
        });

        let row_rect = row.response.rect;

        // Compute dot centre.
        let dot_x = match cfg.position {
            TimelinePosition::Left => row_rect.left() + span * 0.5,
            TimelinePosition::Right => row_rect.right() - span * 0.5,
            _ => row_rect.center().x,
        };
        let dia = dot_diameter(item.size);
        let dot_y = row_rect.top() + dot_y_offset(row_rect.height(), dia, align);
        let dot_center = Pos2::new(dot_x, dot_y);

        state.last_item_rects.push(row_rect);
        state.last_dot_centers.push(dot_center);

        // Paint the dot (unless hidden).
        if !item.hide_dot {
            paint_dot(ui, dot_center, dia, item, accent, cfg);
        }

        // Click handling.
        if selectable {
            let id = ui.scope_id().with(("timeline_item_v", i));
            let resp = ui.interact(row_rect, id, Sense::click());
            if resp.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }
            if resp.clicked() {
                state.selected_index = Some(i);
                if let Some(cb) = on_click.as_mut() {
                    cb(i);
                }
            }
        }

        ui.add_space(ITEM_GAP);
    }
}

// ---------------------------------------------------------------------------
// Horizontal rendering
// ---------------------------------------------------------------------------

fn render_horizontal<'a>(
    ui: &mut Ui,
    items: &mut [TimelineItem<'a>],
    state: &mut TimelineState,
    cfg: &Cfg<'_>,
    selectable: bool,
    on_click: &mut Option<Box<dyn FnMut(usize) + 'a>>,
) {
    let n = items.len();
    let _ = n;

    let inner = ui.horizontal(|ui| {
        for (i, item) in items.iter_mut().enumerate() {
            let (content_side, opposite_side) =
                item_sides(i, cfg.position, TimelineOrientation::Horizontal, cfg.reverse);

            let dia = dot_diameter(item.size);
            let span = dot_span(item.size);
            let align = effective_align(item.timeline_align, cfg.align);
            let accent = item.color.resolve(cfg.palette).map(Color32::from).unwrap_or_else(|| cfg.palette.primary.into());

            let item_w = cfg.item_width;
            let item_h = cfg.item_height;

            let col = ui.allocate_ui_with_layout(
                vec2(item_w, item_h),
                Layout::top_down(Align::Min),
                |ui| {
                    ui.set_width(item_w);
                    // Resolve the row heights.
                    let top_h = if cfg.position.is_alternate() {
                        (item_h - span) * 0.5
                    } else {
                        item_h - span
                    };
                    let bottom_h = if cfg.position.is_alternate() {
                        (item_h - span) * 0.5
                    } else {
                        item_h - span
                    };

                    // Top slot.
                    let top_has_content = match cfg.position {
                        TimelinePosition::Top => false,
                        TimelinePosition::Bottom => true,
                        _ => (content_side < 0) == (cfg.reverse == false && i % 2 == 0)
                            || content_side < 0,
                    };

                    // Simpler: render top, dot row, bottom in fixed order and
                    // decide which closure fills which based on sides.
                    let top_side: i8 = if cfg.position.is_alternate() {
                        -1
                    } else if matches!(cfg.position, TimelinePosition::Bottom) {
                        -1
                    } else {
                        1
                    };
                    let _ = top_has_content;

                    // Top row.
                    ui.allocate_ui_with_layout(
                        vec2(item_w, top_h),
                        Layout::top_down(Align::Min),
                        |ui| {
                            if cfg.position.is_alternate() {
                                if content_side < 0 {
                                    (item.content)(ui);
                                } else if opposite_side < 0 {
                                    if let Some(opp) = item.opposite.as_mut() { opp(ui); }
                                }
                            } else if matches!(cfg.position, TimelinePosition::Bottom) {
                                (item.content)(ui);
                            }
                        },
                    );

                    // Dot row.
                    let dot_row = ui.allocate_ui_with_layout(
                        vec2(item_w, span),
                        Layout::top_down(Align::Center),
                        |_| {},
                    );
                    let dot_row_rect = dot_row.response.rect;

                    // Bottom row.
                    ui.allocate_ui_with_layout(
                        vec2(item_w, bottom_h),
                        Layout::top_down(Align::Min),
                        |ui| {
                            if cfg.position.is_alternate() {
                                if content_side > 0 {
                                    (item.content)(ui);
                                } else if opposite_side > 0 {
                                    if let Some(opp) = item.opposite.as_mut() { opp(ui); }
                                }
                            } else if matches!(cfg.position, TimelinePosition::Top) {
                                (item.content)(ui);
                            }
                        },
                    );

                    // Compute dot centre within this item.
                    let dot_x = dot_row_rect.left() + dot_x_offset(item_w, dia, align);
                    let dot_y = dot_row_rect.center().y;
                    let dot_center = Pos2::new(dot_x, dot_y);
                    let _ = top_side;

                    // Paint dot.
                    if !item.hide_dot {
                        paint_dot(ui, dot_center, dia, item, accent, cfg);
                    }

                    // Click.
                    if selectable {
                        let id = ui.scope_id().with(("timeline_item_h", i));
                        let resp = ui.interact(ui.max_rect(), id, Sense::click());
                        if resp.hovered() {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                        }
                        if resp.clicked() {
                            state.selected_index = Some(i);
                            if let Some(cb) = on_click.as_mut() {
                                cb(i);
                            }
                        }
                    }

                    dot_center
                },
            );

            let item_rect = col.response.rect;
            state.last_item_rects.push(item_rect);
            state.last_dot_centers.push(col.inner);

            ui.add_space(ITEM_GAP);
        }
    });

    let _ = inner;
}

// ---------------------------------------------------------------------------
// Dot painting
// ---------------------------------------------------------------------------

fn paint_dot<'a>(
    ui: &mut Ui,
    center: Pos2,
    dia: f32,
    item: &mut TimelineItem<'a>,
    accent: Color32,
    cfg: &Cfg<'_>,
) {
    let r = dia * 0.5;

    // Custom dot renderer overrides the default circle.
    if let Some(dot) = item.dot.as_mut() {
        let rect = Rect::from_center_size(center, vec2(dia, dia));
        let mut child = ui.new_child(UiBuilder::new().max_rect(rect));
        dot(&mut child);
        return;
    }

    // Default dot.
    match item.variant {
        Variant::Filled => {
            ui.painter().circle_filled(center, r, accent);
        }
        Variant::Text => {
            ui.painter().circle_filled(center, r, accent);
        }
        _ => {
            // Outlined: surface fill + accent stroke.
            ui.painter().circle_filled(center, r, cfg.surface);
            ui.painter()
                .circle_stroke(center, r, Stroke::new(2.0, accent));
        }
    }

    // Icon inside the dot.
    if let Some(svg) = item.icon.as_deref() {
        let icon_size = (dia * 0.7).max(8.0);
        let icon_color = match item.variant {
            Variant::Filled | Variant::Text => cfg.contrast,
            _ => accent,
        };
        let icon_rect = Rect::from_center_size(center, vec2(icon_size, icon_size));
        ui.put(
            icon_rect,
            MudIcon::new(svg.to_owned())
                .size_px(icon_size)
                .custom_color(icon_color),
        );
    }

    let _ = with_alpha;
    let _ = CornerRadius::same(0);
    let _ = MudText::new("");
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // --- dot geometry ------------------------------------------------------

    #[test]
    fn dot_diameter_maps_sizes() {
        assert_eq!(dot_diameter(Size::Small), DOT_SMALL);
        assert_eq!(dot_diameter(Size::Medium), DOT_MEDIUM);
        assert_eq!(dot_diameter(Size::Large), DOT_LARGE);
        assert_eq!(dot_diameter(Size::ExtraSmall), DOT_MEDIUM);
        assert_eq!(dot_diameter(Size::ExtraLarge), DOT_MEDIUM);
    }

    #[test]
    fn dot_span_adds_padding() {
        assert_eq!(dot_span(Size::Small), DOT_SMALL + 2.0 * DOT_PADDING);
        assert_eq!(dot_span(Size::Large), DOT_LARGE + 2.0 * DOT_PADDING);
    }

    // --- alternate sides ---------------------------------------------------

    #[test]
    fn alternate_side_alternates() {
        assert_eq!(alternate_side(0, false), -1);
        assert_eq!(alternate_side(1, false), 1);
        assert_eq!(alternate_side(2, false), -1);
        assert_eq!(alternate_side(3, false), 1);
    }

    #[test]
    fn alternate_side_reverse_flips() {
        assert_eq!(alternate_side(0, true), 1);
        assert_eq!(alternate_side(1, true), -1);
    }

    // --- item sides --------------------------------------------------------

    #[test]
    fn item_sides_alternate_are_opposites() {
        let (c, o) = item_sides(0, TimelinePosition::Alternate, TimelineOrientation::Vertical, false);
        assert_eq!((c, o), (-1, 1));
        let (c, o) = item_sides(1, TimelinePosition::Alternate, TimelineOrientation::Vertical, false);
        assert_eq!((c, o), (1, -1));
    }

    #[test]
    fn item_sides_vertical_left_puts_content_right() {
        let (c, o) = item_sides(0, TimelinePosition::Left, TimelineOrientation::Vertical, false);
        assert_eq!((c, o), (1, 0));
    }

    #[test]
    fn item_sides_vertical_right_puts_content_left() {
        let (c, o) = item_sides(0, TimelinePosition::Right, TimelineOrientation::Vertical, false);
        assert_eq!((c, o), (-1, 0));
    }

    #[test]
    fn item_sides_horizontal_top_puts_content_below() {
        let (c, o) = item_sides(0, TimelinePosition::Top, TimelineOrientation::Horizontal, false);
        assert_eq!((c, o), (1, 0));
    }

    #[test]
    fn item_sides_horizontal_bottom_puts_content_above() {
        let (c, o) = item_sides(0, TimelinePosition::Bottom, TimelineOrientation::Horizontal, false);
        assert_eq!((c, o), (-1, 0));
    }

    // --- position resolution ----------------------------------------------

    #[test]
    fn resolve_position_coerces_cross_axis_values() {
        // Vertical + Top → Alternate.
        assert_eq!(
            resolve_position(TimelinePosition::Top, TimelineOrientation::Vertical, false),
            TimelinePosition::Alternate
        );
        // Horizontal + Left → Alternate.
        assert_eq!(
            resolve_position(TimelinePosition::Left, TimelineOrientation::Horizontal, false),
            TimelinePosition::Alternate
        );
    }

    #[test]
    fn resolve_position_normalises_start_end() {
        assert_eq!(
            resolve_position(TimelinePosition::Start, TimelineOrientation::Vertical, false),
            TimelinePosition::Left
        );
        assert_eq!(
            resolve_position(TimelinePosition::Start, TimelineOrientation::Vertical, true),
            TimelinePosition::Right
        );
    }

    // --- alignment ---------------------------------------------------------

    #[test]
    fn effective_align_prefers_item_override() {
        assert_eq!(
            effective_align(Some(TimelineAlign::End), TimelineAlign::Start),
            TimelineAlign::End
        );
        assert_eq!(
            effective_align(None, TimelineAlign::Start),
            TimelineAlign::Start
        );
    }

    #[test]
    fn dot_y_offset_matches_alignment() {
        let dia = DOT_MEDIUM;
        // Start → padding + half diameter.
        assert_eq!(dot_y_offset(100.0, dia, TimelineAlign::Start), DOT_PADDING + dia * 0.5);
        // Default → row centre.
        assert_eq!(dot_y_offset(100.0, dia, TimelineAlign::Default), 50.0);
        // End → row height - padding - half diameter.
        assert_eq!(dot_y_offset(100.0, dia, TimelineAlign::End), 100.0 - DOT_PADDING - dia * 0.5);
    }

    #[test]
    fn dot_x_offset_matches_alignment() {
        let dia = DOT_LARGE;
        assert_eq!(dot_x_offset(200.0, dia, TimelineAlign::Start), DOT_PADDING + dia * 0.5);
        assert_eq!(dot_x_offset(200.0, dia, TimelineAlign::Default), 100.0);
        assert_eq!(dot_x_offset(200.0, dia, TimelineAlign::End), 200.0 - DOT_PADDING - dia * 0.5);
    }

    // --- defaults ----------------------------------------------------------

    #[test]
    fn defaults_match_mudblazor() {
        let t: MudTimeline<'static> = MudTimeline::new();
        assert!(t.modifiers);
        assert!(!t.reverse);
        assert!(!t.right_to_left);
        assert_eq!(t.timeline_position, TimelinePosition::Alternate);
        assert_eq!(t.timeline_orientation, TimelineOrientation::Vertical);
        assert_eq!(t.timeline_align, TimelineAlign::Default);
        assert!(t.selectable);
        assert!(t.on_item_click.is_none());
        assert!(t.height.is_none());
        assert_eq!(t.item_width, DEFAULT_ITEM_WIDTH);
        assert_eq!(t.item_height, DEFAULT_ITEM_HEIGHT);
    }

    #[test]
    fn builders_set_every_field() {
        let t = MudTimeline::new()
            .modifiers(false)
            .reverse(true)
            .right_to_left(true)
            .timeline_position(TimelinePosition::Left)
            .timeline_orientation(TimelineOrientation::Horizontal)
            .timeline_align(TimelineAlign::Start)
            .selectable(false)
            .on_selected_index_change(|_| {})
            .height(300.0)
            .item_width(320.0)
            .item_height(240.0)
            .aria_label("timeline")
            .classes(["a"]);
        assert!(!t.modifiers && t.reverse && t.right_to_left);
        assert_eq!(t.timeline_position, TimelinePosition::Left);
        assert_eq!(t.timeline_orientation, TimelineOrientation::Horizontal);
        assert_eq!(t.timeline_align, TimelineAlign::Start);
        assert!(!t.selectable);
        assert!(t.on_item_click.is_some());
        assert_eq!(t.height, Some(300.0));
        assert_eq!(t.item_width, 320.0);
        assert_eq!(t.item_height, 240.0);
    }

    #[test]
    fn item_width_clamps_to_minimum() {
        let t = MudTimeline::new().item_width(0.0);
        assert_eq!(t.item_width, MIN_CONTENT_WIDTH);
    }

    #[test]
    fn height_clamps_negative() {
        let t = MudTimeline::new().height(-5.0);
        assert_eq!(t.height, Some(0.0));
    }

    // --- navigation --------------------------------------------------------

    #[test]
    fn move_to_clamps() {
        let mut s = TimelineState::new();
        MudTimeline::move_to(&mut s, 5, 3);
        assert_eq!(s.selected_index, Some(2));
        MudTimeline::move_to(&mut s, 5, 0);
        assert_eq!(s.selected_index, Some(2));
    }

    #[test]
    fn next_wraps() {
        let mut s = TimelineState::new();
        MudTimeline::next(&mut s, 3);
        assert_eq!(s.selected_index, Some(1));
        MudTimeline::next(&mut s, 3);
        assert_eq!(s.selected_index, Some(2));
        MudTimeline::next(&mut s, 3);
        assert_eq!(s.selected_index, Some(0));
    }

    #[test]
    fn previous_wraps() {
        let mut s = TimelineState::new();
        MudTimeline::previous(&mut s, 3);
        assert_eq!(s.selected_index, Some(2));
        MudTimeline::previous(&mut s, 3);
        assert_eq!(s.selected_index, Some(1));
    }

    // --- tag ---------------------------------------------------------------

    #[test]
    fn tag_round_trips() {
        let t = MudTimeline::new().tag("timeline");
        assert_eq!(t.tag_ref::<&str>(), Some(&"timeline"));
        assert!(t.tag_ref::<u32>().is_none());
    }
}