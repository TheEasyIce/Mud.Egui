//! `MudCarousel` — shows one item at a time with arrows, bullets, swipe, and
//! optional auto-cycling.
//!
//! Port of `MudCarousel<TData>.razor.cs` (MudBlazor 8.13.0). Public
//! parameter surface:
//!
//! | MudBlazor parameter | Rust method | Default |
//! |---------------------|-------------|---------|
//! | `ArrowsPosition`    | [`MudCarousel::arrows_position`] | [`Position::Center`] |
//! | `AutoCycle`         | [`MudCarousel::auto_cycle`] | `false` |
//! | `AutoCycleTime`     | [`MudCarousel::auto_cycle_time`] | `Duration::ZERO` |
//! | `BulletTemplate`    | [`MudCarousel::bullet_template`] | `None` |
//! | `BulletsClass`      | [`MudCarousel::bullets_class`] | `None` |
//! | `BulletsColor`      | [`MudCarousel::bullets_color`] | `None` |
//! | `BulletsPosition`   | [`MudCarousel::bullets_position`] | [`Position::Bottom`] |
//! | `CheckedIcon`       | [`MudCarousel::checked_icon`] | `RADIO_BUTTON_CHECKED` |
//! | `EnableSwipeGesture`| [`MudCarousel::enable_swipe_gesture`] | `true` |
//! | `NavigationButtonsClass` | [`MudCarousel::navigation_buttons_class`] | `None` |
//! | `NextButtonTemplate`| [`MudCarousel::next_button_template`] | `None` |
//! | `NextIcon`          | [`MudCarousel::next_icon`] | `NAVIGATE_NEXT` |
//! | `PreviousButtonTemplate` | [`MudCarousel::previous_button_template`] | `None` |
//! | `PreviousIcon`      | [`MudCarousel::previous_icon`] | `NAVIGATE_BEFORE` |
//! | `RightToLeft`       | [`MudCarousel::right_to_left`] | `false` |
//! | `ShowArrows`        | [`MudCarousel::show_arrows`] | `true` |
//! | `ShowBullets`       | [`MudCarousel::show_bullets`] | `true` |
//! | `UncheckedIcon`     | [`MudCarousel::unchecked_icon`] | `RADIO_BUTTON_UNCHECKED` |
//! | `SelectedIndex`     | [`MudCarousel::selected_index`] | `0` |
//!
//! # Auto-cycle
//!
//! When [`auto_cycle`](MudCarousel::auto_cycle) is `true` **and**
//! [`auto_cycle_time`](MudCarousel::auto_cycle_time) is greater than
//! `Duration::ZERO`, the carousel advances every `auto_cycle_time`. This
//! matches MudBlazor exactly: a zero `AutoCycleTime` disables cycling even
//! when `AutoCycle` is `true`.
//!
//! Timing persists across frames via [`egui::Memory`] keyed by the carousel's
//! response id, so rebuilding the `MudCarousel` every frame (the builder
//! pattern) does not reset the clock.
//!
//! # Swipe
//!
//! Drag tracking records the pointer press-origin and the last in-drag
//! position in [`egui::Memory`]. On release, the total horizontal delta is
//! compared against [`SWIPE_THRESHOLD`]. This is correct where the original
//! `Response::drag_delta()`-on-`drag_stopped()` approach silently failed
//! (that method returns the per-frame delta, which is near zero on release).
//!
//! # Visible items
//!
//! [`MudCarouselItem::visible`] is an extension (MudBlazor's `Visible` is
//! get-only and always `true` for live items). Invisible items are skipped for
//! rendering, navigation, and bullets. `selected_index` refers to the
//! **visible** list, so if no items are hidden it matches the raw index
//! (backward compatible).
//!
//! # Utility classes
//!
//! `MudCarousel` accepts a [`ComponentBase`], [`StyleFlavor`], and the
//! Bootstrap / Tailwind resolvers, so `classes(["..."])` styles the carousel
//! wrapper exactly like other `mud_egui` components.

use std::time::Duration;

use egui::{vec2, Color32, CursorIcon, Pos2, Rect, Response, Sense, Ui, Vec2};
use mud_egui_bootstrap::bs_style::BsStyle;
use mud_egui_core::base::component_base::ComponentBase;
use mud_egui_core::enums::{Color, Position, StyleFlavor, Transition};
use mud_egui_icons_material::filled;
use mud_egui_tailwind::tw_style::{TwState, TwStyle};
use mud_egui_theming::theme::MudTheme;

use crate::data_display::carousel::mud_carousel_item::MudCarouselItem;
use crate::data_display::icons::mud_icon::MudIcon;

/// Minimum horizontal swipe (in points) to trigger a slide change.
pub(crate) const SWIPE_THRESHOLD: f32 = 50.0;

/// Opacity applied to arrows and bullets (matching MudBlazor).
pub(crate) const OVERLAY_OPACITY: f32 = 0.75;

/// Default carousel height when [`MudCarousel::height`] is not set.
pub(crate) const DEFAULT_HEIGHT: f32 = 200.0;

/// Size of the arrow hit-rects, in logical pixels.
pub(crate) const ARROW_SIZE: f32 = 40.0;

/// Size of one bullet, in logical pixels.
pub(crate) const BULLET_SIZE: f32 = 10.0;

/// Horizontal/vertical gap between bullets.
pub(crate) const BULLET_SPACING: f32 = 4.0;

/// Edge padding for arrows and bullets.
pub(crate) const EDGE_PADDING: f32 = 8.0;

/// Base rem size (in points) passed to the Tailwind resolver.
const REM_SIZE_PT: f32 = 14.0;

