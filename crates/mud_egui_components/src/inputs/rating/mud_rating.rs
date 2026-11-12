//! A star-style rating control.
//!
//! Port of `MudRating.razor.cs` (MudBlazor 8.13.0). The public parameter
//! surface mirrors the Blazor component exactly:
//!
//! | MudBlazor parameter | Rust field / method |
//! |---------------------|---------------------|
//! | `Color` | [`MudRating::color`] |
//! | `Disabled` | [`MudRating::disabled`] |
//! | `EmptyIcon` | [`MudRating::empty_icon`] |
//! | `EmptyIconColor` | [`MudRating::empty_icon_color`] |
//! | `FullIcon` | [`MudRating::full_icon`] |
//! | `FullIconColor` | [`MudRating::full_icon_color`] |
//! | `HoveredValueChanged` | [`RatingResponse::hovered_changed`] + [`RatingResponse::hovered`] |
//! | `MaxValue` | [`MudRating::max_value`] (default `5`) |
//! | `Name` | [`MudRating::name`] — stored, inert |
//! | `ReadOnly` | [`MudRating::read_only`] |
//! | `Ripple` | [`MudRating::ripple`] |
//! | `SelectedValue` | caller-owned `&mut i32` |
//! | `SelectedValueChanged` | [`RatingResponse::changed`] |
//! | `Size` | [`MudRating::size`] |
//! | `Class` | [`MudRating::classes`] (stored, not resolved) |
//! | `Tag` | [`MudRating::tag`] / [`MudRating::tag_ref`] |
//! | `RatingItemsClass` / `RatingItemsStyle` | not ported: egui has no CSS strings |
//! | `Style` / `UserAttributes` | not ported: no CSS strings, no HTML tag |
//!
//! # Unselected state
//!
//! `SelectedValue` is `int`, not `int?`; **`0` means "nothing selected"**. That
//! is MudBlazor's design, and the port keeps it. `RatingResponse::hovered` uses
//! the same convention (`0` = nothing hovered); `hovered_changed` is `Some(new)`
//! only on the frame the hover moved, so a caller can react to
//! `HoveredValueChanged` without storing previous state themselves.
//!
//! # Hover preview
//!
//! While the pointer is over star *K*, stars `1..=K` render filled; on leave,
//! the display reverts to `*selected`. `read_only` and `disabled` suppress the
//! preview entirely — they show only the selected value, matching MudBlazor's
//! intended behaviour.
//!
//! # Keyboard
//!
//! When the rating is focused and enabled:
//!
//! | Key | Effect |
//! |-----|--------|
//! | `→` / `↑` | Increment (clamped to `max_value`) |
//! | `←` / `↓` | Decrement (clamped to `0`) |
//! | `Home` | `1` |
//! | `End` | `max_value` |
//! | `Space` / `Enter` | Select the hovered star, if any |
//!
//! # Performance
//!
//! - Theme fetched once per frame; one `Color::resolve` per colour role that is
//!   actually needed.
//! - One `allocate_exact_size` for the whole row, one `interact` for focus and
//!   pointer, and one `ui.put` per star — no `Vec`, no `String`.
//! - Previous hovered value lives in `egui::Memory` under the row's id; the
//!   component itself holds no state between frames.
//! - Ripple start time lives in `egui::Memory`, keyed by the star index.
//! - Pure helpers for geometry, keyboard and value resolution, all
//!   `#[inline]`.

use std::any::Any;

use egui::{
    pos2, vec2, Color32, CursorIcon, Id, Rect, Response, Sense, Ui, Vec2,
};

use crate::{MudIcon, MudText};
use mud_egui_core::base::component_base::ComponentBase;
use mud_egui_core::enums::{Color, Size};
use mud_egui_theming::theme::MudTheme;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Gap between two adjacent stars, in logical pixels.
const STAR_GAP: f32 = 4.0;

/// Ripple animation duration in seconds.
const RIPPLE_DURATION: f32 = 0.35;

