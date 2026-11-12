//! A single radio option.
//!
//! Port of `MudRadio.razor.cs` (MudBlazor 8.13.0). The public parameter surface
//! mirrors the Blazor component exactly:
//!
//! | MudBlazor parameter | Rust field / method |
//! |---------------------|---------------------|
//! | `CheckedIcon` | [`MudRadio::checked_icon`] |
//! | `Dense` | [`MudRadio::dense`] |
//! | `IndeterminateIcon` | [`MudRadio::indeterminate_icon`] (stored, never painted) |
//! | `UncheckedColor` | [`MudRadio::unchecked_color`] |
//! | `UncheckedIcon` | [`MudRadio::unchecked_icon`] |
//! | `ChildContent` | [`MudRadio::new`] / [`MudRadio::content`] |
//! | `Disabled` | [`MudRadio::disabled`] |
//! | `ReadOnly` | [`MudRadio::read_only`] |
//! | `LabelPlacement` | [`MudRadio::label_placement`] |
//! | `Color` | [`MudRadio::color`] |
//! | `Size` | [`MudRadio::size`] |
//! | `Ripple` | [`MudRadio::ripple`] |
//! | `Class` | [`MudRadio::classes`] (stored, not resolved) |
//! | `Tag` | [`MudRadio::tag`] / [`MudRadio::tag_ref`] |
//!
//! `Style` and `UserAttributes` are not ported: egui has no CSS strings and no
//! HTML tag.
//!
//! # Standalone vs. grouped
//!
//! A radio is normally rendered through a [`MudRadioGroup`](super::MudRadioGroup),
//! which owns the `&mut Option<T>` binding and enforces mutual exclusion.
//! [`MudRadio::show`] also works standalone: pass the same `&mut Option<T>` to
//! every radio and they will behave as a group.

use std::any::Any;

