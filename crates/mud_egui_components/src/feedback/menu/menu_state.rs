//! Caller-owned state for [`MudMenu`](super::MudMenu).

use crate::feedback::popover::PopoverState;

/// Caller-owned state for a menu.
///
/// Wraps the popover state — the menu's positioning, dismissal, and sizing
/// all live in `MudPopover`, and the menu simply adds the activator +
/// item list. Persist this struct across frames:
///
/// ```ignore
/// struct MyPage { menu: MenuState }
///
/// impl MyPage {
///     fn new() -> Self {
///         Self { menu: MenuState::new("file-menu") }
///     }
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MenuState {
    /// The underlying popover state.
    pub popover: PopoverState,
}

impl MenuState {
    /// Creates a closed menu with a stable id derived from `id_salt`.
    pub fn new(id_salt: impl std::hash::Hash + std::fmt::Debug) -> Self {
        Self {
            popover: PopoverState::new(id_salt),
        }
    }

    /// Whether the menu is currently open.
    #[inline]
    pub fn is_open(&self) -> bool {
        self.popover.open
    }

    /// Opens the menu.
    #[inline]
    pub fn open(&mut self) {
        self.popover.open = true;
    }

    /// Closes the menu.
    #[inline]
    pub fn close(&mut self) {
        self.popover.open = false;
    }

    /// Toggles the menu.
    #[inline]
    pub fn toggle(&mut self) {
        self.popover.open = !self.popover.open;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_starts_closed() {
        let s = MenuState::new("test");
        assert!(!s.is_open());
    }

    #[test]
    fn open_close_toggle_work() {
        let mut s = MenuState::new("test");
        s.open();
        assert!(s.is_open());
        s.close();
        assert!(!s.is_open());
        s.toggle();
        assert!(s.is_open());
    }

    #[test]
    fn id_is_stable_across_constructions() {
        let a = MenuState::new("m");
        let b = MenuState::new("m");
        assert_eq!(a.popover.id, b.popover.id);
    }
}