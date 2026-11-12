//! `MudImage` — displays an image with optional fallback, CSS-like
//! `object-fit` / `object-position`, elevation, and full async loading of
//! remote URLs.
//!
//! Port of `MudImage.razor.cs` (MudBlazor 8.13.0). See the module README for
//! the complete parity table and the async-loading setup instructions.
//!
//! # MudBlazor parity
//!
//! | MudBlazor parameter | Rust method | Default |
//! |---------------------|-------------|---------|
//! | `Alt`               | [`MudImage::alt`] | `None` |
//! | `Elevation`         | [`MudImage::elevation`] | `0` |
//! | `FallbackSrc`       | [`MudImage::fallback_src`] | `None` |
//! | `Fluid`             | [`MudImage::fluid`] | `false` |
//! | `Height`            | [`MudImage::height`] | `None` |
//! | `ObjectFit`         | [`MudImage::object_fit`] | [`ObjectFit::Fill`] |
//! | `ObjectPosition`    | [`MudImage::object_position`] | [`ObjectPosition::Center`] |
//! | `Src`               | [`MudImage::new`] / [`MudImage::src`] | required |
//! | `Width`             | [`MudImage::width`] | `None` |
//! | `Class`             | [`MudImage::classes`] | `None` |
//! | `Style`             | [`MudImage::frame`] | `None` |
//! | `Tag`               | [`MudImage::tag`] / [`MudImage::tag_ref`] | `None` |
//! | `UserAttributes`    | **not ported** (no HTML tag) | — |
//!
//! # Async loading
//!
//! `MudImage` uses egui's built-in loader pipeline:
//!
//! 1. `ImageSource::Uri` (from `"https://…"`, `"file://…"`, or a `bytes://`
//!    identifier) is polled with `ImageSource::load` every frame.
//! 2. The loader system returns `Result<TexturePoll, LoadError>` — a
//!    `Pending` state, a `Ready { texture, .. }` state, or an error.
//! 3. `MudImage` maps that to an internal [`ImageLoadPhase`]:
//!    - **Pending** → loading placeholder + `request_repaint`
//!    - **Ready** → paint the image via `egui::Image`
//!    - **Failed** → try the fallback source, then an error placeholder
//!
//! ## One-time setup (required)
//!
//! ```ignore
//! // In your eframe App::new / eframe::run_native creator:
//! egui_extras::install_image_loaders(&cc.egui_ctx);
//! ```
//!
//! And in `Cargo.toml`:
//!
//! ```toml
//! egui_extras = { version = "0.37", features = ["all_loaders"] }
//! image = { version = "0.25", features = ["jpeg", "png", "webp"] }
//! ```
//!
//! Without `all_loaders`, `file://` and `http(s)://` URIs will silently fail.
//!
//! # Object fit / position
//!
//! `ObjectFit` maps to CSS `object-fit` exactly:
//!
//! | Fit | Behaviour |
//! |-----|-----------|
//! | `Fill` (default) | Stretch to fill, distorting aspect |
//! | `Contain` | Fit inside, letterbox |
//! | `Cover` | Fill the container, crop (clipped) |
//! | `None` | Natural size, no scaling |
//! | `ScaleDown` | Natural size, but shrink if it would overflow |
//!
//! `ObjectPosition` maps to CSS `object-position`: the nine-way anchor used
//! when the scaled image is smaller than its container (`Contain`, `None`,
//! `ScaleDown`). For `Fill` and `Cover` the image fills the container, so
//! position is a no-op.
//!
//! # Fluid
//!
//! `Fluid(true)` sets width to the available width and lets height follow the
//! image's aspect ratio (`max-width: 100%; height: auto` in CSS). `Width` /
//! `Height` are ignored while `fluid` is on, matching MudBlazor.
//!
//! # Elevation
//!
//! `Elevation` maps to an `egui::epaint::Shadow` on the wrapping `Frame`,
//! using the same 0–24 scale as `MudAvatar` / `MudBadge` / `MudDataGrid` /
//! `MudSimpleTable` / `MudTable` / `MudTimeline`.

