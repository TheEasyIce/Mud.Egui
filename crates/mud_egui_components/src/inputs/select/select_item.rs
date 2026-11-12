//! One selectable option inside a [`MudSelect`](super::MudSelect).
//!
//! Port of `MudSelectItem<T>.razor.cs` (MudBlazor 8.13.0).

use egui::Ui;

/// One selectable option.
#[must_use = "SelectItem does nothing until you add it to a MudSelect"]
pub struct SelectItem<T> {
    /// The value this item carries (`Value`).
    pub value: T,
    /// Display text (`Text`). When `None`, the select's `tostring_func` is
    /// consulted; if that is also `None`, `"?"` is used.
    pub text: Option<String>,
    /// Leading icon (`Icon`).
    pub icon: Option<String>,
    /// Prevents selecting this item (`Disabled`).
    pub disabled: bool,
    /// Custom content replacing icon + text (`ChildContent`).
    pub child: Option<Box<dyn FnMut(&mut Ui) + 'static>>,
}

impl<T> SelectItem<T> {
    /// Creates an item with the given value and no text.
    pub fn new(value: T) -> Self {
        Self {
            value,
            text: None,
            icon: None,
            disabled: false,
            child: None,
        }
    }

    /// Creates an item with a text label.
    pub fn text(value: T, text: impl Into<String>) -> Self {
        Self {
            value,
            text: Some(text.into()),
            icon: None,
            disabled: false,
            child: None,
        }
    }

    #[inline]
    pub fn with_text(mut self, s: impl Into<String>) -> Self {
        self.text = Some(s.into());
        self
    }

    #[inline]
    pub fn icon(mut self, svg: impl Into<String>) -> Self {
        self.icon = Some(svg.into());
        self
    }

    #[inline]
    pub fn disabled(mut self, b: bool) -> Self {
        self.disabled = b;
        self
    }

    /// Provides custom content that replaces icon + text. The closure runs
    /// only in the list rows, not in the trigger.
    #[inline]
    pub fn child(mut self, f: impl FnMut(&mut Ui) + 'static) -> Self {
        self.child = Some(Box::new(f));
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_has_no_text_or_icon() {
        let it = SelectItem::new(42);
        assert_eq!(it.value, 42);
        assert!(it.text.is_none());
        assert!(it.icon.is_none());
        assert!(!it.disabled);
        assert!(it.child.is_none());
    }

    #[test]
    fn text_constructor_sets_label() {
        let it = SelectItem::text("a", "Alpha");
        assert_eq!(it.value, "a");
        assert_eq!(it.text.as_deref(), Some("Alpha"));
    }

    #[test]
    fn builders_set_every_field() {
        let it = SelectItem::new(1)
            .with_text("One")
            .icon("M0 0")
            .disabled(true)
            .child(|_| {});
        assert_eq!(it.text.as_deref(), Some("One"));
        assert_eq!(it.icon.as_deref(), Some("M0 0"));
        assert!(it.disabled);
        assert!(it.child.is_some());
    }
}