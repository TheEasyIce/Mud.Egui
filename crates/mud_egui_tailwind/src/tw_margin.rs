//! Tailwind-style margin utilities (`m-*`, `mx-*`, `mt-*`, etc.) for `egui`.
//!
//! This module provides a robust parser for modern Tailwind CSS margin classes.
//!
//! # Supported Syntax
//! - **Standard Scale**: `m-4`, `mx-2`, `mt-8` (Calculated as `step * 0.25 * rem`).
//! - **Negative Margins**: `-mt-2`, `-mx-4` (Standard Tailwind prefix for negative values).
//! - **Fixed Pixels**: `m-px`, `-m-px` (Exactly 1px or -1px).
//! - **Arbitrary Values**: `mt-[12px]`, `m-[0.5rem]`, `mb-[3.5]`.
//! - **Auto Margins**: `mx-auto`, `ml-auto`.
//!
//! # Auto Margins in egui
//! Because `egui` does not inherently understand CSS `margin: auto`, this parser tracks
//! `auto` declarations and translates them into `egui::Align` properties:
//! - `mx-auto` -> Centers horizontally (`Align::Center`).
//! - `ml-auto` / `ms-auto` -> Pushes the element to the right (`Align::Max`).
//! - `mr-auto` / `me-auto` -> Pushes the element to the left (`Align::Min`).

use egui::{Align, Margin};
use mud_egui_core::base::component_base::ComponentBase;

/// The resolved margin and alignment configuration.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct TwMarginConfig {
    /// The computed raw pixel margins for all four sides.
    pub margin: Margin,

    /// The inferred horizontal alignment based on `auto` margin classes.
    pub h_align: Option<Align>,

    /// The inferred vertical alignment based on `auto` margin classes.
    pub v_align: Option<Align>,
}

/// A cache for the computed Tailwind margin configuration.
///
/// Parsing string classes on every frame is a major performance hit in immediate-mode
/// GUIs like `egui`. `TwMarginCache` tracks the class list's version and only
/// re-evaluates the classes when the component's state actually mutates.
#[derive(Default)]
pub struct TwMarginCache {
    /// The version of the class list when this cache was last updated.
    version: u64,
    /// The cached margin configuration, or `None` if no margin classes were applied.
    config: Option<TwMarginConfig>,
}

