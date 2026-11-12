//! `MudDialog` — a modal overlay that presents information, a choice, or input.
//!
//! Port of `MudDialog.razor.cs` (MudBlazor 8.13.0). Pass 1.
//!
//! | MudBlazor parameter | Rust field / method |
//! |---------------------|---------------------|
//! | `ActionsClass` | [`MudDialog::actions_class`] — stored, not resolved |
//! | `ContentClass` | [`MudDialog::content_class`] — stored, not resolved |
//! | `ContentStyle` | [`MudDialog::content_style`] — stored, not resolved |
//! | `DefaultFocus` | [`MudDialog::default_focus`] (default `Element`) |
//! | `DialogActions` | `actions` argument of [`MudDialog::show_with_actions`] |
//! | `DialogContent` | `content` argument of [`MudDialog::show`] |
//! | `Gutters` | [`MudDialog::gutters`] (default `true`) — newer builds call this `DisableSidePadding` |
//! | `IsVisible` | [`DialogState::open`] |
//! | `IsVisibleChanged` | [`DialogResponse::opened`] / [`DialogResponse::closed`] |
//! | `OnBackdropClick` | [`MudDialog::on_backdrop_click`] |
//! | `OnKeyDown` / `OnKeyUp` | deferred — browser `KeyboardEventArgs` have no egui equivalent |
//! | `Options` | [`DialogState::options`] / [`MudDialog::options`] |
//! | `Title` | [`MudDialog::title`] / [`DialogState::title`] |
//! | `TitleClass` | [`MudDialog::title_class`] — stored, not resolved |
//! | `TitleContent` | [`MudDialog::title_content`] |
//! | `Class` | [`MudDialog::classes`] — stored, not resolved |
//! | `Style` / `Tag` / `UserAttributes` | not ported: no CSS strings, no HTML tag |
//! | `Close(DialogResult)` | [`DialogState::close_with`] / [`DialogControl::close_with`] |
//! | `Show(title, options)` (inline) | [`DialogState::open_with`] |
//! | `IDialogService.ShowAsync<T>` | [`DialogState::open`] + [`MudDialog::show`] |
//! | `IMudDialogInstance` | the `&mut DialogControl` handed to the content |
//!
//! # Composition, not reimplementation
//!
//! | Primitive | Role |
//! |-----------|------|
//! | [`MudPopover`](crate::feedback::popover::MudPopover) | The `Area` ordering and size-cache pattern reused here. |
//! | [`MudText`] | The title and any text the content draws. |
//! | [`MudIcon`] | The header's close (×) glyph. |
//!
//! # Layout
//!
//! ```
//! ┌───────────────────────────────┐
//! │ [ title ]               [ ✕ ] │  title row (skipped by no_header)
//! ├───────────────────────────────┤
//! │ content                       │
//! ├───────────────────────────────┤
//! │                       [actions]│
//! └───────────────────────────────┘
//! ```
//!
//! The panel's size is cached in `egui::Memory` under `state.id` so a
//! position that depends on the size (centering) can be computed on the first
//! open frame — the same one-frame-lag pattern `MudPopover` uses.
//!
//! # Text and icons
//!
//! Every visible string is drawn through [`MudText`]; the close glyph through
//! [`MudIcon`]. The content closure is responsible for using them too.

use egui::{
    pos2, vec2, Color32, CornerRadius, CursorIcon, Id, Margin, Order, Pos2, Rect, Response,
    Sense, Stroke, Ui, Vec2,
};
use mud_egui_core::base::component_base::ComponentBase;
use mud_egui_core::enums::{Color, DefaultFocus, DialogPosition, MaxWidth, Typo};
use mud_egui_icons_material::filled;
use mud_egui_theming::models::palettes::palette::Palette;
use mud_egui_theming::theme::MudTheme;

use crate::data_display::icons::mud_icon::MudIcon;
use crate::typography::mud_text::MudText;

