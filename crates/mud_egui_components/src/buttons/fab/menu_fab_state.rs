//! Caller-owned state for [`MudMenuFab`](super::MudMenuFab).
//!
//! Wraps a [`PopoverState`] — the open flag, the stable id used by
//! `MudPopover`, and nothing else. The port exposes the state so a caller can
//! open/close the action menu programmatically, the same way
//! `MudDatePicker`, `MudTimePicker`, `MudColorPicker` and `MudDialog` expose
//! theirs.

use crate::feedback::popover::PopoverState;

/// Caller-owned state for a [`MudMenuFab`](super::MudMenuFab).
#[derive(Debug, Clone, Copy)]
pub struct MenuFabState {
    /// The action-menu popover state.
    pub popover: PopoverState,
}

impl MenuFabState {
    /// Creates a closed action menu with a stable id.
    #[inline]
    pub fn new(id_salt: impl std::hash::Hash + std::fmt::Debug) -> Self {
        Self {
            popover: PopoverState::new(id_salt),
        }
    }

    /// Whether the action menu is open.
    #[inline]
    pub fn is_open(&self) -> bool {
        self.popover.open
    }

    /// Opens the action menu.
    #[inline]
    pub fn open(&mut self) {
        self.popover.open = true;
    }

    /// Closes the action menu.
    #[inline]
    pub fn close(&mut self) {
        self.popover.open = false;
    }

    /// Toggles the action menu.
    #[inline]
    pub fn toggle(&mut self) {
        self.popover.toggle();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_starts_closed() {
        assert!(!MenuFabState::new("t").is_open());
    }

    #[test]
    fn mutators_flip_the_flag() {
        let mut s = MenuFabState::new("t");
        s.open();
        assert!(s.is_open());
        s.close();
        assert!(!s.is_open());
        s.toggle();
        assert!(s.is_open());
    }

    #[test]
    fn id_is_stable_across_constructions() {
        assert_eq!(MenuFabState::new("x").popover.id, MenuFabState::new("x").popover.id);
        assert_ne!(MenuFabState::new("x").popover.id, MenuFabState::new("y").popover.id);
    }
}