//! Runtime status flags for one step.
//!
//! Kept in its own file per the project convention that every type lives in
//! its own file when it carries independent semantics. `StepFlags` is the
//! caller-owned state for `Completed` / `HasError` / `Skipped` on one
//! [`MudStep`](super::MudStep).

/// Runtime status flags for a single step.
///
/// These map to MudBlazor's bindable `MudStep.Completed` / `HasError` /
/// `Skipped` parameters. `Disabled` is **not** stored here — it is a static
/// property of [`MudStep`](super::MudStep) and the caller can change it by
/// rebuilding the step next frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct StepFlags {
    /// `MudStep.Completed`.
    pub completed: bool,
    /// `MudStep.HasError`.
    pub has_error: bool,
    /// `MudStep.Skipped`.
    pub skipped: bool,
}

impl StepFlags {
    /// Creates flags with all booleans set to `false`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Builder: sets `completed`.
    pub fn with_completed(mut self, b: bool) -> Self {
        self.completed = b;
        self
    }

    /// Builder: sets `has_error`.
    pub fn with_has_error(mut self, b: bool) -> Self {
        self.has_error = b;
        self
    }

    /// Builder: sets `skipped`.
    pub fn with_skipped(mut self, b: bool) -> Self {
        self.skipped = b;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_all_false() {
        let f = StepFlags::new();
        assert!(!f.completed && !f.has_error && !f.skipped);
    }

    #[test]
    fn builders_set_fields() {
        let f = StepFlags::new()
            .with_completed(true)
            .with_has_error(true)
            .with_skipped(true);
        assert!(f.completed && f.has_error && f.skipped);
    }
}