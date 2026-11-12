//! Tailwind-style `bg-*` utilities for `egui`.
//!
//! This module provides a robust parser for modern Tailwind CSS background classes,
//! translating them into `egui::Color32` or theme resource keys.
//!
//! # Supported Syntax
//! - **Arbitrary colors:** `bg-[#f00]`, `bg-[#ff000080]`
//! - **Standard tokens:** `bg-primary`, `bg-secondary-dark`
//! - **Opacity modifiers:** `bg-red/50`, `bg-primary/[0.33]`
//! - **Arbitrary Theme Variables:** `bg-(--MudPaletteInfo)`
//! - **Interactive/State Variants:** `hover:bg-primary`, `dark:bg-white`, `hover:dark:bg-black`
//!
//! # Specificity
//! When multiple classes target the background, the one with the *most* matching variants wins.
//! For example, if a component has `bg-white dark:bg-black` and the theme is currently dark,
//! `dark:bg-black` wins because it has 1 variant, while `bg-white` has 0.

use egui::Color32;
use mud_egui_core::base::component_base::ComponentBase;

/// The parsed result of a `bg-*` class.
#[derive(Debug, Clone, PartialEq)]
pub enum TwBg {
    /// A hardcoded, literal `egui` color.
    /// Generated from arbitrary hex values (`bg-[#f00]`) or standard aliases (`bg-white`).
    Literal(Color32),

    /// A dynamic theme resource key.
    /// The actual color must be resolved later against the active `MudPalette`.
    Resource {
        /// The CamelCase key mapping to the palette (e.g., "MudPalettePrimary").
        key: String,
        /// An opacity modifier ranging from 0.0 to 1.0 (e.g., `0.5` for `bg-primary/50`).
        alpha: f32
    },
}

/// A cache for the computed background class.
///
/// Because Tailwind allows dynamic state variants (like `hover:`), the background must be
/// re-evaluated not just when the class string changes, but also when the *interaction state*
/// (mouse hover, click) or *theme state* (dark mode) changes.
#[derive(Default)]
pub struct TwBgCache {
    /// The version of the class list when this cache was last updated.
    version: u64,
    /// The hover state used during the last evaluation.
    hovered: bool,
    /// The active (clicked/focused) state used during the last evaluation.
    active: bool,
    /// The theme mode used during the last evaluation.
    dark: bool,
    /// The cached background configuration.
    config: Option<TwBg>,
}

impl TwBgCache {
    /// Resolves the Tailwind background class.
    ///
    /// Note: You MUST pass the current interaction and theme state so the cache can
    /// correctly filter and rank `hover:`, `active:`, and `dark:` variants.
    ///
    /// # Arguments
    /// * `base` - The component providing the class list.
    /// * `is_hovered` - `true` if the mouse is currently over the component.
    /// * `is_active` - `true` if the component is currently being clicked/pressed.
    /// * `is_dark` - `true` if the application is currently in dark mode.
    pub fn resolve(
        &mut self,
        base: &ComponentBase,
        is_hovered: bool,
        is_active: bool,
        is_dark: bool
    ) -> Option<TwBg> {
        // Cache hit: avoid re-parsing if the class list AND all relevant UI states are unchanged.
        if self.version == base.classes_version()
            && self.hovered == is_hovered
            && self.active == is_active
            && self.dark == is_dark
        {
            return self.config.clone();
        }

        let mut best_config = None;
        // We use variant count as our specificity score.
        // Starts at -1 so a standard class (0 variants) will overwrite it.
        let mut best_variant_count = -1;

        for cls in base.class_names() {
            if let Some((parsed_bg, variant_count, applies)) = try_parse(cls, is_hovered, is_active, is_dark) {
                // Skip this class if its variants don't match the current UI state
                // (e.g., skip `hover:bg-red` if `is_hovered` is false).
                if !applies { continue; }

                // Tailwind Specificity Rule:
                // More variants = more specific (`dark:hover:bg-red` beats `hover:bg-red`).
                // Ties go to the last class written in the string.
                if variant_count >= best_variant_count {
                    best_config = Some(parsed_bg);
                    best_variant_count = variant_count;
                }
            }
        }

        // Update the cache for future frames.
        self.version = base.classes_version();
        self.hovered = is_hovered;
        self.active = is_active;
        self.dark = is_dark;
        self.config = best_config.clone();

        best_config
    }
}

