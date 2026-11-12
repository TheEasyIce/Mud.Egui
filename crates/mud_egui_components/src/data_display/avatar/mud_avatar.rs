//! `MudAvatar` — a small circular (or rounded-square) badge for a user,
//! icon, or initials.
//!
//! Port of `MudAvatar.razor.cs` (MudBlazor 8.13.0). Public parameter surface
//! mirrors the Blazor component:
//!
//! | MudBlazor parameter | Rust method | Default |
//! |---------------------|-------------|---------|
//! | `Svg`               | [`MudAvatar::svg`] | none |
//! | `Image`             | [`MudAvatar::image`] | none |
//! | `Alt`               | *not exposed* (no clean `WidgetInfo` mapping) | — |
//! | `ChildContent`      | [`MudAvatar::text`] / [`MudAvatar::child`] | none |
//! | `Variant`           | [`MudAvatar::variant`] | [`Variant::Filled`] |
//! | `Color`             | [`MudAvatar::color`] | [`Color::Default`] |
//! | `Square`            | [`MudAvatar::square`] | `false` |
//! | `Rounded`           | [`MudAvatar::rounded`] | `false` |
//! | `Size`              | [`MudAvatar::size`] | [`Size::Medium`] |
//! | `Elevation`         | [`MudAvatar::elevation`] | `0` |
//!
//! The private `ringed` / `ring_color` parameters are used by
//! [`MudAvatarGroup`](super::mud_avatar_group::MudAvatarGroup) to draw the
//! surface-coloured ring that visually separates overlapping avatars.
//!
//! # Per-frame model
//!
//! `show` consumes the avatar and returns a [`Response`]. The `tw_state`
//! (hover / active) is read by the Tailwind class resolver on the next
//! frame; because `show` consumes `self`, callers who need Tailwind hover
//! variants must keep the avatar in a field and reconstruct it each frame
//! with the cached hover/active flags.
//!
//! # Elevation
//!
//! `elevation` maps to an egui shadow on the wrapping [`Frame`]. Values
//! above 24 are clamped, matching MudBlazor's elevation scale.

use egui::{vec2, Color32, CornerRadius, Frame, Response, Sense, Stroke, Ui};
use mud_egui_bootstrap::bs_style::BsStyle;
use mud_egui_core::base::component_base::ComponentBase;
use mud_egui_core::enums::{Color, ObjectFit, Size, StyleFlavor, Variant};
use mud_egui_tailwind::tw_style::{TwState, TwStyle};
use mud_egui_theming::theme::MudTheme;

use crate::data_display::icons::mud_icon::MudIcon;
use crate::typography::mud_text::MudText;
use crate::MudImage;

/// Base rem size (in points) passed to the Tailwind resolver.
const REM_SIZE_PT: f32 = 14.0;

/// Thickness of the ring drawn around "ringed" avatars (used by
/// [`MudAvatarGroup`](super::mud_avatar_group::MudAvatarGroup)).
pub(crate) const RING_THICKNESS: f32 = 2.0;

/// Maximum elevation recognised by MudBlazor. Values above this are clamped.
pub(crate) const MAX_ELEVATION: u8 = 24;

#[inline]
fn resolve_color(c: Color, theme: &MudTheme, default: Color32) -> Color32 {
    let palette = theme.current_palette();
    match c {
        Color::Default | Color::Inherit => default,
        other => other.resolve(palette).unwrap_or(default),
    }
}

/// Pure size metrics for the avatar's standard `Size` buckets.
pub(crate) struct AvatarMetrics {
    pub(crate) diameter: f32,
    pub(crate) font_size: f32,
}

