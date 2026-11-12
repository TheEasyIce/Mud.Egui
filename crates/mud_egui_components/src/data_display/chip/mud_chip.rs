//! `MudChip` — a compact, selectable, closable token.
//!
//! Port of `MudChip<T>.razor.cs` (MudBlazor 8.13.0).
//!
//! # MudBlazor parity
//!
//! | MudBlazor parameter | Rust method | Default |
//! |---------------------|-------------|---------|
//! | `CheckMark`         | [`MudChip::check_mark`] | `false` |
//! | `CheckedIcon`       | [`MudChip::checked_icon`] | [`filled::CHECK`] |
//! | `ChildContent`      | [`MudChip::new`] / [`MudChip::text`] | required |
//! | `CloseIcon`         | [`MudChip::close_icon`] | [`filled::CANCEL`] |
//! | `Color`             | [`MudChip::color`] | [`Color::Default`] |
//! | `Disabled`          | [`MudChip::disabled`] | `false` |
//! | `Icon`              | [`MudChip::icon`] | `None` |
//! | `IconColor`         | [`MudChip::icon_color`] | [`Color::Inherit`] |
//! | `Label`             | [`MudChip::label`] | `false` |
//! | `OnClick`           | [`MudChip::on_click`] | `None` |
//! | `OnClose`           | [`MudChip::on_closed`] | `None` |
//! | `Selected`          | [`MudChip::selected`] | `false` |
//! | `SelectedChanged`   | [`MudChip::on_selected_changed`] | `None` |
//! | `SelectedColor`     | [`MudChip::selected_color`] | `None` |
//! | `Size`              | [`MudChip::size`] | [`Size::Medium`] |
//! | `Variant`           | [`MudChip::variant`] | [`Variant::Filled`] |
//! | `Class` (base)      | [`MudChip::classes`] | `None` |
//! | `Id` (base)         | [`MudChip::id`] | `None` |
//! | `Style` (base)      | *(egui has no CSS strings; use `classes`)* | — |
//! | `UserAttributes`    | **not ported** (no HTML tag) | — |
//!
//! ### Extensions (not in MudBlazor)
//!
//! | Extension | Description |
//! |-----------|-------------|
//! | [`MudChip::value`] | A stable key used by `MudChipSet` for selection. |
//! | [`MudChip::closable`] | Explicit `closable` flag. |
//! | [`MudChip::elevation`] | Drop-shadow depth (0–24). |
//! | [`MudChip::aria_label`] | Accessible label. |
//!
//! ### `Class` resolution
//!
//! `MudChip` resolves its `Class` through `BsStyle` / `TwStyle`. Utility
//! classes override the chip's own metrics the way CSS utilities override
//! `.mud-chip` in MudBlazor:
//!
//! * `rounded-*` (`rounded-0`, `rounded-pill`, …) → replaces `metrics.radius`.
//! * `bg-*` (`bg-background`, `bg-primary`, `bg-transparent`, …) → replaces
//!   the computed fill for the current state.
//! * `border*` (`border`, `border-dashed-2`, …) → replaces the outlined stroke.
//! * `p-*` / `px-*` / `py-*` / `pt-*` / `pb-*` / `ps-*` / `pe-*` — replaces
//!   the chip's internal horizontal / vertical padding.
//! * `m-*` — outer margin (unchanged; applied via a wrapper frame).
//!
//! The chip paints its background/stroke/padding itself using the resolved
//! values (it cannot rely on `Frame::show`, which allocates only the content's
//! natural size and would put the fill around the chip instead of on it).
//!
//! # Text and icons
//!
//! The chip's label is rendered through [`MudText`]; the leading icon and the
//! close button are rendered through [`MudIcon`]. Text width is measured with
//! `Painter::layout_no_wrap` (the same font `MudText` will use), then painted
//! via `MudText`.
//!
//! # Interaction
//!
//! [`MudChip::show`] returns `(Response, bool)`:
//!
//! * **`Response`** — the body hit-rect's response.
//! * **`bool`** — `true` when the close button was clicked this frame.
//!
//! # Example
//!
//! ```ignore
//! ui.add(MudChip::new("Rust").icon(icons::CODE).closable(true));
//! ```

