//! A checkbox for boolean or tri-state values.
//!
//! Port of `MudCheckBox.razor.cs` (MudBlazor 8.13.0). The public parameter
//! surface mirrors the Blazor component exactly:
//!
//! | MudBlazor parameter | Rust field / method |
//! |---------------------|---------------------|
//! | `CheckedIcon` | [`MudCheckBox::checked_icon`] |
//! | `Dense` | [`MudCheckBox::dense`] |
//! | `IndeterminateIcon` | [`MudCheckBox::indeterminate_icon`] |
//! | `KeyboardEnabled` | [`MudCheckBox::keyboard_enabled`] |
//! | `TriState` | [`MudCheckBox::tri_state`] |
//! | `UncheckedColor` | [`MudCheckBox::unchecked_color`] |
//! | `UncheckedIcon` | [`MudCheckBox::unchecked_icon`] |
//! | `Value` | caller-owned `&mut T` (see [`CheckBoxValue`]) |
//! | `ValueChanged` | [`CheckBoxResponse::changed`] |
//! | `Label` | [`MudCheckBox::label`] |
//! | `LabelPlacement` | [`MudCheckBox::label_placement`] |
//! | `Ripple` | [`MudCheckBox::ripple`] |
//! | `Size` | [`MudCheckBox::size`] |
//! | `Color` | [`MudCheckBox::color`] |
//! | `Disabled` | [`MudCheckBox::disabled`] |
//! | `ReadOnly` | [`MudCheckBox::read_only`] |
//! | `Class` | [`MudCheckBox::classes`] (stored, not resolved) |
//! | `Tag` | [`MudCheckBox::tag`] / [`MudCheckBox::tag_ref`] |
//! | `Style` | not ported: egui has no inline CSS strings |
//! | `UserAttributes` | not ported: there is no HTML tag |
//!
//! # Binding
//!
//! `MudCheckBox` is generic over [`CheckBoxValue`], which is implemented for
//! `bool` (boolean mode) and `Option<bool>` (tri-state mode):
//!
//! ```ignore
//! let mut accepted = false;
//! MudCheckBox::new(&mut accepted).label("I agree").show(ui);
//!
//! let mut state: Option<bool> = None;
//! MudCheckBox::new(&mut state).tri_state(true).show(ui);
//! ```
//!
//! # Tri-state cycle
//!
//! | Current | Click result |
//! |---------|--------------|
//! | `None` (indeterminate) | `Some(true)` |
//! | `Some(true)` | `Some(false)` |
//! | `Some(false)` | `None` |
//!
//! With `tri_state(false)`, the value toggles between `true` and `false`; a
//! `None` binding becomes `Some(true)` on the first click.
//!
//! # Keyboard
//!
//! When `keyboard_enabled` is true and the checkbox is focused:
//!
//! | Key | Effect |
//! |-----|--------|
//! | `Space` | Cycles the value (same as a click) |
//! | `Enter` | Sets `Some(true)` |
//! | `Delete` | Sets `Some(false)` |
//! | `Backspace` | Sets `None` (only when `tri_state`) |
//!
//! # Icons
//!
//! The default glyphs are `filled::CHECK_BOX` (checked),
//! `filled::CHECK_BOX_OUTLINE_BLANK` (unchecked) and
//! `filled::INDETERMINATE_CHECK_BOX` (indeterminate). Icons are drawn through
//! [`MudIcon`]; the label is drawn through [`MudText`].
//!
//! # Performance
//!
//! - Theme fetched once per frame, and only when the checkbox is not fully
//!   transparent.
//! - Pure helpers for size, tri-state cycling, keyboard, and layout.
//! - Ripple state lives in `egui::Memory`, keyed by the widget id; no per-frame
//!   allocation beyond the `Vec<Shape>` epaint builds internally.
//! - The label is measured once per frame with `layout_no_wrap` and moved into
//!   the paint call.

use std::any::Any;

use egui::{
    pos2, vec2, Align, Color32, CornerRadius, CursorIcon, FontId, Id, Layout, Rect, Response,
    Sense, Shape, Stroke, Ui, UiBuilder, Vec2,
};

