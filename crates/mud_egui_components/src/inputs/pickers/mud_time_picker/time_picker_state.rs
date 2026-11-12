//! Caller-owned state for [`MudTimePicker`](super::MudTimePicker).
//!
//! Holds the committed value, the trigger text, the popover state, the dial
//! currently on screen, and the *pending hour* that bridges the hour and
//! minute steps of one selection. Persist it across frames — the same
//! caller-owned-state pattern `MudTable`, `MudTreeView`, `MudDatePicker` and
//! the rest of the picker family use.

use mud_egui_core::enums::OpenTo;

use crate::feedback::popover::PopoverState;

use super::simple_time::SimpleTime;
use super::time_picker_view::TimePickerView;

/// Caller-owned time-picker state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimePickerState {
    /// Committed value, if any. Mirrors `Time`.
    pub value: Option<SimpleTime>,
    /// Trigger text. Kept in sync with `value` by `commit` / `clear` /
    /// `with_value`; **not** re-synced from `value` on every frame, so typed
    /// edits in `editable(true)` mode are preserved.
    pub text: String,
    /// Popover open/closed state and stable id.
    pub popover: PopoverState,
    /// Dial currently on screen.
    pub view: TimePickerView,
    /// The hour chosen in the hour step, held until the minute step
    /// completes. Cleared on commit.
    pub pending_hour: Option<u8>,
}

impl TimePickerState {
    /// Creates empty state, closed, on the hour dial.
    pub fn new(id_salt: impl std::hash::Hash + std::fmt::Debug) -> Self {
        Self {
            value: None,
            text: String::new(),
            popover: PopoverState::new(id_salt),
            view: TimePickerView::Hours,
            pending_hour: None,
        }
    }

    /// Seeds the value and its display text.
    pub fn with_value(
        mut self,
        value: SimpleTime,
        text: impl Into<String>,
    ) -> Self {
        self.value = Some(value);
        self.text = text.into();
        self
    }

    /// Whether the popover is open.
    #[inline]
    pub fn is_open(&self) -> bool {
        self.popover.open
    }

    /// Opens the popover at `open_to` and clears the pending hour.
    pub fn open(&mut self, open_to: OpenTo) {
        self.popover.open = true;
        self.view = view_for_open(open_to);
        self.pending_hour = None;
    }

    /// Closes the popover and drops the pending hour.
    pub fn close(&mut self) {
        self.popover.open = false;
        self.pending_hour = None;
    }

    /// Toggles the popover.
    pub fn toggle(&mut self, open_to: OpenTo) {
        if self.popover.open {
            self.close();
        } else {
            self.open(open_to);
        }
    }

    /// Commits `value` and updates the display text. Closes the popover.
    pub fn commit(&mut self, value: SimpleTime, text: impl Into<String>) {
        self.value = Some(value);
        self.text = text.into();
        self.pending_hour = None;
        self.close();
    }

    /// Clears the value and text. Closes the popover.
    pub fn clear(&mut self) {
        self.value = None;
        self.text.clear();
        self.pending_hour = None;
        self.close();
    }

    /// The hour a minute click should commit: the pending hour if the user
    /// already chose one, else the current value's hour, else `0`.
    #[inline]
    pub fn effective_hour(&self) -> u8 {
        self.pending_hour
            .or_else(|| self.value.map(|t| t.hours))
            .unwrap_or(0)
    }
}

/// Maps `OpenTo` to the initial dial. `Hours` and anything that is not
/// `Minutes` fall through to [`TimePickerView::Hours`].
#[inline]
pub fn view_for_open(open_to: OpenTo) -> TimePickerView {
    match open_to {
        OpenTo::Minutes => TimePickerView::Minutes,
        _ => TimePickerView::Hours,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- initial state ----------------------------------------------------

    #[test]
    fn new_starts_empty_closed_and_on_hours() {
        let s = TimePickerState::new("t");
        assert!(s.value.is_none());
        assert!(s.text.is_empty());
        assert!(!s.is_open());
        assert_eq!(s.view, TimePickerView::Hours);
        assert!(s.pending_hour.is_none());
    }

    #[test]
    fn with_value_seeds_value_and_text() {
        let s = TimePickerState::new("t").with_value(SimpleTime::hm(9, 30), "09:30");
        assert_eq!(s.value, Some(SimpleTime::hm(9, 30)));
        assert_eq!(s.text, "09:30");
    }

    // --- open / close -----------------------------------------------------

    #[test]
    fn open_sets_view_from_open_to() {
        let mut s = TimePickerState::new("t");
        s.open(OpenTo::Minutes);
        assert!(s.is_open());
        assert_eq!(s.view, TimePickerView::Minutes);
        s.close();
        s.open(OpenTo::Hours);
        assert_eq!(s.view, TimePickerView::Hours);
    }

    #[test]
    fn toggle_flips_the_popover() {
        let mut s = TimePickerState::new("t");
        s.toggle(OpenTo::Hours);
        assert!(s.is_open());
        s.toggle(OpenTo::Hours);
        assert!(!s.is_open());
    }

    #[test]
    fn open_resets_pending_hour() {
        let mut s = TimePickerState::new("t");
        s.pending_hour = Some(7);
        s.open(OpenTo::Hours);
        assert!(s.pending_hour.is_none());
    }

    // --- commit / clear ---------------------------------------------------

    #[test]
    fn commit_sets_value_text_and_closes() {
        let mut s = TimePickerState::new("t");
        s.open(OpenTo::Hours);
        s.pending_hour = Some(14);
        s.commit(SimpleTime::hm(14, 45), "14:45");
        assert_eq!(s.value, Some(SimpleTime::hm(14, 45)));
        assert_eq!(s.text, "14:45");
        assert!(!s.is_open());
        assert!(s.pending_hour.is_none());
    }

    #[test]
    fn clear_drops_value_and_text() {
        let mut s = TimePickerState::new("t").with_value(SimpleTime::hm(9, 0), "09:00");
        s.open(OpenTo::Hours);
        s.clear();
        assert!(s.value.is_none());
        assert!(s.text.is_empty());
        assert!(!s.is_open());
    }

    // --- effective hour ---------------------------------------------------

    #[test]
    fn effective_hour_prefers_pending_then_value_then_zero() {
        let mut s = TimePickerState::new("t");
        assert_eq!(s.effective_hour(), 0);
        s.value = Some(SimpleTime::hm(17, 0));
        assert_eq!(s.effective_hour(), 17);
        s.pending_hour = Some(8);
        assert_eq!(s.effective_hour(), 8);
    }

    // --- view_for_open ----------------------------------------------------

    #[test]
    fn view_for_open_maps_minutes_and_defaults_to_hours() {
        assert_eq!(view_for_open(OpenTo::Minutes), TimePickerView::Minutes);
        assert_eq!(view_for_open(OpenTo::Hours), TimePickerView::Hours);
        assert_eq!(view_for_open(OpenTo::Day), TimePickerView::Hours);
        assert_eq!(view_for_open(OpenTo::Year), TimePickerView::Hours);
    }
}