#[inline]
fn resolve_color(c: Color, theme: &MudTheme, default: Color32) -> Color32 {
    let palette = theme.current_palette();
    match c {
        Color::Default | Color::Inherit => default,
        other => other.resolve(palette).unwrap_or(default),
    }
}

// =============================================================================
// Pure helpers (unit-testable without egui::Context)
// =============================================================================

/// Vertical position of the arrow row for a given [`Position`].
///
/// Only `Top`, `Center`, and `Bottom` are meaningful for arrows; `Left`,
/// `Right`, `Start`, `End` fall back to `Center` (MudBlazor behaves the same
/// way: arrows live on the left/right edges and only their vertical placement
/// is configurable).
#[inline]
pub(crate) fn arrow_y(rect: Rect, size: f32, position: Position) -> f32 {
    match position {
        Position::Top => rect.top() + EDGE_PADDING,
        Position::Bottom => rect.bottom() - size - EDGE_PADDING,
        _ => rect.center().y - size * 0.5,
    }
}

/// Returns the bullet centres for a horizontal or vertical bullet strip.
///
/// * `Bottom` (default), `Top`, `Center` → horizontal strip on the bottom /
///   top / middle edge.
/// * `Left` / `Start` → vertical strip on the left edge (or right when
///   `right_to_left`).
/// * `Right` / `End` → vertical strip on the right edge (or left when
///   `right_to_left`).
///
/// Returns an empty vec when `count == 0`.
pub(crate) fn bullet_positions(
    rect: Rect,
    count: usize,
    bullet_size: f32,
    spacing: f32,
    position: Position,
    right_to_left: bool,
) -> Vec<Pos2> {
    if count == 0 {
        return Vec::new();
    }
    let step = bullet_size + spacing;
    let total = count as f32 * bullet_size + (count - 1) as f32 * spacing;

    let horizontal = |y: f32, align_end: bool| -> Vec<Pos2> {
        let start_x = if align_end {
            rect.right() - EDGE_PADDING - total
        } else {
            rect.center().x - total * 0.5
        };
        (0..count)
            .map(|i| egui::pos2(start_x + i as f32 * step + bullet_size * 0.5, y))
            .collect()
    };

    let vertical = |x: f32| -> Vec<Pos2> {
        let start_y = rect.center().y - total * 0.5;
        (0..count)
            .map(|i| egui::pos2(x, start_y + i as f32 * step + bullet_size * 0.5))
            .collect()
    };

    match position {
        Position::Top => horizontal(rect.top() + EDGE_PADDING + bullet_size * 0.5, false),
        Position::Center => horizontal(rect.center().y, false),
        Position::Left => vertical(if right_to_left {
            rect.right() - EDGE_PADDING - bullet_size * 0.5
        } else {
            rect.left() + EDGE_PADDING + bullet_size * 0.5
        }),
        Position::Right => vertical(if right_to_left {
            rect.left() + EDGE_PADDING + bullet_size * 0.5
        } else {
            rect.right() - EDGE_PADDING - bullet_size * 0.5
        }),
        Position::Start => vertical(if right_to_left {
            rect.right() - EDGE_PADDING - bullet_size * 0.5
        } else {
            rect.left() + EDGE_PADDING + bullet_size * 0.5
        }),
        Position::End => vertical(if right_to_left {
            rect.left() + EDGE_PADDING + bullet_size * 0.5
        } else {
            rect.right() - EDGE_PADDING - bullet_size * 0.5
        }),
        // Bottom (default)
        Position::Bottom => horizontal(
            rect.bottom() - EDGE_PADDING - bullet_size * 0.5,
            false,
        ),
    }
}

/// Which way a swipe should navigate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SwipeDir {
    Next,
    Prev,
}

/// Decides whether `(dx, dy)` constitutes a horizontal swipe and in which
/// direction. Returns `None` when the horizontal delta is below the threshold
/// or dominated by vertical movement.
#[inline]
pub(crate) fn swipe_direction(dx: f32, dy: f32) -> Option<SwipeDir> {
    if dx.abs() > SWIPE_THRESHOLD && dx.abs() > dy.abs() {
        Some(if dx < 0.0 { SwipeDir::Next } else { SwipeDir::Prev })
    } else {
        None
    }
}

/// Next index with wrap-around, ignoring direction. `count <= 1` returns
/// `current` unchanged.
#[inline]
pub(crate) fn next_index(current: usize, count: usize) -> usize {
    if count > 1 {
        (current + 1) % count
    } else {
        current
    }
}

/// Previous index with wrap-around. `count <= 1` returns `current` unchanged.
#[inline]
pub(crate) fn prev_index(current: usize, count: usize) -> usize {
    if count > 1 {
        (current + count - 1) % count
    } else {
        current
    }
}

/// Maps a `selected_index` into `0..count`, returning `0` for an empty list.
#[inline]
pub(crate) fn clamp_index(current: usize, count: usize) -> usize {
    if count == 0 { 0 } else { current.min(count - 1) }
}

/// Builds the list of raw indices for items whose `visible` flag is `true`.
#[inline]
pub(crate) fn visible_indices(flags: &[bool]) -> Vec<usize> {
    flags
        .iter()
        .enumerate()
        .filter(|(_, &v)| v)
        .map(|(i, _)| i)
        .collect()
}

/// Formats a duration as seconds with one decimal, used for diagnostics.
#[inline]
pub(crate) fn secs_f32(d: Duration) -> f32 {
    d.as_secs_f32()
}

// =============================================================================
// MudCarousel
// =============================================================================