use crate::{MudIcon, MudText};
use mud_egui_core::base::component_base::ComponentBase;
use mud_egui_core::enums::{Color, Placement, Size, Typo};
use mud_egui_theming::theme::MudTheme;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Icon side in logical pixels for `Small`.
const ICON_SMALL: f32 = 18.0;
/// Icon side in logical pixels for `Medium`.
const ICON_MEDIUM: f32 = 24.0;
/// Icon side in logical pixels for `Large`.
const ICON_LARGE: f32 = 32.0;

/// Gap between the icon and the label, in logical pixels.
const GAP: f32 = 8.0;
/// Gap when `dense` is true.
const DENSE_GAP: f32 = 4.0;

/// Ripple animation duration in seconds.
const RIPPLE_DURATION: f32 = 0.35;
/// Maximum ripple radius as a multiple of half the icon side.
const RIPPLE_MAX_RADIUS_FACTOR: f32 = 1.2;
/// Alpha of the ripple fill.
const RIPPLE_ALPHA: u8 = 60;

/// Opacity applied to a disabled checkbox.
const DISABLED_OPACITY: f32 = 0.38;

/// Alpha of the unchecked outline when no `UncheckedColor` is set.
const UNCHECKED_OUTLINE_ALPHA: u8 = 160;

// ---------------------------------------------------------------------------
// CheckBoxValue
// ---------------------------------------------------------------------------

/// A binding a [`MudCheckBox`] can write to.
///
/// Implemented for `bool` (boolean mode) and `Option<bool>` (tri-state mode).
/// The trait exists so a single component serves both modes without duplicating
/// the rendering path.
pub trait CheckBoxValue {
    /// Current value as an `Option<bool>`; `None` means indeterminate.
    fn get(&self) -> Option<bool>;
    /// Replaces the value.
    fn set(&mut self, value: Option<bool>);
}

impl CheckBoxValue for bool {
    #[inline]
    fn get(&self) -> Option<bool> {
        Some(*self)
    }

    #[inline]
    fn set(&mut self, value: Option<bool>) {
        *self = value.unwrap_or(false);
    }
}

impl CheckBoxValue for Option<bool> {
    #[inline]
    fn get(&self) -> Option<bool> {
        *self
    }

    #[inline]
    fn set(&mut self, value: Option<bool>) {
        *self = value;
    }
}

// ---------------------------------------------------------------------------
// MudCheckBox
// ---------------------------------------------------------------------------

/// A checkbox for boolean or tri-state values.
///
/// Mirrors `MudCheckBox<T>` in MudBlazor 8.13.0. The Blazor two-way binding
/// `Value` / `ValueChanged` becomes a caller-owned `&mut T` plus a
/// [`CheckBoxResponse::changed`] report, the same split `MudAlert` and
/// `MudCollapse` use.
#[must_use = "MudCheckBox does nothing until you call `.show(ui)`"]
pub struct MudCheckBox<'a, T: CheckBoxValue> {
    /// Shared component state (utility classes). Stored for parity.
    pub base: ComponentBase,
    /// The bound value. `bool` for boolean mode, `Option<bool>` for tri-state.
    pub value: &'a mut T,
    /// Label text. Mirrors `Label`.
    pub label: Option<String>,
    /// Label placement. Mirrors `LabelPlacement` (default `End`).
    pub label_placement: Placement,
    /// Compact padding. Mirrors `Dense`.
    pub dense: bool,
    /// Accent colour of the checked state. Mirrors `Color`.
    pub color: Color,
    /// Colour of the unchecked outline. Mirrors `UncheckedColor`.
    pub unchecked_color: Option<Color>,
    /// Icon size preset. Mirrors `Size`.
    pub size: Size,
    /// Enables the click ripple. Mirrors `Ripple`.
    pub ripple: bool,
    /// Disables all interaction and dims the component. Mirrors `Disabled`.
    pub disabled: bool,
    /// Prevents value changes without dimming. Mirrors `ReadOnly`.
    pub read_only: bool,
    /// Enables the indeterminate state. Mirrors `TriState`.
    pub tri_state: bool,
    /// Enables keyboard control. Mirrors `KeyboardEnabled` (default `true`).
    pub keyboard_enabled: bool,
    /// Glyph for the checked state. Mirrors `CheckedIcon`.
    pub checked_icon: Option<String>,
    /// Glyph for the unchecked state. Mirrors `UncheckedIcon`.
    pub unchecked_icon: Option<String>,
    /// Glyph for the indeterminate state. Mirrors `IndeterminateIcon`.
    pub indeterminate_icon: Option<String>,
    /// Arbitrary user data. Mirrors `Tag`.
    pub tag: Option<Box<dyn Any + Send + Sync>>,
}

