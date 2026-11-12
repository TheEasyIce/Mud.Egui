//! Caller-owned state for [`MudPopover`](super::MudPopover).
//!
//! This is the egui equivalent of MudBlazor's `MudPopover.Open` parameter,
//! plus a stable `id` used to key the size cache in `egui::Memory`.

use egui::Id;

/// Caller-owned popover state.
///
/// ```ignore
/// struct MyPage {
///     select_popover: PopoverState,
/// }
///
/// impl MyPage {
///     fn new() -> Self {
///         Self {
///             select_popover: PopoverState::new("select-popover"),
///         }
///     }
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PopoverState {
    /// Whether the popover is currently open.
    pub open: bool,
    /// Stable identity for the popover's `egui::Area` and its size cache.
    /// Two `PopoverState`s with the same `id` share a size cache; use a
    /// unique salt per popover.
    pub id: Id,
}

impl PopoverState {
    /// Creates a closed popover state with a stable id derived from
    /// `id_salt`.
    pub fn new(id_salt: impl std::hash::Hash + std::fmt::Debug) -> Self {
        Self {
            open: false,
            id: Id::new(id_salt),
        }
    }

    /// Opens the popover.
    pub fn open(&mut self) {
        self.open = true;
    }

    /// Closes the popover.
    pub fn close(&mut self) {
        self.open = false;
    }

    /// Toggles the open state.
    pub fn toggle(&mut self) {
        self.open = !self.open;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_starts_closed() {
        let s = PopoverState::new("test");
        assert!(!s.open);
    }

    #[test]
    fn open_and_close_set_the_flag() {
        let mut s = PopoverState::new("test");
        s.open();
        assert!(s.open);
        s.close();
        assert!(!s.open);
    }

    #[test]
    fn toggle_flips_the_flag() {
        let mut s = PopoverState::new("test");
        s.toggle();
        assert!(s.open);
        s.toggle();
        assert!(!s.open);
    }

    #[test]
    fn id_is_stable_across_constructions() {
        let a = PopoverState::new("test");
        let b = PopoverState::new("test");
        assert_eq!(a.id, b.id);
    }

    #[test]
    fn different_salts_produce_different_ids() {
        let a = PopoverState::new("a");
        let b = PopoverState::new("b");
        assert_ne!(a.id, b.id);
    }
}