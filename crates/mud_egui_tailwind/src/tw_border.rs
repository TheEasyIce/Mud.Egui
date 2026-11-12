//! Tailwind-style border utilities (`border-*`) for `egui`.
//!
//! This module provides a parser for Tailwind CSS border classes.
//! Unlike Bootstrap where widths and styles are often combined (e.g., `border-dashed-3`),
//! Tailwind composes them as entirely separate classes that accumulate.
//!
//! # Supported Syntax
//! - **Base**: `border` (applies a 1px solid border).
//! - **Styles**: `border-solid`, `border-dashed`, `border-dotted`, `border-double`, `border-none`.
//! - **Standard Widths**: `border-2`, `border-4`, `border-8`.
//! - **Arbitrary Widths**: `border-[3px]`, `border-[0.25rem]`, `border-[2.5]`.

use egui::Stroke;
use mud_egui_core::base::component_base::ComponentBase;

/// The visual style of the Tailwind border.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TwBorderStyle {
    /// Standard continuous line.
    Solid,
    /// Line consisting of short dashes.
    Dashed,
    /// Line consisting of round dots.
    Dotted,
    /// Two parallel solid lines (may require custom drawing in egui).
    Double,
    /// Explicitly removes the border (`border-none` or `border-hidden`).
    None,
}

/// The resolved configuration for a Tailwind border.
///
/// Tailwind calculates borders by combining independent classes.
/// For example, `border-2 border-dashed` results in `{ style: Dashed, width: 2.0 }`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TwBorderConfig {
    /// The visual pattern of the border.
    pub style: TwBorderStyle,
    /// The thickness of the border in physical pixels.
    pub width: f32,
}

/// A cache for the computed Tailwind border class configuration.
///
/// Parsing string classes on every frame is expensive in immediate-mode GUIs.
/// `TwBorderCache` tracks a `version` (usually representing the component's state)
/// and caches the resulting config so parsing only occurs when classes change.
#[derive(Default)]
pub struct TwBorderCache {
    /// The version of the class list when this cache was last updated.
    version: u64,
    /// The cached border configuration, or `None` if no border classes were found.
    config: Option<TwBorderConfig>,
}

impl TwBorderCache {
    /// Resolves the Tailwind border configuration based on the current classes.
    ///
    /// # Arguments
    /// * `base` - The component providing the `class_names` and `classes_version`.
    /// * `rem` - The current scale factor for converting `rem` to pixels (usually 16.0).
    pub fn resolve(&mut self, base: &ComponentBase, rem: f32) -> Option<TwBorderConfig> {
        // Cache hit: avoid re-parsing if the class list is unchanged.
        if self.version == base.classes_version() {
            return self.config;
        }

        // We track style and width independently because Tailwind handles them as separate classes.
        let mut style: Option<TwBorderStyle> = None;
        let mut width: Option<f32> = None;

        // Iterate through classes. "Last class written wins" for overrides.
        for cls in base.class_names() {
            if cls.starts_with("border-") {
                // Strip "border-" prefix
                let payload = &cls[7..];

                // First, check if the payload matches a known style string.
                match payload {
                    "solid" => style = Some(TwBorderStyle::Solid),
                    "dashed" => style = Some(TwBorderStyle::Dashed),
                    "dotted" => style = Some(TwBorderStyle::Dotted),
                    "double" => style = Some(TwBorderStyle::Double),
                    "none" | "hidden" => style = Some(TwBorderStyle::None),
                    _ => {
                        // If it's not a style, attempt to parse it as a width value.
                        // (e.g., "2", "[3px]", "[1.5rem]")
                        if let Some(w) = parse_width(payload, rem) {
                            width = Some(w);
                        }
                    }
                }
            } else if cls == "border" {
                // The bare "border" class in Tailwind defaults to a 1px solid border.
                // We only set the width here. The style will default to Solid during resolution.
                width = Some(1.0);
            }
        }

        // Final Resolution:
        // We only generate a config if at least one border class (style or width) was found.
        let result = if style.is_some() || width.is_some() {
            Some(TwBorderConfig {
                // Tailwind default: If width is defined but no style is given, it's solid.
                style: style.unwrap_or(TwBorderStyle::Solid),
                // Tailwind default: If style is defined but no width is given, it's 1px.
                width: width.unwrap_or(1.0),
            })
        } else {
            None
        };

        // Update the cache for future frames.
        self.version = base.classes_version();
        self.config = result;

        result
    }
}

/// Parses the width portion of a Tailwind border class.
///
/// Handles standard numbers (`2`), and arbitrary bracket notation
/// with various units (`[2px]`, `[0.5rem]`, `[3.5]`).
///
/// # Arguments
/// * `payload` - The string following the "border-" prefix (e.g., "2" or "[3px]").
/// * `rem` - The pixel multiplier for rem values.
fn parse_width(payload: &str, rem: f32) -> Option<f32> {
    // Check for arbitrary value syntax: `[...]`
    if let Some(inner) = payload.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {

        // Handle CSS 'rem' units: multiply by the rem conversion factor
        if let Some(rem_val) = inner.strip_suffix("rem") {
            return rem_val.parse::<f32>().ok().map(|n| n * rem);
        }

        // Handle CSS 'px' units: parse directly to float
        if let Some(px_val) = inner.strip_suffix("px") {
            return px_val.parse::<f32>().ok();
        }

        // Handle unitless arbitrary values: `border-[3.5]`
        return inner.parse::<f32>().ok();
    }

    // Normal predefined Tailwind scale number (e.g., "2", "4", "8")
    payload.parse::<f32>().ok()
}