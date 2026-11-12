//! `TimePickerAction` — what a [`MudTimePicker`](super::MudTimePicker) did
//! this frame.
//!
//! **Extension**: MudBlazor reports the outcomes through three callbacks
//! (`PickerOpened`, `PickerClosed`, `TimeChanged`). The port promotes the
//! outcome to one enum so a caller can read it from the return value.
//! Kept in its own file per Rule 2.

use super::simple_time::SimpleTime;

/// What a [`MudTimePicker`](super::MudTimePicker) did this frame.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum TimePickerAction {
    /// No interaction this frame.
    #[default]
    None,
    /// The popover opened.
    Opened,
    /// The popover closed.
    Closed,
    /// A time was picked and committed. Mirrors `TimeChanged`.
    TimeSelected(SimpleTime),
    /// The value was cleared.
    Cleared,
}

impl TimePickerAction {
    /// `true` when the bound value changed (`TimeSelected` / `Cleared`).
    #[inline]
    pub fn changed_value(&self) -> bool {
        matches!(self, Self::TimeSelected(_) | Self::Cleared)
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
            Self::TimeSelected(_) => "TimeSelected",
            Self::Cleared => "Cleared",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_none() {
        assert_eq!(TimePickerAction::default(), TimePickerAction::None);
    }

    #[test]
    fn changed_value_only_for_value_mutations() {
        assert!(TimePickerAction::TimeSelected(SimpleTime::hm(9, 0)).changed_value());
        assert!(TimePickerAction::Cleared.changed_value());
        assert!(!TimePickerAction::None.changed_value());
        assert!(!TimePickerAction::Opened.changed_value());
        assert!(!TimePickerAction::Closed.changed_value());
    }

    #[test]
    fn toggled_open_only_for_open_close() {
        assert!(TimePickerAction::Opened.toggled_open());
        assert!(TimePickerAction::Closed.toggled_open());
        assert!(!TimePickerAction::None.toggled_open());
        assert!(!TimePickerAction::TimeSelected(SimpleTime::hm(9, 0)).toggled_open());
    }

    #[test]
    fn names_match_variants() {
        assert_eq!(TimePickerAction::None.name(), "None");
        assert_eq!(TimePickerAction::Opened.name(), "Opened");
        assert_eq!(TimePickerAction::Closed.name(), "Closed");
        assert_eq!(
            TimePickerAction::TimeSelected(SimpleTime::hm(9, 0)).name(),
            "TimeSelected"
        );
        assert_eq!(TimePickerAction::Cleared.name(), "Cleared");
    }
}