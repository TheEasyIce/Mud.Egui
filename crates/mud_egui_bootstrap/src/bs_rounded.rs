//! Bootstrap 5.3-style border-radius utilities (`rounded-*`) for `egui`.
//!
//! This module provides an efficient parser to translate standard Bootstrap `rounded` classes
//! (like `rounded`, `rounded-top-3`, `rounded-circle`, etc.) into `egui::CornerRadius`.
//!
//! # Specificity
//! To match standard CSS behavior, we must respect Bootstrap's specificity order.
//! In CSS, declaring `rounded-top` after `rounded-start` behaves based on the CSS file order,
//! which in Bootstrap is: `rounded` < `top` < `end` < `bottom` < `start`.
//! Our cache evaluates each class in a single pass, tracking the highest specificity
//! applied to each of the four corners independently.

use egui::CornerRadius;
use mud_egui_core::base::component_base::ComponentBase;

/// Base multiplier for CSS `rem` values. 1 rem is treated as 16 physical pixels.
const REM: f32 = 16.0;

/// Mock value for `rounded-pill` and `rounded-circle`.
/// Bootstrap uses `50rem` or `50%` for pills to ensure maximum roundness.
const PILL_REM: f32 = 50.0;

/// A cache for computed Bootstrap border-radius values.
///
/// Because parsing strings on every frame is a massive performance hit in an
/// immediate-mode GUI, we cache the result based on the component's class version.
#[derive(Default)]
pub struct BsRoundedCache {
    /// The version of the class list when this cache was last updated.
    version: u64,
    /// The cached corner radius configuration, or `None` if no rounding classes were found.
    config: Option<CornerRadius>,
}

impl BsRoundedCache {
    /// Resolves the border-radius based on the current classes attached to the `ComponentBase`.
    ///
    /// # Arguments
    /// * `base` - The component providing the `class_names` and `classes_version`.
    pub fn resolve(&mut self, base: &ComponentBase) -> Option<CornerRadius> {
        // Cache hit: avoid re-parsing if the class list is unchanged.
        if self.version == base.classes_version() {
            return self.config;
        }

        // Standard Bootstrap 5 rounded scale multipliers.
        // Index maps to the class step (e.g., `rounded-0` -> 0.0, `rounded-3` -> 0.5rem).
        let scale = [0.0, 0.25, 0.375, 0.5, 1.0, 2.0];

        // Arrays to track the state of the 4 corners.
        // Mapping index to corner:
        // 0: TopLeft (NW), 1: TopRight (NE), 2: BottomRight (SE), 3: BottomLeft (SW)
        let mut radii = [0.0; 4];

        // `tiers` tracks the highest CSS specificity tier that has mutated each corner.
        // Starts at -1 (unmodified). If a higher tier class targets a corner, it overwrites.
        let mut tiers = [-1; 4];
        let mut matched = false;

        // Single pass over all classes. We rely on the `tier` returned by `parse_rounded`
        // to handle overwriting, rather than sorting the classes.
        for cls in base.class_names() {
            if let Some((corners_bitmask, tier, step)) = parse_rounded(cls, scale) {
                matched = true;

                // Calculate the final pixel value.
                // A step of `-1` is our internal sentinel for `pill` or `circle`.
                let val = if step == -1 { PILL_REM * REM } else { scale[step as usize] * REM };

                // Apply the radius to targeted corners, but ONLY if the specificity tier
                // of this class is greater than or equal to the tier that previously set it.
                // Note: bitmask & 1 == TopLeft, bitmask & 2 == TopRight, etc.
                if corners_bitmask & 1 != 0 && tier >= tiers[0] { radii[0] = val; tiers[0] = tier; }
                if corners_bitmask & 2 != 0 && tier >= tiers[1] { radii[1] = val; tiers[1] = tier; }
                if corners_bitmask & 4 != 0 && tier >= tiers[2] { radii[2] = val; tiers[2] = tier; }
                if corners_bitmask & 8 != 0 && tier >= tiers[3] { radii[3] = val; tiers[3] = tier; }
            }
        }

        // Map our computed arrays into `egui::CornerRadius`.
        // `egui` uses compass directions (nw, ne, se, sw).
        let result = if matched {
            // Note: egui's CornerRadius fields are often `u8`.
            // If your egui version uses `f32` natively, remove the `as u8` casts.
            Some(CornerRadius {
                nw: radii[0] as u8, // Top Left
                ne: radii[1] as u8, // Top Right
                se: radii[2] as u8, // Bottom Right
                sw: radii[3] as u8, // Bottom Left
            })
        } else {
            None
        };

        // Update the cache with the new version and calculated result.
        self.version = base.classes_version();
        self.config = result;

        result
    }
}

