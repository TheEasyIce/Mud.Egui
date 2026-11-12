//! Caller-owned state for [`MudDatePicker`](super::MudDatePicker).

use crate::feedback::popover::PopoverState;

use super::date_picker_view::DatePickerView;
use super::simple_date::SimpleDate;

/// Caller-owned date-picker state.
///
/// Holds the committed value, the trigger's display text, the popover state,
/// and the calendar view (which month / year is on screen). Persist it across
/// frames — matching the caller-owned-state pattern used by `MudTable`,
/// `MudTreeView`, `MudSelect`, `MudAutocomplete`, `MudMenu`, `MudTooltip`,
/// `MudChipSet`, `MudNavMenu`, `MudPagination`, `MudStepper`, and `MudTabs`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatePickerState {
    /// Committed date, if any.
    pub value: Option<SimpleDate>,
    /// Trigger field text. Kept in sync with `value` by `commit` / `clear` /
    /// `with_value`; not re-synced from `value` on every frame, so typed
    /// edits in `editable(true)` mode are preserved.
    pub text: String,
    /// Popover open / closed state and stable id.
    pub popover: PopoverState,
    /// First day of the month currently displayed in the calendar.
    pub view_month: SimpleDate,
    /// First year of the decade displayed in the year grid.
    pub view_year_decade: i32,
    /// Current zoom level.
    pub view: DatePickerView,
}

impl DatePickerState {
    /// Creates empty state pointing at `initial_view_month`, closed.
    pub fn new(
        id_salt: impl std::hash::Hash + std::fmt::Debug,
        initial_view_month: SimpleDate,
    ) -> Self {
        Self {
            value: None,
            text: String::new(),
            popover: PopoverState::new(id_salt),
            view_month: initial_view_month.first_of_this_month(),
            view_year_decade: (initial_view_month.year.div_euclid(10)) * 10,
            view: DatePickerView::Day,
        }
    }

    /// Seeds the value and its display text.
    pub fn with_value(mut self, value: SimpleDate, text: impl Into<String>) -> Self {
        self.value = Some(value);
        self.text = text.into();
        self.view_month = value.first_of_this_month();
        self.view_year_decade = value.year.div_euclid(10) * 10;
        self
    }

    /// Whether the popover is open.
    #[inline]
    pub fn is_open(&self) -> bool {
        self.popover.open
    }

    /// Opens the popover, re-syncing the view to the selected value.
    pub fn open(&mut self) {
        self.popover.open = true;
        if let Some(v) = self.value {
            self.view_month = v.first_of_this_month();
            self.view_year_decade = v.year.div_euclid(10) * 10;
        }
        self.view = DatePickerView::Day;
    }

    /// Closes the popover.
    pub fn close(&mut self) {
        self.popover.open = false;
    }

    /// Toggles the popover.
    pub fn toggle(&mut self) {
        if self.popover.open {
            self.close();
        } else {
            self.open();
        }
    }

    /// Commits `value` and updates the display text. Closes the popover.
    pub fn commit(&mut self, value: SimpleDate, text: impl Into<String>) {
        self.value = Some(value);
        self.text = text.into();
        self.close();
    }

    /// Clears the value and text. Closes the popover.
    pub fn clear(&mut self) {
        self.value = None;
        self.text.clear();
        self.close();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u8, day: u8) -> SimpleDate {
        SimpleDate::new(y, m, day)
    }

    #[test]
    fn new_starts_empty_and_closed() {
        let s = DatePickerState::new("test", d(2024, 3, 15));
        assert!(s.value.is_none());
        assert!(s.text.is_empty());
        assert!(!s.is_open());
        assert_eq!(s.view_month, d(2024, 3, 1));
        assert_eq!(s.view_year_decade, 2020);
        assert_eq!(s.view, DatePickerView::Day);
    }

    #[test]
    fn with_value_seeds_value_text_and_view() {
        let s = DatePickerState::new("test", d(2020, 1, 1))
            .with_value(d(2025, 7, 4), "2025-07-04");
        assert_eq!(s.value, Some(d(2025, 7, 4)));
        assert_eq!(s.text, "2025-07-04");
        assert_eq!(s.view_month, d(2025, 7, 1));
        assert_eq!(s.view_year_decade, 2020);
    }

    #[test]
    fn open_resyncs_view() {
        let mut s = DatePickerState::new("t", d(2020, 1, 1))
            .with_value(d(2025, 7, 4), "x");
        s.view_month = d(2000, 1, 1);
        s.view_year_decade = 2000;
        s.view = DatePickerView::Year;
        s.open();
        assert!(s.is_open());
        assert_eq!(s.view_month, d(2025, 7, 1));
        assert_eq!(s.view_year_decade, 2020);
        assert_eq!(s.view, DatePickerView::Day, "open resets view to Day");
    }

    #[test]
    fn commit_updates_and_closes() {
        let mut s = DatePickerState::new("t", d(2024, 1, 1));
        s.open();
        s.commit(d(2024, 6, 15), "2024-06-15");
        assert_eq!(s.value, Some(d(2024, 6, 15)));
        assert_eq!(s.text, "2024-06-15");
        assert!(!s.is_open());
    }

    #[test]
    fn clear_empties_and_closes() {
        let mut s = DatePickerState::new("t", d(2024, 1, 1))
            .with_value(d(2024, 6, 15), "2024-06-15");
        s.open();
        s.clear();
        assert!(s.value.is_none());
        assert!(s.text.is_empty());
        assert!(!s.is_open());
    }

    #[test]
    fn toggle_flips() {
        let mut s = DatePickerState::new("t", d(2024, 1, 1));
        s.toggle();
        assert!(s.is_open());
        s.toggle();
        assert!(!s.is_open());
    }

    #[test]
    fn negative_year_decade_rounds_correctly() {
        let s = DatePickerState::new("t", d(-5, 1, 1));
        assert_eq!(s.view_year_decade, -10);
    }
}