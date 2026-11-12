//! A horizontal or vertical rule with optional inset and middle margins.
//!
//! Port of `MudDivider.cs` / `_divider.scss` from MudBlazor. Renders a line
//! using the theme's palette colors, optionally with a text label in the middle
//! (horizontal dividers only).
//!
//! # MudBlazor API parity
//!
//! The real `MudDivider` exposes exactly five component parameters
//! (`Absolute`, `DividerType`, `FlexItem`, `Light`, `Vertical`) plus the
//! `MudComponentBase` members (`Class`, `Style`, `Tag`, `UserAttributes`).
//! This port implements every one that has a meaning in an immediate-mode UI
//! and does **not** invent bindings, events or variants that MudBlazor does not
//! have (a divider has no value, no click, no focus and no elevation).
//!
//! | MudBlazor        | Rust                         | Notes                                           |
//! |------------------|------------------------------|-------------------------------------------------|
//! | `Absolute`       | [`MudDivider::absolute`]     | Pinned to the bottom (or right) of the parent.  |
//! | `DividerType`    | [`MudDivider::divider_type`] | `FullWidth`, `Inset` (72 pt), `Middle` (16 pt). |
//! | `FlexItem`       | [`MudDivider::flex_item`]    | Vertical dividers stretch to the available height. |
//! | `Light`          | [`MudDivider::light`]        | Lighter line color.                             |
//! | `Vertical`       | [`MudDivider::vertical`]     | Vertical line.                                  |
//! | `Class`          | [`MudDivider::classes`]      | Utility classes (Bootstrap / Tailwind flavor).  |
//! | `Tag`            | [`MudDivider::tag`]          | Arbitrary user data, read with [`MudDivider::tag_ref`]. |
//! | `Style`, `UserAttributes` | — | CSS strings / HTML attribute splatting do not exist in egui. |
//!
//! Extensions that already existed in this port and are kept: `color`,
//! `text` (centered label), `thickness`, `height`, `flavor`.
//!
//! # Architecture
//!
//! Every decision that can be made from numbers is made by a small pure
//! function (margins, segment widths, extents, sanitizing, color choice,
//! absolute geometry), so it is unit tested without a live `egui::Context`.
//! The `show` / `render_*` methods are thin glue around them. A handful of
//! smoke tests additionally run real frames against a headless context.
//!
//! # Example
//!
//! ```ignore
//! use mud_egui_core::enums::divider_type::DividerType;
//!
//! // Plain rule.
//! MudDivider::new().show(ui);
//!
//! // Inset rule between list items, lighter than the default.
//! MudDivider::new().divider_type(DividerType::Inset).light(true).show(ui);
//!
//! // Labelled rule.
//! ui.add(MudDivider::new().text("OR"));
//!
//! // Vertical rule between toolbar buttons that stretches with its row.
//! ui.horizontal(|ui| {
//!     ui.button("Left");
//!     MudDivider::new().vertical(true).flex_item(true).show(ui);
//!     ui.button("Right");
//! });
//! ```

use std::any::Any;
use std::sync::Arc;

use egui::{
    pos2, vec2, Color32, FontId, Frame, Margin, Pos2, Rect, Response, Sense, Stroke, Ui, Widget,
    WidgetInfo, WidgetType,
};
use mud_egui_bootstrap::bs_style::BsStyle;
use mud_egui_core::base::component_base::ComponentBase;
use mud_egui_core::enums::color::Color;
use mud_egui_core::enums::divider_type::DividerType;
use mud_egui_core::enums::StyleFlavor;
use mud_egui_tailwind::tw_style::{TwState, TwStyle};
use mud_egui_theming::theme::{to_egui_family, MudTheme};

/// Base rem size (in points) passed to the Tailwind resolver.
const REM_SIZE_PT: f32 = 14.0;

