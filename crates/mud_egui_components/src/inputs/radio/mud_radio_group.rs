//! A mutually exclusive set of [`MudRadio`]s.
//!
//! Port of `MudRadioGroup.razor.cs` (MudBlazor 8.13.0). The public parameter
//! surface mirrors the Blazor component exactly:
//!
//! | MudBlazor parameter | Rust field / method |
//! |---------------------|---------------------|
//! | `ChildContent` | closure to [`MudRadioGroup::show`] |
//! | `Value` / `ValueChanged` | caller-owned `&mut Option<T>` in [`MudRadioGroup::new`] |
//! | `ReadOnly` | [`MudRadioGroup::read_only`] |
//! | `Disabled` | [`MudRadioGroup::disabled`] |
//! | `Name` | [`MudRadioGroup::name`] — stored, inert |
//! | `Class` | [`MudRadioGroup::classes`] (stored, not resolved) |
//! | `Tag` | [`MudRadioGroup::tag`] / [`MudRadioGroup::tag_ref`] |
//!
//! `Style` and `UserAttributes` are not ported.
//!
//! # Collector pattern
//!
//! Like [`MudToggleGroup`](crate::MudToggleGroup), the group takes a closure
//! that receives a [`RadioGroupUi`] collector. Nothing is rendered until the
//! closure returns, which is what lets the group lay the radios out in a
//! wrapping row without measuring them twice.
//!
//! ```ignore
//! let mut selected: Option<&str> = None;
//! MudRadioGroup::new(&mut selected).show(ui, |g| {
//!     g.radio(Some("a"), "Option A");
//!     g.radio(Some("b"), "Option B");
//! });
//! ```
//!
//! # `Name` is inert
//!
//! In Blazor, `Name` becomes the HTML `<input name="…">`, which groups radios
//! for form submission. egui has no form submission, so the parameter is stored
//! and documented as inert.

use std::any::Any;

use egui::{Response, Sense, Ui};

use super::mud_radio::MudRadio;
use mud_egui_core::base::component_base::ComponentBase;

// ---------------------------------------------------------------------------
// MudRadioGroup
// ---------------------------------------------------------------------------

/// A mutually exclusive set of [`MudRadio`]s.
///
/// Mirrors `MudRadioGroup<T>` in MudBlazor 8.13.0. The Blazor two-way binding
/// `Value` / `ValueChanged` becomes a caller-owned `&mut Option<T>`; the group
/// writes `Some(option)` when a radio is selected.
#[must_use = "MudRadioGroup does nothing until you call `.show(ui, ..)`"]
pub struct MudRadioGroup<'a, T: PartialEq + Clone> {
    /// Shared component state (utility classes). Stored for parity.
    pub base: ComponentBase,
    /// The bound selection. Mirrors MudBlazor `Value`.
    pub selected: &'a mut Option<T>,
    /// Disables every radio. Mirrors `Disabled`.
    pub disabled: bool,
    /// Prevents selection without dimming. Mirrors `ReadOnly`.
    pub read_only: bool,
    /// HTML form name. Stored, inert. Mirrors `Name`.
    pub name: Option<String>,
    /// Arbitrary user data. Mirrors `Tag`.
    pub tag: Option<Box<dyn Any + Send + Sync>>,
}

impl<'a, T: PartialEq + Clone> MudRadioGroup<'a, T> {
    /// Creates a group bound to `selected`.
    pub fn new(selected: &'a mut Option<T>) -> Self {
        Self {
            base: ComponentBase::new(),
            selected,
            disabled: false,
            read_only: false,
            name: None,
            tag: None,
        }
    }

    /// Disables every radio. Mirrors `Disabled`.
    #[inline]
    pub fn disabled(mut self, b: bool) -> Self {
        self.disabled = b;
        self
    }

    /// Prevents selection without dimming. Mirrors `ReadOnly`.
    #[inline]
    pub fn read_only(mut self, b: bool) -> Self {
        self.read_only = b;
        self
    }

    /// Sets the HTML form name. Stored, inert. Mirrors `Name`.
    #[inline]
    pub fn name(mut self, s: impl Into<String>) -> Self {
        self.name = Some(s.into());
        self
    }

    /// Adds utility classes. Stored for parity.
    pub fn classes<S: AsRef<str>>(mut self, classes: impl IntoIterator<Item = S>) -> Self {
        self.base = self.base.classes(classes);
        self
    }

