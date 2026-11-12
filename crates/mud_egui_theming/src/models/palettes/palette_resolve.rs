//! Translation layer for resolving semantic CSS classes into concrete colors.
//!
//! # Architecture
//! The utility parsing crates (like `mud_egui_tailwind` and `mud_egui_bootstrap`) are
//! strictly decoupled from the theming engine. They do not know what "Primary" or "Surface"
//! look like. Instead, when they parse a class like `bg-primary`, they generate a semantic
//! string key such as `MudPalettePrimary`.
//!
//! This module extends the `Palette` struct with a `resolve` method. It is the sole
//! location where those string keys are mapped into actual `MudColor` and `egui::Color32` values
//! based on the currently active Light or Dark palette.

use egui::Color32;

use super::palette::Palette;
use crate::utilities::mud_color::MudColor;

impl Palette {
    /// Resolves a semantic string key produced by the `bs_*` or `tw_*` parsers
    /// into a concrete `MudColor`.
    ///
    /// # Supported Naming Conventions
    /// All keys are expected to begin with the `MudPalette` prefix.
    /// - **Direct Fields:** `Primary`, `Surface`, `AppbarBackground`, `LinesDefault`.
    /// - **Lightness Variants:** `<Field>Darken` or `<Field>Lighten` (Applies a 7.5% HSL lightness shift).
    /// - **Opacity Variants:** `<Field>HoverBrush` (Retrieves the field color, but overrides its alpha
    ///   channel with the palette's `hover_opacity`).
    /// - **Contrast Variants:** `PrimaryText`, `SecondaryText`, `TertiaryText` (Maps to the specific
    ///   `contrast_text` field designed to be legible on top of that background).
    ///
    /// # Returns
    /// The requested `MudColor`, or `None` if the key is completely unrecognized. Returning `None`
    /// allows the upstream caller to gracefully fall back to a default (usually transparent).
    pub fn get(&self, key: &str) -> Option<MudColor> {
        // All valid semantic keys must start with this prefix.
        let name = key.strip_prefix("MudPalette")?;

        // 1. Exact field names win.
        // We do this first because some fields natively end in "Text"
        // (e.g., `AppbarText`), and we don't want them intercepted by the variant parser below.
        if let Some(c) = self.field(name) {
            return Some(c);
        }

        // 2. Computed Variants.
        // If it wasn't an exact field match, we check for known suffix modifiers.

        // HoverBrush: Apply the global hover opacity to the base color.
        if let Some(base) = name.strip_suffix("HoverBrush") {
            return Some(self.field(base)?.with_alpha(self.hover_opacity));
        }

        // Darken: Decrease lightness by 7.5% in HSL space.
        if let Some(base) = name.strip_suffix("Darken") {
            return Some(self.field(base)?.darken(0.075));
        }

        // Lighten: Increase lightness by 7.5% in HSL space.
        if let Some(base) = name.strip_suffix("Lighten") {
            return Some(self.field(base)?.lighten(0.075));
        }

        // Contrast Text: Map semantic names directly to their specific contrast fields.
        // We only map Primary, Secondary, and Tertiary here. Things like `DrawerText`
        // were already caught by the exact field match above.
        if let Some(base) = name.strip_suffix("Text") {
            return match base {
                "Primary" => Some(self.primary_contrast_text),
                "Secondary" => Some(self.secondary_contrast_text),
                "Tertiary" => Some(self.tertiary_contrast_text),
                _ => None,
            };
        }

        // If the key made it this far, it doesn't match any known pattern.
        None
    }

    /// Resolves a semantic string key directly into an `egui::Color32`.
    ///
    /// This is a convenience wrapper around [`Palette::get`] that handles the
    /// type conversion, as `egui` components require `Color32` for rendering.
    pub fn resolve(&self, key: &str) -> Option<Color32> {
        self.get(key).map(Into::into)
    }

    /// Internal helper that maps a CamelCase string to its corresponding struct field.
    ///
    /// This acts as a manual reflection/lookup table since Rust does not have runtime
    /// reflection for struct fields.
    fn field(&self, name: &str) -> Option<MudColor> {
        Some(match name {
            // Brand Colors
            "Primary" => self.primary,
            "PrimaryContrastText" => self.primary_contrast_text,
            "Secondary" => self.secondary,
            "SecondaryContrastText" => self.secondary_contrast_text,
            "Tertiary" => self.tertiary,
            "TertiaryContrastText" => self.tertiary_contrast_text,

            // State Colors
            "Info" => self.info,
            "Success" => self.success,
            "Warning" => self.warning,
            "Error" => self.error,

            // Text Colors
            "TextPrimary" => self.text_primary,
            "TextSecondary" => self.text_secondary,
            "TextDisabled" => self.text_disabled,

            // Action Colors
            "ActionDefault" => self.action_default,
            "ActionDisabled" => self.action_disabled,
            "ActionDisabledBackground" => self.action_disabled_background,

            // Surface Colors
            "Background" => self.background,
            "BackgroundGray" => self.background_gray,
            "Surface" => self.surface,

            // Component Backgrounds
            "AppbarBackground" => self.appbar_background,
            "AppbarText" => self.appbar_text,
            "DrawerBackground" => self.drawer_background,
            "DrawerText" => self.drawer_text,
            "DrawerIcon" => self.drawer_icon,

            // Lines and Borders
            "LinesDefault" => self.lines_default,
            "LinesInputs" => self.lines_inputs,
            "TableLines" => self.table_lines,
            "TableStriped" => self.table_striped,
            "TableHover" => self.table_hover,
            "Divider" => self.divider,
            "DividerLight" => self.divider_light,

            // Misc
            "Skeleton" => self.skeleton,

            // Unrecognized field
            _ => return None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_fields_and_variants() {
        let p = Palette::new_light();

        // Exact Field Match
        assert_eq!(p.get("MudPalettePrimary"), Some(p.primary));

        // Suffix Match -> Contrast Text
        assert_eq!(p.get("MudPalettePrimaryText"), Some(p.primary_contrast_text));

        // Exact Field Match (Ensuring it isn't parsed as "Appbar" + "Text")
        assert_eq!(p.get("MudPaletteAppbarText"), Some(p.appbar_text));

        // Suffix Match -> Darken Math
        assert_eq!(p.get("MudPalettePrimaryDarken"), Some(p.primary_darken()));

        // Suffix Match -> Opacity override
        assert_eq!(
            p.get("MudPalettePrimaryHoverBrush"),
            Some(p.primary.with_alpha(p.hover_opacity))
        );

        // Exact Field Match (Lines)
        assert_eq!(p.get("MudPaletteLinesDefault"), Some(p.lines_default));

        // Bad prefix/Bad field tests
        assert_eq!(p.get("MudPaletteNope"), None);
        assert_eq!(p.get("NotAKey"), None);
    }
}