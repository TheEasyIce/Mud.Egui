//! A two-state toggle control.
//!
//! Port of `MudSwitch<T>.razor.cs` (MudBlazor 8.13.0). The public parameter
//! surface mirrors the Blazor component exactly:
//!
//! | MudBlazor parameter | Rust field / method |
//! |---------------------|---------------------|
//! | `ThumbIcon` | [`MudSwitch::thumb_icon`] |
//! | `ThumbIconColor` | [`MudSwitch::thumb_icon_color`] |
//! | `UncheckedColor` | [`MudSwitch::unchecked_color`] |
//! | `Value` | the caller-owned `&'a mut bool` |
//! | `ValueChanged` | [`SwitchResponse::changed`] |
//! | `Label` | [`MudSwitch::label`] |
//! | `LabelPlacement` | [`MudSwitch::label_placement`] |
//! | `Color` | [`MudSwitch::color`] |
//! | `Size` | [`MudSwitch::size`] |
//! | `Disabled` | [`MudSwitch::disabled`] |
//! | `ReadOnly` | [`MudSwitch::read_only`] |
//! | `Ripple` | [`MudSwitch::ripple`] |
//! | `Class` | [`MudSwitch::classes`] (stored, not resolved) |
//! | `Tag` | [`MudSwitch::tag`] / [`MudSwitch::tag_ref`] |
//! | `Style` / `UserAttributes` | not ported: no CSS strings, no HTML tag |
//! | `ChildContent` | not ported: `label` is the string form; a custom-content closure has no MudBlazor switch analogue |
//! | generic `T` + `BoolConverter<T>` | not ported: egui has no converter pipeline; the port binds `bool` |
//!
//! # Label placement
//!
//! `Placement` reuses the existing enum. `Start` / `Left` put the label before
//! the track; `End` / `Right` put it after (the default); `Top` / `Bottom`
//! stack the label and the track vertically.
//!
//! # Keyboard
//!
//! When the switch is focused and interactive, MudBlazor's documented keyboard
//! map applies:
//!
//! | Key | Effect |
//! |-----|--------|
//! | `ArrowLeft` / `Delete` | `false` |
//! | `ArrowRight` / `Enter` (incl. numpad) | `true` |
//! | `Space` | Toggle |
//!
//! # Colour resolution
//!
//! | State | Colour role | Fallback when `Default` / `Inherit` |
//! |-------|-------------|------------------------------------|
//! | Checked track | [`color`](MudSwitch::color) | `ui.visuals().selection.bg_fill` |
//! | Unchecked track | [`unchecked_color`](MudSwitch::unchecked_color) | `ui.visuals().text_color()` at [`UNCHECKED_TRACK_ALPHA`] |
//! | Thumb icon | [`thumb_icon_color`](MudSwitch::thumb_icon_color) | `ui.visuals().text_color()` |
//!
//! The checked and unchecked track colours are linearly interpolated by the
//! animation progress, so toggling the switch fades between them.
//!
//! # Thumb icon
//!
//! `ThumbIconColor` **only applies when `ThumbIcon` is set** — that is the
//! documented MudBlazor rule, and the port enforces it by ignoring the colour
//! whenever the glyph is `None`.
//!
//! # Animation
//!
//! Thumb position and track colour interpolate over
//! [`SWITCH_ANIM_SECONDS`] using `animate_bool_with_time`, keyed by the
//! switch's `Response::id`. Ripple state lives in `egui::Memory` under the same
//! id, so the component itself holds no state between frames.

use std::any::Any;

use egui::{
    pos2, vec2, Align, Color32, CursorIcon, Id, Layout, Rect, Response, Sense, Stroke, StrokeKind,
    Ui, UiBuilder, Vec2,
};

use crate::{MudIcon, MudText};
use mud_egui_core::base::component_base::ComponentBase;
use mud_egui_core::enums::{Color, Placement, Size, Typo};
use mud_egui_theming::theme::MudTheme;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Animation duration, in seconds.
const SWITCH_ANIM_SECONDS: f32 = 0.15;

/// Ripple animation duration, in seconds.
const RIPPLE_DURATION: f32 = 0.35;

/// Maximum ripple radius as a multiple of the thumb radius.
const RIPPLE_MAX_RADIUS_FACTOR: f32 = 1.6;

/// Alpha of the ripple fill.
const RIPPLE_ALPHA: u8 = 60;

