//! Caller-owned state for [`MudTabs`](super::MudTabs).

/// Caller-owned tabs state.
///
/// This is the egui equivalent of MudBlazor's bindable `ActivePanelIndex`
/// parameter.
///
/// ```ignore
/// struct MyPage { tabs: TabsState }
///
/// MudTabs::new().show(ui, &mut self.tabs, |t| {
///     t.panel(MudTabPanel::new("One").content(|ui| { ui.label("First"); }));
///     t.panel(MudTabPanel::new("Two").content(|ui| { ui.label("Second"); }));
/// });
/// // `self.tabs.active_index` now reflects the user's selection.
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TabsState {
    /// 0-based index of the active panel.
    pub active_index: usize,
}

impl TabsState {
    /// Creates state with `active_index = 0`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Builder: seeds the active index.
    pub fn with_active_index(mut self, index: usize) -> Self {
        self.active_index = index;
        self
    }

    /// Activates `index`, clamped into `0..count`. Returns `true` when the
    /// active index changed.
    pub fn activate(&mut self, index: usize, count: usize) -> bool {
        if count == 0 {
            return false;
        }
        let clamped = index.min(count - 1);
        if clamped == self.active_index {
            return false;
        }
        self.active_index = clamped;
        true
    }

    /// Advances to the next visible index, wrapping around `0..count`.
    /// Returns `true` when the active index changed.
    pub fn next(&mut self, count: usize) -> bool {
        if count == 0 {
            return false;
        }
        self.activate((self.active_index + 1) % count, count)
    }

    /// Moves to the previous visible index, wrapping around `0..count`.
    /// Returns `true` when the active index changed.
    pub fn previous(&mut self, count: usize) -> bool {
        if count == 0 {
            return false;
        }
        self.activate((self.active_index + count - 1) % count, count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_starts_at_zero() {
        assert_eq!(TabsState::new().active_index, 0);
    }

    #[test]
    fn with_active_index_sets_value() {
        assert_eq!(TabsState::new().with_active_index(3).active_index, 3);
    }

    #[test]
    fn activate_clamps_to_range() {
        let mut s = TabsState::new();
        assert!(s.activate(2, 5));
        assert_eq!(s.active_index, 2);
        assert!(!s.activate(2, 5), "same index → no change");
        assert!(s.activate(99, 5));
        assert_eq!(s.active_index, 4);
    }

    #[test]
    fn activate_noops_on_empty() {
        let mut s = TabsState::new();
        assert!(!s.activate(3, 0));
        assert_eq!(s.active_index, 0);
    }

    #[test]
    fn next_wraps() {
        let mut s = TabsState::new().with_active_index(2);
        assert!(s.next(3));
        assert_eq!(s.active_index, 0);
        assert!(s.next(3));
        assert_eq!(s.active_index, 1);
    }

    #[test]
    fn previous_wraps() {
        let mut s = TabsState::new();
        assert!(s.previous(3));
        assert_eq!(s.active_index, 2);
        assert!(s.previous(3));
        assert_eq!(s.active_index, 1);
    }
}