/// Start-side inset for [`DividerType::Inset`]. Matches `.mud-divider-inset`
/// (`margin-left: 72px`) in MudBlazor's `_divider.scss`.
const INSET_START: i8 = 72;
/// Left and right margin for [`DividerType::Middle`]. Matches
/// `.mud-divider-middle` (`margin: 0 16px`) in MudBlazor's `_divider.scss`.
const MIDDLE_SIDE: i8 = 16;
/// Default height of a non-flex vertical divider in logical pixels.
const DEFAULT_VERTICAL_HEIGHT: f32 = 24.0;
/// Gap spacing between the center text label and the surrounding line segments.
const TEXT_GAP: f32 = 8.0;
/// Default fallback line thickness in points.
const DEFAULT_THICKNESS: f32 = 1.0;
/// Smallest line thickness that still renders visibly; thinner values are raised to it.
const MIN_THICKNESS: f32 = 0.5;
/// Label font size used when the theme's `body2` size is missing or invalid.
const FALLBACK_LABEL_FONT_SIZE: f32 = 14.0;
/// Multiplier applied to the line color when `light` is set.
///
/// MudBlazor reads a dedicated `DividerLight` palette entry. This port derives
/// the light color from the divider color instead, so it never depends on a
/// palette field that may not exist, and it always renders *lighter* than the
/// default (see MudBlazor issue #5669, where the stock palette values make the
/// "light" divider look darker).
const LIGHT_FACTOR: f32 = 0.5;

// =============================================================================
// Private pure helpers
// =============================================================================

#[inline]
fn sanitize_thickness(thickness: f32) -> f32 {
    if thickness.is_finite() {
        thickness.max(MIN_THICKNESS)
    } else {
        DEFAULT_THICKNESS
    }
}

#[inline]
fn sanitize_height(height: f32) -> f32 {
    if height.is_finite() {
        height.max(0.0)
    } else {
        DEFAULT_VERTICAL_HEIGHT
    }
}

#[inline]
fn sanitize_width(width: f32) -> f32 {
    if width.is_finite() {
        width.max(0.0)
    } else {
        0.0
    }
}

#[inline]
fn sanitize_font_size(size: f32) -> f32 {
    if size.is_finite() && size > 0.0 {
        size
    } else {
        FALLBACK_LABEL_FONT_SIZE
    }
}

/// Chooses the final line color.
///
/// * `Color::Default` uses the palette's divider color.
/// * Any other color uses the resolved palette color, falling back to the
///   divider color if the palette cannot resolve it.
/// * `light` then fades the result.
#[inline]
fn resolve_line_color(
    is_default: bool,
    light: bool,
    divider: Color32,
    resolved: Option<Color32>,
) -> Color32 {
    let base = if is_default {
        divider
    } else {
        resolved.unwrap_or(divider)
    };
    if light {
        base.gamma_multiply(LIGHT_FACTOR)
    } else {
        base
    }
}

#[inline]
fn label_row_height(text_height: f32, thickness: f32) -> f32 {
    sanitize_width(text_height).max(thickness)
}

/// Shrinks `rect` by `margin` on every side, never producing a negative size.
#[inline]
fn inset_rect(rect: Rect, margin: Margin) -> Rect {
    let min = pos2(
        rect.left() + f32::from(margin.left),
        rect.top() + f32::from(margin.top),
    );
    let max = pos2(
        (rect.right() - f32::from(margin.right)).max(min.x),
        (rect.bottom() - f32::from(margin.bottom)).max(min.y),
    );
    Rect::from_min_max(min, max)
}

/// End points of an absolutely positioned line inside `bounds`.
///
/// Mirrors `.mud-divider-absolute` (`left: 0; width: 100%; bottom: 0`): a
/// horizontal line sits on the bottom edge, a vertical one on the right edge
/// (the side that carries the `border-right` in the vertical variant). The line
/// is inset by half its thickness so the stroke stays inside `bounds`.
#[inline]
fn absolute_endpoints(bounds: Rect, vertical: bool, thickness: f32) -> [Pos2; 2] {
    let half = thickness * 0.5;
    if vertical {
        let x = bounds.right() - half;
        [pos2(x, bounds.top()), pos2(x, bounds.bottom())]
    } else {
        let y = bounds.bottom() - half;
        [pos2(bounds.left(), y), pos2(bounds.right(), y)]
    }
}

