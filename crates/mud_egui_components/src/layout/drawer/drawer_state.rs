//! Caller-owned state for [`MudDrawer`](super::MudDrawer).

/// Caller-owned drawer state.
///
/// Mirrors MudBlazor's two-way bound `Open` parameter plus the mini variant's
/// hover expansion, which MudBlazor keeps internally.
///
/// **Defaults to open**, matching `MudBlazor.MudDrawer.Open` (whose default is
/// `true`). Use [`closed`](Self::closed) to start a drawer hidden.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DrawerState {
    /// Whether the drawer is open. Mirrors `Open`.
    pub open: bool,
    /// `true` while a `Mini` drawer's pointer is over it and
    /// `OpenMiniOnHover` is enabled. Exposed so a caller can pin it open from
    /// outside.
    pub hover_expanded: bool,
}

impl Default for DrawerState {
    /// `open: true`, matching MudBlazor.
    fn default() -> Self {
        Self { open: true, hover_expanded: false }
    }
}

impl DrawerState {
    /// Creates a drawer with MudBlazor's default state (**open**).
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates an open drawer.
    #[inline]
    pub fn opened() -> Self {
        Self { open: true, hover_expanded: false }
    }

    /// Creates a closed drawer.
    #[inline]
    pub fn closed() -> Self {
        Self { open: false, hover_expanded: false }
    }

    #[inline]
    pub fn is_open(&self) -> bool {
        self.open
    }

    #[inline]
    pub fn open(&mut self) {
        self.open = true;
    }

    #[inline]
    pub fn close(&mut self) {
        self.open = false;
        self.hover_expanded = false;
    }

    #[inline]
    pub fn toggle(&mut self) {
        if self.open { self.close() } else { self.open() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_open() {
        let s = DrawerState::default();
        assert!(s.open);
        assert!(!s.hover_expanded);
    }

    #[test]
    fn closed_starts_closed() {
        assert!(!DrawerState::closed().is_open());
    }

    #[test]
    fn opened_matches_default() {
        assert_eq!(DrawerState::opened(), DrawerState::default());
    }

    #[test]
    fn new_matches_default() {
        assert_eq!(DrawerState::new(), DrawerState::default());
    }

    #[test]
    fn open_close_toggle() {
        let mut s = DrawerState::closed();
        s.open();
        assert!(s.is_open());
        s.close();
        assert!(!s.is_open());
        s.toggle();
        assert!(s.is_open());
        s.toggle();
        assert!(!s.is_open());
    }

    #[test]
    fn close_clears_the_hover_latch() {
        let mut s = DrawerState::new();
        s.hover_expanded = true;
        s.close();
        assert!(!s.hover_expanded);
    }
}