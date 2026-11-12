//! `DatePickerAction` — what a [`MudDatePicker`](super::MudDatePicker) did this frame.
//!
//! **Extension**: MudBlazor reports the equivalent outcomes through three
//! separate callbacks (`PickerOpened`, `PickerClosed`, `DateChanged`). The port
//! promotes the outcome to a single enum so callers can read it from the return
//! value without registering three callbacks. This is documented in the README
//! under "Differences from MudBlazor".

use super::simple_date::SimpleDate;

/// What a [`MudDatePicker`](super::MudDatePicker) did this frame.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum DatePickerAction {
    /// No interaction this frame.
    #[default]
    None,
    /// The popover opened.
    Opened,
    /// The popover closed.
    Closed,
    /// A day was picked and committed as the new value.
    DateSelected(SimpleDate),
    /// The clear button / programmatic `clear` was triggered.
    Cleared,
}

impl DatePickerAction {
    /// `true` when the bound value was mutated (`DateSelected` / `Cleared`).
    #[inline]
    pub fn changed_value(&self) -> bool {
        matches!(self, Self::DateSelected(_) | Self::Cleared)
    }

    /// `true` when the popover open/close state changed.
    #[inline]
    pub fn toggled_open(&self) -> bool {
        matches!(self, Self::Opened | Self::Closed)
    }

    /// Human-readable name, matching the variant.
    #[inline]
    pub fn name(&self) -> &'static str {
        match self {
            Self::None => "None",
            Self::Opened => "Opened",
            Self::Closed => "Closed",
            Self::DateSelected(_) => "DateSelected",
            Self::Cleared => "Cleared",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u8, day: u8) -> SimpleDate {
        SimpleDate::new(y, m, day)
    }

    #[test]
    fn default_is_none() {
        assert_eq!(DatePickerAction::default(), DatePickerAction::None);
    }

    #[test]
    fn changed_value_true_only_for_value_mutations() {
        assert!(DatePickerAction::DateSelected(d(2024, 1, 1)).changed_value());
        assert!(DatePickerAction::Cleared.changed_value());
        assert!(!DatePickerAction::None.changed_value());
        assert!(!DatePickerAction::Opened.changed_value());
        assert!(!DatePickerAction::Closed.changed_value());
    }

    #[test]
    fn toggled_open_true_only_for_open_close() {
        assert!(DatePickerAction::Opened.toggled_open());
        assert!(DatePickerAction::Closed.toggled_open());
        assert!(!DatePickerAction::None.toggled_open());
        assert!(!DatePickerAction::DateSelected(d(2024, 1, 1)).toggled_open());
        assert!(!DatePickerAction::Cleared.toggled_open());
    }

    #[test]
    fn names_match_variants() {
        assert_eq!(DatePickerAction::None.name(), "None");
        assert_eq!(DatePickerAction::Opened.name(), "Opened");
        assert_eq!(DatePickerAction::Closed.name(), "Closed");
        assert_eq!(
            DatePickerAction::DateSelected(d(2024, 1, 1)).name(),
            "DateSelected"
        );
        assert_eq!(DatePickerAction::Cleared.name(), "Cleared");
    }
}