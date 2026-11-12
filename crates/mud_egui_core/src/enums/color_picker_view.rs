//! `ColorPickerView` — which selection surface a colour picker shows.
//!
//! Port of `MudBlazor.ColorPickerView` (MudBlazor 8.13.0). Discriminants match
//! the C# enum exactly[reference:13].
//!
//! Kept in its own file per Rule 2.

use std::fmt;

/// The selection surface currently on screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ColorPickerView {
    /// A 2D saturation/lightness field with a hue slider. The default.
    #[default]
    Spectrum = 0,
    /// A pre-defined list of colours (`MudColorPicker::palette`).
    Palette = 1,
    /// A compact grid of colours (`MudColorPicker::palette`).
    Grid = 2,
}

impl ColorPickerView {
    /// Every variant, in discriminant order.
    pub const ALL: [Self; 3] = [Self::Spectrum, Self::Palette, Self::Grid];

    /// Lower-case name, matching the variant.
    #[inline]
    pub fn name(self) -> &'static str {
        match self {
            Self::Spectrum => "spectrum",
            Self::Palette => "palette",
            Self::Grid => "grid",
        }
    }

    /// `true` for `Spectrum`.
    #[inline]
    pub fn is_spectrum(self) -> bool {
        matches!(self, Self::Spectrum)
    }

    /// `true` for `Palette` or `Grid` — both render from `palette`.
    #[inline]
    pub fn is_swatch_based(self) -> bool {
        !self.is_spectrum()
    }
}

impl fmt::Display for ColorPickerView {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_spectrum() {
        assert_eq!(ColorPickerView::default(), ColorPickerView::Spectrum);
    }

    #[test]
    fn discriminants_match_mudblazor() {
        assert_eq!(ColorPickerView::Spectrum as i32, 0);
        assert_eq!(ColorPickerView::Palette as i32, 1);
        assert_eq!(ColorPickerView::Grid as i32, 2);
    }

    #[test]
    fn all_has_three_unique_variants() {
        assert_eq!(ColorPickerView::ALL.len(), 3);
        let mut names: Vec<&str> = ColorPickerView::ALL.iter().map(|v| v.name()).collect();
        names.sort_unstable();
        let mut deduped = names.clone();
        deduped.dedup();
        assert_eq!(names, deduped);
    }

    #[test]
    fn spectrum_predicate_is_exclusive() {
        assert!(ColorPickerView::Spectrum.is_spectrum());
        assert!(!ColorPickerView::Palette.is_spectrum());
        assert!(!ColorPickerView::Grid.is_spectrum());
        assert!(ColorPickerView::Palette.is_swatch_based());
        assert!(ColorPickerView::Grid.is_swatch_based());
        assert!(!ColorPickerView::Spectrum.is_swatch_based());
    }

    #[test]
    fn display_matches_name() {
        for v in ColorPickerView::ALL {
            assert_eq!(v.to_string(), v.name());
        }
    }
}