//! What a [`MudPagination`](super::MudPagination) click did.
//!
//! Kept in its own file per the project convention that every enum lives in
//! its own file. Public because it is a field of
//! [`PaginationResponse`](super::PaginationResponse).

use mud_egui_core::enums::PageControl;

/// The outcome of the most recent click on a pagination button.
///
/// [`Self::None`] on every frame where nothing happened, so callers can match
/// without wrapping in `Option`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PaginationAction {
    /// No click this frame, or the pagination is disabled.
    #[default]
    None,

    /// A page number button was clicked. The value is 1-based.
    Page(u32),

    /// A control button (First / Previous / Next / Last) was clicked.
    Control(PageControl),
}

impl PaginationAction {
    /// Whether the action triggered a page change.
    #[inline]
    pub fn is_change(self) -> bool {
        !matches!(self, Self::None)
    }

    /// Whether the action was a control button.
    #[inline]
    pub fn is_control(self) -> bool {
        matches!(self, Self::Control(_))
    }

    /// The target page number (1-based), when the action resolved to one.
    ///
    /// For [`Self::Control`] the caller needs the `PageControl` variant to
    /// know the target (it depends on the current page), so this returns
    /// `None` for control actions.
    #[inline]
    pub fn target_page(self) -> Option<u32> {
        match self {
            Self::Page(n) => Some(n),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_none() {
        assert_eq!(PaginationAction::default(), PaginationAction::None);
    }

    #[test]
    fn none_is_inert() {
        let a = PaginationAction::None;
        assert!(!a.is_change());
        assert!(!a.is_control());
        assert!(a.target_page().is_none());
    }

    #[test]
    fn page_reports_target() {
        let a = PaginationAction::Page(7);
        assert!(a.is_change());
        assert!(!a.is_control());
        assert_eq!(a.target_page(), Some(7));
    }

    #[test]
    fn control_reports_no_target() {
        let a = PaginationAction::Control(PageControl::Next);
        assert!(a.is_change());
        assert!(a.is_control());
        assert!(a.target_page().is_none());
    }
}