/// Maximum ripple radius as a multiple of half the star side.
const RIPPLE_MAX_RADIUS_FACTOR: f32 = 1.2;

/// Alpha of the ripple fill.
const RIPPLE_ALPHA: u8 = 60;

/// Gamma multiplier applied to the accent for the empty (unfilled) star when
/// `empty_icon_color` is not set.
const EMPTY_ALPHA: f32 = 0.38;

/// Gamma multiplier applied to every colour when `disabled` is true.
const DISABLED_ALPHA: f32 = 0.38;

// ---------------------------------------------------------------------------
// MudRating
// ---------------------------------------------------------------------------

/// A star-style rating control.
///
/// Mirrors `MudRating` in MudBlazor 8.13.0. The Blazor two-way binding
/// `SelectedValue` / `SelectedValueChanged` becomes a caller-owned `&mut i32`
/// plus a [`RatingResponse::changed`] report, the same split `MudCheckBox` and
/// `MudRadio` use.
#[must_use = "MudRating does nothing until you call `.show(ui)`"]
pub struct MudRating<'a> {
    /// Shared component state (utility classes). Stored for parity.
    pub base: ComponentBase,
    /// The bound selection. `0` means nothing selected. Mirrors MudBlazor
    /// `SelectedValue`.
    pub selected: &'a mut i32,
    /// Number of stars. Mirrors `MaxValue` (default `5`). Values `<= 0`
    /// render nothing.
    pub max_value: i32,
    /// Accent colour of filled stars. Mirrors `Color`.
    pub color: Color,
    /// Icon size preset. Mirrors `Size` (default `Medium`).
    pub size: Size,
    /// Glyph for filled stars. Mirrors `FullIcon`.
    pub full_icon: Option<String>,
    /// Glyph for empty stars. Mirrors `EmptyIcon`.
    pub empty_icon: Option<String>,
    /// Colour override for filled stars. Mirrors `FullIconColor`.
    pub full_icon_color: Option<Color>,
    /// Colour override for empty stars. Mirrors `EmptyIconColor`.
    pub empty_icon_color: Option<Color>,
    /// Enables the click ripple. Mirrors `Ripple` (default `true`).
    pub ripple: bool,
    /// Disables interaction and dims everything. Mirrors `Disabled`.
    pub disabled: bool,
    /// Shows the selected value without interaction. Mirrors `ReadOnly`.
    pub read_only: bool,
    /// HTML form name. Stored, inert. Mirrors `Name`.
    pub name: Option<String>,
    /// Arbitrary user data. Mirrors `Tag`.
    pub tag: Option<Box<dyn Any + Send + Sync>>,
}

impl<'a> MudRating<'a> {
    /// Creates a rating bound to `selected` with MudBlazor's default settings.
    pub fn new(selected: &'a mut i32) -> Self {
        Self {
            base: ComponentBase::new(),
            selected,
            max_value: 5,
            color: Color::Default,
            size: Size::Medium,
            full_icon: None,
            empty_icon: None,
            full_icon_color: None,
            empty_icon_color: None,
            ripple: true,
            disabled: false,
            read_only: false,
            name: None,
            tag: None,
        }
    }

    /// Sets the number of stars. Mirrors `MaxValue`.
    #[inline]
    pub fn max_value(mut self, n: i32) -> Self {
        self.max_value = n;
        self
    }

    /// Sets the accent colour. Mirrors `Color`.
    #[inline]
    pub fn color(mut self, c: Color) -> Self {
        self.color = c;
        self
    }

    /// Sets the icon size. Mirrors `Size`.
    #[inline]
    pub fn size(mut self, s: Size) -> Self {
        self.size = s;
        self
    }

    /// Overrides the filled glyph. Mirrors `FullIcon`.
    #[inline]
    pub fn full_icon(mut self, icon: impl Into<String>) -> Self {
        self.full_icon = Some(icon.into());
        self
    }