    /// Attaches arbitrary user data. Mirrors `Tag`.
    #[inline]
    pub fn tag<U: Any + Send + Sync>(mut self, tag: U) -> Self {
        self.tag = Some(Box::new(tag));
        self
    }

    /// Reads user data attached with [`tag`](Self::tag).
    #[inline]
    pub fn tag_ref<U: Any + Send + Sync>(&self) -> Option<&U> {
        self.tag.as_ref().and_then(|b| b.downcast_ref::<U>())
    }

    /// Draws the group and reports this frame’s outcome.
    ///
    /// Radios are laid out in a wrapping row. The closure receives a
    /// [`RadioGroupUi`] and registers each radio with
    /// [`radio`](RadioGroupUi::radio) or [`radio_full`](RadioGroupUi::radio_full).
    pub fn show<R>(
        mut self,
        ui: &mut Ui,
        add_radios: impl FnOnce(&mut RadioGroupUi<T>) -> R,
    ) -> RadioGroupResponse {
        let mut radios: Vec<MudRadio<T>> = Vec::new();
        {
            let mut collector = RadioGroupUi { radios: &mut radios };
            let _ = add_radios(&mut collector);
        }

        if radios.is_empty() {
            return RadioGroupResponse {
                response: ui.allocate_response(egui::Vec2::ZERO, Sense::hover()),
                inner: None,
            };
        }

        let group_disabled = self.disabled;
        let group_read_only = self.read_only;
        let selected = &mut self.selected;

        let inner = ui
            .horizontal_wrapped(|ui| {
                for mut radio in radios.drain(..) {
                    if group_disabled {
                        radio = radio.disabled(true);
                    }
                    if group_read_only {
                        radio = radio.read_only(true);
                    }
                    radio.show(ui, selected);
                }
            })
            .response;

        RadioGroupResponse {
            response: inner,
            inner: None,
        }
    }
}

// ---------------------------------------------------------------------------
// RadioGroupUi
// ---------------------------------------------------------------------------

/// Collector passed to [`MudRadioGroup::show`].
///
/// Records radios; nothing is rendered until the closure returns.
pub struct RadioGroupUi<'a, T> {
    radios: &'a mut Vec<MudRadio<T>>,
}

impl<'a, T: PartialEq + Clone> RadioGroupUi<'a, T> {
    /// Adds a radio for `option` with `label`.
    pub fn radio(&mut self, option: T, label: impl Into<String>) {
        self.radios.push(MudRadio::new(option).label(label));
    }

    /// Adds a fully configured [`MudRadio`].
    pub fn radio_full(&mut self, radio: MudRadio<T>) {
        self.radios.push(radio);
    }

    /// Number of radios registered so far.
    #[inline]
    pub fn len(&self) -> usize {
        self.radios.len()
    }

    /// Whether no radio has been registered yet.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.radios.is_empty()
    }
}

// ---------------------------------------------------------------------------
// RadioGroupResponse
// ---------------------------------------------------------------------------

/// What [`MudRadioGroup::show`] did this frame.
#[derive(Debug)]
pub struct RadioGroupResponse {
    /// The group’s `Response`.
    pub response: Response,
    /// Reserved for the builder closure’s value; `None` because the group has
    /// no closure return.
    pub inner: Option<()>,
}

impl std::ops::Deref for RadioGroupResponse {
    type Target = Response;

    fn deref(&self) -> &Self::Target {
        &self.response
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_mudblazor() {
        let mut selected: Option<&str> = None;
        let group = MudRadioGroup::new(&mut selected);
        assert!(!group.disabled);
        assert!(!group.read_only);
        assert!(group.name.is_none());
        assert!(group.tag.is_none());
    }

    #[test]
    fn builders_set_every_field() {
        let mut selected: Option<&str> = None;
        let group = MudRadioGroup::new(&mut selected)
            .disabled(true)
            .read_only(true)
            .name("payment")
            .classes(["mt-2"])
            .tag("group");
        assert!(group.disabled && group.read_only);
        assert_eq!(group.name.as_deref(), Some("payment"));
        assert!(!group.base.class_names().is_empty());
        assert_eq!(group.tag_ref::<&str>(), Some(&"group"));
    }
}