use egui::{
    vec2, Color32, CornerRadius, CursorIcon, Frame, Layout, Pos2, Rect, Response, Sense,
    Stroke, StrokeKind, Ui, UiBuilder, Widget,
};
use mud_egui_bootstrap::bs_style::BsStyle;
use mud_egui_core::base::component_base::ComponentBase;
use mud_egui_core::enums::{Color, Size, StyleFlavor, Variant};
use mud_egui_icons_material::filled;
use mud_egui_tailwind::tw_style::{TwState, TwStyle};
use mud_egui_theming::models::palettes::palette::Palette;
use mud_egui_theming::theme::MudTheme;

use crate::data_display::icons::mud_icon::MudIcon;
use crate::typography::mud_text::MudText;

// =============================================================================
// Constants
// =============================================================================

/// Base rem size passed to the Tailwind resolver.
const REM_SIZE_PT: f32 = 14.0;

/// Resting elevation for a filled chip.
const ELEVATION_REST: u8 = 1;

/// Hover elevation for a filled chip.
const ELEVATION_HOVER: u8 = 3;

/// Opacity applied to text and icons when the chip is disabled.
const DISABLED_ALPHA: f32 = 0.38;

/// Close-icon size relative to the chip's leading-icon size.
const CLOSE_ICON_SCALE: f32 = 0.85;

/// Minimum close-icon side length, in logical pixels.
const CLOSE_ICON_MIN: f32 = 12.0;

// =============================================================================
// Pure helpers
// =============================================================================

/// Applies an alpha multiplier to a colour.
#[inline]
pub fn with_alpha(c: Color32, alpha: f32) -> Color32 {
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

/// Resolves a semantic [`Color`] to a concrete [`Color32`].
#[inline]
pub fn resolve_color(c: Color, theme: &MudTheme, default: Color32) -> Color32 {
    let palette = theme.current_palette();
    match c {
        Color::Default | Color::Inherit => default,
        other => other.resolve(palette).unwrap_or(default),
    }
}

/// Applies the "selected" override to the resolved [`Variant`].
#[inline]
pub fn effective_variant(selected: bool, variant: Option<Variant>) -> Variant {
    let base = variant.unwrap_or(Variant::Filled);
    if !selected {
        return base;
    }
    match base {
        Variant::Text => Variant::Filled,
        Variant::Filled => Variant::Text,
        Variant::Outlined => Variant::Outlined,
    }
}

/// Resolves the chip's text colour for the given state and variant.
#[inline]
pub fn chip_text_color(
    disabled: bool,
    variant: Variant,
    accent: Color32,
    palette: &Palette,
) -> Color32 {
    if disabled {
        return with_alpha(palette.text_disabled.into(), DISABLED_ALPHA);
    }
    match variant {
        Variant::Filled => palette.primary_contrast_text.into(),
        Variant::Outlined | Variant::Text => accent,
    }
}

/// `true` when any class in `base` starts with `prefix`.
#[inline]
fn has_prefix(base: &ComponentBase, prefix: &str) -> bool {
    base.class_names().iter().any(|c| c.starts_with(prefix))
}

/// `true` when any padding class (`p-*`, `px-*`, `py-*`, `pt-*`, `pb-*`,
/// `ps-*`, `pe-*`) is present.
fn has_padding_class(base: &ComponentBase) -> bool {
    const PREFIXES: &[&str] = &[
        "p-", "px-", "py-", "pt-", "pb-", "ps-", "pe-", "pl-", "pr-",
    ];
    PREFIXES.iter().any(|p| has_prefix(base, p))
}

// =============================================================================
// ChipMetrics
// =============================================================================

/// Resolved pixel metrics for one chip.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ChipMetrics {
    /// Total chip height, in logical pixels.
    pub height: f32,
    /// Label font size.
    pub font_size: f32,
    /// Leading icon and checkbox side length.
    pub icon_size_px: f32,
    /// Horizontal padding on each side.
    pub padding_x: f32,
    /// Gap between icon, label, and close button.
    pub gap: f32,
    /// Corner radius. `4.0` when `label` is `true`.
    pub radius: f32,
}

