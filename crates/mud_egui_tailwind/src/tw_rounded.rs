//! Tailwind-style border-radius utilities (`rounded-*`) for `egui`.
//!
//! This module parses modern Tailwind CSS `rounded` classes and translates
//! them into an `egui::CornerRadius`.
//!
//! # Supported Syntax
//! - **Base**: `rounded` (applies a default 0.25rem radius).
//! - **Global Sizes**: `rounded-none`, `rounded-sm`, `rounded-lg`, `rounded-full`.
//! - **Specific Sides**: `rounded-t-lg` (Top), `rounded-b-none` (Bottom), `rounded-l-md` (Left).
//! - **Specific Corners**: `rounded-tl-xl` (Top Left), `rounded-br` (Bottom Right).
//! - **Arbitrary Sizes**: `rounded-[12px]`, `rounded-t-[2rem]`.
//! - **Logical Properties**: `rounded-s-lg` (Start), `rounded-se-full` (Start-End).
//!
//! # Specificity
//! Tailwind applies border-radius classes using exact specificity rules.
//! Global sizes (`rounded-lg`) are overridden by specific sides (`rounded-t-lg`),
//! which are in turn overridden by specific corners (`rounded-tr-lg`).

use egui::CornerRadius;
use mud_egui_core::base::component_base::ComponentBase;

/// A cache for the computed Tailwind border-radius configuration.
///
/// Parsing string classes on every frame is a major performance hit in immediate-mode
/// GUIs like `egui`. `TwRoundedCache` tracks the class list's version and only
/// re-evaluates the classes when the component's state actually mutates.
#[derive(Default)]
pub struct TwRoundedCache {
    /// The version of the class list when this cache was last updated.
    version: u64,
    /// The cached corner radius configuration, or `None` if no rounding classes were applied.
    config: Option<CornerRadius>,
}

