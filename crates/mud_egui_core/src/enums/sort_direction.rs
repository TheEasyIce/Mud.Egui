//! `SortDirection` — direction of a single column's sort.
//!
//! Port of `MudBlazor.SortDirection`. Three states so that a header can be
//! toggled into "unsorted" when the grid's `AllowUnsorted` parameter is
//! enabled (see [`SortDirection::next`]).

/// The direction applied by one entry in the sort list.
///
/// `None` means "no sort applied to this column". This is distinct from
/// "this column has no sort definition at all": [`sort_direction_for`] in
/// `mud_data_grid.rs` returns `None` for both cases, so callers can treat
/// them identically.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SortDirection {
    /// No sort is applied to this column.
    #[default]
    None,
    /// Smallest first (A → Z, 0 → 9).
    Ascending,
    /// Largest first (Z → A, 9 → 0).
    Descending,
}

impl SortDirection {
    /// Whether a sort is actually applied.
    ///
    /// `None` returns `false`; both active directions return `true`.
    #[inline]
    pub fn is_active(self) -> bool {
        !matches!(self, Self::None)
    }

    /// The direction reached by clicking a column header one more time.
    ///
    /// The cycle depends on `allow_unsorted`:
    ///
    /// | current → next | `allow_unsorted = false` | `allow_unsorted = true` |
    /// |----------------|--------------------------|-------------------------|
    /// | `None`         | `Ascending`              | `Ascending`             |
    /// | `Ascending`    | `Descending`             | `Descending`            |
    /// | `Descending`   | `Ascending`              | `None`                  |
    ///
    /// This matches MudBlazor's `AllowUnsorted` remark exactly.
    #[inline]
    pub fn next(self, allow_unsorted: bool) -> Self {
        match (self, allow_unsorted) {
            (Self::None, _) => Self::Ascending,
            (Self::Ascending, _) => Self::Descending,
            (Self::Descending, true) => Self::None,
            (Self::Descending, false) => Self::Ascending,
        }
    }

    /// Reverses an active direction. `None` stays `None`.
    #[inline]
    pub fn reversed(self) -> Self {
        match self {
            Self::Ascending => Self::Descending,
            Self::Descending => Self::Ascending,
            Self::None => Self::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_none() {
        assert_eq!(SortDirection::default(), SortDirection::None);
    }

    #[test]
    fn is_active_matches_variant() {
        assert!(!SortDirection::None.is_active());
        assert!(SortDirection::Ascending.is_active());
        assert!(SortDirection::Descending.is_active());
    }

    #[test]
    fn next_cycles_without_unsorted() {
        assert_eq!(SortDirection::None.next(false), SortDirection::Ascending);
        assert_eq!(SortDirection::Ascending.next(false), SortDirection::Descending);
        assert_eq!(SortDirection::Descending.next(false), SortDirection::Ascending);
    }

    #[test]
    fn next_cycles_with_unsorted() {
        assert_eq!(SortDirection::None.next(true), SortDirection::Ascending);
        assert_eq!(SortDirection::Ascending.next(true), SortDirection::Descending);
        assert_eq!(SortDirection::Descending.next(true), SortDirection::None);
    }

    #[test]
    fn reversed_is_involutive() {
        for d in [
            SortDirection::None,
            SortDirection::Ascending,
            SortDirection::Descending,
        ] {
            assert_eq!(d.reversed().reversed(), d);
        }
    }

    #[test]
    fn reversed_maps_active_pairs() {
        assert_eq!(
            SortDirection::Ascending.reversed(),
            SortDirection::Descending
        );
        assert_eq!(
            SortDirection::Descending.reversed(),
            SortDirection::Ascending
        );
        assert_eq!(SortDirection::None.reversed(), SortDirection::None);
    }
}