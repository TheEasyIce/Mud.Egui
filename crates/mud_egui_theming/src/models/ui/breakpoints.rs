//! Responsive design breakpoints for `mud_egui`.
//!
//! This module provides the `Breakpoints` configuration and the `BreakpointBucket` enum.
//! Breakpoints are specific viewport widths where the application layout shifts
//! (e.g., collapsing a side drawer, wrapping a grid, or shrinking a container)
//! to best fit the user's screen size.
//!
//! The defaults provided here strictly follow the standard Material Design / MudBlazor
//! responsive layout grid.

/// Represents the discrete responsive size categories (buckets) of the current viewport.
///
/// These buckets are used to make programmatic layout decisions without needing
/// to check raw pixel values throughout your UI code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BreakpointBucket {
    /// Extra Small: Typically represents mobile phones in portrait mode.
    Xs,
    /// Small: Typically represents mobile phones in landscape mode, or small tablets in portrait.
    Sm,
    /// Medium: Typically represents tablets in landscape mode, or small laptops.
    Md,
    /// Large: Typically represents standard desktop monitors.
    Lg,
    /// Extra Large: Typically represents large, high-resolution desktop monitors.
    Xl,
    /// Double Extra Large: Typically represents ultra-wide or 4K monitors.
    Xxl,
}

/// The configuration defining the exact logical pixel widths where layout transitions occur.
///
/// Each value represents the *minimum* width required to enter that specific breakpoint bucket.
#[derive(Debug, Clone)]
pub struct Breakpoints {
    /// The absolute minimum width. Always `0.0`.
    pub xs: f32,
    /// The minimum width for the Small (`Sm`) bucket.
    pub sm: f32,
    /// The minimum width for the Medium (`Md`) bucket.
    pub md: f32,
    /// The minimum width for the Large (`Lg`) bucket.
    pub lg: f32,
    /// The minimum width for the Extra Large (`Xl`) bucket.
    pub xl: f32,
    /// The minimum width for the Double Extra Large (`Xxl`) bucket.
    pub xxl: f32,
}

impl Default for Breakpoints {
    /// Generates the standard Material Design / MudBlazor breakpoint thresholds.
    ///
    /// These values are defined in logical pixels (points) rather than physical pixels.
    fn default() -> Self {
        Self {
            // [0px - 599px]: Mobile phones (Portrait)
            xs: 0.0,

            // [600px - 959px]: Tablets (Portrait) / Large Phones (Landscape)
            sm: 600.0,

            // [960px - 1279px]: Tablets (Landscape) / Small Laptops
            md: 960.0,

            // [1280px - 1919px]: Desktop Monitors
            lg: 1280.0,

            // [1920px - 2559px]: Large Desktop Monitors (1080p / 1440p)
            xl: 1920.0,

            // [2560px and up]: Ultrawide and 4K+ Monitors
            xxl: 2560.0,
        }
    }
}

impl Breakpoints {
    /// Evaluates a given screen width and returns the active `BreakpointBucket`.
    ///
    /// # Arguments
    /// * `width` - The current width of the application window or central panel in logical pixels.
    ///
    /// # Returns
    /// The `BreakpointBucket` that this width falls into.
    ///
    /// # Usage
    /// Typically, you grab the screen width using `ctx.screen_rect().width()` in `egui`
    /// and pass it into this function to decide whether to render mobile navigation
    /// or desktop navigation.
    pub fn get_bucket(&self, width: f32) -> BreakpointBucket {
        // We check from smallest to largest.
        // If the width is less than the `sm` threshold, it must be `Xs`.
        if width < self.sm {
            BreakpointBucket::Xs
        }
        else if width < self.md {
            BreakpointBucket::Sm
        }
        else if width < self.lg {
            BreakpointBucket::Md
        }
        else if width < self.xl {
            BreakpointBucket::Lg
        }
        else if width < self.xxl {
            BreakpointBucket::Xl
        }
        else {
            // Anything greater than or equal to `xxl` defaults to the largest bucket.
            BreakpointBucket::Xxl
        }
    }
}