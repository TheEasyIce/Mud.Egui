//! What a [`MudBreadcrumbs`](super::MudBreadcrumbs) frame did when an item
//! was clicked.
//!
//! Kept in its own file per the project convention that every enum lives in
//! its own file. Public because it is a field of [`BreadcrumbResponse`](
//! super::BreadcrumbResponse).

/// The outcome of the most recent click on a breadcrumb item.
///
/// [`Self::None`] on every frame where nothing happened, so callers can
/// match without wrapping in `Option`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum BreadcrumbAction {
    /// No click this frame, or the clicked item had no `href` and no
    /// navigation callback.
    #[default]
    None,

    /// The item navigated **internally** through the `NavigationManager`.
    Internal {
        /// Index of the clicked item in the source slice.
        index: usize,
        /// URI requested (as passed to `navigate_to`).
        uri: String,
    },

    /// The `NavigationManager` was asked to navigate but a guard vetoed it.
    Vetoed {
        /// Index of the clicked item.
        index: usize,
        /// URI that was vetoed.
        uri: String,
    },

    /// The `NavigationManager` failed (malformed URI, outside base,
    /// re-entrant call).
    Error {
        /// Index of the clicked item.
        index: usize,
        /// URI the caller asked for.
        uri: String,
        /// Human-readable reason.
        message: String,
    },

    /// The item opened an **external** URL in the system browser.
    External {
        /// Index of the clicked item.
        index: usize,
        /// URL that was opened.
        url: String,
    },
}

impl BreadcrumbAction {
    /// Whether the action performed any navigation (internal or external).
    #[inline]
    pub fn navigated(&self) -> bool {
        matches!(self, Self::Internal { .. } | Self::External { .. })
    }

    /// Whether the action was blocked (vetoed or errored).
    #[inline]
    pub fn was_blocked(&self) -> bool {
        matches!(self, Self::Vetoed { .. } | Self::Error { .. })
    }

    /// Whether the action opened the system browser.
    #[inline]
    pub fn is_external(&self) -> bool {
        matches!(self, Self::External { .. })
    }

    /// The target URI / URL, when there is one.
    #[inline]
    pub fn target(&self) -> Option<&str> {
        match self {
            Self::None => None,
            Self::Internal { uri, .. }
            | Self::Vetoed { uri, .. }
            | Self::Error { uri, .. } => Some(uri.as_str()),
            Self::External { url, .. } => Some(url.as_str()),
        }
    }

    /// The index of the item that triggered the action, when there is one.
    #[inline]
    pub fn index(&self) -> Option<usize> {
        match self {
            Self::None => None,
            Self::Internal { index, .. }
            | Self::Vetoed { index, .. }
            | Self::Error { index, .. }
            | Self::External { index, .. } => Some(*index),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_none() {
        assert_eq!(BreadcrumbAction::default(), BreadcrumbAction::None);
    }

    #[test]
    fn none_is_inert() {
        let a = BreadcrumbAction::None;
        assert!(!a.navigated());
        assert!(!a.was_blocked());
        assert!(!a.is_external());
        assert!(a.target().is_none());
        assert!(a.index().is_none());
    }

    #[test]
    fn internal_is_a_navigation() {
        let a = BreadcrumbAction::Internal {
            index: 1,
            uri: "/x".into(),
        };
        assert!(a.navigated());
        assert!(!a.was_blocked());
        assert_eq!(a.target(), Some("/x"));
        assert_eq!(a.index(), Some(1));
    }

    #[test]
    fn vetoed_and_error_are_blocks() {
        let v = BreadcrumbAction::Vetoed {
            index: 2,
            uri: "/y".into(),
        };
        let e = BreadcrumbAction::Error {
            index: 3,
            uri: "/z".into(),
            message: "bad".into(),
        };
        assert!(v.was_blocked() && !v.navigated());
        assert!(e.was_blocked() && !e.navigated());
        assert_eq!(v.index(), Some(2));
        assert_eq!(e.index(), Some(3));
    }

    #[test]
    fn external_reports_url() {
        let a = BreadcrumbAction::External {
            index: 0,
            url: "https://docs.rs".into(),
        };
        assert!(a.is_external() && a.navigated());
        assert_eq!(a.target(), Some("https://docs.rs"));
        assert_eq!(a.index(), Some(0));
    }
}