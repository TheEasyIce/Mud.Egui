//! Caller-owned state for [`MudTooltip`](super::MudTooltip).
//!
//! The tooltip has four timing concerns that must persist across frames:
//! * the hover-delay timer (MudBlazor's `Delay`),
//! * the fade-in animation start,
//! * the fade-out animation start,
//! * the click-toggle latch (for `ShowOnClick`).
//!
//! All four live here. The underlying [`PopoverState`] is also stored so the
//! `MudPopover` area has a stable id across frames.

use egui::Id;

use crate::feedback::popover::PopoverState;

/// Caller-owned state for a tooltip.
///
/// ```ignore
/// struct MyPage {
///     tip: TooltipState,
/// }
///
/// impl MyPage {
///     fn new() -> Self {
///         Self { tip: TooltipState::new("save-tip") }
///     }
/// }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct TooltipState {
    /// Logical visibility. Written by [`MudTooltip::show`](super::MudTooltip::show)
    /// when a trigger fires; readable by the caller. Fires
    /// [`on_visible_changed`](super::MudTooltip::on_visible_changed) on
    /// transition.
    pub visible: bool,
    /// Underlying popover state (id, size cache, and open flag).
    pub popover: PopoverState,
    /// When the pointer first entered the trigger. `None` when not hovering.
    pub hover_started_at: Option<f64>,
    /// When the tooltip began fading in.
    pub show_started_at: Option<f64>,
    /// When the tooltip began fading out.
    pub hide_started_at: Option<f64>,
    /// Click-toggle latch (`ShowOnClick`). Flips on every trigger click.
    pub click_open: bool,
}

impl TooltipState {
    /// Creates a hidden state with a stable popover id derived from `id_salt`.
    pub fn new(id_salt: impl std::hash::Hash + std::fmt::Debug) -> Self {
        Self {
            visible: false,
            popover: PopoverState::new(id_salt),
            hover_started_at: None,
            show_started_at: None,
            hide_started_at: None,
            click_open: false,
        }
    }

    /// Whether the tooltip is logically visible.
    #[inline]
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Forces the tooltip visible starting at `now`.
    pub fn show(&mut self, now: f64) {
        if self.visible {
            return;
        }
        self.visible = true;
        self.show_started_at = Some(now);
        self.hide_started_at = None;
    }

    /// Forces the tooltip hidden starting at `now`.
    pub fn hide(&mut self, now: f64) {
        if !self.visible {
            return;
        }
        self.visible = false;
        self.hide_started_at = Some(now);
    }

    /// Flips visibility at `now`.
    pub fn toggle(&mut self, now: f64) {
        if self.visible {
            self.hide(now);
        } else {
            self.show(now);
        }
    }

    /// Resets every animation timer. Called by the component once opacity
    /// reaches zero on a hide animation, so the next `show` starts cleanly.
    pub(crate) fn reset_animation(&mut self) {
        self.show_started_at = None;
        self.hide_started_at = None;
        self.popover.open = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_starts_hidden() {
        let s = TooltipState::new("t");
        assert!(!s.is_visible());
        assert!(s.hover_started_at.is_none());
        assert!(s.show_started_at.is_none());
        assert!(s.hide_started_at.is_none());
        assert!(!s.click_open);
    }

    #[test]
    fn show_then_hide_tracks_timers() {
        let mut s = TooltipState::new("t");
        s.show(1.0);
        assert!(s.is_visible());
        assert_eq!(s.show_started_at, Some(1.0));
        assert!(s.hide_started_at.is_none());
        s.hide(2.0);
        assert!(!s.is_visible());
        assert_eq!(s.hide_started_at, Some(2.0));
    }

    #[test]
    fn show_is_idempotent() {
        let mut s = TooltipState::new("t");
        s.show(1.0);
        s.show(2.0);
        assert_eq!(s.show_started_at, Some(1.0), "should not reset timer");
    }

    #[test]
    fn hide_is_idempotent() {
        let mut s = TooltipState::new("t");
        s.show(1.0);
        s.hide(2.0);
        s.hide(3.0);
        assert_eq!(s.hide_started_at, Some(2.0));
    }

    #[test]
    fn toggle_flips() {
        let mut s = TooltipState::new("t");
        s.toggle(1.0);
        assert!(s.is_visible());
        s.toggle(2.0);
        assert!(!s.is_visible());
    }

    #[test]
    fn reset_animation_clears_timers_and_closes_popover() {
        let mut s = TooltipState::new("t");
        s.show(1.0);
        s.popover.open = true;
        s.reset_animation();
        assert!(s.show_started_at.is_none());
        assert!(s.hide_started_at.is_none());
        assert!(!s.popover.open);
    }
}