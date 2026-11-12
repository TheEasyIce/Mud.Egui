//! `MudTooltip` — a small popup that shows additional information when the
//! pointer hovers, the child gains focus, or the child is clicked.
//!
//! Port of `MudTooltip.razor.cs` (MudBlazor 8.13.0).
//!
//! # Composition
//!
//! `MudTooltip` is built from [`MudPopover`] — anchor positioning,
//! flip-when-overflowing, and z-ordering are all delegated. Its own additions
//! are the **trigger state machine** (hover delay + focus + click latch),
//! the **fade in/out animation**, and the **arrow indicator**.
//!
//! Unlike `MudSelect` / `MudAutocomplete`, `MudPopover`'s dismissal hooks are
//! disabled: a tooltip is **display-only**, so the component itself decides
//! when to hide (hover leave, blur, or click-toggle). See the `dismiss` note
//! in the README.
//!
//! # MudBlazor parity
//!
//! | MudBlazor parameter | Rust method | Default |
//! |---------------------|-------------|---------|
//! | `Arrow`             | [`MudTooltip::arrow`] | `false` |
//! | `ChildContent`      | the `child` argument of [`MudTooltip::show`] | required |
//! | `Color`             | [`MudTooltip::color`] | [`Color::Default`] |
//! | `Delay`             | [`MudTooltip::delay_ms`] | `0` |
//! | `Disabled`          | [`MudTooltip::disabled`] | `false` |
//! | `Duration`          | [`MudTooltip::duration_ms`] | `251` |
//! | `Inline`            | [`MudTooltip::inline`] | `true` (stored; no-op) |
//! | `Placement`         | [`MudTooltip::placement`] | [`Placement::Bottom`] |
//! | `RightToLeft`       | [`MudTooltip::right_to_left`] | `false` |
//! | `RootClass`         | [`MudTooltip::classes`] | `None` |
//! | `RootStyle`         | *(use `classes`)* | — |
//! | `ShowOnClick`       | [`MudTooltip::show_on_click`] | `false` |
//! | `ShowOnFocus`       | [`MudTooltip::show_on_focus`] | `true` |
//! | `ShowOnHover`       | [`MudTooltip::show_on_hover`] | `true` |
//! | `Text`              | [`MudTooltip::text`] | `None` |
//! | `TooltipContent`    | [`MudTooltip::tooltip_content`] | `None` |
//! | `Visible`           | [`TooltipState::visible`] | `false` |
//! | `VisibleChanged`    | [`MudTooltip::on_visible_changed`] | `None` |
//!
//! # Extensions (not in MudBlazor)
//!
//! | Extension | Description |
//! |-----------|-------------|
//! | [`TooltipState`] | Caller-owned visibility + timers. |
//! | [`TooltipResponse`] | Frame's `child` response, resulting `visible`, and whether it changed. |
//! | [`compute_opacity`] / [`resolve_placement`] / [`placement_to_origins`] | Pure helpers, unit-tested. |
//!
//! # Deferred to pass 2
//!
//! `Inline` (MudBlazor's block/inline toggle has no egui equivalent — tooltips
//! are always overlaid), `RootStyle` (use `classes(..)`), and a **proper
//! protruding arrow** (pass 1 draws a small triangle inside the tooltip's
//! paper, not outside it — see the README's "Arrow" note).
//!
//! # Text and icons
//!
//! The default tooltip body renders [`text`](MudTooltip::text) through
//! [`MudText`]. Custom bodies via [`tooltip_content`](MudTooltip::tooltip_content)
//! are expected to use `MudText` / `MudIcon` per project convention.

use egui::{pos2, vec2, Color32, CornerRadius, Response, Sense, Shape, Stroke, Ui, Vec2};
use mud_egui_bootstrap::bs_style::BsStyle;
use mud_egui_core::base::component_base::ComponentBase;
use mud_egui_core::enums::{AnchorOrigin, Color, Placement, StyleFlavor, TransformOrigin, Typo};
use mud_egui_tailwind::tw_style::{TwState, TwStyle};
use mud_egui_theming::models::palettes::palette::Palette;
use mud_egui_theming::theme::MudTheme;

use crate::feedback::popover::MudPopover;
use crate::typography::mud_text::MudText;

