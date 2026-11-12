//! `MudTextField` — a themed single-line text field.
//!
//! Port of `MudTextField<T>.razor.cs` / `MudBaseInput<T>` /
//! `MudFormComponent<T>` (MudBlazor 8.13.0).
//!
//! # Pass 1 scope
//!
//! This is **pass 1** of the TextField port. It ships everything the
//! overlay family (`MudSelect`, `MudAutocomplete`, `Mud*Picker`, `MudMenu`)
//! needs to render an input-with-a-dropdown-chevron on top of it. See the
//! README for the full list of deferred parameters (pass 2).
//!
//! # MudBlazor parity — pass 1
//!
//! | MudBlazor parameter | Rust method | Default |
//! |---------------------|-------------|---------|
//! | `Value` / `ValueChanged` | `value: &'a mut String` argument of [`MudTextField::new`] | required |
//! | `Label`             | [`MudTextField::label`] | `None` |
//! | `Placeholder`       | [`MudTextField::placeholder`] | `None` |
//! | `HelperText`        | [`MudTextField::helper_text`] | `None` |
//! | `Error`             | [`MudTextField::error`] | `false` |
//! | `ErrorText`         | [`MudTextField::error_text`] | `None` |
//! | `Variant`           | [`MudTextField::variant`] | [`Variant::Text`] |
//! | `Margin`            | [`MudTextField::margin`] | [`Margin::None`] |
//! | `Color`             | [`MudTextField::color`] | [`Color::Primary`] |
//! | `Size`              | [`MudTextField::size`] | [`Size::Medium`] |
//! | `Disabled`          | [`MudTextField::disabled`] | `false` |
//! | `ReadOnly`          | [`MudTextField::readonly`] | `false` |
//! | `Clearable`         | [`MudTextField::clearable`] | `false` |
//! | `Adornment` + `AdornmentIcon` + `AdornmentText` | [`MudTextField::adornment_end`] | `None` |
//! | `AdornmentColor`    | [`MudTextField::adornment_end_color`] | `None` |
//! | `OnAdornmentClick`  | [`MudTextField::on_adornment_click`] | `None` |
//! | `Class` (base)      | [`MudTextField::classes`] | `None` |
//! | `Style` (base)      | *(use `classes`)* | — |
//! | `Tag` (base)        | **not ported** (no HTML tag) | — |
//! | `UserAttributes`    | **not ported** | — |
//!
//! ### Extensions (not in MudBlazor)
//!
//! | Extension | Description |
//! |-----------|-------------|
//! | [`id`](MudTextField::id) | Stable interaction id salt. |
//! | [`aria_label`](MudTextField::aria_label) | Accessible label overriding the caption. |
//! | [`clear_icon`](MudTextField::clear_icon) | Glyph used by the clear button. |
//! | [`TextFieldResponse`] | Rich return value with `changed` / `focus` / `submitted`. |
//!
//! ### Deferred to pass 2
//!
//! `InputType` (password / email / number), `Mask`, `Lines` / `AutoGrow` /
//! `MaxLines`, `Required` / `RequiredError`, `Counter` / `MaxLength`,
//! `DebounceInterval` / `Immediate`, `AdornmentStart` (leading adornment),
//! floating-label-with-border-cutout, `Typo` on the input body.
//!
//! # Text and icons
//!
//! The label, helper text, error text, and text-form adornment render
//! through [`MudText`]; icon-form adornments and the clear button render
//! through [`MudIcon`]. The actual editable text is rendered by
//! [`egui::TextEdit`] (its glyphs are egui's responsibility, exactly as
//! MudBlazor delegates to `<input>`).
//!
//! # Focus state
//!
//! egui's `TextEdit` stores focus in `egui::Memory` under its id. We give
//! the field a stable id and read the previous frame's focus state from
//! memory to paint the focus ring, so the border follows the *real* focus
//! state with a single frame of lag — imperceptible in practice, and the
//! only approach that works with `TextEdit`'s internal id derivation.
//!
//! # Click propagation
//!
//! The adornment and clear-button regions are *interactive* (they use their
//! own `ui.interact(..)` to detect hover / click), so their hit-rects are
//! **unioned** into the final [`TextFieldResponse::response`]. That means
//! callers (in particular `MudSelect`, whose trigger is a `readonly` text
//! field) can inspect `response.clicked()` and reliably detect a click on
//! *either* the edit area or the chevron. Without this union, clicking the
//! chevron would fire `on_adornment_click` but leave `response.clicked()`
//! `false` — the trigger for `MudSelect` would not toggle.
//!
//! # Not a `Widget` impl
//!
//! `MudTextField<'a>` holds `&'a mut String`, so it cannot implement
//! [`egui::Widget`] cleanly — a `Widget::ui(self)` call would need to
//! return the mutated string along with the response, which the trait
//! does not allow. Use [`MudTextField::show`] directly.

