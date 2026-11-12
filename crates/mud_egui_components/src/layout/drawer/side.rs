//! `Side` — the concrete edge a [`MudDrawer`](super::MudDrawer) is anchored
//! to, after `AnchorOrigin` and `right_to_left` are resolved.
//!
//! Port of the internal placement resolution of `MudDrawer.razor.cs`
//! (MudBlazor 8.13.0). `AnchorOrigin::{Start, End}` are logical and flip with
//! RTL; `Side` is the physical edge the drawer paints on.
//!
//! **Private to the crate.** MudBlazor does not expose a `Side` type;
//! `AnchorOrigin` is the public parameter. Kept in its own file per Rule 2
//! (one enum, one file) even though it is `pub(crate)` and internal.

use egui::{pos2, Rect};

use mud_egui_core::enums::AnchorOrigin;

/// The concrete edge a drawer is anchored to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Side {
    /// Left edge in LTR, right edge in RTL.
    Start,
    /// Right edge in LTR, left edge in RTL.
    End,
    /// Top edge.
    Top,
    /// Bottom edge.
    Bottom,
}

impl Side {
    /// `true` for `Start` and `End` — a horizontal drawer.
    #[inline]
    pub(crate) fn is_horizontal(self) -> bool {
        matches!(self, Self::Start | Self::End)
    }
}

/// Resolves `AnchorOrigin` to a concrete [`Side`], honouring `right_to_left`.
pub(crate) fn resolve_side(anchor: AnchorOrigin, right_to_left: bool) -> Side {
    match anchor {
        AnchorOrigin::Left => Side::Start,
        AnchorOrigin::Right => Side::End,
        AnchorOrigin::Start => {
            if right_to_left { Side::End } else { Side::Start }
        }
        AnchorOrigin::End => {
            if right_to_left { Side::Start } else { Side::End }
        }
        AnchorOrigin::Top => Side::Top,
        AnchorOrigin::Bottom => Side::Bottom,
    }
}

/// The rect of a drawer of `extent` on `side`, inside `screen`, starting at
/// `top_offset`.
pub(crate) fn side_rect(screen: Rect, side: Side, extent: f32, top_offset: f32) -> Rect {
    let top = screen.top() + top_offset;
    match side {
        Side::Start => Rect::from_min_max(
            pos2(screen.left(), top),
            pos2(screen.left() + extent, screen.bottom()),
        ),
        Side::End => Rect::from_min_max(
            pos2(screen.right() - extent, top),
            pos2(screen.right(), screen.bottom()),
        ),
        Side::Top => Rect::from_min_max(
            pos2(screen.left(), screen.top()),
            pos2(screen.right(), screen.top() + extent),
        ),
        Side::Bottom => Rect::from_min_max(
            pos2(screen.left(), screen.bottom() - extent),
            pos2(screen.right(), screen.bottom()),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_side_maps_all_anchors() {
        assert_eq!(resolve_side(AnchorOrigin::Left, false), Side::Start);
        assert_eq!(resolve_side(AnchorOrigin::Right, false), Side::End);
        assert_eq!(resolve_side(AnchorOrigin::Start, false), Side::Start);
        assert_eq!(resolve_side(AnchorOrigin::Start, true), Side::End);
        assert_eq!(resolve_side(AnchorOrigin::End, false), Side::End);
        assert_eq!(resolve_side(AnchorOrigin::End, true), Side::Start);
        assert_eq!(resolve_side(AnchorOrigin::Top, false), Side::Top);
        assert_eq!(resolve_side(AnchorOrigin::Bottom, false), Side::Bottom);
    }

    fn screen() -> Rect {
        Rect::from_min_size(pos2(0.0, 0.0), egui::vec2(1000.0, 800.0))
    }

    #[test]
    fn side_rect_start_is_the_left_strip() {
        let r = side_rect(screen(), Side::Start, 240.0, 0.0);
        assert_eq!(
            r,
            Rect::from_min_max(pos2(0.0, 0.0), pos2(240.0, 800.0))
        );
    }

    #[test]
    fn side_rect_end_is_the_right_strip() {
        let r = side_rect(screen(), Side::End, 240.0, 0.0);
        assert_eq!(
            r,
            Rect::from_min_max(pos2(760.0, 0.0), pos2(1000.0, 800.0))
        );
    }

    #[test]
    fn side_rect_honours_top_offset() {
        let r = side_rect(screen(), Side::Start, 240.0, 64.0);
        assert_eq!(r.top(), 64.0);
        assert_eq!(r.bottom(), 800.0);
    }

    #[test]
    fn is_horizontal_only_for_start_end() {
        assert!(Side::Start.is_horizontal());
        assert!(Side::End.is_horizontal());
        assert!(!Side::Top.is_horizontal());
        assert!(!Side::Bottom.is_horizontal());
    }
}