use super::tooltip_state::TooltipState;

// =============================================================================
// Constants
// =============================================================================

/// Base rem size passed to the Tailwind resolver.
const REM_SIZE_PT: f32 = 14.0;

/// Horizontal padding inside the tooltip body.
const TOOLTIP_PADDING: f32 = 8.0;

/// Size of the arrow triangle, in logical pixels.
const ARROW_SIZE: f32 = 6.0;

/// Extra padding applied on the anchor-facing edge when `arrow` is true, so
/// the triangle has visual room.
const ARROW_PADDING: f32 = 6.0;

/// Default delay before showing the tooltip, in milliseconds.
pub const DEFAULT_DELAY_MS: u32 = 0;

/// Default fade duration, in milliseconds.
pub const DEFAULT_DURATION_MS: u32 = 251;

// =============================================================================
// Pure helpers
// =============================================================================

/// Resolves `Placement::Start` / `End` under the active layout direction.
#[inline]
pub fn resolve_placement(placement: Placement, right_to_left: bool) -> Placement {
    match (placement, right_to_left) {
        (Placement::Start, true) => Placement::End,
        (Placement::End, true) => Placement::Start,
        (p, _) => p,
    }
}

/// Maps a (resolved) [`Placement`] to the `MudPopover` anchor + transform
/// pair that puts the tooltip on the correct side of the anchor.
///
/// The physical `Left` / `Right` variants map identically to their logical
/// counterparts `Start` / `End` (LTR). Under RTL, [`resolve_placement`] has
/// already flipped `Start` → `End` and vice-versa, but `Left` / `Right` are
/// unaffected by [`resolve_placement`] because they are physical.
#[inline]
pub fn placement_to_origins(placement: Placement) -> (AnchorOrigin, TransformOrigin) {
    match placement {
        Placement::Top => (AnchorOrigin::Top, TransformOrigin::BottomCenter),
        Placement::Bottom => (AnchorOrigin::Bottom, TransformOrigin::TopCenter),
        Placement::Start | Placement::Left => {
            (AnchorOrigin::Left, TransformOrigin::CenterRight)
        }
        Placement::End | Placement::Right => {
            (AnchorOrigin::Right, TransformOrigin::CenterLeft)
        }
    }
}

/// Computes the current fade opacity in `0.0..=1.0`.
///
/// * No `show_started_at` and no `hide_started_at` → `0.0` (idle).
/// * `show_started_at` only → linear fade-in over `duration_secs`.
/// * `hide_started_at` set → linear fade-out over `duration_secs`.
///
/// A `duration_secs <= 0.0` short-circuits to `1.0` while visible and `0.0`
/// otherwise (no animation).
pub fn compute_opacity(state: &TooltipState, now: f64, duration_secs: f64) -> f32 {
    if duration_secs <= 0.0 {
        return if state.visible { 1.0 } else { 0.0 };
    }
    if let Some(hide_started) = state.hide_started_at {
        let t = ((now - hide_started) / duration_secs).clamp(0.0, 1.0) as f32;
        1.0 - t
    } else if let Some(show_started) = state.show_started_at {
        let t = ((now - show_started) / duration_secs).clamp(0.0, 1.0) as f32;
        t
    } else {
        0.0
    }
}

/// Whether the given trigger states should make the tooltip visible.
#[inline]
pub fn desired_visibility(
    disabled: bool,
    hover_ready: bool,
    focus_ready: bool,
    click_open: bool,
) -> bool {
    !disabled && (hover_ready || focus_ready || click_open)
}

/// Scales a [`Color32`]'s alpha by `opacity` (in `0.0..=1.0`).
#[inline]
fn with_opacity(c: Color32, opacity: f32) -> Color32 {
    if opacity >= 1.0 {
        return c;
    }
    if opacity <= 0.0 {
        return Color32::TRANSPARENT;
    }
    Color32::from_rgba_unmultiplied(
        c.r(),
        c.g(),
        c.b(),
        (c.a() as f32 * opacity).round() as u8,
    )
}