impl<'a, T: CheckBoxValue> MudCheckBox<'a, T> {
    /// Creates a checkbox bound to `value` with MudBlazor's default settings.
    pub fn new(value: &'a mut T) -> Self {
        Self {
            base: ComponentBase::new(),
            value,
            label: None,
            label_placement: Placement::End,
            dense: false,
            color: Color::Primary,
            unchecked_color: None,
            size: Size::Medium,
            ripple: true,
            disabled: false,
            read_only: false,
            tri_state: false,
            keyboard_enabled: true,
            checked_icon: None,
            unchecked_icon: None,
            indeterminate_icon: None,
            tag: None,
        }
    }

    /// Sets the label text. Mirrors `Label`.
    #[inline]
    pub fn label(mut self, text: impl Into<String>) -> Self {
        self.label = Some(text.into());
        self
    }

    /// Sets the label placement. Mirrors `LabelPlacement`.
    #[inline]
    pub fn label_placement(mut self, p: Placement) -> Self {
        self.label_placement = p;
        self
    }

    /// Enables compact padding. Mirrors `Dense`.
    #[inline]
    pub fn dense(mut self, b: bool) -> Self {
        self.dense = b;
        self
    }

    /// Sets the accent colour. Mirrors `Color`.
    #[inline]
    pub fn color(mut self, c: Color) -> Self {
        self.color = c;
        self
    }

    /// Sets the unchecked outline colour. Mirrors `UncheckedColor`.
    #[inline]
    pub fn unchecked_color(mut self, c: Color) -> Self {
        self.unchecked_color = Some(c);
        self
    }

    /// Sets the icon size. Mirrors `Size`.
    #[inline]
    pub fn size(mut self, s: Size) -> Self {
        self.size = s;
        self
    }

    /// Enables the click ripple. Mirrors `Ripple`.
    #[inline]
    pub fn ripple(mut self, b: bool) -> Self {
        self.ripple = b;
        self
    }

    /// Disables all interaction and dims the component. Mirrors `Disabled`.
    #[inline]
    pub fn disabled(mut self, b: bool) -> Self {
        self.disabled = b;
        self
    }

    /// Prevents value changes without dimming. Mirrors `ReadOnly`.
    #[inline]
    pub fn read_only(mut self, b: bool) -> Self {
        self.read_only = b;
        self
    }

    /// Enables the indeterminate state. Mirrors `TriState`.
    #[inline]
    pub fn tri_state(mut self, b: bool) -> Self {
        self.tri_state = b;
        self
    }

    /// Enables keyboard control. Mirrors `KeyboardEnabled`.
    #[inline]
    pub fn keyboard_enabled(mut self, b: bool) -> Self {
        self.keyboard_enabled = b;
        self
    }

    /// Overrides the checked glyph. Mirrors `CheckedIcon`.
    #[inline]
    pub fn checked_icon(mut self, icon: impl Into<String>) -> Self {
        self.checked_icon = Some(icon.into());
        self
    }

    /// Overrides the unchecked glyph. Mirrors `UncheckedIcon`.
    #[inline]
    pub fn unchecked_icon(mut self, icon: impl Into<String>) -> Self {
        self.unchecked_icon = Some(icon.into());
        self
    }

    /// Overrides the indeterminate glyph. Mirrors `IndeterminateIcon`.
    #[inline]
    pub fn indeterminate_icon(mut self, icon: impl Into<String>) -> Self {
        self.indeterminate_icon = Some(icon.into());
        self
    }

    /// Adds utility classes. Stored for parity.
    pub fn classes<S: AsRef<str>>(mut self, classes: impl IntoIterator<Item = S>) -> Self {
        self.base = self.base.classes(classes);
        self
    }

