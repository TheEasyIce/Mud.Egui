//! Cross-axis alignment options for layout containers in `mud_egui`.
//!
//! This module defines the `AlignItems` enum, which mirrors the CSS `align-items` property 
//! (commonly used in Flexbox grids). It dictates how child elements are aligned along 
//! the cross-axis of their parent container (e.g., vertical alignment inside a horizontal row).

/// Defines how child items are aligned along the cross-axis of a layout container.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AlignItems {
    /// Aligns items flush against the start edge of the cross-axis 
    /// (e.g., the top of a horizontal row, or the left of a vertical column).
    Start,

    /// Centers items along the cross-axis.
    Center,

    /// Aligns items flush against the end edge of the cross-axis 
    /// (e.g., the bottom of a horizontal row, or the right of a vertical column).
    End,

    /// Stretches items to fill the entire cross-axis dimension of the container.
    /// This is the standard default for many UI components (like cards or form fields) 
    /// to ensure uniform sizing.
    #[default]
    Stretch,
}