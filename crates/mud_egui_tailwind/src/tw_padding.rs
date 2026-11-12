//! Tailwind-style padding utilities (`p-*`, `px-*`, `pt-*`, etc.) for `egui`.
//!
//! This module provides a robust parser for modern Tailwind CSS padding classes.
//! It functions similarly to the margin parser, but enforces Tailwind's padding rules:
//! padding cannot be negative, and it does not support `auto`.
//!
//! # Supported Syntax
//! - **Standard Scale**: `p-4`, `px-2`, `pt-8` (Calculated as `step * 0.25 * rem`).
//! - **Fixed Pixels**: `p-px` (Exactly 1 physical pixel).
//! - **Arbitrary Values**: `pt-[12px]`, `p-[0.5rem]`, `pb-[3.5]`.
//! - **Logical & Physical Sides**: `ps-*` / `pe-*` (Start/End) and `pl-*` / `pr-*` (Left/Right).

use egui::Margin;
use mud_egui_core::base::component_base::ComponentBase;

/// A cache for the computed Tailwind padding configuration.
///
/// Parsing string classes on every frame is a major performance hit in immediate-mode
/// GUIs like `egui`. `TwPaddingCache` tracks the class list's version and only
/// re-evaluates the classes when the component's state actually mutates.
#[derive(Default)]
pub struct TwPaddingCache {
    /// The version of the class list when this cache was last updated.
    version: u64,
    /// The cached padding margin, or `None` if no padding classes were applied.
    config: Option<Margin>,
}

impl TwPaddingCache {
    /// Resolves the padding margin based on the current classes.
    ///
    /// # Arguments
    /// * `base` - The component providing the `class_names` and `classes_version`.
    /// * `rem` - The current scale factor for converting `rem` to pixels (usually 16.0).
    pub fn resolve(&mut self, base: &ComponentBase, rem: f32) -> Option<Margin> {
        // Cache hit: avoid re-parsing if the class list is unchanged.
        if self.version == base.classes_version() {
            return self.config;
        }

        let mut left = 0.0;
        let mut right = 0.0;
        let mut top = 0.0;
        let mut bottom = 0.0;
        let mut matched = false;

        // Enforce CSS specificity without sorting the classes.
        // We use 4 passes (0 to 3) moving from general to highly specific.
        // This ensures that `pt-4` will overwrite the top edge of a preceding `p-2`.
        // 0: All sides (`p-*`)
        // 1: Axes (`px-*`, `py-*`)
        // 2: Logical sides (`ps-*`, `pe-*`) - Usually maps to LTR (Left/Right)
        // 3: Physical sides (`pt-*`, `pb-*`, `pl-*`, `pr-*`)
        for pass in 0..=3 {
            for cls in base.class_names() {
                if let Some(padding) = parse_padding(cls, pass, rem) {
                    matched = true;
                    // Apply values if this specific class/pass targeted these edges.
                    if let Some(v) = padding.left { left = v; }
                    if let Some(v) = padding.right { right = v; }
                    if let Some(v) = padding.top { top = v; }
                    if let Some(v) = padding.bottom { bottom = v; }
                }
            }
        }

        // Translate the computed f32 values into `egui::Margin`.
        // Note: egui's Margin uses i8 for compact storage.
        let result = if matched {
            Some(Margin {
                left: left as i8,
                right: right as i8,
                top: top as i8,
                bottom: bottom as i8
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

/// Intermediate representation of padding from a single CSS class.
///
/// Uses `Option<f32>` to explicitly track which sides are being modified,
/// ensuring unaffected sides remain `None` and do not overwrite prior specificity passes.
struct PartialPadding {
    left: Option<f32>,
    right: Option<f32>,
    top: Option<f32>,
    bottom: Option<f32>
}

/// Parses a single class string into a `PartialPadding` struct.
///
/// # Arguments
/// * `cls` - The class string to parse (e.g., "p-4", "px-[10px]").
/// * `current_pass` - The specificity tier currently being evaluated (0 through 3).
/// * `rem` - The pixel multiplier for rem values.
fn parse_padding(cls: &str, current_pass: usize, rem: f32) -> Option<PartialPadding> {
    // Fast path: if it doesn't start with 'p', it's not a padding class.
    if !cls.starts_with('p') { return None; }

    // Categorize the class into its target sides, specificity tier, and the payload string.
    let (sides, tier, rest) = if cls.starts_with("p-") { ("all", 0, &cls[2..]) }
    else if cls.starts_with("px-") { ("x", 1, &cls[3..]) }
    else if cls.starts_with("py-") { ("y", 1, &cls[3..]) }
    // Logical properties (Start/End)
    else if cls.starts_with("ps-") { ("s", 2, &cls[3..]) }
    else if cls.starts_with("pe-") { ("e", 2, &cls[3..]) }
    // Physical properties (Top/Right/Bottom/Left)
    else if cls.starts_with("pt-") { ("t", 3, &cls[3..]) }
    else if cls.starts_with("pr-") { ("r", 3, &cls[3..]) }
    else if cls.starts_with("pb-") { ("b", 3, &cls[3..]) }
    else if cls.starts_with("pl-") { ("l", 3, &cls[3..]) }
    else { return None; };

    // Ignore this class if it doesn't belong to the specificity pass we are currently on.
    if tier != current_pass { return None; }

    // Calculate the final numeric value.
    let val = if rest == "px" {
        // Tailwind's `p-px` represents exactly 1 physical pixel.
        1.0
    } else if let Some(inner) = rest.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
        // Handle arbitrary bracket syntax: `p-[...]`
        if let Some(r) = inner.strip_suffix("rem") {
            // e.g., `p-[2rem]`
            r.parse::<f32>().ok()? * rem
        } else if let Some(p) = inner.strip_suffix("px") {
            // e.g., `p-[12px]`
            p.parse::<f32>().ok()?
        } else {
            // Unitless arbitrary value: `p-[3.5]`
            inner.parse::<f32>().ok()?
        }
    } else {
        // Handle standard Tailwind numeric scale (e.g., "1", "2", "4", "8").
        // Tailwind's default spacing scale uses increments of 0.25rem per step.
        rest.parse::<f32>().ok()? * 0.25 * rem
    };

    let mut p = PartialPadding { left: None, right: None, top: None, bottom: None };

    // Apply the computed value strictly to the targeted sides.
    match sides {
        "all" => { p.left = Some(val); p.right = Some(val); p.top = Some(val); p.bottom = Some(val); }
        "x" => { p.left = Some(val); p.right = Some(val); }
        "y" => { p.top = Some(val); p.bottom = Some(val); }
        "s" | "l" => p.left = Some(val), // Map both 'start' and 'left' to left edge (assuming LTR)
        "e" | "r" => p.right = Some(val), // Map both 'end' and 'right' to right edge (assuming LTR)
        "t" => p.top = Some(val),
        "b" => p.bottom = Some(val),
        _ => unreachable!(),
    }

    Some(p)
}