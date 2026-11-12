//! Positioning and attachment variants for `mud_egui` components.
//!
//! This module defines the `Placement` enum, which dictates the physical or
//! logical positioning of transient or anchored components (such as popovers,
//! tooltips, badges, or side drawers) relative to their parent elements or the screen.

/// Defines the position of a component relative to its anchor or container.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Placement {
    /// Positions the component at the logical starting edge.
    ///
    /// In standard Left-To-Right (LTR) layouts, this is visually equivalent to `Left`.
    /// In Right-To-Left (RTL) layouts, this dynamically maps to the right edge.
    Start,

    /// Positions the component at the logical ending edge.
    ///
    /// In standard Left-To-Right (LTR) layouts, this is visually equivalent to `Right`.
    /// In Right-To-Left (RTL) layouts, this dynamically maps to the left edge.
    End,

    /// Positions the component explicitly on the absolute physical left side,
    /// ignoring the current text direction/reading flow.
    Left,

    /// Positions the component explicitly on the absolute physical right side,
    /// ignoring the current text direction/reading flow.
    Right,

    /// Positions the component explicitly on the absolute top edge.
    Top,

    /// Positions the component explicitly on the absolute bottom edge.
    Bottom,
}