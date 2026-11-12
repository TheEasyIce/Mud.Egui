//! `OverflowBehavior` — what a popover does when it doesn't fit in the screen.
//!
//! Port of `MudBlazor.OverflowBehavior` (MudBlazor 8.13.0).

/// How a popover reacts when its preferred position overflows the screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum OverflowBehavior {
    /// Flip to the opposite side of the anchor if the popover doesn't fit.
    /// Falls back to clamping if flipping doesn't help. This is MudBlazor's
    /// default and the recommended choice for dropdowns.
    #[default]
    Flip,
    /// Shift the popover into the screen bounds (clamp) without flipping.
    Reposition,
    /// Shrink the popover so it fits. **Not implemented in pass 1** —
    /// treated as `Flip` with the caller responsible for setting a
    /// `max_height`. Reserved for a future pass where popovers can measure
    /// their content before placing.
    Shrink,
}

impl OverflowBehavior {
    /// Human-readable name matching the C# enum member.
    #[inline]
    pub fn name(self) -> &'static str {
        match self {
            Self::Flip => "Flip",
            Self::Reposition => "Reposition",
            Self::Shrink => "Shrink",
        }
    }

    /// Whether this behavior should attempt to flip the anchor when the
    /// preferred position overflows.
    #[inline]
    pub fn tries_flip(self) -> bool {
        matches!(self, Self::Flip | Self::Shrink)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_flip() {
        assert_eq!(OverflowBehavior::default(), OverflowBehavior::Flip);
    }

    #[test]
    fn tries_flip_matches_variants() {
        assert!(OverflowBehavior::Flip.tries_flip());
        assert!(OverflowBehavior::Shrink.tries_flip());
        assert!(!OverflowBehavior::Reposition.tries_flip());
    }

    #[test]
    fn names_match_variants() {
        assert_eq!(OverflowBehavior::Flip.name(), "Flip");
        assert_eq!(OverflowBehavior::Reposition.name(), "Reposition");
        assert_eq!(OverflowBehavior::Shrink.name(), "Shrink");
    }
}