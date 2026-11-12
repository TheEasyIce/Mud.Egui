//! `Position` — the nine-way positioning anchor used by components that
//! attach content to an edge or the centre of a container.
//!
//! Port of `MudBlazor.Position` (MudBlazor 8.13.0). It is deliberately
//! **distinct** from [`Placement`](super::placement::Placement), which is the
//! four-way enum used for form-field label placement (`Start` / `End` /
//! `Top` / `Bottom`). `Position` also includes `Left`, `Right`, and
//! `Center`, and is used by:
//!
//! * [`MudCarousel`](crate::data_display::carousel::mud_carousel::MudCarousel)
//!   for `arrows_position` and `bullets_position`.
//! * Any future `MudTabs` / `MudBadge` / `MudTooltip` positioning that needs
//!   the same nine-anchor vocabulary.
//!
//! # Logical vs. physical anchors
//!
//! `Start` and `End` are **logical**: in LTR they resolve to `Left` and
//! `Right` respectively; in RTL they swap. Call [`Position::resolve`] to get
//! the physical direction, or use [`Position::is_horizontal`] /
//! [`Position::is_vertical`] / [`Position::is_center`] to reason about the
//! axis the anchor lies on.
//!
//! ```
//! # use mud_egui_core::enums::Position;
//! assert_eq!(Position::Start.resolve(false), Position::Left);
//! assert_eq!(Position::Start.resolve(true),  Position::Right);
//! assert_eq!(Position::End.resolve(false),   Position::Right);
//! assert_eq!(Position::End.resolve(true),    Position::Left);
//! assert_eq!(Position::Top.resolve(true),    Position::Top); // untouched
//!
//! assert!(Position::Left.is_horizontal());
//! assert!(Position::Start.is_horizontal());  // logical → horizontal
//! assert!(Position::Top.is_vertical());
//! assert!(Position::Center.is_center());
//! ```

use serde::{Deserialize, Serialize};

/// Nine-way positioning anchor.
///
/// Mirrors `MudBlazor.Position`. See the [module docs](self) for the
/// logical-vs-physical rules for `Start` and `End`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Position {
    /// Top edge (vertical anchor, start of the Y axis).
    Top,
    /// Bottom edge (vertical anchor, end of the Y axis).
    Bottom,
    /// Logical start of the horizontal axis.
    ///
    /// Resolves to `Left` in LTR and `Right` in RTL. See
    /// [`Position::resolve`].
    Start,
    /// Logical end of the horizontal axis.
    ///
    /// Resolves to `Right` in LTR and `Left` in RTL. See
    /// [`Position::resolve`].
    End,
    /// Physical left edge.
    Left,
    /// Physical right edge.
    Right,
    /// The centre of the container.
    Center,
}

impl Default for Position {
    /// Matches MudBlazor's default for `ArrowsPosition` and most placement
    /// parameters: the centre.
    fn default() -> Self {
        Self::Center
    }
}

impl Position {
    /// Resolves a logical `Start` / `End` into a physical `Left` / `Right`
    /// given the active layout direction.
    ///
    /// Physical anchors (`Top`, `Bottom`, `Left`, `Right`, `Center`) pass
    /// through unchanged.
    #[inline]
    pub fn resolve(self, right_to_left: bool) -> Self {
        match (self, right_to_left) {
            (Self::Start, false) | (Self::End, true) => Self::Left,
            (Self::Start, true) | (Self::End, false) => Self::Right,
            (other, _) => other,
        }
    }

    /// Whether the anchor lies on the **horizontal** axis (left/right edges),
    /// including the logical `Start` / `End` variants.
    #[inline]
    pub fn is_horizontal(self) -> bool {
        matches!(self, Self::Left | Self::Right | Self::Start | Self::End)
    }

    /// Whether the anchor lies on the **vertical** axis (top/bottom edges).
    #[inline]
    pub fn is_vertical(self) -> bool {
        matches!(self, Self::Top | Self::Bottom)
    }

    /// Whether the anchor is the container's centre.
    #[inline]
    pub fn is_center(self) -> bool {
        matches!(self, Self::Center)
    }

    /// Returns the CSS-style `justify-content`/`align-items` equivalent on
    /// the horizontal axis, if the anchor has one.
    ///
    /// Useful when the anchor needs to be converted into a
    /// `justify: start | center | end` value for layouts.
    #[inline]
    pub fn horizontal_component(self, right_to_left: bool) -> Option<HorizontalAnchor> {
        match self.resolve(right_to_left) {
            Self::Left => Some(HorizontalAnchor::Start),
            Self::Right => Some(HorizontalAnchor::End),
            Self::Center => Some(HorizontalAnchor::Center),
            _ => None,
        }
    }

    /// Returns the vertical equivalent of
    /// [`horizontal_component`](Self::horizontal_component), if the anchor
    /// has one.
    #[inline]
    pub fn vertical_component(self) -> Option<VerticalAnchor> {
        match self {
            Self::Top => Some(VerticalAnchor::Start),
            Self::Bottom => Some(VerticalAnchor::End),
            Self::Center => Some(VerticalAnchor::Center),
            _ => None,
        }
    }
}