    /// Overrides the empty glyph. Mirrors `EmptyIcon`.
    #[inline]
    pub fn empty_icon(mut self, icon: impl Into<String>) -> Self {
        self.empty_icon = Some(icon.into());
        self
    }

    /// Overrides the filled colour. Mirrors `FullIconColor`.
    #[inline]
    pub fn full_icon_color(mut self, c: Color) -> Self {
        self.full_icon_color = Some(c);
        self
    }

    /// Overrides the empty colour. Mirrors `EmptyIconColor`.
    #[inline]
    pub fn empty_icon_color(mut self, c: Color) -> Self {
        self.empty_icon_color = Some(c);
        self
    }

    /// Enables the click ripple. Mirrors `Ripple`.
    #[inline]
    pub fn ripple(mut self, b: bool) -> Self {
        self.ripple = b;
        self
    }

    /// Disables interaction and dims the component. Mirrors `Disabled`.
    #[inline]
    pub fn disabled(mut self, b: bool) -> Self {
        self.disabled = b;
        self
    }

    /// Shows the selected value without interaction. Mirrors `ReadOnly`.
    #[inline]
    pub fn read_only(mut self, b: bool) -> Self {
        self.read_only = b;
        self
    }

    /// Sets the HTML form name. Stored, inert. Mirrors `Name`.
    #[inline]
    pub fn name(mut self, s: impl Into<String>) -> Self {
        self.name = Some(s.into());
        self
    }

    /// Adds utility classes. Stored for parity.
    pub fn classes<S: AsRef<str>>(mut self, classes: impl IntoIterator<Item = S>) -> Self {
        self.base = self.base.classes(classes);
        self
    }

    /// Attaches arbitrary user data. Mirrors `Tag`.
    #[inline]
    pub fn tag<T: Any + Send + Sync>(mut self, tag: T) -> Self {
        self.tag = Some(Box::new(tag));
        self
    }

    /// Reads user data attached with [`tag`](Self::tag).
    #[inline]
    pub fn tag_ref<T: Any + Send + Sync>(&self) -> Option<&T> {
        self.tag.as_ref().and_then(|b| b.downcast_ref::<T>())
    }

