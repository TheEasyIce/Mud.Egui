//! `DrawerClipMode` — how a drawer clips against the app bar.
//!
//! Port of `MudBlazor.DrawerClipMode` (MudBlazor 8.13.0). Discriminants match
//! the C# enum exactly.
//!
//! Kept in its own file per Rule 2.

use std::fmt;

/// How a drawer clips against the app bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DrawerClipMode {
    /// The drawer runs full height, above the app bar.
    #[default]
    Never = 0,
    /// The drawer starts below the app bar and pushes content aside.
    Docked = 1,
    /// The drawer starts below the app bar and overlays content.
    Always = 2,
}

impl DrawerClipMode {
    /// Every variant, in discriminant order.
    pub const ALL: [Self; 3] = [Self::Never, Self::Docked, Self::Always];

    /// Lower-case name, matching the variant.
    pub fn name(self) -> &'static str {
        match self {
            Self::Never => "never",
            Self::Docked => "docked",
            Self::Always => "always",
        }
    }

    /// `true` when the drawer starts at the very top of the layout (above the
    /// app bar). Only `Never`.
    #[inline]
    pub fn starts_above_app_bar(self) -> bool {
        matches!(self, Self::Never)
    }
}

impl fmt::Display for DrawerClipMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_never() {
        assert_eq!(DrawerClipMode::default(), DrawerClipMode::Never);
    }

    #[test]
    fn discriminants_match_mudblazor() {
        assert_eq!(DrawerClipMode::Never as i32, 0);
        assert_eq!(DrawerClipMode::Docked as i32, 1);
        assert_eq!(DrawerClipMode::Always as i32, 2);
    }

    #[test]
    fn only_never_starts_above_the_app_bar() {
        assert!(DrawerClipMode::Never.starts_above_app_bar());
        assert!(!DrawerClipMode::Docked.starts_above_app_bar());
        assert!(!DrawerClipMode::Always.starts_above_app_bar());
    }

    #[test]
    fn all_has_three_unique_names() {
        let mut names: Vec<&str> = DrawerClipMode::ALL.iter().map(|v| v.name()).collect();
        names.sort_unstable();
        let mut deduped = names.clone();
        deduped.dedup();
        assert_eq!(names, deduped);
    }

    #[test]
    fn display_matches_name() {
        for v in DrawerClipMode::ALL {
            assert_eq!(v.to_string(), v.name());
        }
    }
}