    /// Attaches arbitrary user data. Mirrors `Tag`.
    #[inline]
    pub fn tag<U: Any + Send + Sync>(mut self, tag: U) -> Self {
        self.tag = Some(Box::new(tag));
        self
    }

    /// Reads user data attached with [`tag`](Self::tag).
    #[inline]
    pub fn tag_ref<U: Any + Send + Sync>(&self) -> Option<&U> {
        self.tag.as_ref().and_then(|b| b.downcast_ref::<U>())
    }

    /// Draws the checkbox and reports this frame's change.
    pub fn show(mut self, ui: &mut Ui) -> CheckBoxResponse {
        let current = self.value.get();
        let theme = MudTheme::fetch(ui.ctx()).unwrap_or_default();
        let palette = theme.current_palette();

        let accent = self.color.resolve(palette).unwrap_or_else(|| {
            ui.visuals().selection.bg_fill
        });
        let unchecked_accent = self
            .unchecked_color
            .and_then(|c| c.resolve(palette))
            .unwrap_or_else(|| with_alpha(accent, UNCHECKED_OUTLINE_ALPHA));

        let icon_side = icon_side(self.size);
        let gap = if self.dense { DENSE_GAP } else { GAP };
        let label_font = FontId::proportional(14.0);

        // --- measure label --------------------------------------------------
        let label_text = self.label.as_deref().unwrap_or("");
        let label_width = if label_text.is_empty() {
            0.0
        } else {
            ui.painter()
                .layout_no_wrap(label_text.to_owned(), label_font.clone(), Color32::WHITE)
                .size()
                .x
        };

        let (total_w, total_h) = layout_size(
            icon_side,
            label_width,
            gap,
            self.label_placement,
        );

        let sense = if self.disabled || self.read_only {
            Sense::hover()
        } else {
            Sense::click()
        };

        let (rect, response) = ui.allocate_exact_size(vec2(total_w, total_h), sense);
        if !ui.is_rect_visible(rect) {
            return CheckBoxResponse {
                response,
                changed: false,
            };
        }

        // --- interaction ----------------------------------------------------
        let enabled = !self.disabled && !self.read_only;
        let clicked = response.clicked() && enabled;
        let key_result = if self.keyboard_enabled && enabled && response.has_focus() {
            apply_keys(ui, current, self.tri_state)
        } else {
            None
        };

        let new_value = if clicked {
            Some(next_value(current, self.tri_state))
        } else {
            key_result
        };

        let changed = if let Some(v) = new_value {
            self.value.set(v);
            true
        } else {
            false
        };

        if clicked && self.ripple {
            start_ripple(ui.ctx(), response.id);
        }

        // --- paint ----------------------------------------------------------
        let icon_rect = icon_rect(rect, self.label_placement, icon_side, label_width, gap);
        let label_rect = label_rect(rect, self.label_placement, icon_side, label_width, gap);

        let state = paint_state(self.value.get(), self.tri_state);
        let glyph = resolve_icon(
            state,
            self.checked_icon.as_deref(),
            self.unchecked_icon.as_deref(),
            self.indeterminate_icon.as_deref(),
        );
        let icon_color = if state == PaintState::Unchecked {
            unchecked_accent
        } else {
            accent
        };
        let alpha = if self.disabled { DISABLED_OPACITY } else { 1.0 };

        // Ripple
        if self.ripple {
            paint_ripple(ui, icon_rect.center(), icon_side * 0.5, response.id, accent);
        }

        ui.put(
            icon_rect,
            MudIcon::new(glyph.to_owned())
                .size_px(icon_side)
                .custom_color(icon_color.gamma_multiply(alpha)),
        );

        if !label_text.is_empty() {
            let mut child = ui.new_child(
                UiBuilder::new()
                    .max_rect(label_rect)
                    .layout(Layout::left_to_right(Align::Center)),
            );
            let color = if self.disabled {
                Color32::PLACEHOLDER.gamma_multiply(DISABLED_OPACITY)
            } else {
                child.visuals().text_color()
            };
            child.add(
                MudText::new(label_text)
                    .typo(Typo::Body1)
                    .inline(true)
                    .custom_color(color),
            );
        }

        CheckBoxResponse { response, changed }
    }
}

