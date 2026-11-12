//! A surface that groups a header, media, content and actions.
//!
//! Port of `MudCard.razor.cs` (MudBlazor 8.13.0). The public parameter surface
//! mirrors the Blazor component exactly:
//!
//! | MudBlazor parameter | Rust field / method |
//! |---------------------|---------------------|
//! | `ChildContent` | the closure passed to [`MudCard::show`] |
//! | `Elevation` | [`MudCard::elevation`] (default `1`) |
//! | `Outlined` | [`MudCard::outlined`] |
//! | `Square` | [`MudCard::square`] |
//! | `Class` | [`MudCard::classes`] (stored, not resolved) |
//! | `Tag` | [`MudCard::tag`] / [`MudCard::tag_ref`] |
//! | `Style` | not ported: egui has no inline CSS strings |
//! | `UserAttributes` | not ported: there is no HTML tag |
//!
//! # What a card is
//!
//! `MudBlazor` implements `MudCard` as a `MudPaper` with zero inner padding;
//! the visual padding comes from the companion components
//! ([`MudCardHeader`](super::MudCardHeader),
//! [`MudCardContent`](super::MudCardContent),
//! [`MudCardActions`](super::MudCardActions)). This port keeps that split: the
//! card paints only the surface, and the companions own the padding.
//!
//! # Elevation vs. outlined
//!
//! | `outlined` | `elevation` | Result |
//! |------------|-------------|--------|
//! | `false` | `0` | Flat surface, no shadow |
//! | `false` | `1..=24` | Surface with a drop shadow |
//! | `true` | any | 1 pt outline, **shadow suppressed** |
//!
//! Suppressing the shadow when outlined is MudBlazor's rule (`MudPaper`
//! ignores `Elevation` once `Outlined` is set), and it avoids the muddy
//! double-edge you get when a stroke and a shadow are drawn on the same edge.
//!
//! # Clipping
//!
//! egui's `Frame` does not clip children to its rounded corners. A
//! [`MudCardMedia`](super::MudCardMedia) placed directly inside a card with a
//! non-zero radius will therefore show square corners. The README documents the
//! `ui.scope` recipe for callers who need the MudBlazor `overflow: hidden`
//! behaviour; it is not applied implicitly because `MudCard` sizes to its
//! content and therefore does not know its own final rectangle while it is
//! still drawing.

use std::any::Any;

use egui::epaint::Shadow;
use egui::{Color32, CornerRadius, InnerResponse, Margin, Stroke, Ui};

use mud_egui_core::base::component_base::ComponentBase;
use mud_egui_theming::theme::MudTheme;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Corner radius of a non-square card, in logical pixels.
///
/// Matches MudBlazor's `MudGlobal.Rounded` default. `Square(true)` sets it to
/// zero.
pub(crate) const CARD_RADIUS: f32 = 4.0;

/// Width of the outlined card's border stroke, in logical pixels.
pub(crate) const OUTLINE_STROKE_WIDTH: f32 = 1.0;

/// Highest elevation MudBlazor accepts; values above this are clamped.
pub(crate) const MAX_ELEVATION: u8 = 24;

// ---------------------------------------------------------------------------
// MudCard
// ---------------------------------------------------------------------------

/// A surface that groups a header, media, content and actions.
///
/// Build it with the builder methods and draw it with [`MudCard::show`]:
///
/// ```ignore
/// MudCard::new().show(ui, |ui| {
///     MudCardHeader::new().show(ui, |ui| { ui.add(MudText::h6("Title")); });
///     MudCardContent::new().show(ui, |ui| { ui.add(MudText::body2("Body")); });
///     MudCardActions::new().show(ui, |ui| { ui.button("OK"); });
/// });
/// ```
///
/// Mirrors `MudCard` in MudBlazor 8.13.0.
#[must_use = "MudCard does nothing until you call `.show(ui, ..)`"]
pub struct MudCard {
    /// Shared component state (utility classes). Stored for parity with other
    /// components; not resolved while rendering, because the card owns its
    /// fill, border and radius.
    pub base: ComponentBase,
    /// Drop-shadow depth, clamped to [`MAX_ELEVATION`]. Mirrors MudBlazor
    /// `Elevation` (default `1`).
    pub elevation: u8,
    /// Draw a border instead of a shadow. Mirrors MudBlazor `Outlined`.
    pub outlined: bool,
    /// Disable corner rounding. Mirrors MudBlazor `Square`.
    pub square: bool,
    /// Arbitrary user data. Mirrors MudBlazor `Tag`.
    pub tag: Option<Box<dyn Any + Send + Sync>>,
}

impl Default for MudCard {
    /// Elevation `1`, rounded, not outlined — MudBlazor's defaults.
    fn default() -> Self {
        Self {
            base: ComponentBase::new(),
            elevation: 1,
            outlined: false,
            square: false,
            tag: None,
        }
    }
}

impl MudCard {
    /// Creates a card with MudBlazor's default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the drop-shadow depth (clamped to 24). Mirrors `Elevation`.
    #[inline]
    pub fn elevation(mut self, e: u8) -> Self {
        self.elevation = e;
        self
    }

    /// Draws a border and suppresses the shadow. Mirrors `Outlined`.
    #[inline]
    pub fn outlined(mut self, b: bool) -> Self {
        self.outlined = b;
        self
    }

