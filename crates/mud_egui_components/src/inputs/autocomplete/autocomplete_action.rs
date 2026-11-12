//! What a [`MudAutocomplete`](super::MudAutocomplete) did this frame.
//!
//! Kept in its own file per the project convention that every enum lives in
//! its own file. Public because it is a field of [`AutocompleteResponse`](
//! super::AutocompleteResponse).

/// The outcome of one [`MudAutocomplete::show`](super::MudAutocomplete::show)
/// frame.
///
/// `T` is the candidate value type. The `Selected` variant carries one
/// candidate; all other variants carry no payload, so they can be constructed
/// at any `T` (see the note on test turbofish in `mod tests`).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum AutocompleteAction<T> {
    /// No interaction this frame.
    #[default]
    None,
    /// The dropdown opened this frame.
    Opened,
    /// The dropdown closed this frame (blur, Escape, selection commit).
    Closed,
    /// A candidate was selected and committed to `state.selected`.
    Selected { value: T },
    /// The selection was cleared (`Clearable` button or empty-text blur).
    Cleared,
}

impl<T> AutocompleteAction<T> {
    /// Whether the action committed a selection or cleared one.
    #[inline]
    pub fn changed_selection(&self) -> bool {
        matches!(self, Self::Selected { .. } | Self::Cleared)
    }

    /// Whether the action opened or closed the dropdown.
    #[inline]
    pub fn toggled_open(&self) -> bool {
        matches!(self, Self::Opened | Self::Closed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_none() {
        let a: AutocompleteAction<i32> = AutocompleteAction::default();
        assert!(matches!(a, AutocompleteAction::None));
        assert!(!a.changed_selection());
        assert!(!a.toggled_open());
    }

    #[test]
    fn selection_variants_report_change() {
        assert!(AutocompleteAction::<i32>::Selected { value: 1 }.changed_selection());
        assert!(AutocompleteAction::<i32>::Cleared.changed_selection());
    }

    #[test]
    fn toggle_variants_report_open() {
        assert!(AutocompleteAction::<i32>::Opened.toggled_open());
        assert!(AutocompleteAction::<i32>::Closed.toggled_open());
        assert!(!AutocompleteAction::<i32>::Selected { value: 1 }.toggled_open());
    }
}