//! `StepAction` — the requested behaviour of a stepper button.
//!
//! Port of `MudBlazor.StepAction` (MudBlazor 8.13.0). Passed to
//! `MudStepper::OnPreviewInteraction` so the caller can inspect *why* a step
//! change is being requested.

/// A requested step-change action.
///
/// Numeric values match the C# enum exactly (`Activate = 0`, `Complete = 1`,
/// `Skip = 2`, `Reset = 3`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StepAction {
    /// A request to activate a step (via Previous or a non-linear click).
    #[default]
    Activate = 0,
    /// A request to complete the current step and advance.
    Complete = 1,
    /// A request to skip the current step and advance.
    Skip = 2,
    /// A request to reset the stepper to the first step.
    Reset = 3,
}

impl StepAction {
    /// The C# discriminant (`Activate = 0`, `Complete = 1`, `Skip = 2`,
    /// `Reset = 3`).
    #[inline]
    pub const fn discriminant(self) -> u8 {
        self as u8
    }

    /// Human-readable name matching the C# enum member.
    #[inline]
    pub fn name(self) -> &'static str {
        match self {
            Self::Activate => "Activate",
            Self::Complete => "Complete",
            Self::Skip => "Skip",
            Self::Reset => "Reset",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_activate() {
        assert_eq!(StepAction::default(), StepAction::Activate);
    }

    #[test]
    fn discriminants_match_mudblazor() {
        assert_eq!(StepAction::Activate.discriminant(), 0);
        assert_eq!(StepAction::Complete.discriminant(), 1);
        assert_eq!(StepAction::Skip.discriminant(), 2);
        assert_eq!(StepAction::Reset.discriminant(), 3);
    }

    #[test]
    fn names_match_variants() {
        assert_eq!(StepAction::Activate.name(), "Activate");
        assert_eq!(StepAction::Complete.name(), "Complete");
        assert_eq!(StepAction::Skip.name(), "Skip");
        assert_eq!(StepAction::Reset.name(), "Reset");
    }
}