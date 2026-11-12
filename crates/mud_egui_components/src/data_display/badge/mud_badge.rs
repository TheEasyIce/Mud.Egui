//! `MudBadge` — overlays a dot or short count onto the corner of its child.
//!
//! Port of `MudBadge.razor.cs` (MudBlazor 8.13.0). Public parameter surface:
//!
//! | MudBlazor parameter | Rust method | Default |
//! |---------------------|-------------|---------|
//! | `BadgeAriaLabel`    | [`MudBadge::badge_aria_label`] | `None` |
//! | `BadgeClass`        | [`MudBadge::badge_class`] | `None` |
//! | `Bordered`          | [`MudBadge::bordered`] | `false` |
//! | `ChildContent`      | the `child` closure of [`MudBadge::show`] | required |
//! | `Color`             | [`MudBadge::color`] | [`Color::Default`] |
//! | `Content`           | [`MudBadge::content`] / [`MudBadge::count`] | `None` |
//! | `Dot`               | [`MudBadge::dot`] | `false` |
//! | `Elevation`         | [`MudBadge::elevation`] | `0` |
//! | `Icon`              | [`MudBadge::icon`] | `None` |
//! | `Max`               | [`MudBadge::max`] | `99` |
//! | `OnClick`           | [`MudBadge::on_click`] | `None` |
//! | `Origin`            | [`MudBadge::origin`] | [`Origin::TopRight`] |
//! | `Overlap`           | [`MudBadge::overlap`] | `false` |
//! | `Visible`           | [`MudBadge::visible`] | `true` |
//!
//! # Positioning
//!
//! The badge centre is placed **on** the child's edge when `overlap(true)`;
//! when `overlap(false)` (the MudBlazor default) it is pushed fully outside
//! the child by half its own size. This mirrors MudBlazor's CSS
//! `transform: translate(…%)` on the badge element, so wide `"99+"` text
//! badges never clip into the child.
//!
//! # Content kinds
//!
//! Exactly one of [`dot`](MudBadge::dot), [`icon`](MudBadge::icon), or
//! [`content`](MudBadge::content) determines the rendering path. `dot` wins
//! over `icon`, which wins over `content`. If none is set, no badge is drawn.
//!
//! # Click handling
//!
//! The badge becomes a click target as soon as [`on_click`](MudBadge::on_click)
//! is set (MudBlazor's `OnClick.HasDelegate`). The child keeps its own
//! interaction; [`show`](MudBadge::show) returns both click results so
//! callers can tell them apart.
//!
//! # Rendering extension
//!
//! A 1.5 px surface-coloured mask is painted around the badge so it separates
//! cleanly from the child, regardless of the child's background. This is a
//! rendering choice, not a MudBlazor parameter.

use egui::{
    vec2, Color32, CornerRadius, CursorIcon, Pos2, Rect, Response, Sense, Stroke,
    StrokeKind, Ui,
};
use mud_egui_bootstrap::bs_style::BsStyle;
use mud_egui_core::base::component_base::ComponentBase;
use mud_egui_core::enums::{Color, Origin, StyleFlavor};
use mud_egui_tailwind::tw_style::{TwState, TwStyle};
use mud_egui_theming::theme::MudTheme;

use crate::data_display::avatar::mud_avatar::elevation_shadow;
use crate::data_display::icons::mud_icon::MudIcon;
use crate::typography::mud_text::MudText;

/// Base rem size (in points) passed to the Tailwind resolver.
const REM_SIZE_PT: f32 = 14.0;

/// Diameter of a dot badge, in logical pixels.
const DOT_EXTENT: f32 = 9.0;
/// Minimum side length of an icon or text badge.
const MIN_TEXT_EXTENT: f32 = 20.0;
/// Horizontal padding inside a text badge.
const TEXT_PADDING_X: f32 = 6.0;
/// Tighter horizontal padding inside a bordered text badge.
const BORDERED_TEXT_PADDING_X: f32 = 4.0;
/// Font size used inside a text badge.
const BADGE_FONT_SIZE: f32 = 11.0;
/// Thickness of the `bordered` ring stroke.
const BORDER_RING_THICKNESS: f32 = 2.0;
/// Height of a text or icon badge.
const BADGE_HEIGHT: f32 = 20.0;
/// Glyph size inside an icon badge.
const BADGE_ICON_SIZE: f32 = 12.0;
/// Thickness of the surface mask painted around the badge.
const MASK_THICKNESS: f32 = 1.5;