/// Resolves the tooltip background + foreground colours from the `Color`
/// parameter and the active palette.
///
/// `Color::Default` and `Color::Inherit` use the palette's `background_gray`
/// with `text_primary` on top; explicit colors use the accent with
/// `primary_contrast_text` on top.
#[inline]
fn resolve_tooltip_colors(color: Color, palette: &Palette) -> (Color32, Color32) {
    match color {
        Color::Default | Color::Inherit => (palette.background_gray.into(), palette.text_primary.into()),
        other => {
            let bg = other
                .resolve(palette)
                .map(Color32::from)
                .unwrap_or_else(|| palette.background_gray.into());
            (bg, palette.primary_contrast_text.into())
        }
    }
}

// =============================================================================
// Response
// =============================================================================

/// The result of one [`MudTooltip::show`] frame.
#[derive(Debug)]
pub struct TooltipResponse {
    /// The trigger's response.
    pub child: Response,
    /// The tooltip's logical visibility **after** this frame.
    pub visible: bool,
    /// `true` when `visible` transitioned this frame.
    pub changed: bool,
}

// =============================================================================
// MudTooltip
// =============================================================================

/// A small popup that shows additional information on hover, focus, or click.
#[must_use = "MudTooltip does nothing until you call `.show(ui, state, child)`"]
pub struct MudTooltip<'a> {
    // ── MudBlazor parameters ───────────────────────────────────────────────
    /// Draws a small triangle on the anchor-facing edge of the tooltip body
    /// (`Arrow`, default `false`). Pass 1 draws it inside the paper, not as a
    /// protruding caret; see the README.
    pub arrow: bool,
    /// Tooltip accent colour (`Color`, default `Default`).
    pub color: Color,
    /// Delay before showing, in milliseconds (`Delay`, default `0`).
    pub delay_ms: u32,
    /// Fade in / out duration in milliseconds (`Duration`, default `251`).
    pub duration_ms: u32,
    /// Disables every trigger (`Disabled`, default `false`).
    pub disabled: bool,
    /// Stored for parity (`Inline`, default `true`). No-op in egui.
    pub inline: bool,
    /// Tooltip side relative to the anchor (`Placement`, default `Bottom`).
    pub placement: Placement,
    /// RTL flag for `Placement::Start` / `End`.
    pub right_to_left: bool,
    /// Show on trigger click (`ShowOnClick`, default `false`).
    pub show_on_click: bool,
    /// Show when the trigger gains focus (`ShowOnFocus`, default `true`).
    pub show_on_focus: bool,
    /// Show on pointer hover (`ShowOnHover`, default `true`).
    pub show_on_hover: bool,
    /// Tooltip text (`Text`). Ignored when `tooltip_content` is set.
    pub text: Option<String>,
    /// Custom tooltip body (`TooltipContent`). Overrides `text`.
    pub tooltip_content: Option<Box<dyn FnMut(&mut Ui) + 'a>>,
    /// Fires when `visible` changes (`VisibleChanged`).
    pub on_visible_changed: Option<Box<dyn FnMut(bool) + 'a>>,

    // ── Utilities ──────────────────────────────────────────────────────────
    /// Utility classes on the tooltip body (`RootClass`).
    pub base: ComponentBase,
    /// Bootstrap or Tailwind resolver.
    pub flavor: StyleFlavor,
    /// Bootstrap resolver state.
    pub bs_style: BsStyle,
    /// Tailwind resolver state.
    pub tw_style: TwStyle,
    /// Tailwind interaction state.
    pub tw_state: TwState,
}

impl<'a> Default for MudTooltip<'a> {
    fn default() -> Self {
        Self {
            arrow: false,
            color: Color::Default,
            delay_ms: DEFAULT_DELAY_MS,
            duration_ms: DEFAULT_DURATION_MS,
            disabled: false,
            inline: true,
            placement: Placement::Bottom,
            right_to_left: false,
            show_on_click: false,
            show_on_focus: true,
            show_on_hover: true,
            text: None,
            tooltip_content: None,
            on_visible_changed: None,
            base: ComponentBase::new(),
            flavor: StyleFlavor::default(),
            bs_style: BsStyle::default(),
            tw_style: TwStyle::default(),
            tw_state: TwState::default(),
        }
    }
}

