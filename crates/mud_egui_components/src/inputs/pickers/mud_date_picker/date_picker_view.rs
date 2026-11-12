//! `DatePickerView` — the three zoom levels a date picker renders.
//!
//! **Extension**: MudBlazor's `MudBasePicker` tracks this internally without
//! exposing the enum. The port promotes it to a public enum because it is a
//! public field of [`DatePickerState`](super::DatePickerState).
//!
//! Kept in its own file per the project convention that every enum lives in
//! its own file.

/// The zoom level the calendar is currently showing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DatePickerView {
    /// Show the day grid (calendar month).
    #[default]
    Day,
    /// Show the month grid (12 months of one year).
    Month,
    /// Show the year grid (a decade).
    Year,
}

impl DatePickerView {
    /// Human-readable name matching the variant.
    #[inline]
    pub fn name(self) -> &'static str {
        match self {
            Self::Day => "Day",
            Self::Month => "Month",
            Self::Year => "Year",
        }
    }

    /// Zoom out one level: `Day → Month → Year`.
    pub fn zoom_out(self) -> Self {
        match self {
            Self::Day => Self::Month,
            Self::Month => Self::Year,
            Self::Year => Self::Year,
        }
    }

    /// Zoom in one level: `Year → Month → Day`.
    pub fn zoom_in(self) -> Self {
        match self {
            Self::Day => Self::Day,
            Self::Month => Self::Day,
            Self::Year => Self::Month,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_day() {
        assert_eq!(DatePickerView::default(), DatePickerView::Day);
    }

    #[test]
    fn zoom_out_cycles_up() {
        assert_eq!(DatePickerView::Day.zoom_out(), DatePickerView::Month);
        assert_eq!(DatePickerView::Month.zoom_out(), DatePickerView::Year);
        assert_eq!(DatePickerView::Year.zoom_out(), DatePickerView::Year);
    }

    #[test]
    fn zoom_in_cycles_down() {
        assert_eq!(DatePickerView::Year.zoom_in(), DatePickerView::Month);
        assert_eq!(DatePickerView::Month.zoom_in(), DatePickerView::Day);
        assert_eq!(DatePickerView::Day.zoom_in(), DatePickerView::Day);
    }

    #[test]
    fn names_match_variants() {
        assert_eq!(DatePickerView::Day.name(), "Day");
        assert_eq!(DatePickerView::Month.name(), "Month");
        assert_eq!(DatePickerView::Year.name(), "Year");
    }
}