/// A carousel that shows one item at a time.
#[must_use = "MudCarousel does nothing until you call `.show(ui, |c| ...)`"]
pub struct MudCarousel {
    /// Whether the carousel advances by itself (requires a non-zero
    /// [`auto_cycle_time`](Self::auto_cycle_time)).
    pub auto_cycle: bool,
    /// Time between automatic advances. `Duration::ZERO` disables auto-cycle.
    pub auto_cycle_time: Duration,
    /// Whether a horizontal drag / swipe changes the item.
    pub enable_swipe_gesture: bool,
    /// The index of the shown item (position in the *visible* list).
    pub selected_index: usize,
    /// Whether the previous / next arrows are shown.
    pub show_arrows: bool,
    /// Whether the bullets are shown.
    pub show_bullets: bool,
    /// Vertical placement of the arrows (`Top`, `Center`, `Bottom`).
    pub arrows_position: Position,
    /// Placement of the bullet strip (`Bottom`, `Top`, `Left`, `Right`,
    /// `Center`, `Start`, `End`).
    pub bullets_position: Position,
    /// Optional override for the bullet colour.
    pub bullets_color: Option<Color>,
    /// Utility classes applied to the bullet strip.
    pub bullets_class: Option<String>,
    /// Utility classes applied to the arrow buttons.
    pub navigation_buttons_class: Option<String>,
    /// Custom bullet renderer; receives `is_selected`.
    pub bullet_template: Option<Box<dyn FnMut(&mut Ui, bool)>>,
    /// Custom "next" button renderer.
    pub next_button_template: Option<Box<dyn FnMut(&mut Ui)>>,
    /// Custom "previous" button renderer.
    pub previous_button_template: Option<Box<dyn FnMut(&mut Ui)>>,
    /// Icon shown for the selected bullet.
    pub checked_icon: String,
    /// Icon shown for non-selected bullets.
    pub unchecked_icon: String,
    /// Icon of the previous arrow.
    pub previous_icon: String,
    /// Icon of the next arrow.
    pub next_icon: String,
    /// Optional explicit height. Defaults to [`DEFAULT_HEIGHT`].
    pub height: Option<f32>,
    /// Right-to-left layout (swaps left/right anchors for arrows and
    /// `Start`/`End` bullets).
    pub right_to_left: bool,
    /// Fires whenever the selection changes (auto-cycle, swipe, arrows,
    /// bullets). The callback runs **after** the change; the new index is
    /// also returned from [`show`](Self::show).
    pub on_selected_index_change: Option<Box<dyn FnMut(usize)>>,

    // CSS utilities (wrapper frame).
    pub base: ComponentBase,
    pub flavor: StyleFlavor,
    pub bs_style: BsStyle,
    pub tw_style: TwStyle,
    pub tw_state: TwState,
}

impl Default for MudCarousel {
    fn default() -> Self {
        Self::new()
    }
}

impl MudCarousel {
    /// Creates a carousel with MudBlazor defaults.
    pub fn new() -> Self {
        Self {
            auto_cycle: false,
            auto_cycle_time: Duration::ZERO,
            enable_swipe_gesture: true,
            selected_index: 0,
            show_arrows: true,
            show_bullets: true,
            arrows_position: Position::Center,
            bullets_position: Position::Bottom,
            bullets_color: None,
            bullets_class: None,
            navigation_buttons_class: None,
            bullet_template: None,
            next_button_template: None,
            previous_button_template: None,
            checked_icon: filled::RADIO_BUTTON_CHECKED.to_string(),
            unchecked_icon: filled::RADIO_BUTTON_UNCHECKED.to_string(),
            previous_icon: filled::NAVIGATE_BEFORE.to_string(),
            next_icon: filled::NAVIGATE_NEXT.to_string(),
            height: None,
            right_to_left: false,
            on_selected_index_change: None,
            base: ComponentBase::new(),
            flavor: StyleFlavor::default(),
            bs_style: BsStyle::default(),
            tw_style: TwStyle::default(),
            tw_state: TwState::default(),
        }
    }

    // ── Builder methods ──────────────────────────────────────────────────────

    /// Enables or disables automatic cycling. Mirrors `AutoCycle`.
    #[inline]
    pub fn auto_cycle(mut self, b: bool) -> Self {
        self.auto_cycle = b;
        self
    }

    /// Sets the auto-cycle interval. `Duration::ZERO` disables cycling even
    /// when [`auto_cycle(true)`](Self::auto_cycle) is set. Mirrors
    /// `AutoCycleTime` (default `TimeSpan.Zero`).
    #[inline]
    pub fn auto_cycle_time(mut self, t: Duration) -> Self {
        self.auto_cycle_time = t;
        self
    }

    /// Enables or disables swipe gesture navigation. Mirrors
    /// `EnableSwipeGesture` (default `true`).
    #[inline]
    pub fn enable_swipe_gesture(mut self, b: bool) -> Self {
        self.enable_swipe_gesture = b;
        self
    }

    /// Sets the selected index (position in the visible list). Mirrors
    /// `SelectedIndex`.
    #[inline]
    pub fn selected_index(mut self, i: usize) -> Self {
        self.selected_index = i;
        self
    }

    /// Shows or hides the navigation arrows. Mirrors `ShowArrows`.
    #[inline]
    pub fn show_arrows(mut self, b: bool) -> Self {
        self.show_arrows = b;
        self
    }

    /// Shows or hides the bullet strip. Mirrors `ShowBullets`.
    #[inline]
    pub fn show_bullets(mut self, b: bool) -> Self {
        self.show_bullets = b;
        self
    }

