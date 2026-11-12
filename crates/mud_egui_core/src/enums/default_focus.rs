//! `DefaultFocus` — which element receives focus when a dialog opens.
//!
//! Port of `MudBlazor.DefaultFocus` (MudBlazor 8.13.0). Discriminants match
//! the C# enum exactly.
//!
//! Kept in its own file per Rule 2.

use std::fmt;

/// The element that receives focus when a dialog opens.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultFocus {
    /// No focus will occur.
    None = 0,
    /// The dialog container itself. The default.
    #[default]
    Element = 1,
    /// The first focusable child inside the dialog.
    FirstChild = 2,
    /// The last focusable child inside the dialog.
    LastChild = 3,
}

impl DefaultFocus {
    /// Every variant, in discriminant order.
    pub const ALL: [Self; 4] = [Self::None, Self::Element, Self::FirstChild, Self::LastChild];

    /// Lower-case name, matching the variant.
    pub fn name(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Element => "element",
            Self::FirstChild => "first-child",
            Self::LastChild => "last-child",
        }
    }

    /// `true` when the dialog should try to claim focus on open.
    #[inline]
    pub fn requests_focus(self) -> bool {
        !matches!(self, Self::None)
    }
}

impl fmt::Display for DefaultFocus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_element() {
        assert_eq!(DefaultFocus::default(), DefaultFocus::Element);
    }

    #[test]
    fn discriminants_match_mudblazor() {
        assert_eq!(DefaultFocus::None as i32, 0);
        assert_eq!(DefaultFocus::Element as i32, 1);
        assert_eq!(DefaultFocus::FirstChild as i32, 2);
        assert_eq!(DefaultFocus::LastChild as i32, 3);
    }

    #[test]
    fn only_none_skips_focus() {
        assert!(!DefaultFocus::None.requests_focus());
        assert!(DefaultFocus::Element.requests_focus());
        assert!(DefaultFocus::FirstChild.requests_focus());
        assert!(DefaultFocus::LastChild.requests_focus());
    }

    #[test]
    fn all_has_four_unique_names() {
        assert_eq!(DefaultFocus::ALL.len(), 4);
        let mut names: Vec<&str> = DefaultFocus::ALL.iter().map(|v| v.name()).collect();
        names.sort_unstable();
        let mut deduped = names.clone();
        deduped.dedup();
        assert_eq!(names, deduped);
    }

    #[test]
    fn display_matches_name() {
        for v in DefaultFocus::ALL {
            assert_eq!(v.to_string(), v.name());
        }
    }
}