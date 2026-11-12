//! `StepperAction` — what a [`MudStepper`](super::MudStepper) did this frame.
//!
//! **Extension**: MudBlazor only fires callbacks (`ActiveIndexChanged`,
//! `OnPreviewInteraction`); the port also returns a rich action so callers
//! can log or test without wiring a callback.
//!
//! Kept in its own file per the project convention that every enum lives in
//! its own file.

/// The outcome of a [`MudStepper::show`](super::MudStepper::show) frame.
///
/// [`Self::None`] on frames where nothing happened, so callers can match
/// without wrapping in `Option`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StepperAction {
    /// No click this frame.
    #[default]
    None,
    /// A step became active (Previous button or non-linear step click).
    Activated {
        /// Index of the previously active step.
        from: usize,
        /// Index of the newly active step.
        to: usize,
    },
    /// The active step was completed and the stepper advanced.
    Completed {
        /// Index of the step that was completed.
        index: usize,
        /// Index of the newly active step (unchanged when already on the last).
        to: usize,
    },
    /// The active step was skipped and the stepper advanced.
    Skipped {
        /// Index of the step that was skipped.
        index: usize,
        /// Index of the newly active step.
        to: usize,
    },
    /// The stepper was reset to its initial state.
    Reset,
}

impl StepperAction {
    /// Whether the action changed the active step.
    #[inline]
    pub fn changed_active_index(self) -> bool {
        matches!(
            self,
            Self::Activated { .. } | Self::Completed { .. } | Self::Skipped { .. } | Self::Reset
        )
    }

    /// Whether the action completed or skipped a step.
    #[inline]
    pub fn advanced(self) -> bool {
        matches!(self, Self::Completed { .. } | Self::Skipped { .. })
    }

    /// The step that triggered the action, if any.
    #[inline]
    pub fn trigger_index(self) -> Option<usize> {
        match self {
            Self::None | Self::Reset => None,
            Self::Activated { from, .. } => Some(from),
            Self::Completed { index, .. } | Self::Skipped { index, .. } => Some(index),
        }
    }

    /// The step that became active, if any.
    #[inline]
    pub fn new_active_index(self) -> Option<usize> {
        match self {
            Self::None => None,
            Self::Reset => Some(0),
            Self::Activated { to, .. }
            | Self::Completed { to, .. }
            | Self::Skipped { to, .. } => Some(to),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_none() {
        assert_eq!(StepperAction::default(), StepperAction::None);
    }

    #[test]
    fn none_is_inert() {
        let a = StepperAction::None;
        assert!(!a.changed_active_index());
        assert!(!a.advanced());
        assert!(a.trigger_index().is_none());
        assert!(a.new_active_index().is_none());
    }

    #[test]
    fn activated_changes_index_but_does_not_advance() {
        let a = StepperAction::Activated { from: 2, to: 1 };
        assert!(a.changed_active_index());
        assert!(!a.advanced());
        assert_eq!(a.trigger_index(), Some(2));
        assert_eq!(a.new_active_index(), Some(1));
    }

    #[test]
    fn completed_and_skipped_advance() {
        let c = StepperAction::Completed { index: 0, to: 1 };
        let s = StepperAction::Skipped { index: 1, to: 2 };
        assert!(c.advanced() && s.advanced());
        assert_eq!(c.trigger_index(), Some(0));
        assert_eq!(s.trigger_index(), Some(1));
        assert_eq!(c.new_active_index(), Some(1));
        assert_eq!(s.new_active_index(), Some(2));
    }

    #[test]
    fn reset_targets_first_step() {
        let r = StepperAction::Reset;
        assert!(r.changed_active_index());
        assert!(!r.advanced());
        assert!(r.trigger_index().is_none());
        assert_eq!(r.new_active_index(), Some(0));
    }
}