use egui::{
    load::{SizeHint, TexturePoll},
    vec2, Color32, CornerRadius, Rect, Response, Sense, Stroke, TextureOptions, Ui, Vec2,
    Widget,
};
use mud_egui_bootstrap::bs_style::BsStyle;
use mud_egui_core::base::component_base::ComponentBase;
use mud_egui_core::enums::{ObjectFit, ObjectPosition, StyleFlavor};
use mud_egui_tailwind::tw_style::{TwState, TwStyle};
use mud_egui_theming::theme::MudTheme;

use super::image_phase::ImageLoadPhase;

/// Base rem size passed to the Tailwind resolver.
const REM_SIZE_PT: f32 = 14.0;

/// Maximum elevation supported (MudBlazor clamps at 24).
pub const MAX_ELEVATION: u8 = 24;

/// Texture filtering used when polling the loader.
const TEXTURE_OPTIONS: TextureOptions = TextureOptions::LINEAR;

// ---------------------------------------------------------------------------
// Pure helpers
// ---------------------------------------------------------------------------

/// Resolves the effective [`ObjectFit`] from `fluid` and the optional override.
///
/// Mirrors MudBlazor: `Fluid` implies `Contain` when `ObjectFit` is not
/// explicitly set.
#[inline]
pub fn resolve_fit(fluid: bool, object_fit: Option<ObjectFit>) -> ObjectFit {
    object_fit.unwrap_or(if fluid {
        ObjectFit::Contain
    } else {
        ObjectFit::Fill
    })
}

/// Computes the scaled image size under CSS `object-fit` semantics.
///
/// `natural` is the image's intrinsic size; `container` is the box it must
/// fit into. Non-finite or non-positive inputs fall back to `container`.
pub fn compute_fit_size(natural: Vec2, container: Vec2, fit: ObjectFit) -> Vec2 {
    if !natural.x.is_finite()
        || !natural.y.is_finite()
        || natural.x <= 0.0
        || natural.y <= 0.0
        || container.x <= 0.0
        || container.y <= 0.0
    {
        return container;
    }
    let sx = container.x / natural.x;
    let sy = container.y / natural.y;
    match fit {
        ObjectFit::Fill => container,
        ObjectFit::Contain => {
            let s = sx.min(sy);
            natural * s
        }
        ObjectFit::Cover => {
            let s = sx.max(sy);
            natural * s
        }
        ObjectFit::None => natural,
        ObjectFit::ScaleDown => {
            let s = sx.min(sy).min(1.0);
            natural * s
        }
    }
}

/// Computes the top-left offset of a `scaled` image inside a `container` under
/// CSS `object-position` semantics.
pub fn compute_position_offset(
    scaled: Vec2,
    container: Vec2,
    position: ObjectPosition,
) -> Vec2 {
    let dx = container.x - scaled.x;
    let dy = container.y - scaled.y;
    let (fx, fy) = match position {
        ObjectPosition::TopLeft => (0.0, 0.0),
        ObjectPosition::Top => (0.5, 0.0),
        ObjectPosition::TopRight => (1.0, 0.0),
        ObjectPosition::Left => (0.0, 0.5),
        ObjectPosition::Center => (0.5, 0.5),
        ObjectPosition::Right => (1.0, 0.5),
        ObjectPosition::BottomLeft => (0.0, 1.0),
        ObjectPosition::Bottom => (0.5, 1.0),
        ObjectPosition::BottomRight => (1.0, 1.0),
    };
    vec2(dx * fx, dy * fy)
}

/// Resolves the allocated rect for the wrapper, honouring `Width` / `Height`
/// overrides and `Fluid`.
#[inline]
pub fn resolve_alloc_size(
    available: Vec2,
    fallback: Vec2,
    width: Option<f32>,
    height: Option<f32>,
    fluid: bool,
) -> Vec2 {
    if fluid {
        return vec2(
            if available.x.is_finite() && available.x > 0.0 {
                available.x
            } else {
                fallback.x
            },
            fallback.y,
        );
    }
    let w = width.filter(|w| w.is_finite() && *w >= 0.0).unwrap_or(fallback.x);
    let h = height.filter(|h| h.is_finite() && *h >= 0.0).unwrap_or(fallback.y);
    vec2(w, h)
}

