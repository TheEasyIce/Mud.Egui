//! `BackdropMode` — the overlay behind a popover.
//!
//! **Extension**: MudBlazor does not expose a backdrop parameter on
//! `MudPopover` — popovers are backdrop-less by default and dismissal is
//! handled by an outside-click listener. The port exposes three modes so
//! the same primitive can serve popovers (no backdrop) and can be reused by
//! the dialog host later (`Dimmed`).

/// The visual and interaction behaviour of the layer behind a popover.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum BackdropMode {
    /// No backdrop. Clicks outside the popover still dismiss it (if
    /// [`dismiss_on_outside_click`](crate::overlays::popover::MudPopover::dismiss_on_outside_click)
    /// is enabled), and the underlying UI remains interactive.
    #[default]
    None,
    /// A full-screen invisible click-catcher. Clicks anywhere outside the
    /// popover are consumed (dismiss + do not reach the parent) but nothing
    /// is painted.
    Transparent,
    /// A semi-transparent black overlay. `alpha` is in `0.0..=1.0`.
    /// Reserved for the dialog host; not typically used on popovers.
    Dimmed(f32),
}

impl BackdropMode {
    /// Whether a backdrop layer should be painted or captured at all.
    #[inline]
    pub fn is_active(self) -> bool {
        !matches!(self, Self::None)
    }

    /// Whether the backdrop swallows clicks (blocks the parent UI).
    #[inline]
    pub fn captures_clicks(self) -> bool {
        matches!(self, Self::Transparent | Self::Dimmed(_))
    }

    /// The dimming alpha, clamped to `0.0..=1.0`. `0.0` for non-dimmed modes.
    #[inline]
    pub fn alpha(self) -> f32 {
        match self {
            Self::Dimmed(a) => a.clamp(0.0, 1.0),
            _ => 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_none() {
        assert_eq!(BackdropMode::default(), BackdropMode::None);
    }

    #[test]
    fn none_is_inert() {
        assert!(!BackdropMode::None.is_active());
        assert!(!BackdropMode::None.captures_clicks());
        assert_eq!(BackdropMode::None.alpha(), 0.0);
    }

    #[test]
    fn transparent_captures_but_does_not_paint() {
        let mode = BackdropMode::Transparent;
        assert!(mode.is_active());
        assert!(mode.captures_clicks());
        assert_eq!(mode.alpha(), 0.0);
    }

    #[test]
    fn dimmed_captures_and_clamps_alpha() {
        let mode = BackdropMode::Dimmed(0.5);
        assert!(mode.is_active());
        assert!(mode.captures_clicks());
        assert_eq!(mode.alpha(), 0.5);

        assert_eq!(BackdropMode::Dimmed(-1.0).alpha(), 0.0);
        assert_eq!(BackdropMode::Dimmed(2.0).alpha(), 1.0);
    }
}