/// Horizontal component of a [`Position`], used by layout adapters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HorizontalAnchor {
    Start,
    Center,
    End,
}

/// Vertical component of a [`Position`], used by layout adapters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VerticalAnchor {
    Start,
    Center,
    End,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // --- defaults ----------------------------------------------------------

    #[test]
    fn default_is_center() {
        assert_eq!(Position::default(), Position::Center);
    }

    // --- resolve -----------------------------------------------------------

    #[test]
    fn resolve_start_swaps_with_rtl() {
        assert_eq!(Position::Start.resolve(false), Position::Left);
        assert_eq!(Position::Start.resolve(true), Position::Right);
    }

    #[test]
    fn resolve_end_swaps_with_rtl() {
        assert_eq!(Position::End.resolve(false), Position::Right);
        assert_eq!(Position::End.resolve(true), Position::Left);
    }

    #[test]
    fn resolve_physical_anchors_are_identity() {
        for p in [
            Position::Top,
            Position::Bottom,
            Position::Left,
            Position::Right,
            Position::Center,
        ] {
            assert_eq!(p.resolve(false), p);
            assert_eq!(p.resolve(true), p);
        }
    }

    // --- axis predicates ---------------------------------------------------

    #[test]
    fn is_horizontal_covers_physical_and_logical_sides() {
        for p in [Position::Left, Position::Right, Position::Start, Position::End] {
            assert!(p.is_horizontal(), "{p:?} should be horizontal");
        }
        for p in [Position::Top, Position::Bottom, Position::Center] {
            assert!(!p.is_horizontal(), "{p:?} should not be horizontal");
        }
    }

    #[test]
    fn is_vertical_covers_top_and_bottom() {
        for p in [Position::Top, Position::Bottom] {
            assert!(p.is_vertical(), "{p:?} should be vertical");
        }
        for p in [Position::Left, Position::Right, Position::Start, Position::End, Position::Center]
        {
            assert!(!p.is_vertical(), "{p:?} should not be vertical");
        }
    }

    #[test]
    fn is_center_only_matches_center() {
        assert!(Position::Center.is_center());
        for p in [
            Position::Top,
            Position::Bottom,
            Position::Left,
            Position::Right,
            Position::Start,
            Position::End,
        ] {
            assert!(!p.is_center(), "{p:?} should not be center");
        }
    }

    #[test]
    fn axis_predicates_are_mutually_exclusive_except_center() {
        for p in [
            Position::Top,
            Position::Bottom,
            Position::Left,
            Position::Right,
            Position::Start,
            Position::End,
        ] {
            assert_ne!(
                p.is_horizontal(),
                p.is_vertical(),
                "{p:?} must be exactly one axis"
            );
            assert!(!p.is_center());
        }
        // Center is neither horizontal nor vertical, but is `is_center`.
        assert!(!Position::Center.is_horizontal());
        assert!(!Position::Center.is_vertical());
        assert!(Position::Center.is_center());
    }

    // --- component adapters ------------------------------------------------

    #[test]
    fn horizontal_component_resolves_logical_anchors() {
        assert_eq!(
            Position::Start.horizontal_component(false),
            Some(HorizontalAnchor::Start)
        );
        assert_eq!(
            Position::Start.horizontal_component(true),
            Some(HorizontalAnchor::End)
        );
        assert_eq!(
            Position::End.horizontal_component(false),
            Some(HorizontalAnchor::End)
        );
        assert_eq!(
            Position::End.horizontal_component(true),
            Some(HorizontalAnchor::Start)
        );
        assert_eq!(
            Position::Center.horizontal_component(false),
            Some(HorizontalAnchor::Center)
        );
    }

    #[test]
    fn horizontal_component_none_for_vertical_anchors() {
        assert_eq!(Position::Top.horizontal_component(false), None);
        assert_eq!(Position::Bottom.horizontal_component(true), None);
    }

    #[test]
    fn vertical_component_maps_top_and_bottom() {
        assert_eq!(Position::Top.vertical_component(), Some(VerticalAnchor::Start));
        assert_eq!(Position::Bottom.vertical_component(), Some(VerticalAnchor::End));
        assert_eq!(
            Position::Center.vertical_component(),
            Some(VerticalAnchor::Center)
        );
        assert_eq!(Position::Left.vertical_component(), None);
        assert_eq!(Position::Right.vertical_component(), None);
        assert_eq!(Position::Start.vertical_component(), None);
        assert_eq!(Position::End.vertical_component(), None);
    }

    // --- serialisation -----------------------------------------------------

    #[test]
    fn serde_round_trip_lowercase() {
        for p in [
            Position::Top,
            Position::Bottom,
            Position::Start,
            Position::End,
            Position::Left,
            Position::Right,
            Position::Center,
        ] {
            let s = serde_json::to_string(&p).unwrap();
            let lower = s.trim_matches('"').to_string();
            assert_eq!(lower, format!("{:?}", p).to_lowercase());
            let back: Position = serde_json::from_str(&s).unwrap();
            assert_eq!(back, p);
        }
    }
}