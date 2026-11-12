//! Icon flipping and mirroring transformations for `mud_egui`.
//!
//! This module defines the `IconFlip` enum, which dictates whether an icon
//! (either an SVG path or a font glyph) should be mirrored horizontally, vertically,
//! or both before being rendered on the screen.

/// Specifies the mirroring transformation applied to a `MudIcon`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum IconFlip {
    /// No transformation. The icon is rendered in its original, default orientation.
    #[default]
    None,

    /// Mirrors the icon along the vertical axis (flips left-to-right).
    /// Useful for directional icons in Right-To-Left (RTL) layouts.
    Horizontal,

    /// Mirrors the icon along the horizontal axis (flips top-to-bottom).
    Vertical,

    /// Mirrors the icon along both the vertical and horizontal axes
    /// (visually equivalent to a 180-degree rotation).
    Both,
}