    /// Sets the vertical placement of the arrows. Mirrors `ArrowsPosition`
    /// (default `Center`).
    #[inline]
    pub fn arrows_position(mut self, p: Position) -> Self {
        self.arrows_position = p;
        self
    }

    /// Sets the placement of the bullet strip. Mirrors `BulletsPosition`
    /// (default `Bottom`).
    #[inline]
    pub fn bullets_position(mut self, p: Position) -> Self {
        self.bullets_position = p;
        self
    }

    /// Overrides the bullet colour. Mirrors `BulletsColor`.
    #[inline]
    pub fn bullets_color(mut self, c: Color) -> Self {
        self.bullets_color = Some(c);
        self
    }

    /// Utility classes for the bullet strip. Mirrors `BulletsClass`.
    #[inline]
    pub fn bullets_class(mut self, s: impl Into<String>) -> Self {
        self.bullets_class = Some(s.into());
        self
    }

    /// Utility classes for the arrow buttons. Mirrors
    /// `NavigationButtonsClass`.
    #[inline]
    pub fn navigation_buttons_class(mut self, s: impl Into<String>) -> Self {
        self.navigation_buttons_class = Some(s.into());
        self
    }

    /// Custom bullet renderer. Mirrors `BulletTemplate` (receives
    /// `is_selected`).
    #[inline]
    pub fn bullet_template(mut self, f: impl FnMut(&mut Ui, bool) + 'static) -> Self {
        self.bullet_template = Some(Box::new(f));
        self
    }

    /// Custom "next" button renderer. Mirrors `NextButtonTemplate`.
    #[inline]
    pub fn next_button_template(mut self, f: impl FnMut(&mut Ui) + 'static) -> Self {
        self.next_button_template = Some(Box::new(f));
        self
    }

    /// Custom "previous" button renderer. Mirrors `PreviousButtonTemplate`.
    #[inline]
    pub fn previous_button_template(mut self, f: impl FnMut(&mut Ui) + 'static) -> Self {
        self.previous_button_template = Some(Box::new(f));
        self
    }

    /// Sets the selected-bullet icon. Mirrors `CheckedIcon`.
    #[inline]
    pub fn checked_icon(mut self, s: impl Into<String>) -> Self {
        self.checked_icon = s.into();
        self
    }

    /// Sets the non-selected bullet icon. Mirrors `UncheckedIcon`.
    #[inline]
    pub fn unchecked_icon(mut self, s: impl Into<String>) -> Self {
        self.unchecked_icon = s.into();
        self
    }

    /// Sets the previous-arrow icon. Mirrors `PreviousIcon`.
    #[inline]
    pub fn previous_icon(mut self, s: impl Into<String>) -> Self {
        self.previous_icon = s.into();
        self
    }

    /// Sets the next-arrow icon. Mirrors `NextIcon`.
    #[inline]
    pub fn next_icon(mut self, s: impl Into<String>) -> Self {
        self.next_icon = s.into();
        self
    }

    /// Sets an explicit height in logical pixels.
    #[inline]
    pub fn height(mut self, h: f32) -> Self {
        self.height = Some(h);
        self
    }

    /// Enables right-to-left layout. Mirrors `RightToLeft`.
    #[inline]
    pub fn right_to_left(mut self, b: bool) -> Self {
        self.right_to_left = b;
        self
    }

    /// Registers a selection-change callback. Extension (MudBlazor uses
    /// `SelectedIndexChanged` + two-way binding).
    #[inline]
    pub fn on_selected_index_change(mut self, f: impl FnMut(usize) + 'static) -> Self {
        self.on_selected_index_change = Some(Box::new(f));
        self
    }

    /// Selects the utility-class resolver.
    #[inline]
    pub fn flavor(mut self, f: StyleFlavor) -> Self {
        self.flavor = f;
        self
    }

    /// Adds utility classes to the carousel wrapper. Mirrors `Class`.
    #[inline]
    pub fn classes<S: AsRef<str>>(mut self, classes: impl IntoIterator<Item = S>) -> Self {
        self.base = self.base.classes(classes);
        self
    }

    // ── Navigation ───────────────────────────────────────────────────────────

    /// Moves to `index`, clamped into `0..count`.
    pub fn move_to(&mut self, index: usize, count: usize) {
        if count > 0 {
            self.selected_index = index.min(count - 1);
        }
    }

    /// Advances to the next item with wrap-around.
    pub fn next(&mut self, count: usize) {
        self.selected_index = next_index(self.selected_index, count);
    }

    /// Moves to the previous item with wrap-around.
    pub fn previous(&mut self, count: usize) {
        self.selected_index = prev_index(self.selected_index, count);
    }

    // ── Rendering ────────────────────────────────────────────────────────────