use egui::{
    pos2, vec2, Color32, CornerRadius, CursorIcon, Id, Layout, Rect, Response, Sense,
    Stroke, StrokeKind, Ui, UiBuilder, Vec2,
};
use mud_egui_bootstrap::bs_style::BsStyle;
use mud_egui_core::base::component_base::ComponentBase;
use mud_egui_core::enums::{Color, Margin, Size, StyleFlavor, Variant};
use mud_egui_icons_material::filled;
use mud_egui_tailwind::tw_style::{TwState, TwStyle};
use mud_egui_theming::models::palettes::palette::Palette;
use mud_egui_theming::theme::MudTheme;

use crate::data_display::icons::mud_icon::MudIcon;
use crate::typography::mud_text::MudText;

use super::adornment_end::AdornmentEnd;

// =============================================================================
// Constants
// =============================================================================

/// Base rem size passed to the Tailwind resolver.
const REM_SIZE_PT: f32 = 14.0;

/// Field height for `Size::Small`.
const SMALL_HEIGHT: f32 = 32.0;

/// Field height for `Size::Medium`.
const MEDIUM_HEIGHT: f32 = 40.0;

/// Field height for `Size::Large`.
const LARGE_HEIGHT: f32 = 48.0;

/// Vertical adjustment applied when `Margin::Dense`.
const MARGIN_DENSE_OFFSET: f32 = -4.0;

/// Vertical adjustment applied when `Margin::Normal`.
const MARGIN_NORMAL_OFFSET: f32 = 4.0;

/// Corner radius of the field box (Outlined / Filled).
const FIELD_RADIUS: u8 = 4;

/// Gap between the label caption and the field box.
const LABEL_GAP: f32 = 4.0;

/// Gap between the field box and the helper / error row.
const HELPER_GAP: f32 = 4.0;

// =============================================================================
// Pure helpers
// =============================================================================

/// Field height in logical pixels for a given [`Size`] and [`Margin`].
///
/// Base heights: `Small` = 32, `Medium` = 40, `Large` = 48.
/// `Margin::Dense` subtracts 4 px; `Margin::Normal` adds 4 px.
#[inline]
pub fn height_for(size: Size, margin: Margin) -> f32 {
    let base = match size {
        Size::Small => SMALL_HEIGHT,
        Size::Large => LARGE_HEIGHT,
        _ => MEDIUM_HEIGHT,
    };
    match margin {
        Margin::Dense => base + MARGIN_DENSE_OFFSET,
        Margin::Normal => base + MARGIN_NORMAL_OFFSET,
        Margin::None => base,
    }
        .max(0.0)
}

/// Font size in points for the input text.
#[inline]
pub fn font_for(size: Size) -> f32 {
    match size {
        Size::Small => 12.0,
        Size::Large => 16.0,
        _ => 14.0,
    }
}

/// Horizontal padding inside the field box.
#[inline]
pub fn padding_for(size: Size) -> f32 {
    match size {
        Size::Small => 8.0,
        Size::Large => 16.0,
        _ => 12.0,
    }
}

/// Resolves a semantic [`Color`] against the palette, falling back to `default`.
#[inline]
fn resolve_color(c: Color, palette: &Palette, default: Color32) -> Color32 {
    match c {
        Color::Default | Color::Inherit => default,
        other => other.resolve(palette).unwrap_or(default),
    }
}

