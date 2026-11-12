//! `StepperInteraction` — the Rust equivalent of MudBlazor's
//! `StepperInteractionEventArgs`.
//!
//! Port of `StepperInteractionEventArgs` (MudBlazor 8.13.0). Passed to
//! [`MudStepper::on_preview_interaction`](super::MudStepper::on_preview_interaction)
//! so the caller can veto a step change.

use mud_egui_core::enums::StepAction;

/// A request to change the active step.
///
/// The caller receives this by `&mut` and may set
/// [`cancel`](Self::cancel) (or call [`prevent`](Self::prevent)) to stop the
/// change. This mirrors MudBlazor's `StepperInteractionEventArgs.Cancel`.
///
/// # Example
///
/// ```ignore
/// MudStepper::new().on_preview_interaction(|i| {
///     if i.action == StepAction::Complete && !form_is_valid() {
///         i.prevent();
///     }
/// });
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StepperInteraction {
    /// The 0-based index of the step the action targets.
    pub step_index: usize,
    /// The requested action.
    pub action: StepAction,
    /// When `true`, the action is discarded and the stepper does not change.
    pub cancel: bool,
}

impl StepperInteraction {
    /// Creates a non-cancelled interaction.
    pub fn new(step_index: usize, action: StepAction) -> Self {
        Self {
            step_index,
            action,
            cancel: false,
        }
    }

    /// Sets [`cancel`](Self::cancel) to `true`.
    #[inline]
    pub fn prevent(&mut self) {
        self.cancel = true;
    }

    /// Whether the interaction has been cancelled.
    #[inline]
    pub fn is_prevented(&self) -> bool {
        self.cancel
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_is_not_cancelled() {
        let i = StepperInteraction::new(2, StepAction::Complete);
        assert_eq!(i.step_index, 2);
        assert_eq!(i.action, StepAction::Complete);
        assert!(!i.cancel);
        assert!(!i.is_prevented());
    }

    #[test]
    fn prevent_sets_cancel() {
        let mut i = StepperInteraction::new(0, StepAction::Skip);
        i.prevent();
        assert!(i.cancel);
        assert!(i.is_prevented());
    }
}