//! Caller-owned state for [`MudDateRangePicker`](super::MudDateRangePicker).
//!
//! Holds the committed [`DateRange`], the two trigger texts, the popover
//! state, the calendar view, and the **pending start** that bridges the two
//! clicks of one selection. Persist it across frames — the same
//! caller-owned-state pattern `MudDatePicker`, `MudTimePicker`, `MudTable`
//! and the rest of the library use.

use crate::feedback::popover::PopoverState;
use crate::inputs::pickers::mud_date_picker::{DatePickerView, SimpleDate};

use super::date_range::DateRange;

/// Caller-owned date-range-picker state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DateRangePickerState {
    /// Committed range. Mirrors `DateRange`.
    pub range: DateRange,
    /// Text of the start trigger field.
    pub text_start: String,
    /// Text of the end trigger field.
    pub text_end: String,
    /// Popover open/closed state and stable id.
    pub popover: PopoverState,
    /// First day of the month currently displayed in the calendar.
    pub view_month: SimpleDate,
    /// First year of the decade displayed in the year grid.
    pub view_year_decade: i32,
    /// Current zoom level.
    pub view: DatePickerView,
    /// The first click of a two-click selection, held until the second
    /// click. Mirrors MudBlazor's private `_firstDate`.
    pub pending_start: Option<SimpleDate>,
}

impl DateRangePickerState {
    /// Creates empty state pointing at `initial_view_month`, closed.
    pub fn new(
        id_salt: impl std::hash::Hash + std::fmt::Debug,
        initial_view_month: SimpleDate,
    ) -> Self {
        Self {
            range: DateRange::empty(),
            text_start: String::new(),
            text_end: String::new(),
            popover: PopoverState::new(id_salt),
            view_month: initial_view_month.first_of_this_month(),
            view_year_decade: initial_view_month.year.div_euclid(10) * 10,
            view: DatePickerView::Day,
            pending_start: None,
        }
    }

    /// Seeds the range and both trigger texts.
    pub fn with_range(
        mut self,
        range: DateRange,
        text_start: impl Into<String>,
        text_end: impl Into<String>,
    ) -> Self {
        self.range = range.normalized();
        self.text_start = text_start.into();
        self.text_end = text_end.into();
        if let Some(start) = self.range.start {
            self.view_month = start.first_of_this_month();
            self.view_year_decade = start.year.div_euclid(10) * 10;
        }
        self
    }

    /// Whether the popover is open.
    #[inline]
    pub fn is_open(&self) -> bool {
        self.popover.open
    }

    /// Opens the popover on the day view, re-syncing the month to the
    /// committed start (if any) and starting a fresh two-click selection.
    pub fn open(&mut self) {
        self.popover.open = true;
        self.pending_start = None;
        self.view = DatePickerView::Day;
        if let Some(start) = self.range.start {
            self.view_month = start.first_of_this_month();
            self.view_year_decade = start.year.div_euclid(10) * 10;
        }
    }

    /// Closes the popover and drops any half-finished selection.
    pub fn close(&mut self) {
        self.popover.open = false;
        self.pending_start = None;
    }

    /// Toggles the popover.
    pub fn toggle(&mut self) {
        if self.popover.open {
            self.close();
        } else {
            self.open();
        }
    }

    /// Commits `range` (normalised) and its two display texts. Closes the
    /// popover and drops the pending start.
    pub fn commit_range(
        &mut self,
        range: DateRange,
        text_start: impl Into<String>,
        text_end: impl Into<String>,
    ) {
        self.range = range.normalized();
        self.text_start = text_start.into();
        self.text_end = text_end.into();
        self.pending_start = None;
        self.popover.open = false;
    }

    /// Clears the range and both texts. Closes the popover.
    pub fn clear(&mut self) {
        self.range = DateRange::empty();
        self.text_start.clear();
        self.text_end.clear();
        self.pending_start = None;
        self.popover.open = false;
    }

    /// The range the calendar should paint.
    ///
    /// While a two-click selection is in progress the pending start replaces
    /// the committed start and the end is blank — MudBlazor's "the clicked
    /// date becomes `_firstDate` and `_secondDate` is cleared" rule.
    #[inline]
    pub fn display_range(&self) -> DateRange {
        match self.pending_start {
            Some(s) => DateRange::new(Some(s), None),
            None => self.range,
        }
    }

    /// The start day of the two-click selection in progress, if any.
    #[inline]
    pub fn first_click(&self) -> Option<SimpleDate> {
        self.pending_start
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u8, day: u8) -> SimpleDate {
        SimpleDate::new(y, m, day)
    }

