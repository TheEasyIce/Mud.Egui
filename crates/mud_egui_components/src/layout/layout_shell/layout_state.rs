//! `LayoutState` — the shared measurements a [`MudLayout`](super::MudLayout)
//! shell publishes to its children.
//!
//! Caller-owned and reset at the top of every `MudLayout::show`. Each
//! `MudAppBar` / `MudDrawer` writes its own measurement as it is declared;
//! [`MudMainContent`](super::MudMainContent) reads the totals to decide how
//! much space is left.
//!
//! # Why `shell_height` exists
//!
//! egui's `ui.horizontal(..)` clamps its child `Ui`'s `max_rect.height` to
//! `spacing.interact_size.y` (~20 pt). Inside such a row, `ui.available_height()`
//! is therefore *never* the row's true height — it is always ~20. A drawer or
//! main-content widget that sizes itself from `available_height()` would
//! collapse to that single line.
//!
//! `MudLayout::show` publishes the shell's full screen height as
//! `shell_height`, so a push-mode `MudDrawer` (and `MudMainContent`) can size
//! itself from `available_row_height()` — `shell_height − top_inset() −
//! bottom_inset()` — and allocate a rect tall enough to fill the shell
//! regardless of the row's clamped height.
//!
//! The **declaration order** matters, because `available_row_height()` is
//! computed from the app bar and drawers that have already been declared:
//!
//! * `DrawerClipMode::Docked` / `Always` — app bar first, then the drawer.
//! * `DrawerClipMode::Never` — drawer first, then the app bar.

/// Measurements published by the shell's children.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct LayoutState {
    /// The shell's full screen height, in points. Published by
    /// `MudLayout::show`. `0.0` when the shell has not run — callers that
    /// consult it must fall back to `ui.available_height()` in that case.
    pub shell_height: f32,
    /// Height reserved by a top or bottom [`MudAppBar`](super::MudAppBar).
    pub app_bar_height: f32,
    /// `true` when the app bar is anchored to the bottom edge.
    pub app_bar_at_bottom: bool,
    /// Width reserved by a drawer anchored to the left (or to `Start` in LTR).
    pub start_drawer_width: f32,
    /// Width reserved by a drawer anchored to the right (or to `End` in LTR).
    pub end_drawer_width: f32,
    /// Height reserved by a drawer anchored to the top edge.
    pub top_drawer_height: f32,
    /// Height reserved by a drawer anchored to the bottom edge.
    pub bottom_drawer_height: f32,
    /// `true` when a temporary / responsive-below-breakpoint drawer is open.
    /// [`MudMainContent`](super::MudMainContent) can use this to dim itself.
    pub any_overlay_open: bool,
}

impl LayoutState {
    /// Zeroes every field. Called by `MudLayout::show` at the start of the
    /// frame.
    #[inline]
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Total height the main content loses to the app bar and to top / bottom
    /// drawers.
    #[inline]
    pub fn top_inset(&self) -> f32 {
        let app = if self.app_bar_at_bottom { 0.0 } else { self.app_bar_height };
        app + self.top_drawer_height
    }

    /// Total height the main content loses at the bottom edge.
    #[inline]
    pub fn bottom_inset(&self) -> f32 {
        let app = if self.app_bar_at_bottom { self.app_bar_height } else { 0.0 };
        app + self.bottom_drawer_height
    }

    /// Height available to the row that holds push-mode drawers and
    /// `MudMainContent`.
    ///
    /// Returns `0.0` when the shell has not published its height, so callers
    /// that compare against `ui.available_height()` degrade gracefully to
    /// their previous behaviour.
    #[inline]
    pub fn available_row_height(&self) -> f32 {
        (self.shell_height - self.top_inset() - self.bottom_inset()).max(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_all_zero() {
        let s = LayoutState::default();
        assert_eq!(s.shell_height, 0.0);
        assert_eq!(s.app_bar_height, 0.0);
        assert!(!s.app_bar_at_bottom);
        assert_eq!(s.start_drawer_width, 0.0);
        assert_eq!(s.end_drawer_width, 0.0);
        assert_eq!(s.top_drawer_height, 0.0);
        assert_eq!(s.bottom_drawer_height, 0.0);
        assert!(!s.any_overlay_open);
    }

    #[test]
    fn reset_zeroes_everything() {
        let mut s = LayoutState {
            shell_height: 800.0,
            app_bar_height: 64.0,
            app_bar_at_bottom: true,
            start_drawer_width: 240.0,
            end_drawer_width: 180.0,
            top_drawer_height: 20.0,
            bottom_drawer_height: 30.0,
            any_overlay_open: true,
        };
        s.reset();
        assert_eq!(s, LayoutState::default());
    }

    #[test]
    fn top_inset_counts_a_top_app_bar() {
        let s = LayoutState {
            app_bar_height: 64.0,
            top_drawer_height: 20.0,
            ..Default::default()
        };
        assert_eq!(s.top_inset(), 84.0);
        assert_eq!(s.bottom_inset(), 0.0);
    }

    #[test]
    fn top_inset_ignores_a_bottom_app_bar() {
        let s = LayoutState {
            app_bar_height: 64.0,
            app_bar_at_bottom: true,
            ..Default::default()
        };
        assert_eq!(s.top_inset(), 0.0);
        assert_eq!(s.bottom_inset(), 64.0);
    }

    #[test]
    fn available_row_height_is_zero_without_a_shell() {
        assert_eq!(LayoutState::default().available_row_height(), 0.0);
    }

    #[test]
    fn available_row_height_subtracts_insets() {
        let s = LayoutState {
            shell_height: 800.0,
            app_bar_height: 48.0,
            ..Default::default()
        };
        assert_eq!(s.available_row_height(), 752.0);
    }

    #[test]
    fn available_row_height_with_bottom_app_bar() {
        let s = LayoutState {
            shell_height: 800.0,
            app_bar_height: 64.0,
            app_bar_at_bottom: true,
            ..Default::default()
        };
        assert_eq!(s.available_row_height(), 736.0);
    }

    #[test]
    fn available_row_height_saturates() {
        let s = LayoutState {
            shell_height: 20.0,
            app_bar_height: 48.0,
            ..Default::default()
        };
        assert_eq!(s.available_row_height(), 0.0);
    }
}