impl AvatarMetrics {
    /// Resolve metrics for a [`Size`] bucket. MudBlazor only defines Small,
    /// Medium, Large; other variants fall back to Medium.
    pub(crate) fn resolve(size: Size) -> Self {
        match size {
            Size::Small => Self {
                diameter: 24.0,
                font_size: 12.0,
            },
            Size::Large => Self {
                diameter: 56.0,
                font_size: 24.0,
            },
            _ => Self {
                diameter: 40.0,
                font_size: 20.0,
            },
        }
    }
}

/// Builds the egui shadow that corresponds to a MudBlazor elevation value
/// (0–24). Elevation 0 produces [`egui::epaint::Shadow::NONE`] and is not
/// applied at all by [`MudAvatar::show`].
pub(crate) fn elevation_shadow(elevation: u8) -> egui::epaint::Shadow {
    let e = elevation.min(MAX_ELEVATION) as f32;
    egui::epaint::Shadow {
        offset: [0, (e * 0.4) as i8],
        blur: (e * 1.5) as u8,
        spread: 0,
        color: Color32::from_black_alpha(((e * 3.0) as u8).min(60)),
    }
}

/// The avatar's content slot.
enum AvatarContent<'a> {
    /// No content — the circle renders empty.
    None,
    /// An SVG path drawn via [`MudIcon`] (MudBlazor's `Svg` parameter).
    Svg(String),
    /// Short text (initials) rendered via [`MudText`].
    Text(String),
    /// A raster image (MudBlazor's `Image` parameter).
    Image(egui::ImageSource<'a>),
    /// Arbitrary content — MudBlazor's `ChildContent`.
    Custom(Box<dyn FnMut(&mut Ui) + 'a>),
}

/// A small circular (or rounded-square) avatar bubble.
#[must_use = "MudAvatar does nothing until you call `.show(ui)`"]
pub struct MudAvatar<'a> {
    content: AvatarContent<'a>,
    color: Color,
    size: Option<Size>,
    variant: Variant,
    rounded: bool,
    square: bool,
    elevation: u8,
    /// When true, draws a thick ring around the avatar (for group overlap).
    ringed: bool,
    ring_color: Option<Color32>,

    pub base: ComponentBase,
    pub flavor: StyleFlavor,
    pub bs_style: BsStyle,
    pub tw_style: TwStyle,
    pub tw_state: TwState,
}

impl<'a> Default for MudAvatar<'a> {
    fn default() -> Self {
        Self {
            content: AvatarContent::None,
            color: Color::Default,
            size: None,
            variant: Variant::Filled,
            rounded: false,
            square: false,
            elevation: 0,
            ringed: false,
            ring_color: None,
            base: ComponentBase::new(),
            flavor: StyleFlavor::default(),
            bs_style: BsStyle::default(),
            tw_style: TwStyle::default(),
            tw_state: TwState::default(),
        }
    }
}

impl<'a> MudAvatar<'a> {
    /// Creates an empty avatar with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the accent [`Color`]. Defaults to [`Color::Default`], which maps
    /// to the theme's primary colour.
    #[inline]
    pub fn color(mut self, c: Color) -> Self {
        self.color = c;
        self
    }

    /// Sets the [`Size`] bucket (Small / Medium / Large).
    #[inline]
    pub fn size(mut self, s: Size) -> Self {
        self.size = Some(s);
        self
    }

    /// Sets the visual [`Variant`] (Filled / Outlined / Text).
    #[inline]
    pub fn variant(mut self, v: Variant) -> Self {
        self.variant = v;
        self
    }

    /// Uses 4 px corner radius instead of a full circle.
    #[inline]
    pub fn rounded(mut self, b: bool) -> Self {
        self.rounded = b;
        self
    }

    /// Uses zero corner radius (sharp square).
    #[inline]
    pub fn square(mut self, b: bool) -> Self {
        self.square = b;
        self
    }

    /// Sets the elevation (0–24; clamped).
    #[inline]
    pub fn elevation(mut self, e: u8) -> Self {
        self.elevation = e;
        self
    }

