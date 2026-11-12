//! `TimePickerView` — the two dials a time picker can show.
//!
//! MudBlazor tracks this internally; the port promotes it to a public enum
//! because it is a public field of
//! [`TimePickerState`](super::TimePickerState). Kept in its own file per
//! Rule 2.

/// The dial currently on screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TimePickerView {
    /// The hour ring (12-hour or 24-hour). The default.
    #[default]
    Hours,
    /// The minute ring.
    Minutes,
}

impl TimePickerView {
    /// Human-readable name, matching the variant.
    #[inline]
    pub fn name(self) -> &'static str {
        match self {
            Self::Hours => "Hours",
            Self::Minutes => "Minutes",
        }
    }

    /// The other view.
    #[inline]
    pub fn toggle(self) -> Self {
        match self {
            Self::Hours => Self::Minutes,
            Self::Minutes => Self::Hours,
        }
    }

    /// `true` for `Hours`.
    #[inline]
    pub fn is_hours(self) -> bool {
        matches!(self, Self::Hours)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_hours() {
        assert_eq!(TimePickerView::default(), TimePickerView::Hours);
    }

    #[test]
    fn toggle_flips() {
        assert_eq!(TimePickerView::Hours.toggle(), TimePickerView::Minutes);
        assert_eq!(TimePickerView::Minutes.toggle(), TimePickerView::Hours);
    }

    #[test]
    fn toggle_is_involutive() {
        for v in [TimePickerView::Hours, TimePickerView::Minutes] {
            assert_eq!(v.toggle().toggle(), v);
        }
    }

    #[test]
    fn names_match_variants() {
        assert_eq!(TimePickerView::Hours.name(), "Hours");
        assert_eq!(TimePickerView::Minutes.name(), "Minutes");
    }
}