/// Maps a MudBlazor elevation (0–24) to an egui shadow.
pub fn elevation_shadow(elevation: u8) -> egui::epaint::Shadow {
    let e = elevation.min(MAX_ELEVATION) as f32;
    egui::epaint::Shadow {
        offset: [0, (e * 0.4) as i8],
        blur: (e * 1.5) as u8,
        spread: 0,
        color: Color32::from_black_alpha(((e * 3.0) as u8).min(60)),
    }
}

// ---------------------------------------------------------------------------
// MudImage
// ---------------------------------------------------------------------------

/// An image control with async loading, fallback, and CSS-like object-fit.
#[must_use = "MudImage does nothing until you call `.show(ui, size)` or `ui.add(..)`"]
pub struct MudImage<'a> {
    /// The primary image source (`Src`).
    pub src: Option<egui::ImageSource<'a>>,
    /// The fallback image source (`FallbackSrc`). Rendered when `src` is
    /// `None` **or** fails to load.
    pub fallback_src: Option<egui::ImageSource<'a>>,
    /// Accessible text (`Alt`). Rendered as a hover tooltip.
    pub alt: Option<String>,

    /// Scales to the parent width (`Fluid`, default `false`).
    pub fluid: bool,
    /// Controls how the image is resized (`ObjectFit`, default `Fill`).
    pub object_fit: Option<ObjectFit>,
    /// Where the image sits within its container (`ObjectPosition`).
    pub object_position: ObjectPosition,
    /// Explicit width in pixels (`Width`).
    pub width: Option<f32>,
    /// Explicit height in pixels (`Height`).
    pub height: Option<f32>,

    /// Drop shadow depth (`Elevation`, default `0`, clamped to 24).
    pub elevation: u8,

    /// Custom loading placeholder. Receives the allocated rect.
    pub loading_placeholder: Option<Box<dyn FnMut(&mut Ui, Rect) + 'a>>,
    /// Custom error placeholder. Receives the allocated rect.
    pub error_placeholder: Option<Box<dyn FnMut(&mut Ui, Rect) + 'a>>,

    // CSS utilities
    pub base: ComponentBase,
    pub flavor: StyleFlavor,
    pub bs_style: BsStyle,
    pub tw_style: TwStyle,
    pub tw_state: TwState,
    /// Overrides the resolved wrapper frame (`Style`).
    pub frame_override: Option<egui::Frame>,
    tag_value: Option<std::sync::Arc<dyn std::any::Any + Send + Sync>>,
}

impl<'a> Default for MudImage<'a> {
    fn default() -> Self {
        Self {
            src: None,
            fallback_src: None,
            alt: None,
            fluid: false,
            object_fit: None,
            object_position: ObjectPosition::Center,
            width: None,
            height: None,
            elevation: 0,
            loading_placeholder: None,
            error_placeholder: None,
            base: ComponentBase::new(),
            flavor: StyleFlavor::default(),
            bs_style: BsStyle::default(),
            tw_style: TwStyle::default(),
            tw_state: TwState::default(),
            frame_override: None,
            tag_value: None,
        }
    }
}

impl<'a> MudImage<'a> {
    /// Creates an image with the given primary source.
    pub fn new(src: impl Into<egui::ImageSource<'a>>) -> Self {
        Self {
            src: Some(src.into()),
            ..Default::default()
        }
    }

    /// Creates an image with no primary source (fallback-only, if set).
    pub fn new_empty() -> Self {
        Self::default()
    }

    /// Replaces the primary image source (`Src`).
    #[inline]
    pub fn src(mut self, src: impl Into<egui::ImageSource<'a>>) -> Self {
        self.src = Some(src.into());
        self
    }

