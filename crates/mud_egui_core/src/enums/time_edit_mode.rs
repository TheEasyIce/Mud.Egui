//! `TimeEditMode` — which parts of a time the picker lets the user edit.
//!
//! Port of `MudBlazor.TimeEditMode` (MudBlazor 8.13.0). The numeric
//! discriminants match the C# enum, which declares `OnlyMinutes = 1` before
//! `OnlyHours = 2` in the docs but assigns the same values[reference:4].
//!
//! Kept in its own file per the project's one-enum-per-file rule (Rule 2).

use std::fmt;

/// Which values a [`MudTimePicker`](crate::MudTimePicker) lets the user edit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TimeEditMode {
    /// Both hours and minutes can be edited. The default.
    #[default]
    Normal = 0,
    /// Only minutes can be edited; the hour comes from the current value.
    OnlyMinutes = 1,
    /// Only hours can be edited; the minute comes from the current value.
    OnlyHours = 2,
}

impl TimeEditMode {
    /// Every variant, in discriminant order.
    pub const ALL: [Self; 3] = [Self::Normal, Self::OnlyMinutes, Self::OnlyHours];

    /// Lower-case name, matching the variant.
    pub fn name(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::OnlyMinutes => "only-minutes",
            Self::OnlyHours => "only-hours",
        }
    }

    /// `true` for `OnlyHours`: the hour click commits immediately.
    #[inline]
    pub fn commits_after_hour(self) -> bool {
        matches!(self, Self::OnlyHours)
    }

    /// `true` for `OnlyMinutes`: the picker opens on the minute ring.
    #[inline]
    pub fn opens_on_minutes(self) -> bool {
        matches!(self, Self::OnlyMinutes)
    }
}

impl fmt::Display for TimeEditMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_normal() {
        assert_eq!(TimeEditMode::default(), TimeEditMode::Normal);
    }

    #[test]
    fn discriminants_match_mudblazor() {
        assert_eq!(TimeEditMode::Normal as i32, 0);
        assert_eq!(TimeEditMode::OnlyMinutes as i32, 1);
        assert_eq!(TimeEditMode::OnlyHours as i32, 2);
    }

    #[test]
    fn all_has_three_unique_variants() {
        assert_eq!(TimeEditMode::ALL.len(), 3);
        let names: Vec<_> = TimeEditMode::ALL.iter().map(|v| v.name()).collect();
        let mut sorted = names.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), 3);
    }

    #[test]
    fn display_matches_name() {
        for v in TimeEditMode::ALL {
            assert_eq!(v.to_string(), v.name());
        }
    }

    #[test]
    fn commit_and_open_predicates() {
        assert!(TimeEditMode::OnlyHours.commits_after_hour());
        assert!(!TimeEditMode::Normal.commits_after_hour());
        assert!(!TimeEditMode::OnlyMinutes.commits_after_hour());
        assert!(TimeEditMode::OnlyMinutes.opens_on_minutes());
        assert!(!TimeEditMode::Normal.opens_on_minutes());
        assert!(!TimeEditMode::OnlyHours.opens_on_minutes());
    }
}