    /// Renders the carousel. Returns `(response, selected_index, user_result)`.
    ///
    /// The caller is expected to persist the returned `selected_index` into
    /// its own state (or use [`on_selected_index_change`](Self::on_selected_index_change)).
    pub fn show<R>(
        mut self,
        ui: &mut Ui,
        add_items: impl FnOnce(&mut CarouselCtx<'_>) -> R,
    ) -> (Response, usize, R) {
        let theme = MudTheme::fetch(ui.ctx()).unwrap_or_default();
        let palette = theme.current_palette();

        // 1. Collect items.
        let mut ctx = CarouselCtx { items: Vec::new() };
        let user_result = add_items(&mut ctx);
        let mut items = ctx.items;

        // 2. Resolve the wrapper frame.
        let lookup = |key: &str| palette.resolve(key).map(Color32::from);
        self.tw_state.dark = ui.visuals().dark_mode;
        let frame = match self.flavor {
            StyleFlavor::Bootstrap => self.bs_style.build_frame(&self.base, &lookup),
            StyleFlavor::Tailwind => self
                .tw_style
                .build_frame(&self.base, self.tw_state, REM_SIZE_PT, &lookup),
        };

        let carousel_h = self.height.unwrap_or(DEFAULT_HEIGHT);

        let mut selected = self.selected_index;
        let mut changed = false;
        let mut response: Option<Response> = None;

        let outer = frame.show(ui, |ui| {
            // 3. Filter visible items.
            let visible_flags: Vec<bool> = items.iter().map(|it| it.meta.visible).collect();
            let visible = visible_indices(&visible_flags);
            let visible_count = visible.len();

            let total_size = vec2(ui.available_width(), carousel_h);

            // 4. Empty case.
            if visible_count == 0 {
                let (rect, resp) = ui.allocate_exact_size(total_size, Sense::hover());
                ui.painter()
                    .rect_filled(rect, 8.0, ui.visuals().extreme_bg_color);
                response = Some(resp);
                return;
            }

            // 5. Clamp selected index into the visible list.
            selected = clamp_index(selected, visible_count);
            let current_raw = visible[selected];

            // 6. Allocate the interaction region.
            let (rect, resp) = ui.allocate_exact_size(total_size, Sense::click_and_drag());
            let response_id = resp.id;

            // 7. Paint background + content.
            {
                let current_item = &mut items[current_raw];
                if let Some(bg) = current_item.resolve_background(palette) {
                    ui.painter().rect_filled(rect, 8.0, bg);
                } else {
                    ui.painter()
                        .rect_filled(rect, 8.0, ui.visuals().faint_bg_color);
                }

                let mut content_ui = ui.new_child(
                    egui::UiBuilder::new()
                        .max_rect(rect.shrink(16.0))
                        .layout(egui::Layout::centered_and_justified(
                            egui::Direction::TopDown,
                        )),
                );
                current_item.render_content(&mut content_ui);
            }

            // 8. Auto-cycle (memory-backed, survives the builder pattern).
            let auto_advanced = maybe_auto_advance(
                &mut self,
                ui,
                response_id,
                visible_count,
            );
            if auto_advanced {
                selected = clamp_index(selected, visible_count);
                changed = true;
            }

            // 9. Swipe handling (memory-backed, total-delta correct).
            if self.enable_swipe_gesture && visible_count > 1 {
                if let Some(dir) =
                    handle_swipe(ui, &resp, response_id)
                {
                    selected = match dir {
                        SwipeDir::Next => next_index(selected, visible_count),
                        SwipeDir::Prev => prev_index(selected, visible_count),
                    };
                    changed = true;
                }
            }

            // 10. Arrows.
            if self.show_arrows && visible_count > 1 {
                let arrow_y = arrow_y(rect, ARROW_SIZE, self.arrows_position);
                let arrow_color =
                    Color32::from_white_alpha((OVERLAY_OPACITY * 255.0) as u8);

                let (left_x, right_x) = if self.right_to_left {
                    (rect.right() - EDGE_PADDING - ARROW_SIZE, rect.left() + EDGE_PADDING)
                } else {
                    (rect.left() + EDGE_PADDING, rect.right() - EDGE_PADDING - ARROW_SIZE)
                };

                // Previous.
                let prev_rect = Rect::from_min_size(
                    egui::pos2(left_x, arrow_y),
                    vec2(ARROW_SIZE, ARROW_SIZE),
                );
                let prev_id = response_id.with("carousel_prev");
                let prev_resp = ui.interact(prev_rect, prev_id, Sense::click());
                if prev_resp.hovered() {
                    ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
                    ui.painter().rect_filled(
                        prev_rect,
                        20.0,
                        Color32::from_black_alpha(60),
                    );
                }
                if let Some(t) = self.previous_button_template.as_mut() {
                    let mut child = ui.new_child(
                        egui::UiBuilder::new().max_rect(prev_rect),
                    );
                    t(&mut child);
                } else {
                    ui.put(
                        prev_rect,
                        MudIcon::new(self.previous_icon.clone())
                            .size(2)
                            .custom_color(arrow_color),
                    );
                }
                if prev_resp.clicked() {
                    selected = prev_index(selected, visible_count);
                    changed = true;
                }

                // Next.
                let next_rect = Rect::from_min_size(
                    egui::pos2(right_x, arrow_y),
                    vec2(ARROW_SIZE, ARROW_SIZE),
                );
                let next_id = response_id.with("carousel_next");
                let next_resp = ui.interact(next_rect, next_id, Sense::click());
                if next_resp.hovered() {
                    ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
                    ui.painter().rect_filled(
                        next_rect,
                        20.0,
                        Color32::from_black_alpha(60),
                    );
                }
                if let Some(t) = self.next_button_template.as_mut() {
                    let mut child = ui.new_child(
                        egui::UiBuilder::new().max_rect(next_rect),
                    );
                    t(&mut child);
                } else {
                    ui.put(
                        next_rect,
                        MudIcon::new(self.next_icon.clone())
                            .size(2)
                            .custom_color(arrow_color),
                    );
                }
                if next_resp.clicked() {
                    selected = next_index(selected, visible_count);
                    changed = true;
                }
            }

            // 11. Bullets.
            if self.show_bullets && visible_count > 1 {
                let positions = bullet_positions(
                    rect,
                    visible_count,
                    BULLET_SIZE,
                    BULLET_SPACING,
                    self.bullets_position,
                    self.right_to_left,
                );
                let bullet_color = match self.bullets_color {
                    Some(c) => resolve_color(
                        c,
                        &theme,
                        Color32::from_white_alpha(
                            (OVERLAY_OPACITY * 255.0) as u8,
                        ),
                    ),
                    None => Color32::from_white_alpha(
                        (OVERLAY_OPACITY * 255.0) as u8,
                    ),
                };

                for (i, center) in positions.iter().enumerate() {
                    let bullet_rect = Rect::from_center_size(
                        *center,
                        vec2(BULLET_SIZE, BULLET_SIZE),
                    );
                    let bullet_id = response_id.with(("carousel_bullet", i));
                    let bullet_resp =
                        ui.interact(bullet_rect, bullet_id, Sense::click());

                    let is_sel = i == selected;
                    if let Some(t) = self.bullet_template.as_mut() {
                        let mut child = ui.new_child(
                            egui::UiBuilder::new().max_rect(bullet_rect),
                        );
                        t(&mut child, is_sel);
                    } else {
                        let icon = if is_sel {
                            &self.checked_icon
                        } else {
                            &self.unchecked_icon
                        };
                        ui.put(
                            bullet_rect,
                            MudIcon::new(icon.clone())
                                .size_px(BULLET_SIZE)
                                .custom_color(bullet_color),
                        );
                    }

                    if bullet_resp.clicked() && !is_sel {
                        selected = i;
                        changed = true;
                    }
                    if bullet_resp.hovered() {
                        ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
                    }
                }
            }

            response = Some(resp);
        });

        // 12. Fire selection callback.
        if changed {
            if let Some(cb) = self.on_selected_index_change.as_mut() {
                cb(selected);
            }
        }

        // 13. Update Tailwind state for the next frame.
        self.tw_state.hovered = outer.response.hovered();
        self.tw_state.active = outer.response.is_pointer_button_down_on();

        let resp = response
            .unwrap_or_else(|| ui.allocate_response(Vec2::ZERO, Sense::hover()));
        (resp, selected, user_result)
    }
}

// =============================================================================
// Auto-cycle (memory-backed)
// =============================================================================

/// Advances the carousel when the auto-cycle deadline has passed.
///
/// Timing is stored in [`egui::Memory`] under `response_id.with("auto_cycle_last")`
/// so rebuilding the `MudCarousel` every frame does not reset the clock.
/// Returns `true` when the carousel advanced this frame.
fn maybe_auto_advance(
    carousel: &mut MudCarousel,
    ui: &mut Ui,
    response_id: egui::Id,
    count: usize,
) -> bool {
    if !carousel.auto_cycle || carousel.auto_cycle_time.is_zero() || count <= 1 {
        return false;
    }

    let last_id = response_id.with("auto_cycle_last");
    let now = ui.input(|i| i.time);
    let interval = carousel.auto_cycle_time.as_secs_f64();

    let last: Option<f64> = ui.ctx().memory(|m| m.data.get_temp(last_id));
    let Some(last) = last else {
        // First frame: start the clock, request a repaint at the deadline.
        ui.ctx().memory_mut(|m| m.data.insert_temp(last_id, now));
        ui.ctx()
            .request_repaint_after(Duration::from_secs_f64(interval));
        return false;
    };

    let elapsed = now - last;
    if elapsed >= interval {
        carousel.next(count);
        ui.ctx().memory_mut(|m| m.data.insert_temp(last_id, now));
        ui.ctx()
            .request_repaint_after(Duration::from_secs_f64(interval));
        true
    } else {
        let wait = (interval - elapsed).max(0.0);
        ui.ctx()
            .request_repaint_after(Duration::from_secs_f64(wait));
        false
    }
}

// =============================================================================
// Swipe (memory-backed)
// =============================================================================

/// Tracks drag start / last position and returns a swipe direction on release.
fn handle_swipe(
    ui: &mut Ui,
    response: &Response,
    response_id: egui::Id,
) -> Option<SwipeDir> {
    let start_id = response_id.with("swipe_start");
    let last_id = response_id.with("swipe_last");

    if response.drag_started() {
        if let Some(pos) = response.interact_pointer_pos() {
            ui.ctx().memory_mut(|m| {
                m.data.insert_temp(start_id, pos);
                m.data.insert_temp(last_id, pos);
            });
        }
    }

    if response.dragged() {
        if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
            ui.ctx().memory_mut(|m| m.data.insert_temp(last_id, pos));
        }
    }