#[inline]
fn with_alpha(c: Color32, alpha: f32) -> Color32 {
    if alpha >= 1.0 {
        return c;
    }
    if alpha <= 0.0 {
        return Color32::TRANSPARENT;
    }
    Color32::from_rgba_unmultiplied(
        c.r(),
        c.g(),
        c.b(),
        (c.a() as f32 * alpha).round() as u8,
    )
}

// =============================================================================
// Response
// =============================================================================

/// The result of one [`MudTextField::show`] frame.
///
/// `response` is the union of the outer field rect, the inner `TextEdit`, and
/// any adornment / clear-button regions, so `.clicked()` / `.hovered()`
/// reflect clicks anywhere in the field.
#[derive(Debug)]
pub struct TextFieldResponse {
    /// The combined field + edit + adornment response.
    pub response: Response,
    /// `true` when the user edited the value this frame.
    pub changed: bool,
    /// `true` when the field lost focus this frame.
    pub lost_focus: bool,
    /// `true` while the field has focus.
    pub has_focus: bool,
    /// `true` when the user pressed Enter while focused.
    pub submitted: bool,
}

// =============================================================================
// MudTextField
// =============================================================================

/// A themed single-line text field.
#[must_use = "MudTextField does nothing until you call `.show(ui)`"]
pub struct MudTextField<'a> {
    // ── Data ───────────────────────────────────────────────────────────────
    /// The caller-owned value. The field edits it in place.
    pub value: &'a mut String,

    // ── MudBlazor parameters ───────────────────────────────────────────────
    /// Label caption drawn above the field (`Label`).
    pub label: Option<String>,
    /// Placeholder shown when empty (`Placeholder`).
    pub placeholder: Option<String>,
    /// Helper text below the field (`HelperText`).
    pub helper_text: Option<String>,
    /// Error state (`Error`). Switches the accent to `palette.error`.
    pub error: bool,
    /// Error text below the field (`ErrorText`).
    pub error_text: Option<String>,
    /// Visual variant (`Variant`).
    pub variant: Variant,
    /// Vertical density (`Margin`).
    pub margin: Margin,
    /// Accent colour for focus / underline (`Color`).
    pub color: Color,
    /// Size preset (`Size`).
    pub size: Size,
    /// Disables the field (`Disabled`).
    pub disabled: bool,
    /// Prevents editing but keeps the visual state (`ReadOnly`).
    pub readonly: bool,
    /// Shows a clear button when the value is non-empty (`Clearable`).
    pub clearable: bool,
    /// Trailing adornment (`Adornment` + `AdornmentIcon` / `AdornmentText`).
    pub adornment_end: Option<AdornmentEnd>,
    /// Explicit colour for the trailing adornment (`AdornmentColor`).
    pub adornment_end_color: Option<Color>,
    /// Fires when the trailing adornment is clicked (`OnAdornmentClick`).
    pub on_adornment_click: Option<Box<dyn FnMut() + 'a>>,

    // ── Extensions ─────────────────────────────────────────────────────────
    /// Glyph for the clear button. Defaults to `filled::CLEAR`.
    pub clear_icon: String,
    /// Stable interaction id salt.
    pub id: Option<Id>,
    /// Accessible label. Falls back to [`label`](Self::label) when `None`.
    pub aria_label: Option<String>,

    // ── Utilities ──────────────────────────────────────────────────────────
    /// Utility classes on the wrapper.
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

impl<'a> MudTextField<'a> {
    /// Creates a text field editing `value`.
    pub fn new(value: &'a mut String) -> Self {
        Self {
            value,
            label: None,
            placeholder: None,
            helper_text: None,
            error: false,
            error_text: None,
            variant: Variant::Text,
            margin: Margin::None,
            color: Color::Primary,
            size: Size::Medium,
            disabled: false,
            readonly: false,
            clearable: false,
            adornment_end: None,
            adornment_end_color: None,
            on_adornment_click: None,
            clear_icon: filled::CLEAR.to_owned(),
            id: None,
            aria_label: None,
            base: ComponentBase::new(),
            flavor: StyleFlavor::default(),
            bs_style: BsStyle::default(),
            tw_style: TwStyle::default(),
            tw_state: TwState::default(),
        }
    }

