//! `TimelineOrientation` — axis along which a timeline is laid out.
//!
//! Port of `MudBlazor.TimelineOrientation` (MudBlazor 8.13.0).

/// Specifies the orientation of items in a
/// [`MudTimeline`](crate::MudTimeline).
///
/// Mirrors MudBlazor's `TimelineOrientation` enum exactly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TimelineOrientation {
    /// Items are displayed vertically (top to bottom). This is the default.
    #[default]
    Vertical,
    /// Items are displayed horizontally (left to right).
    Horizontal,
}

impl TimelineOrientation {
    /// Whether the timeline runs along the vertical axis.
    #[inline]
    pub fn is_vertical(self) -> bool {
        matches!(self, Self::Vertical)
    }

    /// Whether the timeline runs along the horizontal axis.
    #[inline]
    pub fn is_horizontal(self) -> bool {
        matches!(self, Self::Horizontal)
    }

    /// The other orientation.
    #[inline]
    pub fn flipped(self) -> Self {
        match self {
            Self::Vertical => Self::Horizontal,
            Self::Horizontal => Self::Vertical,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_vertical() {
        assert_eq!(TimelineOrientation::default(), TimelineOrientation::Vertical);
    }

    #[test]
    fn predicates_and_flip() {
        assert!(TimelineOrientation::Vertical.is_vertical());
        assert!(!TimelineOrientation::Vertical.is_horizontal());
        assert!(TimelineOrientation::Horizontal.is_horizontal());
        assert_eq!(
            TimelineOrientation::Vertical.flipped(),
            TimelineOrientation::Horizontal
        );
        assert_eq!(
            TimelineOrientation::Horizontal.flipped(),
            TimelineOrientation::Vertical
        );
    }
}