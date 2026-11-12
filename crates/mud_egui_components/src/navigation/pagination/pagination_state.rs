//! Caller-owned state for [`MudPagination`](super::MudPagination).

/// Caller-owned pagination state.
///
/// This is the egui equivalent of MudBlazor's bindable `Selected` parameter.
/// The value is **1-based** (`1..=count`), matching `MudPagination::Selected`.
///
/// # MudBlazor's 0-based `NavigateToAsync(int)`
///
/// MudBlazor has two `NavigateToAsync` overloads:
///
/// * `NavigateToAsync(Page page)` — takes the `Page` enum.
/// * `NavigateToAsync(int pageIndex)` — takes a **0-based** index.
///
/// This port only exposes 1-based methods
/// ([`navigate_to`](Self::navigate_to), [`next`](Self::next),
/// [`previous`](Self::previous)) because the `Selected` parameter itself is
/// 1-based. Callers who want to mirror the 0-based API can do
/// `state.selected = index + 1`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PaginationState {
    /// Currently selected page (1-based). Clamped into `1..=count` by
    /// [`MudPagination::show`](super::MudPagination::show).
    pub selected: u32,
}

impl Default for PaginationState {
    fn default() -> Self {
        Self { selected: 1 }
    }
}

impl PaginationState {
    /// Creates state with `selected = 1`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates state with an explicit selection. Values below 1 are clamped
    /// to 1.
    pub fn with_selected(selected: u32) -> Self {
        Self {
            selected: selected.max(1),
        }
    }

    /// Navigates to `page`, clamped into `1..=count`. Returns `true` when the
    /// selection changed.
    pub fn navigate_to(&mut self, page: u32, count: u32) -> bool {
        let clamped = page.clamp(1, count.max(1));
        if clamped == self.selected {
            return false;
        }
        self.selected = clamped;
        true
    }

    /// Advances one page, clamped at `count`. Returns `true` when the
    /// selection changed.
    pub fn next(&mut self, count: u32) -> bool {
        self.navigate_to(self.selected.saturating_add(1), count)
    }

    /// Moves back one page, clamped at `1`. Returns `true` when the
    /// selection changed.
    pub fn previous(&mut self, count: u32) -> bool {
        self.navigate_to(self.selected.saturating_sub(1).max(1), count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_starts_at_one() {
        assert_eq!(PaginationState::new().selected, 1);
    }

    #[test]
    fn with_selected_clamps_to_one() {
        assert_eq!(PaginationState::with_selected(0).selected, 1);
        assert_eq!(PaginationState::with_selected(5).selected, 5);
    }

    #[test]
    fn navigate_to_clamps() {
        let mut s = PaginationState::new();
        assert!(s.navigate_to(5, 10));
        assert_eq!(s.selected, 5);
        assert!(!s.navigate_to(5, 10), "same page → no change");
        assert!(s.navigate_to(99, 10));
        assert_eq!(s.selected, 10);
        assert!(s.navigate_to(0, 10));
        assert_eq!(s.selected, 1);
    }

    #[test]
    fn next_clamps_at_count() {
        let mut s = PaginationState::with_selected(9);
        assert!(s.next(10));
        assert_eq!(s.selected, 10);
        assert!(!s.next(10), "already at last");
    }

    #[test]
    fn previous_clamps_at_one() {
        let mut s = PaginationState::with_selected(2);
        assert!(s.previous(10));
        assert_eq!(s.selected, 1);
        assert!(!s.previous(10), "already at first");
    }

    #[test]
    fn count_zero_treated_as_one() {
        let mut s = PaginationState::with_selected(5);
        assert!(s.navigate_to(1, 0));
        assert_eq!(s.selected, 1);
        s.selected = 5;
        assert!(s.navigate_to(99, 0));
        assert_eq!(s.selected, 1);
    }
}