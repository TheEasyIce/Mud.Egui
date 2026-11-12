//! `TimelinePosition` — how items are placed relative to the timeline axis.
//!
//! Port of `MudBlazor.TimelinePosition` (MudBlazor 8.13.0).

/// Specifies how items are drawn in a [`MudTimeline`](crate::MudTimeline).
///
/// Mirrors MudBlazor's `TimelinePosition` enum exactly. The valid combinations
/// depend on the orientation:
///
/// * Vertical → `Left`, `Right`, `Alternate`, `Start`, `End`
/// * Horizontal → `Top`, `Bottom`, `Alternate`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TimelinePosition {
    /// Items alternate on either side of centered dots.
    #[default]
    Alternate,
    /// Dots are displayed above the text of each timeline item.
    Top,
    /// Dots are displayed below the text of each timeline item.
    Bottom,
    /// Dots are displayed to the left of text for each timeline item.
    Left,
    /// Dots are displayed to the right of text for each timeline item.
    Right,
    /// Dots are displayed at the start based on the right-to-left setting.
    Start,
    /// Dots are displayed at the end based on the right-to-left setting.
    End,
}

impl TimelinePosition {
    /// Whether this is the alternating layout.
    #[inline]
    pub fn is_alternate(self) -> bool {
        matches!(self, Self::Alternate)
    }

    /// Whether this position is valid for a vertical timeline.
    #[inline]
    pub fn is_vertical_axis(self) -> bool {
        matches!(self, Self::Left | Self::Right | Self::Start | Self::End)
    }

    /// Whether this position is valid for a horizontal timeline.
    #[inline]
    pub fn is_horizontal_axis(self) -> bool {
        matches!(self, Self::Top | Self::Bottom)
    }

    /// Resolves `Start` / `End` into the physical `Left` / `Right` for
    /// vertical timelines (or `Top` / `Bottom` for horizontal timelines).
    ///
    /// Physical variants (`Alternate`, `Left`, `Right`, `Top`, `Bottom`)
    /// pass through unchanged.
    #[inline]
    pub fn resolve(self, right_to_left: bool) -> Self {
        match (self, right_to_left) {
            (Self::Start, false) | (Self::End, true) => Self::Left,
            (Self::Start, true) | (Self::End, false) => Self::Right,
            (other, _) => other,
        }
    }

    /// Human-readable name, matching the C# enum member.
    #[inline]
    pub fn name(self) -> &'static str {
        match self {
            Self::Alternate => "Alternate",
            Self::Top => "Top",
            Self::Bottom => "Bottom",
            Self::Left => "Left",
            Self::Right => "Right",
            Self::Start => "Start",
            Self::End => "End",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_alternate() {
        assert_eq!(TimelinePosition::default(), TimelinePosition::Alternate);
    }

    #[test]
    fn is_alternate_only_for_alternate() {
        assert!(TimelinePosition::Alternate.is_alternate());
        for p in [
            TimelinePosition::Top,
            TimelinePosition::Bottom,
            TimelinePosition::Left,
            TimelinePosition::Right,
            TimelinePosition::Start,
            TimelinePosition::End,
        ] {
            assert!(!p.is_alternate());
        }
    }

    #[test]
    fn axis_predicates_are_disjoint() {
        for p in [TimelinePosition::Left, TimelinePosition::Right, TimelinePosition::Start, TimelinePosition::End] {
            assert!(p.is_vertical_axis());
            assert!(!p.is_horizontal_axis());
        }
        for p in [TimelinePosition::Top, TimelinePosition::Bottom] {
            assert!(p.is_horizontal_axis());
            assert!(!p.is_vertical_axis());
        }
        assert!(!TimelinePosition::Alternate.is_vertical_axis());
        assert!(!TimelinePosition::Alternate.is_horizontal_axis());
    }

    #[test]
    fn resolve_start_and_end_by_rtl() {
        assert_eq!(TimelinePosition::Start.resolve(false), TimelinePosition::Left);
        assert_eq!(TimelinePosition::Start.resolve(true), TimelinePosition::Right);
        assert_eq!(TimelinePosition::End.resolve(false), TimelinePosition::Right);
        assert_eq!(TimelinePosition::End.resolve(true), TimelinePosition::Left);
    }

    #[test]
    fn resolve_physical_variants_are_identity() {
        for p in [
            TimelinePosition::Alternate,
            TimelinePosition::Top,
            TimelinePosition::Bottom,
            TimelinePosition::Left,
            TimelinePosition::Right,
        ] {
            assert_eq!(p.resolve(false), p);
            assert_eq!(p.resolve(true), p);
        }
    }

    #[test]
    fn name_matches_variant() {
        assert_eq!(TimelinePosition::Alternate.name(), "Alternate");
        assert_eq!(TimelinePosition::End.name(), "End");
    }
}