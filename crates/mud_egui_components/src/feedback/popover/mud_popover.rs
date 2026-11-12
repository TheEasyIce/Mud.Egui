//! `MudPopover` — the anchored overlay primitive.
//!
//! Port of `MudPopover.razor.cs` (MudBlazor 8.13.0). This is the primitive
//! that `MudSelect`, `MudAutocomplete`, all five pickers, `MudMenu`, and
//! `MudTooltip` sit on top of. Everything those six components need from an
//! overlay is here: anchor attachment, flip-when-overflowing, backdrop,
//! Escape / outside-click dismissal, z-ordering, size-cache-backed
//! positioning.
//!
//! # MudBlazor parity
//!
//! | MudBlazor parameter | Rust method | Default |
//! |---------------------|-------------|---------|
//! | `AnchorOrigin`      | [`MudPopover::anchor_origin`] | `Bottom` |
//! | `TransformOrigin`   | [`MudPopover::transform_origin`] | `TopLeft` |
//! | `OverflowBehavior`  | [`MudPopover::overflow_behavior`] | `Flip` |
//! | `Paper`             | [`MudPopover::paper`] | `true` |
//! | `Elevation`         | [`MudPopover::elevation`] | `8` |
//! | `Square`            | [`MudPopover::square`] | `false` |
//! | `Fixed`             | *(unused in egui — Areas are always screen-fixed)* | — |
//! | `Open`              | [`PopoverState::open`] | `false` |
//! | `Class`             | [`MudPopover::classes`] | `None` |
//! | `Style`             | *(use `classes`)* | — |
//! | `ChildContent`      | closure to [`MudPopover::show`] | required |
//!
//! ### Extensions (not in MudBlazor)
//!
//! | Extension | Description |
//! |-----------|-------------|
//! | [`backdrop`](MudPopover::backdrop) | `BackdropMode` (default `None`). |
//! | [`dismiss_on_escape`](MudPopover::dismiss_on_escape) | (default `true`). |
//! | [`dismiss_on_outside_click`](MudPopover::dismiss_on_outside_click) | (default `true`). |
//! | [`min_width`](MudPopover::min_width) / [`max_width`](MudPopover::max_width) | Explicit clamps. |
//! | [`match_anchor_width`](MudPopover::match_anchor_width) | Sets `min_width = anchor_rect.width()`. |
//! | [`offset`](MudPopover::offset) | Gap between anchor and popover (default `0.0`). |
//! | [`right_to_left`](MudPopover::right_to_left) | Resolves `Start` / `End` anchors. |
//! | [`size_hint`](MudPopover::size_hint) | Initial size guess used on the first open frame. |
//! | [`PopoverResponse`] | Rich return value with `open` and `response`. |
//!
//! ### Not ported
//!
//! * **`Fixed`** — egui's `Area` is always screen-fixed; the parameter is a
//!   no-op in this port.
//! * **`OverflowBehavior::Shrink`** — needs to know the popover size before
//!   placing it. Deferred to pass 2 (see "Size cache" below for the current
//!   approximation).
//!
//! # Size cache
//!
//! egui is immediate-mode; you cannot know how big the popover will be before
//! you render it, and you cannot know where to put it before you know its
//! size. The port solves this with a **size cache** keyed by
//! [`PopoverState::id`]:
//!
//! 1. **Frame 1** — use [`size_hint`](MudPopover::size_hint) (or
//!    `DEFAULT_SIZE_HINT`) to compute the position.
//! 2. **Rendering** — record the actual popover size into `egui::Memory`.
//! 3. **Frame 2 onward** — use the cached size, which is exact.
//!
//! The one-frame lag on the first open frame is generally invisible.
//!
//! # Text and icons
//!
//! `MudPopover` renders no text or icons itself — the content closure is
//! responsible. It is expected to use `MudText` / `MudIcon` per project
//! convention.

use egui::{
    pos2, vec2, Color32, CornerRadius, Frame, Id, Margin, Order, Pos2, Rect, Response,
    Sense, Ui, Vec2,
};
use mud_egui_bootstrap::bs_style::BsStyle;
use mud_egui_core::base::component_base::ComponentBase;
use mud_egui_core::enums::{
    AnchorOrigin, BackdropMode, OverflowBehavior, StyleFlavor, TransformOrigin,
};
use mud_egui_tailwind::tw_style::{TwState, TwStyle};
use mud_egui_theming::theme::MudTheme;

