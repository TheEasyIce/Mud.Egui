//! The action strip of a [`MudCard`](super::MudCard), usually holding buttons.
//!
//! Port of `MudCardActions.razor.cs` (MudBlazor 8.13.0). The component has
//! exactly one parameter, `ChildContent`; it applies MudBlazor's 8 pt padding
//! and lays its children out in a horizontally centred row.
//!
//! | MudBlazor parameter | Rust mapping |
//! |---------------------|--------------|
//! | `ChildContent` | the closure passed to [`MudCardActions::show`] |
//! | `Class` | [`MudCardActions::classes`] (stored, not resolved) |
//! | `Tag` | [`MudCardActions::tag`] / [`MudCardActions::tag_ref`] |
//! | `Style` / `UserAttributes` | not ported |
//!
//! # Alignment
//!
//! MudBlazor's stylesheet gives `.mud-card-actions` `display: flex;
//! align-items: center` and **no** `justify-content`, so buttons flow from the
//! start edge. This port reproduces exactly that: a left-to-right row,
//! vertically centred. To push actions to the end, insert a
//! [`MudSpacer`](crate::MudSpacer) as the last child — the same technique the
//! `MudToolBar` README recommends. No `justify` parameter is invented here.

use std::any::Any;

use egui::{Align, InnerResponse, Layout, Margin, Ui};

use super::mud_card::pad_i8;
use mud_egui_core::base::component_base::ComponentBase;

/// Padding of a card action strip, in logical pixels.
pub(crate) const ACTIONS_PADDING: f32 = 8.0;

/// The action strip of a [`MudCard`](super::MudCard).
///
/// Mirrors `MudCardActions` in MudBlazor 8.13.0: an 8 pt padded, vertically
/// centred horizontal row.
#[must_use = "MudCardActions does nothing until you call `.show(ui, ..)`"]
pub struct MudCardActions {
    /// Shared component state (utility classes). Stored for parity.
    pub base: ComponentBase,
    /// Arbitrary user data. Mirrors MudBlazor `Tag`.
    pub tag: Option<Box<dyn Any + Send + Sync>>,
}

impl Default for MudCardActions {
    fn default() -> Self {
        Self {
            base: ComponentBase::new(),
            tag: None,
        }
    }
}

impl MudCardActions {
    /// Creates an action strip with MudBlazor's default padding.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds utility classes to the action strip's [`ComponentBase`].
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

    /// Draws the action strip and runs `add_contents` in a horizontal row.
    pub fn show<R>(
        self,
        ui: &mut Ui,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> InnerResponse<R> {
        egui::Frame::new()
            .inner_margin(Margin::same(pad_i8(ACTIONS_PADDING)))
            .show(ui, |ui| {
                ui.with_layout(Layout::left_to_right(Align::Center), add_contents)
                    .inner
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn padding_matches_mudblazor() {
        assert_eq!(ACTIONS_PADDING, 8.0);
        assert_eq!(pad_i8(ACTIONS_PADDING), 8);
    }

    #[test]
    fn defaults_are_empty() {
        let a = MudCardActions::new();
        assert!(a.base.class_names().is_empty());
        assert!(a.tag.is_none());
    }

    #[test]
    fn builders_set_every_field() {
        let a = MudCardActions::new().classes(["justify-end"]).tag("foot");
        assert!(!a.base.class_names().is_empty());
        assert_eq!(a.tag_ref::<&str>(), Some(&"foot"));
        assert!(a.tag_ref::<u32>().is_none());
    }
}