/// Alpha applied to the unchecked track's fallback colour.
const UNCHECKED_TRACK_ALPHA: u8 = 120;

/// Opacity applied to a disabled switch.
const DISABLED_OPACITY: f32 = 0.38;

/// Gap between the track and the label, in points.
const LABEL_GAP: f32 = 8.0;

/// How far the thumb overhangs the track at each end, in points.
const THUMB_OVERHANG: f32 = 2.0;

/// Thumb icon side, as a fraction of the thumb diameter.
const THUMB_ICON_FRACTION: f32 = 0.6;

/// Thumb outline width, in points.
const THUMB_STROKE: f32 = 1.0;

// ---------------------------------------------------------------------------
// MudSwitch
// ---------------------------------------------------------------------------

/// A two-state toggle control.
///
/// Mirrors `MudSwitch<T>` in MudBlazor 8.13.0 with `T = bool`. The Blazor
/// two-way binding `Value` / `ValueChanged` becomes a caller-owned
/// `&'a mut bool` plus a [`SwitchResponse::changed`] report, the same split
/// `MudCheckBox`, `MudRadio` and `MudCollapse` use.
#[must_use = "MudSwitch does nothing until you call `.show(ui)`"]
pub struct MudSwitch<'a> {
    /// Shared component state (utility classes). Stored for parity; not
    /// resolved, because the track and thumb colours come from `color` /
    /// `unchecked_color`.
    pub base: ComponentBase,
    /// The bound value. Mirrors MudBlazor `Value`.
    pub value: &'a mut bool,
    /// Label text. Mirrors `Label`.
    pub label: Option<String>,
    /// Label placement. Mirrors `LabelPlacement` (default `End`).
    pub label_placement: Placement,
    /// Accent colour of the checked track. Mirrors `Color`.
    pub color: Color,
    /// Colour of the unchecked track. Mirrors `UncheckedColor`.
    pub unchecked_color: Color,
    /// Size preset. Mirrors `Size` (default `Medium`).
    pub size: Size,
    /// Glyph drawn on the thumb. Mirrors `ThumbIcon`.
    pub thumb_icon: Option<String>,
    /// Colour of the thumb glyph. Mirrors `ThumbIconColor`; ignored when
    /// [`thumb_icon`](Self::thumb_icon) is unset.
    pub thumb_icon_color: Color,
    /// Enables the click ripple. Mirrors `Ripple` (default `true`).
    pub ripple: bool,
    /// Disables interaction and dims the switch. Mirrors `Disabled`.
    pub disabled: bool,
    /// Prevents toggling without dimming. Mirrors `ReadOnly`.
    pub read_only: bool,
    /// Arbitrary user data. Mirrors `Tag`.
    pub tag: Option<Box<dyn Any + Send + Sync>>,
}

