//! `TransformOrigin` — which corner of the popover attaches to the anchor.
//!
//! Port of `MudBlazor.Origin` (MudBlazor 8.13.0).

/// Which corner of the popover is placed at the anchor attachment point.
///
/// Combined with [`AnchorOrigin`](super::AnchorOrigin), this fully defines
/// where the popover appears. Common pairs:
///
/// | Behaviour | `AnchorOrigin` | `TransformOrigin` |
/// |-----------|----------------|-------------------|
/// | Dropdown below, left-aligned | `Bottom` | `TopLeft` |
/// | Dropdown below, centered | `Bottom` | `TopCenter` |
/// | Dropdown below, right-aligned | `Bottom` | `TopRight` |
/// | Tooltip above, centered | `Top` | `BottomCenter` |
/// | Submenu to the right | `Right` | `CenterLeft` |
/// | Submenu to the left | `Left` | `CenterRight` |
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TransformOrigin {
    /// Top-left corner of the popover.
    #[default]
    TopLeft,
    /// Top-center of the popover.
    TopCenter,
    /// Top-right corner of the popover.
    TopRight,
    /// Left-center of the popover.
    CenterLeft,
    /// Center of the popover.
    Center,
    /// Right-center of the popover.
    CenterRight,
    /// Bottom-left corner of the popover.
    BottomLeft,
    /// Bottom-center of the popover.
    BottomCenter,
    /// Bottom-right corner of the popover.
    BottomRight,
}

impl TransformOrigin {
    /// Flips the vertical half (Top ↔ Bottom). Center variants return `None`.
    #[inline]
    pub fn flip_vertical(self) -> Option<Self> {
        match self {
            Self::TopLeft => Some(Self::BottomLeft),
            Self::TopCenter => Some(Self::BottomCenter),
            Self::TopRight => Some(Self::BottomRight),
            Self::BottomLeft => Some(Self::TopLeft),
            Self::BottomCenter => Some(Self::TopCenter),
            Self::BottomRight => Some(Self::TopRight),
            _ => None,
        }
    }

    /// Flips the horizontal half (Left ↔ Right). Center variants return `None`.
    #[inline]
    pub fn flip_horizontal(self) -> Option<Self> {
        match self {
            Self::TopLeft => Some(Self::TopRight),
            Self::CenterLeft => Some(Self::CenterRight),
            Self::BottomLeft => Some(Self::BottomRight),
            Self::TopRight => Some(Self::TopLeft),
            Self::CenterRight => Some(Self::CenterLeft),
            Self::BottomRight => Some(Self::BottomLeft),
            _ => None,
        }
    }

    /// Human-readable name matching the C# enum member.
    #[inline]
    pub fn name(self) -> &'static str {
        match self {
            Self::TopLeft => "TopLeft",
            Self::TopCenter => "TopCenter",
            Self::TopRight => "TopRight",
            Self::CenterLeft => "CenterLeft",
            Self::Center => "Center",
            Self::CenterRight => "CenterRight",
            Self::BottomLeft => "BottomLeft",
            Self::BottomCenter => "BottomCenter",
            Self::BottomRight => "BottomRight",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_top_left() {
        assert_eq!(TransformOrigin::default(), TransformOrigin::TopLeft);
    }

    #[test]
    fn flip_vertical_swaps_top_and_bottom() {
        assert_eq!(TransformOrigin::TopLeft.flip_vertical(), Some(TransformOrigin::BottomLeft));
        assert_eq!(TransformOrigin::TopCenter.flip_vertical(), Some(TransformOrigin::BottomCenter));
        assert_eq!(TransformOrigin::BottomRight.flip_vertical(), Some(TransformOrigin::TopRight));
        assert_eq!(TransformOrigin::CenterLeft.flip_vertical(), None);
        assert_eq!(TransformOrigin::Center.flip_vertical(), None);
    }

    #[test]
    fn flip_horizontal_swaps_left_and_right() {
        assert_eq!(TransformOrigin::TopLeft.flip_horizontal(), Some(TransformOrigin::TopRight));
        assert_eq!(TransformOrigin::CenterLeft.flip_horizontal(), Some(TransformOrigin::CenterRight));
        assert_eq!(TransformOrigin::BottomRight.flip_horizontal(), Some(TransformOrigin::BottomLeft));
        assert_eq!(TransformOrigin::TopCenter.flip_horizontal(), None);
        assert_eq!(TransformOrigin::Center.flip_horizontal(), None);
    }

    #[test]
    fn names_match_variants() {
        assert_eq!(TransformOrigin::TopLeft.name(), "TopLeft");
        assert_eq!(TransformOrigin::Center.name(), "Center");
        assert_eq!(TransformOrigin::BottomRight.name(), "BottomRight");
    }
}