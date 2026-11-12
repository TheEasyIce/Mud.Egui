//! Bootstrap-style border utilities (`border-*`) for `egui`.
//!
//! This module provides an efficient parser to translate standard Bootstrap `border` classes
//! (such as `border-2`, `border-dashed`, `border-dotted-3`) into a configuration that can
//! be applied to `egui` components.
//!
//! # Differences from Avalonia
//! In Avalonia, borders are drawn by the control itself, so the C# code uses an
//! `Adorner` to paint dashed/dotted lines over the control and hides the native border.
//! In `egui` (being an immediate-mode GUI), we simply resolve the configuration
//! (style + width) here and let your widget pass it directly into an `egui::Frame`
//! or `egui::Stroke` during the render pass.
//!
//! # Supported Classes
//! - Styles: `border-solid`, `border-dashed`, `border-dotted`
//! - Generic widths: `border-1`, `border-2`, etc.
//! - Combined: `border-dashed-3`, `border-dotted-2`

use egui::Stroke;
use mud_egui_core::base::component_base::ComponentBase;

/// The available visual styles for a border.
///
/// Maps directly to standard CSS `border-style` properties.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BorderStyle {
    /// A standard, continuous solid line.
    Solid,
    /// A line consisting of short dashes.
    Dashed,
    /// A line consisting of round dots.
    Dotted,
}

/// The resolved configuration for a border.
///
/// This struct holds the final computed style and width after all classes
/// have been parsed. It is designed to be easily convertible into an `egui::Stroke`
/// once combined with a color.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BorderConfig {
    /// The visual pattern of the border.
    pub style: BorderStyle,
    /// The thickness of the border in raw pixels.
    pub width: f32,
}

impl Default for BorderConfig {
    /// Defaults to a standard 1px solid border, matching browser default behavior
    /// when a border is applied without explicit styles.
    fn default() -> Self {
        Self {
            style: BorderStyle::Solid,
            width: 1.0,
        }
    }
}

/// Caches the parsed border configuration.
///
/// String parsing on every single frame is a major performance anti-pattern in
/// immediate-mode GUIs like `egui`. `BsBorderCache` tracks the class list's version
/// and only re-evaluates the classes when the component's state actually changes.
#[derive(Default)]
pub struct BsBorderCache {
    /// The version of the class list when this cache was last updated.
    version: u64,
    /// The cached border configuration, or `None` if no border classes were applied.
    config: Option<BorderConfig>,
}

impl BsBorderCache {
    /// Resolves the border config from the classes on the `ComponentBase`.
    ///
    /// # Arguments
    /// * `base` - The component providing the `class_names` and `classes_version`.
    ///
    /// # Returns
    /// Returns `Some(BorderConfig)` if a valid border style was found, otherwise `None`.
    pub fn resolve(&mut self, base: &ComponentBase) -> Option<BorderConfig> {
        // Cache hit: Fast path using a cheap u64 equality check.
        if self.version == base.classes_version() {
            return self.config;
        }

        let mut style = None;

        // We track two different width types to handle CSS overrides correctly:
        // 1. `width_step`: Attached directly to a style (e.g., `border-dashed-3`).
        // 2. `generic_step`: A standalone width declaration (e.g., `border-2`).
        let mut width_step = None;
        let mut generic_step = None;

        // Iterate through all classes.
        // Rule: "The last class written wins." This is a simplified specificity model
        // that relies on the order of classes in the string to determine priority.
        for cls in base.class_names() {
            // Only process classes starting with "border-"
            if let Some(payload) = cls.strip_prefix("border-") {

                // Attempt to split the payload by `-` to separate styles from widths.
                // e.g., "dashed-3" -> token: "dashed", step_str: Some("3")
                // e.g., "solid"    -> token: "solid",  step_str: None
                let (token, step_str) = match payload.find('-') {
                    Some(idx) => (&payload[..idx], Some(&payload[idx + 1..])),
                    None => (payload, None),
                };

                // Determine if the primary token is a recognized style.
                let parsed_style = match token {
                    "solid" => Some(BorderStyle::Solid),
                    "dashed" => Some(BorderStyle::Dashed),
                    "dotted" => Some(BorderStyle::Dotted),
                    _ => None,
                };

                if let Some(s) = parsed_style {
                    // A style was found. Update the active style (last one wins).
                    style = Some(s);

                    // If the style had an attached width suffix (e.g., "dashed-3"),
                    // parse it into a float and save it as the specific width step.
                    if let Some(st) = step_str.and_then(|s| s.parse::<u32>().ok()) {
                        width_step = Some(st as f32);
                    }
                } else if let Ok(st) = payload.parse::<u32>() {
                    // The payload wasn't a style, but it IS a number.
                    // This handles generic width classes like "border-2".
                    generic_step = Some(st as f32);
                }
            }
        }

        // Final Resolution:
        // We only generate a config if a specific `style` was requested.
        // Width priority: Specific style width (`border-dashed-3`) overrides
        // generic width (`border-2`), which overrides the default (`1.0`).
        let config = style.map(|s| BorderConfig {
            style: s,
            width: width_step.or(generic_step).unwrap_or(1.0),
        });

        // Update the cache for future frames.
        self.version = base.classes_version();
        self.config = config;

        config
    }
}