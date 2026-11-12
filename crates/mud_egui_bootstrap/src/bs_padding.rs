//! Bootstrap-style padding utilities (`p-*`, `px-*`, `pt-*`, etc.) for `egui`.
//!
//! This module provides an efficient parser to translate standard Bootstrap padding classes
//! into an `egui::Margin` configuration.
//!
//! # Specificity
//! To accurately mirror CSS behavior, this parser strictly enforces Bootstrap's specificity rules.
//! In CSS, more specific rules override general ones regardless of the order they appear in the class list.
//!
//! Evaluation order (lowest to highest specificity):
//! 1. `p-*` (All sides)
//! 2. `px-*` / `py-*` (Axes)
//! 3. `pt-*` / `pb-*` / `ps-*` / `pe-*` (Specific sides)
//!
//! *Example:* If you write `pt-5 p-2`, the `pt-5` wins for the top edge because it is more specific.

use egui::Margin;
use mud_egui_core::base::component_base::ComponentBase;

/// Base multiplier for CSS `rem` values. 
/// Mock conversion: 1 rem is treated as 16 physical pixels.
const REM: f32 = 16.0;

/// A cache for computed Bootstrap padding margins.
///
/// String parsing on every single frame is a major performance hit in an immediate-mode 
/// GUI like `egui`. `BsPaddingCache` tracks the class list's version and only re-evaluates 
/// the classes when the component's state actually mutates.
#[derive(Default)]
pub struct BsPaddingCache {
    /// The version of the class list when this cache was last updated.
    version: u64,
    /// The cached margin configuration, or `None` if no padding classes were applied.
    config: Option<Margin>,
}

impl BsPaddingCache {
    /// Resolves the padding margin based on the current classes attached to the `ComponentBase`.
    ///
    /// If the component's class version hasn't changed, this returns the cached value instantly.
    /// Otherwise, it parses the class list across 3 specificity passes to build the final margin.
    ///
    /// # Arguments
    /// * `base` - The component providing the `class_names` and `classes_version`.
    pub fn resolve(&mut self, base: &ComponentBase) -> Option<Margin> {
        // Cache hit: avoid re-parsing if the class list is unchanged.
        if self.version == base.classes_version() {
            return self.config;
        }

        // Standard Bootstrap 5 spacing scale multipliers.
        // Index maps to the class step (e.g., `p-0` -> 0.0, `p-3` -> 1.0, `p-5` -> 3.0).
        let scale = [0.0, 0.25, 0.5, 1.0, 1.5, 3.0];

        let mut left = 0.0;
        let mut right = 0.0;
        let mut top = 0.0;
        let mut bottom = 0.0;
        let mut matched = false;

        // We use a 3-pass system to enforce CSS specificity without sorting the classes.
        // Pass 0: Applies 'p-*' (all sides)
        // Pass 1: Applies 'px-*' and 'py-*' (overwriting 'p-*' where applicable)
        // Pass 2: Applies 'pt-*', 'pb-*', 'ps-*', 'pe-*' (overwriting both above)
        for pass in 0..=2 {
            for cls in base.class_names() {
                if let Some(padding) = parse_padding(cls, pass, scale) {
                    matched = true;
                    // Apply values if this specific class/pass targeted these edges.
                    if let Some(v) = padding.left { left = v; }
                    if let Some(v) = padding.right { right = v; }
                    if let Some(v) = padding.top { top = v; }
                    if let Some(v) = padding.bottom { bottom = v; }
                }
            }
        }

        // Convert the computed f32 values into `egui::Margin`.
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
/// * `cls` - The class string to parse (e.g., "p-3", "px-4").
/// * `current_pass` - The specificity tier currently being evaluated (0, 1, or 2).
/// * `scale` - The array of rem multipliers mapping to class suffixes 0-5.
fn parse_padding(cls: &str, current_pass: usize, scale: [f32; 6]) -> Option<PartialPadding> {
    // Fast path: skip non-padding classes immediately
    if !cls.starts_with('p') { return None; }

    // Categorize the class into its target sides, specificity tier, and the remaining payload string.
    let (sides, tier, rest) = if cls.starts_with("p-") { ("all", 0, &cls[2..]) }
    else if cls.len() >= 4 && cls.starts_with("px-") { ("x", 1, &cls[3..]) }
    else if cls.starts_with("py-") { ("y", 1, &cls[3..]) }
    else if cls.starts_with("pt-") { ("t", 2, &cls[3..]) }
    else if cls.starts_with("pb-") { ("b", 2, &cls[3..]) }
    // Bootstrap 5 uses 's' (start) and 'e' (end) instead of 'l'/'r' for LTR/RTL support.
    // Here we map 's' to left and 'e' to right (assuming LTR context).
    else if cls.starts_with("ps-") { ("s", 2, &cls[3..]) }
    else if cls.starts_with("pe-") { ("e", 2, &cls[3..]) }
    else { return None; };

    // Ignore this class if it doesn't belong to the specificity pass we are currently on.
    if tier != current_pass { return None; }

    // Extract the step number from the class name (e.g., the '3' in 'p-3') and check bounds.
    let step = rest.parse::<usize>().ok()?;
    if step >= scale.len() { return None; }

    // Calculate final pixel value: (scale multiplier * 16px)
    let val = scale[step] * REM;

    let mut p = PartialPadding { left: None, right: None, top: None, bottom: None };

    // Apply the computed value strictly to the targeted sides.
    match sides {
        "all" => { p.left = Some(val); p.right = Some(val); p.top = Some(val); p.bottom = Some(val); }
        "x" => { p.left = Some(val); p.right = Some(val); }
        "y" => { p.top = Some(val); p.bottom = Some(val); }
        "t" => p.top = Some(val),
        "b" => p.bottom = Some(val),
        "s" => p.left = Some(val),
        "e" => p.right = Some(val),
        _ => unreachable!(),
    }

    Some(p)
}