//! `TabHeaderPosition` — where a `Header` / `TabPanelHeader` fragment is placed.
//!
//! Port of `MudBlazor.TabHeaderPosition` (MudBlazor 8.13.0). Used by
//! `MudTabs::HeaderPosition` and `MudTabs::TabPanelHeaderPosition`.

/// Where custom header content is placed relative to the tab list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TabHeaderPosition {
    /// Before the tabs.
    Before,
    /// After the tabs. This is MudBlazor's default.
    #[default]
    After,
    /// Do not render the header content.
    None,
}

impl TabHeaderPosition {
    /// Human-readable name matching the C# enum member.
    #[inline]
    pub fn name(self) -> &'static str {
        match self {
            Self::Before => "Before",
            Self::After => "After",
            Self::None => "None",
        }
    }

    /// Whether the content should be rendered at all.
    #[inline]
    pub fn is_rendered(self) -> bool {
        !matches!(self, Self::None)
    }

    /// Whether the content should be rendered before the tabs.
    #[inline]
    pub fn is_before(self) -> bool {
        matches!(self, Self::Before)
    }

    /// Whether the content should be rendered after the tabs.
    #[inline]
    pub fn is_after(self) -> bool {
        matches!(self, Self::After)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_after() {
        assert_eq!(TabHeaderPosition::default(), TabHeaderPosition::After);
    }

    #[test]
    fn predicates_match_variants() {
        assert!(TabHeaderPosition::Before.is_before());
        assert!(!TabHeaderPosition::Before.is_after());
        assert!(TabHeaderPosition::After.is_after());
        assert!(!TabHeaderPosition::After.is_before());
        assert!(!TabHeaderPosition::None.is_rendered());
        assert!(TabHeaderPosition::Before.is_rendered());
        assert!(TabHeaderPosition::After.is_rendered());
    }

    #[test]
    fn names_match_variants() {
        assert_eq!(TabHeaderPosition::Before.name(), "Before");
        assert_eq!(TabHeaderPosition::After.name(), "After");
        assert_eq!(TabHeaderPosition::None.name(), "None");
    }
}