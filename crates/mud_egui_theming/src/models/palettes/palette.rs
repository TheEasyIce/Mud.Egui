//! Semantic color palette configuration for `mud_egui`.
//!
//! This module defines the core `Palette` struct, which is a direct port of the
//! standard MudBlazor palette architecture. It provides predefined Light and Dark
//! themes using semantic tokens (e.g., `primary`, `surface`, `text_secondary`)
//! rather than hardcoded hex values, allowing for cohesive and switchable theming.

use crate::utilities::mud_color::MudColor;

/// A comprehensive semantic color palette for styling UI components.
///
/// This struct holds the raw colors used to paint everything from backgrounds
/// and borders to text and interactive states.
#[derive(Debug, Clone)]
pub struct Palette {
    // ------------------------------------------------------------------------
    // Brand Colors
    // ------------------------------------------------------------------------

    /// The main brand color, used for primary actions, active states, and highlights.
    pub primary: MudColor,
    /// Text color designed to be legible on top of the `primary` color.
    pub primary_contrast_text: MudColor,

    /// A secondary accent color, used for floating action buttons or less prominent highlights.
    pub secondary: MudColor,
    /// Text color designed to be legible on top of the `secondary` color.
    pub secondary_contrast_text: MudColor,

    /// A tertiary accent color, often used for badges, tags, or alternative highlights.
    pub tertiary: MudColor,
    /// Text color designed to be legible on top of the `tertiary` color.
    pub tertiary_contrast_text: MudColor,

    // ------------------------------------------------------------------------
    // State Colors
    // ------------------------------------------------------------------------

    /// Semantic color indicating a neutral, informational state (often blue).
    pub info: MudColor,
    /// Semantic color indicating a successful action or positive state (often green).
    pub success: MudColor,
    /// Semantic color indicating a warning or non-critical issue (often amber/orange).
    pub warning: MudColor,
    /// Semantic color indicating a critical error or destructive action (often red).
    pub error: MudColor,

    // ------------------------------------------------------------------------
    // Text Colors
    // ------------------------------------------------------------------------

    /// The default color for body text. High contrast against backgrounds.
    pub text_primary: MudColor,
    /// Color for less important text, like subtitles or placeholder text. Medium contrast.
    pub text_secondary: MudColor,
    /// Color for text inside disabled components. Low contrast.
    pub text_disabled: MudColor,

    // ------------------------------------------------------------------------
    // Action / Interactive Colors
    // ------------------------------------------------------------------------

    /// The default color for interactive elements like standard icon buttons.
    pub action_default: MudColor,
    /// The color of the icon/text inside a disabled interactive element.
    pub action_disabled: MudColor,
    /// The background color of a disabled interactive element.
    pub action_disabled_background: MudColor,

    // ------------------------------------------------------------------------
    // Backgrounds & Surfaces
    // ------------------------------------------------------------------------

    /// The underlying background color of the application or main canvas.
    pub background: MudColor,
    /// An alternative background color, often used to create subtle visual grouping.
    pub background_gray: MudColor,
    /// The background color for elevated UI surfaces (Cards, Dialogs, Menus).
    pub surface: MudColor,

    // ------------------------------------------------------------------------
    // Component-Specific Backgrounds
    // ------------------------------------------------------------------------

    /// The background color of the top application navigation bar.
    pub appbar_background: MudColor,
    /// The text color used on the appbar.
    pub appbar_text: MudColor,

    /// The background color of side navigation drawers.
    pub drawer_background: MudColor,
    /// The text color used inside the drawer.
    pub drawer_text: MudColor,
    /// The color of icons used inside the drawer.
    pub drawer_icon: MudColor,

    // ------------------------------------------------------------------------
    // Lines & Borders
    // ------------------------------------------------------------------------

    /// The default color for subtle borders and structural outlines.
    pub lines_default: MudColor,
    /// The color used for the outlines of text fields and input components.
    pub lines_inputs: MudColor,
    /// The color used to separate rows/columns in a data table.
    pub table_lines: MudColor,
    /// The background color used on alternating rows in a striped data table.
    pub table_striped: MudColor,
    /// The background color used when hovering over a table row.
    pub table_hover: MudColor,
    /// The standard color for dividing lines (`<hr>`).
    pub divider: MudColor,
    /// A softer, lower-contrast version of the divider line.
    pub divider_light: MudColor,

    // ------------------------------------------------------------------------
    // Misc
    // ------------------------------------------------------------------------

    /// The color used for loading skeleton placeholders.
    pub skeleton: MudColor,

    /// The opacity modifier for generic borders.
    pub border_opacity: f32,
    /// The opacity modifier applied when a user hovers over an interactive element.
    pub hover_opacity: f32,
    /// The opacity modifier applied during a click/press animation.
    pub ripple_opacity: f32,
}