impl ChipMetrics {
    /// Resolves the metrics for a [`Size`] and the `label` flag.
    pub fn resolve(size: Size, label: bool) -> Self {
        let (height, font_size, icon_size_px, padding_x, gap, pill_radius) = match size {
            Size::Small => (24.0, 12.0, 14.0, 8.0, 4.0, 12.0),
            Size::Large => (40.0, 16.0, 20.0, 16.0, 8.0, 20.0),
            _ => (32.0, 14.0, 18.0, 12.0, 6.0, 16.0),
        };
        Self {
            height,
            font_size,
            icon_size_px,
            padding_x,
            gap,
            radius: if label { 4.0 } else { pill_radius },
        }
    }

    /// Close-icon side length.
    #[inline]
    pub fn close_icon_size(&self) -> f32 {
        (self.icon_size_px * CLOSE_ICON_SCALE).max(CLOSE_ICON_MIN)
    }
}

// =============================================================================
// MudChip
// =============================================================================

/// A compact, selectable, closable token.
#[must_use = "MudChip does nothing until you call `.show(ui)` or `ui.add(..)`"]
pub struct MudChip {
    pub text: String,
    pub value: Option<String>,
    pub icon: Option<String>,
    pub checked_icon: Option<String>,
    pub close_icon: Option<String>,
    pub selected: bool,
    pub disabled: bool,
    pub closable: bool,
    pub show_check_mark: bool,
    pub label: Option<bool>,
    pub color: Option<Color>,
    pub icon_color: Option<Color>,
    pub selected_color: Option<Color>,
    pub variant: Option<Variant>,
    pub size: Option<Size>,
    pub elevation: Option<u8>,
    pub id: Option<String>,
    pub aria_label: Option<String>,

    pub base: ComponentBase,
    pub flavor: StyleFlavor,
    pub bs_style: BsStyle,
    pub tw_style: TwStyle,
    pub tw_state: TwState,

    pub on_click: Option<Box<dyn FnMut()>>,
    pub on_closed: Option<Box<dyn FnMut()>>,
    pub on_selected_changed: Option<Box<dyn FnMut(bool)>>,
}

impl Default for MudChip {
    fn default() -> Self {
        Self {
            text: String::new(),
            value: None,
            icon: None,
            checked_icon: None,
            close_icon: None,
            selected: false,
            disabled: false,
            closable: false,
            show_check_mark: false,
            label: None,
            color: None,
            icon_color: None,
            selected_color: None,
            variant: None,
            size: None,
            elevation: None,
            id: None,
            aria_label: None,
            base: ComponentBase::new(),
            flavor: StyleFlavor::default(),
            bs_style: BsStyle::default(),
            tw_style: TwStyle::default(),
            tw_state: TwState::default(),
            on_click: None,
            on_closed: None,
            on_selected_changed: None,
        }
    }
}

