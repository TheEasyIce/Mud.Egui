//! `MudFab` — a floating action button.
//!
//! Port of `MudFab.razor.cs` (MudBlazor 8.13.0). Pass 1.
//!
//! # Composition
//!
//! `MudFab` is self-painted, like [`MudButton`]. Text goes through
//! [`MudText`]; icons go through [`MudIcon`].
//!
//! # Layout
//!
//! * **Icon-only** — square `side × side` with a fully-circular corner radius.
//! * **With `label`** — pill-shaped: `[start_icon] [label] [end_icon]` with
//!   horizontal padding, height fixed to `side`.
//! * **`child` set** — the closure is given the circular/square rect; its
//!   content decides what fills it.
//!
//! # Fixed positioning
//!
//! `fixed(true)` renders the FAB in an [`egui::Area`] anchored to
//! `Align2::RIGHT_BOTTOM` at `-(right, bottom)`. Space is not reserved in the
//! parent `Ui`, matching CSS `position: fixed`.

use egui::{
    pos2, vec2, Color32, CornerRadius, CursorIcon, FontId, Rect, Response, Sense,
    Stroke, Ui, UiBuilder, Vec2,
};
use mud_egui_core::base::component_base::ComponentBase;
use mud_egui_core::enums::{Color, Size, Typo, Variant};
use mud_egui_theming::models::palettes::palette::Palette;
use mud_egui_theming::theme::MudTheme;

use crate::data_display::icons::mud_icon::MudIcon;
use crate::typography::mud_text::MudText;

/// Default `right` offset for a fixed FAB.
pub const DEFAULT_RIGHT: f32 = 16.0;

/// Default `bottom` offset for a fixed FAB.
pub const DEFAULT_BOTTOM: f32 = 16.0;

const HOVER_ALPHA: u8 = 18;
const DISABLED_ALPHA: f32 = 0.38;

/// Per-size dimensions for a FAB.
#[derive(Debug, Clone, Copy)]
pub struct FabMetrics {
    pub side: f32,
    pub icon_px: f32,
    pub pad_x: f32,
    pub label_gap: f32,
}

impl FabMetrics {
    /// The metrics for `size`. `ExtraSmall` folds into `Small` and
    /// `ExtraLarge` into `Large`.
    pub fn resolve(size: Size) -> Self {
        match size {
            Size::ExtraSmall | Size::Small => Self {
                side: 40.0,
                icon_px: 20.0,
                pad_x: 16.0,
                label_gap: 8.0,
            },
            Size::Medium => Self {
                side: 48.0,
                icon_px: 24.0,
                pad_x: 20.0,
                label_gap: 8.0,
            },
            Size::Large | Size::ExtraLarge => Self {
                side: 56.0,
                icon_px: 24.0,
                pad_x: 24.0,
                label_gap: 8.0,
            },
        }
    }
}

/// Font size used for a labelled FAB, in points.
#[inline]
pub(crate) fn fab_label_font_size(size: Size) -> f32 {
    match size {
        Size::ExtraSmall | Size::Small => 12.0,
        Size::Medium => 14.0,
        Size::Large | Size::ExtraLarge => 16.0,
    }
}

/// Resolved colours for a FAB.
#[derive(Debug, Clone, Copy)]
pub(crate) struct FabColors {
    pub fill: Color32,
    pub ink: Color32,
    pub stroke: Option<Stroke>,
}

pub(crate) fn resolve_colors(
    variant: Variant,
    color: Color,
    disabled: bool,
    fallback: Color32,
    palette: &Palette,
) -> FabColors {
    let accent = match color {
        Color::Default | Color::Inherit => fallback,
        other => other.resolve(palette).map(Color32::from).unwrap_or(fallback),
    };
    let contrast = if (accent.r() as u32 + accent.g() as u32 + accent.b() as u32) > 384 {
        Color32::BLACK
    } else {
        Color32::WHITE
    };
    let alpha = if disabled { DISABLED_ALPHA } else { 1.0 };
    let a = accent.gamma_multiply(alpha);
    match variant {
        Variant::Filled => FabColors {
            fill: a,
            ink: contrast.gamma_multiply(alpha),
            stroke: None,
        },
        Variant::Outlined => FabColors {
            fill: Color32::TRANSPARENT,
            ink: a,
            stroke: Some(Stroke::new(1.0, a)),
        },
        Variant::Text => FabColors {
            fill: Color32::TRANSPARENT,
            ink: a,
            stroke: None,
        },
    }
}