    /// Sets the fallback image source (`FallbackSrc`).
    #[inline]
    pub fn fallback_src(mut self, src: impl Into<egui::ImageSource<'a>>) -> Self {
        self.fallback_src = Some(src.into());
        self
    }

    /// Sets the accessible text (`Alt`).
    #[inline]
    pub fn alt(mut self, alt: impl Into<String>) -> Self {
        self.alt = Some(alt.into());
        self
    }

    /// Enables fluid sizing (`Fluid`).
    #[inline]
    pub fn fluid(mut self, fluid: bool) -> Self {
        self.fluid = fluid;
        self
    }

    /// Sets the object-fit mode (`ObjectFit`).
    #[inline]
    pub fn object_fit(mut self, fit: ObjectFit) -> Self {
        self.object_fit = Some(fit);
        self
    }

    /// Sets the object-position (`ObjectPosition`).
    #[inline]
    pub fn object_position(mut self, pos: ObjectPosition) -> Self {
        self.object_position = pos;
        self
    }

    /// Sets the explicit width in pixels (`Width`).
    #[inline]
    pub fn width(mut self, px: f32) -> Self {
        self.width = Some(px.max(0.0));
        self
    }

    /// Sets the explicit height in pixels (`Height`).
    #[inline]
    pub fn height(mut self, px: f32) -> Self {
        self.height = Some(px.max(0.0));
        self
    }

    /// Sets the drop-shadow elevation (`Elevation`).
    #[inline]
    pub fn elevation(mut self, e: u8) -> Self {
        self.elevation = e;
        self
    }

    /// Provides a custom loading placeholder.
    #[inline]
    pub fn loading_placeholder(
        mut self,
        f: impl FnMut(&mut Ui, Rect) + 'a,
    ) -> Self {
        self.loading_placeholder = Some(Box::new(f));
        self
    }

    /// Provides a custom error placeholder.
    #[inline]
    pub fn error_placeholder(
        mut self,
        f: impl FnMut(&mut Ui, Rect) + 'a,
    ) -> Self {
        self.error_placeholder = Some(Box::new(f));
        self
    }

    /// Selects the utility-class resolver.
    #[inline]
    pub fn flavor(mut self, flavor: StyleFlavor) -> Self {
        self.flavor = flavor;
        self
    }

    /// Adds utility classes to the wrapper frame (`Class`).
    #[inline]
    pub fn classes<S: AsRef<str>>(
        mut self,
        classes: impl IntoIterator<Item = S>,
    ) -> Self {
        self.base = self.base.classes(classes);
        self
    }

    /// Overrides the resolved wrapper frame (`Style`).
    #[inline]
    pub fn frame(mut self, f: egui::Frame) -> Self {
        self.frame_override = Some(f);
        self
    }

    /// Attaches arbitrary user data (`Tag`).
    pub fn tag<T: std::any::Any + Send + Sync>(mut self, value: T) -> Self {
        self.tag_value = Some(std::sync::Arc::new(value));
        self
    }

    /// Reads the tag back if it is of type `U`.
    pub fn tag_ref<U: std::any::Any + Send + Sync>(&self) -> Option<&U> {
        self.tag_value.as_ref()?.downcast_ref::<U>()
    }

    // ── Rendering ──────────────────────────────────────────────────────────