use super::dialog_control::DialogControl;
use super::dialog_options::DialogOptions;
use super::dialog_result::DialogResult;
use super::dialog_state::DialogState;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Alpha of the full-screen backdrop.
const BACKDROP_ALPHA: u8 = 96;

/// Panel corner radius, in points.
const PANEL_RADIUS: u8 = 8;

/// Panel shadow depth.
const PANEL_ELEVATION: u8 = 6;

/// Horizontal padding around the title, content and actions.
pub const PADDING_X: f32 = 24.0;

/// Bottom padding below the actions.
pub const PADDING_BOTTOM: f32 = 20.0;

/// Height of the title row.
pub const TITLE_ROW_HEIGHT: f32 = 48.0;

/// Side of the close button's hit rect.
const CLOSE_HIT: f32 = 28.0;

/// Side of the close glyph.
const CLOSE_GLYPH: f32 = 18.0;

/// Gap above the actions row.
const ACTIONS_GAP: f32 = 8.0;

/// Smallest panel width.
pub const MIN_DIALOG_WIDTH: f32 = 280.0;

/// Smallest panel height.
pub const MIN_DIALOG_HEIGHT: f32 = 120.0;

/// Largest fraction of the screen a dialog may occupy.
pub const MAX_SCREEN_FRACTION: f32 = 0.9;

/// Gap between the panel and the screen edges.
const SCREEN_MARGIN: f32 = 24.0;

/// First-open-frame size guess.
const DEFAULT_DIALOG_SIZE: Vec2 = Vec2::new(560.0, 320.0);

// ---------------------------------------------------------------------------
// DialogResponse
// ---------------------------------------------------------------------------

/// The result of one [`MudDialog::show`] frame.
#[derive(Debug)]
pub struct DialogResponse<R> {
    /// The panel's own `Response`; zero-sized when the dialog is closed.
    pub response: Response,
    /// The closing frame's result, drained from [`DialogState::result`].
    /// `Some` only on the frame the dialog closed.
    pub result: Option<DialogResult>,
    /// `true` on the frame the dialog opened.
    pub opened: bool,
    /// `true` on the frame the dialog closed.
    pub closed: bool,
    /// The content closure's value; `None` while closed.
    pub content: Option<R>,
}

impl<R> DialogResponse<R> {
    /// `true` when this frame produced a result.
    #[inline]
    pub fn has_result(&self) -> bool {
        self.result.is_some()
    }
}

// ---------------------------------------------------------------------------
// MudDialog
// ---------------------------------------------------------------------------

/// A modal overlay that presents information, a choice, or input.
///
/// Mirrors `MudDialog` in MudBlazor 8.13.0. The `IDialogService` /
/// `IMudDialogInstance` machinery becomes a caller-owned [`DialogState`] plus
/// a [`DialogControl`] handed to the content — the same split every other
/// component in this workspace uses.
#[must_use = "MudDialog does nothing until you call `.show(ui, state, ..)`"]
pub struct MudDialog<'a> {
    /// Title text. Overridden by [`DialogState::title`] when set.
    pub title: Option<String>,
    /// Custom title body. Overrides `title` when set.
    pub title_content: Option<Box<dyn FnMut(&mut Ui) + 'a>>,
    /// Which element receives focus on open. Mirrors `DefaultFocus`.
    pub default_focus: DefaultFocus,
    /// Initial options. Overridden by [`DialogState::options`] when set.
    pub options: DialogOptions,
    /// Side padding. Mirrors `Gutters` (default `true`); newer builds call
    /// this `DisableSidePadding` with the opposite polarity.
    pub gutters: bool,
    /// Stored, not resolved. Mirrors `ActionsClass`.
    pub actions_class: Option<String>,
    /// Stored, not resolved. Mirrors `ContentClass`.
    pub content_class: Option<String>,
    /// Stored, not resolved. Mirrors `ContentStyle`.
    pub content_style: Option<String>,
    /// Stored, not resolved. Mirrors `TitleClass`.
    pub title_class: Option<String>,
    /// Accent colour for the panel surface and focus ring. Not a MudBlazor
    /// parameter — the dialog inherits the theme surface.
    pub color: Color,
    /// Fires on a backdrop click. When set, it replaces the default
    /// close-on-backdrop behaviour. Mirrors `OnBackdropClick`.
    pub on_backdrop_click: Option<Box<dyn FnMut() + 'a>>,
    /// Shared component state (utility classes). Stored for parity.
    pub base: ComponentBase,
}

