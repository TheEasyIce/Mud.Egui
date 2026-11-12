//! `TabInteraction` — the Rust equivalent of MudBlazor's
//! `TabInteractionEventArgs`.
//!
//! Port of `TabInteractionEventArgs` (MudBlazor 8.13.0). Passed to
//! [`MudTabs::on_preview_interaction`](super::MudTabs::on_preview_interaction).

/// A request to activate a tab.
///
/// The caller receives this by `&mut` and may set
/// [`cancel`](Self::cancel) (or call [`prevent`](Self::prevent)) to stop the
/// change. This mirrors MudBlazor's `TabInteractionEventArgs.Cancel`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TabInteraction {
    /// The 0-based index the action targets.
    pub target_index: usize,
    /// The currently active index when the interaction was created.
    pub from_index: usize,
    /// When `true`, the action is discarded.
    pub cancel: bool,
}

impl TabInteraction {
    /// Creates a non-cancelled interaction.
    pub fn new(from_index: usize, target_index: usize) -> Self {
        Self {
            target_index,
            from_index,
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
        let i = TabInteraction::new(0, 2);
        assert_eq!(i.from_index, 0);
        assert_eq!(i.target_index, 2);
        assert!(!i.cancel);
        assert!(!i.is_prevented());
    }

    #[test]
    fn prevent_sets_cancel() {
        let mut i = TabInteraction::new(1, 3);
        i.prevent();
        assert!(i.cancel);
        assert!(i.is_prevented());
    }
}