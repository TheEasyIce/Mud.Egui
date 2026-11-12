//! Caller-owned state for [`MudSelect`](super::MudSelect).

use crate::feedback::popover::PopoverState;
use crate::inputs::select::SelectItem;

/// Caller-owned select state.
///
/// The `selected` vec mirrors MudBlazor's `SelectedValues`: it always holds
/// zero or more values, and in single-selection mode it holds at most one.
/// Persist it across frames.
#[derive(Debug, Clone)]
pub struct SelectState<T> {
    /// The currently selected value(s).
    pub selected: Vec<T>,
    /// The underlying popover state (open/closed, id, size cache).
    pub popover: PopoverState,
}

impl<T> SelectState<T> {
    /// Creates an empty selection with a stable id derived from `id_salt`.
    pub fn new(id_salt: impl std::hash::Hash + std::fmt::Debug) -> Self {
        Self {
            selected: Vec::new(),
            popover: PopoverState::new(id_salt),
        }
    }

    /// Seeds the selection from an iterator.
    pub fn with_selected<I>(mut self, values: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        self.selected = values.into_iter().collect();
        self
    }

    /// Whether the dropdown is currently open.
    #[inline]
    pub fn is_open(&self) -> bool {
        self.popover.open
    }

    /// Opens the dropdown.
    #[inline]
    pub fn open(&mut self) {
        self.popover.open = true;
    }

    /// Closes the dropdown.
    #[inline]
    pub fn close(&mut self) {
        self.popover.open = false;
    }

    /// Toggles the dropdown.
    #[inline]
    pub fn toggle(&mut self) {
        self.popover.open = !self.popover.open;
    }

    /// Whether `value` is in the selection.
    #[inline]
    pub fn is_selected(&self, value: &T) -> bool
    where
        T: PartialEq,
    {
        self.selected.iter().any(|v| v == value)
    }
}

/// Collector passed to [`MudSelect::show`](super::MudSelect::show).
pub struct SelectUi<T> {
    pub(crate) items: Vec<SelectItem<T>>,
}

impl<T> SelectUi<T> {
    /// Adds an item.
    pub fn item(&mut self, item: SelectItem<T>) {
        self.items.push(item);
    }

    /// Adds a text item (convenience).
    pub fn text_item(&mut self, value: T, text: impl Into<String>) {
        self.items.push(SelectItem::text(value, text));
    }

    /// Adds a value-only item (its display text is derived from the select's
    /// `tostring_func`).
    pub fn value(&mut self, value: T) {
        self.items.push(SelectItem::new(value));
    }

    /// Number of collected items.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Whether nothing has been collected.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_starts_empty_and_closed() {
        let s: SelectState<i32> = SelectState::new("test");
        assert!(s.selected.is_empty());
        assert!(!s.is_open());
    }

    #[test]
    fn with_selected_seeds_values() {
        let s = SelectState::new("t").with_selected([1, 2, 3]);
        assert_eq!(s.selected, vec![1, 2, 3]);
    }

    #[test]
    fn open_close_toggle() {
        let mut s: SelectState<i32> = SelectState::new("t");
        s.open();
        assert!(s.is_open());
        s.close();
        assert!(!s.is_open());
        s.toggle();
        assert!(s.is_open());
    }

    #[test]
    fn is_selected_checks_membership() {
        let s = SelectState::new("t").with_selected([1, 2]);
        assert!(s.is_selected(&1));
        assert!(!s.is_selected(&3));
    }

    #[test]
    fn collector_preserves_order() {
        let mut c: SelectUi<i32> = SelectUi { items: Vec::new() };
        c.text_item(1, "One");
        c.value(2);
        c.item(SelectItem::text(3, "Three"));
        assert_eq!(c.len(), 3);
        assert_eq!(c.items[0].text.as_deref(), Some("One"));
        assert!(c.items[1].text.is_none());
        assert_eq!(c.items[2].text.as_deref(), Some("Three"));
    }
}