//! Horizontal alignment of content inside a component.
//!
//! Port of `MudBlazor.HorizontalAlignment` (MudBlazor 8.13.0). `Left`, `Center`
//! and `Right` are physical edges; `Start` and `End` are logical and flip under
//! right-to-left layout. This is why `ColumnAlign` (a table-only, physical
//! three-way enum) is not reused here.

/// Where horizontally to place a component's content.
///
/// Mirrors `MudBlazor.HorizontalAlignment`, including the logical `Start` /
/// `End` variants that [`ColumnAlign`](super::ColumnAlign) does not have.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HorizontalAlignment {
    /// Pin to the left edge, regardless of text direction.
    Left,
    /// Centre.
    Center,
    /// Pin to the right edge, regardless of text direction.
    Right,
    /// Pin to the reading-order start (left in LTR, right in RTL).
    Start,
    /// Pin to the reading-order end (right in LTR, left in RTL).
    End,
}

impl Default for HorizontalAlignment {
    /// [`HorizontalAlignment::Left`], matching MudBlazor's default.
    fn default() -> Self {
        Self::Left
    }
}

impl HorizontalAlignment {
    /// Every variant, in MudBlazor's declaration order.
    pub const ALL: [HorizontalAlignment; 5] = [
        HorizontalAlignment::Left,
        HorizontalAlignment::Center,
        HorizontalAlignment::Right,
        HorizontalAlignment::Start,
        HorizontalAlignment::End,
    ];

    /// Lower-case name, useful for logging and test ids.
    #[inline]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Left => "left",
            Self::Center => "center",
            Self::Right => "right",
            Self::Start => "start",
            Self::End => "end",
        }
    }

    /// Whether this variant depends on the text direction.
    #[inline]
    pub const fn is_logical(self) -> bool {
        matches!(self, Self::Start | Self::End)
    }
}

impl core::fmt::Display for HorizontalAlignment {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_left() {
        assert_eq!(HorizontalAlignment::default(), HorizontalAlignment::Left);
    }

    #[test]
    fn all_has_five_unique_variants() {
        assert_eq!(HorizontalAlignment::ALL.len(), 5);
        for (i, a) in HorizontalAlignment::ALL.iter().enumerate() {
            for b in &HorizontalAlignment::ALL[i + 1..] {
                assert_ne!(a, b);
            }
        }
    }

    #[test]
    fn only_start_and_end_are_logical() {
        assert!(HorizontalAlignment::Start.is_logical());
        assert!(HorizontalAlignment::End.is_logical());
        assert!(!HorizontalAlignment::Left.is_logical());
        assert!(!HorizontalAlignment::Center.is_logical());
        assert!(!HorizontalAlignment::Right.is_logical());
    }

    #[test]
    fn display_matches_name() {
        for a in HorizontalAlignment::ALL {
            assert_eq!(a.to_string(), a.name());
        }
    }
}