// =============================================================================
// Component
// =============================================================================

/// A thin line that groups content in lists and layouts.
///
/// See the [module documentation](self) for the MudBlazor parity table.
///
/// # Example
///
/// ```ignore
/// ui.add(MudDivider::new().divider_type(DividerType::Middle).text("Section"));
/// ```
#[must_use = "MudDivider does nothing until you call `.show(ui)` or `ui.add(..)`"]
pub struct MudDivider {
    /// If `true`, draws a vertical line. If `false`, draws a horizontal line.
    ///
    /// MudBlazor: `Vertical`. Default: `false`.
    pub vertical: bool,

    /// Controls the layout margins (`FullWidth`, `Inset`, `Middle`).
    ///
    /// MudBlazor: `DividerType`. Default: [`DividerType::FullWidth`].
    /// As in MudBlazor, the margin classes also apply to vertical dividers
    /// when the type is not `FullWidth`.
    pub divider_type: DividerType,

    /// If `true` and `vertical` is true, the divider stretches to fill the
    /// available container height instead of using [`MudDivider::height`].
    ///
    /// MudBlazor: `FlexItem`. Default: `false`.
    pub flex_item: bool,

    /// If `true`, the divider uses a lighter color.
    ///
    /// MudBlazor: `Light`. Default: `false`.
    pub light: bool,

    /// If `true`, the divider is pinned to the bottom edge (horizontal) or
    /// right edge (vertical) of the parent's rectangle and takes no layout
    /// space. Absolute dividers ignore the label and the utility-class frame.
    ///
    /// MudBlazor: `Absolute`. Default: `false`.
    pub absolute: bool,

    /// The semantic color role of the line. *Extension.*
    pub color: Color,

    /// Optional text label rendered in the dead center of a horizontal line.
    /// *Extension.* Stored as `Arc<str>` so cloning a divider spec is cheap.
    pub child_content: Option<Arc<str>>,

    /// Line thickness in points. *Extension.*
    pub thickness: f32,

    /// Fixed height for a non-flex vertical divider. *Extension.*
    pub height: f32,

    /// Shared component state (CSS-like utility classes, elevation, etc.).
    /// MudBlazor: `Class`.
    pub base: ComponentBase,

    /// Which utility-class system to resolve against.
    pub flavor: StyleFlavor,

    /// Bootstrap resolver cache.
    pub bs_style: BsStyle,

    /// Tailwind resolver cache.
    pub tw_style: TwStyle,

    /// State fed to the Tailwind resolver.
    pub tw_state: TwState,

    /// Arbitrary user data. MudBlazor: `Tag`. Read it with [`MudDivider::tag_ref`].
    tag: Option<Arc<dyn Any + Send + Sync>>,
}

impl Default for MudDivider {
    fn default() -> Self {
        Self {
            vertical: false,
            divider_type: DividerType::FullWidth,
            flex_item: false,
            light: false,
            absolute: false,
            color: Color::Default,
            child_content: None,
            thickness: DEFAULT_THICKNESS,
            height: DEFAULT_VERTICAL_HEIGHT,
            base: ComponentBase::new(),
            flavor: StyleFlavor::default(),
            bs_style: BsStyle::default(),
            tw_style: TwStyle::default(),
            tw_state: TwState::default(),
            tag: None,
        }
    }
}

impl MudDivider {
    /// Creates a full-width horizontal divider with default settings.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets whether the divider is vertical.
    #[inline]
    pub fn vertical(mut self, vertical: bool) -> Self {
        self.vertical = vertical;
        self
    }

    /// Sets the margin style (`FullWidth`, `Inset`, `Middle`).
    #[inline]
    pub fn divider_type(mut self, divider_type: DividerType) -> Self {
        self.divider_type = divider_type;
        self
    }