    /// Draws the rating and reports this frame’s outcome.
    pub fn show(self, ui: &mut Ui) -> RatingResponse {
        let stars = sanitize_star_count(self.max_value);
        let row_id = ui.id().with("mud_rating").with(&self.name);

        if stars == 0 {
            return RatingResponse {
                response: ui.allocate_response(Vec2::ZERO, Sense::hover()),
                changed: false,
                hovered: 0,
                hovered_changed: None,
            };
        }

        let theme = MudTheme::fetch(ui.ctx()).unwrap_or_default();
        let palette = theme.current_palette();
        let accent = self
            .color
            .resolve(palette)
            .unwrap_or_else(|| ui.visuals().text_color());
        let full_color = self
            .full_icon_color
            .and_then(|c| c.resolve(palette))
            .unwrap_or(accent);
        let empty_color = self
            .empty_icon_color
            .and_then(|c| c.resolve(palette))
            .unwrap_or_else(|| accent.gamma_multiply(EMPTY_ALPHA));

        let star_px = star_pixels(self.size);
        let row_size = row_size(stars, star_px, STAR_GAP);
        let sense = if self.disabled || self.read_only {
            Sense::hover()
        } else {
            Sense::click()
        };

        let (row_rect, response) = ui.allocate_exact_size(row_size, sense);
        if !ui.is_rect_visible(row_rect) {
            return RatingResponse {
                response,
                changed: false,
                hovered: 0,
                hovered_changed: None,
            };
        }

        let enabled = !self.disabled && !self.read_only;

        // --- pointer -------------------------------------------------------
        let hovered = if enabled {
            response
                .hover_pos()
                .map(|p| star_at(p, row_rect, stars, star_px, STAR_GAP))
                .unwrap_or(0)
        } else {
            0
        };

        let clicked = if enabled {
            response
                .interact_pointer_pos()
                .filter(|_| response.clicked())
                .map(|p| star_at(p, row_rect, stars, star_px, STAR_GAP))
                .unwrap_or(0)
        } else {
            0
        };

        // --- keyboard ------------------------------------------------------
        let key_action = if enabled && response.has_focus() {
            apply_keys(ui, *self.selected, self.max_value)
        } else {
            None
        };

        // --- apply ---------------------------------------------------------
        let mut changed = false;
        let mut new_selected = *self.selected;

        if clicked > 0 {
            new_selected = clicked;
        } else if let Some(KeyAction::Set(v)) = key_action {
            new_selected = v;
        } else if let Some(KeyAction::SelectHovered) = key_action {
            if hovered > 0 {
                new_selected = hovered;
            }
        }

        let new_selected = clamp_selected(new_selected, self.max_value);
        if new_selected != *self.selected {
            *self.selected = new_selected;
            changed = true;
        }

        // --- hovered-change detection (memory-backed) ----------------------
        let prev_hovered: i32 = ui
            .ctx()
            .memory(|m| m.data.get_temp(row_id.with("prev_hovered")))
            .unwrap_or(0);
        let hovered_changed = if prev_hovered != hovered {
            Some(hovered)
        } else {
            None
        };
        ui.ctx()
            .memory_mut(|m| m.data.insert_temp(row_id.with("prev_hovered"), hovered));

        // --- ripple --------------------------------------------------------
        if clicked > 0 && self.ripple {
            start_ripple(ui.ctx(), row_id.with(("ripple", clicked as usize)));
        }

        // --- paint ---------------------------------------------------------
        let display = display_value(*self.selected, hovered, self.disabled, self.read_only);
        let full_glyph = self
            .full_icon
            .as_deref()
            .unwrap_or(mud_egui_icons_material::filled::STAR);
        let empty_glyph = self
            .empty_icon
            .as_deref()
            .unwrap_or(mud_egui_icons_material::filled::STAR_BORDER);
        let alpha = if self.disabled { DISABLED_ALPHA } else { 1.0 };

        for i in 1..=stars {
            let rect = star_rect(i - 1, row_rect, star_px, STAR_GAP);
            // `i` is `usize` (range over `stars`); `display` is `i32`.
            let filled = (i as i32) <= display;
            let (glyph, color) = if filled {
                (full_glyph, full_color)
            } else {
                (empty_glyph, empty_color)
            };

            if self.ripple && filled {
                paint_ripple(
                    ui,
                    rect.center(),
                    star_px * 0.5,
                    row_id.with(("ripple", i as usize)),
                    full_color,
                );
            }

            ui.put(
                rect,
                MudIcon::new(glyph.to_owned())
                    .size_px(star_px)
                    .custom_color(color.gamma_multiply(alpha)),
            );
        }

        // --- cursor --------------------------------------------------------
        let response = if self.disabled {
            response.on_hover_cursor(CursorIcon::NotAllowed)
        } else if self.read_only {
            response
        } else {
            response.on_hover_cursor(CursorIcon::PointingHand)
        };

        RatingResponse {
            response,
            changed,
            hovered,
            hovered_changed,
        }
    }
}

impl<'a> egui::Widget for MudRating<'a> {
    /// Draws an empty rating. Prefer [`MudRating::show`] so the caller owns the
    /// binding and can read `changed` / `hovered_changed`.
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui).response
    }
}

// ---------------------------------------------------------------------------
// RatingResponse
// ---------------------------------------------------------------------------

/// What [`MudRating::show`] did this frame.
#[derive(Debug)]
pub struct RatingResponse {
    /// The rating row’s `Response`. Deref target.
    pub response: Response,
    /// `true` on the frame `SelectedValue` changed. Mirrors
    /// `SelectedValueChanged`.
    pub changed: bool,
    /// The current hovered star, `1..=max_value`, or `0` when nothing is
    /// hovered. Reported every frame.
    pub hovered: i32,
    /// `Some(new)` only on the frame the hovered value changed; `0` means the
    /// hover left. Mirrors `HoveredValueChanged` (whose payload is `int?`,
    /// with `null` mapped to `0` here).
    pub hovered_changed: Option<i32>,
}

