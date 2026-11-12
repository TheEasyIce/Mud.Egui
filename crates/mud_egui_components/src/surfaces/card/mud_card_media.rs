//! An image strip at the top of a [`MudCard`](super::MudCard).
//!
//! Port of `MudCardMedia.razor.cs` (MudBlazor 8.13.0). The component has three
//! parameters and no `ChildContent`:
//!
//! | MudBlazor parameter | Rust field / method |
//! |---------------------|---------------------|
//! | `Image` | [`MudCardMedia::image`] |
//! | `Height` | [`MudCardMedia::height`] (default `300`) |
//! | `Title` | [`MudCardMedia::title`] |
//! | `Class` | [`MudCardMedia::classes`] (stored, not resolved) |
//! | `Tag` | [`MudCardMedia::tag`] / [`MudCardMedia::tag_ref`] |
//! | `Style` / `UserAttributes` | not ported |
//!
//! # How it draws
//!
//! MudBlazor renders `<img>` with `object-fit: cover` and the configured pixel
//! height. This port delegates to [`MudImage`](crate::MudImage) with
//! `ObjectFit::Cover` and the same height, so async loading, fallback sources
//! and the broken-image placeholder all come for free. `Title` becomes the
//! image's `alt`, which is what the HTML `title` attribute ends up doing for a
//! screen reader.
//!
//! A media strip with no `Image` still reserves `height` pixels, matching the
//! `<img>` element's box before it loads. Nothing is painted.

use std::any::Any;

use egui::{vec2, Response, Sense, Ui};

use crate::MudImage;
use mud_egui_core::base::component_base::ComponentBase;
use mud_egui_core::enums::ObjectFit;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Default media height, in logical pixels. Matches MudBlazor's `Height`
/// default of `300`.
pub(crate) const DEFAULT_MEDIA_HEIGHT: f32 = 300.0;

// ---------------------------------------------------------------------------
// MudCardMedia
// ---------------------------------------------------------------------------

/// An image strip for a [`MudCard`](super::MudCard).
///
/// Mirrors `MudCardMedia` in MudBlazor 8.13.0.
#[must_use = "MudCardMedia does nothing until you call `.show(ui)`"]
pub struct MudCardMedia {
    /// Shared component state (utility classes). Stored for parity.
    pub base: ComponentBase,
    /// Image URL or path. Mirrors MudBlazor `Image`.
    pub image: Option<String>,
    /// Height of the strip, in logical pixels. Mirrors MudBlazor `Height`
    /// (default `300`).
    pub height: f32,
    /// Tooltip / accessible text. Mirrors MudBlazor `Title`.
    pub title: Option<String>,
    /// Arbitrary user data. Mirrors MudBlazor `Tag`.
    pub tag: Option<Box<dyn Any + Send + Sync>>,
}

impl Default for MudCardMedia {
    /// No image, height `300`, no title — MudBlazor's defaults.
    fn default() -> Self {
        Self {
            base: ComponentBase::new(),
            image: None,
            height: DEFAULT_MEDIA_HEIGHT,
            title: None,
            tag: None,
        }
    }
}

impl MudCardMedia {
    /// Creates an empty media strip at MudBlazor's default height.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the image source. Mirrors `Image`.
    #[inline]
    pub fn image(mut self, src: impl Into<String>) -> Self {
        self.image = Some(src.into());
        self
    }

    /// Sets the strip height in logical pixels. Mirrors `Height`.
    ///
    /// Negative values become `0`; non-finite values fall back to
    /// [`DEFAULT_MEDIA_HEIGHT`].
    #[inline]
    pub fn height(mut self, h: f32) -> Self {
        self.height = h;
        self
    }

    /// Sets the tooltip / accessible text. Mirrors `Title`.
    #[inline]
    pub fn title(mut self, t: impl Into<String>) -> Self {
        self.title = Some(t.into());
        self
    }

    /// Adds utility classes to the strip's [`ComponentBase`].
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

    /// Draws the image strip at the full available width.
    pub fn show(self, ui: &mut Ui) -> Response {
        let width = ui.available_width();
        let height = sane_height(self.height);
        let size = vec2(width, height);

        let Some(src) = self.image else {
            // Same box as an unloaded <img>: reserve the space, paint nothing.
            let (_, resp) = ui.allocate_exact_size(size, Sense::hover());
            return resp;
        };

        let mut img = MudImage::new(src.as_str())
            .object_fit(ObjectFit::Cover)
            .height(height);
        if let Some(title) = self.title {
            img = img.alt(title);
        }
        img.show(ui, size)
    }
}

// ---------------------------------------------------------------------------
// Pure helpers
// ---------------------------------------------------------------------------

/// Repairs a media height: negative becomes `0`, non-finite falls back to the
/// default.
#[inline]
fn sane_height(h: f32) -> f32 {
    if !h.is_finite() {
        DEFAULT_MEDIA_HEIGHT
    } else {
        h.max(0.0)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_height_matches_mudblazor() {
        assert_eq!(MudCardMedia::new().height, 300.0);
    }

    #[test]
    fn sane_height_repairs_bad_values() {
        assert_eq!(sane_height(120.0), 120.0);
        assert_eq!(sane_height(0.0), 0.0);
        assert_eq!(sane_height(-10.0), 0.0);
        assert_eq!(sane_height(f32::NAN), DEFAULT_MEDIA_HEIGHT);
        assert_eq!(sane_height(f32::INFINITY), DEFAULT_MEDIA_HEIGHT);
    }

    #[test]
    fn defaults_match_mudblazor() {
        let m = MudCardMedia::new();
        assert!(m.image.is_none());
        assert_eq!(m.height, DEFAULT_MEDIA_HEIGHT);
        assert!(m.title.is_none());
        assert!(m.tag.is_none());
    }

    #[test]
    fn builders_set_every_field() {
        let m = MudCardMedia::new()
            .image("https://example.com/a.png")
            .height(180.0)
            .title("A photo")
            .classes(["rounded-t"])
            .tag("hero");
        assert_eq!(m.image.as_deref(), Some("https://example.com/a.png"));
        assert_eq!(m.height, 180.0);
        assert_eq!(m.title.as_deref(), Some("A photo"));
        assert!(!m.base.class_names().is_empty());
        assert_eq!(m.tag_ref::<&str>(), Some(&"hero"));
        assert!(m.tag_ref::<u32>().is_none());
    }
}