    /// Makes a vertical divider fill the available height.
    #[inline]
    pub fn flex_item(mut self, flex_item: bool) -> Self {
        self.flex_item = flex_item;
        self
    }

    /// Uses a lighter line color.
    #[inline]
    pub fn light(mut self, light: bool) -> Self {
        self.light = light;
        self
    }

    /// Pins the divider to the bottom (or right) edge of the parent rectangle.
    #[inline]
    pub fn absolute(mut self, absolute: bool) -> Self {
        self.absolute = absolute;
        self
    }

    /// Sets the semantic color role of the line.
    #[inline]
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Sets the centered label. An empty string clears the label.
    #[inline]
    pub fn text(mut self, text: impl Into<Arc<str>>) -> Self {
        let text: Arc<str> = text.into();
        self.child_content = if text.is_empty() { None } else { Some(text) };
        self
    }

    /// Sets the line thickness in points (sanitized, minimum `0.5`).
    #[inline]
    pub fn thickness(mut self, thickness: f32) -> Self {
        self.thickness = sanitize_thickness(thickness);
        self
    }

    /// Sets the fixed height of a non-flex vertical divider.
    #[inline]
    pub fn height(mut self, height: f32) -> Self {
        self.height = sanitize_height(height);
        self
    }

    /// Selects the utility-class resolver.
    #[inline]
    pub fn flavor(mut self, flavor: StyleFlavor) -> Self {
        self.flavor = flavor;
        self
    }

    /// Adds utility classes.
    #[inline]
    pub fn classes<S: AsRef<str>>(mut self, classes: impl IntoIterator<Item = S>) -> Self {
        self.base = self.base.classes(classes);
        self
    }

    /// Attaches arbitrary user data to the component.
    ///
    /// ```ignore
    /// let d = MudDivider::new().tag(42_u32);
    /// assert_eq!(d.tag_ref::<u32>(), Some(&42));
    /// ```
    #[inline]
    pub fn tag<T: Any + Send + Sync>(mut self, tag: T) -> Self {
        self.tag = Some(Arc::new(tag));
        self
    }

    /// Returns the attached user data if it exists and has type `T`.
    #[inline]
    pub fn tag_ref<T: Any>(&self) -> Option<&T> {
        self.tag.as_deref().and_then(|t| t.downcast_ref::<T>())
    }

    // =========================================================================
    // Pure helpers
    // =========================================================================

    /// Margins for a divider type in a left-to-right layout.
    pub(crate) fn margins_for(divider_type: DividerType) -> Margin {
        Self::margins_for_dir(divider_type, false)
    }

    /// Margins for a divider type, mirroring the inset for right-to-left layouts
    /// (MudBlazor uses `margin-inline-start`).
    pub(crate) fn margins_for_dir(divider_type: DividerType, rtl: bool) -> Margin {
        let mut margin = Margin::ZERO;
        match divider_type {
            DividerType::FullWidth => {}
            DividerType::Inset => {
                if rtl {
                    margin.right = INSET_START;
                } else {
                    margin.left = INSET_START;
                }
            }
            DividerType::Middle => {
                margin.left = MIDDLE_SIDE;
                margin.right = MIDDLE_SIDE;
            }
        }
        margin
    }

    #[inline]
    pub(crate) fn line_segment_width(total: f32, text_width: f32, gap: f32) -> f32 {
        ((total - text_width - gap * 2.0) / 2.0).max(0.0)
    }

    #[inline]
    pub(crate) fn vertical_extent(flex_item: bool, available: f32, fixed: f32) -> f32 {
        if flex_item && available.is_finite() && available > 0.0 {
            available
        } else {
            sanitize_height(fixed)
        }
    }

    #[inline]
    fn sane_thickness(&self) -> f32 {
        sanitize_thickness(self.thickness)
    }

    // =========================================================================
    // Rendering
    // =========================================================================

