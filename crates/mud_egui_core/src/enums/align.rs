//! Semantic text and component alignment abstractions for `mud_egui`.
//!
//! This module defines the `MudAlign` enum, which mirrors the alignment properties
//! found in MudBlazor (and standard CSS). Because `egui` handles text alignment
//! dynamically based on the current layout of the `Ui` container, this enum
//! provides a conversion method (`layout()`) to generate the exact `egui::Layout`
//! required to force the desired alignment.

pub use egui::{Align, Layout};

/// Semantic alignment options for text and container contents.
///
/// # Note on RTL (Right-to-Left) languages
/// Material Design uses `Start` and `End` rather than `Left` and `Right` to support
/// automatic flipping for RTL languages (like Arabic or Hebrew). Currently, `egui`
/// does not natively support RTL layouts, so `Start` is hardcoded to behave exactly
/// like `Left`, and `End` like `Right`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MudAlign {
    /// Leaves the alignment up to the parent `egui::Ui` layout.
    /// This is the default, preventing unnecessary layout allocations.
    #[default]
    Inherit,

    /// Forces alignment to the absolute left of the container.
    Left,

    /// Forces alignment to the horizontal center of the container.
    Center,

    /// Forces alignment to the absolute right of the container.
    Right,

    /// Forces text to stretch so that lines align flush with both the left and right edges.
    Justify,

    /// Aligns to the reading start edge (currently Left).
    Start,

    /// Aligns to the reading end edge (currently Right).
    End,
}

impl MudAlign {
    /// Translates the semantic alignment into a concrete `egui::Layout`.
    ///
    /// # Returns
    /// * `Some(Layout)` - If a specific alignment is requested, returns the `egui` layout
    ///   configuration needed to enforce it.
    /// * `None` - If the alignment is `Inherit`, returning `None` allows the component
    ///   rendering logic to skip allocating a new UI layout, saving processing time and
    ///   relying on the parent container's flow.
    pub fn layout(self) -> Option<Layout> {
        match self {
            // Do not force a new layout; inherit whatever the parent Ui is doing.
            MudAlign::Inherit => None,

            // Align::Min pushes content to the minimum X-coordinate (Left).
            MudAlign::Left | MudAlign::Start => Some(Layout::top_down(Align::Min)),

            // Align::Center aligns content equally between the min and max X-coordinates.
            MudAlign::Center => Some(Layout::top_down(Align::Center)),

            // Align::Max pushes content to the maximum X-coordinate (Right).
            MudAlign::Right | MudAlign::End => Some(Layout::top_down(Align::Max)),

            // Text Justification Trick:
            // `egui::Label` doesn't have a simple `.justify()` method. Instead, it checks
            // if the `Ui` it is placed in has a "cross-justified" layout.
            // `top_down_justified` tells `egui` to stretch elements to fill the available width,
            // which internally signals the `Label` to render with justified text wrapping.
            MudAlign::Justify => Some(Layout::top_down_justified(Align::Min)),
        }
    }
}