impl<'a> Default for MudDialog<'a> {
    fn default() -> Self {
        Self {
            title: None,
            title_content: None,
            default_focus: DefaultFocus::Element,
            options: DialogOptions::default(),
            gutters: true,
            actions_class: None,
            content_class: None,
            content_style: None,
            title_class: None,
            color: Color::Default,
            on_backdrop_click: None,
            base: ComponentBase::new(),
        }
    }
}

impl<'a> MudDialog<'a> {
    /// Creates a dialog with MudBlazor's default settings.
    pub fn new() -> Self {
        Self::default()
    }

    // ── Builders ───────────────────────────────────────────────────────────

    #[inline] pub fn title(mut self, s: impl Into<String>) -> Self { self.title = Some(s.into()); self }
    #[inline] pub fn title_content(mut self, f: impl FnMut(&mut Ui) + 'a) -> Self { self.title_content = Some(Box::new(f)); self }
    #[inline] pub fn default_focus(mut self, f: DefaultFocus) -> Self { self.default_focus = f; self }
    #[inline] pub fn options(mut self, o: DialogOptions) -> Self { self.options = o; self }
    #[inline] pub fn gutters(mut self, b: bool) -> Self { self.gutters = b; self }
    #[inline] pub fn actions_class(mut self, s: impl Into<String>) -> Self { self.actions_class = Some(s.into()); self }
    #[inline] pub fn content_class(mut self, s: impl Into<String>) -> Self { self.content_class = Some(s.into()); self }
    #[inline] pub fn content_style(mut self, s: impl Into<String>) -> Self { self.content_style = Some(s.into()); self }
    #[inline] pub fn title_class(mut self, s: impl Into<String>) -> Self { self.title_class = Some(s.into()); self }
    #[inline] pub fn color(mut self, c: Color) -> Self { self.color = c; self }
    #[inline] pub fn on_backdrop_click(mut self, f: impl FnMut() + 'a) -> Self { self.on_backdrop_click = Some(Box::new(f)); self }
    #[inline] pub fn classes<S: AsRef<str>>(mut self, classes: impl IntoIterator<Item = S>) -> Self { self.base = self.base.classes(classes); self }

    // ── Rendering ──────────────────────────────────────────────────────────

    /// Draws the dialog with a content body and no actions row.
    pub fn show<R>(
        self,
        ui: &mut Ui,
        state: &mut DialogState,
        content: impl FnOnce(&mut Ui, &mut DialogControl) -> R,
    ) -> DialogResponse<R> {
        self.render(ui, state, content, None)
    }

    /// Draws the dialog with a content body and an actions row.
    ///
    /// The actions closure is boxed only when present — a plain `show` call
    /// allocates nothing for it.
    pub fn show_with_actions<R>(
        self,
        ui: &mut Ui,
        state: &mut DialogState,
        content: impl FnOnce(&mut Ui, &mut DialogControl) -> R,
        actions: impl FnOnce(&mut Ui, &mut DialogControl),
    ) -> DialogResponse<R> {
        let boxed: Option<Box<dyn FnOnce(&mut Ui, &mut DialogControl) + '_>> =
            Some(Box::new(actions));
        self.render(ui, state, content, boxed)
    }