impl<'a, T: CheckBoxValue> egui::Widget for MudCheckBox<'a, T> {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui).response
    }
}

// ---------------------------------------------------------------------------
// CheckBoxResponse
// ---------------------------------------------------------------------------

/// What [`MudCheckBox::show`] did this frame.
#[derive(Debug)]
pub struct CheckBoxResponse {
    /// The checkbox's `Response` (the whole row, icon + label).
    pub response: Response,
    /// `true` when the value changed this frame.
    pub changed: bool,
}

impl std::ops::Deref for CheckBoxResponse {
    type Target = Response;

    fn deref(&self) -> &Self::Target {
        &self.response
    }
}

// ---------------------------------------------------------------------------
// PaintState
// ---------------------------------------------------------------------------

/// The visual state a checkbox glyph represents.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PaintState {
    Checked,
    Unchecked,
    Indeterminate,
}

// ---------------------------------------------------------------------------
// Pure helpers (no `egui::Context` required)
// ---------------------------------------------------------------------------

/// Icon side in logical pixels for a [`Size`].
#[inline]
fn icon_side(size: Size) -> f32 {
    match size {
        Size::ExtraSmall | Size::Small => ICON_SMALL,
        Size::Medium => ICON_MEDIUM,
        Size::Large | Size::ExtraLarge => ICON_LARGE,
    }
}

/// The value after one toggle.
fn next_value(current: Option<bool>, tri_state: bool) -> Option<bool> {
    if tri_state {
        match current {
            None => Some(true),
            Some(true) => Some(false),
            Some(false) => None,
        }
    } else {
        Some(!current.unwrap_or(false))
    }
}

/// `(width, height)` of the whole row.
fn layout_size(icon_side: f32, label_width: f32, gap: f32, placement: Placement) -> (f32, f32) {
    let has_label = label_width > 0.0;
    if is_vertical(placement) {
        let w = icon_side.max(label_width);
        let h = icon_side + if has_label { gap + label_width } else { 0.0 };
        (w, h)
    } else {
        let w = icon_side + if has_label { gap + label_width } else { 0.0 };
        (w, icon_side)
    }
}

/// `true` when the placement stacks label and icon vertically.
fn is_vertical(placement: Placement) -> bool {
    matches!(placement, Placement::Top | Placement::Bottom)
}

/// `true` when the label leads the icon.
fn label_leads(placement: Placement) -> bool {
    matches!(
        placement,
        Placement::Start | Placement::Left | Placement::Top
    )
}

/// The icon's rectangle inside the row.
fn icon_rect(
    row: Rect,
    placement: Placement,
    icon_side: f32,
    label_width: f32,
    gap: f32,
) -> Rect {
    if is_vertical(placement) {
        let y = if label_leads(placement) {
            row.top() + label_width + gap
        } else {
            row.top()
        };
        Rect::from_min_size(pos2(row.left(), y), Vec2::splat(icon_side))
    } else {
        let x = if label_leads(placement) {
            row.left() + label_width + gap
        } else {
            row.left()
        };
        Rect::from_min_size(pos2(x, row.top()), Vec2::splat(icon_side))
    }
}

/// The label's rectangle inside the row.
fn label_rect(
    row: Rect,
    placement: Placement,
    icon_side: f32,
    label_width: f32,
    gap: f32,
) -> Rect {
    if is_vertical(placement) {
        let y = if label_leads(placement) {
            row.top()
        } else {
            row.top() + icon_side + gap
        };
        Rect::from_min_size(pos2(row.left(), y), vec2(label_width, icon_side))
    } else {
        let x = if label_leads(placement) {
            row.left()
        } else {
            row.left() + icon_side + gap
        };
        Rect::from_min_size(pos2(x, row.top()), vec2(label_width, icon_side))
    }
}

/// Maps the current value and tri-state flag to a paint state.
fn paint_state(value: Option<bool>, tri_state: bool) -> PaintState {
    match value {
        Some(true) => PaintState::Checked,
        Some(false) | None if tri_state && value.is_none() => PaintState::Indeterminate,
        Some(false) | None => PaintState::Unchecked,
    }
}

