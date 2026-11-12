//! `OpenTo` — which view a picker opens to first.
//!
//! Port of `MudBlazor.OpenTo` (MudBlazor 8.13.0). Shared by every picker
//! in the family: `MudDatePicker` accepts `Year` / `Month` / `Day`;
//! `MudTimePicker` accepts `Hours` / `Minutes`. Each picker only accepts
//! the variants that apply to it — the enum carries all five so the type is
//! shared.

/// Which view a picker opens to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum OpenTo {
    /// Date pickers: open on the calendar month view (the default for
    /// `MudDatePicker`).
    #[default]
    Day,
    /// Date pickers: open on the month grid (12 months of one year).
    Month,
    /// Date pickers: open on the year grid (a decade of years).
    Year,
    /// Time pickers: open on the hour clock.
    Hours,
    /// Time pickers: open on the minute clock.
    Minutes,
}

impl OpenTo {
    /// Human-readable name matching the C# enum member.
    #[inline]
    pub fn name(self) -> &'static str {
        match self {
            Self::Day => "Day",
            Self::Month => "Month",
            Self::Year => "Year",
            Self::Hours => "Hours",
            Self::Minutes => "Minutes",
        }
    }

    /// Whether this variant applies to date pickers.
    #[inline]
    pub fn is_date_variant(self) -> bool {
        matches!(self, Self::Day | Self::Month | Self::Year)
    }

    /// Whether this variant applies to time pickers.
    #[inline]
    pub fn is_time_variant(self) -> bool {
        matches!(self, Self::Hours | Self::Minutes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_day() {
        assert_eq!(OpenTo::default(), OpenTo::Day);
    }

    #[test]
    fn date_variants() {
        assert!(OpenTo::Day.is_date_variant());
        assert!(OpenTo::Month.is_date_variant());
        assert!(OpenTo::Year.is_date_variant());
        assert!(!OpenTo::Hours.is_date_variant());
        assert!(!OpenTo::Minutes.is_date_variant());
    }

    #[test]
    fn time_variants() {
        assert!(OpenTo::Hours.is_time_variant());
        assert!(OpenTo::Minutes.is_time_variant());
        assert!(!OpenTo::Day.is_time_variant());
        assert!(!OpenTo::Month.is_time_variant());
        assert!(!OpenTo::Year.is_time_variant());
    }

    #[test]
    fn names_match_variants() {
        assert_eq!(OpenTo::Day.name(), "Day");
        assert_eq!(OpenTo::Hours.name(), "Hours");
    }
}