    #[allow(clippy::too_many_lines)]
    fn render<R>(
        mut self,
        ui: &mut Ui,
        state: &mut DialogState,
        content: impl FnOnce(&mut Ui, &mut DialogControl) -> R,
        actions: Option<Box<dyn FnOnce(&mut Ui, &mut DialogControl) + '_>>,
    ) -> DialogResponse<R> {
        // 0. Closed: drain any externally-set result and bail.
        if !state.open {
            let response = ui.allocate_response(Vec2::ZERO, Sense::hover());
            return DialogResponse {
                response,
                result: state.take_result(),
                opened: false,
                closed: false,
                content: None,
            };
        }

        let panel_id = state.id;
        let opened = state.take_focus_request();

        // 1. Snapshot everything the panel needs. `state.options` wins over
        //    the builder's `options` (the `SetOptionsAsync` override).
        let options = if state.options != DialogOptions::default() {
            state.options.clone()
        } else {
            self.options.clone()
        };
        let title = state.title.clone().or_else(|| self.title.clone());
        let screen = ui.ctx().input(|i| i.content_rect());

        // 2. Deferred interactions, applied after the panel is drawn.
        let mut pending: Option<DialogResult> = None;

        // Escape dismissal.
        if options.close_on_escape_key
            && ui.ctx().input(|i| i.key_pressed(egui::Key::Escape))
        {
            pending = Some(DialogResult::cancel());
        }

        // 3. Backdrop (Middle order — behind the panel).
        let backdrop_clicked = {
            let backdrop = egui::Area::new(panel_id.with("dialog_backdrop"))
                .order(Order::Middle)
                .fixed_pos(screen.min)
                .interactable(true);
            let out = backdrop.show(ui.ctx(), |ui| {
                let resp = ui.allocate_rect(screen, Sense::click());
                ui.painter().rect_filled(
                    screen,
                    CornerRadius::ZERO,
                    Color32::from_black_alpha(BACKDROP_ALPHA),
                );
                resp
            });
            out.inner.clicked()
        };

        // 4. Panel geometry (size cache → position → Area).
        let cached = read_cached_size(ui.ctx(), panel_id);
        let natural = cached.unwrap_or(DEFAULT_DIALOG_SIZE);
        let panel_size = resolve_panel_size(screen.size(), natural, &options);
        let panel_pos = position_origin(
            options.effective_position(),
            screen,
            panel_size,
            SCREEN_MARGIN,
        );

        // 5. Theme + palette.
        let theme = MudTheme::fetch(ui.ctx()).unwrap_or_default();
        let palette = theme.current_palette();
        let p = resolve_palette(self.color, palette);

        let padding_x = if self.gutters { PADDING_X } else { 0.0 };
        let frame = egui::Frame::new()
            .fill(palette.surface.into())
            .corner_radius(CornerRadius::same(PANEL_RADIUS))
            .shadow(elevation_shadow(PANEL_ELEVATION))
            .inner_margin(Margin {
                left: padding_x as i8,
                right: padding_x as i8,
                top: 0,
                bottom: PADDING_BOTTOM as i8,
            });

        let mut control = DialogControl::new(options.clone(), title.clone());

        // 6. Panel.
        let panel_area = egui::Area::new(panel_id)
            .order(Order::Foreground)
            .fixed_pos(panel_pos)
            .interactable(true);

        let has_actions = actions.is_some();
        let focus_id = panel_id.with("dialog_panel_focus");
        let no_header = options.no_header;
        let show_close = options.close_button;
        let default_focus = self.default_focus;
        let mut title_content = self.title_content.take();
        let mut actions = actions;

        let panel = panel_area.show(ui.ctx(), |ui| {
            ui.set_min_width(panel_size.x);
            ui.set_max_width(panel_size.x);
            if options.full_screen {
                ui.set_min_height(screen.height());
            }
            frame
                .show(ui, |ui| {
                    // Reserve a focus target for `DefaultFocus`.
                    if opened && default_focus.requests_focus() {
                        let (rect, resp) =
                            ui.allocate_exact_size(Vec2::ZERO, Sense::hover());
                        let _ = rect;
                        resp.request_focus();
                        let _ = focus_id;
                    }

                    // 6a. Title row.
                    if !no_header {
                        draw_title_row(
                            ui,
                            &mut title_content,
                            title.as_deref(),
                            show_close,
                            &p,
                            &mut control,
                        );
                    }

                    // 6b. Content.
                    let out = content(ui, &mut control);

                    // 6c. Actions.
                    if let Some(mut f) = actions.take() {
                        ui.add_space(ACTIONS_GAP);
                        let y = ui.cursor().min.y;
                        ui.painter().line_segment(
                            [
                                pos2(ui.cursor().min.x, y),
                                pos2(ui.cursor().min.x + ui.available_width(), y),
                            ],
                            Stroke::new(1.0, p.divider),
                        );
                        ui.add_space(ACTIONS_GAP);
                        let width = ui.available_width();
                        ui.allocate_ui_with_layout(
                            vec2(width, 0.0),
                            egui::Layout::right_to_left(egui::Align::Center),
                            |ui| {
                                f(ui, &mut control);
                            },
                        );
                    }
                    let _ = has_actions;
                    out
                })
                .inner
        });

        let response = panel.response;
        let content_out = Some(panel.inner);

        // 7. Cache the measured size for the next frame's positioning.
        let measured = response.rect.size();
        write_cached_size(ui.ctx(), panel_id, measured);

        // 8. Apply deferred control requests.
        if let Some(t) = control.take_title_request() {
            state.title = t;
        }
        if let Some(o) = control.take_options_request() {
            state.options = o;
        }
        if let Some(r) = control.take_close_request() {
            pending = Some(r);
        }

        // 9. Backdrop click.
        if backdrop_clicked && options.backdrop_click {
            if let Some(cb) = self.on_backdrop_click.as_mut() {
                cb();
            } else {
                pending = Some(DialogResult::cancel());
            }
        }

        // 10. Commit a pending close.
        let mut closed = false;
        if let Some(r) = pending {
            state.close_with(r);
            closed = true;
        }
        let result = state.take_result();

        DialogResponse {
            response,
            result,
            opened,
            closed,
            content: content_out,
        }
    }
}