#[inline]
fn resolve_color(c: Color, theme: &MudTheme, default: Color32) -> Color32 {
    let palette = theme.current_palette();
    match c {
        Color::Default | Color::Inherit => default,
        other => other.resolve(palette).unwrap_or(default),
    }
}

/// Resolves the (horizontal, vertical) anchor direction for an [`Origin`].
///
/// Returns `(h, v)` where `-1` = start edge, `0` = centre, `1` = end edge.
#[inline]
fn anchor_sides(origin: Origin) -> (i8, i8) {
    match origin {
        Origin::TopLeft => (-1, -1),
        Origin::TopCenter => (0, -1),
        Origin::TopRight => (1, -1),
        Origin::CenterLeft => (-1, 0),
        Origin::Center => (0, 0),
        Origin::CenterRight => (1, 0),
        Origin::BottomLeft => (-1, 1),
        Origin::BottomCenter => (0, 1),
        Origin::BottomRight => (1, 1),
    }
}

/// Computes the badge centre relative to the child rect.
///
/// * `overlap = true`  → the badge centre sits **on** the child edge.
/// * `overlap = false` → the badge is pushed **fully outside** the child by
///   half its own size on the anchored axis.
///
/// On the non-anchored axis (`h == 0` or `v == 0`) the badge is centred on
/// the child's axis, matching MudBlazor.
pub(crate) fn badge_center(
    child_rect: Rect,
    badge_w: f32,
    badge_h: f32,
    origin: Origin,
    overlap: bool,
) -> Pos2 {
    let (h, v) = anchor_sides(origin);
    let half_w = badge_w * 0.5;
    let half_h = badge_h * 0.5;

    let x = match h {
        -1 => {
            if overlap {
                child_rect.left()
            } else {
                child_rect.left() - half_w
            }
        }
        1 => {
            if overlap {
                child_rect.right()
            } else {
                child_rect.right() + half_w
            }
        }
        _ => child_rect.center().x,
    };

    let y = match v {
        -1 => {
            if overlap {
                child_rect.top()
            } else {
                child_rect.top() - half_h
            }
        }
        1 => {
            if overlap {
                child_rect.bottom()
            } else {
                child_rect.bottom() + half_h
            }
        }
        _ => child_rect.center().y,
    };

    egui::pos2(x, y)
}

/// Formats a badge caption: integer strings larger than `max` become
/// `"{max}+"`. Non-numeric strings pass through unchanged.
pub(crate) fn format_count(content: &str, max: i32) -> String {
    match content.parse::<i32>() {
        Ok(n) if n > max => format!("{max}+"),
        _ => content.to_owned(),
    }
}

/// Resolved visual style for a badge, computed once per frame.
struct BadgeStyle {
    /// Background fill.
    fill: Color32,
    /// Foreground (icon / text) colour.
    ink: Color32,
    /// Mask colour painted 1.5 px around the badge.
    ring: Color32,
    /// Optional drop shadow (`Elevation > 0`).
    shadow: Option<egui::epaint::Shadow>,
    /// Optional outline (`Bordered` or a `border-*` / `ring-*` class).
    stroke: Option<Stroke>,
}

/// Resolves the badge style from the `Color`, `BadgeClass`, `Bordered`, and
/// `Elevation` parameters.
///
/// Takes `&mut BsStyle` / `&mut TwStyle` because `build_frame` mutates
/// internal resolver caches. Both are owned by the caller (`MudBadge`); the
/// borrow does not escape this call.
#[allow(clippy::too_many_arguments)]
fn resolve_badge_style(
    color: Color,
    theme: &MudTheme,
    flavor: StyleFlavor,
    badge_class: Option<&str>,
    bs_style: &mut BsStyle,
    tw_style: &mut TwStyle,
    tw_state: TwState,
    elevation: u8,
    bordered: bool,
) -> BadgeStyle {
    let palette = theme.current_palette();
    let lookup = |key: &str| palette.resolve(key).map(Color32::from);

    let accent: Color32 = resolve_color(color, theme, palette.primary.into());
    let ink: Color32 = palette.primary_contrast_text.into();
    let ring: Color32 = palette.surface.into();

    let mut fill = accent;
    let mut stroke: Option<Stroke> = None;

    if let Some(class) = badge_class.filter(|s| !s.trim().is_empty()) {
        let base = ComponentBase::new().classes(class.split_whitespace());
        let frame = match flavor {
            StyleFlavor::Bootstrap => bs_style.build_frame(&base, &lookup),
            StyleFlavor::Tailwind => {
                tw_style.build_frame(&base, tw_state, REM_SIZE_PT, &lookup)
            }
        };
        // `egui::Frame::fill` is a bare `Color32`. `Color32::TRANSPARENT` is
        // its "class didn't request a fill" sentinel, so only override the
        // accent-derived fill when the class actually sets a colour.
        if frame.fill != Color32::TRANSPARENT {
            fill = frame.fill;
        }
        if frame.stroke != Stroke::NONE {
            stroke = Some(frame.stroke);
        }
    }

    if bordered {
        stroke = Some(Stroke::new(BORDER_RING_THICKNESS, ring));
    }

    BadgeStyle {
        fill,
        ink,
        ring,
        shadow: (elevation > 0).then(|| elevation_shadow(elevation)),
        stroke,
    }
}

