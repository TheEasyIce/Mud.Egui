//! Semantic color abstractions for `mud_egui`.
//!
//! This module defines the `Color` enum, which represents semantic color roles 
//! (like "Primary", "Error", or "TextSecondary") rather than hardcoded RGBA values. 
//!
//! By using semantic colors across all UI components, the application can easily 
//! switch between Light and Dark themes—or entirely different palettes—without 
//! changing a single line of component code.

use egui::Color32;
use mud_egui_theming::models::palettes::palette::Palette;

/// Semantic color roles mapped to the active `MudTheme` palette.
///
/// This enum is passed into component builders (e.g., `MudText::new("Hi").color(Color::Primary)`).
/// It defers the actual calculation of physical pixels until the render pass, where 
/// the current theme's palette is injected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Color {
    /// The standard default color. For text, this typically resolves to `text_primary`.
    #[default]
    Default,

    // --- Brand Colors ---

    /// The primary brand color. Used for prominent buttons and active states.
    Primary,
    /// A secondary accent color. Used for floating action buttons or alternative highlights.
    Secondary,
    /// A tertiary accent color. Used for badges or subtle accents.
    Tertiary,

    // --- State Colors ---

    /// Indicates a neutral, informational state (usually blue).
    Info,
    /// Indicates a successful action or positive state (usually green).
    Success,
    /// Indicates a warning or non-critical issue (usually amber/orange).
    Warning,
    /// Indicates a critical error or destructive action (usually red).
    Error,

    // --- Text-Specific Roles ---

    /// High-contrast text designed to be legible on standard background surfaces.
    TextPrimary,
    /// Medium-contrast text for subtitles or less important information.
    TextSecondary,
    /// Low-contrast text for disabled components.
    TextDisabled,

    // --- Special Directives ---

    /// Tells the component *not* to enforce a color, allowing it to inherit 
    /// whatever text or fill color the parent `egui::Ui` context currently dictates.
    Inherit,

    /// Forces the element to be completely transparent.
    Transparent,

    /// An escape hatch: allows you to explicitly provide an `egui::Color32`, 
    /// completely bypassing the MudBlazor theming engine.
    Custom(Color32),
}

impl Color {
    /// Resolves the semantic color role into a concrete, physical `egui::Color32`.
    ///
    /// # Arguments
    /// * `palette` - The currently active Light or Dark `Palette` (usually fetched from `MudTheme`).
    ///
    /// # Returns
    /// An `Option<Color32>`. It returns `Some(Color32)` for all standard and custom colors.
    /// It returns `None` **only** when the color is `Inherit`, signaling to the rendering 
    /// component that it should fall back to `egui`'s default UI styling.
    pub fn resolve(self, palette: &Palette) -> Option<Color32> {
        match self {
            // Default usually implies standard body text.
            Color::Default | Color::TextPrimary => Some(palette.text_primary.into()),

            // Brand roles
            Color::Primary => Some(palette.primary.into()),
            Color::Secondary => Some(palette.secondary.into()),
            Color::Tertiary => Some(palette.tertiary.into()),

            // State roles
            Color::Info => Some(palette.info.into()),
            Color::Success => Some(palette.success.into()),
            Color::Warning => Some(palette.warning.into()),
            Color::Error => Some(palette.error.into()),

            // Text roles
            Color::TextSecondary => Some(palette.text_secondary.into()),
            Color::TextDisabled => Some(palette.text_disabled.into()),

            // Directives
            // Returning `None` tells the component rendering logic to skip applying 
            // a color override, thereby inheriting the parent UI's text/fill color.
            Color::Inherit => None,
            Color::Transparent => Some(Color32::TRANSPARENT),

            // Escape hatch
            Color::Custom(c) => Some(c),
        }
    }
}