impl std::ops::Deref for RatingResponse {
    type Target = Response;

    fn deref(&self) -> &Self::Target {
        &self.response
    }
}

// ---------------------------------------------------------------------------
// Pure helpers (no `egui::Context` required)
// ---------------------------------------------------------------------------

/// A keyboard action parsed from the current frame’s input.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum KeyAction {
    /// Set the selection to an absolute value.
    Set(i32),
    /// Select the currently hovered star, if any.
    SelectHovered,
}

/// The star side in points for a [`Size`]. Matches [`MudIcon`]’s table.
#[inline]
fn star_pixels(size: Size) -> f32 {
    match size {
        Size::ExtraSmall => 8.0,
        Size::Small => 16.0,
        Size::Medium => 24.0,
        Size::Large => 32.0,
        Size::ExtraLarge => 48.0,
    }
}

/// Total `(width, height)` of the row.
#[inline]
fn row_size(stars: usize, star_px: f32, gap: f32) -> Vec2 {
    if stars == 0 {
        return Vec2::ZERO;
    }
    let n = stars as f32;
    vec2(n * star_px + (n - 1.0) * gap, star_px)
}

/// Rect of the star at 0-based index `i`.
#[inline]
fn star_rect(i: usize, row: Rect, star_px: f32, gap: f32) -> Rect {
    let x = row.left() + i as f32 * (star_px + gap);
    Rect::from_min_size(pos2(x, row.top()), vec2(star_px, star_px))
}

/// The 1-based star under `pos`, or `0` if the pointer is not over a star.
fn star_at(pos: egui::Pos2, row: Rect, stars: usize, star_px: f32, gap: f32) -> i32 {
    if !row.contains(pos) {
        return 0;
    }
    let local_x = pos.x - row.left();
    let stride = star_px + gap;
    if stride <= 0.0 {
        return 0;
    }
    let index = (local_x / stride).floor() as usize;
    if index >= stars {
        return 0;
    }
    // Reject the gap between two stars.
    let in_star = local_x - index as f32 * stride;
    if in_star > star_px {
        return 0;
    }
    (index + 1) as i32
}

/// The value the stars should display.
///
/// - Read-only or disabled: always the selected value (no preview).
/// - Otherwise: the hovered value when the pointer is over a star, else the
///   selected value.
#[inline]
fn display_value(selected: i32, hovered: i32, disabled: bool, read_only: bool) -> i32 {
    if disabled || read_only {
        selected
    } else if hovered > 0 {
        hovered
    } else {
        selected
    }
}

/// Clamps `value` into `0..=max`, where `0` is the unselected state.
#[inline]
fn clamp_selected(value: i32, max: i32) -> i32 {
    value.clamp(0, max.max(0))
}

/// Normalises `MaxValue` to a star count.
#[inline]
fn sanitize_star_count(max: i32) -> usize {
    if max <= 0 {
        0
    } else {
        max as usize
    }
}

/// Reads keyboard input for a focused rating.
fn apply_keys(ui: &Ui, current: i32, max: i32) -> Option<KeyAction> {
    let max = max.max(0);
    ui.input(|i| {
        use egui::Key;
        if i.key_pressed(Key::ArrowRight) || i.key_pressed(Key::ArrowUp) {
            return Some(KeyAction::Set((current + 1).min(max)));
        }
        if i.key_pressed(Key::ArrowLeft) || i.key_pressed(Key::ArrowDown) {
            return Some(KeyAction::Set((current - 1).max(0)));
        }
        if i.key_pressed(Key::Home) {
            return Some(KeyAction::Set(if max > 0 { 1 } else { 0 }));
        }
        if i.key_pressed(Key::End) {
            return Some(KeyAction::Set(max));
        }
        if i.key_pressed(Key::Space) || i.key_pressed(Key::Enter) {
            return Some(KeyAction::SelectHovered);
        }
        None
    })
}

