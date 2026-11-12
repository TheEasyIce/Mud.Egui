//! Material Design inspired typography scaling for `mud_egui`.
//!
//! This module defines the `Typography` configuration used by the `MudTheme`.
//! It provides a structured, semantic way to apply fonts across UI components,
//! closely mirroring the 13-tier Material Design type scale (H1-H6, Body, Subtitle, etc.)
//! that MudBlazor uses in C#.

/// Represents the styling configuration for a specific typographic scale.
///
/// # Note on `egui` rendering
/// While `egui` natively supports `font_family` and `font_size` out of the box via `FontId`,
/// advanced properties like `line_height` and `letter_spacing` are not applied automatically
/// to standard `egui::Label`s. To utilize those properties, widgets must use this variant
/// to construct a custom `egui::text::LayoutJob`.
#[derive(Debug, Clone)]
pub struct FontVariant {
    /// The name of the font family (e.g., "Google Sans", "Roboto", "Proxima Nova").
    /// In `egui`, this typically maps to a registered `FontFamily::Name`.
    pub font_family: &'static str,

    /// The size of the font in logical pixels.
    pub font_size: f32,

    /// The relative height of the line. A value of `1.5` means the line height
    /// is 150% of the `font_size`.
    pub line_height: f32,

    /// The spacing between characters in `em` units (relative to font size).
    /// Negative values tighten the text (common for large headers), while
    /// positive values loosen it (common for all-caps strings).
    pub letter_spacing: f32,

    /// Whether the text should be rendered with a bold font weight.
    pub is_bold: bool,
}

/// The global typography configuration for a theme.
///
/// This structure defines the standard semantic type scales. UI components should
/// query these variants (e.g., `theme.typography.h6`) rather than hardcoding sizes,
/// ensuring that font changes cascade beautifully throughout the entire application.
#[derive(Debug, Clone)]
pub struct Typography {
    /// The fallback variant, typically used for standard component text.
    pub default: FontVariant,

    /// Massive headers. Used sparingly for hero sections or top-level page titles.
    pub h1: FontVariant,
    /// Large section headers.
    pub h2: FontVariant,
    /// Medium section headers.
    pub h3: FontVariant,
    /// Small section headers.
    pub h4: FontVariant,
    /// Card titles or prominent component headers.
    pub h5: FontVariant,
    /// Small bold headers. Often used for dialog titles or list section headers.
    pub h6: FontVariant,

    /// Primary subtitle. Slightly larger than standard body text.
    pub subtitle1: FontVariant,
    /// Secondary subtitle. Same size as body text, but bolded for emphasis.
    pub subtitle2: FontVariant,

    /// The primary body text size. Used for long-form reading and paragraphs.
    pub body1: FontVariant,
    /// The secondary body text size. Slightly smaller, used for dense UI layouts.
    pub body2: FontVariant,

    /// The text style used inside interactive buttons.
    /// Typically bolded with slight tracking (letter spacing) for legibility.
    pub button: FontVariant,

    /// Very small text used for image captions, timestamps, or subtle hints.
    pub caption: FontVariant,

    /// Tiny text intended to be displayed in ALL CAPS above a headline.
    /// Utilizes extreme letter spacing (tracking) to remain legible at small sizes.
    pub overline: FontVariant,
}

impl Default for Typography {
    /// Generates the standard default `Typography` scale.
    ///
    /// This implementation mirrors the exact typographic scale and ratios
    /// found in the C# MudBlazor defaults, utilizing "Google Sans".
    fn default() -> Self {
        Self {
            default: FontVariant {
                font_family: "Google Sans", font_size: 14.0, line_height: 1.5, letter_spacing: 0.0, is_bold: false
            },

            // Headings: As font size increases, line-height decreases (tighter stacking)
            // and letter-spacing goes negative to close the visual gaps between large letters.
            h1: FontVariant {
                font_family: "Google Sans", font_size: 56.0, line_height: 1.15, letter_spacing: -0.02, is_bold: false
            },
            h2: FontVariant {
                font_family: "Google Sans", font_size: 44.0, line_height: 1.18, letter_spacing: -0.015, is_bold: false
            },
            h3: FontVariant {
                font_family: "Google Sans", font_size: 36.0, line_height: 1.22, letter_spacing: -0.01, is_bold: false
            },
            h4: FontVariant {
                font_family: "Google Sans", font_size: 30.0, line_height: 1.27, letter_spacing: -0.005, is_bold: false
            },
            h5: FontVariant {
                font_family: "Google Sans", font_size: 24.0, line_height: 1.33, letter_spacing: 0.0, is_bold: false
            },

            // H6 is the pivot point: smaller size (20px) but switches to Bold to maintain header hierarchy weight.
            h6: FontVariant {
                font_family: "Google Sans", font_size: 20.0, line_height: 1.4, letter_spacing: 0.0, is_bold: true
            },

            // Subtitles
            subtitle1: FontVariant {
                font_family: "Google Sans", font_size: 18.0, line_height: 1.55, letter_spacing: 0.0, is_bold: false
            },
            subtitle2: FontVariant {
                font_family: "Google Sans", font_size: 14.0, line_height: 1.5, letter_spacing: 0.0, is_bold: true
            },

            // Body text: Generous 1.5 - 1.6 line heights for reading comfort.
            body1: FontVariant {
                font_family: "Google Sans", font_size: 16.0, line_height: 1.6, letter_spacing: 0.0, is_bold: false
            },
            body2: FontVariant {
                font_family: "Google Sans", font_size: 14.0, line_height: 1.5, letter_spacing: 0.0, is_bold: false
            },

            // Button: Slightly tracked out (+0.01 spacing) to improve legibility inside constrained boxes.
            button: FontVariant {
                font_family: "Google Sans", font_size: 14.0, line_height: 1.43, letter_spacing: 0.01, is_bold: true
            },

            // Utility text
            caption: FontVariant {
                font_family: "Google Sans", font_size: 12.0, line_height: 1.5, letter_spacing: 0.02, is_bold: false
            },

            // Overline: Extreme letter spacing (+0.08) because it is intended to be used with `.to_uppercase()`.
            overline: FontVariant {
                font_family: "Google Sans", font_size: 12.0, line_height: 1.5, letter_spacing: 0.08, is_bold: true
            },
        }
    }
}