use super::popover_response::PopoverResponse;
use super::popover_state::PopoverState;

// =============================================================================
// Constants
// =============================================================================

/// Base rem size passed to the Tailwind resolver.
const REM_SIZE_PT: f32 = 14.0;

/// Minimum gap between the popover and the screen edge when clamping / flipping.
const SCREEN_MARGIN: f32 = 8.0;

/// Default size guess for the first open frame.
const DEFAULT_SIZE_HINT: Vec2 = Vec2::new(200.0, 200.0);

/// Corner radius of the popover's paper background.
const PAPER_RADIUS: u8 = 4;

/// Inner padding of the popover's paper background.
const PAPER_PADDING: i8 = 4;

// =============================================================================
// Pure geometry
// =============================================================================

/// The point on `anchor_rect` where the popover attaches, given a resolved
/// physical anchor edge.
#[inline]
pub fn anchor_point(anchor_rect: Rect, anchor: AnchorOrigin) -> Pos2 {
    match anchor {
        AnchorOrigin::Top => anchor_rect.center_top(),
        AnchorOrigin::Bottom => anchor_rect.center_bottom(),
        AnchorOrigin::Left => anchor_rect.left_center(),
        AnchorOrigin::Right => anchor_rect.right_center(),
        _ => anchor_rect.center_bottom(),
    }
}

/// The offset applied to the popover so its `transform_origin` corner sits
/// at the anchor point.
#[inline]
pub fn transform_offset(transform: TransformOrigin, size: Vec2) -> Vec2 {
    match transform {
        TransformOrigin::TopLeft => Vec2::ZERO,
        TransformOrigin::TopCenter => vec2(-size.x * 0.5, 0.0),
        TransformOrigin::TopRight => vec2(-size.x, 0.0),
        TransformOrigin::CenterLeft => vec2(0.0, -size.y * 0.5),
        TransformOrigin::Center => vec2(-size.x * 0.5, -size.y * 0.5),
        TransformOrigin::CenterRight => vec2(-size.x, -size.y * 0.5),
        TransformOrigin::BottomLeft => vec2(0.0, -size.y),
        TransformOrigin::BottomCenter => vec2(-size.x * 0.5, -size.y),
        TransformOrigin::BottomRight => vec2(-size.x, -size.y),
    }
}

/// The direction the popover is offset from the anchor, scaled by `offset`.
#[inline]
pub fn offset_vec(anchor: AnchorOrigin, offset: f32) -> Vec2 {
    match anchor {
        AnchorOrigin::Top => vec2(0.0, -offset),
        AnchorOrigin::Bottom => vec2(0.0, offset),
        AnchorOrigin::Left => vec2(-offset, 0.0),
        AnchorOrigin::Right => vec2(offset, 0.0),
        _ => Vec2::ZERO,
    }
}

/// Computes the popover's top-left position given all positioning inputs.
pub fn compute_position(
    anchor_rect: Rect,
    anchor: AnchorOrigin,
    transform: TransformOrigin,
    size: Vec2,
    offset: f32,
) -> Pos2 {
    let attachment = anchor_point(anchor_rect, anchor);
    attachment + offset_vec(anchor, offset) + transform_offset(transform, size)
}

/// Tries to flip `(anchor, transform)` vertically.
pub fn flip_vertical(
    anchor: AnchorOrigin,
    transform: TransformOrigin,
) -> Option<(AnchorOrigin, TransformOrigin)> {
    let a = anchor.flip_vertical()?;
    let t = transform.flip_vertical()?;
    Some((a, t))
}

/// Tries to flip `(anchor, transform)` horizontally.
pub fn flip_horizontal(
    anchor: AnchorOrigin,
    transform: TransformOrigin,
) -> Option<(AnchorOrigin, TransformOrigin)> {
    let a = anchor.flip_horizontal()?;
    let t = transform.flip_horizontal()?;
    Some((a, t))
}