/// Overlays a dot or short count onto an arbitrary corner of its child.
#[must_use = "MudBadge does nothing until you call `.show(ui, child)`"]
pub struct MudBadge {
    color: Color,
    dot: bool,
    content: Option<String>,
    icon: Option<String>,
    max: i32,
    overlap: bool,
    visible: bool,
    bordered: bool,
    origin: Origin,
    elevation: u8,
    badge_class: Option<String>,
    badge_aria_label: Option<String>,
    on_click: Option<Box<dyn FnMut()>>,

    // CSS utilities
    pub base: ComponentBase,
    pub flavor: StyleFlavor,
    pub bs_style: BsStyle,
    pub tw_style: TwStyle,
    pub tw_state: TwState,
}

impl Default for MudBadge {
    fn default() -> Self {
        Self::new()
    }
}

impl MudBadge {
    /// Creates a badge with MudBlazor defaults: `Color::Default`,
    /// `Origin::TopRight`, `Overlap(false)`, `Max = 99`, `Visible = true`.
    pub fn new() -> Self {
        Self {
            color: Color::Default,
            dot: false,
            content: None,
            icon: None,
            max: 99,
            overlap: false,
            visible: true,
            bordered: false,
            origin: Origin::TopRight,
            elevation: 0,
            badge_class: None,
            badge_aria_label: None,
            on_click: None,
            base: ComponentBase::new(),
            flavor: StyleFlavor::default(),
            bs_style: BsStyle::default(),
            tw_style: TwStyle::default(),
            tw_state: TwState::default(),
        }
    }

    // ── Builder methods ──────────────────────────────────────────────────────

    /// Sets the badge colour. Mirrors MudBlazor `Color` (default `Default`,
    /// which resolves to the theme's primary colour).
    #[inline]
    pub fn color(mut self, c: Color) -> Self {
        self.color = c;
        self
    }

    /// Renders a small dot instead of any content. Mirrors MudBlazor `Dot`.
    #[inline]
    pub fn dot(mut self, b: bool) -> Self {
        self.dot = b;
        self
    }

    /// Sets the badge caption. Numeric strings are capped by [`max`](Self::max).
    /// Mirrors MudBlazor `Content` (string form).
    #[inline]
    pub fn content(mut self, text: impl Into<String>) -> Self {
        self.content = Some(text.into());
        self
    }

    /// Sets the badge caption from an integer. Mirrors MudBlazor `Content`
    /// (int form); capping by [`max`](Self::max) is applied automatically.
    #[inline]
    pub fn count(mut self, n: i32) -> Self {
        self.content = Some(n.to_string());
        self
    }

    /// Sets an icon (Material SVG path) inside the badge. Mirrors MudBlazor
    /// `Icon`.
    #[inline]
    pub fn icon(mut self, svg: impl Into<String>) -> Self {
        self.icon = Some(svg.into());
        self
    }

    /// Sets the maximum count; values greater than `max` render as `"{max}+"`.
    /// Mirrors MudBlazor `Max` (default `99`).
    #[inline]
    pub fn max(mut self, m: i32) -> Self {
        self.max = m.max(0);
        self
    }

    /// When `true`, the badge centre sits on the child's edge instead of
    /// being pushed outside. Mirrors MudBlazor `Overlap` (default `false`).
    #[inline]
    pub fn overlap(mut self, b: bool) -> Self {
        self.overlap = b;
        self
    }

