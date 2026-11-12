//! Text input modes for `mud_egui` components.
//!
//! This module defines the `InputType` enum, which dictates how text is displayed
//! and handled within a text input component (such as a `MudTextField` or `MudInput`).
//! It primarily controls whether typed characters are rendered plainly or masked for security.

/// Defines the data entry mode and visual masking behavior for text-based input fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputType {
    /// Standard plain-text input.
    ///
    /// Characters are displayed visually on the screen exactly as they are typed.
    /// This is the standard behavior for most forms, search bars, and general data entry.
    Text,

    /// Obfuscated password input.
    ///
    /// Typed characters are visually masked (typically replaced by bullets `•` or asterisks `*`)
    /// to prevent shoulder-surfing and protect sensitive user credentials.
    Password,
}