/// The result of overflow resolution.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OverflowResult {
    /// Final top-left position of the popover.
    pub pos: Pos2,
    /// Final (possibly flipped) anchor edge.
    pub anchor: AnchorOrigin,
    /// Final (possibly flipped) transform origin.
    pub transform: TransformOrigin,
}

/// Resolves overflow by flipping (when the behavior allows) and clamping.
pub fn resolve_overflow(
    anchor_rect: Rect,
    anchor: AnchorOrigin,
    transform: TransformOrigin,
    size: Vec2,
    offset: f32,
    screen: Rect,
    behavior: OverflowBehavior,
) -> OverflowResult {
    let initial = compute_position(anchor_rect, anchor, transform, size, offset);
    let initial_rect = Rect::from_min_size(initial, size);

    let fits = |r: Rect| {
        r.left() >= screen.left() + SCREEN_MARGIN
            && r.right() <= screen.right() - SCREEN_MARGIN
            && r.top() >= screen.top() + SCREEN_MARGIN
            && r.bottom() <= screen.bottom() - SCREEN_MARGIN
    };

    if fits(initial_rect) {
        return OverflowResult { pos: initial, anchor, transform };
    }

    let mut final_anchor = anchor;
    let mut final_transform = transform;
    let mut final_pos = initial;

    if behavior.tries_flip() {
        let v_overflow = initial_rect.top() < screen.top() + SCREEN_MARGIN
            || initial_rect.bottom() > screen.bottom() - SCREEN_MARGIN;
        if v_overflow {
            if let Some((a, t)) = flip_vertical(anchor, transform) {
                let p = compute_position(anchor_rect, a, t, size, offset);
                if fits(Rect::from_min_size(p, size)) {
                    return OverflowResult { pos: p, anchor: a, transform: t };
                }
                final_anchor = a;
                final_transform = t;
                final_pos = p;
            }
        }

        let h_overflow = final_pos.x < screen.left() + SCREEN_MARGIN
            || final_pos.x + size.x > screen.right() - SCREEN_MARGIN;
        if h_overflow {
            if let Some((a, t)) = flip_horizontal(final_anchor, final_transform) {
                let p = compute_position(anchor_rect, a, t, size, offset);
                if fits(Rect::from_min_size(p, size)) {
                    return OverflowResult { pos: p, anchor: a, transform: t };
                }
                final_anchor = a;
                final_transform = t;
                final_pos = p;
            }
        }
    }

    let max_x = (screen.right() - SCREEN_MARGIN - size.x).max(screen.left() + SCREEN_MARGIN);
    let max_y = (screen.bottom() - SCREEN_MARGIN - size.y).max(screen.top() + SCREEN_MARGIN);
    let clamped = pos2(
        final_pos.x.max(screen.left() + SCREEN_MARGIN).min(max_x),
        final_pos.y.max(screen.top() + SCREEN_MARGIN).min(max_y),
    );

    OverflowResult {
        pos: clamped,
        anchor: final_anchor,
        transform: final_transform,
    }
}

#[inline]
fn elevation_shadow(elevation: u8) -> egui::epaint::Shadow {
    let e = elevation.min(24) as f32;
    egui::epaint::Shadow {
        offset: [0, (e * 0.4) as i8],
        blur: (e * 1.5) as u8,
        spread: 0,
        color: Color32::from_black_alpha(((e * 3.0) as u8).min(60)),
    }
}

// =============================================================================
// MudPopover
// =============================================================================

/// The anchored overlay primitive.
#[must_use = "MudPopover does nothing until you call `.show(ui, state, anchor_rect, ..)`"]
pub struct MudPopover {
    // ── MudBlazor parameters ───────────────────────────────────────────────
    pub anchor_origin: AnchorOrigin,
    pub transform_origin: TransformOrigin,
    pub overflow_behavior: OverflowBehavior,
    pub paper: bool,
    pub elevation: u8,
    pub square: bool,

    // ── Extensions ─────────────────────────────────────────────────────────
    pub backdrop: BackdropMode,
    pub dismiss_on_escape: bool,
    pub dismiss_on_outside_click: bool,
    pub offset: f32,
    pub min_width: Option<f32>,
    pub max_width: Option<f32>,
    pub match_anchor_width: bool,
    pub right_to_left: bool,
    pub size_hint: Option<Vec2>,

