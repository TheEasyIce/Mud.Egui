//! A high-performance themed button.
//!
//! Port of `MudButton.razor.cs` (MudBlazor 8.13.0). Handles variants (Text,
//! Outlined, Filled), sizes, start/end icons, disabled states, hover/pressed
//! feedback, focus ring, drop shadow and semantic colouring based on the
//! active theme.
//!
//! # Design
//!
//! The label is measured once with `layout_no_wrap` (so the button size can be
//! decided before the content is placed) and then **rendered through
//! [`MudText`]** in a child `Ui` sized to the measurement. That keeps Rule 3 —
//! every visible string goes through `MudText` — while the geometry stays
//! deterministic. `MudText::button(..).uppercase(false)` is used because the
//! transform is already applied once by [`display_label`].
//!
//! Icons go through [`MudIcon`] (`SvgPath` or `Glyph`, auto-detected).
//!
//! The widget is **self-painted**: it measures its content, allocates exactly
//! one rectangle through `Ui::allocate_exact_size`, and paints background,
//! border, shadow, focus ring and icons itself.
//!
//! All decision logic (colours, interaction overlays, layout planning) lives
//! in small pure functions that never touch `&Ui`, so it is unit tested
//! without a live `egui::Context`.
//!
//! # Example
//!
//! ```ignore
//! let response = MudButton::new("Save")
//!     .variant(Variant::Filled)
//!     .color(Color::Primary)
//!     .size(Size::Large)
//!     .start_icon("★")
//!     .show(ui);
//! ```

use egui::{
    pos2, Align, Color32, CornerRadius, FontId, Layout, Rect, Response, Sense, Stroke,
    StrokeKind, Ui, UiBuilder, Vec2, Widget, WidgetInfo, WidgetType,
};
use mud_egui_core::enums::color::Color;
use mud_egui_core::enums::size::Size;
use mud_egui_core::enums::variant::Variant;
use mud_egui_core::enums::{IconKind, Typo};
use mud_egui_core::base::component_base::ComponentBase;
use mud_egui_theming::theme::{to_egui_family, MudTheme};

use crate::data_display::icons::mud_icon::MudIcon;
use crate::typography::mud_text::MudText;

const DISABLED_OPACITY: f32 = 0.38;
const DISABLED_FILL_OPACITY: f32 = 0.12;
const HOVER_OVERLAY: f32 = 0.08;
const PRESSED_OVERLAY: f32 = 0.16;
const CONTRAST_THRESHOLD: u32 = 384;
const OUTLINE_WIDTH: f32 = 1.0;
const FOCUS_WIDTH: f32 = 2.0;
const FOCUS_OFFSET: f32 = 2.0;
const ICON_SIZES_PX: [f32; 5] = [8.0, 16.0, 24.0, 32.0, 48.0];
const ICON_FALLBACK_PX: f32 = 24.0;

#[inline]
fn corner_radius() -> CornerRadius {
    CornerRadius::same(4)
}