    fn range() -> DateRange {
        DateRange::new(Some(d(2024, 3, 1)), Some(d(2024, 3, 10)))
    }

    // --- construction -----------------------------------------------------

    #[test]
    fn new_starts_empty_and_closed() {
        let s = DateRangePickerState::new("t", d(2024, 3, 15));
        assert!(s.range.is_empty());
        assert!(s.text_start.is_empty());
        assert!(s.text_end.is_empty());
        assert!(!s.is_open());
        assert_eq!(s.view_month, d(2024, 3, 1));
        assert_eq!(s.view_year_decade, 2020);
        assert_eq!(s.view, DatePickerView::Day);
        assert!(s.pending_start.is_none());
    }

    #[test]
    fn with_range_seeds_range_texts_and_view() {
        let s = DateRangePickerState::new("t", d(2020, 1, 1))
            .with_range(range(), "2024-03-01", "2024-03-10");
        assert_eq!(s.range, range());
        assert_eq!(s.text_start, "2024-03-01");
        assert_eq!(s.text_end, "2024-03-10");
        assert_eq!(s.view_month, d(2024, 3, 1));
        assert_eq!(s.view_year_decade, 2020);
    }

    #[test]
    fn with_range_normalises_a_reversed_pair() {
        let reversed = DateRange::new(Some(d(2024, 3, 10)), Some(d(2024, 3, 1)));
        let s = DateRangePickerState::new("t", d(2024, 1, 1)).with_range(reversed, "a", "b");
        assert_eq!(s.range, range());
    }

    // --- open / close -----------------------------------------------------

    #[test]
    fn open_sets_day_view_and_resyncs_month() {
        let mut s = DateRangePickerState::new("t", d(2020, 1, 1)).with_range(range(), "a", "b");
        s.view_month = d(1999, 1, 1);
        s.view = DatePickerView::Year;
        s.open();
        assert!(s.is_open());
        assert_eq!(s.view, DatePickerView::Day);
        assert_eq!(s.view_month, d(2024, 3, 1));
    }

    #[test]
    fn open_drops_a_pending_start() {
        let mut s = DateRangePickerState::new("t", d(2024, 1, 1));
        s.pending_start = Some(d(2024, 5, 5));
        s.open();
        assert!(s.pending_start.is_none());
    }

    #[test]
    fn close_drops_a_pending_start() {
        let mut s = DateRangePickerState::new("t", d(2024, 1, 1));
        s.open();
        s.pending_start = Some(d(2024, 5, 5));
        s.close();
        assert!(!s.is_open());
        assert!(s.pending_start.is_none());
    }

    #[test]
    fn toggle_flips_the_popover() {
        let mut s = DateRangePickerState::new("t", d(2024, 1, 1));
        s.toggle();
        assert!(s.is_open());
        s.toggle();
        assert!(!s.is_open());
    }

    // --- commit / clear ---------------------------------------------------

    #[test]
    fn commit_range_normalises_stores_and_closes() {
        let mut s = DateRangePickerState::new("t", d(2024, 1, 1));
        s.open();
        s.pending_start = Some(d(2024, 3, 10));
        s.commit_range(
            DateRange::new(Some(d(2024, 3, 10)), Some(d(2024, 3, 1))),
            "2024-03-01",
            "2024-03-10",
        );
        assert_eq!(s.range, range());
        assert_eq!(s.text_start, "2024-03-01");
        assert_eq!(s.text_end, "2024-03-10");
        assert!(!s.is_open());
        assert!(s.pending_start.is_none());
    }

    #[test]
    fn clear_drops_range_and_texts() {
        let mut s = DateRangePickerState::new("t", d(2024, 1, 1)).with_range(range(), "a", "b");
        s.open();
        s.clear();
        assert!(s.range.is_empty());
        assert!(s.text_start.is_empty());
        assert!(s.text_end.is_empty());
        assert!(!s.is_open());
    }

    // --- display ----------------------------------------------------------

    #[test]
    fn display_range_prefers_the_pending_start() {
        let mut s = DateRangePickerState::new("t", d(2024, 1, 1)).with_range(range(), "a", "b");
        assert_eq!(s.display_range(), range());
        s.pending_start = Some(d(2024, 6, 15));
        let display = s.display_range();
        assert_eq!(display.start, Some(d(2024, 6, 15)));
        assert_eq!(display.end, None);
        assert_eq!(s.first_click(), Some(d(2024, 6, 15)));
    }

    #[test]
    fn display_range_without_pending_is_the_committed_range() {
        let s = DateRangePickerState::new("t", d(2024, 1, 1)).with_range(range(), "a", "b");
        assert!(s.first_click().is_none());
        assert_eq!(s.display_range(), range());
    }
}