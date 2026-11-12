//! `TimelineAlign` — alignment of an item's dot relative to its text.
//!
//! Port of `MudBlazor.TimelineAlign` (MudBlazor 8.13.0).

/// Specifies the alignment of each item's dot relative to its text in a
/// [`MudTimeline`](crate::MudTimeline).
///
/// Mirrors MudBlazor's `TimelineAlign` enum exactly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TimelineAlign {
    /// The dot is centered relative to its text.
    #[default]
    Default,
    /// The dot is aligned with the start of the text.
    Start,
    /// The dot is aligned with the end of the text.
    End,
}

impl TimelineAlign {
    /// Whether the dot should be vertically/horizontally centered.
    #[inline]
    pub fn is_center(self) -> bool {
        matches!(self, Self::Default)
    }

    /// Whether the dot should be aligned to the start edge.
    #[inline]
    pub fn is_start(self) -> bool {
        matches!(self, Self::Start)
    }

    /// Whether the dot should be aligned to the end edge.
    #[inline]
    pub fn is_end(self) -> bool {
        matches!(self, Self::End)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_center() {
        assert_eq!(TimelineAlign::default(), TimelineAlign::Default);
    }

    #[test]
    fn predicates_match_variants() {
        assert!(TimelineAlign::Default.is_center());
        assert!(!TimelineAlign::Default.is_start());
        assert!(!TimelineAlign::Default.is_end());

        assert!(TimelineAlign::Start.is_start());
        assert!(TimelineAlign::End.is_end());
    }
}