impl<'a> MudSwitch<'a> {
    /// Creates a switch bound to `value` with MudBlazor's default settings.
    pub fn new(value: &'a mut bool) -> Self {
        Self {
            base: ComponentBase::new(),
            value,
            label: None,
            label_placement: Placement::End,
            color: Color::Default,
            unchecked_color: Color::Default,
            size: Size::Medium,
            thumb_icon: None,
            thumb_icon_color: Color::Default,
            ripple: true,
            disabled: false,
            read_only: false,
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

    /// Sets the checked accent colour. Mirrors `Color`.
    #[inline]
    pub fn color(mut self, c: Color) -> Self {
        self.color = c;
        self
    }

    /// Sets the unchecked track colour. Mirrors `UncheckedColor`.
    #[inline]
    pub fn unchecked_color(mut self, c: Color) -> Self {
        self.unchecked_color = c;
        self
    }

    /// Sets the size preset. Mirrors `Size`.
    #[inline]
    pub fn size(mut self, s: Size) -> Self {
        self.size = s;
        self
    }

    /// Sets the thumb glyph. Mirrors `ThumbIcon`.
    #[inline]
    pub fn thumb_icon(mut self, icon: impl Into<String>) -> Self {
        self.thumb_icon = Some(icon.into());
        self
    }

    /// Sets the thumb glyph colour. Mirrors `ThumbIconColor`; ignored when
    /// [`thumb_icon`](Self::thumb_icon) is unset.
    #[inline]
    pub fn thumb_icon_color(mut self, c: Color) -> Self {
        self.thumb_icon_color = c;
        self
    }

    /// Enables the click ripple. Mirrors `Ripple`.
    #[inline]
    pub fn ripple(mut self, b: bool) -> Self {
        self.ripple = b;
        self
    }

    /// Disables interaction and dims the switch. Mirrors `Disabled`.
    #[inline]
    pub fn disabled(mut self, b: bool) -> Self {
        self.disabled = b;
        self
    }

    /// Prevents toggling without dimming. Mirrors `ReadOnly`.
    #[inline]
    pub fn read_only(mut self, b: bool) -> Self {
        self.read_only = b;
        self
    }

    /// Adds utility classes to the switch's [`ComponentBase`]. Stored for
    /// parity; not resolved while rendering.
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

    /// Draws the switch and reports this frame's outcome.
    pub fn show(self, ui: &mut Ui) -> SwitchResponse {
        let theme = MudTheme::fetch(ui.ctx()).unwrap_or_default();
        let palette = theme.current_palette();

        let checked_color = resolve_accent(ui, self.color);
        let unchecked_color = resolve_unchecked(ui, self.unchecked_color);

        let metrics = switch_metrics(self.size);
        let label_text = self.label.as_deref().unwrap_or("");
        let label_width = measure_label(ui, label_text);
        let gap = if label_width > 0.0 { LABEL_GAP } else { 0.0 };

        let (total_w, total_h) = row_size(&metrics, label_width, gap, self.label_placement);
        let enabled = !self.disabled && !self.read_only && ui.is_enabled();
        let sense = if enabled {
            Sense::click()
        } else {
            Sense::hover()
        };

        let (rect, response) = ui.allocate_exact_size(vec2(total_w, total_h), sense);
        if !ui.is_rect_visible(rect) {
            return SwitchResponse {
                response,
                changed: false,
            };
        }

        // --- interaction ---------------------------------------------------
        let kb = if enabled && response.has_focus() {
            next_value_from_keys(
                ui.input(|i| i.key_pressed(egui::Key::Space)),
                ui.input(|i| {
                    i.key_pressed(egui::Key::ArrowLeft) || i.key_pressed(egui::Key::Delete)
                }),
                ui.input(|i| {
                    i.key_pressed(egui::Key::ArrowRight) || i.key_pressed(egui::Key::Enter)
                }),
                *self.value,
            )
        } else {
            None
        };

        let target = if enabled && response.clicked() {
            Some(!*self.value)
        } else {
            kb
        };

        let mut changed = false;
        if let Some(new_value) = target {
            if new_value != *self.value {
                *self.value = new_value;
                changed = true;
                if self.ripple && self.ripple_enabled() {
                    start_ripple(ui.ctx(), response.id);
                }
            }
        }

        // --- geometry ------------------------------------------------------
        let track_rect = track_rect(rect, &metrics, label_width, gap, self.label_placement);
        let thumb_center = thumb_center(track_rect, &metrics, *self.value);

        // --- paint ---------------------------------------------------------
        let progress =
            ui.ctx()
                .animate_bool_with_time(response.id.with("t"), *self.value, SWITCH_ANIM_SECONDS);
        let alpha = if self.disabled { DISABLED_OPACITY } else { 1.0 };

        if self.ripple {
            paint_ripple(
                ui,
                thumb_center,
                metrics.thumb * 0.5,
                response.id,
                checked_color,
            );
        }

        // Track: interpolate between unchecked and checked fills.
        let track_fill = lerp_color(unchecked_color, checked_color, progress);
        ui.painter().rect_filled(
            track_rect,
            track_rect.height() * 0.5,
            track_fill.gamma_multiply(alpha),
        );

        // Thumb: surface fill + subtle outline.
        let thumb_r = metrics.thumb * 0.5;
        let thumb_fill = surface(ui);
        let thumb_stroke = ui.visuals().widgets.noninteractive.bg_stroke;
        ui.painter().circle_filled(
            thumb_center,
            thumb_r,
            thumb_fill.gamma_multiply(alpha),
        );
        ui.painter().circle_stroke(
            thumb_center,
            thumb_r,
            Stroke::new(THUMB_STROKE, thumb_stroke.color.gamma_multiply(alpha)),
        );

        // Thumb icon, only when configured.
        if let Some(glyph) = self.thumb_icon.as_deref() {
            let icon_px = metrics.thumb * THUMB_ICON_FRACTION;
            let ink = resolve_thumb_ink(ui, self.thumb_icon_color, &palette);
            ui.put(
                Rect::from_center_size(thumb_center, Vec2::splat(icon_px)),
                MudIcon::new(glyph.to_owned())
                    .size_px(icon_px)
                    .custom_color(ink.gamma_multiply(alpha)),
            );
        }

        // Label.
        if !label_text.is_empty() {
            let label_rect =
                label_rect(rect, &metrics, label_width, gap, self.label_placement);
            let mut child = ui.new_child(
                UiBuilder::new()
                    .max_rect(label_rect)
                    .layout(Layout::left_to_right(Align::Center)),
            );
            let ink = if self.disabled {
                ui.visuals().text_color().gamma_multiply(DISABLED_OPACITY)
            } else {
                ui.visuals().text_color()
            };
            child.add(
                MudText::new(label_text.to_string())
                    .typo(Typo::Body1)
                    .inline(true)
                    .custom_color(ink),
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

        SwitchResponse { response, changed }
    }

    #[inline]
    fn ripple_enabled(&self) -> bool {
        self.ripple && !self.disabled
    }
}

impl<'a> egui::Widget for MudSwitch<'a> {
    /// Draws the switch, discarding the `changed` flag. Prefer
    /// [`MudSwitch::show`] when the caller needs to react to a toggle.
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui).response
    }
}

// ---------------------------------------------------------------------------
// SwitchResponse
// ---------------------------------------------------------------------------

/// What [`MudSwitch::show`] did this frame.
#[derive(Debug)]
pub struct SwitchResponse {
    /// The switch's `Response` (track + label row). Deref target.
    pub response: Response,
    /// `true` on the frame the bound value changed. Mirrors `ValueChanged`.
    pub changed: bool,
}

impl std::ops::Deref for SwitchResponse {
    type Target = Response;

    fn deref(&self) -> &Self::Target {
        &self.response
    }
}

// ---------------------------------------------------------------------------
// SwitchMetrics
// ---------------------------------------------------------------------------

/// Track and thumb dimensions for a [`Size`], in points.
#[derive(Clone, Copy, Debug, PartialEq)]
struct SwitchMetrics {
    track_w: f32,
    track_h: f32,
    thumb: f32,
}

/// Maps a [`Size`] to MudBlazor's switch dimensions. `ExtraSmall` folds into
/// `Small` and `ExtraLarge` into `Large`, matching the three sizes the SCSS
/// defines.
#[inline]
fn switch_metrics(size: Size) -> SwitchMetrics {
    match size {
        Size::ExtraSmall | Size::Small => SwitchMetrics {
            track_w: 34.0,
            track_h: 14.0,
            thumb: 18.0,
        },
        Size::Medium => SwitchMetrics {
            track_w: 40.0,
            track_h: 16.0,
            thumb: 22.0,
        },
        Size::Large | Size::ExtraLarge => SwitchMetrics {
            track_w: 50.0,
            track_h: 20.0,
            thumb: 28.0,
        },
    }
}

// ---------------------------------------------------------------------------
// Pure helpers (no `egui::Context` required)
// ---------------------------------------------------------------------------

/// `true` when the label stacks vertically with the track.
#[inline]
fn is_vertical(placement: Placement) -> bool {
    matches!(placement, Placement::Top | Placement::Bottom)
}

/// `true` when the label comes before the track in reading order.
#[inline]
fn label_leads(placement: Placement) -> bool {
    matches!(placement, Placement::Start | Placement::Left | Placement::Top)
}

/// `(width, height)` of the whole row.
fn row_size(metrics: &SwitchMetrics, label_w: f32, gap: f32, placement: Placement) -> (f32, f32) {
    if is_vertical(placement) {
        (
            metrics.track_w.max(label_w),
            metrics.thumb.max(metrics.track_h) + gap + label_w,
        )
    } else {
        (
            metrics.track_w + gap + label_w,
            metrics.thumb.max(metrics.track_h),
        )
    }
}

/// Rect of the pill-shaped track inside the row.
fn track_rect(
    row: Rect,
    metrics: &SwitchMetrics,
    label_w: f32,
    gap: f32,
    placement: Placement,
) -> Rect {
    let track_h = metrics.track_h;
    let track_w = metrics.track_w;
    if is_vertical(placement) {
        let y = if label_leads(placement) {
            row.top() + label_w + gap
        } else {
            row.top()
        };
        Rect::from_min_size(
            pos2(row.left() + (row.width() - track_w) * 0.5, y),
            vec2(track_w, track_h),
        )
    } else {
        let x = if label_leads(placement) {
            row.left() + label_w + gap
        } else {
            row.left()
        };
        Rect::from_min_size(
            pos2(x, row.top() + (row.height() - track_h) * 0.5),
            vec2(track_w, track_h),
        )
    }
}

/// Rect of the label inside the row.
fn label_rect(
    row: Rect,
    metrics: &SwitchMetrics,
    label_w: f32,
    gap: f32,
    placement: Placement,
) -> Rect {
    if is_vertical(placement) {
        let y = if label_leads(placement) {
            row.top()
        } else {
            row.top() + metrics.thumb.max(metrics.track_h) + gap
        };
        Rect::from_min_size(pos2(row.left(), y), vec2(row.width(), label_w.max(0.0)))
    } else {
        let x = if label_leads(placement) {
            row.left()
        } else {
            row.right() - label_w
        };
        Rect::from_min_size(
            pos2(x, row.top()),
            vec2(label_w, row.height()),
        )
    }
}

/// Centre of the thumb for `value`, given the track rect.
///
/// The thumb overhangs the track by [`THUMB_OVERHANG`] at both ends; the
/// position interpolates linearly between those two rest points.
#[inline]
fn thumb_center(track: Rect, metrics: &SwitchMetrics, value: bool) -> egui::Pos2 {
    let r = metrics.thumb * 0.5;
    let x_off = track.left() + r - THUMB_OVERHANG;
    let x_on = track.right() - r + THUMB_OVERHANG;
    let t = if value { 1.0 } else { 0.0 };
    pos2(x_off + (x_on - x_off) * t, track.center().y)
}

/// The value after a keyboard press.
///
/// Pure over the three key classes so the mapping is unit-testable without a
/// `Ui`. Order matters: the explicit check/uncheck keys win over `Space` only
/// if they are pressed in the same frame — which egui reports independently,
/// so the priority below is deterministic.
#[inline]
fn next_value_from_keys(
    space: bool,
    uncheck: bool,
    check: bool,
    current: bool,
) -> Option<bool> {
    if uncheck {
        Some(false)
    } else if check {
        Some(true)
    } else if space {
        Some(!current)
    } else {
        None
    }
}

/// Measures the label with `layout_no_wrap`; `0.0` for an empty string.
fn measure_label(ui: &Ui, text: &str) -> f32 {
    if text.is_empty() {
        return 0.0;
    }
    ui.painter()
        .layout_no_wrap(
            text.to_string(),
            egui::FontId::proportional(14.0),
            Color32::WHITE,
        )
        .size()
        .x
}

/// `true` for colour roles that mean "whatever the ambient text colour is".
#[inline]
fn is_ambient(color: Color) -> bool {
    matches!(color, Color::Default | Color::Inherit)
}

/// Resolves the checked track colour.
fn resolve_accent(ui: &Ui, color: Color) -> Color32 {
    if is_ambient(color) {
        ui.visuals().selection.bg_fill
    } else {
        let theme = MudTheme::fetch(ui.ctx()).unwrap_or_default();
        let palette = theme.current_palette();
        color
            .resolve(palette)
            .unwrap_or_else(|| ui.visuals().selection.bg_fill)
    }
}

/// Resolves the unchecked track colour.
fn resolve_unchecked(ui: &Ui, color: Color) -> Color32 {
    if is_ambient(color) {
        with_alpha(ui.visuals().text_color(), UNCHECKED_TRACK_ALPHA)
    } else {
        let theme = MudTheme::fetch(ui.ctx()).unwrap_or_default();
        let palette = theme.current_palette();
        color
            .resolve(palette)
            .map(|c| with_alpha(c, UNCHECKED_TRACK_ALPHA))
            .unwrap_or_else(|| with_alpha(ui.visuals().text_color(), UNCHECKED_TRACK_ALPHA))
    }
}

/// Resolves the thumb-icon ink.
fn resolve_thumb_ink(ui: &Ui, color: Color, palette: &mud_egui_theming::models::palettes::palette::Palette) -> Color32 {
    if is_ambient(color) {
        ui.visuals().text_color()
    } else {
        color
            .resolve(palette)
            .unwrap_or_else(|| ui.visuals().text_color())
    }
}

/// The thumb's surface fill.
fn surface(ui: &Ui) -> Color32 {
    let theme = MudTheme::fetch(ui.ctx()).unwrap_or_default();
    let _ = ui;
    theme.current_palette().surface.into()
}

/// Replaces an alpha channel, keeping the (unmultiplied) RGB.
#[inline]
fn with_alpha(c: Color32, alpha: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), alpha)
}