    /// Draws the image into `ui` with the given fallback size.
    pub fn show(mut self, ui: &mut Ui, size: Vec2) -> Response {
        let theme = MudTheme::fetch(ui.ctx()).unwrap_or_default();
        let palette = theme.current_palette();

        // 1. Resolve wrapper frame.
        let lookup = |key: &str| palette.resolve(key).map(Color32::from);
        self.tw_state.dark = ui.visuals().dark_mode;
        let mut frame = if let Some(f) = self.frame_override.take() {
            f
        } else {
            match self.flavor {
                StyleFlavor::Bootstrap => self.bs_style.build_frame(&self.base, &lookup),
                StyleFlavor::Tailwind => self.tw_style.build_frame(
                    &self.base,
                    self.tw_state,
                    REM_SIZE_PT,
                    &lookup,
                ),
            }
        };
        if self.elevation > 0 {
            frame = frame.shadow(elevation_shadow(self.elevation));
        }

        // 2. Resolve allocation size.
        let available = ui.available_size();
        let alloc = resolve_alloc_size(
            available,
            size,
            self.width,
            self.height,
            self.fluid,
        );

        let inner = frame.show(ui, |ui| {
            let (rect, response) =
                ui.allocate_exact_size(alloc, Sense::hover());

            // 3. Poll primary, then fallback.
            let primary_phase = self.poll_source(ui, self.src.as_ref());
            let primary_ready = primary_phase.is_ready();

            if primary_ready {
                // Clone the source reference into a local so the mutable
                // borrow taken by `paint_loading` / `paint_error` later in
                // this block cannot conflict with the immutable borrow used
                // by the argument to `paint_loaded`.
                if let Some(source) = self.src.clone() {
                    self.paint_loaded(ui, &source, rect);
                }
            } else if primary_phase.needs_repaint() {
                self.paint_loading(ui, rect);
                ui.ctx().request_repaint();
            } else {
                // Absent or Failed: try fallback.
                let fallback_phase = self.poll_source(ui, self.fallback_src.as_ref());
                if fallback_phase.is_ready() {
                    if let Some(source) = self.fallback_src.clone() {
                        self.paint_loaded(ui, &source, rect);
                    }
                } else if fallback_phase.needs_repaint() {
                    self.paint_loading(ui, rect);
                    ui.ctx().request_repaint();
                } else {
                    // Both failed (or neither set): error placeholder.
                    self.paint_error(ui, rect);
                }
            }

            // 4. Alt tooltip.
            if let Some(alt) = self.alt.clone() {
                response.on_hover_text(alt)
            } else {
                response
            }
        });

        self.tw_state.hovered = inner.response.hovered();
        self.tw_state.active = inner.response.is_pointer_button_down_on();

        inner.inner
    }

    /// Draws the image into `ui` using the available size.
    pub fn show_default(self, ui: &mut Ui) -> Response {
        let size = ui.available_size();
        self.show(ui, size)
    }

    // ── Internals ──────────────────────────────────────────────────────────

    /// Polls the loader state for `source` and maps it to an
    /// [`ImageLoadPhase`].
    ///
    /// `ImageSource::load` triggers the async pipeline and returns the cached
    /// state. In egui 0.37 the return type is `Result<TexturePoll, LoadError>`.
    fn poll_source(
        &self,
        ui: &Ui,
        source: Option<&egui::ImageSource<'a>>,
    ) -> ImageLoadPhase {
        let Some(source) = source else {
            return ImageLoadPhase::Absent;
        };
        match source
            .clone()
            .load(ui.ctx(), TEXTURE_OPTIONS, SizeHint::default())
        {
            Ok(TexturePoll::Ready { .. }) => ImageLoadPhase::Ready,
            Ok(TexturePoll::Pending { .. }) => ImageLoadPhase::Pending,
            Err(_) => ImageLoadPhase::Failed,
        }
    }

    /// Paints a ready image with the current fit / position.
    ///
    /// Takes `&self` (not `&mut self`): the function only reads fields, and
    /// callers pass `self.src.as_ref()`-derived arguments, so `&mut self`
    /// would conflict with the borrowed argument.
    fn paint_loaded(
        &self,
        ui: &mut Ui,
        source: &egui::ImageSource<'a>,
        container: Rect,
    ) {
        // Resolve the natural size from the loader's cached `TexturePoll`.
        let natural = match source
            .clone()
            .load(ui.ctx(), TEXTURE_OPTIONS, SizeHint::default())
        {
            Ok(TexturePoll::Ready { texture, .. }) => texture.size,
            _ => return,
        };

        let fit = resolve_fit(self.fluid, self.object_fit);
        let scaled = compute_fit_size(natural, container.size(), fit);
        let offset = compute_position_offset(
            scaled,
            container.size(),
            self.object_position,
        );
        let paint_rect = Rect::from_min_size(container.min + offset, scaled);

        let needs_clip = matches!(fit, ObjectFit::Cover);

        if needs_clip {
            ui.scope(|ui| {
                ui.set_clip_rect(container);
                ui.put(
                    paint_rect,
                    egui::Image::new(source.clone())
                        .fit_to_exact_size(scaled)
                        .maintain_aspect_ratio(true),
                );
            });
        } else {
            ui.put(
                paint_rect,
                egui::Image::new(source.clone())
                    .fit_to_exact_size(scaled)
                    .maintain_aspect_ratio(false),
            );
        }
    }