/// Parses a single class string into a `TwBg` struct, calculating its specificity.
///
/// # Returns
/// An `Option` containing a tuple of:
/// 1. `TwBg`: The parsed background configuration.
/// 2. `i32`: The variant count (specificity score).
/// 3. `bool`: True if the class applies to the *current* UI state (hover, dark, etc).
fn try_parse(cls: &str, hovered: bool, active: bool, dark: bool) -> Option<(TwBg, i32, bool)> {
    let mut applies = true;
    let mut variant_count = 0;
    let mut payload = cls;

    // 1. Handle Variants (e.g., "hover:", "dark:", "focus:")
    // We peel these off one by one from the left side of the string.
    while let Some(colon_idx) = payload.find(':') {
        let variant = &payload[..colon_idx];
        payload = &payload[colon_idx + 1..];
        variant_count += 1;

        // Bitwise AND the conditions. If *any* variant fails, `applies` becomes false.
        match variant {
            "hover" => applies &= hovered,
            "active" => applies &= active,
            // egui doesn't have a strict CSS-like "focus" concept for most widgets,
            // so we map it to `active` as a reasonable proxy.
            "focus" => applies &= active,
            "disabled" => applies &= false, // TODO: Map this to actual disabled state if tracked
            "dark" => applies &= dark,
            "light" => applies &= !dark,
            _ => return None, // Unknown variant, reject the whole class
        }
    }

    // 2. Strip "bg-" prefix. If it's missing, this isn't a background class.
    let mut payload = payload.strip_prefix("bg-")?;
    if payload.is_empty() { return None; }

    // 3. Handle Opacity Modifier (e.g., "bg-primary/50" or "bg-primary/[0.33]")
    let mut alpha = 1.0;

    // Look for the LAST slash to support division syntax.
    if let Some(slash_idx) = payload.rfind('/') {
        let (base_part, opacity_part) = payload.split_at(slash_idx);
        let opacity_str = &opacity_part[1..];

        // Update payload to just the color part
        payload = base_part;

        // Parse the opacity.
        // If wrapped in brackets `/[0.33]`, parse as raw float.
        // Otherwise `/50`, parse as percentage and divide by 100.
        if let Some(stripped) = opacity_str.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
            alpha = stripped.parse::<f32>().ok()?;
        } else {
            alpha = opacity_str.parse::<f32>().ok()? / 100.0;
        }
    }

    // 4. Handle Literals & Arbitrary Hex Values
    let bg = match payload {
        "transparent" => TwBg::Literal(Color32::TRANSPARENT),
        "white" => TwBg::Literal(Color32::WHITE),
        "black" => TwBg::Literal(Color32::BLACK),

        // Map "bg-body" to the standard background token.
        "body" => TwBg::Resource { key: "MudPaletteBackground".to_owned(), alpha },

        // Handle arbitrary hex syntax: `bg-[#f00]`
        _ if payload.starts_with('[') && payload.ends_with(']') => {
            let hex = &payload[1..payload.len()-1];
            let color = parse_hex(hex)?;

            // Apply the opacity modifier (if any) to the hex's native alpha channel.
            let final_color = Color32::from_rgba_unmultiplied(
                color.r(), color.g(), color.b(), (color.a() as f32 * alpha) as u8
            );
            TwBg::Literal(final_color)
        }

        // 5. Handle Arbitrary Theme Variables: `bg-(--VarName)`
        _ if payload.starts_with("(--") && payload.ends_with(')') => {
            // Extract the variable name without the `(--` and `)`
            TwBg::Resource { key: payload[3..payload.len()-1].to_owned(), alpha }
        }

        // 6. Normal Palette Tokens
        // e.g., "primary-dark" becomes "MudPalettePrimaryDarken"
        _ => {
            let key = build_key(payload)?;
            TwBg::Resource { key, alpha }
        }
    };

    Some((bg, variant_count, applies))
}

/// Translates a Tailwind semantic token into a MudBlazor palette key.
/// Example: `primary-foreground` -> `MudPalettePrimaryText`
fn build_key(token: &str) -> Option<String> {
    // Check if the token ends in a known modifier suffix (e.g., "-foreground").
    let (base, suffix) = if let Some(idx) = token.rfind('-') {
        let (b, s) = token.split_at(idx);
        let replacement = match &s[1..] {
            "foreground" => "Text",
            "dark" => "Darken",
            "light" => "Lighten",
            _ => "",
        };
        (b, replacement)
    } else {
        (token, "")
    };

    // Construct the CamelCase palette key.
    let mut key = String::with_capacity("MudPalette".len() + base.len() + suffix.len());
    key.push_str("MudPalette");

    // Capitalize the first letter of each dash-separated word.
    let mut upper = true;
    for c in base.chars() {
        if c == '-' {
            upper = true;
            continue;
        }
        if !c.is_ascii_alphanumeric() { return None; } // Reject invalid chars

        key.push(if upper { c.to_ascii_uppercase() } else { c });
        upper = false;
    }

    key.push_str(suffix);
    Some(key)
}

/// Parses a standard CSS Hex string into an `egui::Color32`.
/// Supports `#rgb`, `#rgba`, `#rrggbb`, and `#rrggbbaa`.
fn parse_hex(hex: &str) -> Option<Color32> {
    let hex = hex.strip_prefix('#')?;
    let r; let g; let b; let a;

    // Use radix 16 to parse hex chunks.
    // For 3 and 4 character shorthand (e.g. #F00), we multiply by 17 (0x11)
    // because `F` expands to `FF` (15 * 17 = 255).
    match hex.len() {
        3 => {
            r = u8::from_str_radix(&hex[0..1], 16).ok()? * 17;
            g = u8::from_str_radix(&hex[1..2], 16).ok()? * 17;
            b = u8::from_str_radix(&hex[2..3], 16).ok()? * 17;
            a = 255;
        }
        4 => {
            r = u8::from_str_radix(&hex[0..1], 16).ok()? * 17;
            g = u8::from_str_radix(&hex[1..2], 16).ok()? * 17;
            b = u8::from_str_radix(&hex[2..3], 16).ok()? * 17;
            a = u8::from_str_radix(&hex[3..4], 16).ok()? * 17;
        }
        6 => {
            r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            a = 255;
        }
        8 => {
            r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            a = u8::from_str_radix(&hex[6..8], 16).ok()?;
        }
        _ => return None,
    }

    Some(Color32::from_rgba_unmultiplied(r, g, b, a))
}