//! The primary content block of a [`MudCard`](super::MudCard).
//!
//! Port of `MudCardContent.razor.cs` (MudBlazor 8.13.0). The component has
//! exactly one parameter, `ChildContent`; it applies MudBlazor's content
//! padding (16 pt on every side) and stacks its children vertically.
//!
//! | MudBlazor parameter | Rust mapping |
//! |---------------------|--------------|
//! | `ChildContent` | the closure passed to [`MudCardContent::show`] |
//! | `Class` | [`MudCardContent::classes`] (stored, not resolved) |
//! | `Tag` | [`MudCardContent::tag`] / [`MudCardContent::tag_ref`] |
//! | `Style` / `UserAttributes` | not ported |

use std::any::Any;

use egui::{InnerResponse, Margin, Ui};

use super::mud_card::pad_i8;
use mud_egui_core::base::component_base::ComponentBase;

/// Padding of a card content block, in logical pixels.
pub(crate) const CONTENT_PADDING: f32 = 16.0;

/// The primary content block of a [`MudCard`](super::MudCard).
///
/// Mirrors `MudCardContent` in MudBlazor 8.13.0: a 16 pt padded block whose
/// children stack vertically.
#[must_use = "MudCardContent does nothing until you call `.show(ui, ..)`"]
pub struct MudCardContent {
    /// Shared component state (utility classes). Stored for parity.
    pub base: ComponentBase,
    /// Arbitrary user data. Mirrors MudBlazor `Tag`.
    pub tag: Option<Box<dyn Any + Send + Sync>>,
}

impl Default for MudCardContent {
    fn default() -> Self {
        Self {
            base: ComponentBase::new(),
            tag: None,
        }
    }
}

impl MudCardContent {
    /// Creates a content block with MudBlazor's default padding.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds utility classes to the content block's [`ComponentBase`].
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

    /// Draws the content block and runs `add_contents` inside its padding.
    pub fn show<R>(
        self,
        ui: &mut Ui,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> InnerResponse<R> {
        egui::Frame::new()
            .inner_margin(Margin::same(pad_i8(CONTENT_PADDING)))
            .show(ui, add_contents)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn padding_matches_mudblazor() {
        assert_eq!(CONTENT_PADDING, 16.0);
        assert_eq!(pad_i8(CONTENT_PADDING), 16);
    }

    #[test]
    fn defaults_are_empty() {
        let c = MudCardContent::new();
        assert!(c.base.class_names().is_empty());
        assert!(c.tag.is_none());
    }

    #[test]
    fn builders_set_every_field() {
        let c = MudCardContent::new().classes(["pa-0"]).tag("body");
        assert!(!c.base.class_names().is_empty());
        assert_eq!(c.tag_ref::<&str>(), Some(&"body"));
        assert!(c.tag_ref::<u32>().is_none());
    }
}