    if response.drag_stopped() {
        let start: Option<Pos2> = ui.ctx().memory(|m| m.data.get_temp(start_id));
        let end: Option<Pos2> = ui.ctx().memory(|m| m.data.get_temp(last_id));
        let dir = match (start, end) {
            (Some(s), Some(e)) => swipe_direction(e.x - s.x, e.y - s.y),
            _ => None,
        };
        ui.ctx().memory_mut(|m| {
            m.data.remove_temp::<Pos2>(start_id);
            m.data.remove_temp::<Pos2>(last_id);
        });
        return dir;
    }

    None
}

// =============================================================================
// Builder context
// =============================================================================

/// Builder context passed to [`MudCarousel::show`]'s closure.
pub struct CarouselCtx<'a> {
    items: Vec<CarouselItemEntry<'a>>,
}

/// Internal representation of a slide registered via the builder.
struct CarouselItemEntry<'a> {
    meta: MudCarouselItem,
    content: Box<dyn FnMut(&mut Ui) + 'a>,
}

impl<'a> CarouselItemEntry<'a> {
    fn resolve_background(
        &self,
        palette: &mud_egui_theming::models::palettes::palette::Palette,
    ) -> Option<Color32> {
        self.meta.resolve_background(palette)
    }

    fn render_content(&mut self, ui: &mut Ui) {
        (self.content)(ui);
    }
}

