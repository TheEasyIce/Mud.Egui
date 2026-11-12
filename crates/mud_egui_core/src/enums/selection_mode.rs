//! Selection modes for toggle groups in `mud_egui`.
//!
//! This module defines the `SelectionMode` enum, which controls the cardinality 
//! and toggling rules of items within a `MudToggleGroup` (the `mud_egui` equivalent 
//! of a MudBlazor button group or segmented control).

/// Defines how a `MudToggleGroup` handles the selection state of its child items.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionMode {
    /// Standard radio-button behavior.
    ///
    /// Exactly one item is selected at a time. Clicking a different item moves 
    /// the selection to the new item. Clicking the currently selected item does 
    /// nothing (the group cannot be emptied).
    Single,

    /// Nullable radio-button behavior.
    ///
    /// At most one item is selected at a time (0 or 1). Clicking a different item 
    /// moves the selection. Clicking the currently selected item deselects it, 
    /// leaving the group with no active selection.
    Toggle,

    /// Independent checkbox behavior.
    ///
    /// Multiple items can be selected simultaneously (0 to N). Clicking any item 
    /// toggles its individual selection state independently of the other items 
    /// in the group.
    Multi,
}