    /// Shows or hides the badge without removing the child. Mirrors MudBlazor
    /// `Visible` (default `true`).
    #[inline]
    pub fn visible(mut self, b: bool) -> Self {
        self.visible = b;
        self
    }

    /// Draws a border ring around the badge. Mirrors MudBlazor `Bordered`.
    #[inline]
    pub fn bordered(mut self, b: bool) -> Self {
        self.bordered = b;
        self
    }

    /// Sets the anchor corner. Mirrors MudBlazor `Origin` (default `TopRight`).
    #[inline]
    pub fn origin(mut self, o: Origin) -> Self {
        self.origin = o;
        self
    }

    /// Sets the drop-shadow elevation (0–24; clamped). Mirrors MudBlazor
    /// `Elevation`.
    #[inline]
    pub fn elevation(mut self, e: u8) -> Self {
        self.elevation = e;
        self
    }

    /// Applies utility classes to the badge itself (separate from the wrapper
    /// classes set via [`classes`](Self::classes)). Mirrors MudBlazor
    /// `BadgeClass`.
    #[inline]
    pub fn badge_class(mut self, class: impl Into<String>) -> Self {
        self.badge_class = Some(class.into());
        self
    }

    /// Sets the ARIA label published on the badge's accessibility node.
    /// Mirrors MudBlazor `BadgeAriaLabel`.
    #[inline]
    pub fn badge_aria_label(mut self, label: impl Into<String>) -> Self {
        self.badge_aria_label = Some(label.into());
        self
    }

    /// Registers a click handler. The badge becomes clickable as soon as this
    /// is set, matching MudBlazor's `OnClick.HasDelegate` behaviour. Mirrors
    /// MudBlazor `OnClick`.
    #[inline]
    pub fn on_click(mut self, f: impl FnMut() + 'static) -> Self {
        self.on_click = Some(Box::new(f));
        self
    }

    /// Selects the utility-class resolver. Extension, not in MudBlazor.
    #[inline]
    pub fn flavor(mut self, f: StyleFlavor) -> Self {
        self.flavor = f;
        self
    }

    /// Adds utility classes to the **wrapper** frame. Mirrors MudBlazor
    /// `Class` (base `ComponentBase`).
    #[inline]
    pub fn classes<S: AsRef<str>>(mut self, classes: impl IntoIterator<Item = S>) -> Self {
        self.base = self.base.classes(classes);
        self
    }

    // ── Rendering ────────────────────────────────────────────────────────────

