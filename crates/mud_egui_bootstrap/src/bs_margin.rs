//! Bootstrap-style margin utilities (`m-*`, `mx-*`, `mt-*`, etc.) for `egui`.
//!
//! This module parses standard Bootstrap spacing classes and translates them into an
//! `egui::Margin` configuration.
//!
//! # Specificity
//! To accurately mirror CSS behavior, this parser strictly enforces Bootstrap's specificity rules.
//! In CSS, more specific rules override general ones regardless of the order they appear in the class list.
//!
//! Evaluation order (lowest to highest specificity):
//! 1. `m-*` (All sides)
//! 2. `mx-*` / `my-*` (Axes)
//! 3. `mt-*` / `mb-*` / `ms-*` / `me-*` (Specific sides)
//!
//! *Example:* If you write `mt-2 m-1`, the `mt-2` wins for the top edge because it is more specific.
//!
//! # Auto Margins
//! In CSS, `margin: auto` is frequently used to center elements or push them to the sides.
//! `egui` does not inherently understand "auto margins", so we map them directly to `egui::Align`:
//! * `mx-auto` -> Centers horizontally (`Align::Center`)
//! * `ms-auto` (margin-start: auto) -> Pushes the element to the right (`Align::Max`)
//! * `me-auto` (margin-end: auto) -> Pushes the element to the left (`Align::Min`)

use egui::{Align, Margin};
use mud_egui_core::base::component_base::ComponentBase;

/// Base multiplier for CSS `rem` values.
/// Mock conversion: 1 rem is treated as 16 physical pixels.
///
/// *Note:* In a complete UI framework, this should eventually be fetched from a dynamic
/// `mud_egui_theming` crate rather than being hardcoded.
const REM: f32 = 16.0;

/// The resolved margin and alignment configuration.
///
/// Because `auto` margins in CSS affect alignment rather than raw pixel spacing,
/// this struct returns both the computed pixel margins and the inferred `egui` alignments.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct MarginConfig {
    /// The computed raw pixel margins for all four sides.
    pub margin: Margin,

    /// The inferred horizontal alignment.
    /// Maps `mx-auto` to `Center`, `ms-auto` to `Max` (right), and `me-auto` to `Min` (left).
    pub h_align: Option<Align>,

    /// The inferred vertical alignment.
    /// Maps `my-auto` to `Center`, `mt-auto` to `Max` (bottom), and `mb-auto` to `Min` (top).
    pub v_align: Option<Align>,
}

/// A cache for computed Bootstrap margin configurations.
///
/// String parsing on every single frame is a major performance hit in an immediate-mode
/// GUI like `egui`. `BsMarginCache` tracks the class list's version and only re-evaluates
/// the classes when the component's state actually mutates.
#[derive(Default)]
pub struct BsMarginCache {
    /// The version of the class list when this cache was last updated.
    version: u64,
    /// The cached margin configuration, or `None` if no margin classes were applied.
    config: Option<MarginConfig>,
}