    /// Sets the avatar content to an SVG path (MudBlazor's `Svg` parameter).
    ///
    /// This is the primary API for icon avatars, e.g.
    /// `.svg(mud_egui_icons_material::filled::PERSON)`.
    #[inline]
    pub fn svg(mut self, svg: impl Into<String>) -> Self {
        self.content = AvatarContent::Svg(svg.into());
        self
    }

    /// Deprecated alias for [`svg`](Self::svg), kept for backward
    /// compatibility with pre-parity code.
    #[inline]
    #[deprecated(note = "renamed to `svg` for MudBlazor parity")]
    pub fn icon(self, svg: impl Into<String>) -> Self {
        self.svg(svg)
    }

    /// Sets the avatar content to short text (initials).
    #[inline]
    pub fn text(mut self, t: impl Into<String>) -> Self {
        self.content = AvatarContent::Text(t.into());
        self
    }

    /// Sets the avatar content to a raster image (e.g. profile photo).
    #[inline]
    pub fn image(mut self, src: egui::ImageSource<'a>) -> Self {
        self.content = AvatarContent::Image(src);
        self
    }

    /// Provides arbitrary custom content (MudBlazor's `ChildContent`).
    ///
    /// The closure receives the inner content [`Ui`] (inside the ring).
    #[inline]
    pub fn child(mut self, f: impl FnMut(&mut Ui) + 'a) -> Self {
        self.content = AvatarContent::Custom(Box::new(f));
        self
    }

    /// Internal: enable the shared ring drawn by [`MudAvatarGroup`].
    #[inline]
    pub fn ringed(mut self, b: bool) -> Self {
        self.ringed = b;
        self
    }

    /// Internal: set ring colour (usually the group's surface).
    #[inline]
    pub fn ring_color(mut self, c: Color32) -> Self {
        self.ring_color = Some(c);
        self
    }

    /// Sets the CSS flavour (Bootstrap vs Tailwind).
    #[inline]
    pub fn flavor(mut self, f: StyleFlavor) -> Self {
        self.flavor = f;
        self
    }

    /// Adds utility classes.
    #[inline]
    pub fn classes<S: AsRef<str>>(mut self, classes: impl IntoIterator<Item = S>) -> Self {
        self.base = self.base.classes(classes);
        self
    }