    /// Renders the divider and returns the response of the drawn line.
    pub fn show(mut self, ui: &mut Ui) -> Response {
        let theme = MudTheme::fetch(ui.ctx()).unwrap_or_default();
        let palette = theme.current_palette();

        let divider_color: Color32 = palette.divider.into();
        let is_default = self.color == Color::Default;
        let resolved = if is_default {
            None
        } else {
            self.color.resolve(palette)
        };
        let color = resolve_line_color(is_default, self.light, divider_color, resolved);
        let stroke = Stroke::new(self.sane_thickness(), color);

        // Absolute dividers take no layout space, so they bypass the frame.
        if self.absolute {
            return self.render_absolute(ui, stroke);
        }

        // Resolve CSS utility frame.
        let lookup = |key: &str| palette.resolve(key).map(Color32::from);
        self.tw_state.dark = ui.visuals().dark_mode;

        let frame = match self.flavor {
            StyleFlavor::Bootstrap => self.bs_style.build_frame(&self.base, &lookup),
            StyleFlavor::Tailwind => {
                self.tw_style
                    .build_frame(&self.base, self.tw_state, REM_SIZE_PT, &lookup)
            }
        };

        frame
            .show(ui, |ui| self.render_content(ui, stroke, &theme))
            .inner
    }

    fn render_content(&self, ui: &mut Ui, stroke: Stroke, theme: &MudTheme) -> Response {
        let rtl = ui.layout().prefer_right_to_left();
        let margin = Self::margins_for_dir(self.divider_type, rtl);

        let draw = |ui: &mut Ui| -> Response {
            if self.vertical {
                self.render_vertical(ui, stroke)
            } else {
                self.render_horizontal(ui, stroke, theme)
            }
        };

        if margin == Margin::ZERO {
            draw(ui)
        } else {
            Frame::new().outer_margin(margin).show(ui, draw).inner
        }
    }

    fn render_absolute(&self, ui: &mut Ui, stroke: Stroke) -> Response {
        let rtl = ui.layout().prefer_right_to_left();
        let margin = Self::margins_for_dir(self.divider_type, rtl);
        let bounds = inset_rect(ui.max_rect(), margin);
        let [a, b] = absolute_endpoints(bounds, self.vertical, stroke.width);
        let rect = Rect::from_two_pos(a, b).expand(stroke.width * 0.5);

        let id = ui.id().with(("mud_divider_absolute", self.vertical));
        let response = ui.interact(rect, id, Sense::hover());
        if ui.is_rect_visible(rect) {
            ui.painter().line_segment([a, b], stroke);
        }
        response
    }

    fn render_vertical(&self, ui: &mut Ui, stroke: Stroke) -> Response {
        let height = Self::vertical_extent(self.flex_item, ui.available_height(), self.height);
        let (rect, response) = ui.allocate_exact_size(vec2(stroke.width, height), Sense::hover());

        if ui.is_rect_visible(rect) {
            ui.painter()
                .line_segment([rect.center_top(), rect.center_bottom()], stroke);
        }
        response
    }

    fn render_horizontal(&self, ui: &mut Ui, stroke: Stroke, theme: &MudTheme) -> Response {
        let width = sanitize_width(ui.available_width());
        match self.child_content.as_deref() {
            Some(text) => Self::draw_labelled(ui, text, Self::label_font(theme), width, stroke),
            None => {
                let (rect, response) =
                    ui.allocate_exact_size(vec2(width, stroke.width), Sense::hover());
                if ui.is_rect_visible(rect) {
                    ui.painter()
                        .line_segment([rect.left_center(), rect.right_center()], stroke);
                }
                response
            }
        }
    }

    #[inline]
    fn label_font(theme: &MudTheme) -> FontId {
        let variant = &theme.typography.body2;
        FontId::new(
            sanitize_font_size(variant.font_size),
            to_egui_family(variant.font_family),
        )
    }