// ---------------------------------------------------------------------------
// Pure helpers
// ---------------------------------------------------------------------------

/// `(left, top)` of a dialog of `size` placed by `position` inside `screen`,
/// keeping `margin` points from every edge.
pub fn position_origin(position: DialogPosition, screen: Rect, size: Vec2, margin: f32) -> Pos2 {
    let inner = Rect::from_min_max(
        screen.min + Vec2::splat(margin),
        screen.max - Vec2::splat(margin),
    );
    let free = vec2(
        (inner.width() - size.x).max(0.0),
        (inner.height() - size.y).max(0.0),
    );
    let (fx, fy) = position.fractions();
    pos2(inner.left() + free.x * fx, inner.top() + free.y * fy)
}

/// Resolves the panel size from the screen, the cached natural size, and the
/// options.
pub fn resolve_panel_size(screen: Vec2, natural: Vec2, options: &DialogOptions) -> Vec2 {
    if options.full_screen {
        return vec2(screen.x.max(1.0), screen.y.max(1.0));
    }
    let available = vec2(
        (screen.x * MAX_SCREEN_FRACTION).max(1.0),
        (screen.y * MAX_SCREEN_FRACTION).max(1.0),
    );
    let cap = options
        .max_width
        .map(dialog_max_width_px)
        .unwrap_or(available.x)
        .min(available.x);
    let width = if options.full_width {
        cap
    } else {
        natural.x.max(MIN_DIALOG_WIDTH.min(cap)).min(cap)
    };
    let height = natural
        .y
        .max(MIN_DIALOG_HEIGHT.min(available.y))
        .min(available.y);
    vec2(width, height)
}

