//! `TabAction` — what a [`MudTabs`](super::MudTabs) click did.
//!
//! Kept in its own file per the project convention that every enum lives in
//! its own file. Public because it is a field of
//! [`TabsResponse`](super::TabsResponse).
//!
//! **Extension**: MudBlazor only fires `ActivePanelIndexChanged` and
//! `OnPreviewInteraction`; the port also returns a rich action so callers can
//! log or test without wiring a callback.

/// The outcome of a [`MudTabs::show`](super::MudTabs::show) frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TabAction {
    /// No click this frame, or the click did not change the active tab.
    #[default]
    None,
    /// A tab became active.
    Activated {
        /// The previously active index.
        from: usize,
        /// The newly active index.
        to: usize,
    },
    /// A tab activation was requested but `on_preview_interaction` cancelled
    /// it.
    Vetoed {
        /// The index that the user tried to activate.
        target: usize,
    },
}

impl TabAction {
    /// Whether the action changed the active index.
    #[inline]
    pub fn changed_active_index(self) -> bool {
        matches!(self, Self::Activated { .. })
    }

    /// Whether the action was vetoed.
    #[inline]
    pub fn was_vetoed(self) -> bool {
        matches!(self, Self::Vetoed { .. })
    }

    /// The target index of the action, if any.
    #[inline]
    pub fn target_index(self) -> Option<usize> {
        match self {
            Self::None => None,
            Self::Activated { to, .. } => Some(to),
            Self::Vetoed { target } => Some(target),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_none() {
        assert_eq!(TabAction::default(), TabAction::None);
    }

    #[test]
    fn none_is_inert() {
        let a = TabAction::None;
        assert!(!a.changed_active_index());
        assert!(!a.was_vetoed());
        assert!(a.target_index().is_none());
    }

    #[test]
    fn activated_reports_target() {
        let a = TabAction::Activated { from: 0, to: 2 };
        assert!(a.changed_active_index());
        assert!(!a.was_vetoed());
        assert_eq!(a.target_index(), Some(2));
    }

    #[test]
    fn vetoed_reports_target() {
        let a = TabAction::Vetoed { target: 3 };
        assert!(!a.changed_active_index());
        assert!(a.was_vetoed());
        assert_eq!(a.target_index(), Some(3));
    }
}