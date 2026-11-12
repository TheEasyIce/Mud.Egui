//! Caller-owned state for [`MudTimeline`](super::MudTimeline).

use egui::{Pos2, Rect};

/// Caller-owned timeline state.
///
/// This is the egui equivalent of MudBlazor's bindable `SelectedIndex`
/// parameter (`MudBaseItemsControl<T>.SelectedIndex`).
#[derive(Debug, Clone, Default)]
pub struct TimelineState {
    /// Currently selected item index (`SelectedIndex`).
    pub selected_index: Option<usize>,
    /// Screen-space rectangles of every rendered item row in the last frame.
    pub last_item_rects: Vec<Rect>,
    /// Screen-space centers of every rendered dot in the last frame.
    pub last_dot_centers: Vec<Pos2>,
}

impl TimelineState {
    /// Creates empty state with no selection.
    pub fn new() -> Self {
        Self::default()
    }

    /// Builder form of [`selected_index`](Self::selected_index).
    pub fn with_selected_index(mut self, index: usize) -> Self {
        self.selected_index = Some(index);
        self
    }

    /// Clears the selection.
    pub fn clear_selection(&mut self) {
        self.selected_index = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_is_empty() {
        let s = TimelineState::new();
        assert!(s.selected_index.is_none());
        assert!(s.last_item_rects.is_empty());
        assert!(s.last_dot_centers.is_empty());
    }

    #[test]
    fn with_selected_index_sets_value() {
        let s = TimelineState::new().with_selected_index(3);
        assert_eq!(s.selected_index, Some(3));
    }

    #[test]
    fn clear_selection_resets() {
        let mut s = TimelineState::new().with_selected_index(2);
        s.clear_selection();
        assert!(s.selected_index.is_none());
    }
}