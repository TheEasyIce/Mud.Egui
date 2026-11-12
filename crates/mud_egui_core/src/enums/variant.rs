//! Visual presentation variants for `mud_egui` components.
//!
//! This module defines the `Variant` enum, which is a direct port of MudBlazor's 
//! `Variant` property. It dictates the structural presentation—specifically background 
//! fills and borders—for interactive components such as buttons, text fields, and chips, 
//! adhering to classic Material Design guidelines.

/// Defines the structural and visual presentation style of a component.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Variant {
    /// A minimalist style with no background fill and no border. 
    ///
    /// For components like buttons, this relies entirely on typography and hover states 
    /// to indicate interactivity. Often used for less prominent, secondary actions. 
    /// This is the standard default.
    #[default]
    Text,

    /// A style with a transparent background but a distinct solid outline (stroke). 
    ///
    /// Used for medium-emphasis actions that require clear boundaries without 
    /// drawing as much visual attention as a filled component.
    Outlined,

    /// A high-emphasis style with a solid background fill color. 
    ///
    /// Commonly used for primary actions or highly visible form inputs. 
    /// For buttons, this variant is frequently accompanied by elevation shadows.
    Filled,
}