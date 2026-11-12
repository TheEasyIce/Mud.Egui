//! Caller-owned state for [`MudStepper`](super::MudStepper).

use super::step_flags::StepFlags;

/// Caller-owned stepper state.
///
/// This is the egui equivalent of MudBlazor's bindable `ActiveIndex` plus the
/// per-step bindable `Completed` / `HasError` / `Skipped` parameters.
///
/// Persist it across frames:
///
/// ```ignore
/// struct MyPage { stepper: StepperState }
///
/// MudStepper::new().show(ui, &mut self.stepper, |s| {
///     s.step(MudStep::new("Account").completed(true));
///     s.step(MudStep::new("Profile"));
/// });
///
/// // Read the outcome:
/// if self.stepper.is_completed(2) { /* … */ }
/// ```
#[derive(Debug, Clone, Default)]
pub struct StepperState {
    /// 0-based index of the active step.
    pub active_index: usize,
    /// Per-step status flags, indexed by step position.
    ///
    /// Seeded once from `MudStep::completed(..)` / `has_error(..)` /
    /// `skipped(..)` when a step is first rendered; after that this Vec is
    /// authoritative and the builder flags are ignored.
    pub steps: Vec<StepFlags>,
}

impl StepperState {
    /// Creates state with `active_index = 0` and no steps.
    pub fn new() -> Self {
        Self::default()
    }

    /// Ensures a `StepFlags` entry exists for `index`, seeding it from the
    /// builder values on first render.
    pub fn ensure_step(&mut self, index: usize, completed: bool, has_error: bool, skipped: bool) {
        while self.steps.len() <= index {
            self.steps.push(StepFlags::default());
        }
        // Seed only if the entry was just created (all-false and index equals
        // the newly pushed slot). A user-toggled flag is non-false, so the
        // seed is a no-op after first render.
        let slot = &mut self.steps[index];
        if !slot.completed && !slot.has_error && !slot.skipped {
            slot.completed = completed;
            slot.has_error = has_error;
            slot.skipped = skipped;
        }
    }

    /// Flags for `index`, or [`StepFlags::default()`] if out of range.
    pub fn flags(&self, index: usize) -> StepFlags {
        self.steps.get(index).copied().unwrap_or_default()
    }

    /// Mutable flags for `index`, growing the Vec as needed.
    pub fn flags_mut(&mut self, index: usize) -> &mut StepFlags {
        while self.steps.len() <= index {
            self.steps.push(StepFlags::default());
        }
        &mut self.steps[index]
    }

    /// Sets `completed` for `index` (the port of `MudStep.SetCompletedAsync`).
    pub fn set_completed(&mut self, index: usize, v: bool) {
        self.flags_mut(index).completed = v;
    }

    /// Sets `has_error` for `index` (the port of `MudStep.SetHasErrorAsync`).
    pub fn set_has_error(&mut self, index: usize, v: bool) {
        self.flags_mut(index).has_error = v;
    }

    /// Sets `skipped` for `index` (the port of `MudStep.SetSkippedAsync`).
    pub fn set_skipped(&mut self, index: usize, v: bool) {
        self.flags_mut(index).skipped = v;
    }

    /// Whether every step in `0..count` is completed.
    ///
    /// Returns `false` when `count == 0` (a stepper with no steps is never
    /// "completed").
    pub fn is_completed(&self, count: usize) -> bool {
        count > 0 && (0..count).all(|i| self.flags(i).completed)
    }

    /// Resets `active_index` to `0` and clears every flag (the port of
    /// `MudStepper.ResetAsync`).
    pub fn reset(&mut self) {
        self.active_index = 0;
        for f in &mut self.steps {
            *f = StepFlags::default();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_starts_at_zero_with_no_steps() {
        let s = StepperState::new();
        assert_eq!(s.active_index, 0);
        assert!(s.steps.is_empty());
    }

    #[test]
    fn ensure_step_grows_and_seeds_once() {
        let mut s = StepperState::new();
        s.ensure_step(2, true, false, false);
        assert_eq!(s.steps.len(), 3);
        assert!(s.steps[2].completed);
        assert!(!s.steps[0].completed && !s.steps[1].completed);

        // Second call must not overwrite a user toggle.
        s.steps[2].completed = false;
        s.ensure_step(2, true, false, false);
        assert!(!s.steps[2].completed, "seed must not overwrite");
    }

    #[test]
    fn flags_returns_default_for_missing_index() {
        let s = StepperState::new();
        assert_eq!(s.flags(5), StepFlags::default());
    }

    #[test]
    fn setters_grow_as_needed() {
        let mut s = StepperState::new();
        s.set_completed(3, true);
        assert!(s.flags(3).completed);
        s.set_has_error(1, true);
        assert!(s.flags(1).has_error);
        s.set_skipped(0, true);
        assert!(s.flags(0).skipped);
    }

    #[test]
    fn is_completed_requires_all_and_non_empty() {
        let mut s = StepperState::new();
        assert!(!s.is_completed(0));
        s.set_completed(0, true);
        assert!(!s.is_completed(2), "one of two incomplete");
        s.set_completed(1, true);
        assert!(s.is_completed(2));
    }

    #[test]
    fn reset_clears_flags_and_index() {
        let mut s = StepperState::new();
        s.active_index = 2;
        s.set_completed(0, true);
        s.set_skipped(1, true);
        s.reset();
        assert_eq!(s.active_index, 0);
        assert!(!s.flags(0).completed);
        assert!(!s.flags(1).skipped);
    }
}