impl<'a> MudTooltip<'a> {
    /// Creates a tooltip with MudBlazor defaults.
    pub fn new() -> Self {
        Self::default()
    }

    // ── Builders ───────────────────────────────────────────────────────────

    #[inline] pub fn arrow(mut self, b: bool) -> Self { self.arrow = b; self }
    #[inline] pub fn color(mut self, c: Color) -> Self { self.color = c; self }
    #[inline] pub fn delay_ms(mut self, ms: u32) -> Self { self.delay_ms = ms; self }
    #[inline] pub fn duration_ms(mut self, ms: u32) -> Self { self.duration_ms = ms; self }
    #[inline] pub fn disabled(mut self, b: bool) -> Self { self.disabled = b; self }
    #[inline] pub fn inline(mut self, b: bool) -> Self { self.inline = b; self }
    #[inline] pub fn placement(mut self, p: Placement) -> Self { self.placement = p; self }
    #[inline] pub fn right_to_left(mut self, b: bool) -> Self { self.right_to_left = b; self }
    #[inline] pub fn show_on_click(mut self, b: bool) -> Self { self.show_on_click = b; self }
    #[inline] pub fn show_on_focus(mut self, b: bool) -> Self { self.show_on_focus = b; self }
    #[inline] pub fn show_on_hover(mut self, b: bool) -> Self { self.show_on_hover = b; self }
    #[inline] pub fn text(mut self, s: impl Into<String>) -> Self { self.text = Some(s.into()); self }
    #[inline] pub fn tooltip_content(mut self, f: impl FnMut(&mut Ui) + 'a) -> Self { self.tooltip_content = Some(Box::new(f)); self }
    #[inline] pub fn on_visible_changed(mut self, f: impl FnMut(bool) + 'a) -> Self { self.on_visible_changed = Some(Box::new(f)); self }
    #[inline] pub fn classes<S: AsRef<str>>(mut self, classes: impl IntoIterator<Item = S>) -> Self { self.base = self.base.classes(classes); self }
    #[inline] pub fn flavor(mut self, f: StyleFlavor) -> Self { self.flavor = f; self }

    // ── Rendering ──────────────────────────────────────────────────────────

    /// Draws the trigger and, when visible, the tooltip.
    ///
    /// Returns a [`TooltipResponse`] with the child's response, the resulting
    /// visibility, and whether it changed this frame.
    pub fn show(
        mut self,
        ui: &mut Ui,
        state: &mut TooltipState,
        child: impl FnOnce(&mut Ui) -> Response,
    ) -> TooltipResponse {
        let now = ui.input(|i| i.time);
        let delay_secs = self.delay_ms as f64 / 1000.0;
        let duration_secs = self.duration_ms as f64 / 1000.0;

        // 1. Render the child (trigger). Its response drives every trigger.
        let child_resp = child(ui);

        // 2. Update the click latch.
        if self.show_on_click && child_resp.clicked() && !self.disabled {
            state.click_open = !state.click_open;
        }

        // 3. Hover start / stop with delay.
        if self.show_on_hover && child_resp.hovered() && !self.disabled {
            if state.hover_started_at.is_none() {
                state.hover_started_at = Some(now);
                // Ask egui to wake us when the delay elapses.
                if self.delay_ms > 0 {
                    ui.ctx()
                        .request_repaint_after(std::time::Duration::from_millis(
                            self.delay_ms as u64,
                        ));
                }
            }
        } else {
            state.hover_started_at = None;
        }

        let hover_ready = self.show_on_hover
            && state.hover_started_at.is_some()
            && now - state.hover_started_at.unwrap() >= delay_secs;
        let focus_ready = self.show_on_focus && child_resp.has_focus();

        // 4. Reconcile the logical `visible` state.
        let desired = desired_visibility(
            self.disabled,
            hover_ready,
            focus_ready,
            state.click_open,
        );
        let was_visible = state.visible;
        let changed = desired != was_visible;
        if changed {
            if desired {
                state.show(now);
            } else {
                state.hide(now);
            }
            if let Some(cb) = self.on_visible_changed.as_mut() {
                cb(state.visible);
            }
        }

        // 5. Compute fade opacity.
        let opacity = compute_opacity(state, now, duration_secs);

        // 6. Reset animation once the hide finishes.
        if opacity <= 0.0 && !state.visible && state.hide_started_at.is_some() {
            state.reset_animation();
        }

        // 7. Render the popover if anything is on screen.
        if opacity > 0.0 {
            // Always keep the popover open while the tooltip is fading in or
            // out. The dismiss hooks are disabled below — the tooltip's
            // visibility is entirely caller-controlled.
            state.popover.open = true;

            let theme = MudTheme::fetch(ui.ctx()).unwrap_or_default();
            let palette = theme.current_palette();
            let (bg, ink) = resolve_tooltip_colors(self.color, palette);
            let bg = with_opacity(bg, opacity);
            let ink = with_opacity(ink, opacity);

            let resolved = resolve_placement(self.placement, self.right_to_left);
            let (anchor, transform) = placement_to_origins(resolved);

            let popover = MudPopover::new()
                .anchor_origin(anchor)
                .transform_origin(transform)
                .paper(true)
                .elevation(4)
                .dismiss_on_escape(false)
                .dismiss_on_outside_click(false)
                .match_anchor_width(false);

            let mut tooltip_content = self.tooltip_content.take();
            let text = self.text.clone();
            let arrow = self.arrow;
            let placement = resolved;

            let _ = popover.show(ui, &mut state.popover, child_resp.rect, |pop_ui| {
                render_body(
                    pop_ui,
                    &mut tooltip_content,
                    text.as_deref(),
                    arrow,
                    placement,
                    bg,
                    ink,
                );
            });

            // Request another frame while fading so opacity advances.
            if state.visible && state.show_started_at.is_some() && opacity < 1.0 {
                ui.ctx().request_repaint();
            } else if !state.visible && opacity > 0.0 {
                ui.ctx().request_repaint();
            }
        }

        TooltipResponse {
            child: child_resp,
            visible: state.visible,
            changed,
        }
    }
}