    fn draw_labelled(ui: &mut Ui, text: &str, font: FontId, width: f32, stroke: Stroke) -> Response {
        let text_color = ui.visuals().weak_text_color();
        let galley = ui
            .painter()
            .layout_no_wrap(text.to_owned(), font, text_color);

        let row_height = label_row_height(galley.size().y, stroke.width);
        let (rect, response) = ui.allocate_exact_size(vec2(width, row_height), Sense::hover());

        let enabled = ui.is_enabled();
        response.widget_info(|| WidgetInfo::labeled(WidgetType::Label, enabled, text));

        if ui.is_rect_visible(rect) {
            let line_w = Self::line_segment_width(width, galley.size().x, TEXT_GAP);
            let y = rect.center().y;
            let painter = ui.painter();

            painter.line_segment([pos2(rect.left(), y), pos2(rect.left() + line_w, y)], stroke);
            painter.galley(pos2(rect.left() + line_w + TEXT_GAP, rect.top()), galley, text_color);
            painter.line_segment([pos2(rect.right() - line_w, y), pos2(rect.right(), y)], stroke);
        }
        response
    }
}

impl Widget for MudDivider {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui)
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f32 = 0.01;

    /// Runs `f` inside a real egui frame on a headless 400x300 context.
    fn run_in_ui<R>(f: impl FnOnce(&mut Ui) -> R) -> R {
        let ctx = egui::Context::default();
        let input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(400.0, 300.0))),
            ..Default::default()
        };
        let mut f = Some(f);
        let mut out = None;

        // Use `run_ui` which yields a `&mut Ui` for the top-level viewport
        let _ = ctx.run_ui(input, |ui| {
            // Use `show_inside` when you already have a `Ui`
            egui::CentralPanel::default().show_inside(ui, |ui| {
                if let Some(f) = f.take() {
                    out = Some(f(ui));
                }
            });
        });

        out.expect("the frame closure must run exactly once")
    }

    // ---- sanitizers ---------------------------------------------------------

    #[test]
    fn thickness_is_clamped_and_nan_safe() {
        assert!((sanitize_thickness(0.1) - MIN_THICKNESS).abs() < EPS);
        assert!((sanitize_thickness(3.0) - 3.0).abs() < EPS);
        assert!((sanitize_thickness(f32::NAN) - DEFAULT_THICKNESS).abs() < EPS);
        assert!((sanitize_thickness(f32::INFINITY) - DEFAULT_THICKNESS).abs() < EPS);
    }

    #[test]
    fn height_and_width_are_sanitized() {
        assert_eq!(sanitize_height(-5.0), 0.0);
        assert_eq!(sanitize_height(f32::NAN), DEFAULT_VERTICAL_HEIGHT);
        assert_eq!(sanitize_width(-1.0), 0.0);
        assert_eq!(sanitize_width(f32::NAN), 0.0);
        assert_eq!(sanitize_width(12.0), 12.0);
    }

    #[test]
    fn font_size_falls_back() {
        assert_eq!(sanitize_font_size(0.0), FALLBACK_LABEL_FONT_SIZE);
        assert_eq!(sanitize_font_size(-3.0), FALLBACK_LABEL_FONT_SIZE);
        assert_eq!(sanitize_font_size(f32::NAN), FALLBACK_LABEL_FONT_SIZE);
        assert_eq!(sanitize_font_size(16.0), 16.0);
    }

    // ---- color --------------------------------------------------------------

    #[test]
    fn default_color_uses_divider_color() {
        let divider = Color32::from_rgb(10, 20, 30);
        let other = Color32::from_rgb(200, 0, 0);
        assert_eq!(resolve_line_color(true, false, divider, Some(other)), divider);
    }

    #[test]
    fn semantic_color_uses_resolved_or_falls_back() {
        let divider = Color32::from_rgb(10, 20, 30);
        let other = Color32::from_rgb(200, 0, 0);
        assert_eq!(resolve_line_color(false, false, divider, Some(other)), other);
        assert_eq!(resolve_line_color(false, false, divider, None), divider);
    }

    #[test]
    fn light_is_never_more_opaque_than_normal() {
        let divider = Color32::from_rgba_unmultiplied(0, 0, 0, 120);
        let normal = resolve_line_color(true, false, divider, None);
        let light = resolve_line_color(true, true, divider, None);
        assert!(light.a() < normal.a());
    }

    // ---- margins ------------------------------------------------------------

    #[test]
    fn full_width_has_no_margin() {
        assert_eq!(MudDivider::margins_for(DividerType::FullWidth), Margin::ZERO);
    }

    #[test]
    fn inset_matches_mudblazor_72() {
        let m = MudDivider::margins_for(DividerType::Inset);
        assert_eq!(m.left, 72);
        assert_eq!((m.right, m.top, m.bottom), (0, 0, 0));
    }

    #[test]
    fn inset_mirrors_in_rtl() {
        let m = MudDivider::margins_for_dir(DividerType::Inset, true);
        assert_eq!(m.right, 72);
        assert_eq!(m.left, 0);
    }

    #[test]
    fn middle_matches_mudblazor_16_sides_only() {
        let m = MudDivider::margins_for(DividerType::Middle);
        assert_eq!((m.left, m.right), (16, 16));
        assert_eq!((m.top, m.bottom), (0, 0));
    }

    // ---- geometry -----------------------------------------------------------

    #[test]
    fn line_segment_width_never_negative() {
        assert_eq!(MudDivider::line_segment_width(10.0, 100.0, 8.0), 0.0);
        assert!((MudDivider::line_segment_width(200.0, 40.0, 8.0) - 72.0).abs() < EPS);
    }

    #[test]
    fn vertical_extent_flex_vs_fixed() {
        assert_eq!(MudDivider::vertical_extent(true, 120.0, 24.0), 120.0);
        assert_eq!(MudDivider::vertical_extent(true, 0.0, 24.0), 24.0);
        assert_eq!(MudDivider::vertical_extent(true, f32::INFINITY, 24.0), 24.0);
        assert_eq!(MudDivider::vertical_extent(false, 120.0, 24.0), 24.0);
        assert_eq!(MudDivider::vertical_extent(false, 120.0, -4.0), 0.0);
    }

    #[test]
    fn label_row_is_at_least_the_thickness() {
        assert_eq!(label_row_height(0.0, 2.0), 2.0);
        assert_eq!(label_row_height(18.0, 2.0), 18.0);
        assert_eq!(label_row_height(f32::NAN, 2.0), 2.0);
    }

    #[test]
    fn inset_rect_shrinks_and_never_inverts() {
        let r = Rect::from_min_size(pos2(0.0, 0.0), vec2(100.0, 50.0));
        let m = Margin { left: 10, right: 20, top: 5, bottom: 5 };
        let s = inset_rect(r, m);
        assert_eq!((s.left(), s.right(), s.top(), s.bottom()), (10.0, 80.0, 5.0, 45.0));

        let tiny = Rect::from_min_size(pos2(0.0, 0.0), vec2(4.0, 4.0));
        let huge = Margin { left: 100, right: 100, top: 100, bottom: 100 };
        let s = inset_rect(tiny, huge);
        assert!(s.width() >= 0.0 && s.height() >= 0.0);
    }

    #[test]
    fn absolute_endpoints_hug_bottom_or_right_edge() {
        let b = Rect::from_min_size(pos2(10.0, 20.0), vec2(100.0, 50.0));
        let [p, q] = absolute_endpoints(b, false, 2.0);
        assert_eq!((p.x, q.x), (10.0, 110.0));
        assert_eq!((p.y, q.y), (69.0, 69.0));

        let [p, q] = absolute_endpoints(b, true, 2.0);
        assert_eq!((p.x, q.x), (109.0, 109.0));
        assert_eq!((p.y, q.y), (20.0, 70.0));
    }

    // ---- builder ------------------------------------------------------------

    #[test]
    fn defaults_match_mudblazor() {
        let d = MudDivider::new();
        assert!(!d.vertical && !d.flex_item && !d.light && !d.absolute);
        assert_eq!(d.divider_type, DividerType::FullWidth);
        assert_eq!(d.color, Color::Default);
        assert!(d.child_content.is_none());
        assert!((d.thickness - DEFAULT_THICKNESS).abs() < EPS);
        assert!((d.height - DEFAULT_VERTICAL_HEIGHT).abs() < EPS);
    }

    #[test]
    fn builder_sets_every_mudblazor_parameter() {
        let d = MudDivider::new()
            .vertical(true)
            .flex_item(true)
            .light(true)
            .absolute(true)
            .divider_type(DividerType::Middle);
        assert!(d.vertical && d.flex_item && d.light && d.absolute);
        assert_eq!(d.divider_type, DividerType::Middle);
    }

    #[test]
    fn builder_sanitizes_numbers() {
        let d = MudDivider::new().thickness(f32::NAN).height(-9.0);
        assert!((d.thickness - DEFAULT_THICKNESS).abs() < EPS);
        assert_eq!(d.height, 0.0);
    }

    #[test]
    fn empty_text_clears_the_label() {
        assert!(MudDivider::new().text("").child_content.is_none());
        let d = MudDivider::new().text("OR");
        assert_eq!(d.child_content.as_deref(), Some("OR"));
        assert!(d.text(String::new()).child_content.is_none());
    }

    #[test]
    fn tag_round_trips_by_type() {
        let d = MudDivider::new().tag(42_u32);
        assert_eq!(d.tag_ref::<u32>(), Some(&42));
        assert_eq!(d.tag_ref::<String>(), None);
        assert_eq!(MudDivider::new().tag_ref::<u32>(), None);
    }

    // ---- rendering (headless frames) ---------------------------------------

    #[test]
    fn horizontal_divider_spans_width_and_is_thin() {
        let r = run_in_ui(|ui| MudDivider::new().thickness(2.0).show(ui));
        assert!(r.rect.width() > 100.0);
        assert!((r.rect.height() - 2.0).abs() < EPS);
    }

    #[test]
    fn vertical_divider_uses_fixed_height() {
        let r = run_in_ui(|ui| MudDivider::new().vertical(true).height(40.0).show(ui));
        assert!((r.rect.height() - 40.0).abs() < EPS);
        assert!((r.rect.width() - DEFAULT_THICKNESS).abs() < EPS);
    }

    #[test]
    fn vertical_flex_divider_fills_available_height() {
        let r = run_in_ui(|ui| {
            MudDivider::new()
                .vertical(true)
                .flex_item(true)
                .height(10.0)
                .show(ui)
        });
        assert!(r.rect.height() > 100.0);
    }

    #[test]
    fn labelled_divider_is_taller_than_the_line() {
        let r = run_in_ui(|ui| MudDivider::new().text("OR").show(ui));
        assert!(r.rect.height() > DEFAULT_THICKNESS);
        assert!(r.rect.width() > 100.0);
    }

    #[test]
    fn inset_divider_starts_after_the_inset() {
        let (left, r) = run_in_ui(|ui| {
            let left = ui.max_rect().left();
            (left, MudDivider::new().divider_type(DividerType::Inset).show(ui))
        });
        assert!(r.rect.left() >= left + 71.0);
    }

    #[test]
    fn middle_divider_is_inset_on_both_sides() {
        let (bounds, r) = run_in_ui(|ui| {
            let bounds = ui.max_rect();
            (bounds, MudDivider::new().divider_type(DividerType::Middle).show(ui))
        });
        assert!(r.rect.left() >= bounds.left() + 15.0);
        assert!(r.rect.right() <= bounds.right() - 15.0);
    }

    #[test]
    fn absolute_divider_takes_no_layout_space() {
        let (before, after, r) = run_in_ui(|ui| {
            let before = ui.min_rect();
            let r = MudDivider::new().absolute(true).show(ui);
            (before, ui.min_rect(), r)
        });
        assert_eq!(before, after);
        assert!(r.rect.width() > 100.0);
    }

    #[test]
    fn widget_trait_renders_like_show() {
        let r = run_in_ui(|ui| ui.add(MudDivider::new()));
        assert!(r.rect.width() > 0.0);
    }
}