    // ── Builders ───────────────────────────────────────────────────────────

    #[inline]
    pub fn label(mut self, s: impl Into<String>) -> Self {
        self.label = Some(s.into());
        self
    }

    #[inline]
    pub fn placeholder(mut self, s: impl Into<String>) -> Self {
        self.placeholder = Some(s.into());
        self
    }

    #[inline]
    pub fn helper_text(mut self, s: impl Into<String>) -> Self {
        self.helper_text = Some(s.into());
        self
    }

    #[inline]
    pub fn error(mut self, b: bool) -> Self {
        self.error = b;
        self
    }

    #[inline]
    pub fn error_text(mut self, s: impl Into<String>) -> Self {
        self.error_text = Some(s.into());
        self
    }

    #[inline]
    pub fn variant(mut self, v: Variant) -> Self {
        self.variant = v;
        self
    }

    #[inline]
    pub fn margin(mut self, m: Margin) -> Self {
        self.margin = m;
        self
    }

    #[inline]
    pub fn color(mut self, c: Color) -> Self {
        self.color = c;
        self
    }

    #[inline]
    pub fn size(mut self, s: Size) -> Self {
        self.size = s;
        self
    }

    #[inline]
    pub fn disabled(mut self, b: bool) -> Self {
        self.disabled = b;
        self
    }

    #[inline]
    pub fn readonly(mut self, b: bool) -> Self {
        self.readonly = b;
        self
    }

    #[inline]
    pub fn clearable(mut self, b: bool) -> Self {
        self.clearable = b;
        self
    }

    #[inline]
    pub fn clear_icon(mut self, svg: impl Into<String>) -> Self {
        self.clear_icon = svg.into();
        self
    }

    #[inline]
    pub fn adornment_end(mut self, a: AdornmentEnd) -> Self {
        self.adornment_end = Some(a);
        self
    }

    #[inline]
    pub fn adornment_end_color(mut self, c: Color) -> Self {
        self.adornment_end_color = Some(c);
        self
    }

    #[inline]
    pub fn on_adornment_click(mut self, f: impl FnMut() + 'a) -> Self {
        self.on_adornment_click = Some(Box::new(f));
        self
    }

    #[inline]
    pub fn id(mut self, id_salt: impl std::hash::Hash + std::fmt::Debug) -> Self {
        self.id = Some(Id::new(id_salt));
        self
    }

    #[inline]
    pub fn aria_label(mut self, s: impl Into<String>) -> Self {
        self.aria_label = Some(s.into());
        self
    }

    #[inline]
    pub fn classes<S: AsRef<str>>(mut self, classes: impl IntoIterator<Item = S>) -> Self {
        self.base = self.base.classes(classes);
        self
    }

    #[inline]
    pub fn flavor(mut self, f: StyleFlavor) -> Self {
        self.flavor = f;
        self
    }

    // ── Rendering ──────────────────────────────────────────────────────────

    /// Draws the field and returns a [`TextFieldResponse`].
    ///
    /// `mut self` is required because `BsStyle::build_frame` and
    /// `TwStyle::build_frame` take `&mut self` on their resolver caches.
    pub fn show(mut self, ui: &mut Ui) -> TextFieldResponse {
        let theme = MudTheme::fetch(ui.ctx()).unwrap_or_default();
        let palette = theme.current_palette();
        let lookup = |key: &str| palette.resolve(key).map(Color32::from);

        let mut tw_state = self.tw_state;
        tw_state.dark = ui.visuals().dark_mode;
        let frame = match self.flavor {
            StyleFlavor::Bootstrap => self.bs_style.build_frame(&self.base, &lookup),
            StyleFlavor::Tailwind => {
                self.tw_style.build_frame(&self.base, tw_state, REM_SIZE_PT, &lookup)
            }
        };

        frame.show(ui, |ui| self.render_inner(ui, palette)).inner
    }