    /// Removes the corner rounding. Mirrors `Square`.
    #[inline]
    pub fn square(mut self, b: bool) -> Self {
        self.square = b;
        self
    }

    /// Adds utility classes to the card's [`ComponentBase`].
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

    /// Draws the card surface and runs `add_contents` inside it.
    ///
    /// The card adds no padding of its own; use the companion components
    /// ([`MudCardHeader`](super::MudCardHeader),
    /// [`MudCardContent`](super::MudCardContent),
    /// [`MudCardActions`](super::MudCardActions)) for spacing.
    pub fn show<R>(
        self,
        ui: &mut Ui,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> InnerResponse<R> {
        let theme = MudTheme::fetch(ui.ctx()).unwrap_or_default();
        let fill: Color32 = theme.current_palette().surface.into();
        let outline = ui.visuals().widgets.noninteractive.bg_stroke.color;

        let mut frame = egui::Frame::new()
            .fill(fill)
            .corner_radius(CornerRadius::same(card_radius(self.square)))
            .inner_margin(Margin::same(0));

        if let Some(stroke) = card_stroke(self.outlined, outline) {
            frame = frame.stroke(stroke);
        }
        if let Some(shadow) = card_shadow(self.elevation) {
            frame = frame.shadow(shadow);
        }

        frame.show(ui, add_contents)
    }
}

// ---------------------------------------------------------------------------
// Pure helpers (no `egui::Context` required)
// ---------------------------------------------------------------------------

/// Corner radius in `u8`, as `CornerRadius` expects.
#[inline]
fn card_radius(square: bool) -> u8 {
    if square {
        0
    } else {
        pad_u8(CARD_RADIUS)
    }
}

/// The card's border stroke, or `None` when the card is not outlined.
#[inline]
fn card_stroke(outlined: bool, color: Color32) -> Option<Stroke> {
    outlined.then(|| Stroke::new(OUTLINE_STROKE_WIDTH, color))
}

/// The card's drop shadow, or `None` at elevation `0` or when outlined.
#[inline]
fn card_shadow(elevation: u8) -> Option<Shadow> {
    if elevation == 0 {
        return None;
    }
    let e = elevation.min(MAX_ELEVATION) as f32;
    Some(Shadow {
        offset: [0, (e * 0.5) as i8],
        blur: (e * 1.5) as u8,
        spread: 0,
        color: Color32::from_black_alpha(((e * 6.0) as u8).min(96)),
    })
}

/// Clamps a non-negative pixel value into the `i8` range used by
/// `egui::Margin` and `CornerRadius`.
///
/// `NaN` becomes `0`; values above `127` saturate instead of wrapping.
#[inline]
pub(crate) fn pad_u8(px: f32) -> u8 {
    if px.is_finite() {
        px.clamp(0.0, 255.0) as u8
    } else {
        0
    }
}

/// Clamps a pixel value into `i8`, the payload type of `egui::Margin`.
#[inline]
pub(crate) fn pad_i8(px: f32) -> i8 {
    pad_u8(px).min(127) as i8
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn card_radius_is_zero_when_square() {
        assert_eq!(card_radius(true), 0);
        assert_eq!(card_radius(false), CARD_RADIUS as u8);
    }

    #[test]
    fn card_stroke_only_when_outlined() {
        let red = Color32::from_rgb(200, 10, 10);
        assert!(card_stroke(false, red).is_none());
        let stroke = card_stroke(true, red).expect("outlined must draw a border");
        assert_eq!(stroke.width, OUTLINE_STROKE_WIDTH);
        assert_eq!(stroke.color, red);
    }

    #[test]
    fn card_shadow_is_none_at_zero() {
        assert!(card_shadow(0).is_none());
    }

    #[test]
    fn card_shadow_scales_and_clamps() {
        let s4 = card_shadow(4).expect("elevation 4");
        let s8 = card_shadow(8).expect("elevation 8");
        assert!(s8.blur > s4.blur);
        assert!(s8.color.a() > s4.color.a());
        assert_eq!(card_shadow(40), card_shadow(MAX_ELEVATION));
    }

    #[test]
    fn pad_helpers_sanitize() {
        assert_eq!(pad_u8(16.0), 16);
        assert_eq!(pad_u8(-4.0), 0);
        assert_eq!(pad_u8(f32::NAN), 0);
        assert_eq!(pad_u8(f32::INFINITY), 255);
        assert_eq!(pad_i8(200.0), 127);
        assert_eq!(pad_i8(8.0), 8);
    }

    #[test]
    fn defaults_match_mudblazor() {
        let c = MudCard::new();
        assert_eq!(c.elevation, 1);
        assert!(!c.outlined);
        assert!(!c.square);
        assert!(c.tag.is_none());
    }

    #[test]
    fn builders_set_every_field() {
        let c = MudCard::new()
            .elevation(6)
            .outlined(true)
            .square(true)
            .classes(["my-4"]);
        assert_eq!(c.elevation, 6);
        assert!(c.outlined);
        assert!(c.square);
        assert!(!c.base.class_names().is_empty());
    }

    #[test]
    fn tag_round_trips_by_type() {
        let c = MudCard::new().tag("product-42");
        assert_eq!(c.tag_ref::<&str>(), Some(&"product-42"));
        assert!(c.tag_ref::<u32>().is_none());
    }
}