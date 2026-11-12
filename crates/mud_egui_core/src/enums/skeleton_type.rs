//! The shape of a skeleton placeholder.
//!
//! Port of `MudBlazor.SkeletonType` (MudBlazor 8.13.0). The 8.13.0 enum has
//! three variants: `Text = 0`, `Circle = 1`, `Rectangle = 2`. The numeric
//! values are preserved in the doc comments for reference.
//!
//! | MudBlazor | Rust | Meaning |
//! |-----------|------|---------|
//! | `Text = 0` | [`Text`](Self::Text) | A placeholder for a text run. |
//! | `Circle = 1` | [`Circle`](Self::Circle) | A circular placeholder (avatar, badge). |
//! | `Rectangle = 2` | [`Rectangle`](Self::Rectangle) | A rectangular placeholder (image, card). |
//!
//! Used by `MudSkeleton`.

/// The shape a skeleton placeholder draws.
///
/// Mirrors `MudBlazor.SkeletonType`. `Text` is the default, matching
/// MudBlazor's `SkeletonType.Text`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SkeletonType {
    /// A placeholder for a text run. Rounded corners, a single line-height
    /// tall, fills the available width by default. Equivalent to MudBlazor's
    /// `SkeletonType.Text = 0`. **Default.**
    #[default]
    Text,
    /// A circular placeholder (avatar, badge). Width and height are forced to
    /// the same value; the corner radius is half that diameter. Equivalent to
    /// MudBlazor's `SkeletonType.Circle = 1`.
    Circle,
    /// A rectangular placeholder (image, card). Square corners by default.
    /// Equivalent to MudBlazor's `SkeletonType.Rectangle = 2`.
    Rectangle,
}

impl SkeletonType {
    /// Every variant, in MudBlazor's declaration order.
    pub const ALL: [SkeletonType; 3] =
        [SkeletonType::Text, SkeletonType::Circle, SkeletonType::Rectangle];

    /// Lower-case name, useful for logging and test ids.
    #[inline]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Circle => "circle",
            Self::Rectangle => "rectangle",
        }
    }
}

impl core::fmt::Display for SkeletonType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_text() {
        assert_eq!(SkeletonType::default(), SkeletonType::Text);
    }

    #[test]
    fn all_has_three_unique_variants() {
        assert_eq!(SkeletonType::ALL.len(), 3);
        for (i, a) in SkeletonType::ALL.iter().enumerate() {
            for b in &SkeletonType::ALL[i + 1..] {
                assert_ne!(a, b);
            }
        }
    }

    #[test]
    fn names_are_lowercase_and_unique() {
        let mut seen = std::collections::BTreeSet::new();
        for s in SkeletonType::ALL {
            assert!(seen.insert(s.name()), "duplicate name: {}", s.name());
            assert_eq!(s.name(), s.name().to_lowercase());
        }
    }

    #[test]
    fn display_matches_name() {
        for s in SkeletonType::ALL {
            assert_eq!(s.to_string(), s.name());
        }
    }
}