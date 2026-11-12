//! Main-axis alignment and distribution options for layout containers in `mud_egui`.
//!
//! This module defines the `Justify` enum, which directly mirrors the CSS `justify-content`
//! property (commonly used in Flexbox layouts). It dictates how extra space along the
//! **main-axis** of a container is distributed between and around its child items.

/// Defines how child items are distributed along the main-axis of a layout container.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Justify {
    /// Packs items toward the start edge of the main-axis.
    /// Any remaining free space is left at the end. This is the standard default.
    #[default]
    Start,

    /// Centers items along the main-axis, dividing any remaining free space equally
    /// on both sides of the group.
    Center,

    /// Packs items toward the end edge of the main-axis.
    /// Any remaining free space is placed at the start.
    End,

    /// Distributes items evenly along the main-axis.
    /// The first item is flush against the start edge, the last item is flush against
    /// the end edge, and the remaining space is divided equally between the items.
    SpaceBetween,

    /// Distributes items evenly along the main-axis, with half-size spaces
    /// placed at the very beginning and very end, and full-size spaces between the items.
    SpaceAround,

    /// Distributes items evenly so that the spacing between any two items
    /// (as well as the space before the first item and after the last item) is absolute equal.
    SpaceEvenly,
}