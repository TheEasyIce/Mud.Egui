//! `StepPhase` — the resolved visual state of one step.
//!
//! **Extension**: MudBlazor derives this state inside the renderer without
//! exposing it. The port promotes it to a public enum because it is the
//! natural argument to `LabelTemplate` / `TitleTemplate` closures and to the
//! pure helper [`step_phase`](super::step_phase) that tests cover.
//!
//! Kept in its own file per the project convention that every enum lives in
//! its own file.

/// The visual phase of a step, derived from its [`StepFlags`](super::StepFlags)
/// and whether it is the active step.
///
/// Priority order (highest first):
///
/// 1. [`Error`](Self::Error) — `HasError == true`
/// 2. [`Skipped`](Self::Skipped) — `Skipped == true`
/// 3. [`Completed`](Self::Completed) — `Completed == true`
/// 4. [`Active`](Self::Active) — the currently active step
/// 5. [`Pending`](Self::Pending) — everything else
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StepPhase {
    /// Not yet reached.
    #[default]
    Pending,
    /// The currently active step.
    Active,
    /// Completed successfully.
    Completed,
    /// Has an error.
    Error,
    /// Skipped by the user.
    Skipped,
}

impl StepPhase {
    /// Whether the step's circle should be filled with an accent colour.
    #[inline]
    pub fn is_filled(self) -> bool {
        !matches!(self, Self::Pending)
    }

    /// Whether the step should show an icon instead of its index.
    #[inline]
    pub fn shows_icon(self) -> bool {
        matches!(self, Self::Completed | Self::Error | Self::Skipped)
    }

    /// Human-readable name matching the variant.
    #[inline]
    pub fn name(self) -> &'static str {
        match self {
            Self::Pending => "Pending",
            Self::Active => "Active",
            Self::Completed => "Completed",
            Self::Error => "Error",
            Self::Skipped => "Skipped",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_pending() {
        assert_eq!(StepPhase::default(), StepPhase::Pending);
    }

    #[test]
    fn pending_is_not_filled_and_shows_no_icon() {
        assert!(!StepPhase::Pending.is_filled());
        assert!(!StepPhase::Pending.shows_icon());
    }

    #[test]
    fn other_phases_are_filled() {
        for p in [
            StepPhase::Active,
            StepPhase::Completed,
            StepPhase::Error,
            StepPhase::Skipped,
        ] {
            assert!(p.is_filled(), "{p:?} should be filled");
        }
    }

    #[test]
    fn only_completed_error_skipped_show_icons() {
        assert!(!StepPhase::Active.shows_icon());
        assert!(StepPhase::Completed.shows_icon());
        assert!(StepPhase::Error.shows_icon());
        assert!(StepPhase::Skipped.shows_icon());
    }

    #[test]
    fn names_match_variants() {
        assert_eq!(StepPhase::Pending.name(), "Pending");
        assert_eq!(StepPhase::Active.name(), "Active");
        assert_eq!(StepPhase::Completed.name(), "Completed");
        assert_eq!(StepPhase::Error.name(), "Error");
        assert_eq!(StepPhase::Skipped.name(), "Skipped");
    }
}