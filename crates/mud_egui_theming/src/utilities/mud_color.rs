//! MudBlazor-inspired color utilities for `egui`.
//!
//! This module provides the `MudColor` struct, which exists to bridge the gap between
//! standard CSS-like color definitions (Hex, RGB, HSL) and `egui::Color32`.
//! It provides built-in color math (lightening, darkening) using HSL color space,
//! which `egui` does not support natively out of the box.

use egui::Color32;

/// A framework-agnostic RGBA color representation designed for CSS-like manipulation.
///
/// Under the hood, this stores unmultiplied RGBA channels as standard `u8`s.
/// It implements `Into<Color32>` so it can be seamlessly passed into `egui` rendering functions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct MudColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl MudColor {
    /// Creates a new `MudColor` from unmultiplied RGBA byte values.
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Returns a completely transparent color.
    pub const fn transparent() -> Self { Self::new(0, 0, 0, 0) }

    /// Returns pure opaque black.
    pub const fn black() -> Self { Self::new(0, 0, 0, 255) }

    /// Returns pure opaque white.
    pub const fn white() -> Self { Self::new(255, 255, 255, 255) }

    /// Parses a standard CSS Hex color string.
    ///
    /// # Supported Formats
    /// - 3-digit: `#RGB` (Expands to `#RRGGBB` with full opacity)
    /// - 4-digit: `#RGBA` (Expands to `#RRGGBBAA`)
    /// - 6-digit: `#RRGGBB` (Full opacity)
    /// - 8-digit: `#RRGGBBAA`
    pub fn parse(hex: &str) -> Option<Self> {
        let hex = hex.trim_start_matches('#');

        match hex.len() {
            // 3-digit shorthand (e.g., "F00" -> Red)
            // Multiplying by 17 (0x11) expands "F" (15) to "FF" (255).
            3 => {
                let r = u8::from_str_radix(&hex[0..1], 16).ok()? * 17;
                let g = u8::from_str_radix(&hex[1..2], 16).ok()? * 17;
                let b = u8::from_str_radix(&hex[2..3], 16).ok()? * 17;
                Some(Self::new(r, g, b, 255))
            }
            // 4-digit shorthand (e.g., "F008" -> Semi-transparent Red)
            4 => {
                let r = u8::from_str_radix(&hex[0..1], 16).ok()? * 17;
                let g = u8::from_str_radix(&hex[1..2], 16).ok()? * 17;
                let b = u8::from_str_radix(&hex[2..3], 16).ok()? * 17;
                let a = u8::from_str_radix(&hex[3..4], 16).ok()? * 17;
                Some(Self::new(r, g, b, a))
            }
            // Standard 6-digit hex (e.g., "FF0000" -> Red)
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                Some(Self::new(r, g, b, 255))
            }
            // 8-digit hex with explicit alpha (e.g., "FF000080" -> 50% Red)
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
                Some(Self::new(r, g, b, a))
            }
            _ => None,
        }
    }

    /// Converts the RGB color to the HSL (Hue, Saturation, Lightness) color space.
    ///
    /// # Returns
    /// A tuple of `(h, s, l)` where:
    /// - `h` (Hue) is degrees from `0.0` to `360.0`.
    /// - `s` (Saturation) is a float from `0.0` to `1.0`.
    /// - `l` (Lightness) is a float from `0.0` to `1.0`.
    fn to_hsl(&self) -> (f32, f32, f32) {
        // Normalize RGB values to 0.0 - 1.0
        let r = self.r as f32 / 255.0;
        let g = self.g as f32 / 255.0;
        let b = self.b as f32 / 255.0;

        let max = r.max(g).max(b);
        let min = r.min(g).min(b);

        // Lightness is always the midpoint between the max and min RGB values.
        let l = (max + min) / 2.0;

        // If min and max are the same, there is no saturation (it's a shade of gray)
        // and hue is technically undefined (we default to 0.0).
        if max == min {
            return (0.0, 0.0, l);
        }

        let d = max - min;

        // Calculate Saturation based on Lightness
        let s = if l > 0.5 { d / (2.0 - max - min) } else { d / (max + min) };

        // Calculate Hue by finding which color channel is dominant
        let h = if max == r {
            (g - b) / d + (if g < b { 6.0 } else { 0.0 })
        } else if max == g {
            (b - r) / d + 2.0
        } else {
            (r - g) / d + 4.0
        };

        // Convert hue to standard 360-degree scale
        (h * 60.0, s, l)
    }

    /// Reconstructs a `MudColor` from HSL values and an explicit alpha channel.
    fn from_hsl(h: f32, s: f32, l: f32, a: u8) -> Self {
        // Standard HSL to RGB conversion math (Chroma and intermediate X)
        let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
        let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
        let m = l - c / 2.0;

        // Map the Chroma and X values to RGB based on the 60-degree sector of the hue circle.
        let (r1, g1, b1) = if h < 60.0 { (c, x, 0.0) }
        else if h < 120.0 { (x, c, 0.0) }
        else if h < 180.0 { (0.0, c, x) }
        else if h < 240.0 { (0.0, x, c) }
        else if h < 300.0 { (x, 0.0, c) }
        else { (c, 0.0, x) };

        // Add the lightness match (m) and scale back up to 0-255 u8 space.
        let to_byte = |v: f32| ((v + m) * 255.0).round().clamp(0.0, 255.0) as u8;
        Self::new(to_byte(r1), to_byte(g1), to_byte(b1), a)
    }

    /// Darkens the color by decreasing its lightness in the HSL color space.
    ///
    /// # Arguments
    /// * `amount` - A float from `0.0` to `1.0` representing the percentage to subtract from lightness.
    pub fn darken(&self, amount: f32) -> Self {
        let (h, s, l) = self.to_hsl();
        Self::from_hsl(h, s, (l - amount).clamp(0.0, 1.0), self.a)
    }

    /// Lightens the color by increasing its lightness in the HSL color space.
    ///
    /// # Arguments
    /// * `amount` - A float from `0.0` to `1.0` representing the percentage to add to lightness.
    pub fn lighten(&self, amount: f32) -> Self {
        let (h, s, l) = self.to_hsl();
        Self::from_hsl(h, s, (l + amount).clamp(0.0, 1.0), self.a)
    }

    /// Returns a copy of the color with its alpha channel replaced.
    ///
    /// # Arguments
    /// * `alpha` - A float from `0.0` to `1.0` (0.0 is fully transparent, 1.0 is fully opaque).
    pub fn with_alpha(&self, alpha: f32) -> Self {
        Self::new(self.r, self.g, self.b, (alpha * 255.0).round() as u8)
    }
}

// ----------------------------------------------------------------------------
// Type Conversions
// ----------------------------------------------------------------------------

/// Seamless conversion from `MudColor` into `egui::Color32`.
///
/// We use `from_rgba_unmultiplied` because UI frameworks and CSS define colors in
/// unmultiplied (straight) RGB space, whereas `egui` often prefers pre-multiplied internally.
impl From<MudColor> for Color32 {
    fn from(c: MudColor) -> Self {
        Color32::from_rgba_unmultiplied(c.r, c.g, c.b, c.a)
    }
}