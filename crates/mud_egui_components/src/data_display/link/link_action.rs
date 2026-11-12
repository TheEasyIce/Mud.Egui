//! The outcome of one [`MudLink`](super::MudLink) frame.
//!
//! Kept in its own file per the project convention that every enum lives in
//! its own file. Public because it is the return type of
//! [`MudLink::show`](super::MudLink::show).

/// What a [`MudLink`](super::MudLink) did when it was clicked.
///
/// The value is [`Self::None`] on every frame the link was not clicked, so
/// callers can safely match on it without wrapping in `Option`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinkAction {
    /// Nothing happened: either the link was not clicked this frame, it has no
    /// `href`, it is disabled, or `on_click` already handled the event and the
    /// caller used [`MudLink::prevent_navigation`](super::MudLink::prevent_navigation).
    None,

    /// The link navigated **internally** through the `NavigationManager`.
    /// `uri` is the URI that was requested (as passed to `navigate_to`).
    Internal {
        /// The URI the link asked the router to navigate to.
        uri: String,
    },

    /// The link asked the `NavigationManager` to navigate internally, but a
    /// registered veto handler prevented it.
    Vetoed {
        /// The URI that was vetoed.
        uri: String,
    },

    /// The link asked the `NavigationManager` to navigate internally, but
    /// navigation failed (malformed URI, outside base URI, re-entrant call).
    Error {
        /// The URI the link asked the router to navigate to.
        uri: String,
        /// Human-readable reason, produced by `NavigateError::Display`.
        message: String,
    },

    /// The link opened an **external** URL in the system browser.
    External {
        /// The absolute URL that was opened.
        url: String,
        /// Whether the URL was opened in a new tab (`Target="_blank"`).
        new_tab: bool,
    },
}

impl LinkAction {
    /// Whether the link performed any navigation (internal or external).
    #[inline]
    pub fn navigated(&self) -> bool {
        matches!(self, Self::Internal { .. } | Self::External { .. })
    }

    /// Whether the link was blocked (vetoed or errored).
    #[inline]
    pub fn was_blocked(&self) -> bool {
        matches!(self, Self::Vetoed { .. } | Self::Error { .. })
    }

    /// The target URI / URL for internal, vetoed, error, and external actions.
    #[inline]
    pub fn target(&self) -> Option<&str> {
        match self {
            Self::None => None,
            Self::Internal { uri } | Self::Vetoed { uri } | Self::Error { uri, .. } => {
                Some(uri.as_str())
            }
            Self::External { url, .. } => Some(url.as_str()),
        }
    }

    /// Whether the action opened the system browser.
    #[inline]
    pub fn is_external(&self) -> bool {
        matches!(self, Self::External { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn none_is_inert() {
        let a = LinkAction::None;
        assert!(!a.navigated());
        assert!(!a.was_blocked());
        assert!(!a.is_external());
        assert!(a.target().is_none());
    }

    #[test]
    fn internal_is_a_navigation() {
        let a = LinkAction::Internal { uri: "/x".into() };
        assert!(a.navigated());
        assert!(!a.was_blocked());
        assert_eq!(a.target(), Some("/x"));
    }

    #[test]
    fn vetoed_and_error_are_blocks() {
        let v = LinkAction::Vetoed { uri: "/x".into() };
        let e = LinkAction::Error {
            uri: "/x".into(),
            message: "boom".into(),
        };
        assert!(!v.navigated() && v.was_blocked());
        assert!(!e.navigated() && e.was_blocked());
    }

    #[test]
    fn external_reports_url_and_new_tab() {
        let a = LinkAction::External {
            url: "https://example.com".into(),
            new_tab: true,
        };
        assert!(a.navigated() && a.is_external());
        assert_eq!(a.target(), Some("https://example.com"));
        match a {
            LinkAction::External { new_tab, .. } => assert!(new_tab),
            _ => unreachable!(),
        }
    }
}