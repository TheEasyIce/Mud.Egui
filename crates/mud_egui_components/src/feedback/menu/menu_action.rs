//! What a [`MudMenu`](super::MudMenu) did this frame.
//!
//! **Extension**: MudBlazor fires `OnClick` per-item; the port also returns
//! an action so callers can log or react without wiring per-item callbacks.

/// The outcome of a [`MudMenu::show`](super::MudMenu::show) frame.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum MenuAction {
    /// No interaction this frame.
    #[default]
    None,
    /// The menu was opened this frame.
    Opened,
    /// The menu was closed this frame.
    Closed,
    /// An item was activated.
    ItemActivated {
        /// The item's `text`, if any. `None` for items that only show an icon.
        label: Option<String>,
        /// The item's `href`, if any.
        href: Option<String>,
    },
}

impl MenuAction {
    /// Whether the menu opened this frame.
    #[inline]
    pub fn opened(&self) -> bool {
        matches!(self, Self::Opened)
    }

    /// Whether the menu closed this frame.
    #[inline]
    pub fn closed(&self) -> bool {
        matches!(self, Self::Closed)
    }

    /// Whether an item was activated this frame.
    #[inline]
    pub fn activated(&self) -> bool {
        matches!(self, Self::ItemActivated { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_none() {
        assert_eq!(MenuAction::default(), MenuAction::None);
    }

    #[test]
    fn predicates_match_variants() {
        assert!(MenuAction::Opened.opened());
        assert!(!MenuAction::Opened.closed());
        assert!(MenuAction::Closed.closed());
        assert!(MenuAction::ItemActivated { label: None, href: None }.activated());
        assert!(!MenuAction::None.opened());
    }
}