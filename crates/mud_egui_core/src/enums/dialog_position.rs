//! `DialogPosition` — where a dialog sits on the screen.
//!
//! Port of `MudBlazor.DialogPosition` (MudBlazor 8.13.0). Discriminants match
//! the C# enum exactly.
//!
//! Kept in its own file per Rule 2.

use std::fmt;

/// The location of a dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DialogPosition {
    /// Center of the screen. The default.
    #[default]
    Center = 0,
    /// Center-left (center-right in RTL).
    CenterLeft = 1,
    /// Center-right (center-left in RTL).
    CenterRight = 2,
    /// Top of the screen, centered horizontally.
    TopCenter = 3,
    /// Top-left corner.
    TopLeft = 4,
    /// Top-right corner.
    TopRight = 5,
    /// Bottom of the screen, centered horizontally.
    BottomCenter = 6,
    /// Bottom-left corner.
    BottomLeft = 7,
    /// Bottom-right corner.
    BottomRight = 8,
    /// Custom placement — the port positions at the top-left, matching
    /// MudBlazor's `.mud-dialog-custom` behaviour for a class with no rules.
    Custom = 9,
}

impl DialogPosition {
    /// Every variant, in discriminant order.
    pub const ALL: [Self; 10] = [
        Self::Center,
        Self::CenterLeft,
        Self::CenterRight,
        Self::TopCenter,
        Self::TopLeft,
        Self::TopRight,
        Self::BottomCenter,
        Self::BottomLeft,
        Self::BottomRight,
        Self::Custom,
    ];

    /// Lower-case, hyphen-free name, matching the variant.
    pub fn name(self) -> &'static str {
        match self {
            Self::Center => "center",
            Self::CenterLeft => "center-left",
            Self::CenterRight => "center-right",
            Self::TopCenter => "top-center",
            Self::TopLeft => "top-left",
            Self::TopRight => "top-right",
            Self::BottomCenter => "bottom-center",
            Self::BottomLeft => "bottom-left",
            Self::BottomRight => "bottom-right",
            Self::Custom => "custom",
        }
    }

    /// `(horizontal fraction, vertical fraction)` of the free space left
    /// after the dialog is placed. `Center` is `(0.5, 0.5)`, `TopLeft` is
    /// `(0.0, 0.0)`, `BottomRight` is `(1.0, 1.0)`.
    #[inline]
    pub fn fractions(self) -> (f32, f32) {
        match self {
            Self::Center => (0.5, 0.5),
            Self::CenterLeft => (0.0, 0.5),
            Self::CenterRight => (1.0, 0.5),
            Self::TopCenter => (0.5, 0.0),
            Self::TopLeft => (0.0, 0.0),
            Self::TopRight => (1.0, 0.0),
            Self::BottomCenter => (0.5, 1.0),
            Self::BottomLeft => (0.0, 1.0),
            Self::BottomRight => (1.0, 1.0),
            Self::Custom => (0.0, 0.0),
        }
    }
}

impl fmt::Display for DialogPosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_center() {
        assert_eq!(DialogPosition::default(), DialogPosition::Center);
    }

    #[test]
    fn discriminants_match_mudblazor() {
        assert_eq!(DialogPosition::Center as i32, 0);
        assert_eq!(DialogPosition::CenterLeft as i32, 1);
        assert_eq!(DialogPosition::CenterRight as i32, 2);
        assert_eq!(DialogPosition::TopCenter as i32, 3);
        assert_eq!(DialogPosition::TopLeft as i32, 4);
        assert_eq!(DialogPosition::TopRight as i32, 5);
        assert_eq!(DialogPosition::BottomCenter as i32, 6);
        assert_eq!(DialogPosition::BottomLeft as i32, 7);
        assert_eq!(DialogPosition::BottomRight as i32, 8);
        assert_eq!(DialogPosition::Custom as i32, 9);
    }

    #[test]
    fn fractions_place_every_corner() {
        assert_eq!(DialogPosition::TopLeft.fractions(), (0.0, 0.0));
        assert_eq!(DialogPosition::TopCenter.fractions(), (0.5, 0.0));
        assert_eq!(DialogPosition::TopRight.fractions(), (1.0, 0.0));
        assert_eq!(DialogPosition::CenterLeft.fractions(), (0.0, 0.5));
        assert_eq!(DialogPosition::Center.fractions(), (0.5, 0.5));
        assert_eq!(DialogPosition::CenterRight.fractions(), (1.0, 0.5));
        assert_eq!(DialogPosition::BottomLeft.fractions(), (0.0, 1.0));
        assert_eq!(DialogPosition::BottomCenter.fractions(), (0.5, 1.0));
        assert_eq!(DialogPosition::BottomRight.fractions(), (1.0, 1.0));
        assert_eq!(DialogPosition::Custom.fractions(), (0.0, 0.0));
    }

    #[test]
    fn all_has_ten_unique_names() {
        assert_eq!(DialogPosition::ALL.len(), 10);
        let mut names: Vec<&str> = DialogPosition::ALL.iter().map(|v| v.name()).collect();
        names.sort_unstable();
        let mut deduped = names.clone();
        deduped.dedup();
        assert_eq!(names, deduped);
    }

    #[test]
    fn display_matches_name() {
        for v in DialogPosition::ALL {
            assert_eq!(v.to_string(), v.name());
        }
    }
}