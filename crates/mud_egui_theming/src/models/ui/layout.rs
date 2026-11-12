//! Structural layout configurations for `mud_egui`.
//!
//! This module defines the `LayoutProperties` struct, which dictates the core geometric
//! measurements of the application shell. By centralizing these values (such as appbar heights
//! and drawer widths) in the theme, the entire UI can remain perfectly aligned and proportionally
//! responsive, mimicking the behavior of the C# MudBlazor framework.

/// Defines the global geometric properties for the application layout.
///
/// These values represent logical pixels (or `egui` points) and are used by the
/// main application shell components (like `MudLayout`, `MudAppBar`, and `MudDrawer`)
/// to calculate available content space dynamically.
#[derive(Debug, Clone)]
pub struct LayoutProperties {
    /// The default border radius applied to UI elements (buttons, cards, dialogs)
    /// when no specific rounding class (e.g., `rounded-lg`) is provided.
    pub default_border_radius: f32,

    /// The width of the left navigation drawer when it is in its collapsed ("mini") state.
    /// Typically wide enough to comfortably display a vertical column of icons.
    pub drawer_mini_width_left: f32,

    /// The width of the right navigation drawer when it is in its collapsed ("mini") state.
    pub drawer_mini_width_right: f32,

    /// The width of the left navigation drawer when it is fully expanded.
    pub drawer_width_left: f32,

    /// The width of the right navigation drawer when it is fully expanded.
    pub drawer_width_right: f32,

    /// The fixed height of the top application navigation bar (`MudAppBar`).
    pub appbar_height: f32,
}

impl Default for LayoutProperties {
    /// Generates the standard default `LayoutProperties`.
    ///
    /// These defaults mirror the exact standard measurements found in MudBlazor
    /// and the Material Design 2/3 specifications.
    fn default() -> Self {
        Self {
            // Material Design standard for subtle, modern corners on cards and buttons.
            default_border_radius: 4.0,

            // 56px is the standard Material Design minimum touch target size.
            // It provides exactly enough room for a 24px icon with 16px of padding on each side.
            drawer_mini_width_left: 56.0,
            drawer_mini_width_right: 56.0,

            // 240px is the standard expanded drawer width, offering a good balance
            // between reading long menu text and preserving main canvas space.
            drawer_width_left: 240.0,
            drawer_width_right: 240.0,

            // 64px is the standard Material Design desktop App Bar height.
            // (Note: Mobile apps often use 56px, but for desktop/web, 64px is preferred).
            appbar_height: 64.0,
        }
    }
}