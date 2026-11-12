//! Inline utility-class parsing engine for `mud_egui` components.
//!
//! This module provides the `Utilities` and `Edges` structs, which parse standard
//! CSS utility classes (such as `p-4`, `mx-2`, `pt-0.5`, or `bg-primary`) directly into
//! native `egui::Margin` and semantic color roles.
//!
//! This allows lightweight components (like `MudText`) to support tailwind/bootstrap-style
//! spacing and background rules inline without requiring a heavy layout engine.

use egui::Margin;
use mud_egui_core::enums::color::Color;

/// The base spacing multiplier used by spacing utility classes.
/// Standard Bootstrap / Tailwind / MudBlazor mapping: 1 spacing unit = 0.25rem = 4 logical pixels.
pub const SPACING_UNIT: f32 = 4.0;

/// Represents per-side geometric values (padding or margin) in logical pixels.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Edges {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl Edges {
    /// Returns `true` if all four edges are set to exactly zero pixels.
    pub fn is_zero(&self) -> bool {
        self.top == 0.0 && self.right == 0.0 && self.bottom == 0.0 && self.left == 0.0
    }

    /// Converts the `f32` logical pixel values into an `egui::Margin` configuration.
    ///
    /// # Note on `egui` constraints
    /// `egui::Margin` stores values as signed 8-bit integers (`i8`). This method rounds
    /// the floating-point pixels and clamps them safely between `0` and `i8::MAX`
    /// to prevent numeric overflow panics.
    pub fn to_margin(self) -> Margin {
        let px = |v: f32| v.round().clamp(0.0, i8::MAX as f32) as i8;
        Margin {
            left: px(self.left),
            right: px(self.right),
            top: px(self.top),
            bottom: px(self.bottom),
        }
    }
}

/// A structured collection of parsed utility class properties.
///
/// Tracks padding, margins, and optional background color roles extracted
/// from a component's class list.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Utilities {
    /// Padding configuration across all four edges.
    pub padding: Edges,
    /// Margin configuration across all four edges.
    pub margin: Edges,
    /// Optional semantic background color role.
    pub background: Option<Color>,
}

impl Utilities {
    /// Parses an iterator of class strings into a structured `Utilities` object.
    ///
    /// Whitespace around class names is automatically trimmed. Unrecognized or
    /// malformed classes are safely ignored.
    pub fn parse<'a>(classes: impl IntoIterator<Item = &'a str>) -> Self {
        let mut out = Self::default();
        for class in classes {
            out.apply(class.trim());
        }
        out
    }

    /// Returns `true` if no valid padding, margin, or background classes were found.
    ///
    /// When `true`, components can skip wrapping themselves in an `egui::Frame`,
    /// optimizing rendering performance.
    pub fn is_empty(&self) -> bool {
        self.padding.is_zero() && self.margin.is_zero() && self.background.is_none()
    }

    /// Internal helper that parses and applies a single utility class string
    /// (e.g., `"px-4"` or `"bg-error"`).
    fn apply(&mut self, class: &str) {
        // All supported utility classes must contain a hyphen separating the prefix and value.
        let Some((prefix, value)) = class.split_once('-') else {
            return;
        };

        // --------------------------------------------------------------------
        // 1. Background Color Classes (e.g., "bg-primary")
        // --------------------------------------------------------------------
        if prefix == "bg" {
            self.background = match value {
                "primary" => Some(Color::Primary),
                "secondary" => Some(Color::Secondary),
                "tertiary" => Some(Color::Tertiary),
                "info" => Some(Color::Info),
                "success" => Some(Color::Success),
                "warning" => Some(Color::Warning),
                "error" => Some(Color::Error),
                "transparent" => Some(Color::Transparent),
                // Fallback to existing background if the color name is unrecognized
                _ => self.background,
            };
            return;
        }

        // --------------------------------------------------------------------
        // 2. Spacing Classes (Padding 'p-*' and Margin 'm-*')
        // --------------------------------------------------------------------

        // Ensure the value suffix is a valid, finite, non-negative floating-point number.
        let Ok(n) = value.parse::<f32>() else { return };
        if !n.is_finite() || n < 0.0 {
            return;
        }

        // Convert the unitless multiplier into actual pixels (1 unit = 4px).
        let px = n * SPACING_UNIT;

        // Separate the property prefix ('p' or 'm') from the directional modifier (e.g., 'x', 't', 'left')
        let (edges, side) = match prefix.as_bytes() {
            [b'p', rest @ ..] => (&mut self.padding, rest),
            [b'm', rest @ ..] => (&mut self.margin, rest),
            _ => return, // Unrecognized prefix (not 'p' or 'm')
        };

        // Apply the pixel value to the correct edge(s) based on the directional specifier
        match side {
            // No suffix or 'a' suffix = All sides (e.g., `p-4` or `pa-4`)
            b"" | b"a" => *edges = Edges { top: px, right: px, bottom: px, left: px },

            // 'x' suffix = Horizontal axis (Left and Right)
            b"x" => {
                edges.left = px;
                edges.right = px;
            }

            // 'y' suffix = Vertical axis (Top and Bottom)
            b"y" => {
                edges.top = px;
                edges.bottom = px;
            }

            // 't' suffix = Top edge only
            b"t" => edges.top = px,

            // 'b' suffix = Bottom edge only
            b"b" => edges.bottom = px,

            // 'l' or 's' suffix = Left edge (or Start edge in logical properties)
            b"l" | b"s" => edges.left = px,

            // 'r' or 'e' suffix = Right edge (or End edge in logical properties)
            b"r" | b"e" => edges.right = px,

            // Unrecognized directional modifier
            _ => {}
        }
    }
}