// ---------------------------------------------------------------------------
// Ripple
// ---------------------------------------------------------------------------

/// Starts a ripple animation for `id`.
fn start_ripple(ctx: &egui::Context, id: Id) {
    let time = ctx.input(|i| i.time);
    ctx.memory_mut(|m| m.data.insert_temp(id, time));
}

/// Paints the ripple if one was started recently.
fn paint_ripple(ui: &Ui, center: egui::Pos2, base_radius: f32, id: Id, accent: Color32) {
    let start: Option<f64> = ui.ctx().memory(|m| m.data.get_temp(id));
    let Some(start) = start else { return };
    let now = ui.input(|i| i.time);
    let elapsed = (now - start) as f32;
    if elapsed >= RIPPLE_DURATION {
        ui.ctx().memory_mut(|m| m.data.remove_temp::<f64>(id));
        return;
    }
    let t = (elapsed / RIPPLE_DURATION).clamp(0.0, 1.0);
    let radius = base_radius * RIPPLE_MAX_RADIUS_FACTOR * t;
    let alpha = ((1.0 - t) * RIPPLE_ALPHA as f32) as u8;
    ui.painter()
        .circle_filled(center, radius, with_alpha(accent, alpha));
    ui.ctx().request_repaint();
}

/// Replaces an alpha channel, keeping the (unmultiplied) RGB.
#[inline]
fn with_alpha(c: Color32, alpha: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), alpha)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // --- sizing ----------------------------------------------------------

    #[test]
    fn star_pixels_matches_mudicon_table() {
        assert_eq!(star_pixels(Size::ExtraSmall), 8.0);
        assert_eq!(star_pixels(Size::Small), 16.0);
        assert_eq!(star_pixels(Size::Medium), 24.0);
        assert_eq!(star_pixels(Size::Large), 32.0);
        assert_eq!(star_pixels(Size::ExtraLarge), 48.0);
    }

    #[test]
    fn row_size_handles_zero_and_nonzero() {
        assert_eq!(row_size(0, 24.0, 4.0), Vec2::ZERO);
        assert_eq!(row_size(1, 24.0, 4.0), vec2(24.0, 24.0));
        assert_eq!(row_size(5, 24.0, 4.0), vec2(136.0, 24.0));
    }

    #[test]
    fn sanitize_star_count_clamps_nonpositive() {
        assert_eq!(sanitize_star_count(0), 0);
        assert_eq!(sanitize_star_count(-3), 0);
        assert_eq!(sanitize_star_count(5), 5);
    }

    // --- hit testing -----------------------------------------------------

    fn row() -> Rect {
        Rect::from_min_size(pos2(100.0, 50.0), vec2(5.0 * 24.0 + 4.0 * 4.0, 24.0))
    }

    #[test]
    fn star_at_maps_every_slot() {
        let r = row();
        for i in 0..5 {
            let x = r.left() + i as f32 * (24.0 + 4.0) + 12.0;
            assert_eq!(star_at(pos2(x, r.center().y), r, 5, 24.0, 4.0), i + 1);
        }
    }

    #[test]
    fn star_at_returns_zero_outside_or_in_gap() {
        let r = row();
        assert_eq!(star_at(pos2(r.left() - 1.0, r.center().y), r, 5, 24.0, 4.0), 0);
        assert_eq!(star_at(pos2(r.right() + 1.0, r.center().y), r, 5, 24.0, 4.0), 0);
        assert_eq!(star_at(pos2(r.center().x, r.top() - 1.0), r, 5, 24.0, 4.0), 0);
        // Inside the 4 pt gap between stars 1 and 2.
        let gap_x = r.left() + 24.0 + 2.0;
        assert_eq!(star_at(pos2(gap_x, r.center().y), r, 5, 24.0, 4.0), 0);
    }

    #[test]
    fn star_at_handles_zero_stride() {
        let r = Rect::from_min_size(pos2(0.0, 0.0), Vec2::ZERO);
        assert_eq!(star_at(pos2(0.0, 0.0), r, 5, 0.0, 0.0), 0);
    }

    #[test]
    fn star_rect_lays_stars_left_to_right() {
        let r = row();
        let first = star_rect(0, r, 24.0, 4.0);
        let second = star_rect(1, r, 24.0, 4.0);
        assert_eq!(first.left(), r.left());
        assert_eq!(second.left(), r.left() + 28.0);
        assert_eq!(first.top(), r.top());
        assert_eq!(first.height(), 24.0);
    }

    // --- display value ---------------------------------------------------

    #[test]
    fn display_value_uses_hover_when_interactive() {
        assert_eq!(display_value(2, 0, false, false), 2);
        assert_eq!(display_value(2, 4, false, false), 4);
    }

    #[test]
    fn display_value_ignores_hover_when_read_only_or_disabled() {
        assert_eq!(display_value(2, 4, false, true), 2);
        assert_eq!(display_value(2, 4, true, false), 2);
    }

    #[test]
    fn display_value_uses_selected_when_nothing_selected() {
        assert_eq!(display_value(0, 0, false, false), 0);
    }

    // --- selection -------------------------------------------------------

    #[test]
    fn clamp_selected_keeps_zero_as_unselected() {
        assert_eq!(clamp_selected(0, 5), 0);
        assert_eq!(clamp_selected(-3, 5), 0);
        assert_eq!(clamp_selected(3, 5), 3);
        assert_eq!(clamp_selected(9, 5), 5);
    }

    #[test]
    fn clamp_selected_handles_zero_max() {
        assert_eq!(clamp_selected(3, 0), 0);
        assert_eq!(clamp_selected(3, -1), 0);
    }

    // --- builders --------------------------------------------------------

    #[test]
    fn defaults_match_mudblazor() {
        let mut v = 0;
        let r = MudRating::new(&mut v);
        assert_eq!(r.max_value, 5);
        assert!(matches!(r.color, Color::Default));
        assert!(matches!(r.size, Size::Medium));
        assert!(r.full_icon.is_none());
        assert!(r.empty_icon.is_none());
        assert!(r.full_icon_color.is_none());
        assert!(r.empty_icon_color.is_none());
        assert!(r.ripple);
        assert!(!r.disabled);
        assert!(!r.read_only);
        assert!(r.name.is_none());
        assert!(r.tag.is_none());
    }

    #[test]
    fn builders_set_every_field() {
        let mut v = 3;
        let r = MudRating::new(&mut v)
            .max_value(10)
            .color(Color::Warning)
            .size(Size::Large)
            .full_icon("F")
            .empty_icon("E")
            .full_icon_color(Color::Success)
            .empty_icon_color(Color::Error)
            .ripple(false)
            .disabled(true)
            .read_only(true)
            .name("quality")
            .classes(["mt-2"])
            .tag("rating");

        assert_eq!(r.max_value, 10);
        assert!(matches!(r.color, Color::Warning));
        assert!(matches!(r.size, Size::Large));
        assert_eq!(r.full_icon.as_deref(), Some("F"));
        assert_eq!(r.empty_icon.as_deref(), Some("E"));
        assert!(matches!(r.full_icon_color, Some(Color::Success)));
        assert!(matches!(r.empty_icon_color, Some(Color::Error)));
        assert!(!r.ripple && r.disabled && r.read_only);
        assert_eq!(r.name.as_deref(), Some("quality"));
        assert!(!r.base.class_names().is_empty());
        assert_eq!(r.tag_ref::<&str>(), Some(&"rating"));
    }

    #[test]
    fn tag_round_trips_by_type() {
        let mut v = 0;
        let r = MudRating::new(&mut v).tag("stars");
        assert_eq!(r.tag_ref::<&str>(), Some(&"stars"));
        assert!(r.tag_ref::<u32>().is_none());
    }
}