pub(crate) fn paint_fab_bg(
    ui: &Ui,
    rect: Rect,
    colors: &FabColors,
    radius: f32,
    drop_shadow: bool,
) {
    let r = radius.round().clamp(0.0, 255.0) as u8;
    let cr = CornerRadius::same(r);
    if drop_shadow {
        for (dy, a) in [(3.0_f32, 18u8), (2.0, 28), (1.0, 38)] {
            ui.painter()
                .rect_filled(rect.translate(vec2(0.0, dy)), cr, Color32::from_black_alpha(a));
        }
    }
    if colors.fill.a() > 0 {
        ui.painter().rect_filled(rect, cr, colors.fill);
    }
    if let Some(stroke) = colors.stroke {
        ui.painter()
            .rect_stroke(rect, cr, stroke, egui::StrokeKind::Inside);
    }
}

/// A floating action button.
#[must_use = "MudFab does nothing until you call `.show(ui)`"]
pub struct MudFab<'a> {
    pub label: Option<String>,
    pub icon: Option<String>,
    pub start_icon: Option<String>,
    pub end_icon: Option<String>,
    pub size: Size,
    pub color: Color,
    pub variant: Variant,
    pub disabled: bool,
    pub drop_shadow: bool,
    pub title: Option<String>,
    pub fixed: bool,
    pub bottom: Option<f32>,
    pub right: Option<f32>,
    pub href: Option<String>,
    pub target: Option<String>,
    pub on_click: Option<Box<dyn FnMut() + 'a>>,
    pub child: Option<Box<dyn FnMut(&mut Ui) + 'a>>,
    pub base: ComponentBase,
}

impl<'a> Default for MudFab<'a> {
    fn default() -> Self {
        Self {
            label: None,
            icon: None,
            start_icon: None,
            end_icon: None,
            size: Size::Large,
            color: Color::Default,
            variant: Variant::Filled,
            disabled: false,
            drop_shadow: true,
            title: None,
            fixed: false,
            bottom: None,
            right: None,
            href: None,
            target: None,
            on_click: None,
            child: None,
            base: ComponentBase::new(),
        }
    }
}