/// The pixel cap for a [`MaxWidth`] bucket, at dialog scale.
pub fn dialog_max_width_px(width: MaxWidth) -> f32 {
    match width {
        MaxWidth::Xs => 444.0,
        MaxWidth::Sm => 600.0,
        MaxWidth::Md => 960.0,
        MaxWidth::Lg => 1280.0,
        MaxWidth::Xl => 1920.0,
        MaxWidth::Xxl => 2560.0,
        MaxWidth::False => f32::INFINITY,
    }
}

/// Elevation → `egui::epaint::Shadow`, clamped at 24.
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

// ---------------------------------------------------------------------------
// Title row
// ---------------------------------------------------------------------------

fn draw_title_row(
    ui: &mut Ui,
    title_content: &mut Option<Box<dyn FnMut(&mut Ui) + '_>>,
    title: Option<&str>,
    show_close: bool,
    p: &PickerPalette,
    control: &mut DialogControl,
) {
    let width = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(vec2(width, TITLE_ROW_HEIGHT), Sense::hover());

    let close_w = if show_close { CLOSE_HIT + 8.0 } else { 0.0 };
    let title_rect = Rect::from_min_max(
        rect.min,
        pos2(rect.right() - close_w, rect.bottom()),
    );

    // Title body or text, through MudText / the caller's closure.
    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(title_rect)
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
    );
    match title_content.as_mut() {
        Some(f) => f(&mut child),
        None => {
            if let Some(t) = title {
                child.add(
                    MudText::new(t.to_owned())
                        .typo(Typo::H6)
                        .custom_color(p.text_primary),
                );
            }
        }
    }

    // Close button.
    if show_close {
        let close_rect = Rect::from_center_size(
            pos2(rect.right() - CLOSE_HIT * 0.5 - 4.0, rect.center().y),
            Vec2::splat(CLOSE_HIT),
        );
        let close_id = ui.scope_id().with("dialog_close");
        let resp = ui.interact(close_rect, close_id, Sense::click());
        if resp.hovered() {
            ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
            ui.painter()
                .rect_filled(close_rect.shrink(2.0), CornerRadius::same(4), p.hover_fill);
        }
        if resp.clicked() {
            control.cancel();
        }
        ui.put(
            close_rect,
            MudIcon::new(filled::CLOSE.to_owned())
                .size_px(CLOSE_GLYPH)
                .custom_color(p.text_secondary),
        );
    }
}

// ---------------------------------------------------------------------------
// Palette
// ---------------------------------------------------------------------------

struct PickerPalette {
    text_primary: Color32,
    text_secondary: Color32,
    divider: Color32,
    hover_fill: Color32,
}

fn resolve_palette(color: Color, palette: &Palette) -> PickerPalette {
    let _ = color; // The dialog inherits the theme surface; no accent needed.
    // Extract `Color32` first: `MudColor: Into<Color32>`, not `Into<u8>`, so
    // the direct `.into()` inside `from_rgba_unmultiplied` cannot infer a
    // target type.
    let text_primary: Color32 = palette.text_primary.into();
    PickerPalette {
        text_primary,
        text_secondary: palette.text_secondary.into(),
        divider: palette.divider.into(),
        hover_fill: Color32::from_rgba_unmultiplied(
            text_primary.r(),
            text_primary.g(),
            text_primary.b(),
            16,
        ),
    }
}

// ---------------------------------------------------------------------------
// Size cache
// ---------------------------------------------------------------------------

fn read_cached_size(ctx: &egui::Context, id: Id) -> Option<Vec2> {
    ctx.memory(|m| m.data.get_temp::<Vec2>(id.with("dialog_size")))
}

