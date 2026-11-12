//! The resolved behaviour of a drawer at the current width.
//!
//! `DrawerVariant` is the *configured* policy; the effective variant is what
//! the drawer does **this frame** after the breakpoint and the `open` flag are
//! applied. Kept in its own file per Rule 2.

use mud_egui_core::enums::{Breakpoint, DrawerVariant};

/// The behaviour a drawer actually uses this frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EffectiveVariant {
    /// Paint above the content in a foreground `Area`; does not reserve width.
    Overlay,
    /// Paint inline; reserves its width in the parent `Ui`.
    Push,
    /// Paint inline at `mini_width` when closed; expands on hover / open.
    Mini,
}

/// Resolves the effective variant from the configured one, the breakpoint,
/// and the current container width.
///
/// * `Breakpoint::None` — always [`EffectiveVariant::Overlay`].
/// * `Breakpoint::Always` — never [`EffectiveVariant::Overlay`].
/// * Otherwise, `width < breakpoint.to_pixels()` is below the breakpoint.
pub(crate) fn resolve(
    variant: DrawerVariant,
    breakpoint: Breakpoint,
    width: f32,
) -> EffectiveVariant {
    let below = match breakpoint {
        Breakpoint::None => true,
        Breakpoint::Always => false,
        bp => width < bp.to_pixels(),
    };
    match variant {
        DrawerVariant::Temporary => EffectiveVariant::Overlay,
        DrawerVariant::Persistent => EffectiveVariant::Push,
        DrawerVariant::Responsive => {
            if below {
                EffectiveVariant::Overlay
            } else {
                EffectiveVariant::Push
            }
        }
        DrawerVariant::Mini => {
            if below {
                EffectiveVariant::Overlay
            } else {
                EffectiveVariant::Mini
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn temporary_is_always_overlay() {
        for bp in [Breakpoint::None, Breakpoint::Md, Breakpoint::Always] {
            for w in [100.0_f32, 800.0, 2000.0] {
                assert_eq!(
                    resolve(DrawerVariant::Temporary, bp, w),
                    EffectiveVariant::Overlay,
                );
            }
        }
    }

    #[test]
    fn persistent_is_always_push() {
        for bp in [Breakpoint::None, Breakpoint::Md, Breakpoint::Always] {
            for w in [100.0_f32, 800.0, 2000.0] {
                assert_eq!(
                    resolve(DrawerVariant::Persistent, bp, w),
                    EffectiveVariant::Push,
                );
            }
        }
    }

    #[test]
    fn responsive_follows_the_breakpoint() {
        assert_eq!(
            resolve(DrawerVariant::Responsive, Breakpoint::Md, 800.0),
            EffectiveVariant::Overlay,
        );
        assert_eq!(
            resolve(DrawerVariant::Responsive, Breakpoint::Md, 1000.0),
            EffectiveVariant::Push,
        );
    }

    #[test]
    fn mini_follows_the_breakpoint_and_becomes_mini_above_it() {
        assert_eq!(
            resolve(DrawerVariant::Mini, Breakpoint::Md, 800.0),
            EffectiveVariant::Overlay,
        );
        assert_eq!(
            resolve(DrawerVariant::Mini, Breakpoint::Md, 1000.0),
            EffectiveVariant::Mini,
        );
    }

    #[test]
    fn sentinels_override_the_width() {
        assert_eq!(
            resolve(DrawerVariant::Responsive, Breakpoint::None, 5000.0),
            EffectiveVariant::Overlay,
        );
        assert_eq!(
            resolve(DrawerVariant::Responsive, Breakpoint::Always, 0.0),
            EffectiveVariant::Push,
        );
    }
}