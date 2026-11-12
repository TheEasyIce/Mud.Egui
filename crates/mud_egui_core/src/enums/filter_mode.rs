//! `FilterMode` — where a data grid renders its filter inputs.
//!
//! Port of `MudBlazor.FilterMode`, restricted to the variants that map
//! faithfully onto egui.
//!
//! MudBlazor also offers `Simple` and `ColumnFilterMenu`, but both are
//! popover-based and need a browser DOM to position. The inline row mode is
//! the only one with a direct egui equivalent, so it is the only variant
//! here. See the `MudDataGrid` README's parity section.

/// How per-column filters are rendered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FilterMode {
    /// Filtering is disabled. No filter inputs are rendered and no filter
    /// definitions are consumed.
    None,
    /// One inline text input per filterable column, drawn in a dedicated
    /// row directly below the header.
    #[default]
    ColumnFilterRow,
}

impl FilterMode {
    /// Whether any filter inputs are rendered.
    #[inline]
    pub fn is_enabled(self) -> bool {
        !matches!(self, Self::None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_column_filter_row() {
        assert_eq!(FilterMode::default(), FilterMode::ColumnFilterRow);
    }

    #[test]
    fn is_enabled_matches_variant() {
        assert!(!FilterMode::None.is_enabled());
        assert!(FilterMode::ColumnFilterRow.is_enabled());
    }
}