/// Parses a rounded class and returns its bitmask, specificity tier, and size step.
///
/// # Returns
/// An `Option` containing a tuple of:
/// 1. `u8`: A bitmask representing affected corners (1=TL, 2=TR, 4=BR, 8=BL).
/// 2. `usize`: The specificity tier (0 = global, 4 = most specific).
/// 3. `i32`: The scale index (0-5), or `-1` for pill/circle.
fn parse_rounded(cls: &str, scale: [f32; 6]) -> Option<(u8, i32, i32)> {
    if !cls.starts_with("rounded") { return None; }

    // Strip "rounded" prefix
    let rest = &cls[7..];

    // Handle bare "rounded" class (Defaults: all corners (15), tier 0, step 2 (0.375rem))
    if rest.is_empty() { return Some((15, 0, 2)); }

    // Malformed class safety check
    if !rest.starts_with('-') { return None; }
    let rest = &rest[1..];

    // Check if it's purely a global size modifier like "rounded-3", "rounded-pill"
    // (Still applies to all corners (15), tier 0 specificity)
    if let Some(step) = parse_size(rest, scale) {
        return Some((15, 0, step));
    }

    // If we reach here, it has a side modifier: "rounded-top-3", "rounded-end-pill"
    // Split into the side ("top") and an optional step ("3")
    let (side, step_str) = match rest.find('-') {
        Some(idx) => (&rest[..idx], Some(&rest[idx + 1..])),
        None => (rest, None),
    };

    // Parse the size if provided, otherwise default to step 2 (Bootstrap default)
    let step = if let Some(s) = step_str {
        parse_size(s, scale)?
    } else {
        2 // Default step (0.375rem)
    };

    // Generate the bitmask and specificity tier based on the side targeted.
    // Bitmask mapping: 1 = TL, 2 = TR, 4 = BR, 8 = BL
    // Tiers match Bootstrap's CSS declaration order (lower tier gets overridden by higher tier).
    let (corners, tier) = match side {
        "top"    => ((1 | 2), 1), // Top Left + Top Right
        "end"    => ((2 | 4), 2), // Top Right + Bottom Right (LTR context)
        "bottom" => ((4 | 8), 3), // Bottom Right + Bottom Left
        "start"  => ((1 | 8), 4), // Top Left + Bottom Left (LTR context)
        _ => return None,
    };

    Some((corners, tier, step))
}

/// Parses the size portion of a class into a step index.
///
/// # Returns
/// * `Some(-1)` if the size is "pill" or "circle".
/// * `Some(0..5)` for standard numeric scales.
/// * `None` if invalid or out of bounds.
fn parse_size(s: &str, scale: [f32; 6]) -> Option<i32> {
    // Special string identifiers for heavily rounded elements.
    // We use -1 as a sentinel value since indices must be positive.
    if s == "pill" || s == "circle" { return Some(-1); }

    let step = s.parse::<usize>().ok()?;

    // Ensure the step doesn't exceed our standard array lengths
    if step >= scale.len() { return None; }

    Some(step as i32)
}