    /// `mut self` is required because the adornment callback is invoked via
    /// `self.on_adornment_click.as_mut()`.
    fn render_inner(mut self, ui: &mut Ui, palette: &Palette) -> TextFieldResponse {
        let height = height_for(self.size, self.margin);
        let font = font_for(self.size);
        let pad = padding_for(self.size);

        let accent = resolve_color(self.color, palette, palette.primary.into());
        let error_color: Color32 = palette.error.into();
        let border_color: Color32 = palette.lines_inputs.into();
        let text_secondary: Color32 = palette.text_secondary.into();
        let text_disabled: Color32 = palette.text_disabled.into();
        let bg_gray: Color32 = palette.background_gray.into();
        let effective_accent = if self.error { error_color } else { accent };
        let is_disabled = self.disabled;
        let is_interactive = !self.disabled && !self.readonly;

        let adornment_color = match self.adornment_end_color {
            Some(c) => resolve_color(c, palette, text_secondary),
            None => text_secondary,
        };

        // Reserve ids up front.
        let field_id = self.id.unwrap_or_else(|| ui.next_auto_id());
        let edit_id = field_id.with("edit");
        let was_focused = ui.ctx().memory(|m| m.has_focus(edit_id));

        // Adornment / clear-button responses, unioned at the end so callers
        // can inspect `response.clicked()` and get `true` when the user
        // clicked the chevron or the clear button.
        let mut extra_response: Option<Response> = None;

        // ── Label (caption above) ──────────────────────────────────────────
        if let Some(label) = self.label.as_deref() {
            ui.add(
                MudText::new(label.to_owned())
                    .font_size(font * 0.85)
                    .custom_color(if self.error { error_color } else { text_secondary }),
            );
            ui.add_space(LABEL_GAP);
        }

        // ── Field body ─────────────────────────────────────────────────────
        let total_w = ui.available_width();
        let (field_rect, field_resp) =
            ui.allocate_exact_size(vec2(total_w, height), Sense::click());

        // Paint background (Filled only).
        if matches!(self.variant, Variant::Filled) {
            let bg = if is_disabled {
                with_alpha(bg_gray, 0.5)
            } else {
                bg_gray
            };
            ui.painter()
                .rect_filled(field_rect, CornerRadius::same(FIELD_RADIUS), bg);
        }

        // Paint border / underline.
        let hovered = field_resp.hovered() && !is_disabled;
        match self.variant {
            Variant::Outlined => {
                let color = if self.error {
                    error_color
                } else if was_focused {
                    effective_accent
                } else if hovered {
                    with_alpha(border_color, 0.85)
                } else {
                    border_color
                };
                let width = if was_focused || self.error { 2.0 } else { 1.0 };
                ui.painter().rect_stroke(
                    field_rect,
                    CornerRadius::same(FIELD_RADIUS),
                    Stroke::new(width, color),
                    StrokeKind::Inside,
                );
            }
            _ => {
                let color = if self.error {
                    error_color
                } else if was_focused {
                    effective_accent
                } else if hovered {
                    with_alpha(border_color, 0.85)
                } else {
                    border_color
                };
                let width = if was_focused || self.error { 2.0 } else { 1.0 };
                let y = field_rect.bottom() - width * 0.5;
                ui.painter().line_segment(
                    [pos2(field_rect.left(), y), pos2(field_rect.right(), y)],
                    Stroke::new(width, color),
                );
            }
        }

        // ── Adornment / clear button widths ────────────────────────────────
        let icon_px = height * 0.42;
        let adornment_w = self
            .adornment_end
            .as_ref()
            .map(|a| match a {
                AdornmentEnd::Icon(_) => height * 0.5,
                AdornmentEnd::Text(_) => {
                    let galley = ui.painter().layout_no_wrap(
                        a.content().to_owned(),
                        egui::FontId::proportional(font * 0.9),
                        text_secondary,
                    );
                    galley.size().x + 4.0
                }
            })
            .unwrap_or(0.0);

        let show_clear = self.clearable
            && !self.value.is_empty()
            && self.adornment_end.is_none()
            && !is_disabled
            && !self.readonly;
        let clear_w = if show_clear { height * 0.42 + 4.0 } else { 0.0 };

        // ── Edit rect ──────────────────────────────────────────────────────
        let edit_rect = Rect::from_min_max(
            pos2(field_rect.left() + pad, field_rect.top()),
            pos2(
                (field_rect.right() - pad - adornment_w - clear_w).max(field_rect.left() + pad),
                field_rect.bottom(),
            ),
        );

        // Clicking anywhere in the field focuses the edit.
        if field_resp.clicked() && is_interactive && !was_focused {
            ui.ctx().memory_mut(|m| m.request_focus(edit_id));
        }

        // ── TextEdit ───────────────────────────────────────────────────────
        let mut child = ui.new_child(
            UiBuilder::new()
                .max_rect(edit_rect)
                .layout(Layout::left_to_right(egui::Align::Center)),
        );
        child.set_clip_rect(edit_rect);

        let hint = self.placeholder.clone().unwrap_or_default();
        let text_color: Color32 = palette.text_primary.into();

        // egui 0.32+ `TextEdit::frame` takes a `Frame`, not a `bool`. Pass
        // `Frame::NONE` so the field draws no chrome of its own; the port
        // paints its own border/underline.
        let mut text_edit = egui::TextEdit::singleline(&mut *self.value)
            .frame(egui::Frame::NONE)
            .desired_width(edit_rect.width())
            .font(egui::FontId::proportional(font))
            .margin(egui::Margin::ZERO)
            .text_color(text_color)
            .hint_text(hint);
        if !is_interactive {
            text_edit = text_edit.interactive(false);
        }

        let edit_resp = child.push_id(edit_id, |ui| ui.add(text_edit)).inner;
        let is_focused = edit_resp.has_focus();
        let submitted = edit_resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));

        // ── Trailing adornment ─────────────────────────────────────────────
        if let Some(adornment) = self.adornment_end.clone() {
            let end_x = field_rect.right() - pad - adornment_w;
            let center_y = field_rect.center().y;
            match adornment {
                AdornmentEnd::Icon(svg) => {
                    let icon_rect = Rect::from_center_size(
                        pos2(end_x + adornment_w * 0.5, center_y),
                        Vec2::splat(height * 0.5),
                    );
                    let icon_resp = ui.interact(
                        icon_rect,
                        field_id.with("adornment"),
                        if is_disabled { Sense::hover() } else { Sense::click() },
                    );
                    if icon_resp.hovered() && !is_disabled {
                        ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
                    }
                    // Fire the callback; the response is captured below so
                    // `TextFieldResponse.response.clicked()` reflects it.
                    if icon_resp.clicked() && !is_disabled {
                        if let Some(cb) = self.on_adornment_click.as_mut() {
                            cb();
                        }
                    }
                    ui.put(
                        icon_rect,
                        MudIcon::new(svg)
                            .size_px(icon_px)
                            .custom_color(if is_disabled {
                                text_disabled
                            } else {
                                adornment_color
                            }),
                    );
                    extra_response = Some(icon_resp);
                }
                AdornmentEnd::Text(t) => {
                    let text_rect = Rect::from_min_size(
                        pos2(end_x, field_rect.top()),
                        vec2(adornment_w, height),
                    );
                    let mut text_ui = ui.new_child(
                        UiBuilder::new()
                            .max_rect(text_rect)
                            .layout(Layout::right_to_left(egui::Align::Center)),
                    );
                    text_ui.add(
                        MudText::new(t)
                            .font_size(font * 0.9)
                            .custom_color(adornment_color)
                            .inline(true)
                            .no_wrap(true),
                    );
                }
            }
        }

        // ── Clear button ───────────────────────────────────────────────────
        if show_clear {
            let clear_x = field_rect.right() - pad - clear_w * 0.5;
            let clear_rect = Rect::from_center_size(
                pos2(clear_x, field_rect.center().y),
                vec2(clear_w, height),
            );
            let clear_resp = ui.interact(clear_rect, field_id.with("clear"), Sense::click());
            if clear_resp.hovered() {
                ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
            }
            if clear_resp.clicked() {
                self.value.clear();
            }
            ui.put(
                clear_rect,
                MudIcon::new(self.clear_icon.clone())
                    .size_px(icon_px * 0.85)
                    .custom_color(text_secondary),
            );
            extra_response = Some(match extra_response {
                Some(existing) => existing.union(clear_resp),
                None => clear_resp,
            });
        }

        // ── Helper / error text ────────────────────────────────────────────
        let helper_text = if self.error {
            self.error_text.as_deref().or(self.helper_text.as_deref())
        } else {
            self.helper_text.as_deref()
        };
        if let Some(text) = helper_text {
            ui.add_space(HELPER_GAP);
            ui.add(
                MudText::new(text.to_owned())
                    .font_size(font * 0.85)
                    .custom_color(if self.error { error_color } else { text_secondary }),
            );
        }

        // Accessibility: publish the aria label (or label) on the edit response.
        let accessible_label = self
            .aria_label
            .as_deref()
            .or(self.label.as_deref())
            .unwrap_or("")
            .to_owned();
        if !accessible_label.is_empty() {
            let enabled = !is_disabled;
            edit_resp.clone().widget_info(|| {
                egui::WidgetInfo::labeled(
                    egui::Role::TextInput,
                    enabled,
                    accessible_label.as_str(),
                )
            });
        }

        // ── Final response union ───────────────────────────────────────────
        let mut combined = field_resp.union(edit_resp.clone());
        if let Some(extra) = extra_response {
            combined = combined.union(extra);
        }

        TextFieldResponse {
            response: combined,
            changed: edit_resp.changed(),
            lost_focus: edit_resp.lost_focus(),
            has_focus: is_focused,
            submitted,
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn height_for_default_sizes() {
        assert_eq!(height_for(Size::Small, Margin::None), 32.0);
        assert_eq!(height_for(Size::Medium, Margin::None), 40.0);
        assert_eq!(height_for(Size::Large, Margin::None), 48.0);
    }

    #[test]
    fn height_for_dense_subtracts_four() {
        assert_eq!(height_for(Size::Small, Margin::Dense), 28.0);
        assert_eq!(height_for(Size::Medium, Margin::Dense), 36.0);
        assert_eq!(height_for(Size::Large, Margin::Dense), 44.0);
    }

    #[test]
    fn height_for_normal_adds_four() {
        assert_eq!(height_for(Size::Small, Margin::Normal), 36.0);
        assert_eq!(height_for(Size::Medium, Margin::Normal), 44.0);
        assert_eq!(height_for(Size::Large, Margin::Normal), 52.0);
    }

    #[test]
    fn height_for_unknown_size_defaults_to_medium() {
        assert_eq!(height_for(Size::ExtraSmall, Margin::None), 40.0);
        assert_eq!(height_for(Size::ExtraLarge, Margin::None), 40.0);
    }

    #[test]
    fn height_for_clamps_to_zero_on_extreme_dense() {
        assert!(height_for(Size::Small, Margin::Dense) >= 0.0);
    }

    #[test]
    fn font_for_maps_sizes() {
        assert_eq!(font_for(Size::Small), 12.0);
        assert_eq!(font_for(Size::Medium), 14.0);
        assert_eq!(font_for(Size::Large), 16.0);
        assert_eq!(font_for(Size::ExtraSmall), 14.0);
        assert_eq!(font_for(Size::ExtraLarge), 14.0);
    }

    #[test]
    fn padding_for_maps_sizes() {
        assert_eq!(padding_for(Size::Small), 8.0);
        assert_eq!(padding_for(Size::Medium), 12.0);
        assert_eq!(padding_for(Size::Large), 16.0);
    }

    #[test]
    fn defaults_match_mudblazor() {
        let mut value = String::from("hi");
        let tf = MudTextField::new(&mut value);
        assert!(tf.label.is_none());
        assert!(tf.placeholder.is_none());
        assert!(tf.helper_text.is_none());
        assert!(!tf.error);
        assert!(tf.error_text.is_none());
        assert!(matches!(tf.variant, Variant::Text));
        assert!(matches!(tf.margin, Margin::None));
        assert!(matches!(tf.color, Color::Primary));
        assert!(matches!(tf.size, Size::Medium));
        assert!(!tf.disabled);
        assert!(!tf.readonly);
        assert!(!tf.clearable);
        assert!(tf.adornment_end.is_none());
        assert!(tf.adornment_end_color.is_none());
        assert!(tf.on_adornment_click.is_none());
        assert_eq!(tf.clear_icon, filled::CLEAR);
        assert!(tf.id.is_none());
        assert!(tf.aria_label.is_none());
    }

    #[test]
    fn new_holds_the_value() {
        let mut value = String::from("hello");
        let tf = MudTextField::new(&mut value);
        assert_eq!(*tf.value, "hello");
    }

    #[test]
    fn builders_set_every_field() {
        let mut value = String::from("hi");
        let tf = MudTextField::new(&mut value)
            .label("Name")
            .placeholder("Type…")
            .helper_text("Hint")
            .error(true)
            .error_text("Bad")
            .variant(Variant::Outlined)
            .margin(Margin::Dense)
            .color(Color::Error)
            .size(Size::Large)
            .disabled(true)
            .readonly(true)
            .clearable(true)
            .clear_icon("X")
            .adornment_end(AdornmentEnd::icon("M0 0"))
            .adornment_end_color(Color::Warning)
            .on_adornment_click(|| {})
            .id("my-field")
            .aria_label("Field")
            .classes(["my-class"]);

        assert_eq!(tf.label.as_deref(), Some("Name"));
        assert_eq!(tf.placeholder.as_deref(), Some("Type…"));
        assert_eq!(tf.helper_text.as_deref(), Some("Hint"));
        assert!(tf.error);
        assert_eq!(tf.error_text.as_deref(), Some("Bad"));
        assert!(matches!(tf.variant, Variant::Outlined));
        assert!(matches!(tf.margin, Margin::Dense));
        assert!(matches!(tf.color, Color::Error));
        assert!(matches!(tf.size, Size::Large));
        assert!(tf.disabled && tf.readonly && tf.clearable);
        assert_eq!(tf.clear_icon, "X");
        assert!(tf.adornment_end.is_some());
        assert_eq!(tf.adornment_end_color, Some(Color::Warning));
        assert!(tf.on_adornment_click.is_some());
        assert_eq!(tf.id, Some(Id::new("my-field")));
        assert_eq!(tf.aria_label.as_deref(), Some("Field"));
        assert!(tf.base.has_class("my-class"));
    }

    #[test]
    fn adornment_end_constructor_helpers() {
        let icon = AdornmentEnd::icon("M0 0");
        let text = AdornmentEnd::text("kg");
        assert!(icon.is_icon());
        assert!(!text.is_icon());
        assert_eq!(icon.content(), "M0 0");
        assert_eq!(text.content(), "kg");
    }

    #[test]
    fn resolve_color_default_and_inherit_use_fallback() {
        let theme = MudTheme::default();
        let palette = theme.current_palette();
        let fallback = Color32::from_rgb(1, 2, 3);
        assert_eq!(resolve_color(Color::Default, palette, fallback), fallback);
        assert_eq!(resolve_color(Color::Inherit, palette, fallback), fallback);
    }

    #[test]
    fn resolve_color_primary_maps_to_palette() {
        let theme = MudTheme::default();
        let palette = theme.current_palette();
        let primary: Color32 = palette.primary.into();
        let fallback = Color32::from_rgb(99, 0, 0);
        assert_eq!(resolve_color(Color::Primary, palette, fallback), primary);
    }

    #[test]
    fn with_alpha_full_and_zero() {
        let c = Color32::from_rgb(100, 150, 200);
        assert_eq!(with_alpha(c, 1.0), c);
        assert_eq!(with_alpha(c, 0.0), Color32::TRANSPARENT);
    }

    #[test]
    fn with_alpha_scales_the_alpha_channel() {
        let c = Color32::from_rgba_unmultiplied(100, 150, 200, 200);
        let half = with_alpha(c, 0.5);
        assert_eq!(half.r(), 100);
        assert_eq!(half.g(), 150);
        assert_eq!(half.b(), 200);
        assert_eq!(half.a(), 100);
    }
}