    /// Draws `child`, then overlays the badge at the configured origin corner.
    ///
    /// Returns a tuple of `(child_response, child_return_value, badge_clicked)`.
    /// `badge_clicked` is `false` whenever the badge is hidden, has no
    /// content, or has no [`on_click`](Self::on_click) handler.
    pub fn show<R>(
        mut self,
        ui: &mut Ui,
        child: impl FnOnce(&mut Ui) -> R,
    ) -> (Response, R, bool) {
        let theme = MudTheme::fetch(ui.ctx()).unwrap_or_default();
        let palette = theme.current_palette();

        // Resolve the wrapper CSS frame (applies to the child container).
        let lookup = |key: &str| palette.resolve(key).map(Color32::from);
        self.tw_state.dark = ui.visuals().dark_mode;
        let frame = match self.flavor {
            StyleFlavor::Bootstrap => self.bs_style.build_frame(&self.base, &lookup),
            StyleFlavor::Tailwind => self
                .tw_style
                .build_frame(&self.base, self.tw_state, REM_SIZE_PT, &lookup),
        };

        let mut badge_clicked = false;

        let outer = frame.show(ui, |ui| {
            // 1. Draw the child.
            let inner = ui.scope(child);
            let rect = inner.response.rect;

            // 2. Resolve whether the badge is drawn at all.
            let show_badge = self.visible
                && (self.dot || self.icon.is_some() || self.content.is_some());

            if show_badge {
                // 3. Resolve the badge style once.
                //
                // `self.badge_class.as_deref()` immutably borrows
                // `self.badge_class`; the `&mut self.bs_style` /
                // `&mut self.tw_style` borrows touch *different fields* of
                // `self`, so Rust's disjoint-field capture rules allow the
                // three borrows to coexist.
                let badge_class = self.badge_class.as_deref();
                let style = resolve_badge_style(
                    self.color,
                    &theme,
                    self.flavor,
                    badge_class,
                    &mut self.bs_style,
                    &mut self.tw_style,
                    self.tw_state,
                    self.elevation,
                    self.bordered,
                );

                // 4. Compute the badge rect for the active content kind.
                let (badge_rect, radius) = if self.dot {
                    let center = badge_center(
                        rect,
                        DOT_EXTENT,
                        DOT_EXTENT,
                        self.origin,
                        self.overlap,
                    );
                    (
                        Rect::from_center_size(center, vec2(DOT_EXTENT, DOT_EXTENT)),
                        (DOT_EXTENT * 0.5) as u8,
                    )
                } else if self.icon.is_some() {
                    let center = badge_center(
                        rect,
                        MIN_TEXT_EXTENT,
                        MIN_TEXT_EXTENT,
                        self.origin,
                        self.overlap,
                    );
                    (
                        Rect::from_center_size(
                            center,
                            vec2(MIN_TEXT_EXTENT, MIN_TEXT_EXTENT),
                        ),
                        (MIN_TEXT_EXTENT * 0.5) as u8,
                    )
                } else {
                    let text = format_count(
                        self.content.as_deref().unwrap_or(""),
                        self.max,
                    );
                    let galley = ui.painter().layout_no_wrap(
                        text,
                        egui::FontId::proportional(BADGE_FONT_SIZE),
                        style.ink,
                    );
                    let pad = if self.bordered {
                        BORDERED_TEXT_PADDING_X
                    } else {
                        TEXT_PADDING_X
                    };
                    let w = (galley.size().x + pad * 2.0).max(MIN_TEXT_EXTENT);
                    let h = BADGE_HEIGHT;
                    let center =
                        badge_center(rect, w, h, self.origin, self.overlap);
                    (
                        Rect::from_center_size(center, vec2(w, h)),
                        (h * 0.5) as u8,
                    )
                };

                // 5. Paint the badge.
                paint_badge(ui, badge_rect, radius, &style);

                if !self.dot {
                    if let Some(icon_svg) = self.icon.clone() {
                        let icon_rect = Rect::from_center_size(
                            badge_rect.center(),
                            vec2(BADGE_ICON_SIZE, BADGE_ICON_SIZE),
                        );
                        ui.put(
                            icon_rect,
                            MudIcon::new(icon_svg)
                                .size_px(BADGE_ICON_SIZE)
                                .custom_color(style.ink),
                        );
                    } else {
                        let text = format_count(
                            self.content.as_deref().unwrap_or(""),
                            self.max,
                        );
                        let mut text_ui = ui.new_child(
                            egui::UiBuilder::new()
                                .max_rect(badge_rect)
                                .layout(egui::Layout::centered_and_justified(
                                    egui::Direction::TopDown,
                                )),
                        );
                        text_ui.add(
                            MudText::new(text)
                                .custom_color(style.ink)
                                .font_size(BADGE_FONT_SIZE)
                                .bold(true)
                                .inline(true)
                                .no_wrap(true),
                        );
                    }
                }

                // 6. Hit-test the badge (does not reserve parent layout).
                let interactive = self.on_click.is_some();
                let sense = if interactive {
                    Sense::click()
                } else {
                    Sense::hover()
                };
                let hit_id = ui.id().with((
                    "mud_badge_hit",
                    badge_rect.min.x.to_bits(),
                    badge_rect.min.y.to_bits(),
                ));
                let badge_resp = ui.interact(badge_rect, hit_id, sense);

                if interactive {
                    if badge_resp.hovered() {
                        ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
                    }
                    if badge_resp.clicked() {
                        badge_clicked = true;
                        if let Some(cb) = self.on_click.as_mut() {
                            cb();
                        }
                    }
                }

                if let Some(aria) = &self.badge_aria_label {
                    badge_resp.widget_info(|| {
                        egui::WidgetInfo::labeled(egui::Role::Label, true, aria)
                    });
                }
            }

            inner
        });

        self.tw_state.hovered = outer.response.hovered();
        self.tw_state.active = outer.response.is_pointer_button_down_on();

        (outer.inner.response, outer.inner.inner, badge_clicked)
    }
}

