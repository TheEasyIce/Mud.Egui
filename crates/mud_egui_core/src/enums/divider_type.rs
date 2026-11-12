//! Divider style variants for `mud_egui` components.
//!
//! This module defines the `DividerType` enum, which dictates the layout and
//! indentation styling of a divider component (such as `MudDivider`). It mirrors
//! the visual presentation variants found in Material Design and MudBlazor.

/// Defines the style and indentation variant for a horizontal or vertical divider line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DividerType {
    /// The divider spans the entire width of its parent container without any indentation.
    /// This is the standard default.
    #[default]
    FullWidth,

    /// The divider is indented from the start edge (left or leading side),
    /// typically leaving visual space for an avatar, icon, or list item prefix.
    Inset,

    /// The divider is inset evenly from both ends (left and right / start and end),
    /// creating a centered horizontal rule with balanced margins on both sides.
    Middle,
}