    // ── Utilities ──────────────────────────────────────────────────────────
    pub base: ComponentBase,
    pub flavor: StyleFlavor,
    pub bs_style: BsStyle,
    pub tw_style: TwStyle,
    pub tw_state: TwState,
}

impl Default for MudPopover {
    fn default() -> Self {
        Self {
            anchor_origin: AnchorOrigin::Bottom,
            transform_origin: TransformOrigin::TopLeft,
            overflow_behavior: OverflowBehavior::Flip,
            paper: true,
            elevation: 8,
            square: false,
            backdrop: BackdropMode::None,
            dismiss_on_escape: true,
            dismiss_on_outside_click: true,
            offset: 0.0,
            min_width: None,
            max_width: None,
            match_anchor_width: false,
            right_to_left: false,
            size_hint: None,
            base: ComponentBase::new(),
            flavor: StyleFlavor::default(),
            bs_style: BsStyle::default(),
            tw_style: TwStyle::default(),
            tw_state: TwState::default(),
        }
    }
}

impl MudPopover {
    /// Creates a popover with MudBlazor defaults.
    pub fn new() -> Self {
        Self::default()
    }

    // ── Builders ───────────────────────────────────────────────────────────

    #[inline] pub fn anchor_origin(mut self, a: AnchorOrigin) -> Self { self.anchor_origin = a; self }
    #[inline] pub fn transform_origin(mut self, t: TransformOrigin) -> Self { self.transform_origin = t; self }
    #[inline] pub fn overflow_behavior(mut self, b: OverflowBehavior) -> Self { self.overflow_behavior = b; self }
    #[inline] pub fn paper(mut self, b: bool) -> Self { self.paper = b; self }
    #[inline] pub fn elevation(mut self, e: u8) -> Self { self.elevation = e; self }
    #[inline] pub fn square(mut self, b: bool) -> Self { self.square = b; self }
    #[inline] pub fn backdrop(mut self, b: BackdropMode) -> Self { self.backdrop = b; self }
    #[inline] pub fn dismiss_on_escape(mut self, b: bool) -> Self { self.dismiss_on_escape = b; self }
    #[inline] pub fn dismiss_on_outside_click(mut self, b: bool) -> Self { self.dismiss_on_outside_click = b; self }
    #[inline] pub fn offset(mut self, px: f32) -> Self { self.offset = px; self }
    #[inline] pub fn min_width(mut self, px: f32) -> Self { self.min_width = Some(px); self }
    #[inline] pub fn max_width(mut self, px: f32) -> Self { self.max_width = Some(px); self }
    #[inline] pub fn match_anchor_width(mut self, b: bool) -> Self { self.match_anchor_width = b; self }
    #[inline] pub fn right_to_left(mut self, b: bool) -> Self { self.right_to_left = b; self }
    #[inline] pub fn size_hint(mut self, s: Vec2) -> Self { self.size_hint = Some(s); self }
    #[inline] pub fn classes<S: AsRef<str>>(mut self, classes: impl IntoIterator<Item = S>) -> Self { self.base = self.base.classes(classes); self }
    #[inline] pub fn flavor(mut self, f: StyleFlavor) -> Self { self.flavor = f; self }

    // ── Rendering ──────────────────────────────────────────────────────────