impl MudChip {
    /// Creates a chip with the given label and MudBlazor defaults.
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into(), ..Default::default() }
    }

    // ── Builder methods ────────────────────────────────────────────────────

    #[inline] pub fn text(mut self, s: impl Into<String>) -> Self { self.text = s.into(); self }
    #[inline] pub fn value(mut self, v: impl Into<String>) -> Self { self.value = Some(v.into()); self }
    #[inline] pub fn selected(mut self, b: bool) -> Self { self.selected = b; self }
    #[inline] pub fn disabled(mut self, b: bool) -> Self { self.disabled = b; self }
    #[inline] pub fn color(mut self, c: Color) -> Self { self.color = Some(c); self }
    #[inline] pub fn icon_color(mut self, c: Color) -> Self { self.icon_color = Some(c); self }
    #[inline] pub fn selected_color(mut self, c: Color) -> Self { self.selected_color = Some(c); self }
    #[inline] pub fn variant(mut self, v: Variant) -> Self { self.variant = Some(v); self }
    #[inline] pub fn size(mut self, s: Size) -> Self { self.size = Some(s); self }
    #[inline] pub fn label(mut self, b: bool) -> Self { self.label = Some(b); self }
    #[inline] pub fn icon(mut self, i: impl Into<String>) -> Self { self.icon = Some(i.into()); self }
    #[inline] pub fn checked_icon(mut self, i: impl Into<String>) -> Self { self.checked_icon = Some(i.into()); self }
    #[inline] pub fn close_icon(mut self, i: impl Into<String>) -> Self { self.close_icon = Some(i.into()); self }
    #[inline] pub fn check_mark(mut self, b: bool) -> Self { self.show_check_mark = b; self }
    #[inline] pub fn closable(mut self, b: bool) -> Self { self.closable = b; self }
    #[inline] pub fn elevation(mut self, e: u8) -> Self { self.elevation = Some(e); self }
    #[inline] pub fn id(mut self, id: impl Into<String>) -> Self { self.id = Some(id.into()); self }
    #[inline] pub fn aria_label(mut self, label: impl Into<String>) -> Self { self.aria_label = Some(label.into()); self }
    #[inline] pub fn classes<S: AsRef<str>>(mut self, classes: impl IntoIterator<Item = S>) -> Self { self.base = self.base.classes(classes); self }
    #[inline] pub fn flavor(mut self, f: StyleFlavor) -> Self { self.flavor = f; self }
    #[inline] pub fn on_click(mut self, f: impl FnMut() + 'static) -> Self { self.on_click = Some(Box::new(f)); self }
    #[inline] pub fn on_closed(mut self, f: impl FnMut() + 'static) -> Self { self.on_closed = Some(Box::new(f)); self }
    #[inline] pub fn on_selected_changed(mut self, f: impl FnMut(bool) + 'static) -> Self { self.on_selected_changed = Some(Box::new(f)); self }

    // ── Rendering ──────────────────────────────────────────────────────────

    /// Draws the chip.
    pub fn show(&mut self, ui: &mut Ui) -> (Response, bool) {
        let theme = MudTheme::fetch(ui.ctx()).unwrap_or_default();
        let palette = theme.current_palette();

        let size = self.size.unwrap_or(Size::Medium);
        let label = self.label.unwrap_or(false);
        let metrics = ChipMetrics::resolve(size, label);

        // 1. Resolve `self.base` into a Frame.
        let lookup = |key: &str| palette.resolve(key).map(Color32::from);
        self.tw_state.dark = ui.visuals().dark_mode;
        let frame = match self.flavor {
            StyleFlavor::Bootstrap => self.bs_style.build_frame(&self.base, &lookup),
            StyleFlavor::Tailwind => self
                .tw_style
                .build_frame(&self.base, self.tw_state, REM_SIZE_PT, &lookup),
        };

        // 2. Extract class-driven overrides.
        let has_rounded = has_prefix(&self.base, "rounded-");
        let has_bg = has_prefix(&self.base, "bg-");
        let has_border = has_prefix(&self.base, "border");
        let has_padding = has_padding_class(&self.base);

        let resolved_radius = if has_rounded {
            frame.corner_radius
        } else {
            CornerRadius::same(metrics.radius.max(0.0) as u8)
        };
        let resolved_pad_l = if has_padding { frame.inner_margin.left as f32 } else { metrics.padding_x };
        let resolved_pad_r = if has_padding { frame.inner_margin.right as f32 } else { metrics.padding_x };
        let resolved_pad_t = if has_padding { frame.inner_margin.top as f32 } else { 0.0 };
        let resolved_pad_b = if has_padding { frame.inner_margin.bottom as f32 } else { 0.0 };
        let resolved_height = metrics.height + resolved_pad_t + resolved_pad_b;

        // 3. Colours (chip's own computed; `bg-*` overrides them below).
        let base_color = self.color.unwrap_or(Color::Default);
        let accent = resolve_color(base_color, &theme, palette.primary.into());
        let effective_accent = if self.selected {
            self.selected_color
                .map(|c| resolve_color(c, &theme, accent))
                .unwrap_or(accent)
        } else {
            accent
        };

        let variant = effective_variant(self.selected, self.variant);
        let text_color = chip_text_color(self.disabled, variant, effective_accent, palette);

        let mut close_clicked = false;
        let mut body_clicked = false;

        let base_id = match self.id.as_deref() {
            Some(id) => ui.scope_id().with(id),
            None => ui.scope_id().with(&self.text).with(accent.to_array()),
        };

        // 4. Minimal wrapper: only `m-*` outer margin. Fill / stroke /
        //    corner / padding are extracted above and applied by the chip's
        //    own paint path.
        let wrapper = Frame::new().outer_margin(frame.outer_margin);

        let inner = wrapper.show(ui, |ui| {
            // ── Measurement ──────────────────────────────────────────────
            let show_check = self.selected && (self.show_check_mark || self.checked_icon.is_some());
            let current_icon = if show_check {
                self.checked_icon.as_deref().unwrap_or(filled::CHECK)
            } else {
                self.icon.as_deref().unwrap_or("")
            };
            let has_leading_icon = !current_icon.is_empty();
            let has_close = self.closable;

            let font = egui::FontId::proportional(metrics.font_size);
            let text_width = ui
                .painter()
                .layout_no_wrap(self.text.clone(), font, text_color)
                .size()
                .x;

            let mut content_width = resolved_pad_l + resolved_pad_r + text_width;
            if has_leading_icon {
                content_width += metrics.icon_size_px + metrics.gap;
            }
            if has_close {
                content_width += metrics.gap + metrics.close_icon_size();
            }

            let desired = vec2(content_width, resolved_height);

            // ── Allocate + interact ──────────────────────────────────────
            let sense = if self.disabled { Sense::hover() } else { Sense::click() };
            let (rect, _) = ui.allocate_exact_size(desired, sense);

            let body_id = base_id.with("body");
            let body_resp = ui.interact(rect, body_id, sense);

            let hovered = body_resp.hovered() && !self.disabled;
            let has_focus = body_resp.has_focus();

            // ── Fill ─────────────────────────────────────────────────────
            let computed_bg = match variant {
                Variant::Filled => {
                    if self.disabled {
                        with_alpha(palette.action_disabled.into(), 0.12)
                    } else if hovered {
                        with_alpha(effective_accent, 0.92)
                    } else {
                        effective_accent
                    }
                }
                Variant::Outlined | Variant::Text => {
                    if hovered && !self.disabled {
                        with_alpha(effective_accent, 0.08)
                    } else {
                        Color32::TRANSPARENT
                    }
                }
            };
            // `bg-*` on `self.base` overrides the fill entirely.
            let bg = if has_bg { frame.fill } else { computed_bg };

            // ── Stroke ───────────────────────────────────────────────────
            let computed_stroke = if variant == Variant::Outlined {
                let stroke_color = if self.disabled {
                    with_alpha(effective_accent, DISABLED_ALPHA)
                } else {
                    effective_accent
                };
                Some(Stroke::new(1.0, stroke_color))
            } else {
                None
            };
            // `border*` on `self.base` overrides the stroke.
            let stroke = if has_border && frame.stroke != Stroke::NONE {
                Some(frame.stroke)
            } else {
                computed_stroke
            };

            // ── Paint fill / stroke ──────────────────────────────────────
            ui.painter().rect_filled(rect, resolved_radius, bg);
            if let Some(s) = stroke {
                ui.painter()
                    .rect_stroke(rect, resolved_radius, s, StrokeKind::Inside);
            }

            // ── Elevation shadow ─────────────────────────────────────────
            let elev = self.elevation.unwrap_or_else(|| {
                if variant == Variant::Filled && !self.disabled {
                    if hovered { ELEVATION_HOVER } else { ELEVATION_REST }
                } else {
                    0
                }
            });
            if elev > 0 && !self.disabled {
                let shadow_color = Color32::from_black_alpha((12 + elev * 4).min(40));
                let shadow_rect = rect.translate(vec2(0.0, elev as f32 * 0.4));
                // Fill under-fill first, so the shadow shows around the chip;
                // then re-paint the fill / stroke over the shadow.
                ui.painter().rect_filled(shadow_rect, resolved_radius, shadow_color);
                ui.painter().rect_filled(rect, resolved_radius, bg);
                if let Some(s) = stroke {
                    ui.painter()
                        .rect_stroke(rect, resolved_radius, s, StrokeKind::Inside);
                }
            }

            // ── Focus ring ───────────────────────────────────────────────
            if has_focus && !self.disabled {
                let focus_color = with_alpha(effective_accent, 0.4);
                ui.painter().rect_stroke(
                    rect.expand(2.0),
                    CornerRadius::same((resolved_radius.nw as f32 + 2.0) as u8),
                    Stroke::new(2.0, focus_color),
                    StrokeKind::Outside,
                );
            }

            // ── Leading icon ─────────────────────────────────────────────
            let mut cursor_x = rect.min.x + resolved_pad_l;
            let center_y = rect.center().y;

            if has_leading_icon {
                let icon_color = if self.disabled {
                    with_alpha(palette.text_disabled.into(), DISABLED_ALPHA)
                } else {
                    self.icon_color
                        .map(|c| resolve_color(c, &theme, text_color))
                        .unwrap_or(text_color)
                };
                let icon_rect = Rect::from_center_size(
                    Pos2::new(cursor_x + metrics.icon_size_px * 0.5, center_y),
                    vec2(metrics.icon_size_px, metrics.icon_size_px),
                );
                ui.put(
                    icon_rect,
                    MudIcon::new(current_icon.to_owned())
                        .size_px(metrics.icon_size_px)
                        .custom_color(icon_color),
                );
                cursor_x += metrics.icon_size_px + metrics.gap;
            }

            // ── Label ────────────────────────────────────────────────────
            let text_rect = Rect::from_min_size(
                Pos2::new(cursor_x, center_y - metrics.font_size),
                vec2(text_width + 4.0, metrics.font_size * 2.0),
            );
            let mut text_ui = ui.new_child(
                UiBuilder::new()
                    .max_rect(text_rect)
                    .layout(Layout::left_to_right(egui::Align::Center)),
            );
            text_ui.add(
                MudText::new(self.text.clone())
                    .font_size(metrics.font_size)
                    .custom_color(text_color)
                    .inline(true)
                    .no_wrap(true),
            );
            cursor_x += text_width;

            // ── Close button ─────────────────────────────────────────────
            if has_close {
                cursor_x += metrics.gap;
                let close_icon = self.close_icon.as_deref().unwrap_or(filled::CANCEL);
                let close_size = metrics.close_icon_size();
                let close_rect = Rect::from_center_size(
                    Pos2::new(cursor_x + close_size * 0.5, center_y),
                    vec2(close_size, close_size),
                );
                let close_sense = if self.disabled { Sense::hover() } else { Sense::click() };
                let close_resp = ui.interact(close_rect, base_id.with("close"), close_sense);
                if close_resp.clicked() {
                    close_clicked = true;
                }
                let close_color = if self.disabled {
                    with_alpha(palette.text_disabled.into(), DISABLED_ALPHA)
                } else if close_resp.hovered() {
                    text_color
                } else {
                    with_alpha(text_color, 0.7)
                };
                ui.put(
                    close_rect,
                    MudIcon::new(close_icon.to_owned())
                        .size_px(close_size)
                        .custom_color(close_color),
                );
            }

            // ── Cursor + click ───────────────────────────────────────────
            if !self.disabled && (body_resp.hovered() || close_clicked) {
                ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
            }
            if body_resp.clicked() && !close_clicked {
                body_clicked = true;
            }

            body_resp
        });

        // ── Dispatch callbacks ────────────────────────────────────────────
        if body_clicked && !self.disabled {
            if let Some(cb) = self.on_click.as_mut() {
                cb();
            }
            let prev = self.selected;
            self.selected = !self.selected;
            if prev != self.selected {
                if let Some(cb) = self.on_selected_changed.as_mut() {
                    cb(self.selected);
                }
            }
        }
        if close_clicked {
            if let Some(cb) = self.on_closed.as_mut() {
                cb();
            }
        }

        // ── Accessibility ─────────────────────────────────────────────────
        let label: &str = self.aria_label.as_deref().unwrap_or(self.text.as_str());
        let enabled = !self.disabled;
        inner.inner.clone().widget_info(|| {
            egui::WidgetInfo::labeled(egui::Role::Button, enabled, label)
        });

        (inner.inner, close_clicked)
    }
}

