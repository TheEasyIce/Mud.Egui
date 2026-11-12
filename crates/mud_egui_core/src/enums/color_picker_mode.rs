//! `ColorPickerMode` — which channels a colour picker's inputs show.
//!
//! Port of `MudBlazor.ColorPickerMode` (MudBlazor 8.13.0). Discriminants match
//! the C# enum exactly[reference:14].
//!
//! Kept in its own file per Rule 2.

use std::fmt;

/// The channel set shown by the picker's numeric inputs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ColorPickerMode {
    /// Red / green / blue, each `0..=255`. The default.
    #[default]
    Rgb = 0,
    /// Hue (`0..=360`), saturation and lightness (`0..=1`).
    Hsl = 1,
    /// A single hexadecimal string.
    Hex = 2,
}

impl ColorPickerMode {
    /// Every variant, in discriminant order.
    pub const ALL: [Self; 3] = [Self::Rgb, Self::Hsl, Self::Hex];

    /// Lower-case name, matching the variant.
    #[inline]
    pub fn name(self) -> &'static str {
        match self {
            Self::Rgb => "rgb",
            Self::Hsl => "hsl",
            Self::Hex => "hex",
        }
    }

    /// The next mode in the cycle `RGB → HSL → HEX → RGB`. Mirrors the
    /// picker's mode-switch button.
    #[inline]
    pub fn next(self) -> Self {
        match self {
            Self::Rgb => Self::Hsl,
            Self::Hsl => Self::Hex,
            Self::Hex => Self::Rgb,
        }
    }
}

impl fmt::Display for ColorPickerMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_rgb() {
        assert_eq!(ColorPickerMode::default(), ColorPickerMode::Rgb);
    }

    #[test]
    fn discriminants_match_mudblazor() {
        assert_eq!(ColorPickerMode::Rgb as i32, 0);
        assert_eq!(ColorPickerMode::Hsl as i32, 1);
        assert_eq!(ColorPickerMode::Hex as i32, 2);
    }

    #[test]
    fn next_cycles_through_all_three() {
        let mut m = ColorPickerMode::Rgb;
        for _ in 0..3 {
            m = m.next();
        }
        assert_eq!(m, ColorPickerMode::Rgb);
        assert_eq!(ColorPickerMode::Rgb.next(), ColorPickerMode::Hsl);
        assert_eq!(ColorPickerMode::Hsl.next(), ColorPickerMode::Hex);
        assert_eq!(ColorPickerMode::Hex.next(), ColorPickerMode::Rgb);
    }

    #[test]
    fn all_has_three_unique_names() {
        let mut names: Vec<&str> = ColorPickerMode::ALL.iter().map(|v| v.name()).collect();
        names.sort_unstable();
        let mut deduped = names.clone();
        deduped.dedup();
        assert_eq!(names, deduped);
    }

    #[test]
    fn display_matches_name() {
        for v in ColorPickerMode::ALL {
            assert_eq!(v.to_string(), v.name());
        }
    }
}