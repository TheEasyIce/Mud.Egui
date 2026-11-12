//! Icon rendering variants for `mud_egui` components.
//!
//! This module defines the `IconKind` enum, which dictates how a `MudIcon`
//! component will interpret and render its data. It bridges MudBlazor's standard
//! reliance on raw SVG path strings with traditional font-based icon approaches.

/// Determines the underlying rendering strategy and data format for an icon.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum IconKind {
    /// The icon data represents a raw SVG path string (the `d` attribute of a `<path>` tag).
    ///
    /// This is the standard and default format for MudBlazor icons
    /// (e.g., `Icons.Material.Filled.Home`). The rendering engine will parse this
    /// mathematical string and draw it directly using `egui`'s shape/vector painter.
    #[default]
    SvgPath,

    /// The icon data represents a text character, ligature, or string intended for
    /// a font-based icon rendering system (e.g., FontAwesome, or a loaded Material Icons font).
    ///
    /// The rendering engine will draw this using standard typography text layout
    /// (`egui::Label` or `RichText`).
    Glyph,
}