/// Renders the tooltip body into the popover UI. Adds an arrow band on the
/// anchor-facing edge when `arrow` is `true`.
#[allow(clippy::too_many_arguments)]
fn render_body(
    ui: &mut Ui,
    content: &mut Option<Box<dyn FnMut(&mut Ui) + '_>>,
    text: Option<&str>,
    arrow: bool,
    placement: Placement,
    bg: Color32,
    ink: Color32,
) {
    // Reserve the arrow padding on the anchor-facing edge.
    let (pad_top, pad_bottom, pad_left, pad_right) = if arrow {
        match placement {
            Placement::Top => (0.0, ARROW_PADDING, 0.0, 0.0),
            Placement::Bottom => (ARROW_PADDING, 0.0, 0.0, 0.0),
            // Tooltip to the left of anchor → arrow on the right edge.
            Placement::Start | Placement::Left => (0.0, 0.0, 0.0, ARROW_PADDING),
            // Tooltip to the right of anchor → arrow on the left edge.
            Placement::End | Placement::Right => (0.0, 0.0, ARROW_PADDING, 0.0),
        }
    } else {
        (0.0, 0.0, 0.0, 0.0)
    };

    // Painted background: a filled rounded rect covering the tooltip body.
    let total_w = ui.available_width().max(20.0);
    let (bg_rect, _) = ui.allocate_exact_size(
        vec2(total_w, 0.0), // height determined by content
        Sense::hover(),
    );
    let _ = bg_rect;

    // Draw the triangle in the reserved padding first, so the text sits on
    // top. The triangle points *toward the anchor*: for `Placement::Bottom`
    // the tooltip is below the anchor, so the arrow points up.
    if arrow {
        let arrow_rect = ui.allocate_exact_size(
            match placement {
                // Top / Bottom: arrow band spans the tooltip's width.
                Placement::Top | Placement::Bottom => vec2(total_w, ARROW_PADDING),
                // Left / Right / Start / End: arrow band spans only the
                // reserved edge padding.
                Placement::Left
                | Placement::Right
                | Placement::Start
                | Placement::End => vec2(ARROW_PADDING, 0.0),
            },
            Sense::hover(),
        );
        let r = arrow_rect.0;
        paint_arrow(ui, r, placement, bg);
    }

    // Content area with padding.
    ui.allocate_ui_with_layout(
        vec2(
            (ui.available_width() - TOOLTIP_PADDING * 2.0).max(0.0),
            0.0,
        ),
        egui::Layout::top_down(egui::Align::Min),
        |ui| {
            if let Some(f) = content.as_mut() {
                f(ui);
            } else if let Some(t) = text {
                ui.add(
                    MudText::new(t.to_owned())
                        .typo(Typo::Body2)
                        .custom_color(ink),
                );
            }
        },
    );

    let _ = pad_top;
    let _ = pad_bottom;
    let _ = pad_left;
    let _ = pad_right;
}