    /// Draws the avatar into `ui`.
    pub fn show(mut self, ui: &mut Ui) -> Response {
        let theme = MudTheme::fetch(ui.ctx()).unwrap_or_default();
        let palette = theme.current_palette();
        let metrics = AvatarMetrics::resolve(self.size.unwrap_or(Size::Medium));

        let diameter = metrics.diameter;
        let font_size = metrics.font_size;
        let ring_thickness = if self.ringed { RING_THICKNESS } else { 0.0 };
        let total_size = diameter + ring_thickness * 2.0;

        // Resolve CSS frame.
        let lookup = |key: &str| palette.resolve(key).map(Color32::from);
        self.tw_state.dark = ui.visuals().dark_mode;
        let mut frame = match self.flavor {
            StyleFlavor::Bootstrap => self.bs_style.build_frame(&self.base, &lookup),
            StyleFlavor::Tailwind => self
                .tw_style
                .build_frame(&self.base, self.tw_state, REM_SIZE_PT, &lookup),
        };
        if self.elevation > 0 {
            frame = frame.shadow(elevation_shadow(self.elevation));
        }

        let inner = frame.show(ui, |ui| {
            let (rect, resp) =
                ui.allocate_exact_size(vec2(total_size, total_size), Sense::hover());

            let corner_radius = if self.square {
                CornerRadius::ZERO
            } else if self.rounded {
                CornerRadius::same(4)
            } else {
                CornerRadius::same((total_size / 2.0) as u8)
            };

            let accent = resolve_color(self.color, &theme, palette.primary.into());
            let ink = match self.variant {
                Variant::Filled => palette.primary_contrast_text.into(),
                Variant::Outlined | Variant::Text => accent,
            };
            let bg = match self.variant {
                Variant::Filled => accent,
                _ => Color32::TRANSPARENT,
            };

            ui.painter().rect_filled(rect, corner_radius, bg);

            if self.ringed {
                let ring = self.ring_color.unwrap_or_else(|| palette.surface.into());
                ui.painter().rect_stroke(
                    rect,
                    corner_radius,
                    Stroke::new(ring_thickness, ring),
                    egui::StrokeKind::Inside,
                );
            }

            if self.variant == Variant::Outlined {
                ui.painter().rect_stroke(
                    rect,
                    corner_radius,
                    Stroke::new(1.0, accent),
                    egui::StrokeKind::Inside,
                );
            }

            let content_rect = rect.shrink(ring_thickness);

            match self.content {
                AvatarContent::Svg(svg) => {
                    let inset = diameter * 0.22;
                    let icon_rect = content_rect.shrink(inset);
                    let icon_size = (diameter * 0.55).max(8.0);
                    ui.put(
                        icon_rect,
                        MudIcon::new(svg)
                            .size_px(icon_size)
                            .viewbox_size(24.0)
                            .custom_color(ink),
                    );
                }
                AvatarContent::Text(text) => {
                    let mut text_ui = ui.new_child(
                        egui::UiBuilder::new()
                            .max_rect(content_rect)
                            .layout(egui::Layout::centered_and_justified(
                                egui::Direction::TopDown,
                            )),
                    );
                    text_ui.add(
                        MudText::new(text)
                            .custom_color(ink)
                            .font_size(font_size)
                            .bold(true)
                            .inline(true)
                            .no_wrap(true),
                    );
                }
                AvatarContent::Image(src) => {
                    let mut image_ui = ui.new_child(
                        egui::UiBuilder::new()
                            .max_rect(content_rect)
                            .layout(egui::Layout::top_down(egui::Align::Min)),
                    );
                    MudImage::new(src)
                        .fluid(true)
                        .object_fit(ObjectFit::Cover)
                        .show(&mut image_ui, content_rect.size());
                }
                AvatarContent::Custom(mut f) => {
                    let mut child = ui.new_child(
                        egui::UiBuilder::new()
                            .max_rect(content_rect)
                            .layout(egui::Layout::top_down(egui::Align::Min)),
                    );
                    f(&mut child);
                }
                AvatarContent::None => {}
            }

            resp
        });

        self.tw_state.hovered = inner.response.hovered();
        self.tw_state.active = inner.response.is_pointer_button_down_on();

        inner.inner
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // --- metrics -----------------------------------------------------------

    #[test]
    fn metrics_resolve_small_medium_large() {
        let s = AvatarMetrics::resolve(Size::Small);
        assert_eq!(s.diameter, 24.0);
        assert_eq!(s.font_size, 12.0);
        let m = AvatarMetrics::resolve(Size::Medium);
        assert_eq!(m.diameter, 40.0);
        assert_eq!(m.font_size, 20.0);
        let l = AvatarMetrics::resolve(Size::Large);
        assert_eq!(l.diameter, 56.0);
        assert_eq!(l.font_size, 24.0);
    }

    #[test]
    fn metrics_unknown_size_falls_back_to_medium() {
        let m = AvatarMetrics::resolve(Size::Medium);
        let x = AvatarMetrics::resolve(Size::ExtraSmall);
        assert_eq!(x.diameter, m.diameter);
        assert_eq!(x.font_size, m.font_size);
        let y = AvatarMetrics::resolve(Size::ExtraLarge);
        assert_eq!(y.diameter, m.diameter);
    }

    // --- color -------------------------------------------------------------

    #[test]
    fn resolve_color_default_and_inherit_use_fallback() {
        let theme = MudTheme::default();
        let fallback = Color32::from_rgb(1, 2, 3);
        assert_eq!(resolve_color(Color::Default, &theme, fallback), fallback);
        assert_eq!(resolve_color(Color::Inherit, &theme, fallback), fallback);
    }

    #[test]
    fn resolve_color_primary_maps_to_theme_primary() {
        let theme = MudTheme::default();
        let primary: Color32 = theme.current_palette().primary.into();
        let fallback = Color32::from_rgb(99, 0, 0);
        assert_eq!(resolve_color(Color::Primary, &theme, fallback), primary);
    }

    // --- elevation ---------------------------------------------------------

    #[test]
    fn elevation_zero_is_no_shadow() {
        // elevation 0 is skipped by `show`; the helper still returns a value.
        let s = elevation_shadow(0);
        assert_eq!(s.blur, 0);
        assert_eq!(s.color.a(), 0);
    }

    #[test]
    fn elevation_clamps_above_max() {
        let s24 = elevation_shadow(MAX_ELEVATION);
        let s200 = elevation_shadow(200);
        assert_eq!(s24.offset, s200.offset);
        assert_eq!(s24.blur, s200.blur);
        assert_eq!(s24.color, s200.color);
    }

    #[test]
    fn elevation_scales_monotonically() {
        let s1 = elevation_shadow(1);
        let s8 = elevation_shadow(8);
        let s16 = elevation_shadow(16);
        assert!(s1.blur < s8.blur);
        assert!(s8.blur < s16.blur);
        assert!(s1.color.a() < s8.color.a());
        assert!(s8.color.a() < s16.color.a());
    }

    // --- builders ----------------------------------------------------------

    #[test]
    fn default_has_no_content_and_zero_elevation() {
        let a = MudAvatar::new();
        assert!(matches!(a.content, AvatarContent::None));
        assert_eq!(a.elevation, 0);
        assert!(!a.rounded);
        assert!(!a.square);
        assert!(!a.ringed);
        assert!(matches!(a.color, Color::Default));
        assert!(matches!(a.variant, Variant::Filled));
        assert!(a.size.is_none());
    }

    #[test]
    fn svg_sets_svg_content() {
        let a = MudAvatar::new().svg("M12 2L2 22h20L12 2z");
        match a.content {
            AvatarContent::Svg(s) => assert_eq!(s, "M12 2L2 22h20L12 2z"),
            _ => panic!("expected Svg content"),
        }
    }

    #[test]
    fn text_sets_text_content() {
        let a = MudAvatar::new().text("AB");
        match a.content {
            AvatarContent::Text(s) => assert_eq!(s, "AB"),
            _ => panic!("expected Text content"),
        }
    }

    #[test]
    fn child_sets_custom_content() {
        let a = MudAvatar::new().child(|_ui| {});
        assert!(matches!(a.content, AvatarContent::Custom(_)));
    }

    #[test]
    fn builders_set_all_flags() {
        let a = MudAvatar::new()
            .color(Color::Secondary)
            .size(Size::Large)
            .variant(Variant::Outlined)
            .rounded(true)
            .square(true)
            .elevation(8)
            .ringed(true)
            .ring_color(Color32::RED)
            .classes(["a", "b"]);
        assert!(matches!(a.color, Color::Secondary));
        assert_eq!(a.size, Some(Size::Large));
        assert!(matches!(a.variant, Variant::Outlined));
        assert!(a.rounded && a.square && a.ringed);
        assert_eq!(a.elevation, 8);
        assert_eq!(a.ring_color, Some(Color32::RED));
    }

    // --- size math ---------------------------------------------------------

    #[test]
    fn total_size_accounts_for_ring() {
        let diameter = AvatarMetrics::resolve(Size::Medium).diameter;
        assert_eq!(diameter + RING_THICKNESS * 2.0, 44.0);
        let diameter = AvatarMetrics::resolve(Size::Small).diameter;
        assert_eq!(diameter, 24.0);
    }
}