/// Paints the badge background (shadow, mask, fill, optional stroke).
fn paint_badge(ui: &Ui, rect: Rect, radius: u8, style: &BadgeStyle) {
    let corner = CornerRadius::same(radius);

    if let Some(shadow) = style.shadow {
        ui.painter().add(shadow.as_shape(rect, corner));
    }

    // Surface mask so the badge separates from the child.
    let mask_corner = CornerRadius::same(radius.saturating_add(1));
    ui.painter().rect_filled(
        rect.expand(MASK_THICKNESS),
        mask_corner,
        style.ring,
    );

    // Fill.
    ui.painter().rect_filled(rect, corner, style.fill);

    // Outline.
    if let Some(stroke) = style.stroke {
        ui.painter()
            .rect_stroke(rect, corner, stroke, StrokeKind::Inside);
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{pos2, vec2, Rect};

    // --- anchor sides ------------------------------------------------------

    #[test]
    fn anchor_sides_covers_all_nine_origins() {
        assert_eq!(anchor_sides(Origin::TopLeft), (-1, -1));
        assert_eq!(anchor_sides(Origin::TopCenter), (0, -1));
        assert_eq!(anchor_sides(Origin::TopRight), (1, -1));
        assert_eq!(anchor_sides(Origin::CenterLeft), (-1, 0));
        assert_eq!(anchor_sides(Origin::Center), (0, 0));
        assert_eq!(anchor_sides(Origin::CenterRight), (1, 0));
        assert_eq!(anchor_sides(Origin::BottomLeft), (-1, 1));
        assert_eq!(anchor_sides(Origin::BottomCenter), (0, 1));
        assert_eq!(anchor_sides(Origin::BottomRight), (1, 1));
    }

    // --- badge centre ------------------------------------------------------

    fn child() -> Rect {
        Rect::from_min_size(pos2(100.0, 100.0), vec2(80.0, 80.0))
    }

    #[test]
    fn top_right_overlap_true_centers_on_corner() {
        let c = badge_center(child(), 20.0, 20.0, Origin::TopRight, true);
        assert_eq!(c, pos2(180.0, 100.0));
    }

    #[test]
    fn top_right_overlap_false_pushes_outside() {
        let c = badge_center(child(), 20.0, 20.0, Origin::TopRight, false);
        assert_eq!(c, pos2(190.0, 90.0));
    }

    #[test]
    fn top_left_overlap_false_pushes_outside() {
        let c = badge_center(child(), 30.0, 16.0, Origin::TopLeft, false);
        assert_eq!(c, pos2(85.0, 92.0));
    }

    #[test]
    fn bottom_right_overlap_false_pushes_outside() {
        let c = badge_center(child(), 20.0, 20.0, Origin::BottomRight, false);
        assert_eq!(c, pos2(190.0, 190.0));
    }

    #[test]
    fn center_is_child_center_regardless_of_overlap() {
        assert_eq!(
            badge_center(child(), 20.0, 20.0, Origin::Center, true),
            pos2(140.0, 140.0)
        );
        assert_eq!(
            badge_center(child(), 20.0, 20.0, Origin::Center, false),
            pos2(140.0, 140.0)
        );
    }

    #[test]
    fn top_center_centres_horizontally_and_anchors_top() {
        let c = badge_center(child(), 20.0, 20.0, Origin::TopCenter, true);
        assert_eq!(c, pos2(140.0, 100.0));
        let c = badge_center(child(), 20.0, 20.0, Origin::TopCenter, false);
        assert_eq!(c, pos2(140.0, 90.0));
    }

    #[test]
    fn wide_text_badge_is_pushed_by_half_its_own_width() {
        // A "99+" badge ~36 px wide pushed by half → x = 180 + 18 = 198.
        let c = badge_center(child(), 36.0, 20.0, Origin::TopRight, false);
        assert_eq!(c, pos2(198.0, 90.0));
    }

    // --- content formatting ------------------------------------------------

    #[test]
    fn format_count_passes_through_non_numeric() {
        assert_eq!(format_count("live", 99), "live");
        assert_eq!(format_count("", 99), "");
        assert_eq!(format_count("3.14", 99), "3.14");
    }

    #[test]
    fn format_count_caps_at_max() {
        assert_eq!(format_count("5", 99), "5");
        assert_eq!(format_count("99", 99), "99");
        assert_eq!(format_count("100", 99), "99+");
        assert_eq!(format_count("100000", 99), "99+");
    }

    #[test]
    fn format_count_negative_is_not_capped() {
        assert_eq!(format_count("-1", 99), "-1");
        assert_eq!(format_count("-1", 0), "-1");
    }

    #[test]
    fn format_count_zero_max_caps_any_positive() {
        assert_eq!(format_count("1", 0), "0+");
        assert_eq!(format_count("0", 0), "0");
    }

    // --- defaults ----------------------------------------------------------

    #[test]
    fn defaults_match_mudblazor() {
        let b = MudBadge::new();
        assert!(matches!(b.color, Color::Default));
        assert!(matches!(b.origin, Origin::TopRight));
        assert!(!b.dot);
        assert!(!b.overlap);
        assert!(b.visible);
        assert!(!b.bordered);
        assert_eq!(b.max, 99);
        assert_eq!(b.elevation, 0);
        assert!(b.content.is_none());
        assert!(b.icon.is_none());
        assert!(b.badge_class.is_none());
        assert!(b.badge_aria_label.is_none());
        assert!(b.on_click.is_none());
    }

    #[test]
    fn builders_set_every_field() {
        let b = MudBadge::new()
            .color(Color::Error)
            .dot(true)
            .content("5")
            .icon("M0 0")
            .max(9)
            .overlap(true)
            .visible(false)
            .bordered(true)
            .origin(Origin::BottomLeft)
            .elevation(4)
            .badge_class("bg-error text-white")
            .badge_aria_label("3 new messages")
            .on_click(|| {})
            .classes(["a"]);
        assert!(matches!(b.color, Color::Error));
        assert!(b.dot && b.overlap && b.bordered);
        assert!(!b.visible);
        assert_eq!(b.max, 9);
        assert_eq!(b.elevation, 4);
        assert!(matches!(b.origin, Origin::BottomLeft));
        assert_eq!(b.content.as_deref(), Some("5"));
        assert_eq!(b.icon.as_deref(), Some("M0 0"));
        assert_eq!(b.badge_class.as_deref(), Some("bg-error text-white"));
        assert_eq!(b.badge_aria_label.as_deref(), Some("3 new messages"));
        assert!(b.on_click.is_some());
    }

    #[test]
    fn count_sets_integer_content() {
        let b = MudBadge::new().count(42);
        assert_eq!(b.content.as_deref(), Some("42"));
    }

    #[test]
    fn max_clamps_negative_to_zero() {
        let b = MudBadge::new().max(-5);
        assert_eq!(b.max, 0);
    }

    // --- badge style -------------------------------------------------------

    fn style_for(b: &MudBadge, theme: &MudTheme) -> BadgeStyle {
        // Fresh resolvers are fine for the tests: they exercise the color +
        // shadow + bordered logic, not user-configured class caches.
        let mut bs = BsStyle::default();
        let mut tw = TwStyle::default();
        resolve_badge_style(
            b.color,
            theme,
            b.flavor,
            b.badge_class.as_deref(),
            &mut bs,
            &mut tw,
            b.tw_state,
            b.elevation,
            b.bordered,
        )
    }

    #[test]
    fn default_color_resolves_to_primary() {
        let theme = MudTheme::default();
        let palette = theme.current_palette();
        let b = MudBadge::new();
        let s = style_for(&b, &theme);
        let primary: Color32 = palette.primary.into();
        assert_eq!(s.fill, primary);
        assert_eq!(s.ink, palette.primary_contrast_text.into());
        assert!(s.shadow.is_none());
        assert!(s.stroke.is_none());
    }

    #[test]
    fn bordered_adds_surface_stroke() {
        let theme = MudTheme::default();
        let palette = theme.current_palette();
        let b = MudBadge::new().bordered(true);
        let s = style_for(&b, &theme);
        let stroke = s.stroke.expect("bordered must add a stroke");
        assert_eq!(stroke.width, BORDER_RING_THICKNESS);
        assert_eq!(stroke.color, palette.surface.into());
    }

    #[test]
    fn elevation_adds_shadow() {
        let theme = MudTheme::default();
        let b = MudBadge::new().elevation(4);
        let s = style_for(&b, &theme);
        assert!(s.shadow.is_some());
        let b0 = MudBadge::new();
        assert!(style_for(&b0, &theme).shadow.is_none());
    }

    #[test]
    fn explicit_color_overrides_primary_fill() {
        let theme = MudTheme::default();
        let palette = theme.current_palette();
        let b = MudBadge::new().color(Color::Error);
        let s = style_for(&b, &theme);
        let error: Color32 = palette.error.into();
        assert_eq!(s.fill, error);
    }
}