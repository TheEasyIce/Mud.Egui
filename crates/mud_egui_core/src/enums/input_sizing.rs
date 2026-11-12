//! Dimensional sizing behaviors for input components in `mud_egui`.
//!
//! This module defines the `InputSizing` enum, which dictates how text input 
//! fields (such as a multi-line `MudTextField`) respond to their internal content. 
//! It controls whether the component maintains a rigid bounding box or dynamically 
//! alters its size to fit the typed text.

/// Defines the dimensional scaling strategy for an input field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputSizing {
    /// The input maintains a rigid, predefined dimensional size. 
    ///
    /// If the user's typed text exceeds the available space within the bounding box, 
    /// the component will typically clip the text or enable internal scrolling 
    /// rather than expanding the layout.
    Fixed,

    /// The input dynamically adapts its size to accommodate its content. 
    ///
    /// For multi-line text fields, this behaves like an auto-growing text area, 
    /// expanding its vertical height automatically as the user types new lines, 
    /// ensuring all content remains visible without internal scrollbars.
    Auto,
}