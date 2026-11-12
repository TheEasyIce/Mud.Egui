//! `SortMode` — how many columns a grid may sort at once.
//!
//! Port of `MudBlazor.SortMode` (default `Multiple`).

/// How many columns may be sorted simultaneously.
///
/// Used by `MudDataGrid::sort_mode` and consumed by `toggle_sort` in
/// `mud_data_grid.rs`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SortMode {
    /// Sorting is disabled. Header clicks are no-ops.
    None,
    /// Only one column may be sorted; clicking another replaces the sort.
    Single,
    /// Multiple columns may be sorted, in priority (click) order.
    #[default]
    Multiple,
}

impl SortMode {
    /// Whether any sorting is possible.
    #[inline]
    pub fn is_enabled(self) -> bool {
        !matches!(self, Self::None)
    }

    /// Whether the mode allows more than one active sort at a time.
    #[inline]
    pub fn allows_multiple(self) -> bool {
        matches!(self, Self::Multiple)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_multiple() {
        assert_eq!(SortMode::default(), SortMode::Multiple);
    }

    #[test]
    fn is_enabled_matches_variant() {
        assert!(!SortMode::None.is_enabled());
        assert!(SortMode::Single.is_enabled());
        assert!(SortMode::Multiple.is_enabled());
    }

    #[test]
    fn allows_multiple_only_for_multiple() {
        assert!(!SortMode::None.allows_multiple());
        assert!(!SortMode::Single.allows_multiple());
        assert!(SortMode::Multiple.allows_multiple());
    }
}