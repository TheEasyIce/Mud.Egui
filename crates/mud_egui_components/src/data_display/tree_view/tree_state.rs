//! Caller-owned state for [`MudTreeView`](super::MudTreeView).

use egui::Rect;

use mud_egui_core::enums::SelectionMode;

/// Caller-owned tree state.
///
/// This is the egui equivalent of MudBlazor's bindable parameters
/// `SelectedValue`, `SelectedValues`, and per-item `Expanded`.
///
/// * [`selected_value`](Self::selected_value) is written in `Single` / `Toggle`
///   mode.
/// * [`selected_values`](Self::selected_values) is written in `Multi` mode.
/// * [`expanded`](Self::expanded) stores per-node expansion state, seeded from
///   each [`TreeItem::expanded`](super::TreeItem::expanded) the first time the
///   node is rendered.
#[derive(Debug, Clone)]
pub struct TreeState<T> {
    /// Single selection value (`SelectedValue`).
    pub selected_value: Option<T>,
    /// Multi selection values (`SelectedValues`).
    pub selected_values: Vec<T>,
    /// Expansion state, keyed by node value.
    pub expanded: Vec<(T, bool)>,
    /// Screen-space rectangles of every rendered row in the last frame.
    pub last_node_rects: Vec<Rect>,
}

impl<T> Default for TreeState<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> TreeState<T> {
    /// Creates empty state.
    pub fn new() -> Self {
        Self {
            selected_value: None,
            selected_values: Vec::new(),
            expanded: Vec::new(),
            last_node_rects: Vec::new(),
        }
    }
}

impl<T: PartialEq> TreeState<T> {
    /// Whether `value` is selected under `mode`.
    pub fn is_selected(&self, value: &T, mode: SelectionMode) -> bool {
        match mode {
            SelectionMode::Multi => self.selected_values.iter().any(|v| v == value),
            _ => self.selected_value.as_ref() == Some(value),
        }
    }

    /// Current expansion state for `value`, or `None` if never seeded.
    pub fn is_expanded(&self, value: &T) -> Option<bool> {
        self.expanded
            .iter()
            .find(|(v, _)| v == value)
            .map(|(_, b)| *b)
    }

    /// Seeds the expansion state for `value` with `initial` if it is not yet
    /// known.
    pub fn ensure_expanded(&mut self, value: &T, initial: bool)
    where
        T: Clone,
    {
        if self.is_expanded(value).is_none() {
            self.expanded.push((value.clone(), initial));
        }
    }

    /// Flips the expansion state for `value`.
    pub fn toggle_expanded(&mut self, value: &T)
    where
        T: Clone,
    {
        if let Some(entry) = self.expanded.iter_mut().find(|(v, _)| v == value) {
            entry.1 = !entry.1;
        } else {
            self.expanded.push((value.clone(), true));
        }
    }

    /// Removes every expansion entry. Useful when the tree structure changes
    /// and stale keys accumulate.
    pub fn clear_expanded(&mut self) {
        self.expanded.clear();
    }
}