fn write_cached_size(ctx: &egui::Context, id: Id, size: Vec2) {
    ctx.memory_mut(|m| m.data.insert_temp(id.with("dialog_size"), size));
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // --- position_origin --------------------------------------------------

    fn screen() -> Rect {
        Rect::from_min_size(pos2(0.0, 0.0), vec2(1000.0, 800.0))
    }

    #[test]
    fn center_is_centered() {
        let pos = position_origin(DialogPosition::Center, screen(), vec2(400.0, 200.0), 20.0);
        // Inner rect is 20..980 × 20..780 → free = 560 × 560.
        assert_eq!(pos, pos2(20.0 + 280.0, 20.0 + 280.0));
    }

    #[test]
    fn corners_are_pinned() {
        let size = vec2(400.0, 200.0);
        assert_eq!(
            position_origin(DialogPosition::TopLeft, screen(), size, 20.0),
            pos2(20.0, 20.0)
        );
        assert_eq!(
            position_origin(DialogPosition::TopRight, screen(), size, 20.0),
            pos2(980.0 - 400.0, 20.0)
        );
        assert_eq!(
            position_origin(DialogPosition::BottomLeft, screen(), size, 20.0),
            pos2(20.0, 780.0 - 200.0)
        );
        assert_eq!(
            position_origin(DialogPosition::BottomRight, screen(), size, 20.0),
            pos2(980.0 - 400.0, 780.0 - 200.0)
        );
    }

    #[test]
    fn edges_are_pinned() {
        let size = vec2(400.0, 200.0);
        assert_eq!(
            position_origin(DialogPosition::TopCenter, screen(), size, 20.0),
            pos2(20.0 + 280.0, 20.0)
        );
        assert_eq!(
            position_origin(DialogPosition::BottomCenter, screen(), size, 20.0),
            pos2(20.0 + 280.0, 780.0 - 200.0)
        );
        assert_eq!(
            position_origin(DialogPosition::CenterLeft, screen(), size, 20.0),
            pos2(20.0, 20.0 + 280.0)
        );
        assert_eq!(
            position_origin(DialogPosition::CenterRight, screen(), size, 20.0),
            pos2(980.0 - 400.0, 20.0 + 280.0)
        );
    }

    #[test]
    fn custom_is_top_left() {
        assert_eq!(
            position_origin(DialogPosition::Custom, screen(), vec2(400.0, 200.0), 20.0),
            pos2(20.0, 20.0)
        );
    }

    #[test]
    fn oversized_dialog_clamps_into_the_margins() {
        let pos = position_origin(DialogPosition::Center, screen(), vec2(2000.0, 2000.0), 20.0);
        // Free space is 0 on both axes, so the origin is the inner top-left.
        assert_eq!(pos, pos2(20.0, 20.0));
    }

    // --- resolve_panel_size -----------------------------------------------

    #[test]
    fn natural_size_is_used_when_it_fits() {
        let opts = DialogOptions::default();
        let size = resolve_panel_size(vec2(1200.0, 900.0), vec2(500.0, 300.0), &opts);
        assert_eq!(size, vec2(500.0, 300.0));
    }

    #[test]
    fn tiny_natural_size_is_clamped_up() {
        let opts = DialogOptions::default();
        let size = resolve_panel_size(vec2(1200.0, 900.0), vec2(40.0, 20.0), &opts);
        assert_eq!(size, vec2(MIN_DIALOG_WIDTH, MIN_DIALOG_HEIGHT));
    }

    #[test]
    fn max_width_caps_the_panel() {
        let opts = DialogOptions::new().with_max_width(MaxWidth::Xs);
        let size = resolve_panel_size(vec2(2000.0, 900.0), vec2(900.0, 300.0), &opts);
        assert_eq!(size.x, 444.0);
    }

    #[test]
    fn full_width_stretches_to_the_cap() {
        let opts = DialogOptions::new().with_full_width(true);
        let size = resolve_panel_size(vec2(1000.0, 900.0), vec2(100.0, 300.0), &opts);
        // 90 % of 1000 = 900.
        assert_eq!(size.x, 900.0);
    }

    #[test]
    fn full_width_respects_max_width() {
        let opts = DialogOptions::new()
            .with_full_width(true)
            .with_max_width(MaxWidth::Sm);
        let size = resolve_panel_size(vec2(2000.0, 900.0), vec2(100.0, 300.0), &opts);
        assert_eq!(size.x, 600.0);
    }

    #[test]
    fn full_screen_fills_the_screen() {
        let opts = DialogOptions::new().with_full_screen(true);
        let size = resolve_panel_size(vec2(1200.0, 900.0), vec2(500.0, 300.0), &opts);
        assert_eq!(size, vec2(1200.0, 900.0));
    }

    #[test]
    fn height_never_exceeds_the_screen_fraction() {
        let opts = DialogOptions::default();
        let size = resolve_panel_size(vec2(1000.0, 500.0), vec2(500.0, 5000.0), &opts);
        assert_eq!(size.y, 450.0); // 90 % of 500
    }

    #[test]
    fn degenerate_screen_does_not_panic() {
        let opts = DialogOptions::default();
        let size = resolve_panel_size(vec2(0.0, 0.0), vec2(300.0, 200.0), &opts);
        assert!(size.x >= 1.0 && size.y >= 1.0);
    }

    // --- dialog_max_width_px ---------------------------------------------

    #[test]
    fn max_width_table_matches_the_breakpoints() {
        assert_eq!(dialog_max_width_px(MaxWidth::Xs), 444.0);
        assert_eq!(dialog_max_width_px(MaxWidth::Sm), 600.0);
        assert_eq!(dialog_max_width_px(MaxWidth::Md), 960.0);
        assert_eq!(dialog_max_width_px(MaxWidth::Lg), 1280.0);
        assert_eq!(dialog_max_width_px(MaxWidth::Xl), 1920.0);
        assert_eq!(dialog_max_width_px(MaxWidth::Xxl), 2560.0);
        assert!(dialog_max_width_px(MaxWidth::False).is_infinite());
    }

    // --- elevation_shadow -------------------------------------------------

    #[test]
    fn elevation_zero_is_empty() {
        let s = elevation_shadow(0);
        assert_eq!(s.blur, 0);
        assert_eq!(s.color.a(), 0);
    }

    #[test]
    fn elevation_clamps_above_24() {
        let a = elevation_shadow(24);
        let b = elevation_shadow(200);
        assert_eq!(a.offset, b.offset);
        assert_eq!(a.blur, b.blur);
    }

    // --- builders ---------------------------------------------------------

    #[test]
    fn defaults_match_mudblazor() {
        let d: MudDialog = MudDialog::new();
        assert!(d.title.is_none());
        assert!(d.title_content.is_none());
        assert_eq!(d.default_focus, DefaultFocus::Element);
        assert_eq!(d.options, DialogOptions::default());
        assert!(d.gutters);
        assert!(d.actions_class.is_none());
        assert!(d.content_class.is_none());
        assert!(d.content_style.is_none());
        assert!(d.title_class.is_none());
        assert!(matches!(d.color, Color::Default));
        assert!(d.on_backdrop_click.is_none());
    }

    #[test]
    fn builders_set_every_field() {
        let d: MudDialog = MudDialog::new()
            .title("T")
            .title_content(|_| {})
            .default_focus(DefaultFocus::FirstChild)
            .options(DialogOptions::new().with_close_button(true))
            .gutters(false)
            .actions_class("a")
            .content_class("c")
            .content_style("s")
            .title_class("t")
            .color(Color::Primary)
            .on_backdrop_click(|| {})
            .classes(["cls"]);

        assert_eq!(d.title.as_deref(), Some("T"));
        assert!(d.title_content.is_some());
        assert_eq!(d.default_focus, DefaultFocus::FirstChild);
        assert!(d.options.close_button);
        assert!(!d.gutters);
        assert_eq!(d.actions_class.as_deref(), Some("a"));
        assert_eq!(d.content_class.as_deref(), Some("c"));
        assert_eq!(d.content_style.as_deref(), Some("s"));
        assert_eq!(d.title_class.as_deref(), Some("t"));
        assert!(matches!(d.color, Color::Primary));
        assert!(d.on_backdrop_click.is_some());
        assert!(d.base.has_class("cls"));
    }
}