#[inline]
fn focus_radius() -> CornerRadius {
    CornerRadius::same(6)
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct SizeMetrics {
    pad_x: f32,
    pad_y: f32,
    min_height: f32,
    font_size: f32,
    icon_gap: f32,
    icon_bucket: u8,
}

#[inline]
fn size_to_metrics(size: &Size) -> SizeMetrics {
    let (pad_x, pad_y, min_height, font_size, icon_gap, icon_bucket) = match size {
        Size::ExtraSmall => (6.0, 2.0, 22.0, 11.0, 4.0, 1),
        Size::Small => (10.0, 4.0, 30.0, 12.0, 6.0, 1),
        Size::Medium => (16.0, 6.0, 36.0, 14.0, 8.0, 1),
        Size::Large => (22.0, 8.0, 42.0, 15.0, 8.0, 2),
        Size::ExtraLarge => (28.0, 10.0, 52.0, 17.0, 10.0, 2),
    };
    SizeMetrics { pad_x, pad_y, min_height, font_size, icon_gap, icon_bucket }
}

#[inline]
fn icon_extent(bucket: u8) -> f32 {
    ICON_SIZES_PX.get(bucket as usize).copied().unwrap_or(ICON_FALLBACK_PX)
}

fn looks_like_svg_path(s: &str) -> bool {
    let mut chars = s.trim_start().chars();
    matches!(chars.next(), Some('M' | 'm'))
        && matches!(
            chars.find(|c| !c.is_whitespace()),
            Some(c) if c.is_ascii_digit() || c == '-' || c == '+' || c == '.'
        )
}

#[inline]
fn resolve_uppercase(explicit: Option<bool>, typo_default: bool) -> bool {
    explicit.unwrap_or(typo_default)
}

#[inline]
fn display_label(text: String, uppercase: bool) -> String {
    if uppercase { text.to_uppercase() } else { text }
}

#[inline]
fn icon_kind_for(icon: &str) -> IconKind {
    if looks_like_svg_path(icon) { IconKind::SvgPath } else { IconKind::Glyph }
}

#[inline]
fn sanitize(v: f32) -> f32 {
    if v.is_finite() && v > 0.0 { v } else { 0.0 }
}

#[inline]
fn contrast_color(bg: Color32) -> Color32 {
    let sum = bg.r() as u32 + bg.g() as u32 + bg.b() as u32;
    if sum > CONTRAST_THRESHOLD { Color32::BLACK } else { Color32::WHITE }
}

#[inline]
fn pick_base(is_inherit: bool, inherit_color: Color32, resolved: Option<Color32>) -> Color32 {
    if is_inherit { inherit_color } else { resolved.unwrap_or(Color32::WHITE) }
}

#[inline]
fn mix_color(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    let lerp = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round() as u8;
    Color32::from_rgba_premultiplied(
        lerp(a.r(), b.r()),
        lerp(a.g(), b.g()),
        lerp(a.b(), b.b()),
        lerp(a.a(), b.a()),
    )
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct ButtonColors {
    text: Color32,
    fill: Color32,
    border: Color32,
}

fn resolve_colors(
    variant: &Variant,
    disabled: bool,
    base: Color32,
    contrast: Color32,
) -> ButtonColors {
    if disabled {
        let faded = base.gamma_multiply(DISABLED_OPACITY);
        return match variant {
            Variant::Filled => ButtonColors {
                text: faded,
                fill: base.gamma_multiply(DISABLED_FILL_OPACITY),
                border: Color32::TRANSPARENT,
            },
            Variant::Outlined => ButtonColors { text: faded, fill: Color32::TRANSPARENT, border: faded },
            Variant::Text => ButtonColors { text: faded, fill: Color32::TRANSPARENT, border: Color32::TRANSPARENT },
        };
    }
    match variant {
        Variant::Filled => ButtonColors { text: contrast, fill: base, border: Color32::TRANSPARENT },
        Variant::Outlined => ButtonColors { text: base, fill: Color32::TRANSPARENT, border: base },
        Variant::Text => ButtonColors { text: base, fill: Color32::TRANSPARENT, border: Color32::TRANSPARENT },
    }
}

#[inline]
fn icon_color_for(override_color: Option<Color32>, text_color: Color32, disabled: bool) -> Color32 {
    match override_color {
        Some(c) if disabled => c.gamma_multiply(DISABLED_OPACITY),
        Some(c) => c,
        None => text_color,
    }
}

#[inline]
fn interaction_strength(hovered: bool, pressed: bool) -> f32 {
    if pressed { PRESSED_OVERLAY } else if hovered { HOVER_OVERLAY } else { 0.0 }
}

fn state_fill(
    variant: &Variant,
    colors: &ButtonColors,
    base: Color32,
    contrast: Color32,
    strength: f32,
) -> Color32 {
    if strength <= 0.0 { return colors.fill; }
    match variant {
        Variant::Filled => mix_color(colors.fill, contrast, strength),
        Variant::Outlined | Variant::Text => base.gamma_multiply(strength),
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct LayoutPlan {
    size: Vec2,
    start_x: Option<f32>,
    text_x: f32,
    end_x: Option<f32>,
}

fn plan_layout(
    m: &SizeMetrics,
    start_w: Option<f32>,
    text_w: f32,
    content_h: f32,
    end_w: Option<f32>,
    available_w: f32,
    full_width: bool,
) -> LayoutPlan {
    let start_w = start_w.map(sanitize);
    let end_w = end_w.map(sanitize);
    let text_w = sanitize(text_w);
    let content_h = sanitize(content_h);
    let has_text = text_w > 0.0;

    let items = start_w.is_some() as u32 + has_text as u32 + end_w.is_some() as u32;
    let gaps = m.icon_gap * items.saturating_sub(1) as f32;
    let content_w = start_w.unwrap_or(0.0)
        + if has_text { text_w } else { 0.0 }
        + end_w.unwrap_or(0.0)
        + gaps;

    let natural_w = content_w + 2.0 * m.pad_x;
    let width = if full_width && available_w.is_finite() {
        natural_w.max(available_w)
    } else {
        natural_w
    };
    let height = (content_h + 2.0 * m.pad_y).max(m.min_height);

    let mut cursor = (width - content_w) * 0.5;
    let mut start_x = None;
    if let Some(w) = start_w {
        start_x = Some(cursor);
        cursor += w + m.icon_gap;
    }
    let text_x = cursor;
    if has_text { cursor += text_w + m.icon_gap; }
    let end_x = end_w.map(|_| cursor);

    LayoutPlan { size: Vec2::new(width, height), start_x, text_x, end_x }
}

fn paint_icon(ui: &mut Ui, rect: Rect, icon: &str, color: Color32, bucket: u8) {
    let mut child = ui.new_child(
        UiBuilder::new().max_rect(rect).layout(Layout::left_to_right(Align::Center)),
    );
    let _ = MudIcon::new(icon)
        .kind(icon_kind_for(icon))
        .size(bucket)
        .custom_color(color)
        .show(&mut child);
}

#[must_use = "MudButton does nothing until you call `.show(ui)`"]
#[derive(Debug, Clone)]
pub struct MudButton {
    pub text: String,
    pub variant: Variant,
    pub color: Color,
    pub size: Size,
    pub disabled: bool,
    pub full_width: bool,
    pub drop_shadow: bool,
    pub start_icon: Option<String>,
    pub end_icon: Option<String>,
    pub icon_color: Option<Color>,
    pub uppercase: Option<bool>,
    pub icon_size: Option<u8>,
    pub base: ComponentBase,
}

impl Default for MudButton {
    fn default() -> Self {
        Self {
            text: String::new(),
            variant: Variant::Text,
            color: Color::Primary,
            size: Size::Medium,
            disabled: false,
            full_width: false,
            drop_shadow: true,
            start_icon: None,
            end_icon: None,
            icon_color: None,
            uppercase: None,
            icon_size: None,
            base: ComponentBase::new(),
        }
    }
}

impl MudButton {
    #[inline]
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into(), ..Default::default() }
    }

    #[inline] pub fn variant(mut self, variant: Variant) -> Self { self.variant = variant; self }
    #[inline] pub fn color(mut self, color: Color) -> Self { self.color = color; self }
    #[inline] pub fn size(mut self, size: Size) -> Self { self.size = size; self }
    #[inline] pub fn disabled(mut self, disabled: bool) -> Self { self.disabled = disabled; self }
    #[inline] pub fn full_width(mut self, full_width: bool) -> Self { self.full_width = full_width; self }
    #[inline] pub fn drop_shadow(mut self, drop_shadow: bool) -> Self { self.drop_shadow = drop_shadow; self }
    #[inline] pub fn start_icon(mut self, icon: impl Into<String>) -> Self { self.start_icon = Some(icon.into()); self }
    #[inline] pub fn end_icon(mut self, icon: impl Into<String>) -> Self { self.end_icon = Some(icon.into()); self }
    #[inline] pub fn icon_color(mut self, color: Color) -> Self { self.icon_color = Some(color); self }
    #[inline] pub fn uppercase(mut self, uppercase: bool) -> Self { self.uppercase = Some(uppercase); self }
    #[inline] pub fn icon_size(mut self, bucket: u8) -> Self { self.icon_size = Some(bucket); self }
    #[inline]
    pub fn classes<S: AsRef<str>>(mut self, classes: impl IntoIterator<Item = S>) -> Self {
        self.base = self.base.classes(classes);
        self
    }

    pub fn show(mut self, ui: &mut Ui) -> Response {
        let theme = MudTheme::fetch(ui.ctx()).unwrap_or_default();
        let palette = theme.current_palette();

        let metrics = size_to_metrics(&self.size);
        let disabled = self.disabled || !ui.is_enabled();

        // 1. Resolve colours.
        let is_inherit = self.color == Color::Inherit;
        let resolved = if is_inherit { None } else { self.color.resolve(palette) };
        let base = pick_base(is_inherit, ui.visuals().text_color(), resolved);
        let contrast = contrast_color(base);
        let colors = resolve_colors(&self.variant, disabled, base, contrast);

        let icon_override = if let Some(ic) = self.icon_color {
            Some(ic.resolve(palette).unwrap_or(colors.text))
        } else {
            None
        };
        let icon_color = icon_color_for(icon_override, colors.text, disabled);

        // 2. Measure the label once.
        let has_text = !self.text.is_empty();
        let upper = resolve_uppercase(self.uppercase, Typo::Button.is_uppercase_by_default());
        let text = display_label(std::mem::take(&mut self.text), upper);
        let family = to_egui_family(theme.typography.button.font_family);
        let text_galley = ui
            .painter()
            .layout_no_wrap(text, FontId::new(metrics.font_size, family), colors.text);

        let bucket = self.icon_size.unwrap_or(metrics.icon_bucket);
        let extent = icon_extent(bucket);
        let start_icon = self.start_icon.as_deref().filter(|s| !s.is_empty());
        let end_icon = self.end_icon.as_deref().filter(|s| !s.is_empty());

        let text_w = if has_text { text_galley.size().x } else { 0.0 };
        let text_h = if has_text { text_galley.size().y } else { 0.0 };
        let icon_h = if start_icon.is_some() || end_icon.is_some() { extent } else { 0.0 };
        let content_h = text_h.max(icon_h);

        let plan = plan_layout(
            &metrics,
            start_icon.map(|_| extent),
            text_w,
            content_h,
            end_icon.map(|_| extent),
            ui.available_width(),
            self.full_width,
        );

        // 3. Allocate and interact.
        let sense = if disabled { Sense::hover() } else { Sense::click() };
        let (rect, response) = ui.allocate_exact_size(plan.size, sense);
        response.widget_info(|| {
            WidgetInfo::labeled(WidgetType::Button, !disabled, text_galley.text())
        });

        if !disabled && response.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }

        // 4. Paint.
        if ui.is_rect_visible(rect) {
            let strength = if disabled {
                0.0
            } else {
                interaction_strength(response.hovered(), response.is_pointer_button_down_on())
            };
            let fill = state_fill(&self.variant, &colors, base, contrast, strength);
            let cy = rect.center().y;
            let left = rect.min.x;

            // 4a. Geometry (shadow, fill, border, focus ring) — painter only.
            {
                let painter = ui.painter();
                if self.drop_shadow && self.variant == Variant::Filled && !disabled {
                    let shadow = theme.current_shadows().get(2);
                    painter.add(shadow.as_shape(rect, corner_radius()));
                }
                if fill.a() > 0 {
                    painter.rect_filled(rect, corner_radius(), fill);
                }
                if colors.border.a() > 0 {
                    painter.rect_stroke(
                        rect,
                        corner_radius(),
                        Stroke::new(OUTLINE_WIDTH, colors.border),
                        StrokeKind::Inside,
                    );
                }
                if response.has_focus() && !disabled {
                    painter.rect_stroke(
                        rect.expand(FOCUS_OFFSET),
                        focus_radius(),
                        Stroke::new(FOCUS_WIDTH, base),
                        StrokeKind::Outside,
                    );
                }
            }

            // 4b. Label — through `MudText` (Rule 3).
            //
            // `display_label` already applied the uppercase transform, so
            // `.uppercase(false)` prevents `Typo::Button`'s default from
            // re-applying it (which would show the wrong case whenever the
            // caller set `uppercase(Some(false))`).
            if has_text {
                let pos = pos2(left + plan.text_x, cy - text_galley.size().y * 0.5);
                let text_rect = Rect::from_min_size(pos, text_galley.size());
                // Deliberately NOT `Ui::put`: `put` advances the parent's layout cursor to the
                // end of the label (which lies inside the button, short of its right padding),
                // so the next widget in a row/stack would start too early and swallow the gap.
                // A child `Ui` leaves the parent's cursor at the end of the allocated rect.
                let mut label_ui = ui.new_child(
                    UiBuilder::new()
                        .id_salt(response.id.with("label"))
                        .max_rect(text_rect)
                        .layout(Layout::left_to_right(Align::Center)),
                );
                label_ui.add(
                    MudText::button(text_galley.text())
                        .custom_color(colors.text)
                        .font_size(metrics.font_size)
                        .uppercase(false),
                );
            }

            // 4c. Icons — through `MudIcon`.
            let icon_rect = |x: f32| {
                Rect::from_center_size(pos2(left + x + extent * 0.5, cy), Vec2::splat(extent))
            };
            if let (Some(x), Some(icon)) = (plan.start_x, start_icon) {
                paint_icon(ui, icon_rect(x), icon, icon_color, bucket);
            }
            if let (Some(x), Some(icon)) = (plan.end_x, end_icon) {
                paint_icon(ui, icon_rect(x), icon, icon_color, bucket);
            }
        }

        response
    }
}

impl Widget for MudButton {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f32, b: f32) -> bool { (a - b).abs() < 1e-4 }
    fn medium() -> SizeMetrics { size_to_metrics(&Size::Medium) }

    #[test]
    fn metrics_medium_matches_original_table() {
        let m = medium();
        assert!(approx(m.pad_x, 16.0));
        assert!(approx(m.pad_y, 6.0));
        assert!(approx(m.min_height, 36.0));
        assert!(approx(m.font_size, 14.0));
    }

    #[test]
    fn metrics_grow_monotonically_with_size() {
        let all = [
            size_to_metrics(&Size::ExtraSmall),
            size_to_metrics(&Size::Small),
            size_to_metrics(&Size::Medium),
            size_to_metrics(&Size::Large),
            size_to_metrics(&Size::ExtraLarge),
        ];
        for pair in all.windows(2) {
            assert!(pair[0].min_height < pair[1].min_height);
            assert!(pair[0].font_size < pair[1].font_size);
            assert!(pair[0].pad_x < pair[1].pad_x);
            assert!(pair[0].icon_gap <= pair[1].icon_gap);
        }
    }

    #[test]
    fn uppercase_override_beats_typography_default() {
        assert!(resolve_uppercase(Some(true), false));
        assert!(!resolve_uppercase(Some(false), true));
        assert!(resolve_uppercase(None, true));
        assert!(!resolve_uppercase(None, false));
    }

    #[test]
    fn display_label_transforms_only_when_asked() {
        assert_eq!(display_label("Delete".into(), false), "Delete");
        assert_eq!(display_label("Delete".into(), true), "DELETE");
        assert_eq!(display_label(String::new(), true), "");
        assert_eq!(display_label("straße".into(), true), "STRASSE");
    }

    #[test]
    fn display_label_without_transform_keeps_buffer() {
        let s = String::from("keep");
        let ptr = s.as_ptr();
        assert_eq!(display_label(s, false).as_ptr(), ptr);
    }

    #[test]
    fn icon_extent_matches_buckets_and_falls_back() {
        assert!(approx(icon_extent(0), 8.0));
        assert!(approx(icon_extent(1), 16.0));
        assert!(approx(icon_extent(2), 24.0));
        assert!(approx(icon_extent(3), 32.0));
        assert!(approx(icon_extent(4), 48.0));
        assert!(approx(icon_extent(5), 24.0));
        assert!(approx(icon_extent(255), 24.0));
    }

    #[test]
    fn default_icon_buckets_are_valid_and_non_decreasing() {
        let sizes = [Size::ExtraSmall, Size::Small, Size::Medium, Size::Large, Size::ExtraLarge];
        let mut last = 0;
        for s in &sizes {
            let b = size_to_metrics(s).icon_bucket;
            assert!((b as usize) < ICON_SIZES_PX.len());
            assert!(b >= last);
            last = b;
        }
    }

    #[test]
    fn detects_svg_path_data() {
        assert!(looks_like_svg_path("M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12z"));
        assert!(looks_like_svg_path("m20 12-1.41-1.41L13 16.17"));
        assert!(looks_like_svg_path("  M 2 3"));
        assert!(looks_like_svg_path("M-1 2"));
        assert!(looks_like_svg_path("M.5 1"));
    }

    #[test]
    fn rejects_non_path_icon_strings() {
        assert!(!looks_like_svg_path(""));
        assert!(!looks_like_svg_path("★"));
        assert!(!looks_like_svg_path("Menu"));
        assert!(!looks_like_svg_path("M"));
        assert!(!looks_like_svg_path("M "));
        assert!(!looks_like_svg_path("h8 2"));
    }

    #[test]
    fn icon_kind_follows_detection() {
        assert!(matches!(icon_kind_for("M1 2h3"), IconKind::SvgPath));
        assert!(matches!(icon_kind_for("★"), IconKind::Glyph));
    }

    #[test]
    fn sanitize_repairs_bad_values() {
        assert!(approx(sanitize(5.0), 5.0));
        assert!(approx(sanitize(-3.0), 0.0));
        assert!(approx(sanitize(f32::NAN), 0.0));
        assert!(approx(sanitize(f32::INFINITY), 0.0));
        assert!(approx(sanitize(f32::NEG_INFINITY), 0.0));
    }

    #[test]
    fn contrast_black_on_light_white_on_dark() {
        assert_eq!(contrast_color(Color32::WHITE), Color32::BLACK);
        assert_eq!(contrast_color(Color32::BLACK), Color32::WHITE);
    }

    #[test]
    fn contrast_threshold_is_exclusive() {
        assert_eq!(contrast_color(Color32::from_rgb(128, 128, 128)), Color32::WHITE);
        assert_eq!(contrast_color(Color32::from_rgb(129, 128, 128)), Color32::BLACK);
    }

    #[test]
    fn pick_base_inherit_wins() {
        let inherit = Color32::from_rgb(1, 2, 3);
        assert_eq!(pick_base(true, inherit, Some(Color32::RED)), inherit);
    }

    #[test]
    fn pick_base_uses_resolved_or_white() {
        let inherit = Color32::from_rgb(1, 2, 3);
        assert_eq!(pick_base(false, inherit, Some(Color32::RED)), Color32::RED);
        assert_eq!(pick_base(false, inherit, None), Color32::WHITE);
    }

    #[test]
    fn mix_color_endpoints_and_clamping() {
        assert_eq!(mix_color(Color32::BLACK, Color32::WHITE, 0.0), Color32::BLACK);
        assert_eq!(mix_color(Color32::BLACK, Color32::WHITE, 1.0), Color32::WHITE);
        assert_eq!(mix_color(Color32::BLACK, Color32::WHITE, -5.0), Color32::BLACK);
        assert_eq!(mix_color(Color32::BLACK, Color32::WHITE, 9.0), Color32::WHITE);
    }

    #[test]
    fn mix_color_midpoint() {
        assert_eq!(mix_color(Color32::BLACK, Color32::WHITE, 0.5), Color32::from_rgb(128, 128, 128));
    }

    #[test]
    fn enabled_filled_colors() {
        let base = Color32::from_rgb(10, 20, 200);
        let c = resolve_colors(&Variant::Filled, false, base, Color32::WHITE);
        assert_eq!(c.fill, base);
        assert_eq!(c.text, Color32::WHITE);
        assert_eq!(c.border, Color32::TRANSPARENT);
    }

    #[test]
    fn enabled_outlined_colors() {
        let base = Color32::from_rgb(10, 20, 200);
        let c = resolve_colors(&Variant::Outlined, false, base, Color32::WHITE);
        assert_eq!(c.fill, Color32::TRANSPARENT);
        assert_eq!(c.text, base);
        assert_eq!(c.border, base);
    }

    #[test]
    fn enabled_text_colors() {
        let base = Color32::from_rgb(10, 20, 200);
        let c = resolve_colors(&Variant::Text, false, base, Color32::WHITE);
        assert_eq!(c.fill, Color32::TRANSPARENT);
        assert_eq!(c.text, base);
        assert_eq!(c.border, Color32::TRANSPARENT);
    }

    #[test]
    fn disabled_filled_keeps_visible_fill() {
        let base = Color32::from_rgb(10, 20, 200);
        let c = resolve_colors(&Variant::Filled, true, base, Color32::WHITE);
        assert!(c.fill.a() > 0);
        assert!(c.fill.a() < 255);
        assert!(c.text.a() < 255);
        assert!(c.text.a() > c.fill.a());
    }

    #[test]
    fn disabled_outlined_keeps_visible_border() {
        let base = Color32::from_rgb(10, 20, 200);
        let c = resolve_colors(&Variant::Outlined, true, base, Color32::WHITE);
        assert!(c.border.a() > 0);
        assert!(c.border.a() < 255);
        assert_eq!(c.fill, Color32::TRANSPARENT);
    }

    #[test]
    fn disabled_text_has_no_fill_or_border() {
        let c = resolve_colors(&Variant::Text, true, Color32::RED, Color32::WHITE);
        assert_eq!(c.fill, Color32::TRANSPARENT);
        assert_eq!(c.border, Color32::TRANSPARENT);
        assert!(c.text.a() < 255);
    }

    #[test]
    fn icon_color_follows_text_without_override() {
        let text = Color32::from_rgb(9, 9, 9);
        assert_eq!(icon_color_for(None, text, false), text);
        assert_eq!(icon_color_for(None, text, true), text);
    }

    #[test]
    fn icon_color_override_is_faded_only_when_disabled() {
        assert_eq!(icon_color_for(Some(Color32::RED), Color32::BLACK, false), Color32::RED);
        let faded = icon_color_for(Some(Color32::RED), Color32::BLACK, true);
        assert!(faded.a() < 255);
    }

    #[test]
    fn interaction_strength_priorities() {
        assert!(approx(interaction_strength(false, false), 0.0));
        assert!(approx(interaction_strength(true, false), HOVER_OVERLAY));
        assert!(approx(interaction_strength(true, true), PRESSED_OVERLAY));
        assert!(approx(interaction_strength(false, true), PRESSED_OVERLAY));
        assert!(PRESSED_OVERLAY > HOVER_OVERLAY);
    }

    #[test]
    fn state_fill_rest_returns_resting_fill() {
        let base = Color32::from_rgb(10, 20, 200);
        let c = resolve_colors(&Variant::Filled, false, base, Color32::WHITE);
        assert_eq!(state_fill(&Variant::Filled, &c, base, Color32::WHITE, 0.0), base);
    }

    #[test]
    fn state_fill_filled_moves_toward_contrast() {
        let base = Color32::from_rgb(10, 20, 200);
        let c = resolve_colors(&Variant::Filled, false, base, Color32::WHITE);
        let hovered = state_fill(&Variant::Filled, &c, base, Color32::WHITE, HOVER_OVERLAY);
        assert!(hovered.r() > base.r());
        assert!(hovered.g() > base.g());
        assert!(hovered.b() > base.b());
    }

    #[test]
    fn state_fill_text_and_outlined_show_translucent_wash() {
        let base = Color32::from_rgb(10, 20, 200);
        for v in [Variant::Text, Variant::Outlined] {
            let c = resolve_colors(&v, false, base, Color32::WHITE);
            let hovered = state_fill(&v, &c, base, Color32::WHITE, HOVER_OVERLAY);
            assert!(hovered.a() > 0);
            assert!(hovered.a() < 255);
        }
    }

    #[test]
    fn plan_text_only() {
        let p = plan_layout(&medium(), None, 50.0, 17.0, None, 1000.0, false);
        assert!(approx(p.size.x, 82.0));
        assert!(approx(p.size.y, 36.0));
        assert!(approx(p.text_x, 16.0));
        assert!(p.start_x.is_none());
        assert!(p.end_x.is_none());
    }

    #[test]
    fn plan_start_icon_and_text() {
        let p = plan_layout(&medium(), Some(20.0), 50.0, 17.0, None, 1000.0, false);
        assert!(approx(p.size.x, 110.0));
        assert!(approx(p.start_x.unwrap(), 16.0));
        assert!(approx(p.text_x, 44.0));
    }

    #[test]
    fn plan_text_and_end_icon() {
        let p = plan_layout(&medium(), None, 50.0, 17.0, Some(20.0), 1000.0, false);
        assert!(approx(p.size.x, 110.0));
        assert!(approx(p.text_x, 16.0));
        assert!(approx(p.end_x.unwrap(), 74.0));
    }

    #[test]
    fn plan_icon_only_has_no_phantom_gap() {
        let p = plan_layout(&medium(), Some(20.0), 0.0, 20.0, None, 1000.0, false);
        assert!(approx(p.size.x, 52.0));
        assert!(approx(p.start_x.unwrap(), 16.0));
    }

    #[test]
    fn plan_two_icons_without_text_have_single_gap() {
        let p = plan_layout(&medium(), Some(20.0), 0.0, 20.0, Some(20.0), 1000.0, false);
        assert!(approx(p.size.x, 80.0));
        assert!(approx(p.end_x.unwrap(), 44.0));
    }

    #[test]
    fn plan_full_width_centers_content() {
        let p = plan_layout(&medium(), None, 50.0, 17.0, None, 300.0, true);
        assert!(approx(p.size.x, 300.0));
        assert!(approx(p.text_x, 125.0));
    }

    #[test]
    fn plan_full_width_never_narrower_than_natural() {
        let p = plan_layout(&medium(), None, 50.0, 17.0, None, 40.0, true);
        assert!(approx(p.size.x, 82.0));
    }

    #[test]
    fn plan_full_width_with_non_finite_available_falls_back() {
        let p = plan_layout(&medium(), None, 50.0, 17.0, None, f32::INFINITY, true);
        assert!(approx(p.size.x, 82.0));
        let p = plan_layout(&medium(), None, 50.0, 17.0, None, f32::NAN, true);
        assert!(approx(p.size.x, 82.0));
    }

    #[test]
    fn plan_tall_content_grows_button() {
        let p = plan_layout(&medium(), None, 50.0, 40.0, None, 1000.0, false);
        assert!(approx(p.size.y, 52.0));
    }

    #[test]
    fn plan_repairs_bad_measurements() {
        let p = plan_layout(&medium(), Some(f32::NAN), -4.0, f32::NAN, Some(-1.0), 100.0, false);
        assert!(p.size.x.is_finite() && p.size.y.is_finite());
        assert!(p.text_x.is_finite());
        assert!(approx(p.size.y, 36.0));
    }

    #[test]
    fn defaults() {
        let b = MudButton::default();
        assert!(b.text.is_empty());
        assert!(matches!(b.variant, Variant::Text));
        assert!(matches!(b.size, Size::Medium));
        assert!(b.color == Color::Primary);
        assert!(!b.disabled && !b.full_width && b.drop_shadow);
        assert!(b.start_icon.is_none() && b.end_icon.is_none());
        assert!(b.icon_color.is_none());
        assert!(b.icon_size.is_none());
        assert!(b.uppercase.is_none());
    }

    #[test]
    fn builders_set_fields() {
        let b = MudButton::new("Go")
            .variant(Variant::Filled)
            .color(Color::Inherit)
            .size(Size::Large)
            .disabled(true)
            .full_width(true)
            .drop_shadow(false)
            .start_icon("A")
            .end_icon("B")
            .icon_color(Color::Primary)
            .icon_size(3)
            .uppercase(false)
            .classes(["a", "b"]);
        assert_eq!(b.text, "Go");
        assert!(matches!(b.variant, Variant::Filled));
        assert!(b.color == Color::Inherit);
        assert!(matches!(b.size, Size::Large));
        assert!(b.disabled && b.full_width && !b.drop_shadow);
        assert_eq!(b.start_icon.as_deref(), Some("A"));
        assert_eq!(b.end_icon.as_deref(), Some("B"));
        assert!(b.icon_color.is_some());
        assert_eq!(b.icon_size, Some(3));
        assert_eq!(b.uppercase, Some(false));
    }

    #[test]
    fn new_accepts_str_and_string() {
        assert_eq!(MudButton::new("x").text, "x");
        assert_eq!(MudButton::new(String::from("y")).text, "y");
    }
}