impl TwMarginCache {
    /// Resolves the margin configuration based on the current classes.
    ///
    /// # Arguments
    /// * `base` - The component providing the `class_names` and `classes_version`.
    /// * `rem` - The current scale factor for converting `rem` to pixels (usually 16.0).
    pub fn resolve(&mut self, base: &ComponentBase, rem: f32) -> Option<TwMarginConfig> {
        // Cache hit: avoid re-parsing if the class list is unchanged.
        if self.version == base.classes_version() {
            return self.config;
        }

        let mut left = 0.0;
        let mut right = 0.0;
        let mut top = 0.0;
        let mut bottom = 0.0;

        // Track which specific edges have `auto` applied.
        let mut auto_l = false;
        let mut auto_r = false;
        let mut auto_t = false;
        let mut auto_b = false;

        let mut matched = false;

        // Enforce CSS specificity without sorting the classes.
        // We use 4 passes (0 to 3) moving from general to highly specific.
        // 0: All sides (`m-*`)
        // 1: Axes (`mx-*`, `my-*`)
        // 2: Logical sides (`ms-*`, `me-*`)
        // 3: Physical sides (`mt-*`, `mb-*`, `ml-*`, `mr-*`)
        for pass in 0..=3 {
            for cls in base.class_names() {
                if let Some(c) = parse_margin(cls, pass, rem) {
                    matched = true;
                    // Apply values if this class/pass targeted these edges.
                    // This naturally allows `mt-5` to overwrite the top edge of `m-2`.
                    if let Some(v) = c.left { left = v; auto_l = c.is_auto; }
                    if let Some(v) = c.right { right = v; auto_r = c.is_auto; }
                    if let Some(v) = c.top { top = v; auto_t = c.is_auto; }
                    if let Some(v) = c.bottom { bottom = v; auto_b = c.is_auto; }
                }
            }
        }

        // Translate the parsed values into a final config.
        let result = if matched {
            Some(TwMarginConfig {
                margin: Margin {
                    left: left as i8,
                    right: right as i8,
                    top: top as i8,
                    bottom: bottom as i8
                },
                // Translate auto margins into horizontal `egui` alignments
                h_align: if auto_l && auto_r { Some(Align::Center) }
                else if auto_l { Some(Align::Max) } // ml-auto pushes right
                else if auto_r { Some(Align::Min) } // mr-auto pushes left
                else { None },
                // Translate auto margins into vertical `egui` alignments
                v_align: if auto_t && auto_b { Some(Align::Center) }
                else if auto_t { Some(Align::Max) } // mt-auto pushes down
                else if auto_b { Some(Align::Min) } // mb-auto pushes up
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
/// ensuring unaffected sides remain `None` and do not overwrite prior specificity passes.
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
/// * `cls` - The class string to parse (e.g., "-mt-4", "mx-auto", "m-[10px]").
/// * `current_pass` - The specificity tier currently being evaluated (0 through 3).
/// * `rem` - The pixel multiplier for rem values.
fn parse_margin(cls: &str, current_pass: usize, rem: f32) -> Option<PartialMargin> {
    // 1. Check for negative margin prefix.
    // Tailwind uses a leading hyphen for negative margins (e.g., `-mt-2`).
    let (is_neg, cls) = if let Some(stripped) = cls.strip_prefix('-') {
        (true, stripped)
    } else {
        (false, cls)
    };

    // Fast path: if it doesn't start with 'm' now, it's not a margin class.
    if !cls.starts_with('m') { return None; }

    // 2. Categorize the class into its target sides, specificity tier, and the payload string.
    let (sides, tier, rest) = if cls.starts_with("m-") { ("all", 0, &cls[2..]) }
    else if cls.starts_with("mx-") { ("x", 1, &cls[3..]) }
    else if cls.starts_with("my-") { ("y", 1, &cls[3..]) }
    // Logical properties (Start/End)
    else if cls.starts_with("ms-") { ("s", 2, &cls[3..]) }
    else if cls.starts_with("me-") { ("e", 2, &cls[3..]) }
    // Physical properties (Top/Right/Bottom/Left)
    else if cls.starts_with("mt-") { ("t", 3, &cls[3..]) }
    else if cls.starts_with("mr-") { ("r", 3, &cls[3..]) }
    else if cls.starts_with("mb-") { ("b", 3, &cls[3..]) }
    else if cls.starts_with("ml-") { ("l", 3, &cls[3..]) }
    else { return None; };

    // Ignore this class if it doesn't belong to the specificity pass we are currently on.
    if tier != current_pass { return None; }

    // 3. Handle 'auto' keywords.
    let is_auto = rest == "auto";
    if is_auto && is_neg { return None; } // `-m-auto` is invalid CSS/Tailwind

    // 4. Calculate the final numeric value.
    let val = if is_auto {
        // Auto margins provide alignment, not raw pixel spacing.
        0.0
    } else if rest == "px" {
        // Tailwind's `m-px` represents exactly 1 physical pixel.
        if is_neg { -1.0 } else { 1.0 }
    } else if let Some(inner) = rest.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
        // Handle arbitrary bracket syntax: `m-[...]`
        let mut v = if let Some(r) = inner.strip_suffix("rem") {
            // e.g., `m-[2rem]`
            r.parse::<f32>().ok()? * rem
        } else if let Some(p) = inner.strip_suffix("px") {
            // e.g., `m-[12px]`
            p.parse::<f32>().ok()?
        } else {
            // Unitless arbitrary value: `m-[3.5]`
            inner.parse::<f32>().ok()?
        };

        if is_neg { v = -v; }
        v
    } else {
        // Handle standard Tailwind numeric scale (e.g., "1", "2", "4", "8").
        // Tailwind's default spacing scale uses increments of 0.25rem per step.
        let step = rest.parse::<f32>().ok()?;
        let mut v = step * 0.25 * rem;

        if is_neg { v = -v; }
        v
    };

    let mut p = PartialMargin { left: None, right: None, top: None, bottom: None, is_auto };

    // 5. Apply the computed value strictly to the targeted sides.
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