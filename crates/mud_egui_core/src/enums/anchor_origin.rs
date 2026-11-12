//! `AnchorOrigin` — which edge of the anchor rect a popover attaches to.
//!
//! Port of `MudBlazor.Anchor` (MudBlazor 8.13.0). Renamed to
//! `AnchorOrigin` for clarity (the popover also has a `TransformOrigin`
//! which is the *other* end of the attachment).

/// Which edge of the anchor the popover attaches to.
///
/// `Start` / `End` are logical variants that resolve to `Left` / `Right`
/// based on the active layout direction. `Top` / `Bottom` / `Left` / `Right`
/// are physical and do not change under RTL.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AnchorOrigin {
    /// Attach to the anchor's top edge (popover above).
    Top,
    /// Attach to the anchor's bottom edge (popover below). MudBlazor's
    /// default.
    #[default]
    Bottom,
    /// Attach to the anchor's left edge (popover to the left).
    Left,
    /// Attach to the anchor's right edge (popover to the right).
    Right,
    /// Logical start of the horizontal axis. Resolves to `Left` in LTR,
    /// `Right` in RTL.
    Start,
    /// Logical end of the horizontal axis. Resolves to `Right` in LTR,
    /// `Left` in RTL.
    End,
}

impl AnchorOrigin {
    /// Resolves `Start` / `End` to `Left` / `Right` under the active layout
    /// direction. Physical variants pass through unchanged.
    #[inline]
    pub fn resolve(self, right_to_left: bool) -> Self {
        match self {
            Self::Start => {
                if right_to_left { Self::Right } else { Self::Left }
            }
            Self::End => {
                if right_to_left { Self::Left } else { Self::Right }
            }
            other => other,
        }
    }

    /// The opposite edge on the vertical axis (Top ↔ Bottom). `None` for
    /// horizontal anchors.
    #[inline]
    pub fn flip_vertical(self) -> Option<Self> {
        match self {
            Self::Top => Some(Self::Bottom),
            Self::Bottom => Some(Self::Top),
            _ => None,
        }
    }

    /// The opposite edge on the horizontal axis (Left ↔ Right). `Start` and
    /// `End` also swap.
    #[inline]
    pub fn flip_horizontal(self) -> Option<Self> {
        match self {
            Self::Left => Some(Self::Right),
            Self::Right => Some(Self::Left),
            Self::Start => Some(Self::End),
            Self::End => Some(Self::Start),
            _ => None,
        }
    }

    /// Whether the popover extends vertically from this anchor edge.
    #[inline]
    pub fn is_vertical(self) -> bool {
        matches!(self, Self::Top | Self::Bottom)
    }

    /// Whether the popover extends horizontally from this anchor edge.
    #[inline]
    pub fn is_horizontal(self) -> bool {
        matches!(self, Self::Left | Self::Right | Self::Start | Self::End)
    }

    /// Human-readable name matching the C# enum member.
    #[inline]
    pub fn name(self) -> &'static str {
        match self {
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
    fn default_is_bottom() {
        assert_eq!(AnchorOrigin::default(), AnchorOrigin::Bottom);
    }

    #[test]
    fn resolve_start_end_swaps_under_rtl() {
        assert_eq!(AnchorOrigin::Start.resolve(false), AnchorOrigin::Left);
        assert_eq!(AnchorOrigin::Start.resolve(true), AnchorOrigin::Right);
        assert_eq!(AnchorOrigin::End.resolve(false), AnchorOrigin::Right);
        assert_eq!(AnchorOrigin::End.resolve(true), AnchorOrigin::Left);
    }

    #[test]
    fn resolve_physical_variants_are_identity() {
        for anchor in [
            AnchorOrigin::Top,
            AnchorOrigin::Bottom,
            AnchorOrigin::Left,
            AnchorOrigin::Right,
        ] {
            assert_eq!(anchor.resolve(false), anchor);
            assert_eq!(anchor.resolve(true), anchor);
        }
    }

    #[test]
    fn flip_vertical_only_swaps_top_and_bottom() {
        assert_eq!(AnchorOrigin::Top.flip_vertical(), Some(AnchorOrigin::Bottom));
        assert_eq!(AnchorOrigin::Bottom.flip_vertical(), Some(AnchorOrigin::Top));
        for anchor in [
            AnchorOrigin::Left,
            AnchorOrigin::Right,
            AnchorOrigin::Start,
            AnchorOrigin::End,
        ] {
            assert_eq!(anchor.flip_vertical(), None);
        }
    }

    #[test]
    fn flip_horizontal_only_swaps_left_right_start_end() {
        assert_eq!(AnchorOrigin::Left.flip_horizontal(), Some(AnchorOrigin::Right));
        assert_eq!(AnchorOrigin::Right.flip_horizontal(), Some(AnchorOrigin::Left));
        assert_eq!(AnchorOrigin::Start.flip_horizontal(), Some(AnchorOrigin::End));
        assert_eq!(AnchorOrigin::End.flip_horizontal(), Some(AnchorOrigin::Start));
        for anchor in [AnchorOrigin::Top, AnchorOrigin::Bottom] {
            assert_eq!(anchor.flip_horizontal(), None);
        }
    }

    #[test]
    fn is_vertical_and_is_horizontal_are_disjoint() {
        assert!(AnchorOrigin::Top.is_vertical());
        assert!(AnchorOrigin::Bottom.is_vertical());
        assert!(AnchorOrigin::Left.is_horizontal());
        assert!(AnchorOrigin::Right.is_horizontal());
        assert!(AnchorOrigin::Start.is_horizontal());
        assert!(AnchorOrigin::End.is_horizontal());
        for anchor in [
            AnchorOrigin::Top,
            AnchorOrigin::Bottom,
            AnchorOrigin::Left,
            AnchorOrigin::Right,
            AnchorOrigin::Start,
            AnchorOrigin::End,
        ] {
            assert_ne!(anchor.is_vertical(), anchor.is_horizontal());
        }
    }

    #[test]
    fn names_match_variants() {
        assert_eq!(AnchorOrigin::Top.name(), "Top");
        assert_eq!(AnchorOrigin::Start.name(), "Start");
    }
}