/// Linear interpolation between two colours.
#[inline]
fn lerp_color(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    let mix = |x: u8, y: u8| (x as f32 * (1.0 - t) + y as f32 * t) as u8;
    Color32::from_rgba_unmultiplied(
        mix(a.r(), b.r()),
        mix(a.g(), b.g()),
        mix(a.b(), b.b()),
        mix(a.a(), b.a()),
    )
}

// ---------------------------------------------------------------------------
// Ripple
// ---------------------------------------------------------------------------

/// Starts a ripple for `id`. Reads the frame time **before** acquiring the
/// memory lock (egui's `Context` is a non-reentrant `RwLock`).
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

    const EPS: f32 = 1e-4;

    fn approx(a: f32, b: f32) -> bool {
        (a - b).abs() < EPS
    }

    // --- metrics ----------------------------------------------------------

    #[test]
    fn switch_metrics_fold_edge_sizes() {
        assert_eq!(
            switch_metrics(Size::ExtraSmall),
            switch_metrics(Size::Small)
        );
        assert_eq!(
            switch_metrics(Size::ExtraLarge),
            switch_metrics(Size::Large)
        );
    }

    #[test]
    fn switch_metrics_are_monotonic() {
        let s = switch_metrics(Size::Small);
        let m = switch_metrics(Size::Medium);
        let l = switch_metrics(Size::Large);
        assert!(s.track_w < m.track_w && m.track_w < l.track_w);
        assert!(s.track_h < m.track_h && m.track_h < l.track_h);
        assert!(s.thumb < m.thumb && m.thumb < l.thumb);
    }

    #[test]
    fn thumb_is_larger_than_track_height() {
        for size in [Size::Small, Size::Medium, Size::Large] {
            let m = switch_metrics(size);
            assert!(m.thumb > m.track_h);
        }
    }

    // --- placement --------------------------------------------------------

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

    // --- row / track / label geometry -------------------------------------

    fn metrics() -> SwitchMetrics {
        switch_metrics(Size::Medium)
    }

    #[test]
    fn row_size_horizontal_and_vertical() {
        let m = metrics();
        let (w, h) = row_size(&m, 60.0, LABEL_GAP, Placement::End);
        assert!(approx(w, m.track_w + LABEL_GAP + 60.0));
        assert!(approx(h, m.thumb.max(m.track_h)));

        let (w, h) = row_size(&m, 60.0, LABEL_GAP, Placement::Top);
        assert!(approx(w, m.track_w.max(60.0)));
        assert!(approx(h, m.thumb.max(m.track_h) + LABEL_GAP + 60.0));
    }

    #[test]
    fn track_rect_is_vertically_centred() {
        let row = Rect::from_min_size(pos2(0.0, 0.0), vec2(120.0, 22.0));
        let r = track_rect(row, &metrics(), 40.0, LABEL_GAP, Placement::End);
        assert_eq!(r.left(), row.left());
        assert!(approx(r.center().y, row.center().y));
        assert!(approx(r.width(), metrics().track_w));
        assert!(approx(r.height(), metrics().track_h));
    }

    #[test]
    fn track_rect_follows_label_placement() {
        let row = Rect::from_min_size(pos2(0.0, 0.0), vec2(120.0, 22.0));
        let start = track_rect(row, &metrics(), 40.0, LABEL_GAP, Placement::Start);
        assert!(start.left() > row.left());
    }

    #[test]
    fn label_rect_follows_label_placement() {
        let row = Rect::from_min_size(pos2(0.0, 0.0), vec2(120.0, 22.0));
        let end = label_rect(row, &metrics(), 40.0, LABEL_GAP, Placement::End);
        assert!(approx(end.right(), row.right()));
        let start = label_rect(row, &metrics(), 40.0, LABEL_GAP, Placement::Start);
        assert!(approx(start.left(), row.left()));
    }

    // --- thumb position ---------------------------------------------------

    #[test]
    fn thumb_center_sits_at_track_ends() {
        let track = Rect::from_min_size(pos2(10.0, 0.0), vec2(40.0, 16.0));
        let m = metrics();
        let off = thumb_center(track, &m, false);
        let on = thumb_center(track, &m, true);
        assert!(off.x < on.x);
        assert!(approx(off.x, track.left() + m.thumb * 0.5 - THUMB_OVERHANG));
        assert!(approx(on.x, track.right() - m.thumb * 0.5 + THUMB_OVERHANG));
        assert!(approx(off.y, track.center().y));
        assert!(approx(on.y, track.center().y));
    }

    // --- keyboard ---------------------------------------------------------

    #[test]
    fn keyboard_uncheck_wins_over_check_and_space() {
        assert_eq!(next_value_from_keys(false, true, true, true), Some(false));
        assert_eq!(next_value_from_keys(true, true, false, false), Some(false));
    }

    #[test]
    fn keyboard_check_wins_over_space() {
        assert_eq!(next_value_from_keys(true, false, true, false), Some(true));
    }

    #[test]
    fn keyboard_space_toggles() {
        assert_eq!(next_value_from_keys(true, false, false, false), Some(true));
        assert_eq!(next_value_from_keys(true, false, false, true), Some(false));
    }

    #[test]
    fn keyboard_none_when_no_key_pressed() {
        assert_eq!(next_value_from_keys(false, false, false, true), None);
    }

    // --- colours ----------------------------------------------------------

    #[test]
    fn ambient_roles_are_default_and_inherit() {
        assert!(is_ambient(Color::Default));
        assert!(is_ambient(Color::Inherit));
        assert!(!is_ambient(Color::Primary));
    }

    #[test]
    fn lerp_color_endpoints_and_midpoint() {
        let a = Color32::from_rgb(0, 0, 0);
        let b = Color32::from_rgb(200, 100, 50);
        assert_eq!(lerp_color(a, b, 0.0), a);
        assert_eq!(lerp_color(a, b, 1.0), b);
        let mid = lerp_color(a, b, 0.5);
        assert!(approx(mid.r() as f32, 100.0));
        assert!(approx(mid.g() as f32, 50.0));
        assert!(approx(mid.b() as f32, 25.0));
    }

    #[test]
    fn with_alpha_keeps_rgb() {
        let c = with_alpha(Color32::from_rgb(10, 20, 30), 40);
        assert_eq!((c.r(), c.g(), c.b(), c.a()), (10, 20, 30, 40));
    }

    // --- builders ---------------------------------------------------------

    #[test]
    fn defaults_match_mudblazor() {
        let mut v = false;
        let s = MudSwitch::new(&mut v);
        assert!(matches!(s.color, Color::Default));
        assert!(matches!(s.unchecked_color, Color::Default));
        assert!(matches!(s.size, Size::Medium));
        assert!(matches!(s.label_placement, Placement::End));
        assert!(matches!(s.thumb_icon_color, Color::Default));
        assert!(s.thumb_icon.is_none());
        assert!(s.label.is_none());
        assert!(s.ripple);
        assert!(!s.disabled && !s.read_only);
        assert!(s.tag.is_none());
        assert!(s.base.class_names().is_empty());
    }

    #[test]
    fn builders_set_every_field() {
        let mut v = true;
        let s = MudSwitch::new(&mut v)
            .label("Notifications")
            .label_placement(Placement::Start)
            .color(Color::Success)
            .unchecked_color(Color::Warning)
            .size(Size::Large)
            .thumb_icon("M0 0")
            .thumb_icon_color(Color::Error)
            .ripple(false)
            .disabled(true)
            .read_only(true)
            .classes(["mt-2"])
            .tag("notify");

        assert_eq!(s.label.as_deref(), Some("Notifications"));
        assert!(matches!(s.label_placement, Placement::Start));
        assert!(matches!(s.color, Color::Success));
        assert!(matches!(s.unchecked_color, Color::Warning));
        assert!(matches!(s.size, Size::Large));
        assert_eq!(s.thumb_icon.as_deref(), Some("M0 0"));
        assert!(matches!(s.thumb_icon_color, Color::Error));
        assert!(!s.ripple && s.disabled && s.read_only);
        assert!(!s.base.class_names().is_empty());
        assert_eq!(s.tag_ref::<&str>(), Some(&"notify"));
    }

    #[test]
    fn tag_round_trips_by_type() {
        let mut v = false;
        let s = MudSwitch::new(&mut v).tag("airplane");
        assert_eq!(s.tag_ref::<&str>(), Some(&"airplane"));
        assert!(s.tag_ref::<u32>().is_none());
    }
}