impl TwRoundedCache {
    /// Resolves the border-radius based on the current classes.
    ///
    /// # Arguments
    /// * `base` - The component providing the `class_names` and `classes_version`.
    /// * `rem` - The current scale factor for converting `rem` to pixels (usually 16.0).
    pub fn resolve(&mut self, base: &ComponentBase, rem: f32) -> Option<CornerRadius> {
        // Cache hit: avoid re-parsing if the class list is unchanged.
        if self.version == base.classes_version() {
            return self.config;
        }

        // Arrays to track the state of the 4 corners.
        // Mapping index to corner:
        // 0: TopLeft (NW), 1: TopRight (NE), 2: BottomRight (SE), 3: BottomLeft (SW)
        let mut radii = [0.0; 4];

        // `tiers` tracks the highest CSS specificity tier that has mutated each corner.
        // Starts at -1 (unmodified). If a higher tier class targets a corner, it overwrites.
        let mut tiers: [i32; 4] = [-1; 4];
        let mut matched = false;

        // Single pass over all classes. We rely on the `tier` returned by `parse_rounded`
        // to handle specificity overwriting, ensuring order in the class string doesn't
        // break CSS rules.
        for cls in base.class_names() {
            if let Some((corners_bitmask, tier, val)) = parse_rounded(cls, rem) {
                matched = true;

                // Apply the radius to targeted corners, but ONLY if the specificity tier
                // of this class is greater than or equal to the tier that previously set it.
                // Bitmask check: 1=TL, 2=TR, 4=BR, 8=BL
                if corners_bitmask & 1 != 0 && tier >= tiers[0] { radii[0] = val; tiers[0] = tier; }
                if corners_bitmask & 2 != 0 && tier >= tiers[1] { radii[1] = val; tiers[1] = tier; }
                if corners_bitmask & 4 != 0 && tier >= tiers[2] { radii[2] = val; tiers[2] = tier; }
                if corners_bitmask & 8 != 0 && tier >= tiers[3] { radii[3] = val; tiers[3] = tier; }
            }
        }

        // Map our computed arrays into `egui::CornerRadius`.
        // Note: egui's CornerRadius fields are often `u8`.
        let result = if matched {
            Some(CornerRadius {
                nw: radii[0] as u8, // Top Left
                ne: radii[1] as u8, // Top Right
                se: radii[2] as u8, // Bottom Right
                sw: radii[3] as u8, // Bottom Left
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

/// Parses a rounded class and returns its bitmask, specificity tier, and radius in pixels.
///
/// # Returns
/// An `Option` containing a tuple of:
/// 1. `u8`: A bitmask representing affected corners (1=TL, 2=TR, 4=BR, 8=BL).
/// 2. `i32`: The specificity tier (0 = global, 14 = highly specific corner).
/// 3. `f32`: The computed border radius in pixels.
fn parse_rounded(cls: &str, rem: f32) -> Option<(u8, i32, f32)> {
    if !cls.starts_with("rounded") { return None; }

    // Strip "rounded" prefix
    let rest = &cls[7..];

    // Handle bare "rounded" class (Defaults: all corners (15), tier 0, 0.25rem size)
    if rest.is_empty() { return Some((15, 0, 0.25 * rem)); }

    // Malformed class safety check
    if !rest.starts_with('-') { return None; }
    let rest = &rest[1..];

    // Trailing dash safety check
    if rest.is_empty() { return None; }

    // Check if it's purely a global size modifier like "rounded-lg", "rounded-full"
    // (Applies to all corners (15), tier 0 specificity)
    if let Some(val) = parse_size(rest, rem) {
        return Some((15, 0, val));
    }

    // If we reach here, it has a side modifier: "rounded-t", "rounded-tl-lg", "rounded-b-[10px]"
    // Split into the side ("tl") and an optional size suffix ("lg")
    let (side, size_str) = match rest.find('-') {
        Some(idx) => (&rest[..idx], Some(&rest[idx + 1..])),
        None => (rest, None),
    };

    // Parse the size if provided, otherwise fallback to the default 0.25rem size
    let val = if let Some(s) = size_str {
        parse_size(s, rem)?
    } else {
        0.25 * rem
    };

    // Generate the bitmask and specificity tier based on the side targeted.
    // Bitmask mapping: 1 = TL, 2 = TR, 4 = BR, 8 = BL
    // Tiers enforce that specific corners (tier 11-14) overwrite specific sides (tier 3-6).
    let (corners, tier) = match side {
        // Logical sides (Assuming LTR mapping)
        "s"  => (1 | 8, 1),   // Start (Top Left + Bottom Left)
        "e"  => (2 | 4, 2),   // End (Top Right + Bottom Right)

        // Physical sides
        "t"  => (1 | 2, 3),   // Top (Top Left + Top Right)
        "r"  => (2 | 4, 4),   // Right (Top Right + Bottom Right)
        "b"  => (4 | 8, 5),   // Bottom (Bottom Right + Bottom Left)
        "l"  => (1 | 8, 6),   // Left (Top Left + Bottom Left)

        // Logical corners
        "ss" => (1, 7),       // Start-Start (Top Left)
        "se" => (2, 8),       // Start-End (Top Right)
        "ee" => (4, 9),       // End-End (Bottom Right)
        "es" => (8, 10),      // End-Start (Bottom Left)

        // Physical corners
        "tl" => (1, 11),      // Top Left
        "tr" => (2, 12),      // Top Right
        "br" => (4, 13),      // Bottom Right
        "bl" => (8, 14),      // Bottom Left

        _ => return None,
    };

    Some((corners, tier, val))
}

/// Parses the size portion of a Tailwind border-radius class into physical pixels.
///
/// Handles standard scale string identifiers (`sm`, `md`, `full`) and arbitrary
/// bracket syntax (`[12px]`, `[0.5rem]`).
fn parse_size(s: &str, rem: f32) -> Option<f32> {
    match s {
        "none" => Some(0.0),
        "sm" => Some(0.125 * rem),
        "md" => Some(0.375 * rem),
        "lg" => Some(0.5 * rem),
        "xl" => Some(0.75 * rem),
        "2xl" => Some(1.0 * rem),
        "3xl" => Some(1.5 * rem),
        "4xl" => Some(2.0 * rem),
        "full" => Some(9999.0), // Massive number to ensure perfect pills/circles
        _ => {
            // Handle arbitrary value syntax: `[...]`
            if let Some(inner) = s.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
                if let Some(r) = inner.strip_suffix("rem") {
                    Some(r.parse::<f32>().ok()? * rem)
                } else if let Some(p) = inner.strip_suffix("px") {
                    Some(p.parse::<f32>().ok()?)
                } else {
                    // Unitless arbitrary value
                    inner.parse::<f32>().ok()
                }
            } else {
                None
            }
        }
    }
}