    /// Draws the popover if `state.open` is `true`.
    ///
    /// Returns `(PopoverResponse, Option<R>)`. The inner `R` is `Some` only
    /// when the popover is open.
    pub fn show<R>(
        mut self,
        ui: &mut Ui,
        state: &mut PopoverState,
        anchor_rect: Rect,
        content: impl FnOnce(&mut Ui) -> R,
    ) -> (PopoverResponse, Option<R>) {
        // 1. Early exit when closed.
        if !state.open {
            let response = ui.allocate_response(Vec2::ZERO, Sense::hover());
            return (PopoverResponse { open: false, response }, None);
        }

        // 2. Escape dismissal.
        let escape_pressed = self.dismiss_on_escape
            && ui.ctx().input(|i| i.key_pressed(egui::Key::Escape));
        if escape_pressed {
            state.open = false;
            let response = ui.allocate_response(Vec2::ZERO, Sense::hover());
            return (PopoverResponse { open: false, response }, None);
        }

        // 3. Resolve geometry.
        //
        // egui 0.37 removed `Context::screen_rect()`; read it from the input.
        let screen = ui.ctx().input(|i| i.content_rect());
        let resolved_anchor = self.anchor_origin.resolve(self.right_to_left);
        let cached_size = read_cached_size(ui.ctx(), state.id);
        let size_hint = self.size_hint.unwrap_or(DEFAULT_SIZE_HINT);
        let size_for_layout = cached_size.unwrap_or(size_hint);
        let overflow = resolve_overflow(
            anchor_rect,
            resolved_anchor,
            self.transform_origin,
            size_for_layout,
            self.offset,
            screen,
            self.overflow_behavior,
        );

        // 4. Backdrop (rendered first, so it sits behind the popover).
        let mut backdrop_clicked = false;
        if self.backdrop.is_active() {
            let alpha = self.backdrop.alpha();
            let backdrop_area = egui::Area::new(state.id.with("backdrop"))
                .order(Order::Middle)
                .fixed_pos(screen.min)
                .interactable(true);
            // The closure must return the click response so we can inspect
            // it; the previous version returned `()` and the follow-up
            // `.inner.response` accessed a field on `()`.
            let backdrop_inner = backdrop_area.show(ui.ctx(), |ui| {
                let resp = ui.allocate_rect(screen, Sense::click());
                if alpha > 0.0 {
                    ui.painter().rect_filled(
                        screen,
                        CornerRadius::ZERO,
                        Color32::from_black_alpha((alpha * 255.0).round() as u8),
                    );
                }
                resp
            });
            if backdrop_inner.inner.clicked() {
                backdrop_clicked = true;
            }
        }

        // 5. Resolve paper frame.
        let theme = MudTheme::fetch(ui.ctx()).unwrap_or_default();
        let palette = theme.current_palette();
        let lookup = |key: &str| palette.resolve(key).map(Color32::from);
        self.tw_state.dark = ui.visuals().dark_mode;
        let mut frame = match self.flavor {
            StyleFlavor::Bootstrap => self.bs_style.build_frame(&self.base, &lookup),
            StyleFlavor::Tailwind => self
                .tw_style
                .build_frame(&self.base, self.tw_state, REM_SIZE_PT, &lookup),
        };
        if self.paper {
            if frame.fill.a() == 0 {
                frame = frame.fill(palette.surface.into());
            }
            if self.elevation > 0 {
                frame = frame.shadow(elevation_shadow(self.elevation));
            }
            if !self.square {
                frame = frame.corner_radius(CornerRadius::same(PAPER_RADIUS));
            }
            frame = frame.inner_margin(Margin::same(PAPER_PADDING));
        }

        // 6. Resolve width clamp from the anchor.
        let effective_min_width = self
            .min_width
            .or(if self.match_anchor_width { Some(anchor_rect.width()) } else { None });
        let max_width = self.max_width;

        // 7. Render into a Foreground Area.
        let popover_area = egui::Area::new(state.id)
            .order(Order::Foreground)
            .fixed_pos(overflow.pos)
            .interactable(true);

        let inner = popover_area.show(ui.ctx(), |ui| {
            if let Some(mw) = effective_min_width {
                ui.set_min_width(mw);
            }
            if let Some(mw) = max_width {
                ui.set_max_width(mw);
            }
            frame.show(ui, content).inner
        });

        // 8. Cache the actual size for the next frame.
        let actual_size = inner.response.rect.size();
        write_cached_size(ui.ctx(), state.id, actual_size);

        // 9. Outside-click dismissal (when no backdrop consumes it).
        if self.dismiss_on_outside_click && !self.backdrop.captures_clicks() {
            let popover_rect = inner.response.rect;
            let clicked_outside = ui.ctx().input(|i| {
                i.pointer.any_click()
                    && i.pointer
                    .interact_pos()
                    .map(|p| !popover_rect.contains(p))
                    .unwrap_or(false)
            });
            if clicked_outside {
                backdrop_clicked = true;
            }
        }

        if backdrop_clicked {
            state.open = false;
        }

        let open = state.open;
        (
            PopoverResponse {
                open,
                response: inner.response,
            },
            Some(inner.inner),
        )
    }
}