use egui::{
    pos2, vec2, Align, Color32, CursorIcon, Id, Layout, Response, Sense, Ui, UiBuilder, Vec2,
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
const ICON_MEDIUM: f32 = 20.0;
/// Icon side in logical pixels for `Large`.
const ICON_LARGE: f32 = 24.0;

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

/// Opacity applied to a disabled radio.
const DISABLED_OPACITY: f32 = 0.38;

/// Alpha of the unchecked outline when no `UncheckedColor` is set.
const UNCHECKED_OUTLINE_ALPHA: u8 = 160;

// ---------------------------------------------------------------------------
// MudRadio
// ---------------------------------------------------------------------------

/// A single radio option.
///
/// Mirrors `MudRadio<T>` in MudBlazor 8.13.0. The `option` field is the value
/// this radio represents; whether it is checked is decided by comparing it
/// against the caller's `&mut Option<T>`.
#[must_use = "MudRadio does nothing until you call `.show(ui, ..)`"]
pub struct MudRadio<T> {
    /// Shared component state (utility classes). Stored for parity.
    pub base: ComponentBase,
    /// The value this radio represents. Mirrors MudBlazor `Value`.
    pub option: T,
    /// Label text. Mirrors MudBlazor `ChildContent`.
    pub label: Option<String>,
    /// Custom content replacing the label. Runs once per frame.
    pub content: Option<Box<dyn FnMut(&mut Ui) + 'static>>,
    /// Label placement. Mirrors `LabelPlacement` (default `End`).
    pub label_placement: Placement,
    /// Compact padding. Mirrors `Dense`.
    pub dense: bool,
    /// Accent colour when checked. Mirrors `Color`.
    pub color: Color,
    /// Outline colour when unchecked. Mirrors `UncheckedColor`.
    pub unchecked_color: Option<Color>,
    /// Icon size. Mirrors `Size`.
    pub size: Size,
    /// Enables the click ripple. Mirrors `Ripple`.
    pub ripple: bool,
    /// Disables this radio. Mirrors `Disabled`.
    pub disabled: bool,
    /// Prevents selection without dimming. Mirrors `ReadOnly`.
    pub read_only: bool,
    /// Glyph when checked. Mirrors `CheckedIcon`.
    pub checked_icon: Option<String>,
    /// Glyph when unchecked. Mirrors `UncheckedIcon`.
    pub unchecked_icon: Option<String>,
    /// Stored for parity; never painted. Mirrors `IndeterminateIcon`.
    pub indeterminate_icon: Option<String>,
    /// Arbitrary user data. Mirrors `Tag`.
    pub tag: Option<Box<dyn Any + Send + Sync>>,
}

impl<T> MudRadio<T> {
    /// Creates a radio for `option` with MudBlazor's default appearance.
    pub fn new(option: T) -> Self {
        Self {
            base: ComponentBase::new(),
            option,
            label: None,
            content: None,
            label_placement: Placement::End,
            dense: false,
            color: Color::Primary,
            unchecked_color: None,
            size: Size::Medium,
            ripple: true,
            disabled: false,
            read_only: false,
            checked_icon: None,
            unchecked_icon: None,
            indeterminate_icon: None,
            tag: None,
        }
    }

    /// Creates a radio with `label` as its caption.
    pub fn labelled(option: T, label: impl Into<String>) -> Self {
        Self::new(option).label(label)
    }

    /// Sets the label text.
    #[inline]
    pub fn label(mut self, text: impl Into<String>) -> Self {
        self.label = Some(text.into());
        self
    }

    /// Replaces the label with custom content.
    #[inline]
    pub fn content(mut self, f: impl FnMut(&mut Ui) + 'static) -> Self {
        self.content = Some(Box::new(f));
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

    /// Disables this radio. Mirrors `Disabled`.
    #[inline]
    pub fn disabled(mut self, b: bool) -> Self {
        self.disabled = b;
        self
    }

    /// Prevents selection without dimming. Mirrors `ReadOnly`.
    #[inline]
    pub fn read_only(mut self, b: bool) -> Self {
        self.read_only = b;
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

    /// Stored for parity; never painted. Mirrors `IndeterminateIcon`.
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
}

impl<T: PartialEq + Clone> MudRadio<T> {
    /// Draws the radio and, if clicked and enabled, selects it.
    ///
    /// `selected` is the caller-owned binding. Clicking this radio sets it to
    /// `Some(self.option.clone())`.
    pub fn show(mut self, ui: &mut Ui, selected: &mut Option<T>) -> RadioResponse {
        let checked = selected.as_ref() == Some(&self.option);
        let theme = MudTheme::fetch(ui.ctx()).unwrap_or_default();
        let palette = theme.current_palette();

        let accent = self
            .color
            .resolve(palette)
            .unwrap_or_else(|| ui.visuals().selection.bg_fill);
        let unchecked_accent = self
            .unchecked_color
            .and_then(|c| c.resolve(palette))
            .unwrap_or_else(|| with_alpha(accent, UNCHECKED_OUTLINE_ALPHA));

        let icon_side = icon_side(self.size);
        let gap = if self.dense { DENSE_GAP } else { GAP };
        let label_text = self.label.as_deref().unwrap_or("");
        let label_width = measure_label(ui, label_text);

        let (total_w, total_h) = layout_size(icon_side, label_width, gap, self.label_placement);
        let enabled = !self.disabled && !self.read_only;
        let sense = if enabled {
            Sense::click()
        } else {
            Sense::hover()
        };

        let (rect, response) = ui.allocate_exact_size(vec2(total_w, total_h), sense);
        if !ui.is_rect_visible(rect) {
            return RadioResponse {
                response,
                selected: false,
            };
        }

        let clicked = response.clicked() && enabled;
        let keyboard = enabled && response.has_focus() && ui.input(|i| {
            i.key_pressed(egui::Key::Space) || i.key_pressed(egui::Key::Enter)
        });
        let changed = clicked || keyboard;

        if changed {
            *selected = Some(self.option.clone());
            if self.ripple {
                start_ripple(ui.ctx(), response.id);
            }
        }

        let icon_rect = icon_rect(rect, self.label_placement, icon_side, label_width, gap);
        let label_rect = label_rect(rect, self.label_placement, icon_side, label_width, gap);

        let glyph = if checked {
            self.checked_icon
                .take()
                .unwrap_or_else(|| mud_egui_icons_material::filled::RADIO_BUTTON_CHECKED.to_owned())
        } else {
            self.unchecked_icon
                .take()
                .unwrap_or_else(|| mud_egui_icons_material::filled::RADIO_BUTTON_UNCHECKED.to_owned())
        };
        let ink = if checked {
            accent
        } else {
            unchecked_accent
        };
        let alpha = if self.disabled { DISABLED_OPACITY } else { 1.0 };

        if self.ripple {
            paint_ripple(ui, icon_rect.center(), icon_side * 0.5, response.id, accent);
        }

        ui.put(
            icon_rect,
            MudIcon::new(glyph)
                .size_px(icon_side)
                .custom_color(ink.gamma_multiply(alpha)),
        );

        if let Some(render) = self.content.as_mut() {
            let mut child = ui.new_child(
                UiBuilder::new()
                    .max_rect(label_rect)
                    .layout(Layout::left_to_right(Align::Center)),
            );
            render(&mut child);
        } else if !label_text.is_empty() {
            let mut child = ui.new_child(
                UiBuilder::new()
                    .max_rect(label_rect)
                    .layout(Layout::left_to_right(Align::Center)),
            );
            let text_color = if self.disabled {
                ui.visuals().text_color().gamma_multiply(DISABLED_OPACITY)
            } else {
                ui.visuals().text_color()
            };
            child.add(
                MudText::new(label_text)
                    .typo(Typo::Body1)
                    .inline(true)
                    .custom_color(text_color),
            );
        }

        // `on_hover_cursor` consumes the `Response` and returns a new one, so
        // the result of this chain must be rebound: the original `response` is
        // moved by whichever arm runs.
        let response = if self.disabled {
            response.on_hover_cursor(CursorIcon::NotAllowed)
        } else if self.read_only {
            response
        } else {
            response.on_hover_cursor(CursorIcon::PointingHand)
        };

        RadioResponse {
            response,
            selected: changed,
        }
    }
}

impl<T: PartialEq + Clone> egui::Widget for MudRadio<T> {
    fn ui(self, ui: &mut Ui) -> Response {
        // Standalone use cannot know the binding; it renders as unchecked and
        // never changes state. Use `show` or a `MudRadioGroup` for a binding.
        let mut selected: Option<T> = None;
        self.show(ui, &mut selected).response
    }
}

// ---------------------------------------------------------------------------
// RadioResponse
// ---------------------------------------------------------------------------

/// What [`MudRadio::show`] did this frame.
#[derive(Debug)]
pub struct RadioResponse {
    /// The radio’s `Response`.
    pub response: Response,
    /// `true` when this radio selected itself this frame.
    pub selected: bool,
}

impl std::ops::Deref for RadioResponse {
    type Target = Response;

    fn deref(&self) -> &Self::Target {
        &self.response
    }
}

// ---------------------------------------------------------------------------
// Pure helpers
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

/// `true` when the placement stacks label and icon vertically.
#[inline]
fn is_vertical(placement: Placement) -> bool {
    matches!(placement, Placement::Top | Placement::Bottom)
}

/// `true` when the label leads the icon.
#[inline]
fn label_leads(placement: Placement) -> bool {
    matches!(placement, Placement::Start | Placement::Left | Placement::Top)
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

/// The icon’s rectangle inside the row.
fn icon_rect(row: egui::Rect, placement: Placement, icon_side: f32, label_width: f32, gap: f32) -> egui::Rect {
    if is_vertical(placement) {
        let y = if label_leads(placement) {
            row.top() + label_width + gap
        } else {
            row.top()
        };
        egui::Rect::from_min_size(pos2(row.left(), y), Vec2::splat(icon_side))
    } else {
        let x = if label_leads(placement) {
            row.left() + label_width + gap
        } else {
            row.left()
        };
        egui::Rect::from_min_size(pos2(x, row.top()), Vec2::splat(icon_side))
    }
}

/// The label’s rectangle inside the row.
fn label_rect(row: egui::Rect, placement: Placement, icon_side: f32, label_width: f32, gap: f32) -> egui::Rect {
    if is_vertical(placement) {
        let y = if label_leads(placement) {
            row.top()
        } else {
            row.top() + icon_side + gap
        };
        egui::Rect::from_min_size(pos2(row.left(), y), vec2(label_width, icon_side))
    } else {
        let x = if label_leads(placement) {
            row.left()
        } else {
            row.left() + icon_side + gap
        };
        egui::Rect::from_min_size(pos2(x, row.top()), vec2(label_width, icon_side))
    }
}

/// Measures a label with `layout_no_wrap`; returns `0.0` for an empty string.
fn measure_label(ui: &Ui, text: &str) -> f32 {
    if text.is_empty() {
        return 0.0;
    }
    ui.painter()
        .layout_no_wrap(text.to_owned(), egui::FontId::proportional(14.0), Color32::WHITE)
        .size()
        .x
}

/// Replaces an alpha channel, keeping the (unmultiplied) RGB.
#[inline]
fn with_alpha(c: Color32, alpha: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), alpha)
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

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icon_side_folds_edge_sizes() {
        assert_eq!(icon_side(Size::ExtraSmall), ICON_SMALL);
        assert_eq!(icon_side(Size::Large), ICON_LARGE);
        assert_eq!(icon_side(Size::ExtraLarge), ICON_LARGE);
    }

    #[test]
    fn placement_helpers_are_consistent() {
        assert!(is_vertical(Placement::Top));
        assert!(label_leads(Placement::Start));
        assert!(!label_leads(Placement::End));
    }

    #[test]
    fn layout_size_handles_horizontal_and_vertical() {
        let (w, h) = layout_size(20.0, 80.0, 8.0, Placement::End);
        assert_eq!((w, h), (108.0, 20.0));
        let (w, h) = layout_size(20.0, 80.0, 8.0, Placement::Top);
        assert_eq!((w, h), (80.0, 108.0));
    }

    #[test]
    fn defaults_match_mudblazor() {
        let radio = MudRadio::new(1);
        assert!(matches!(radio.color, Color::Primary));
        assert!(matches!(radio.size, Size::Medium));
        assert!(matches!(radio.label_placement, Placement::End));
        assert!(!radio.dense && !radio.disabled && !radio.read_only && radio.ripple);
        assert!(radio.checked_icon.is_none());
        assert!(radio.unchecked_icon.is_none());
        assert!(radio.indeterminate_icon.is_none());
        assert!(radio.unchecked_color.is_none());
    }

    #[test]
    fn builders_set_every_field() {
        let radio = MudRadio::new("a")
            .label("A")
            .label_placement(Placement::Start)
            .dense(true)
            .color(Color::Success)
            .unchecked_color(Color::Warning)
            .size(Size::Large)
            .ripple(false)
            .disabled(true)
            .read_only(true)
            .checked_icon("C")
            .unchecked_icon("U")
            .indeterminate_icon("I")
            .tag("opt-a");
        assert_eq!(radio.label.as_deref(), Some("A"));
        assert!(matches!(radio.label_placement, Placement::Start));
        assert!(radio.dense && radio.disabled && radio.read_only && !radio.ripple);
        assert!(matches!(radio.color, Color::Success));
        assert!(matches!(radio.unchecked_color, Some(Color::Warning)));
        assert_eq!(radio.checked_icon.as_deref(), Some("C"));
        assert_eq!(radio.unchecked_icon.as_deref(), Some("U"));
        assert_eq!(radio.indeterminate_icon.as_deref(), Some("I"));
        assert_eq!(radio.tag_ref::<&str>(), Some(&"opt-a"));
    }
}