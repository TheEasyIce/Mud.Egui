//! What a [`MudSelect`](super::MudSelect) did this frame.
//!
//! Kept in its own file per the project convention that every enum lives in
//! its own file.

/// The outcome of a [`MudSelect::show`](super::MudSelect::show) frame.
///
/// `T` is `Copy` in spirit — variant construction clones it once; the value
/// is not retained after the frame.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum SelectAction<T> {
    /// No interaction.
    #[default]
    None,
    /// The dropdown opened this frame.
    Opened,
    /// The dropdown closed this frame.
    Closed,
    /// An item was selected (`multi` is `true` when the selection was added
    /// alongside existing values, `false` when it replaced them).
    Selected { value: T, multi: bool },
    /// An item was deselected (multi mode only).
    Deselected { value: T },
    /// The clear button was pressed — the selection is now empty.
    Cleared,
    /// The "select all" checkbox was toggled.
    SelectAll { select: bool },
}

impl<T> SelectAction<T> {
    /// Whether the action changed the selection.
    #[inline]
    pub fn changed_selection(&self) -> bool {
        matches!(
            self,
            Self::Selected { .. }
                | Self::Deselected { .. }
                | Self::Cleared
                | Self::SelectAll { .. }
        )
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
        let a: SelectAction<i32> = SelectAction::default();
        assert!(matches!(a, SelectAction::None));
        assert!(!a.changed_selection());
        assert!(!a.toggled_open());
    }

    #[test]
    fn selection_variants_report_change() {
        assert!(SelectAction::<i32>::Selected { value: 1, multi: false }.changed_selection());
        assert!(SelectAction::<i32>::Deselected { value: 1 }.changed_selection());
        assert!(SelectAction::<i32>::Cleared.changed_selection());
        assert!(SelectAction::<i32>::SelectAll { select: true }.changed_selection());
    }

    #[test]
    fn toggle_variants_report_open() {
        assert!(SelectAction::<i32>::Opened.toggled_open());
        assert!(SelectAction::<i32>::Closed.toggled_open());
        assert!(!SelectAction::<i32>::Selected { value: 1, multi: false }.toggled_open());
    }
}