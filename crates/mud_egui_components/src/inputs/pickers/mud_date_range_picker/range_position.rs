//! `RangePosition` — where one day sits relative to a displayed range.
//!
//! **Private**: this enum never appears in the public API. It exists so the
//! per-cell classification ([`classify`](super::MudDateRangePicker)) can be a
//! pure function returning a simple value, and so the paint code can `match`
//! exhaustively instead of chaining booleans.
//!
//! Kept in its own file per the one-enum-per-file rule (Rule 2).

/// Where a day sits relative to a [`DateRange`](super::DateRange).
///
/// Mirrors the CSS classes MudBlazor applies to range cells:
/// `mud-range-start-selected`, `mud-range-end-selected`,
/// `mud-range-between`, and `mud-selected` for a single-day range.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum RangePosition {
    /// Outside the range.
    Outside,
    /// The range's start day.
    Start,
    /// The range's end day.
    End,
    /// Strictly between the start and the end.
    Between,
    /// Start and end are the same day.
    SingleDay,
}

impl RangePosition {
    /// `true` for `Start`, `End` and `SingleDay` — the days drawn as solid
    /// accent pills rather than as part of the band.
    #[inline]
    pub(crate) fn is_endpoint(self) -> bool {
        matches!(self, Self::Start | Self::End | Self::SingleDay)
    }

    /// `true` for `Between`.
    #[inline]
    pub(crate) fn is_band(self) -> bool {
        matches!(self, Self::Between)
    }

    /// Name matching the variant, for tests and debugging.
    #[cfg(test)]
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Outside => "outside",
            Self::Start => "start",
            Self::End => "end",
            Self::Between => "between",
            Self::SingleDay => "single-day",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn endpoints_are_start_end_and_single_day() {
        assert!(RangePosition::Start.is_endpoint());
        assert!(RangePosition::End.is_endpoint());
        assert!(RangePosition::SingleDay.is_endpoint());
        assert!(!RangePosition::Between.is_endpoint());
        assert!(!RangePosition::Outside.is_endpoint());
    }

    #[test]
    fn only_between_is_a_band() {
        assert!(RangePosition::Between.is_band());
        for v in [
            RangePosition::Outside,
            RangePosition::Start,
            RangePosition::End,
            RangePosition::SingleDay,
        ] {
            assert!(!v.is_band(), "{} must not be a band", v.name());
        }
    }

    #[test]
    fn names_are_unique() {
        let mut names: Vec<&str> = vec![
            RangePosition::Outside.name(),
            RangePosition::Start.name(),
            RangePosition::End.name(),
            RangePosition::Between.name(),
            RangePosition::SingleDay.name(),
        ];
        names.sort_unstable();
        let mut deduped = names.clone();
        deduped.dedup();
        assert_eq!(names, deduped);
    }
}