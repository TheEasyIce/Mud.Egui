//! Flexbox wrapping behavior for layout containers in `mud_egui`.
//!
//! This module defines the `Wrap` enum, which mirrors the CSS `flex-wrap` property. 
//! It dictates whether child items in a flex-style container are forced onto a single line 
//! or permitted to wrap onto multiple lines when they exceed the available main-axis space.

/// Controls whether child items wrap to multiple lines or remain on a single line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Wrap {
    /// Forces all child items onto a single line. If the items exceed the available 
    /// container width or height, they may overflow or clip. This is the default.
    #[default]
    NoWrap,

    /// Permits child items to wrap onto multiple lines (rows or columns) 
    /// as space runs out along the main-axis.
    Wrap,

    /// Permits child items to wrap onto multiple lines, but wraps them in 
    /// reverse order (e.g., subsequent lines stack upward or leftward).
    WrapReverse,
}