impl<'a> MudFab<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    #[inline] pub fn label(mut self, s: impl Into<String>) -> Self { self.label = Some(s.into()); self }
    #[inline] pub fn icon(mut self, s: impl Into<String>) -> Self { self.icon = Some(s.into()); self }
    #[inline] pub fn start_icon(mut self, s: impl Into<String>) -> Self { self.start_icon = Some(s.into()); self }
    #[inline] pub fn end_icon(mut self, s: impl Into<String>) -> Self { self.end_icon = Some(s.into()); self }
    #[inline] pub fn size(mut self, s: Size) -> Self { self.size = s; self }
    #[inline] pub fn color(mut self, c: Color) -> Self { self.color = c; self }
    #[inline] pub fn variant(mut self, v: Variant) -> Self { self.variant = v; self }
    #[inline] pub fn disabled(mut self, b: bool) -> Self { self.disabled = b; self }
    #[inline] pub fn drop_shadow(mut self, b: bool) -> Self { self.drop_shadow = b; self }
    #[inline] pub fn title(mut self, s: impl Into<String>) -> Self { self.title = Some(s.into()); self }
    #[inline] pub fn fixed(mut self, b: bool) -> Self { self.fixed = b; self }
    #[inline] pub fn bottom(mut self, px: f32) -> Self { self.bottom = Some(px); self }
    #[inline] pub fn right(mut self, px: f32) -> Self { self.right = Some(px); self }
    #[inline] pub fn href(mut self, s: impl Into<String>) -> Self { self.href = Some(s.into()); self }
    #[inline] pub fn target(mut self, s: impl Into<String>) -> Self { self.target = Some(s.into()); self }
    #[inline] pub fn on_click(mut self, f: impl FnMut() + 'a) -> Self { self.on_click = Some(Box::new(f)); self }
    #[inline] pub fn child(mut self, f: impl FnMut(&mut Ui) + 'a) -> Self { self.child = Some(Box::new(f)); self }
    #[inline] pub fn classes<S: AsRef<str>>(mut self, classes: impl IntoIterator<Item = S>) -> Self { self.base = self.base.classes(classes); self }

    pub fn show(self, ui: &mut Ui) -> Response {
        if self.fixed {
            self.show_fixed(ui).inner
        } else {
            self.show_in_place(ui)
        }
    }

    fn show_fixed(self, ui: &mut Ui) -> egui::InnerResponse<Response> {
        let right = self.right.unwrap_or(DEFAULT_RIGHT);
        let bottom = self.bottom.unwrap_or(DEFAULT_BOTTOM);
        let id = egui::Id::new("mud_fab_fixed")
            .with(self.icon.as_deref().unwrap_or(""))
            .with(self.label.as_deref().unwrap_or(""))
            .with(right.to_bits())
            .with(bottom.to_bits());
        let area = egui::Area::new(id)
            .order(egui::Order::Foreground)
            .anchor(egui::Align2::RIGHT_BOTTOM, vec2(-right, -bottom));
        area.show(ui.ctx(), |ui| self.show_in_place(ui))
    }

    fn show_in_place(mut self, ui: &mut Ui) -> Response {
        let theme = MudTheme::fetch(ui.ctx()).unwrap_or_default();
        let palette = theme.current_palette();
        let fallback = ui.visuals().text_color();
        let colors = resolve_colors(self.variant, self.color, self.disabled, fallback, palette);
        let metrics = FabMetrics::resolve(self.size);
        let sense = if self.disabled { Sense::hover() } else { Sense::click() };

        // 1. Custom-content path.
        if let Some(mut child) = self.child.take() {
            let (rect, response) = ui.allocate_exact_size(Vec2::splat(metrics.side), sense);
            if ui.is_rect_visible(rect) {
                paint_fab_bg(
                    ui,
                    rect,
                    &colors,
                    metrics.side * 0.5,
                    self.drop_shadow && matches!(self.variant, Variant::Filled),
                );
                if !self.disabled && response.hovered() {
                    ui.painter().rect_filled(
                        rect,
                        CornerRadius::same((metrics.side * 0.5) as u8),
                        with_alpha(colors.ink, HOVER_ALPHA),
                    );
                }
                let mut child_ui = ui.new_child(
                    UiBuilder::new()
                        .max_rect(rect)
                        .layout(egui::Layout::centered_and_justified(egui::Direction::LeftToRight)),
                );
                child(&mut child_ui);
            }
            return finish_response(
                ui,
                response,
                self.disabled,
                self.title.as_deref(),
                self.on_click.as_mut().map(|b| b.as_mut()),
                self.href.as_deref(),
                self.target.as_deref(),
            );
        }

        // 2. Label vs icon-only layout.
        let fs = fab_label_font_size(self.size);
        let label_galley = self.label.as_ref().map(|label| {
            ui.painter()
                .layout_no_wrap(label.clone(), FontId::proportional(fs), colors.ink)
        });

        let (width, height) = match &label_galley {
            Some(g) => {
                let start_w = self.start_icon.as_ref().map_or(0.0, |_| metrics.icon_px + metrics.label_gap);
                let end_w = self.end_icon.as_ref().map_or(0.0, |_| metrics.icon_px + metrics.label_gap);
                let content_w = g.size().x + start_w + end_w;
                ((content_w + 2.0 * metrics.pad_x).max(metrics.side), metrics.side)
            }
            None => (metrics.side, metrics.side),
        };

        let (rect, response) = ui.allocate_exact_size(vec2(width, height), sense);

        if ui.is_rect_visible(rect) {
            let radius = height * 0.5;
            paint_fab_bg(
                ui,
                rect,
                &colors,
                radius,
                self.drop_shadow && matches!(self.variant, Variant::Filled),
            );
            if !self.disabled && response.hovered() {
                ui.painter().rect_filled(
                    rect,
                    CornerRadius::same(radius.round() as u8),
                    with_alpha(colors.ink, HOVER_ALPHA),
                );
            }

            match label_galley {
                Some(g) => {
                    let start_w = self.start_icon.as_ref().map_or(0.0, |_| metrics.icon_px + metrics.label_gap);
                    let end_w = self.end_icon.as_ref().map_or(0.0, |_| metrics.icon_px + metrics.label_gap);
                    let content_w = g.size().x + start_w + end_w;
                    let mut x = rect.center().x - content_w * 0.5;
                    let cy = rect.center().y;

                    if let Some(icon) = self.start_icon.as_deref() {
                        let icon_rect = Rect::from_center_size(
                            pos2(x + metrics.icon_px * 0.5, cy),
                            Vec2::splat(metrics.icon_px),
                        );
                        ui.put(
                            icon_rect,
                            MudIcon::new(icon.to_owned())
                                .size_px(metrics.icon_px)
                                .custom_color(colors.ink),
                        );
                        x += metrics.icon_px + metrics.label_gap;
                    }

                    let label_rect = Rect::from_center_size(
                        pos2(x + g.size().x * 0.5, cy),
                        g.size(),
                    );
                    ui.put(
                        label_rect,
                        MudText::new(self.label.clone().unwrap_or_default())
                            .typo(Typo::Button)
                            .font_size(fs)
                            .custom_color(colors.ink),
                    );
                    x += g.size().x + metrics.label_gap;

                    if let Some(icon) = self.end_icon.as_deref() {
                        let icon_rect = Rect::from_center_size(
                            pos2(x + metrics.icon_px * 0.5, cy),
                            Vec2::splat(metrics.icon_px),
                        );
                        ui.put(
                            icon_rect,
                            MudIcon::new(icon.to_owned())
                                .size_px(metrics.icon_px)
                                .custom_color(colors.ink),
                        );
                    }
                }
                None => {
                    if let Some(icon) = self.icon.as_deref() {
                        let icon_rect = Rect::from_center_size(
                            rect.center(),
                            Vec2::splat(metrics.icon_px),
                        );
                        ui.put(
                            icon_rect,
                            MudIcon::new(icon.to_owned())
                                .size_px(metrics.icon_px)
                                .custom_color(colors.ink),
                        );
                    }
                }
            }
        }

        finish_response(
            ui,
            response,
            self.disabled,
            self.title.as_deref(),
            self.on_click.as_mut().map(|b| b.as_mut()),
            self.href.as_deref(),
            self.target.as_deref(),
        )
    }
}