impl Widget for MudChip {
    fn ui(mut self, ui: &mut Ui) -> Response {
        let (resp, _) = self.show(ui);
        resp
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ── with_alpha ─────────────────────────────────────────────────────────

    #[test]
    fn with_alpha_full_returns_same_color() {
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

    // ── resolve_color ──────────────────────────────────────────────────────

    #[test]
    fn resolve_color_default_uses_fallback() {
        let theme = MudTheme::default();
        let fallback = Color32::from_rgb(1, 2, 3);
        assert_eq!(resolve_color(Color::Default, &theme, fallback), fallback);
    }

    #[test]
    fn resolve_color_inherit_uses_fallback() {
        let theme = MudTheme::default();
        let fallback = Color32::from_rgb(1, 2, 3);
        assert_eq!(resolve_color(Color::Inherit, &theme, fallback), fallback);
    }

    #[test]
    fn resolve_color_primary_maps_to_palette() {
        let theme = MudTheme::default();
        let palette = theme.current_palette();
        let primary: Color32 = palette.primary.into();
        let fallback = Color32::from_rgb(99, 0, 0);
        assert_eq!(resolve_color(Color::Primary, &theme, fallback), primary);
    }

    // ── effective_variant ──────────────────────────────────────────────────

    #[test]
    fn effective_variant_unselected_returns_base() {
        assert_eq!(effective_variant(false, None), Variant::Filled);
        assert_eq!(effective_variant(false, Some(Variant::Text)), Variant::Text);
        assert_eq!(effective_variant(false, Some(Variant::Outlined)), Variant::Outlined);
    }

    #[test]
    fn effective_variant_selected_swaps_filled_and_text() {
        assert_eq!(effective_variant(true, Some(Variant::Filled)), Variant::Text);
        assert_eq!(effective_variant(true, Some(Variant::Text)), Variant::Filled);
    }

    #[test]
    fn effective_variant_selected_keeps_outlined() {
        assert_eq!(effective_variant(true, Some(Variant::Outlined)), Variant::Outlined);
    }

    #[test]
    fn effective_variant_selected_none_defaults_to_text() {
        assert_eq!(effective_variant(true, None), Variant::Text);
    }

    // ── chip_text_color ────────────────────────────────────────────────────

    #[test]
    fn chip_text_color_disabled_uses_disabled_alpha() {
        let theme = MudTheme::default();
        let palette = theme.current_palette();
        let disabled: Color32 = palette.text_disabled.into();
        let expected = with_alpha(disabled, DISABLED_ALPHA);
        let got = chip_text_color(true, Variant::Filled, Color32::RED, palette);
        assert_eq!(got, expected);
    }

    #[test]
    fn chip_text_color_filled_uses_contrast() {
        let theme = MudTheme::default();
        let palette = theme.current_palette();
        let contrast: Color32 = palette.primary_contrast_text.into();
        let got = chip_text_color(false, Variant::Filled, Color32::RED, palette);
        assert_eq!(got, contrast);
    }

    #[test]
    fn chip_text_color_outlined_uses_accent() {
        let theme = MudTheme::default();
        let palette = theme.current_palette();
        let accent = Color32::from_rgb(200, 10, 10);
        assert_eq!(chip_text_color(false, Variant::Outlined, accent, palette), accent);
        assert_eq!(chip_text_color(false, Variant::Text, accent, palette), accent);
    }

    // ── ChipMetrics ────────────────────────────────────────────────────────

    #[test]
    fn metrics_small() {
        let m = ChipMetrics::resolve(Size::Small, false);
        assert_eq!(m.height, 24.0);
        assert_eq!(m.font_size, 12.0);
        assert_eq!(m.icon_size_px, 14.0);
        assert_eq!(m.padding_x, 8.0);
        assert_eq!(m.gap, 4.0);
        assert_eq!(m.radius, 12.0);
    }

    #[test]
    fn metrics_medium() {
        let m = ChipMetrics::resolve(Size::Medium, false);
        assert_eq!(m.height, 32.0);
        assert_eq!(m.font_size, 14.0);
        assert_eq!(m.icon_size_px, 18.0);
        assert_eq!(m.padding_x, 12.0);
        assert_eq!(m.gap, 6.0);
        assert_eq!(m.radius, 16.0);
    }

    #[test]
    fn metrics_large() {
        let m = ChipMetrics::resolve(Size::Large, false);
        assert_eq!(m.height, 40.0);
        assert_eq!(m.font_size, 16.0);
        assert_eq!(m.icon_size_px, 20.0);
        assert_eq!(m.padding_x, 16.0);
        assert_eq!(m.gap, 8.0);
        assert_eq!(m.radius, 20.0);
    }

    #[test]
    fn metrics_label_forces_square_radius() {
        for size in [Size::Small, Size::Medium, Size::Large] {
            let pill = ChipMetrics::resolve(size, false);
            let squared = ChipMetrics::resolve(size, true);
            assert_eq!(squared.radius, 4.0, "{size:?}");
            assert_ne!(squared.radius, pill.radius, "{size:?}");
        }
    }

    #[test]
    fn metrics_unknown_size_defaults_to_medium() {
        let m = ChipMetrics::resolve(Size::ExtraSmall, false);
        assert_eq!(m.height, 32.0);
        let m = ChipMetrics::resolve(Size::ExtraLarge, false);
        assert_eq!(m.height, 32.0);
    }

    #[test]
    fn close_icon_size_scales_and_clamps() {
        let small = ChipMetrics::resolve(Size::Small, false);
        assert_eq!(small.close_icon_size(), CLOSE_ICON_MIN);
        let medium = ChipMetrics::resolve(Size::Medium, false);
        assert!((medium.close_icon_size() - 15.3).abs() < 1e-4);
        let large = ChipMetrics::resolve(Size::Large, false);
        assert!((large.close_icon_size() - 17.0).abs() < 1e-4);
    }

    // ── Class-prefix helpers ───────────────────────────────────────────────

    #[test]
    fn has_prefix_detects_rounded_classes() {
        let base = ComponentBase::new().classes(["rounded-0"]);
        assert!(has_prefix(&base, "rounded-"));
        assert!(!has_prefix(&base, "bg-"));
    }

    #[test]
    fn has_padding_class_detects_every_padding_prefix() {
        for c in ["p-2", "px-2", "py-2", "pt-2", "pb-2", "ps-2", "pe-2", "pl-2", "pr-2"] {
            let base = ComponentBase::new().classes([c]);
            assert!(has_padding_class(&base), "{c}");
        }
        let base = ComponentBase::new().classes(["m-2"]);
        assert!(!has_padding_class(&base));
    }

    // ── Defaults / builders ────────────────────────────────────────────────

    #[test]
    fn defaults_match_mudblazor() {
        let c = MudChip::new("x");
        assert!(!c.selected);
        assert!(!c.disabled);
        assert!(!c.closable);
        assert!(!c.show_check_mark);
        assert!(c.label.is_none());
        assert!(c.color.is_none());
        assert!(c.icon_color.is_none());
        assert!(c.selected_color.is_none());
        assert!(c.variant.is_none());
        assert!(c.size.is_none());
        assert!(c.elevation.is_none());
        assert!(c.id.is_none());
        assert!(c.aria_label.is_none());
        assert!(c.on_click.is_none());
        assert!(c.on_closed.is_none());
        assert!(c.on_selected_changed.is_none());
    }

    #[test]
    fn new_sets_text() {
        let c = MudChip::new("Rust");
        assert_eq!(c.text, "Rust");
    }

    #[test]
    fn builders_set_every_field() {
        let c = MudChip::new("Start")
            .text("Go")
            .value("go")
            .selected(true)
            .disabled(true)
            .closable(true)
            .check_mark(true)
            .label(true)
            .color(Color::Error)
            .icon_color(Color::Success)
            .selected_color(Color::Warning)
            .variant(Variant::Outlined)
            .size(Size::Large)
            .icon("M0 0")
            .checked_icon("C")
            .close_icon("X")
            .elevation(4)
            .id("my-chip")
            .aria_label("Chip")
            .classes(["m-1"])
            .on_click(|| {})
            .on_closed(|| {})
            .on_selected_changed(|_| {});

        assert_eq!(c.text, "Go");
        assert_eq!(c.value.as_deref(), Some("go"));
        assert!(c.selected && c.disabled && c.closable && c.show_check_mark);
        assert_eq!(c.label, Some(true));
        assert_eq!(c.color, Some(Color::Error));
        assert_eq!(c.icon_color, Some(Color::Success));
        assert_eq!(c.selected_color, Some(Color::Warning));
        assert_eq!(c.variant, Some(Variant::Outlined));
        assert_eq!(c.size, Some(Size::Large));
        assert_eq!(c.icon.as_deref(), Some("M0 0"));
        assert_eq!(c.checked_icon.as_deref(), Some("C"));
        assert_eq!(c.close_icon.as_deref(), Some("X"));
        assert_eq!(c.elevation, Some(4));
        assert_eq!(c.id.as_deref(), Some("my-chip"));
        assert_eq!(c.aria_label.as_deref(), Some("Chip"));
        assert!(c.base.has_class("m-1"));
        assert!(c.on_click.is_some());
        assert!(c.on_closed.is_some());
        assert!(c.on_selected_changed.is_some());
    }

    #[test]
    fn flavor_builder_sets_field() {
        let c = MudChip::new("x").flavor(StyleFlavor::Tailwind);
        assert_eq!(c.flavor, StyleFlavor::Tailwind);
    }
}