    /// Paints the loading placeholder (custom, or a default spinner).
    fn paint_loading(&mut self, ui: &mut Ui, rect: Rect) {
        if let Some(f) = self.loading_placeholder.as_mut() {
            let mut child = ui.new_child(
                egui::UiBuilder::new()
                    .max_rect(rect)
                    .layout(egui::Layout::centered_and_justified(
                        egui::Direction::TopDown,
                    )),
            );
            f(&mut child, rect);
        } else {
            let mut child = ui.new_child(
                egui::UiBuilder::new()
                    .max_rect(rect)
                    .layout(egui::Layout::centered_and_justified(
                        egui::Direction::TopDown,
                    )),
            );
            child.add(egui::Spinner::new());
            ui.painter().rect_stroke(
                rect,
                CornerRadius::same(4),
                Stroke::new(1.0, ui.visuals().widgets.noninteractive.bg_stroke.color),
                egui::StrokeKind::Inside,
            );
        }
    }

    /// Paints the error placeholder (custom, or a default broken-image glyph).
    fn paint_error(&mut self, ui: &mut Ui, rect: Rect) {
        ui.painter().rect_filled(
            rect,
            CornerRadius::same(4),
            ui.visuals().faint_bg_color,
        );
        if let Some(f) = self.error_placeholder.as_mut() {
            let mut child = ui.new_child(
                egui::UiBuilder::new()
                    .max_rect(rect)
                    .layout(egui::Layout::centered_and_justified(
                        egui::Direction::TopDown,
                    )),
            );
            f(&mut child, rect);
        } else {
            let mut child = ui.new_child(
                egui::UiBuilder::new()
                    .max_rect(rect)
                    .layout(egui::Layout::centered_and_justified(
                        egui::Direction::TopDown,
                    )),
            );
            child.add(
                crate::MudIcon::new(
                    mud_egui_icons_material::filled::BROKEN_IMAGE.to_owned(),
                )
                    .size(2)
                    .custom_color(ui.visuals().weak_text_color()),
            );
        }
    }
}

