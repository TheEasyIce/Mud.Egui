//! Collector passed to [`MudTreeView::show`](super::MudTreeView::show).

use super::TreeItem;

/// Collector for root-level tree items.
pub struct TreeUi<'a, T> {
    pub(crate) roots: Vec<TreeItem<'a, T>>,
}

impl<'a, T> TreeUi<'a, T> {
    /// Adds a root node.
    pub fn item(&mut self, item: TreeItem<'a, T>) {
        self.roots.push(item);
    }

    /// Number of root nodes collected so far.
    pub fn len(&self) -> usize {
        self.roots.len()
    }

    /// Whether no root nodes have been collected.
    pub fn is_empty(&self) -> bool {
        self.roots.is_empty()
    }
}