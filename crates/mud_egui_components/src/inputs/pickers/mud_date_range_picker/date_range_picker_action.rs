//! `DateRangePickerAction` — what a
//! [`MudDateRangePicker`](super::MudDateRangePicker) did this frame.
//!
//! **Extension**: MudBlazor reports these outcomes through three callbacks
//! (`PickerOpened`, `PickerClosed`, `DateRangeChanged`). The port promotes the
//! outcome to one enum so a caller can read it from the return value, the same
//! way [`DatePickerAction`](crate::inputs::date_picker::DatePickerAction) and
//! `MudRating`'s `RatingResponse` report their frame's result.
//!
//! Kept in its own file per Rule 2.

use super::date_range::DateRange;

/// What a [`MudDateRangePicker`](super::MudDateRangePicker) did this frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DateRangePickerAction {
    /// No interaction this frame.
    #[default]
    None,
    /// The popover opened.
    Opened,
    /// The popover closed.
    Closed,
    /// A complete range was committed. Mirrors `DateRangeChanged`.
    RangeSelected(DateRange),
    /// The range was cleared.
    Cleared,
}

impl DateRangePickerAction {
    /// `true` when the bound range changed (`RangeSelected` / `Cleared`).
    #[inline]
    pub fn changed_value(&self) -> bool {
        matches!(self, Self::RangeSelected(_) | Self::Cleared)
    }

    /// `true` when the popover open/close state changed.
    #[inline]
    pub fn toggled_open(&self) -> bool {
        matches!(self, Self::Opened | Self::Closed)
    }

    /// The committed range, when this frame committed one.
    #[inline]
    pub fn selected(&self) -> Option<DateRange> {
        match self {
            Self::RangeSelected(r) => Some(*r),
            _ => None,
        }
    }

    /// Human-readable name, matching the variant.
    #[inline]
    pub fn name(&self) -> &'static str {
        match self {
            Self::None => "None",
            Self::Opened => "Opened",
            Self::Closed => "Closed",
            Self::RangeSelected(_) => "RangeSelected",
            Self::Cleared => "Cleared",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inputs::pickers::mud_date_picker::SimpleDate;

    fn d(y: i32, m: u8, day: u8) -> SimpleDate {
        SimpleDate::new(y, m, day)
    }

    fn range() -> DateRange {
        DateRange::new(Some(d(2024, 1, 1)), Some(d(2024, 1, 31)))
    }

    #[test]
    fn default_is_none() {
        assert_eq!(DateRangePickerAction::default(), DateRangePickerAction::None);
    }

    #[test]
    fn changed_value_only_for_range_mutations() {
        assert!(DateRangePickerAction::RangeSelected(range()).changed_value());
        assert!(DateRangePickerAction::Cleared.changed_value());
        assert!(!DateRangePickerAction::None.changed_value());
        assert!(!DateRangePickerAction::Opened.changed_value());
        assert!(!DateRangePickerAction::Closed.changed_value());
    }

    #[test]
    fn toggled_open_only_for_open_and_close() {
        assert!(DateRangePickerAction::Opened.toggled_open());
        assert!(DateRangePickerAction::Closed.toggled_open());
        assert!(!DateRangePickerAction::RangeSelected(range()).toggled_open());
    }

    #[test]
    fn selected_returns_the_range_only_for_range_selected() {
        assert_eq!(DateRangePickerAction::RangeSelected(range()).selected(), Some(range()));
        assert_eq!(DateRangePickerAction::None.selected(), None);
        assert_eq!(DateRangePickerAction::Cleared.selected(), None);
    }

    #[test]
    fn names_match_variants() {
        assert_eq!(DateRangePickerAction::None.name(), "None");
        assert_eq!(DateRangePickerAction::Opened.name(), "Opened");
        assert_eq!(DateRangePickerAction::Closed.name(), "Closed");
        assert_eq!(
            DateRangePickerAction::RangeSelected(range()).name(),
            "RangeSelected"
        );
        assert_eq!(DateRangePickerAction::Cleared.name(), "Cleared");
    }
}