impl<'a> egui::Widget for MudFab<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui)
    }
}

/// Finalises the trigger's `Response`. Generic over the callback lifetime so
/// a caller's closure can borrow non-`'static` data.
fn finish_response<'a>(
    ui: &Ui,
    mut response: Response,
    disabled: bool,
    title: Option<&str>,
    on_click: Option<&mut (dyn FnMut() + 'a)>,
    href: Option<&str>,
    target: Option<&str>,
) -> Response {
    if let Some(tip) = title {
        response = response.on_hover_text(tip);
    }
    if !disabled && response.clicked() {
        if let Some(cb) = on_click {
            cb();
        }
        if let Some(url) = href {
            let open = if target == Some("_blank") {
                egui::OpenUrl::new_tab(url.to_owned())
            } else {
                egui::OpenUrl::same_tab(url.to_owned())
            };
            ui.ctx().open_url(open);
        }
    }
    if disabled {
        response.on_hover_cursor(CursorIcon::NotAllowed)
    } else {
        response.on_hover_cursor(CursorIcon::PointingHand)
    }
}

#[inline]
fn with_alpha(c: Color32, alpha: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), alpha)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metrics_match_the_material_sizes() {
        let s = FabMetrics::resolve(Size::Small);
        assert_eq!((s.side, s.icon_px, s.pad_x), (40.0, 20.0, 16.0));
        let m = FabMetrics::resolve(Size::Medium);
        assert_eq!((m.side, m.icon_px, m.pad_x), (48.0, 24.0, 20.0));
        let l = FabMetrics::resolve(Size::Large);
        assert_eq!((l.side, l.icon_px, l.pad_x), (56.0, 24.0, 24.0));
    }

    #[test]
    fn metrics_fold_edge_sizes() {
        assert_eq!(FabMetrics::resolve(Size::ExtraSmall).side, FabMetrics::resolve(Size::Small).side);
        assert_eq!(FabMetrics::resolve(Size::ExtraLarge).side, FabMetrics::resolve(Size::Large).side);
    }

    #[test]
    fn label_font_scales_with_size() {
        assert_eq!(fab_label_font_size(Size::Small), 12.0);
        assert_eq!(fab_label_font_size(Size::Medium), 14.0);
        assert_eq!(fab_label_font_size(Size::Large), 16.0);
    }

    fn palette() -> Palette {
        Palette::new_light()
    }

    #[test]
    fn filled_uses_contrast_ink() {
        let c = resolve_colors(Variant::Filled, Color::Default, false, Color32::from_rgb(20, 20, 20), &palette());
        assert_eq!(c.ink, Color32::WHITE);
        assert!(c.stroke.is_none());
    }

    #[test]
    fn outlined_uses_accent_for_ink_and_stroke() {
        let c = resolve_colors(Variant::Outlined, Color::Default, false, Color32::from_rgb(20, 20, 20), &palette());
        assert_eq!(c.fill, Color32::TRANSPARENT);
        assert!(c.stroke.is_some());
        assert_eq!(c.stroke.unwrap().color, c.ink);
    }

    #[test]
    fn text_has_no_fill_or_stroke() {
        let c = resolve_colors(Variant::Text, Color::Default, false, Color32::from_rgb(20, 20, 20), &palette());
        assert_eq!(c.fill, Color32::TRANSPARENT);
        assert!(c.stroke.is_none());
    }

    #[test]
    fn disabled_fades_everything() {
        let c = resolve_colors(Variant::Filled, Color::Default, true, Color32::from_rgb(20, 20, 20), &palette());
        assert!(c.ink.a() < 255);
    }

    #[test]
    fn defaults_match_mudblazor() {
        let f: MudFab = MudFab::new();
        assert!(f.label.is_none());
        assert!(f.icon.is_none());
        assert!(f.start_icon.is_none() && f.end_icon.is_none());
        assert!(matches!(f.size, Size::Large));
        assert!(matches!(f.color, Color::Default));
        assert!(matches!(f.variant, Variant::Filled));
        assert!(!f.disabled);
        assert!(f.drop_shadow);
        assert!(f.title.is_none());
        assert!(!f.fixed);
        assert!(f.bottom.is_none() && f.right.is_none());
        assert!(f.href.is_none() && f.target.is_none());
        assert!(f.on_click.is_none());
        assert!(f.child.is_none());
    }

    #[test]
    fn builders_set_every_field() {
        let f: MudFab = MudFab::new()
            .label("L")
            .icon("I")
            .start_icon("S")
            .end_icon("E")
            .size(Size::Medium)
            .color(Color::Primary)
            .variant(Variant::Outlined)
            .disabled(true)
            .drop_shadow(false)
            .title("T")
            .fixed(true)
            .bottom(24.0)
            .right(12.0)
            .href("https://x")
            .target("_blank")
            .on_click(|| {})
            .child(|_| {})
            .classes(["cls"]);

        assert_eq!(f.label.as_deref(), Some("L"));
        assert_eq!(f.icon.as_deref(), Some("I"));
        assert_eq!(f.start_icon.as_deref(), Some("S"));
        assert_eq!(f.end_icon.as_deref(), Some("E"));
        assert!(matches!(f.size, Size::Medium));
        assert!(matches!(f.color, Color::Primary));
        assert!(matches!(f.variant, Variant::Outlined));
        assert!(f.disabled && !f.drop_shadow);
        assert_eq!(f.title.as_deref(), Some("T"));
        assert!(f.fixed);
        assert_eq!(f.bottom, Some(24.0));
        assert_eq!(f.right, Some(12.0));
        assert_eq!(f.href.as_deref(), Some("https://x"));
        assert_eq!(f.target.as_deref(), Some("_blank"));
        assert!(f.on_click.is_some());
        assert!(f.child.is_some());
        assert!(f.base.has_class("cls"));
    }
}