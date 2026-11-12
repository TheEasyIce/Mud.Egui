//! `DrawerVariant` — how a [`MudDrawer`] behaves at the current width.
//!
//! Port of `MudBlazor.DrawerVariant` (MudBlazor 8.13.0). Discriminants match
//! the C# enum exactly.
//!
//! Kept in its own file per Rule 2.

use std::fmt;

/// How a drawer behaves at the current width.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DrawerVariant {
    /// Overlays the content. Typically paired with an overlay backdrop.
    Temporary = 0,
    /// `Persistent` above the breakpoint, `Temporary` below it. The default.
    #[default]
    Responsive = 1,
    /// Pushes the content aside instead of overlaying it.
    Persistent = 2,
    /// A responsive drawer that shrinks to `mini_width` instead of closing.
    Mini = 3,
}

impl DrawerVariant {
    /// Every variant, in discriminant order.
    pub const ALL: [Self; 4] = [
        Self::Temporary,
        Self::Responsive,
        Self::Persistent,
        Self::Mini,
    ];

    /// Lower-case name, matching the variant.
    pub fn name(self) -> &'static str {
        match self {
            Self::Temporary => "temporary",
            Self::Responsive => "responsive",
            Self::Persistent => "persistent",
            Self::Mini => "mini",
        }
    }

    /// `true` for `Temporary` — always overlays.
    #[inline]
    pub fn is_always_overlay(self) -> bool {
        matches!(self, Self::Temporary)
    }

    /// `true` for `Persistent` — always pushes.
    #[inline]
    pub fn is_always_push(self) -> bool {
        matches!(self, Self::Persistent)
    }
}

impl fmt::Display for DrawerVariant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_responsive() {
        assert_eq!(DrawerVariant::default(), DrawerVariant::Responsive);
    }

    #[test]
    fn discriminants_match_mudblazor() {
        assert_eq!(DrawerVariant::Temporary as i32, 0);
        assert_eq!(DrawerVariant::Responsive as i32, 1);
        assert_eq!(DrawerVariant::Persistent as i32, 2);
        assert_eq!(DrawerVariant::Mini as i32, 3);
    }

    #[test]
    fn all_has_four_unique_names() {
        assert_eq!(DrawerVariant::ALL.len(), 4);
        let mut names: Vec<&str> = DrawerVariant::ALL.iter().map(|v| v.name()).collect();
        names.sort_unstable();
        let mut deduped = names.clone();
        deduped.dedup();
        assert_eq!(names, deduped);
    }

    #[test]
    fn predicates_are_exclusive() {
        assert!(DrawerVariant::Temporary.is_always_overlay());
        assert!(!DrawerVariant::Persistent.is_always_overlay());
        assert!(DrawerVariant::Persistent.is_always_push());
        assert!(!DrawerVariant::Temporary.is_always_push());
    }

    #[test]
    fn display_matches_name() {
        for v in DrawerVariant::ALL {
            assert_eq!(v.to_string(), v.name());
        }
    }
}