impl Palette {
    /// Creates a standard Light mode palette.
    ///
    /// This palette uses dark text on light backgrounds (Ivory/White) and
    /// defines brand colors (Clay, Blue, Green) that contrast well against white surfaces.
    pub fn new_light() -> Self {
        Self {
            primary: MudColor::parse("#B85533").unwrap(), // Clay
            primary_contrast_text: MudColor::white(),
            secondary: MudColor::parse("#3E6F9E").unwrap(),
            secondary_contrast_text: MudColor::white(),
            tertiary: MudColor::parse("#5F7347").unwrap(),
            tertiary_contrast_text: MudColor::white(),

            info: MudColor::parse("#2196F3").unwrap(),
            success: MudColor::parse("#2E7D4F").unwrap(),
            warning: MudColor::parse("#E5A823").unwrap(),
            error: MudColor::parse("#F44336").unwrap(),

            text_primary: MudColor::parse("#141413").unwrap(),
            text_secondary: MudColor::parse("#141413B3").unwrap(),
            text_disabled: MudColor::parse("#14141361").unwrap(),

            action_default: MudColor::parse("#1414138A").unwrap(),
            action_disabled: MudColor::parse("#14141342").unwrap(),
            action_disabled_background: MudColor::parse("#1414131F").unwrap(),

            background: MudColor::parse("#FAF9F5").unwrap(), // Ivory
            background_gray: MudColor::parse("#F0EEE6").unwrap(),
            surface: MudColor::white(),

            appbar_background: MudColor::white(),
            appbar_text: MudColor::parse("#141413").unwrap(),

            drawer_background: MudColor::parse("#F5F4EE").unwrap(),
            drawer_text: MudColor::parse("#141413").unwrap(),
            drawer_icon: MudColor::parse("#1414138A").unwrap(),

            lines_default: MudColor::parse("#1414131F").unwrap(),
            lines_inputs: MudColor::parse("#14141380").unwrap(),
            table_lines: MudColor::parse("#1414131F").unwrap(),
            table_striped: MudColor::parse("#1414130A").unwrap(),
            table_hover: MudColor::parse("#1414130F").unwrap(),
            divider: MudColor::parse("#1414131F").unwrap(),
            divider_light: MudColor::parse("#1414130F").unwrap(),

            skeleton: MudColor::parse("#1414131C").unwrap(),

            border_opacity: 1.0,
            hover_opacity: 0.06,
            ripple_opacity: 0.1,
        }
    }

    /// Creates a standard Dark mode palette.
    ///
    /// This palette uses light text on dark backgrounds and adjusts brand/state
    /// colors to be slightly lighter and less saturated to reduce eye strain and
    /// maintain contrast against dark surfaces.
    pub fn new_dark() -> Self {
        Self {
            primary: MudColor::parse("#D97757").unwrap(),
            primary_contrast_text: MudColor::parse("#141413").unwrap(),
            secondary: MudColor::parse("#6A9BCC").unwrap(),
            secondary_contrast_text: MudColor::parse("#141413").unwrap(),
            tertiary: MudColor::parse("#788C5D").unwrap(),
            tertiary_contrast_text: MudColor::parse("#141413").unwrap(),

            info: MudColor::parse("#2196F3").unwrap(),
            success: MudColor::parse("#5DB075").unwrap(),
            warning: MudColor::parse("#E8B84A").unwrap(),
            error: MudColor::parse("#F44336").unwrap(),

            text_primary: MudColor::parse("#FAF9F5").unwrap(),
            text_secondary: MudColor::parse("#FAF9F5B3").unwrap(),
            text_disabled: MudColor::parse("#FAF9F561").unwrap(),

            action_default: MudColor::parse("#FAF9F599").unwrap(),
            action_disabled: MudColor::parse("#FAF9F561").unwrap(),
            action_disabled_background: MudColor::parse("#FAF9F51F").unwrap(),

            background: MudColor::parse("#1F1E1D").unwrap(),
            background_gray: MudColor::parse("#2B2A27").unwrap(),
            surface: MudColor::parse("#2B2A27").unwrap(),

            appbar_background: MudColor::parse("#2B2A27").unwrap(),
            appbar_text: MudColor::parse("#FAF9F5").unwrap(),

            drawer_background: MudColor::parse("#1F1E1D").unwrap(),
            drawer_text: MudColor::parse("#FAF9F5").unwrap(),
            drawer_icon: MudColor::parse("#FAF9F5").unwrap(),

            lines_default: MudColor::parse("#FAF9F51F").unwrap(),
            lines_inputs: MudColor::parse("#FAF9F54D").unwrap(),
            table_lines: MudColor::parse("#FAF9F51F").unwrap(),
            table_striped: MudColor::parse("#FAF9F533").unwrap(),
            table_hover: MudColor::parse("#FAF9F529").unwrap(),
            divider: MudColor::parse("#FAF9F51F").unwrap(),
            divider_light: MudColor::parse("#FAF9F50F").unwrap(),

            skeleton: MudColor::parse("#FAF9F51C").unwrap(),

            border_opacity: 0.12,
            hover_opacity: 0.08,
            ripple_opacity: 0.16,
        }
    }

    // ------------------------------------------------------------------------
    // Computed Variants
    // ------------------------------------------------------------------------

    /// Returns a darkened variant of the primary color.
    /// Matches MudBlazor C# behavior (darkened by 7.5% lightness in HSL space).
    pub fn primary_darken(&self) -> MudColor { self.primary.darken(0.075) }

    /// Returns a lightened variant of the primary color.
    /// Matches MudBlazor C# behavior (lightened by 7.5% lightness in HSL space).
    pub fn primary_lighten(&self) -> MudColor { self.primary.lighten(0.075) }

    /// Returns a darkened variant of the secondary color.
    /// Matches MudBlazor C# behavior (darkened by 7.5% lightness in HSL space).
    pub fn secondary_darken(&self) -> MudColor { self.secondary.darken(0.075) }

    /// Returns a lightened variant of the secondary color.
    /// Matches MudBlazor C# behavior (lightened by 7.5% lightness in HSL space).
    pub fn secondary_lighten(&self) -> MudColor { self.secondary.lighten(0.075) }
}