impl<'a> Widget for MudImage<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let size = ui.available_size();
        self.show(ui, size)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use egui::vec2;

    // --- resolve_fit -------------------------------------------------------

    #[test]
    fn resolve_fit_honours_explicit_object_fit() {
        for fit in [
            ObjectFit::Fill,
            ObjectFit::Contain,
            ObjectFit::Cover,
            ObjectFit::None,
            ObjectFit::ScaleDown,
        ] {
            assert_eq!(resolve_fit(false, Some(fit)), fit);
            assert_eq!(resolve_fit(true, Some(fit)), fit);
        }
    }

    #[test]
    fn resolve_fit_defaults_to_fill_without_fluid() {
        assert!(matches!(resolve_fit(false, None), ObjectFit::Fill));
    }

    #[test]
    fn resolve_fit_defaults_to_contain_with_fluid() {
        assert!(matches!(resolve_fit(true, None), ObjectFit::Contain));
    }

    // --- compute_fit_size --------------------------------------------------

    fn natural() -> Vec2 {
        vec2(400.0, 200.0)
    }

    #[test]
    fn fit_fill_stretches_to_container() {
        let out = compute_fit_size(natural(), vec2(100.0, 100.0), ObjectFit::Fill);
        assert_eq!(out, vec2(100.0, 100.0));
    }

    #[test]
    fn fit_contain_preserves_aspect() {
        let out = compute_fit_size(natural(), vec2(100.0, 100.0), ObjectFit::Contain);
        assert_eq!(out, vec2(100.0, 50.0));
        let out = compute_fit_size(natural(), vec2(1000.0, 100.0), ObjectFit::Contain);
        assert_eq!(out, vec2(200.0, 100.0));
    }

    #[test]
    fn fit_cover_covers_container() {
        let out = compute_fit_size(natural(), vec2(100.0, 100.0), ObjectFit::Cover);
        assert_eq!(out, vec2(200.0, 100.0));
    }

    #[test]
    fn fit_none_keeps_natural_size() {
        let out = compute_fit_size(natural(), vec2(100.0, 100.0), ObjectFit::None);
        assert_eq!(out, natural());
    }

    #[test]
    fn fit_scale_down_shrinks_only_when_needed() {
        let out = compute_fit_size(natural(), vec2(800.0, 800.0), ObjectFit::ScaleDown);
        assert_eq!(out, natural());
        let out = compute_fit_size(natural(), vec2(100.0, 100.0), ObjectFit::ScaleDown);
        assert_eq!(out, vec2(100.0, 50.0));
    }

    #[test]
    fn fit_repairs_non_finite_inputs() {
        let out = compute_fit_size(
            vec2(f32::NAN, 10.0),
            vec2(100.0, 100.0),
            ObjectFit::Contain,
        );
        assert_eq!(out, vec2(100.0, 100.0));
    }

    #[test]
    fn fit_repairs_zero_size() {
        let out = compute_fit_size(
            vec2(0.0, 0.0),
            vec2(100.0, 100.0),
            ObjectFit::Fill,
        );
        assert_eq!(out, vec2(100.0, 100.0));
    }

    // --- compute_position_offset ------------------------------------------

    #[test]
    fn position_center_centres() {
        let off = compute_position_offset(
            vec2(50.0, 50.0),
            vec2(100.0, 100.0),
            ObjectPosition::Center,
        );
        assert_eq!(off, vec2(25.0, 25.0));
    }

    #[test]
    fn position_top_left_is_zero() {
        let off = compute_position_offset(
            vec2(50.0, 50.0),
            vec2(100.0, 100.0),
            ObjectPosition::TopLeft,
        );
        assert_eq!(off, vec2(0.0, 0.0));
    }

    #[test]
    fn position_bottom_right_is_positive_max() {
        let off = compute_position_offset(
            vec2(50.0, 50.0),
            vec2(100.0, 100.0),
            ObjectPosition::BottomRight,
        );
        assert_eq!(off, vec2(50.0, 50.0));
    }

    #[test]
    fn position_top_centres_horizontally() {
        let off = compute_position_offset(
            vec2(50.0, 50.0),
            vec2(100.0, 100.0),
            ObjectPosition::Top,
        );
        assert_eq!(off, vec2(25.0, 0.0));
    }

    #[test]
    fn position_right_middle() {
        let off = compute_position_offset(
            vec2(50.0, 50.0),
            vec2(100.0, 100.0),
            ObjectPosition::Right,
        );
        assert_eq!(off, vec2(50.0, 25.0));
    }

    #[test]
    fn position_negative_for_overflowing_image() {
        let off = compute_position_offset(
            vec2(200.0, 100.0),
            vec2(100.0, 100.0),
            ObjectPosition::Center,
        );
        assert_eq!(off, vec2(-50.0, 0.0));
    }

    // --- resolve_alloc_size ------------------------------------------------

    #[test]
    fn alloc_uses_width_height_when_set() {
        let out = resolve_alloc_size(
            vec2(1000.0, 500.0),
            vec2(300.0, 200.0),
            Some(400.0),
            Some(250.0),
            false,
        );
        assert_eq!(out, vec2(400.0, 250.0));
    }

    #[test]
    fn alloc_falls_back_to_size() {
        let out = resolve_alloc_size(
            vec2(1000.0, 500.0),
            vec2(300.0, 200.0),
            None,
            None,
            false,
        );
        assert_eq!(out, vec2(300.0, 200.0));
    }

    #[test]
    fn alloc_fluid_uses_available_width() {
        let out = resolve_alloc_size(
            vec2(800.0, 500.0),
            vec2(300.0, 200.0),
            Some(400.0),
            Some(250.0),
            true,
        );
        assert_eq!(out, vec2(800.0, 200.0));
    }

    #[test]
    fn alloc_fluid_repairs_non_finite_available() {
        let out = resolve_alloc_size(
            vec2(f32::NAN, 500.0),
            vec2(300.0, 200.0),
            None,
            None,
            true,
        );
        assert_eq!(out, vec2(300.0, 200.0));
    }

    // --- elevation ---------------------------------------------------------

    #[test]
    fn elevation_zero_is_no_shadow() {
        let s = elevation_shadow(0);
        assert_eq!(s.blur, 0);
        assert_eq!(s.color.a(), 0);
    }

    #[test]
    fn elevation_clamps_above_max() {
        let a = elevation_shadow(MAX_ELEVATION);
        let b = elevation_shadow(200);
        assert_eq!(a.offset, b.offset);
        assert_eq!(a.blur, b.blur);
        assert_eq!(a.color, b.color);
    }

    #[test]
    fn elevation_scales_monotonically() {
        let s1 = elevation_shadow(1);
        let s8 = elevation_shadow(8);
        let s16 = elevation_shadow(16);
        assert!(s1.blur < s8.blur);
        assert!(s8.blur < s16.blur);
    }

    // --- builders ----------------------------------------------------------

    #[test]
    fn defaults_match_mudblazor() {
        let i: MudImage<'static> = MudImage::new_empty();
        assert!(i.src.is_none());
        assert!(i.fallback_src.is_none());
        assert!(i.alt.is_none());
        assert!(!i.fluid);
        assert!(i.object_fit.is_none());
        assert_eq!(i.object_position, ObjectPosition::Center);
        assert!(i.width.is_none());
        assert!(i.height.is_none());
        assert_eq!(i.elevation, 0);
    }

    #[test]
    fn builders_set_every_field() {
        let i = MudImage::new("https://example.com/a.png")
            .fallback_src("file://fallback.png")
            .alt("A picture")
            .fluid(true)
            .object_fit(ObjectFit::Cover)
            .object_position(ObjectPosition::Top)
            .width(400.0)
            .height(300.0)
            .elevation(4)
            .loading_placeholder(|_, _| {})
            .error_placeholder(|_, _| {})
            .classes(["rounded"]);
        assert!(i.src.is_some());
        assert!(i.fallback_src.is_some());
        assert_eq!(i.alt.as_deref(), Some("A picture"));
        assert!(i.fluid);
        assert!(matches!(i.object_fit, Some(ObjectFit::Cover)));
        assert_eq!(i.object_position, ObjectPosition::Top);
        assert_eq!(i.width, Some(400.0));
        assert_eq!(i.height, Some(300.0));
        assert_eq!(i.elevation, 4);
        assert!(i.loading_placeholder.is_some());
        assert!(i.error_placeholder.is_some());
    }

    #[test]
    fn width_and_height_clamp_negative() {
        let i = MudImage::new_empty().width(-50.0).height(-50.0);
        assert_eq!(i.width, Some(0.0));
        assert_eq!(i.height, Some(0.0));
    }

    #[test]
    fn tag_round_trips() {
        let i = MudImage::new_empty().tag("hero-image");
        assert_eq!(i.tag_ref::<&str>(), Some(&"hero-image"));
        assert!(i.tag_ref::<u32>().is_none());
    }

    // --- ImageLoadPhase ----------------------------------------------------

    #[test]
    fn phase_predicates() {
        assert!(ImageLoadPhase::Pending.needs_repaint());
        assert!(!ImageLoadPhase::Ready.needs_repaint());
        assert!(ImageLoadPhase::Ready.is_ready());
        assert!(!ImageLoadPhase::Failed.is_ready());
    }
}