/// The glyph for a paint state, honouring explicit overrides.
fn resolve_icon<'a>(
    state: PaintState,
    checked: Option<&'a str>,
    unchecked: Option<&'a str>,
    indeterminate: Option<&'a str>,
) -> &'a str {
    use mud_egui_icons_material::filled;
    match state {
        PaintState::Checked => checked.unwrap_or(filled::CHECK_BOX),
        PaintState::Unchecked => unchecked.unwrap_or(filled::CHECK_BOX_OUTLINE_BLANK),
        PaintState::Indeterminate => {
            indeterminate.unwrap_or(filled::INDETERMINATE_CHECK_BOX)
        }
    }
}

/// Replaces an alpha channel, keeping the (unmultiplied) RGB.
#[inline]
fn with_alpha(c: Color32, alpha: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), alpha)
}

/// Reads keyboard input for a focused checkbox.
fn apply_keys(ui: &Ui, current: Option<bool>, tri_state: bool) -> Option<Option<bool>> {
    ui.input(|i| {
        if i.key_pressed(egui::Key::Space) {
            return Some(next_value(current, tri_state));
        }
        if i.key_pressed(egui::Key::Enter) {
            return Some(Some(true));
        }
        if i.key_pressed(egui::Key::Delete) {
            return Some(Some(false));
        }
        if tri_state && i.key_pressed(egui::Key::Backspace) {
            return Some(None);
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
    let color = with_alpha(accent, alpha);
    ui.painter().circle_filled(center, radius, color);
    ui.ctx().request_repaint();
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icon_side_maps_every_size() {
        assert_eq!(icon_side(Size::ExtraSmall), ICON_SMALL);
        assert_eq!(icon_side(Size::Small), ICON_SMALL);
        assert_eq!(icon_side(Size::Medium), ICON_MEDIUM);
        assert_eq!(icon_side(Size::Large), ICON_LARGE);
        assert_eq!(icon_side(Size::ExtraLarge), ICON_LARGE);
    }

    #[test]
    fn next_value_boolean_toggles() {
        assert_eq!(next_value(Some(true), false), Some(false));
        assert_eq!(next_value(Some(false), false), Some(true));
        assert_eq!(next_value(None, false), Some(true));
    }

    #[test]
    fn next_value_tri_state_cycles() {
        assert_eq!(next_value(None, true), Some(true));
        assert_eq!(next_value(Some(true), true), Some(false));
        assert_eq!(next_value(Some(false), true), None);
    }

    #[test]
    fn next_value_tri_state_cycles_back_to_indeterminate() {
        let mut v: Option<bool> = None;
        v = next_value(v, true);
        v = next_value(v, true);
        v = next_value(v, true);
        assert_eq!(v, None);
    }

    #[test]
    fn placement_helpers_are_consistent() {
        assert!(is_vertical(Placement::Top));
        assert!(is_vertical(Placement::Bottom));
        assert!(!is_vertical(Placement::Start));
        assert!(!is_vertical(Placement::End));

        assert!(label_leads(Placement::Start));
        assert!(label_leads(Placement::Left));
        assert!(label_leads(Placement::Top));
        assert!(!label_leads(Placement::End));
        assert!(!label_leads(Placement::Right));
        assert!(!label_leads(Placement::Bottom));
    }

    #[test]
    fn layout_size_horizontal_and_vertical() {
        let (w, h) = layout_size(24.0, 80.0, 8.0, Placement::End);
        assert_eq!((w, h), (112.0, 24.0));

        let (w, h) = layout_size(24.0, 80.0, 8.0, Placement::Top);
        assert_eq!((w, h), (80.0, 112.0));

        let (w, h) = layout_size(24.0, 0.0, 8.0, Placement::End);
        assert_eq!((w, h), (24.0, 24.0));
    }

    #[test]
    fn icon_rect_follows_placement() {
        let row = Rect::from_min_size(pos2(10.0, 20.0), vec2(112.0, 24.0));
        let r = icon_rect(row, Placement::End, 24.0, 80.0, 8.0);
        assert_eq!(r.left(), row.left());
        let r = icon_rect(row, Placement::Start, 24.0, 80.0, 8.0);
        assert_eq!(r.left(), row.left() + 88.0);
    }

    #[test]
    fn label_rect_follows_placement() {
        let row = Rect::from_min_size(pos2(0.0, 0.0), vec2(112.0, 24.0));
        let r = label_rect(row, Placement::End, 24.0, 80.0, 8.0);
        assert_eq!(r.left(), 32.0);
        let r = label_rect(row, Placement::Start, 24.0, 80.0, 8.0);
        assert_eq!(r.left(), 0.0);
    }

    #[test]
    fn paint_state_boolean_mode() {
        assert_eq!(paint_state(Some(true), false), PaintState::Checked);
        assert_eq!(paint_state(Some(false), false), PaintState::Unchecked);
        assert_eq!(paint_state(None, false), PaintState::Unchecked);
    }

    #[test]
    fn paint_state_tri_state_mode() {
        assert_eq!(paint_state(Some(true), true), PaintState::Checked);
        assert_eq!(paint_state(Some(false), true), PaintState::Unchecked);
        assert_eq!(paint_state(None, true), PaintState::Indeterminate);
    }

    #[test]
    fn resolve_icon_uses_overrides_or_defaults() {
        let d = resolve_icon(PaintState::Checked, None, None, None);
        assert!(!d.is_empty());
        let o = resolve_icon(PaintState::Checked, Some("M0 0"), None, None);
        assert_eq!(o, "M0 0");
    }

    #[test]
    fn check_box_value_for_bool_round_trips() {
        let mut b = true;
        assert_eq!(b.get(), Some(true));
        b.set(Some(false));
        assert_eq!(b.get(), Some(false));
        b.set(None);
        assert_eq!(b.get(), Some(false));
    }

    #[test]
    fn check_box_value_for_option_round_trips() {
        let mut o: Option<bool> = None;
        assert_eq!(o.get(), None);
        o.set(Some(true));
        assert_eq!(o.get(), Some(true));
        o.set(None);
        assert_eq!(o.get(), None);
    }

    #[test]
    fn defaults_match_mudblazor() {
        let mut v = false;
        let c = MudCheckBox::new(&mut v);
        assert!(matches!(c.color, Color::Primary));
        assert!(matches!(c.size, Size::Medium));
        assert!(matches!(c.label_placement, Placement::End));
        assert!(!c.dense);
        assert!(!c.disabled);
        assert!(!c.read_only);
        assert!(!c.tri_state);
        assert!(c.keyboard_enabled);
        assert!(c.ripple);
        assert!(c.label.is_none());
        assert!(c.checked_icon.is_none());
        assert!(c.unchecked_icon.is_none());
        assert!(c.indeterminate_icon.is_none());
        assert!(c.tag.is_none());
    }

    #[test]
    fn builders_set_every_field() {
        let mut v: Option<bool> = None;
        let c = MudCheckBox::new(&mut v)
            .label("Accept")
            .label_placement(Placement::Start)
            .dense(true)
            .color(Color::Success)
            .unchecked_color(Color::Warning)
            .size(Size::Large)
            .ripple(false)
            .disabled(true)
            .read_only(true)
            .tri_state(true)
            .keyboard_enabled(false)
            .checked_icon("A")
            .unchecked_icon("B")
            .indeterminate_icon("C")
            .classes(["mt-2"])
            .tag("terms");

        assert_eq!(c.label.as_deref(), Some("Accept"));
        assert!(matches!(c.label_placement, Placement::Start));
        assert!(c.dense && c.disabled && c.read_only && c.tri_state && !c.keyboard_enabled && !c.ripple);
        assert!(matches!(c.color, Color::Success));
        assert!(matches!(c.unchecked_color, Some(Color::Warning)));
        assert!(matches!(c.size, Size::Large));
        assert_eq!(c.checked_icon.as_deref(), Some("A"));
        assert_eq!(c.unchecked_icon.as_deref(), Some("B"));
        assert_eq!(c.indeterminate_icon.as_deref(), Some("C"));
        assert_eq!(c.tag_ref::<&str>(), Some(&"terms"));
        assert!(c.tag_ref::<u32>().is_none());
    }
}