// =============================================================================
// Size cache helpers
// =============================================================================

/// Reads the cached popover size from `egui::Memory`.
fn read_cached_size(ctx: &egui::Context, id: Id) -> Option<Vec2> {
    ctx.memory(|m| m.data.get_temp::<Vec2>(id.with("popover_size")))
}

/// Writes the actual popover size to `egui::Memory`.
fn write_cached_size(ctx: &egui::Context, id: Id, size: Vec2) {
    ctx.memory_mut(|m| m.data.insert_temp(id.with("popover_size"), size));
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ── anchor_point ───────────────────────────────────────────────────────

    fn rect() -> Rect {
        Rect::from_min_size(pos2(100.0, 100.0), vec2(80.0, 40.0))
    }

    #[test]
    fn anchor_point_top_is_top_center() {
        assert_eq!(anchor_point(rect(), AnchorOrigin::Top), pos2(140.0, 100.0));
    }

    #[test]
    fn anchor_point_bottom_is_bottom_center() {
        assert_eq!(anchor_point(rect(), AnchorOrigin::Bottom), pos2(140.0, 140.0));
    }

    #[test]
    fn anchor_point_left_is_left_center() {
        assert_eq!(anchor_point(rect(), AnchorOrigin::Left), pos2(100.0, 120.0));
    }

    #[test]
    fn anchor_point_right_is_right_center() {
        assert_eq!(anchor_point(rect(), AnchorOrigin::Right), pos2(180.0, 120.0));
    }

    // ── transform_offset ───────────────────────────────────────────────────

    fn size() -> Vec2 {
        vec2(200.0, 100.0)
    }

    #[test]
    fn transform_offset_top_left_is_zero() {
        assert_eq!(transform_offset(TransformOrigin::TopLeft, size()), Vec2::ZERO);
    }

    #[test]
    fn transform_offset_top_center_shifts_left_half_width() {
        assert_eq!(
            transform_offset(TransformOrigin::TopCenter, size()),
            vec2(-100.0, 0.0)
        );
    }

    #[test]
    fn transform_offset_bottom_right_shifts_by_full_size() {
        assert_eq!(
            transform_offset(TransformOrigin::BottomRight, size()),
            vec2(-200.0, -100.0)
        );
    }

    #[test]
    fn transform_offset_center_shifts_by_half_size() {
        assert_eq!(
            transform_offset(TransformOrigin::Center, size()),
            vec2(-100.0, -50.0)
        );
    }

    // ── offset_vec ─────────────────────────────────────────────────────────

    #[test]
    fn offset_vec_pushes_away_from_anchor() {
        assert_eq!(offset_vec(AnchorOrigin::Top, 4.0), vec2(0.0, -4.0));
        assert_eq!(offset_vec(AnchorOrigin::Bottom, 4.0), vec2(0.0, 4.0));
        assert_eq!(offset_vec(AnchorOrigin::Left, 4.0), vec2(-4.0, 0.0));
        assert_eq!(offset_vec(AnchorOrigin::Right, 4.0), vec2(4.0, 0.0));
    }

    // ── compute_position ───────────────────────────────────────────────────

    #[test]
    fn dropdown_below_left_aligned() {
        let pos = compute_position(
            rect(),
            AnchorOrigin::Bottom,
            TransformOrigin::TopLeft,
            size(),
            0.0,
        );
        assert_eq!(pos, pos2(140.0, 140.0));
    }

    #[test]
    fn dropdown_below_centered() {
        let pos = compute_position(
            rect(),
            AnchorOrigin::Bottom,
            TransformOrigin::TopCenter,
            size(),
            0.0,
        );
        assert_eq!(pos, pos2(40.0, 140.0));
    }

    #[test]
    fn tooltip_above_centered() {
        let pos = compute_position(
            rect(),
            AnchorOrigin::Top,
            TransformOrigin::BottomCenter,
            size(),
            0.0,
        );
        assert_eq!(pos, pos2(40.0, 0.0));
    }

    #[test]
    fn submenu_to_right_vertically_centered() {
        let pos = compute_position(
            rect(),
            AnchorOrigin::Right,
            TransformOrigin::CenterLeft,
            size(),
            0.0,
        );
        assert_eq!(pos, pos2(180.0, 70.0));
    }

    #[test]
    fn offset_moves_position() {
        let pos = compute_position(
            rect(),
            AnchorOrigin::Bottom,
            TransformOrigin::TopLeft,
            size(),
            8.0,
        );
        assert_eq!(pos, pos2(140.0, 148.0));
    }

    // ── flip helpers ───────────────────────────────────────────────────────

    #[test]
    fn flip_vertical_bottom_to_top() {
        assert_eq!(
            flip_vertical(AnchorOrigin::Bottom, TransformOrigin::TopLeft),
            Some((AnchorOrigin::Top, TransformOrigin::BottomLeft))
        );
    }

    #[test]
    fn flip_vertical_top_to_bottom() {
        assert_eq!(
            flip_vertical(AnchorOrigin::Top, TransformOrigin::BottomCenter),
            Some((AnchorOrigin::Bottom, TransformOrigin::TopCenter))
        );
    }

    #[test]
    fn flip_vertical_returns_none_for_horizontal_anchor() {
        assert_eq!(
            flip_vertical(AnchorOrigin::Left, TransformOrigin::CenterRight),
            None
        );
    }

    #[test]
    fn flip_vertical_returns_none_for_centered_transform() {
        assert_eq!(
            flip_vertical(AnchorOrigin::Bottom, TransformOrigin::CenterLeft),
            None
        );
    }

    #[test]
    fn flip_horizontal_right_to_left() {
        assert_eq!(
            flip_horizontal(AnchorOrigin::Right, TransformOrigin::CenterLeft),
            Some((AnchorOrigin::Left, TransformOrigin::CenterRight))
        );
    }

    #[test]
    fn flip_horizontal_returns_none_for_vertical_anchor() {
        assert_eq!(
            flip_horizontal(AnchorOrigin::Top, TransformOrigin::TopLeft),
            None
        );
    }

    // ── resolve_overflow ───────────────────────────────────────────────────

    fn screen() -> Rect {
        Rect::from_min_size(pos2(0.0, 0.0), vec2(1000.0, 800.0))
    }

    #[test]
    fn overflow_fits_returns_initial() {
        let anchor = Rect::from_min_size(pos2(400.0, 400.0), vec2(80.0, 40.0));
        let result = resolve_overflow(
            anchor,
            AnchorOrigin::Bottom,
            TransformOrigin::TopLeft,
            vec2(200.0, 100.0),
            0.0,
            screen(),
            OverflowBehavior::Flip,
        );
        assert_eq!(result.pos, pos2(440.0, 440.0));
        assert_eq!(result.anchor, AnchorOrigin::Bottom);
        assert_eq!(result.transform, TransformOrigin::TopLeft);
    }

    #[test]
    fn overflow_below_flips_above() {
        let anchor = Rect::from_min_size(pos2(400.0, 750.0), vec2(80.0, 40.0));
        let result = resolve_overflow(
            anchor,
            AnchorOrigin::Bottom,
            TransformOrigin::TopLeft,
            vec2(200.0, 100.0),
            0.0,
            screen(),
            OverflowBehavior::Flip,
        );
        assert_eq!(result.anchor, AnchorOrigin::Top);
        assert_eq!(result.transform, TransformOrigin::BottomLeft);
        assert_eq!(result.pos, pos2(440.0, 650.0));
    }

    #[test]
    fn overflow_reposition_clamps_without_flipping() {
        let anchor = Rect::from_min_size(pos2(400.0, 750.0), vec2(80.0, 40.0));
        let result = resolve_overflow(
            anchor,
            AnchorOrigin::Bottom,
            TransformOrigin::TopLeft,
            vec2(200.0, 100.0),
            0.0,
            screen(),
            OverflowBehavior::Reposition,
        );
        assert_eq!(result.anchor, AnchorOrigin::Bottom);
        assert_eq!(result.transform, TransformOrigin::TopLeft);
        assert_eq!(result.pos.y, 692.0);
    }

    #[test]
    fn overflow_horizontal_flip_when_anchored_right() {
        let anchor = Rect::from_min_size(pos2(950.0, 400.0), vec2(40.0, 40.0));
        let result = resolve_overflow(
            anchor,
            AnchorOrigin::Right,
            TransformOrigin::CenterLeft,
            vec2(200.0, 100.0),
            0.0,
            screen(),
            OverflowBehavior::Flip,
        );
        assert_eq!(result.anchor, AnchorOrigin::Left);
        assert_eq!(result.transform, TransformOrigin::CenterRight);
        assert_eq!(result.pos, pos2(750.0, 370.0));
    }

    #[test]
    fn overflow_clamps_when_flip_cannot_help() {
        let anchor = Rect::from_min_size(pos2(400.0, 400.0), vec2(80.0, 40.0));
        let result = resolve_overflow(
            anchor,
            AnchorOrigin::Bottom,
            TransformOrigin::TopLeft,
            vec2(1200.0, 900.0),
            0.0,
            screen(),
            OverflowBehavior::Flip,
        );
        assert_eq!(result.pos, pos2(8.0, 8.0));
    }

    // ── Defaults ───────────────────────────────────────────────────────────

    #[test]
    fn defaults_match_mudblazor() {
        let p: MudPopover = MudPopover::new();
        assert_eq!(p.anchor_origin, AnchorOrigin::Bottom);
        assert_eq!(p.transform_origin, TransformOrigin::TopLeft);
        assert_eq!(p.overflow_behavior, OverflowBehavior::Flip);
        assert!(p.paper);
        assert_eq!(p.elevation, 8);
        assert!(!p.square);
        assert_eq!(p.backdrop, BackdropMode::None);
        assert!(p.dismiss_on_escape);
        assert!(p.dismiss_on_outside_click);
        assert_eq!(p.offset, 0.0);
        assert!(p.min_width.is_none());
        assert!(p.max_width.is_none());
        assert!(!p.match_anchor_width);
        assert!(!p.right_to_left);
        assert!(p.size_hint.is_none());
    }

    // ── Builders ───────────────────────────────────────────────────────────

    #[test]
    fn builders_set_every_field() {
        let p: MudPopover = MudPopover::new()
            .anchor_origin(AnchorOrigin::Right)
            .transform_origin(TransformOrigin::CenterLeft)
            .overflow_behavior(OverflowBehavior::Reposition)
            .paper(false)
            .elevation(4)
            .square(true)
            .backdrop(BackdropMode::Dimmed(0.5))
            .dismiss_on_escape(false)
            .dismiss_on_outside_click(false)
            .offset(8.0)
            .min_width(200.0)
            .max_width(400.0)
            .match_anchor_width(true)
            .right_to_left(true)
            .size_hint(vec2(300.0, 250.0))
            .classes(["my-popover"]);

        assert_eq!(p.anchor_origin, AnchorOrigin::Right);
        assert_eq!(p.transform_origin, TransformOrigin::CenterLeft);
        assert_eq!(p.overflow_behavior, OverflowBehavior::Reposition);
        assert!(!p.paper);
        assert_eq!(p.elevation, 4);
        assert!(p.square);
        assert_eq!(p.backdrop, BackdropMode::Dimmed(0.5));
        assert!(!p.dismiss_on_escape);
        assert!(!p.dismiss_on_outside_click);
        assert_eq!(p.offset, 8.0);
        assert_eq!(p.min_width, Some(200.0));
        assert_eq!(p.max_width, Some(400.0));
        assert!(p.match_anchor_width);
        assert!(p.right_to_left);
        assert_eq!(p.size_hint, Some(vec2(300.0, 250.0)));
        assert!(p.base.has_class("my-popover"));
    }

    // ── elevation_shadow (sanity) ──────────────────────────────────────────

    #[test]
    fn elevation_shadow_zero_is_empty() {
        let s = elevation_shadow(0);
        assert_eq!(s.blur, 0);
        assert_eq!(s.color.a(), 0);
    }

    #[test]
    fn elevation_shadow_clamps_above_24() {
        let a = elevation_shadow(24);
        let b = elevation_shadow(200);
        assert_eq!(a.offset, b.offset);
        assert_eq!(a.blur, b.blur);
    }
}