impl BsMarginCache {
    /// Resolves the margin configuration based on the current classes attached to the `ComponentBase`.
    ///
    /// If the component's class version hasn't changed, this returns the cached value instantly.
    /// Otherwise, it parses the class list across 3 specificity passes.
    ///
    /// # Arguments
    /// * `base` - The component providing the `class_names` and `classes_version`.
    pub fn resolve(&mut self, base: &ComponentBase) -> Option<MarginConfig> {
        // Cache hit: avoid re-parsing if the class list is unchanged.
        if self.version == base.classes_version() {
            return self.config;
        }

        // Standard Bootstrap 5 spacing scale multipliers.
        // Index maps to the class step (e.g., `m-0` -> 0.0, `m-3` -> 1.0, `m-5` -> 3.0).
        let scale = [0.0, 0.25, 0.5, 1.0, 1.5, 3.0];

        let mut left = 0.0;
        let mut right = 0.0;
        let mut top = 0.0;
        let mut bottom = 0.0;

        // Track which specific edges have `auto` applied.
        let mut auto_l = false; let mut auto_r = false;
        let mut auto_t = false; let mut auto_b = false;

        let mut matched = false;

        // We use a 3-pass system to enforce CSS specificity without sorting the classes.
        // Pass 0: Applies 'm-*' (all sides)
        // Pass 1: Applies 'mx-*' and 'my-*' (overwriting 'm-*' where applicable)
        // Pass 2: Applies 'mt-*', 'mb-*', 'ms-*', 'me-*' (overwriting both above)
        for pass in 0..=2 {
            for cls in base.class_names() {
                if let Some(config) = parse_margin(cls, pass, scale) {
                    matched = true;
                    // Apply pixel values and auto-status if this class targets these edges.
                    if let Some(v) = config.left { left = v; auto_l = config.is_auto; }
                    if let Some(v) = config.right { right = v; auto_r = config.is_auto; }
                    if let Some(v) = config.top { top = v; auto_t = config.is_auto; }
                    if let Some(v) = config.bottom { bottom = v; auto_b = config.is_auto; }
                }
            }
        }

        // Translate the parsed values into a final config.
        let result = if matched {
            Some(MarginConfig {
                margin: Margin {
                    left: left as i8,
                    right: right as i8,
                    top: top as i8,
                    bottom: bottom as i8
                },
                // Map auto margins to horizontal alignments
                h_align: if auto_l && auto_r { Some(Align::Center) }
                else if auto_l { Some(Align::Max) } // ms-auto pushes to the right
                else if auto_r { Some(Align::Min) } // me-auto pushes to the left
                else { None },
                // Map auto margins to vertical alignments
                v_align: if auto_t && auto_b { Some(Align::Center) }
                else if auto_t { Some(Align::Max) } // mt-auto pushes to the bottom
                else if auto_b { Some(Align::Min) } // mb-auto pushes to the top
                else { None },
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

/// Intermediate representation of a margin from a single CSS class.
///
/// Uses `Option<f32>` to explicitly track which sides are being modified,
/// ensuring unaffected sides remain `None` and do not overwrite prior passes.
struct PartialMargin {
    left: Option<f32>,
    right: Option<f32>,
    top: Option<f32>,
    bottom: Option<f32>,
    /// Flags whether this class is an `auto` class (e.g., `mx-auto`)
    is_auto: bool,
}

/// Parses a single class string into a `PartialMargin` struct.
///
/// # Arguments
/// * `cls` - The class string to parse (e.g., "m-3", "mx-auto", "mt-n2").
/// * `current_pass` - The specificity tier currently being evaluated (0, 1, or 2).
/// * `scale` - The array of rem multipliers mapping to class suffixes 0-5.
fn parse_margin(cls: &str, current_pass: usize, scale: [f32; 6]) -> Option<PartialMargin> {
    // Fast path: skip non-margin classes immediately
    if !cls.starts_with('m') { return None; }

    // Categorize the class into its target sides, specificity tier, and the remaining payload string.
    let (sides, tier, rest) = if cls.starts_with("m-") {
        ("all", 0, &cls[2..])
    }
    else if cls.len() >= 4 && cls.starts_with("mx-") { ("x", 1, &cls[3..]) }
    else if cls.starts_with("my-") { ("y", 1, &cls[3..]) }
    else if cls.starts_with("mt-") { ("t", 2, &cls[3..]) }
    else if cls.starts_with("mb-") { ("b", 2, &cls[3..]) }
    // Bootstrap 5 uses 's' (start) and 'e' (end) instead of 'l'/'r' for LTR/RTL support.
    else if cls.starts_with("ms-") { ("s", 2, &cls[3..]) }
    else if cls.starts_with("me-") { ("e", 2, &cls[3..]) }
    else { return None; };

    // Ignore this class if it doesn't belong to the specificity pass we are currently on.
    if tier != current_pass { return None; }

    // Bootstrap supports negative margins by prefixing the number with 'n' (e.g., `mt-n2`).
    let (val_str, is_neg) = if let Some(s) = rest.strip_prefix('n') {
        (s, true)
    } else {
        (rest, false)
    };

    let is_auto = val_str == "auto";

    // Calculate final pixel value.
    let val = if is_auto {
        // Auto margins don't provide raw pixel spacing; they provide alignment.
        0.0
    } else {
        // Extract the step number and check bounds.
        let step = val_str.parse::<usize>().ok()?;
        if step >= scale.len() { return None; }

        let mut v = scale[step] * REM;
        if is_neg { v = -v; }
        v
    };

    let mut p = PartialMargin { left: None, right: None, top: None, bottom: None, is_auto };

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