/// Paints a small triangle inside `rect` on the edge facing the anchor.
fn paint_arrow(ui: &mut Ui, rect: egui::Rect, placement: Placement, color: Color32) {
    let cx = rect.center().x;
    let cy = rect.center().y;
    let s = ARROW_SIZE;
    let points: Vec<egui::Pos2> = match placement {
        // Tooltip below anchor → arrow at top, pointing up.
        Placement::Bottom => vec![
            pos2(cx - s, rect.top() + ARROW_PADDING),
            pos2(cx + s, rect.top() + ARROW_PADDING),
            pos2(cx, rect.top()),
        ],
        // Tooltip above anchor → arrow at bottom, pointing down.
        Placement::Top => vec![
            pos2(cx - s, rect.bottom() - ARROW_PADDING),
            pos2(cx + s, rect.bottom() - ARROW_PADDING),
            pos2(cx, rect.bottom()),
        ],
        // Tooltip to the right of anchor → arrow at left, pointing left.
        Placement::End | Placement::Right => vec![
            pos2(rect.left() + ARROW_PADDING, cy - s),
            pos2(rect.left() + ARROW_PADDING, cy + s),
            pos2(rect.left(), cy),
        ],
        // Tooltip to the left of anchor → arrow at right, pointing right.
        Placement::Start | Placement::Left => vec![
            pos2(rect.right() - ARROW_PADDING, cy - s),
            pos2(rect.right() - ARROW_PADDING, cy + s),
            pos2(rect.right(), cy),
        ],
    };
    ui.painter().add(Shape::convex_polygon(
        points,
        color,
        Stroke::NONE,
    ));
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ── resolve_placement ──────────────────────────────────────────────────

    #[test]
    fn physical_placements_pass_through() {
        for p in [Placement::Top, Placement::Bottom] {
            assert_eq!(resolve_placement(p, false), p);
            assert_eq!(resolve_placement(p, true), p);
        }
    }

    #[test]
    fn start_end_swap_under_rtl() {
        assert_eq!(resolve_placement(Placement::Start, false), Placement::Start);
        assert_eq!(resolve_placement(Placement::Start, true), Placement::End);
        assert_eq!(resolve_placement(Placement::End, false), Placement::End);
        assert_eq!(resolve_placement(Placement::End, true), Placement::Start);
    }

    // ── placement_to_origins ───────────────────────────────────────────────

    #[test]
    fn placement_to_origins_maps_every_variant() {
        assert_eq!(
            placement_to_origins(Placement::Bottom),
            (AnchorOrigin::Bottom, TransformOrigin::TopCenter)
        );
        assert_eq!(
            placement_to_origins(Placement::Top),
            (AnchorOrigin::Top, TransformOrigin::BottomCenter)
        );
        assert_eq!(
            placement_to_origins(Placement::Start),
            (AnchorOrigin::Left, TransformOrigin::CenterRight)
        );
        assert_eq!(
            placement_to_origins(Placement::End),
            (AnchorOrigin::Right, TransformOrigin::CenterLeft)
        );
    }

    // ── compute_opacity ────────────────────────────────────────────────────

    #[test]
    fn opacity_zero_when_idle() {
        let s = TooltipState::new("t");
        assert_eq!(compute_opacity(&s, 0.0, 0.25), 0.0);
    }

    #[test]
    fn opacity_zero_duration_snaps_to_boolean() {
        let mut s = TooltipState::new("t");
        s.show(1.0);
        assert_eq!(compute_opacity(&s, 1.5, 0.0), 1.0);
        s.hide(2.0);
        assert_eq!(compute_opacity(&s, 2.5, 0.0), 0.0);
    }

    #[test]
    fn opacity_fades_in_linearly() {
        let mut s = TooltipState::new("t");
        s.show(1.0);
        assert!((compute_opacity(&s, 1.0, 1.0) - 0.0).abs() < 1e-5);
        assert!((compute_opacity(&s, 1.5, 1.0) - 0.5).abs() < 1e-5);
        assert!((compute_opacity(&s, 2.0, 1.0) - 1.0).abs() < 1e-5);
        assert!((compute_opacity(&s, 9.0, 1.0) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn opacity_fades_out_linearly() {
        let mut s = TooltipState::new("t");
        s.show(1.0);
        s.hide(2.0);
        assert!((compute_opacity(&s, 2.0, 1.0) - 1.0).abs() < 1e-5);
        assert!((compute_opacity(&s, 2.5, 1.0) - 0.5).abs() < 1e-5);
        assert!((compute_opacity(&s, 3.0, 1.0) - 0.0).abs() < 1e-5);
        assert!((compute_opacity(&s, 9.0, 1.0) - 0.0).abs() < 1e-5);
    }

    // ── desired_visibility ─────────────────────────────────────────────────

    #[test]
    fn disabled_blocks_every_trigger() {
        assert!(!desired_visibility(true, true, true, true));
    }

    #[test]
    fn any_trigger_opens() {
        assert!(desired_visibility(false, true, false, false));
        assert!(desired_visibility(false, false, true, false));
        assert!(desired_visibility(false, false, false, true));
        assert!(!desired_visibility(false, false, false, false));
    }

    // ── with_opacity ───────────────────────────────────────────────────────

    #[test]
    fn with_opacity_identity_and_transparent() {
        let c = Color32::from_rgb(100, 150, 200);
        assert_eq!(with_opacity(c, 1.0), c);
        assert_eq!(with_opacity(c, 0.0), Color32::TRANSPARENT);
    }

    #[test]
    fn with_opacity_scales_alpha() {
        let c = Color32::from_rgba_unmultiplied(100, 150, 200, 200);
        assert_eq!(with_opacity(c, 0.5).a(), 100);
    }

    // ── Defaults ───────────────────────────────────────────────────────────

    #[test]
    fn defaults_match_mudblazor() {
        let t: MudTooltip = MudTooltip::new();
        assert!(!t.arrow);
        assert!(matches!(t.color, Color::Default));
        assert_eq!(t.delay_ms, DEFAULT_DELAY_MS);
        assert_eq!(t.duration_ms, DEFAULT_DURATION_MS);
        assert!(!t.disabled);
        assert!(t.inline);
        assert!(matches!(t.placement, Placement::Bottom));
        assert!(!t.right_to_left);
        assert!(!t.show_on_click);
        assert!(t.show_on_focus);
        assert!(t.show_on_hover);
        assert!(t.text.is_none());
        assert!(t.tooltip_content.is_none());
        assert!(t.on_visible_changed.is_none());
    }

    // ── Builders ───────────────────────────────────────────────────────────

    #[test]
    fn builders_set_every_field() {
        let t = MudTooltip::new()
            .arrow(true)
            .color(Color::Secondary)
            .delay_ms(200)
            .duration_ms(150)
            .disabled(true)
            .inline(false)
            .placement(Placement::Top)
            .right_to_left(true)
            .show_on_click(true)
            .show_on_focus(false)
            .show_on_hover(false)
            .text("tip")
            .tooltip_content(|_| {})
            .on_visible_changed(|_| {})
            .classes(["my-tip"]);

        assert!(t.arrow);
        assert!(matches!(t.color, Color::Secondary));
        assert_eq!(t.delay_ms, 200);
        assert_eq!(t.duration_ms, 150);
        assert!(t.disabled && !t.inline);
        assert!(matches!(t.placement, Placement::Top));
        assert!(t.right_to_left);
        assert!(t.show_on_click && !t.show_on_focus && !t.show_on_hover);
        assert_eq!(t.text.as_deref(), Some("tip"));
        assert!(t.tooltip_content.is_some());
        assert!(t.on_visible_changed.is_some());
        assert!(t.base.has_class("my-tip"));
    }
}