impl<'a> CarouselCtx<'a> {
    /// Registers a slide with the given metadata and content.
    pub fn item<F>(&mut self, meta: MudCarouselItem, content: F)
    where
        F: FnMut(&mut Ui) + 'a,
    {
        self.items.push(CarouselItemEntry {
            meta,
            content: Box::new(content),
        });
    }

    /// Registers a slide with default metadata.
    pub fn slide<F>(&mut self, content: F)
    where
        F: FnMut(&mut Ui) + 'a,
    {
        self.item(MudCarouselItem::new(), content);
    }
}

// =============================================================================
// Transition marker (currently ignored at paint time)
// =============================================================================
//
// egui is immediate-mode, so there is no `TransitioningContentControl`.
// `Transition` values are stored on [`MudCarouselItem`] for future animation
// support; they are not consumed by paint.
#[allow(dead_code)]
fn _transition_marker(_t: Transition) {}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{pos2, vec2, Rect};

    // --- swipe direction ---------------------------------------------------

    #[test]
    fn swipe_below_threshold_is_none() {
        assert_eq!(swipe_direction(10.0, 0.0), None);
        assert_eq!(swipe_direction(-10.0, 0.0), None);
    }

    #[test]
    fn swipe_above_threshold_is_directional() {
        assert_eq!(swipe_direction(-100.0, 5.0), Some(SwipeDir::Next));
        assert_eq!(swipe_direction(100.0, 5.0), Some(SwipeDir::Prev));
    }

    #[test]
    fn swipe_dominated_by_vertical_is_none() {
        // dx = 60 (> threshold), but dy = 80 dominates.
        assert_eq!(swipe_direction(60.0, 80.0), None);
        assert_eq!(swipe_direction(-60.0, -80.0), None);
    }

    // --- index math --------------------------------------------------------

    #[test]
    fn next_wraps_and_handles_single() {
        assert_eq!(next_index(0, 3), 1);
        assert_eq!(next_index(2, 3), 0);
        assert_eq!(next_index(0, 1), 0);
        assert_eq!(next_index(5, 0), 5);
    }

    #[test]
    fn prev_wraps_and_handles_single() {
        assert_eq!(prev_index(0, 3), 2);
        assert_eq!(prev_index(2, 3), 1);
        assert_eq!(prev_index(0, 1), 0);
        assert_eq!(prev_index(5, 0), 5);
    }

    #[test]
    fn clamp_index_handles_empty_and_overflow() {
        assert_eq!(clamp_index(0, 0), 0);
        assert_eq!(clamp_index(5, 3), 2);
        assert_eq!(clamp_index(1, 3), 1);
    }

    // --- visible filtering -------------------------------------------------

    #[test]
    fn visible_indices_skips_hidden_items() {
        assert_eq!(visible_indices(&[true, false, true]), vec![0, 2]);
        assert_eq!(visible_indices(&[false, false]), Vec::<usize>::new());
        assert_eq!(visible_indices(&[true, true, true]), vec![0, 1, 2]);
        assert_eq!(visible_indices(&[]), Vec::<usize>::new());
    }

    // --- arrow placement ---------------------------------------------------

    fn rect() -> Rect {
        Rect::from_min_size(pos2(0.0, 0.0), vec2(400.0, 300.0))
    }

    #[test]
    fn arrow_center_is_vertical_midpoint() {
        let y = arrow_y(rect(), ARROW_SIZE, Position::Center);
        assert_eq!(y, 300.0 * 0.5 - ARROW_SIZE * 0.5);
    }

    #[test]
    fn arrow_top_is_padded_from_top_edge() {
        let y = arrow_y(rect(), ARROW_SIZE, Position::Top);
        assert_eq!(y, EDGE_PADDING);
    }

    #[test]
    fn arrow_bottom_is_padded_from_bottom_edge() {
        let y = arrow_y(rect(), ARROW_SIZE, Position::Bottom);
        assert_eq!(y, 300.0 - ARROW_SIZE - EDGE_PADDING);
    }

    #[test]
    fn arrow_unknown_positions_fall_back_to_center() {
        let center = arrow_y(rect(), ARROW_SIZE, Position::Center);
        for p in [Position::Left, Position::Right, Position::Start, Position::End] {
            assert_eq!(arrow_y(rect(), ARROW_SIZE, p), center);
        }
    }

    // --- bullet placement --------------------------------------------------

    #[test]
    fn bullet_bottom_is_horizontal_and_centered() {
        let pos = bullet_positions(rect(), 3, 10.0, 4.0, Position::Bottom, false);
        assert_eq!(pos.len(), 3);
        // First bullet centre = 200 - total/2 + 5, where total = 3*10 + 2*4 = 38.
        assert_eq!(pos[0].x, 200.0 - 19.0 + 5.0);
        assert_eq!(pos[1].x, 200.0 - 19.0 + 14.0 + 5.0);
        assert_eq!(pos[2].x, 200.0 - 19.0 + 28.0 + 5.0);
        // All share the same y, near the bottom.
        assert_eq!(pos[0].y, pos[1].y);
        assert_eq!(pos[0].y, 300.0 - EDGE_PADDING - 5.0);
    }

    #[test]
    fn bullet_top_is_horizontal_at_top() {
        let pos = bullet_positions(rect(), 2, 10.0, 4.0, Position::Top, false);
        assert_eq!(pos.len(), 2);
        assert_eq!(pos[0].y, EDGE_PADDING + 5.0);
        assert_eq!(pos[0].y, pos[1].y);
    }

    #[test]
    fn bullet_left_is_vertical_on_left_edge() {
        let pos = bullet_positions(rect(), 3, 10.0, 4.0, Position::Left, false);
        assert_eq!(pos.len(), 3);
        assert_eq!(pos[0].x, EDGE_PADDING + 5.0);
        assert_eq!(pos[0].x, pos[1].x);
        assert_eq!(pos[1].y, 150.0 - 19.0 + 14.0 + 5.0);
    }

    #[test]
    fn bullet_right_is_vertical_on_right_edge() {
        let pos = bullet_positions(rect(), 2, 10.0, 4.0, Position::Right, false);
        assert_eq!(pos[0].x, 400.0 - EDGE_PADDING - 5.0);
    }

    #[test]
    fn bullet_rtl_swaps_start_and_end() {
        let ltr = bullet_positions(rect(), 2, 10.0, 4.0, Position::Start, false);
        let rtl = bullet_positions(rect(), 2, 10.0, 4.0, Position::Start, true);
        assert_eq!(ltr[0].x, EDGE_PADDING + 5.0);
        assert_eq!(rtl[0].x, 400.0 - EDGE_PADDING - 5.0);
    }

    #[test]
    fn bullet_zero_count_is_empty() {
        assert!(bullet_positions(rect(), 0, 10.0, 4.0, Position::Bottom, false).is_empty());
    }

    // --- defaults ----------------------------------------------------------

    #[test]
    fn defaults_match_mudblazor() {
        let c = MudCarousel::new();
        assert!(!c.auto_cycle);
        assert_eq!(c.auto_cycle_time, Duration::ZERO);
        assert!(c.enable_swipe_gesture);
        assert_eq!(c.selected_index, 0);
        assert!(c.show_arrows);
        assert!(c.show_bullets);
        assert!(matches!(c.arrows_position, Position::Center));
        assert!(matches!(c.bullets_position, Position::Bottom));
        assert!(c.bullets_color.is_none());
        assert!(c.bullets_class.is_none());
        assert!(c.navigation_buttons_class.is_none());
        assert!(c.bullet_template.is_none());
        assert!(!c.right_to_left);
        assert!(c.on_selected_index_change.is_none());
        assert!(c.height.is_none());
    }

    #[test]
    fn builders_set_every_field() {
        let c = MudCarousel::new()
            .auto_cycle(true)
            .auto_cycle_time(Duration::from_secs(3))
            .enable_swipe_gesture(false)
            .selected_index(2)
            .show_arrows(false)
            .show_bullets(false)
            .arrows_position(Position::Top)
            .bullets_position(Position::Top)
            .bullets_color(Color::Error)
            .bullets_class("bg-error")
            .navigation_buttons_class("text-white")
            .bullet_template(|_, _| {})
            .next_button_template(|_| {})
            .previous_button_template(|_| {})
            .checked_icon("X")
            .unchecked_icon("Y")
            .previous_icon("P")
            .next_icon("N")
            .height(300.0)
            .right_to_left(true)
            .on_selected_index_change(|_| {})
            .classes(["a"]);
        assert!(c.auto_cycle);
        assert_eq!(c.auto_cycle_time, Duration::from_secs(3));
        assert!(!c.enable_swipe_gesture);
        assert_eq!(c.selected_index, 2);
        assert!(!c.show_arrows && !c.show_bullets);
        assert!(matches!(c.arrows_position, Position::Top));
        assert!(matches!(c.bullets_position, Position::Top));
        assert!(matches!(c.bullets_color, Some(Color::Error)));
        assert_eq!(c.bullets_class.as_deref(), Some("bg-error"));
        assert_eq!(c.navigation_buttons_class.as_deref(), Some("text-white"));
        assert!(c.bullet_template.is_some());
        assert!(c.next_button_template.is_some());
        assert!(c.previous_button_template.is_some());
        assert_eq!(c.checked_icon, "X");
        assert_eq!(c.unchecked_icon, "Y");
        assert_eq!(c.previous_icon, "P");
        assert_eq!(c.next_icon, "N");
        assert_eq!(c.height, Some(300.0));
        assert!(c.right_to_left);
        assert!(c.on_selected_index_change.is_some());
    }

    // --- navigation methods ------------------------------------------------

    #[test]
    fn move_to_clamps_into_range() {
        let mut c = MudCarousel::new();
        c.move_to(5, 3);
        assert_eq!(c.selected_index, 2);
        c.move_to(1, 3);
        assert_eq!(c.selected_index, 1);
        c.move_to(5, 0);
        assert_eq!(c.selected_index, 1); // unchanged for empty
    }

    #[test]
    fn next_and_previous_wrap() {
        let mut c = MudCarousel::new();
        c.next(3);
        assert_eq!(c.selected_index, 1);
        c.next(3);
        assert_eq!(c.selected_index, 2);
        c.next(3);
        assert_eq!(c.selected_index, 0);
        c.previous(3);
        assert_eq!(c.selected_index, 2);
    }

    // --- secs_f32 ----------------------------------------------------------

    #[test]
    fn secs_f32_reports_duration() {
        assert_eq!(secs_f32(Duration::from_secs(5)), 5.0);
        assert_eq!(secs_f32(Duration::from_millis(500)), 0.5);
        assert_eq!(secs_f32(Duration::ZERO), 0.0);
    }
}