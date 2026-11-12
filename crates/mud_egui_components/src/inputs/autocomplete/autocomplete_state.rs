//! Caller-owned state for [`MudAutocomplete`](super::MudAutocomplete).

use crate::feedback::popover::PopoverState;

/// Caller-owned autocomplete state.
///
/// The caller owns the **search text**, the **candidates** list (populated
/// each frame from the caller's own filter / async channel), and the
/// **selected** value. This is the egui equivalent of MudBlazor's bindable
/// `Value` plus its async `SearchFunc` result set.
///
/// # Async search
///
/// MudBlazor's `SearchFunc` returns `Task<IEnumerable<T>>`. egui is
/// immediate-mode with no async runtime per frame, so the port delegates the
/// search to the caller:
///
/// ```ignore
/// // In your per-frame code:
/// if self.state.text_changed_since_search() {
///     self.state.candidates = my_sync_or_cached_search(&self.state.search_text);
///     self.state.mark_searched();
/// }
/// self.autocomplete = MudAutocomplete::new()
///     .tostring_func(|v| v.label.clone());
/// self.autocomplete.show(ui, &mut self.state);
/// ```
///
/// For truly async search, push results into `candidates` from a channel
/// reader (run on `ctx.request_repaint()`); the component does not need to
/// know.
#[derive(Debug, Clone)]
pub struct AutocompleteState<T> {
    /// Current text in the trigger field. The component writes this back
    /// after every frame (the `TextEdit` updates it in place).
    pub search_text: String,
    /// Committed selection, if any. Set when the user picks a candidate or
    /// the caller calls [`select`](Self::select).
    pub selected: Option<T>,
    /// Candidate values to show in the dropdown. **Owned by the caller** —
    /// the component only reads them.
    pub candidates: Vec<T>,
    /// Underlying popover state (open / closed, id, size cache).
    pub popover: PopoverState,
    /// 0-based index of the keyboard-focused candidate, if any.
    pub focused_candidate: Option<usize>,
    /// Snapshot of [`search_text`](Self::search_text) at the last
    /// [`mark_searched`](Self::mark_searched) call, for change detection.
    pub last_searched_text: String,
}

impl<T> AutocompleteState<T> {
    /// Creates empty state with a stable id derived from `id_salt`.
    pub fn new(id_salt: impl std::hash::Hash + std::fmt::Debug) -> Self {
        Self {
            search_text: String::new(),
            selected: None,
            candidates: Vec::new(),
            popover: PopoverState::new(id_salt),
            focused_candidate: None,
            last_searched_text: String::new(),
        }
    }

    /// Seeds the state with a pre-selected value and its display text.
    pub fn with_selected_text(mut self, value: T, text: impl Into<String>) -> Self {
        self.selected = Some(value);
        self.search_text = text.into();
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
        self.focused_candidate = None;
    }

    /// Toggles the dropdown.
    #[inline]
    pub fn toggle(&mut self) {
        self.popover.open = !self.popover.open;
    }

    /// Commits `value` as the selection. Sets `search_text` to `text`,
    /// closes the popover, and clears the focused candidate.
    pub fn select(&mut self, value: T, text: impl Into<String>) {
        self.selected = Some(value);
        self.search_text = text.into();
        self.popover.close();
        self.focused_candidate = None;
    }

    /// Clears the selection, the search text, and the focused candidate.
    /// Does **not** clear `candidates` (those belong to the caller and are
    /// typically refreshed from the caller's search on the next change).
    pub fn clear(&mut self) {
        self.selected = None;
        self.search_text.clear();
        self.focused_candidate = None;
        self.popover.close();
    }

    /// Whether the search text has changed since the last
    /// [`mark_searched`](Self::mark_searched) call. Use this to decide
    /// whether to re-run your search filter.
    #[inline]
    pub fn text_changed_since_search(&self) -> bool {
        self.search_text != self.last_searched_text
    }

    /// Marks the current `search_text` as "already searched". Call after
    /// updating `candidates` from a search.
    #[inline]
    pub fn mark_searched(&mut self) {
        self.last_searched_text = self.search_text.clone();
    }

    /// Whether `value` is the current selection.
    pub fn is_selected(&self, value: &T) -> bool
    where
        T: PartialEq,
    {
        self.selected.as_ref() == Some(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_starts_empty() {
        let s: AutocompleteState<i32> = AutocompleteState::new("test");
        assert!(s.search_text.is_empty());
        assert!(s.selected.is_none());
        assert!(s.candidates.is_empty());
        assert!(!s.is_open());
        assert!(s.focused_candidate.is_none());
    }

    #[test]
    fn with_selected_text_seeds_both() {
        let s = AutocompleteState::new("t").with_selected_text(42, "Answer");
        assert_eq!(s.selected, Some(42));
        assert_eq!(s.search_text, "Answer");
    }

    #[test]
    fn select_commits_and_closes() {
        let mut s: AutocompleteState<i32> = AutocompleteState::new("t");
        s.open();
        s.select(7, "Seven");
        assert_eq!(s.selected, Some(7));
        assert_eq!(s.search_text, "Seven");
        assert!(!s.is_open());
        assert!(s.focused_candidate.is_none());
    }

    #[test]
    fn clear_resets_everything_but_candidates() {
        let mut s: AutocompleteState<i32> = AutocompleteState::new("t");
        s.selected = Some(3);
        s.search_text = "three".to_owned();
        s.candidates = vec![1, 2, 3];
        s.focused_candidate = Some(2);
        s.open();
        s.clear();
        assert!(s.selected.is_none());
        assert!(s.search_text.is_empty());
        assert_eq!(s.candidates.len(), 3);
        assert!(s.focused_candidate.is_none());
        assert!(!s.is_open());
    }

    #[test]
    fn text_changed_and_mark_searched() {
        let mut s = AutocompleteState::<i32>::new("t");
        assert!(!s.text_changed_since_search());
        s.search_text = "abc".to_owned();
        assert!(s.text_changed_since_search());
        s.mark_searched();
        assert!(!s.text_changed_since_search());
        s.search_text.push('d');
        assert!(s.text_changed_since_search());
    }

    #[test]
    fn is_selected_checks_membership() {
        let s = AutocompleteState::new("t").with_selected_text(1, "One");
        assert!(s.is_selected(&1));
        assert!(!s.is_selected(&2));
    }

    #[test]
    fn open_close_toggle() {
        let mut s: AutocompleteState<i32> = AutocompleteState::new("t");
        s.open();
        assert!(s.is_open());
        s.close();
        assert!(!s.is_open());
        s.toggle();
        assert!(s.is_open());
    }
}