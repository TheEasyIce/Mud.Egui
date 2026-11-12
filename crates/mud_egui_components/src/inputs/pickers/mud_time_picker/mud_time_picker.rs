//! `MudTimePicker` — a text field with a popover clock dial.
//!
//! Port of `MudTimePicker.razor.cs` (MudBlazor 8.13.0). Pass 1.
//!
//! | MudBlazor parameter | Rust field / method |
//! |---------------------|---------------------|
//! | `AmPm` | [`MudTimePicker::am_pm`] (default `false`) |
//! | `AutoClose` | [`MudTimePicker::auto_close`] (default `false`) |
//! | `ClosingDelay` | [`MudTimePicker::closing_delay`] (default `200` ms) |
//! | `MinuteSelectionStep` | [`MudTimePicker::minute_selection_step`] (default `1`) |
//! | `OpenTo` | [`MudTimePicker::open_to`] (default `OpenTo::Hours`) |
//! | `PointerMoving` | dial `Response` state — suppresses hand animation |
//! | `Time` | [`TimePickerState::value`] (caller-owned) |
//! | `TimeChanged` | [`TimePickerResponse::action`] (`TimeSelected`) |
//! | `TimeEditMode` | [`MudTimePicker::time_edit_mode`] (default `Normal`) |
//! | `TimeFormat` | [`MudTimePicker::time_format`] (default `hh:mm tt` / `HH:mm`) |
//! | *base* `Adornment*` | [`MudTimePicker::adornment_icon`] / `adornment_color` |
//! | *base* `Clearable` … `Variant` | the shared picker surface |
//! | `Class` (base) | [`MudTimePicker::classes`] — stored, not resolved |
//! | `Style` / `Tag` / `UserAttributes` | not ported: no CSS strings, no HTML tag |
//! | `Mask` / typed parsing / `Inline` / `Static` | deferred to pass 2 |
//!
//! # Composition, not reimplementation
//!
//! | Primitive | Role |
//! |-----------|------|
//! | [`MudTextField`] | The trigger (adorned with a clock icon). |
//! | [`MudPopover`] | The clock dropdown. |
//! | [`MudText`] / [`MudIcon`] | Every numeral and every glyph. |
//!
//! Only the **clock dial** (hour ring, minute ring, hand, AM/PM markers) and
//! the **hour → minute commit logic** are new. [`SimpleTime`] supplies all
//! time arithmetic and [`TimePickerState`] the selection state.
//!
//! # Clock
//!
//! | Mode | Rings |
//! |------|-------|
//! | `am_pm(false)` (24-hour) | Outer `1..=12`, inner `13..=23` and `00` |
//! | `am_pm(true)` (12-hour) | Single `12, 1..=11` ring, plus AM / PM markers |
//!
//! The minute ring shows `0, 5, …, 55`; `minute_selection_step` snaps the
//! value to its nearest multiple.
//!
//! # Commit flow
//!
//! | `TimeEditMode` | Hour click | Minute click |
//! |----------------|------------|--------------|
//! | `Normal` (default) | Store `pending_hour`, switch to the minute ring | Commit `pending_hour : minute` |
//! | `OnlyHours` | Commit `hour : current_minute` immediately | Never reached |
//! | `OnlyMinutes` | Never shown | Commit `current_hour : minute` |
//!
//! `AutoClose` + `ClosingDelay` are honoured: the deadline is written to
//! `egui::Memory` and the popover closes when it elapses.
//!
//! # State
//!
//! [`TimePickerState::text`] is **not** re-synced from
//! [`TimePickerState::value`] on every frame — `commit()` / `clear()` /
//! `with_value()` keep the two in sync, so typed edits in `editable(true)`
//! mode survive. Call `state.commit(..)` to change the value
//! programmatically.

use std::f32::consts::{FRAC_PI_2, TAU};

use egui::{
    pos2, vec2, Color32, CornerRadius, CursorIcon, Id, Layout, Pos2, Rect, Response,
    Sense, Stroke, Ui, UiBuilder, Vec2,
};
use mud_egui_core::base::component_base::ComponentBase;
use mud_egui_core::enums::{
    AnchorOrigin, BackdropMode, Color, Margin, OpenTo, OverflowBehavior, PickerVariant,
    Size, StyleFlavor, TimeEditMode, TransformOrigin, Typo, Variant,
};
use mud_egui_icons_material::filled;
use mud_egui_theming::models::palettes::palette::Palette;
use mud_egui_theming::theme::MudTheme;

use crate::data_display::icons::mud_icon::MudIcon;
use crate::feedback::popover::MudPopover;
use crate::inputs::text_field::{AdornmentEnd, MudTextField};
use crate::typography::mud_text::MudText;

use super::simple_time::SimpleTime;
use super::time_picker_action::TimePickerAction;
use super::time_picker_state::TimePickerState;
use super::time_picker_view::TimePickerView;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Side length of the clock dial, in points.
pub const DIAL_SIZE: f32 = 240.0;

/// Radius of the outer ring (hours 1..=12 in 24-hour mode; all hours in
/// 12-hour mode; all minutes).
pub const OUTER_RADIUS: f32 = DIAL_SIZE * 0.44;

/// Radius of the inner ring (hours 13..=23 and 00 in 24-hour mode).
pub const INNER_RADIUS: f32 = DIAL_SIZE * 0.30;

/// Radius of the filled dot at the dial centre.
const CENTER_DOT_RADIUS: f32 = 3.0;

/// Radius of the circle at the end of the hand.
const HAND_CAP_RADIUS: f32 = 14.0;

/// Hand stroke width.
const HAND_STROKE: f32 = 2.0;

/// Hit-rect side used for every dial numeral (`MudText` inside).
const NUMERAL_BOX: f32 = 28.0;

/// Height reserved for the AM/PM marker row (12-hour mode only).
pub const AM_PM_ROW_HEIGHT: f32 = 32.0;

/// Seconds the hand takes to glide between two values.
const HAND_ANIM_SECONDS: f32 = 0.12;

/// Width of the clock hand ring outline.
const RING_STROKE: f32 = 1.0;

/// Default `ClosingDelay`, in milliseconds.
pub const DEFAULT_CLOSING_DELAY_MS: u32 = 200;

// ---------------------------------------------------------------------------
// Response
// ---------------------------------------------------------------------------

/// The result of one [`MudTimePicker::show`] frame.
#[derive(Debug)]
pub struct TimePickerResponse {
    /// The trigger's own `Response`.
    pub trigger: Response,
    /// Committed value after this frame.
    pub value: Option<SimpleTime>,
    /// Trigger text after this frame.
    pub text: String,
    /// What the picker did this frame.
    pub action: TimePickerAction,
}

// ---------------------------------------------------------------------------
// MudTimePicker
// ---------------------------------------------------------------------------

/// A text field with a popover clock dial.
///
/// Mirrors `MudTimePicker` in MudBlazor 8.13.0. The Blazor two-way binding
/// `Time` / `TimeChanged` becomes a caller-owned [`TimePickerState::value`]
/// plus a [`TimePickerResponse::action`] report — the same split
/// `MudDatePicker`, `MudRating` and `MudCheckBox` use.
#[must_use = "MudTimePicker does nothing until you call `.show(ui, state)`"]
pub struct MudTimePicker<'a> {
    // ── Trigger text field ─────────────────────────────────────────────────
    pub label: Option<String>,
    pub placeholder: Option<String>,
    pub helper_text: Option<String>,
    pub helper_text_on_focus: bool,
    pub variant: Variant,
    pub margin: Margin,
    pub color: Color,
    pub size: Size,
    pub icon_size: Size,
    pub clearable: bool,
    pub readonly: bool,
    pub disabled: bool,
    pub editable: bool,
    pub immediate_text: bool,

    // ── Adornment ──────────────────────────────────────────────────────────
    pub adornment_icon: String,
    pub adornment_color: Option<Color>,

    // ── Picker variant / display ───────────────────────────────────────────
    pub picker_variant: PickerVariant,
    pub elevation: Option<u8>,
    pub rounded: bool,
    pub square: bool,
    pub modal: bool,
    pub overflow_behavior: OverflowBehavior,
    pub show_toolbar: bool,

    // ── Time-specific ──────────────────────────────────────────────────────
    /// 12-hour clock with AM/PM markers. Mirrors `AmPm` (default `false`).
    pub am_pm: bool,
    /// Close after the value is set. Mirrors `AutoClose` (default `false`).
    pub auto_close: bool,
    /// Milliseconds to wait before closing. Mirrors `ClosingDelay`.
    pub closing_delay: u32,
    /// Step interval for minute selection. Mirrors `MinuteSelectionStep`.
    pub minute_selection_step: u8,
    /// Initial dial. Mirrors `OpenTo` (default `OpenTo::Hours`).
    pub open_to: OpenTo,
    /// Which parts are editable. Mirrors `TimeEditMode`.
    pub time_edit_mode: TimeEditMode,
    /// Display template. Mirrors `TimeFormat`; `None` selects the default
    /// (`hh:mm tt` when `am_pm`, else `HH:mm`).
    pub time_format: Option<String>,

    // ── Customisation ──────────────────────────────────────────────────────
    /// Custom footer row inside the popover. Returns `Some(action)` to
    /// request a state change — applied after the popover closure returns.
    pub picker_actions:
        Option<Box<dyn FnMut(&mut Ui) -> Option<TimePickerAction> + 'a>>,
    pub on_picker_opened: Option<Box<dyn FnMut() + 'a>>,
    pub on_picker_closed: Option<Box<dyn FnMut() + 'a>>,
    pub on_click: Option<Box<dyn FnMut() + 'a>>,

    // ── Utilities ──────────────────────────────────────────────────────────
    pub base: ComponentBase,
    pub flavor: StyleFlavor,
}

impl<'a> Default for MudTimePicker<'a> {
    fn default() -> Self {
        Self {
            label: None,
            placeholder: None,
            helper_text: None,
            helper_text_on_focus: false,
            variant: Variant::Text,
            margin: Margin::None,
            color: Color::Primary,
            size: Size::Medium,
            icon_size: Size::Medium,
            clearable: false,
            readonly: false,
            disabled: false,
            editable: false,
            immediate_text: false,
            adornment_icon: filled::ACCESS_TIME.to_owned(),
            adornment_color: None,
            picker_variant: PickerVariant::Dialog,
            elevation: None,
            rounded: false,
            square: false,
            modal: true,
            overflow_behavior: OverflowBehavior::Flip,
            show_toolbar: true,
            am_pm: false,
            auto_close: false,
            closing_delay: DEFAULT_CLOSING_DELAY_MS,
            minute_selection_step: 1,
            open_to: OpenTo::Hours,
            time_edit_mode: TimeEditMode::Normal,
            time_format: None,
            picker_actions: None,
            on_picker_opened: None,
            on_picker_closed: None,
            on_click: None,
            base: ComponentBase::new(),
            flavor: StyleFlavor::default(),
        }
    }
}

impl<'a> MudTimePicker<'a> {
    /// Creates a picker with MudBlazor's default settings.
    pub fn new() -> Self {
        Self::default()
    }

    // ── Builders ───────────────────────────────────────────────────────────

    #[inline] pub fn label(mut self, s: impl Into<String>) -> Self { self.label = Some(s.into()); self }
    #[inline] pub fn placeholder(mut self, s: impl Into<String>) -> Self { self.placeholder = Some(s.into()); self }
    #[inline] pub fn helper_text(mut self, s: impl Into<String>) -> Self { self.helper_text = Some(s.into()); self }
    #[inline] pub fn helper_text_on_focus(mut self, b: bool) -> Self { self.helper_text_on_focus = b; self }
    #[inline] pub fn variant(mut self, v: Variant) -> Self { self.variant = v; self }
    #[inline] pub fn margin(mut self, m: Margin) -> Self { self.margin = m; self }
    #[inline] pub fn color(mut self, c: Color) -> Self { self.color = c; self }
    #[inline] pub fn size(mut self, s: Size) -> Self { self.size = s; self }
    #[inline] pub fn icon_size(mut self, s: Size) -> Self { self.icon_size = s; self }
    #[inline] pub fn clearable(mut self, b: bool) -> Self { self.clearable = b; self }
    #[inline] pub fn readonly(mut self, b: bool) -> Self { self.readonly = b; self }
    #[inline] pub fn disabled(mut self, b: bool) -> Self { self.disabled = b; self }
    #[inline] pub fn editable(mut self, b: bool) -> Self { self.editable = b; self }
    #[inline] pub fn immediate_text(mut self, b: bool) -> Self { self.immediate_text = b; self }
    #[inline] pub fn adornment_icon(mut self, s: impl Into<String>) -> Self { self.adornment_icon = s.into(); self }
    #[inline] pub fn adornment_color(mut self, c: Color) -> Self { self.adornment_color = Some(c); self }
    #[inline] pub fn picker_variant(mut self, v: PickerVariant) -> Self { self.picker_variant = v; self }
    #[inline] pub fn elevation(mut self, e: u8) -> Self { self.elevation = Some(e); self }
    #[inline] pub fn rounded(mut self, b: bool) -> Self { self.rounded = b; self }
    #[inline] pub fn square(mut self, b: bool) -> Self { self.square = b; self }
    #[inline] pub fn modal(mut self, b: bool) -> Self { self.modal = b; self }
    #[inline] pub fn overflow_behavior(mut self, b: OverflowBehavior) -> Self { self.overflow_behavior = b; self }
    #[inline] pub fn show_toolbar(mut self, b: bool) -> Self { self.show_toolbar = b; self }
    #[inline] pub fn am_pm(mut self, b: bool) -> Self { self.am_pm = b; self }
    #[inline] pub fn auto_close(mut self, b: bool) -> Self { self.auto_close = b; self }
    #[inline] pub fn closing_delay(mut self, ms: u32) -> Self { self.closing_delay = ms; self }
    #[inline] pub fn minute_selection_step(mut self, step: u8) -> Self { self.minute_selection_step = step.max(1); self }
    #[inline] pub fn open_to(mut self, o: OpenTo) -> Self { self.open_to = o; self }
    #[inline] pub fn time_edit_mode(mut self, m: TimeEditMode) -> Self { self.time_edit_mode = m; self }
    #[inline] pub fn time_format(mut self, s: impl Into<String>) -> Self { self.time_format = Some(s.into()); self }
    #[inline] pub fn picker_actions(mut self, f: impl FnMut(&mut Ui) -> Option<TimePickerAction> + 'a) -> Self { self.picker_actions = Some(Box::new(f)); self }
    #[inline] pub fn on_picker_opened(mut self, f: impl FnMut() + 'a) -> Self { self.on_picker_opened = Some(Box::new(f)); self }
    #[inline] pub fn on_picker_closed(mut self, f: impl FnMut() + 'a) -> Self { self.on_picker_closed = Some(Box::new(f)); self }
    #[inline] pub fn on_click(mut self, f: impl FnMut() + 'a) -> Self { self.on_click = Some(Box::new(f)); self }
    #[inline] pub fn classes<S: AsRef<str>>(mut self, classes: impl IntoIterator<Item = S>) -> Self { self.base = self.base.classes(classes); self }
    #[inline] pub fn flavor(mut self, f: StyleFlavor) -> Self { self.flavor = f; self }

    // ── Rendering ──────────────────────────────────────────────────────────

    /// Draws the trigger and, when open, the clock popover.
    pub fn show(mut self, ui: &mut Ui, state: &mut TimePickerState) -> TimePickerResponse {
        // 0. Honour any AutoClose deadline written on an earlier frame.
        //    Read before `MudTextField::new` takes `&mut state.text`.
        let now = ui.input(|i| i.time);
        let close_at: Option<f64> = ui
            .ctx()
            .memory(|m| m.data.get_temp(state.popover.id.with("close_at")));
        let mut action = TimePickerAction::None;
        if let Some(deadline) = close_at {
            if now >= deadline && state.popover.open {
                state.popover.open = false;
                state.pending_hour = None;
                ui.ctx()
                    .memory_mut(|m| m.data.remove_temp::<f64>(state.popover.id.with("close_at")));
                if let Some(cb) = self.on_picker_closed.as_mut() { cb(); }
                action = TimePickerAction::Closed;
            }
        }

        // 1. Render the trigger.
        //    Snapshot the two `state` fields the trigger needs while the
        //    mutable borrow of `state.text` is live.
        let was_open = state.popover.open;
        let text_is_empty = state.text.is_empty();
        let show_helper = self.helper_text.is_some()
            && (!self.helper_text_on_focus || !text_is_empty || was_open);
        let trigger_icon_color = self.adornment_color.unwrap_or(Color::Inherit);

        let mut trigger = MudTextField::new(&mut state.text)
            .disabled(self.disabled)
            .readonly(self.readonly || !self.editable)
            .variant(self.variant)
            .margin(self.margin)
            .color(self.color)
            .size(self.size)
            .adornment_end(AdornmentEnd::icon(self.adornment_icon.clone()))
            .adornment_end_color(trigger_icon_color)
            .clear_icon(filled::CLEAR.to_owned());
        if let Some(s) = self.label.clone() { trigger = trigger.label(s); }
        if let Some(s) = self.placeholder.clone() { trigger = trigger.placeholder(s); }
        if show_helper {
            if let Some(s) = self.helper_text.clone() { trigger = trigger.helper_text(s); }
        }
        if self.clearable { trigger = trigger.clearable(true); }
        let trigger_resp = trigger.show(ui);

        // 2. Trigger interaction. Clear detection runs first so a clear click
        //    does not also open the popover.
        let clear_clicked = self.clearable
            && trigger_resp.changed
            && state.text.is_empty()
            && state.value.is_some();

        if !clear_clicked
            && trigger_resp.response.clicked()
            && !self.disabled
            && self.picker_variant.is_dialog()
        {
            state.toggle(self.open_to);
            if let Some(cb) = self.on_click.as_mut() { cb(); }
        }

        let now_open = state.popover.open;
        if now_open != was_open {
            if now_open {
                if let Some(cb) = self.on_picker_opened.as_mut() { cb(); }
                action = TimePickerAction::Opened;
            } else {
                if let Some(cb) = self.on_picker_closed.as_mut() { cb(); }
                action = TimePickerAction::Closed;
            }
        }

        if clear_clicked {
            state.clear();
            action = TimePickerAction::Cleared;
        }

        // 3. Popover + dial.
        if state.popover.open && self.picker_variant.is_dialog() {
            let theme = MudTheme::fetch(ui.ctx()).unwrap_or_default();
            let palette = theme.current_palette();
            let picker_palette = resolve_palette(self.color, palette, ui);

            let mut popover = MudPopover::new()
                .anchor_origin(AnchorOrigin::Bottom)
                .transform_origin(TransformOrigin::TopLeft)
                .paper(true)
                .elevation(self.elevation.unwrap_or(8))
                .overflow_behavior(self.overflow_behavior)
                .size_hint(vec2(DIAL_SIZE + 32.0, DIAL_SIZE + 80.0))
                .match_anchor_width(false);
            if self.modal {
                popover = popover.backdrop(BackdropMode::Transparent);
            }
            if self.square {
                popover = popover.square(true);
            }

            let child_rect = trigger_resp.response.rect;

            // Deferred interactions: collected inside the closure, applied
            // after `popover.show` releases `&mut state.popover`.
            let mut clicked_hour: Option<u8> = None;
            let mut clicked_minute: Option<u8> = None;
            let mut am_pm_flip = false;
            let mut deferred_action: Option<TimePickerAction> = None;

            let mut picker_actions = self.picker_actions.take();

            let view = state.view;
            let pending_hour = state.pending_hour;
            let value = state.value;
            let am_pm = self.am_pm;
            let step = self.minute_selection_step;
            let edit_mode = self.time_edit_mode;

            let _ = popover.show(ui, &mut state.popover, child_rect, |pop_ui| {
                render_clock(
                    pop_ui,
                    view,
                    value,
                    pending_hour,
                    am_pm,
                    step,
                    &picker_palette,
                    &mut clicked_hour,
                    &mut clicked_minute,
                    &mut am_pm_flip,
                );
                if let Some(actions) = picker_actions.as_mut() {
                    if let Some(a) = actions(pop_ui) {
                        deferred_action = Some(a);
                    }
                }
            });

            self.picker_actions = picker_actions;

            // 4. Apply the deferred interactions.
            let fmt = self.effective_format();
            let mut committed: Option<SimpleTime> = None;

            if am_pm_flip {
                let base = state.effective_hour();
                state.pending_hour = Some(flip_am_pm(base));
                if edit_mode.commits_after_hour() {
                    let minute = state.value.map(|t| t.minutes).unwrap_or(0);
                    committed = Some(SimpleTime::hm(state.pending_hour.unwrap_or(base), minute));
                }
            }

            if let Some(h) = clicked_hour {
                if edit_mode.commits_after_hour() {
                    let minute = state.value.map(|t| t.minutes).unwrap_or(0);
                    committed = Some(SimpleTime::hm(h, minute));
                } else {
                    state.pending_hour = Some(h);
                    state.view = TimePickerView::Minutes;
                }
            }

            if let Some(m) = clicked_minute {
                let hour = state.effective_hour();
                committed = Some(SimpleTime::hm(hour, m));
            }

            if let Some(time) = committed {
                state.commit(time, time.format(fmt));
                action = TimePickerAction::TimeSelected(time);
                if let Some(cb) = self.on_picker_closed.as_mut() { cb(); }
                if self.auto_close {
                    let deadline = now + (self.closing_delay as f64) / 1000.0;
                    ui.ctx().memory_mut(|m| {
                        m.data
                            .insert_temp(state.popover.id.with("close_at"), deadline)
                    });
                    // Keep the popover open until the deadline; the top of
                    // the next frame closes it.
                    state.popover.open = true;
                }
            } else if let Some(deferred) = deferred_action {
                match deferred {
                    TimePickerAction::TimeSelected(time) => {
                        state.commit(time, time.format(fmt));
                        action = TimePickerAction::TimeSelected(time);
                        if let Some(cb) = self.on_picker_closed.as_mut() { cb(); }
                    }
                    TimePickerAction::Cleared => {
                        state.clear();
                        action = TimePickerAction::Cleared;
                        if let Some(cb) = self.on_picker_closed.as_mut() { cb(); }
                    }
                    other => action = other,
                }
            }
        }

        TimePickerResponse {
            trigger: trigger_resp.response,
            value: state.value,
            text: state.text.clone(),
            action,
        }
    }

    /// The display template actually used: `time_format` when set and
    /// non-empty, else `hh:mm tt` (12-hour) / `HH:mm` (24-hour).
    fn effective_format(&self) -> &str {
        match self.time_format.as_deref() {
            Some(s) if !s.is_empty() => s,
            _ => {
                if self.am_pm {
                    "hh:mm tt"
                } else {
                    "HH:mm"
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Pure helpers
// ---------------------------------------------------------------------------

/// Flips the hour across noon: `h < 12 → h + 12`, `h >= 12 → h - 12`.
#[inline]
fn flip_am_pm(hour: u8) -> u8 {
    if hour >= 12 {
        hour - 12
    } else {
        hour + 12
    }
}

/// Snaps `minute` to the nearest multiple of `step`, wrapping into `0..60`.
/// `step <= 1` is a no-op.
pub fn snap_minute(minute: u8, step: u8) -> u8 {
    if step <= 1 {
        return minute.min(59);
    }
    let step = step.min(60) as f32;
    let snapped = ((minute as f32 / step).round() * step) as i32;
    snapped.rem_euclid(60) as u8
}

/// Clock angle of `p` around `center`, in radians. `0` is 12 o'clock and the
/// angle grows clockwise (egui's y axis points down).
#[inline]
pub fn clock_angle(center: Pos2, p: Pos2) -> f32 {
    let d = p - center;
    (d.y.atan2(d.x) + FRAC_PI_2).rem_euclid(TAU)
}

/// The 0-based clock index (`0` = 12 o'clock) nearest to `angle`.
#[inline]
fn clock_index(angle: f32) -> u8 {
    ((angle / (TAU / 12.0)).round() as i32).rem_euclid(12) as u8
}

/// Hour value at clock index `index` on the outer ring (`0 → 12`, `1 → 1`,
/// … `11 → 11`).
#[inline]
pub fn outer_hour_for_index(index: u8) -> u8 {
    match index % 12 {
        0 => 12,
        i => i,
    }
}

/// Hour value at clock index `index` on the inner 24-hour ring (`0 → 0`,
/// `1 → 13`, … `11 → 23`).
#[inline]
pub fn inner_hour_for_index(index: u8) -> u8 {
    match index % 12 {
        0 => 0,
        i => 12 + i,
    }
}

/// `true` when `p` falls on the outer ring.
#[inline]
pub fn is_outer_ring(center: Pos2, p: Pos2) -> bool {
    (p - center).length() > (INNER_RADIUS + OUTER_RADIUS) * 0.5
}

/// Position of clock index `index` at radius `r` around `center`.
#[inline]
pub fn numeral_pos(center: Pos2, r: f32, index: u8) -> Pos2 {
    let a = (index % 12) as f32 * (TAU / 12.0);
    pos2(center.x + a.sin() * r, center.y - a.cos() * r)
}

/// Which hour was clicked, if the click lands on the hour ring.
///
/// `am_pm` selects the 12-hour (single ring) or 24-hour (two rings) mapping.
pub fn hour_at(center: Pos2, p: Pos2, am_pm: bool) -> Option<u8> {
    if (p - center).length() < INNER_RADIUS * 0.5 {
        return None;
    }
    let index = clock_index(clock_angle(center, p));
    if am_pm {
        Some(outer_hour_for_index(index))
    } else if is_outer_ring(center, p) {
        Some(outer_hour_for_index(index))
    } else {
        Some(inner_hour_for_index(index))
    }
}

/// Which minute was clicked, snapped to `step`.
pub fn minute_at(center: Pos2, p: Pos2, step: u8) -> Option<u8> {
    if (p - center).length() < INNER_RADIUS * 0.5 {
        return None;
    }
    let raw = ((clock_angle(center, p) / (TAU / 60.0)).round() as i32)
        .rem_euclid(60) as u8;
    Some(snap_minute(raw, step))
}

// ---------------------------------------------------------------------------
// Palette resolution
// ---------------------------------------------------------------------------

struct PickerPalette {
    text_primary: Color32,
    text_secondary: Color32,
    text_disabled: Color32,
    accent: Color32,
    accent_text: Color32,
    hover_fill: Color32,
    divider: Color32,
}

fn resolve_palette(color: Color, palette: &Palette, ui: &Ui) -> PickerPalette {
    let accent = match color {
        Color::Default | Color::Inherit => palette.primary.into(),
        other => other
            .resolve(palette)
            .map(Color32::from)
            .unwrap_or_else(|| palette.primary.into()),
    };
    PickerPalette {
        text_primary: palette.text_primary.into(),
        text_secondary: palette.text_secondary.into(),
        text_disabled: palette.text_disabled.into(),
        accent,
        accent_text: palette.primary_contrast_text.into(),
        hover_fill: ui.visuals().widgets.hovered.weak_bg_fill,
        divider: palette.divider.into(),
    }
}

// ---------------------------------------------------------------------------
// Clock rendering
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
fn render_clock(
    ui: &mut Ui,
    view: TimePickerView,
    value: Option<SimpleTime>,
    pending_hour: Option<u8>,
    am_pm: bool,
    minute_step: u8,
    p: &PickerPalette,
    clicked_hour: &mut Option<u8>,
    clicked_minute: &mut Option<u8>,
    am_pm_flip: &mut bool,
) {
    let body_h = match view {
        TimePickerView::Hours if am_pm => DIAL_SIZE + AM_PM_ROW_HEIGHT,
        _ => DIAL_SIZE,
    };
    let (rect, resp) = ui.allocate_exact_size(vec2(DIAL_SIZE, body_h), Sense::click_and_drag());
    let center = pos2(rect.center().x, rect.top() + DIAL_SIZE * 0.5);
    let pointer_moving = resp.dragged() || resp.is_pointer_button_down_on();

    // 1. Ring outlines + centre dot.
    //
    // The painter block is *scoped* so the immutable borrow of `*ui` it
    // takes is released before `draw_numeral(ui, ..)` needs `&mut ui`.
    {
        let painter = ui.painter();
        painter.circle_stroke(
            center,
            OUTER_RADIUS,
            Stroke::new(RING_STROKE, p.divider),
        );
        if view == TimePickerView::Hours && !am_pm {
            painter.circle_stroke(
                center,
                INNER_RADIUS,
                Stroke::new(RING_STROKE, p.divider),
            );
        }
        painter.circle_filled(center, CENTER_DOT_RADIUS, p.accent);
    }

    // 2. Numerals — every call needs `&mut ui`.
    match view {
        TimePickerView::Hours => {
            for index in 0..12u8 {
                let outer_value = outer_hour_for_index(index);
                let selected = match (pending_hour, value) {
                    (Some(h), _) => h == outer_value,
                    (None, Some(t)) => t.hours == outer_value,
                    _ => false,
                };
                draw_numeral(
                    ui,
                    numeral_pos(center, OUTER_RADIUS, index),
                    outer_value.to_string(),
                    selected,
                    p,
                );
                if !am_pm {
                    let inner_value = inner_hour_for_index(index);
                    let inner_selected = match (pending_hour, value) {
                        (Some(h), _) => h == inner_value,
                        (None, Some(t)) => t.hours == inner_value,
                        _ => false,
                    };
                    draw_numeral(
                        ui,
                        numeral_pos(center, INNER_RADIUS, index),
                        format!("{inner_value:02}"),
                        inner_selected,
                        p,
                    );
                }
            }
        }
        TimePickerView::Minutes => {
            for index in 0..12u8 {
                let minute = snap_minute(index * 5, minute_step);
                let selected = value.map(|t| t.minutes == minute).unwrap_or(false);
                draw_numeral(
                    ui,
                    numeral_pos(center, OUTER_RADIUS, index),
                    format!("{minute:02}"),
                    selected,
                    p,
                );
            }
        }
    }

    // 3. Hand — the painter borrow is scoped so it ends before the AM/PM
    //    block calls `draw_am_pm_slot(ui, ..)`.
    {
        let hand_target = hand_angle(view, value, pending_hour);
        let hand_id = ui
            .scope_id()
            .with(("hand", view == TimePickerView::Minutes));
        let angle = if pointer_moving {
            hand_target
        } else {
            ui.ctx()
                .animate_value_with_time(hand_id, hand_target, HAND_ANIM_SECONDS)
        };
        let tip = pos2(
            center.x + angle.sin() * OUTER_RADIUS,
            center.y - angle.cos() * OUTER_RADIUS,
        );
        let painter = ui.painter();
        painter.line_segment([center, tip], Stroke::new(HAND_STROKE, p.accent));
        painter.circle_filled(tip, HAND_CAP_RADIUS, p.accent);
    }

    // 4. AM/PM markers — every call needs `&mut ui`.
    if view == TimePickerView::Hours && am_pm {
        let is_pm = value.map(|t| t.is_pm()).unwrap_or(false);
        let row = Rect::from_min_max(
            pos2(rect.left(), rect.bottom() - AM_PM_ROW_HEIGHT),
            pos2(rect.right(), rect.bottom()),
        );
        let half = row.width() * 0.5;
        let am_rect = Rect::from_min_size(row.min, vec2(half, row.height()));
        let pm_rect =
            Rect::from_min_size(pos2(row.left() + half, row.top()), vec2(half, row.height()));
        if draw_am_pm_slot(ui, am_rect, "AM", !is_pm, p).clicked() && is_pm {
            *am_pm_flip = true;
        }
        if draw_am_pm_slot(ui, pm_rect, "PM", is_pm, p).clicked() && !is_pm {
            *am_pm_flip = true;
        }
    }

    // 5. Click / drag input.
    if let Some(pos) = resp.interact_pointer_pos() {
        if let Some(h) = hour_at(center, pos, am_pm)
            .filter(|_| view == TimePickerView::Hours)
        {
            if resp.clicked() || pointer_moving {
                *clicked_hour = Some(h);
            }
        } else if let Some(m) = minute_at(center, pos, minute_step)
            .filter(|_| view == TimePickerView::Minutes)
        {
            if resp.clicked() || pointer_moving {
                *clicked_minute = Some(m);
            }
        }
    }

    if resp.hovered() {
        ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
    }
}

/// Hand angle (radians, `0` = 12 o'clock, clockwise) for the current view.
fn hand_angle(
    view: TimePickerView,
    value: Option<SimpleTime>,
    pending_hour: Option<u8>,
) -> f32 {
    match view {
        TimePickerView::Hours => {
            let h = pending_hour
                .or_else(|| value.map(|t| t.hours))
                .unwrap_or(0);
            (h % 12) as f32 * (TAU / 12.0)
        }
        TimePickerView::Minutes => {
            let m = value.map(|t| t.minutes).unwrap_or(0);
            (m as f32) * (TAU / 60.0)
        }
    }
}

/// Draws one dial numeral through `MudText` (Rule 3).
fn draw_numeral(ui: &mut Ui, center: Pos2, label: String, selected: bool, p: &PickerPalette) {
    let rect = Rect::from_center_size(center, Vec2::splat(NUMERAL_BOX));
    let color = if selected { p.accent_text } else { p.text_primary };
    if selected {
        ui.painter()
            .circle_filled(center, NUMERAL_BOX * 0.42, p.accent);
    }
    let mut child = ui.new_child(
        UiBuilder::new()
            .max_rect(rect)
            .layout(Layout::centered_and_justified(egui::Direction::LeftToRight)),
    );
    child.add(
        MudText::new(label)
            .typo(Typo::Body2)
            .custom_color(color),
    );
}

/// Draws one AM/PM marker through `MudText` (Rule 3) and returns its
/// `Response`.
fn draw_am_pm_slot(ui: &mut Ui, rect: Rect, label: &str, active: bool, p: &PickerPalette) -> Response {
    let id = ui.scope_id().with(("am_pm", label));
    let resp = ui.interact(rect, id, Sense::click());
    let color = if active { p.accent } else { p.text_secondary };
    if resp.hovered() {
        ui.painter()
            .rect_filled(rect.shrink(2.0), CornerRadius::same(4), p.hover_fill);
        ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
    }
    if active {
        let y = rect.bottom() - 3.0;
        ui.painter().line_segment(
            [pos2(rect.center().x - 8.0, y), pos2(rect.center().x + 8.0, y)],
            Stroke::new(2.0, p.accent),
        );
    }
    let mut child = ui.new_child(
        UiBuilder::new()
            .max_rect(rect)
            .layout(Layout::centered_and_justified(egui::Direction::LeftToRight)),
    );
    child.add(MudText::new(label).typo(Typo::Body2).custom_color(color));
    resp
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn c() -> Pos2 {
        pos2(120.0, 120.0)
    }

    // --- flip / snap ------------------------------------------------------

    #[test]
    fn flip_am_pm_crosses_noon() {
        assert_eq!(flip_am_pm(0), 12);
        assert_eq!(flip_am_pm(9), 21);
        assert_eq!(flip_am_pm(12), 0);
        assert_eq!(flip_am_pm(21), 9);
    }

    #[test]
    fn snap_minute_handles_step_one_and_zero() {
        for m in 0..60u8 {
            assert_eq!(snap_minute(m, 0), m);
            assert_eq!(snap_minute(m, 1), m);
        }
    }

    #[test]
    fn snap_minute_rounds_to_nearest_multiple() {
        assert_eq!(snap_minute(7, 15), 0);
        assert_eq!(snap_minute(8, 15), 15);
        assert_eq!(snap_minute(22, 15), 15);
        assert_eq!(snap_minute(23, 15), 30);
        assert_eq!(snap_minute(59, 15), 0); // wraps
        assert_eq!(snap_minute(30, 30), 30);
    }

    // --- angles -----------------------------------------------------------

    #[test]
    fn clock_angle_zero_at_twelve_oclock() {
        let a = clock_angle(c(), pos2(120.0, 20.0)); // straight up
        assert!(a.abs() < 1e-4, "expected ~0, got {a}");
    }

    #[test]
    fn clock_angle_grows_clockwise() {
        let three = clock_angle(c(), pos2(220.0, 120.0));
        assert!((three - FRAC_PI_2).abs() < 1e-4);
        let six = clock_angle(c(), pos2(120.0, 220.0));
        assert!((six - std::f32::consts::PI).abs() < 1e-4);
        let nine = clock_angle(c(), pos2(20.0, 120.0));
        assert!((nine - (3.0 * FRAC_PI_2)).abs() < 1e-4);
    }

    #[test]
    fn clock_index_maps_each_slot() {
        for i in 0..12u8 {
            let a = i as f32 * (TAU / 12.0);
            assert_eq!(clock_index(a), i);
        }
    }

    // --- hour / minute at ------------------------------------------------

    #[test]
    fn outer_hour_maps_twelve_to_twelve() {
        let map: Vec<u8> = (0..12).map(outer_hour_for_index).collect();
        assert_eq!(map, vec![12, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);
    }

    #[test]
    fn inner_hour_maps_zero_and_thirteen_to_twentythree() {
        let map: Vec<u8> = (0..12).map(inner_hour_for_index).collect();
        assert_eq!(map, vec![0, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23]);
    }

    #[test]
    fn hour_at_reads_outer_ring_in_24_hour_mode() {
        let p = numeral_pos(c(), OUTER_RADIUS, 3); // 3 o'clock
        assert_eq!(hour_at(c(), p, false), Some(3));
        let p = numeral_pos(c(), OUTER_RADIUS, 0); // 12 o'clock
        assert_eq!(hour_at(c(), p, false), Some(12));
    }

    #[test]
    fn hour_at_reads_inner_ring_in_24_hour_mode() {
        let p = numeral_pos(c(), INNER_RADIUS, 3);
        assert_eq!(hour_at(c(), p, false), Some(15));
        let p = numeral_pos(c(), INNER_RADIUS, 0);
        assert_eq!(hour_at(c(), p, false), Some(0));
    }

    #[test]
    fn hour_at_ignores_the_inner_ring_in_12_hour_mode() {
        let p = numeral_pos(c(), INNER_RADIUS, 3);
        // Still maps via the outer ring's index in 12-hour mode.
        assert_eq!(hour_at(c(), p, true), Some(3));
    }

    #[test]
    fn hour_at_rejects_the_dead_zone_near_the_centre() {
        assert_eq!(hour_at(c(), c(), false), None);
        assert_eq!(hour_at(c(), pos2(121.0, 121.0), false), None);
    }

    #[test]
    fn minute_at_maps_and_snaps() {
        let p = numeral_pos(c(), OUTER_RADIUS, 3); // 15 minutes
        assert_eq!(minute_at(c(), p, 1), Some(15));
        assert_eq!(minute_at(c(), p, 15), Some(15));
        let p = numeral_pos(c(), OUTER_RADIUS, 0);
        assert_eq!(minute_at(c(), p, 1), Some(0));
    }

    // --- builders ---------------------------------------------------------

    #[test]
    fn defaults_match_mudblazor() {
        let p: MudTimePicker = MudTimePicker::new();
        assert!(p.label.is_none());
        assert!(p.placeholder.is_none());
        assert!(p.helper_text.is_none());
        assert!(!p.helper_text_on_focus);
        assert!(matches!(p.variant, Variant::Text));
        assert!(matches!(p.margin, Margin::None));
        assert!(matches!(p.color, Color::Primary));
        assert!(matches!(p.size, Size::Medium));
        assert!(!p.clearable && !p.readonly && !p.disabled && !p.editable);
        assert_eq!(p.adornment_icon, filled::ACCESS_TIME);
        assert!(p.adornment_color.is_none());
        assert!(matches!(p.picker_variant, PickerVariant::Dialog));
        assert!(p.elevation.is_none());
        assert!(!p.rounded && !p.square);
        assert!(p.modal);
        assert!(matches!(p.overflow_behavior, OverflowBehavior::Flip));
        assert!(p.show_toolbar);
        assert!(!p.am_pm);
        assert!(!p.auto_close);
        assert_eq!(p.closing_delay, DEFAULT_CLOSING_DELAY_MS);
        assert_eq!(p.minute_selection_step, 1);
        assert!(matches!(p.open_to, OpenTo::Hours));
        assert_eq!(p.time_edit_mode, TimeEditMode::Normal);
        assert!(p.time_format.is_none());
        assert!(p.picker_actions.is_none());
        assert!(p.on_picker_opened.is_none());
        assert!(p.on_picker_closed.is_none());
        assert!(p.on_click.is_none());
    }

    #[test]
    fn builders_set_every_field() {
        let p: MudTimePicker = MudTimePicker::new()
            .label("L")
            .placeholder("P")
            .helper_text("H")
            .helper_text_on_focus(true)
            .variant(Variant::Outlined)
            .margin(Margin::Dense)
            .color(Color::Secondary)
            .size(Size::Large)
            .icon_size(Size::Small)
            .clearable(true)
            .readonly(true)
            .disabled(true)
            .editable(true)
            .immediate_text(true)
            .adornment_icon("X")
            .adornment_color(Color::Warning)
            .picker_variant(PickerVariant::Inline)
            .elevation(4)
            .rounded(true)
            .square(true)
            .modal(false)
            .overflow_behavior(OverflowBehavior::Reposition)
            .show_toolbar(false)
            .am_pm(true)
            .auto_close(true)
            .closing_delay(500)
            .minute_selection_step(15)
            .open_to(OpenTo::Minutes)
            .time_edit_mode(TimeEditMode::OnlyHours)
            .time_format("HH:mm:ss")
            .picker_actions(|_ui| None)
            .on_picker_opened(|| {})
            .on_picker_closed(|| {})
            .on_click(|| {})
            .classes(["cls"]);

        assert_eq!(p.label.as_deref(), Some("L"));
        assert_eq!(p.placeholder.as_deref(), Some("P"));
        assert_eq!(p.helper_text.as_deref(), Some("H"));
        assert!(p.helper_text_on_focus);
        assert!(matches!(p.variant, Variant::Outlined));
        assert!(matches!(p.margin, Margin::Dense));
        assert!(matches!(p.color, Color::Secondary));
        assert!(matches!(p.size, Size::Large));
        assert!(matches!(p.icon_size, Size::Small));
        assert!(p.clearable && p.readonly && p.disabled && p.editable);
        assert!(p.immediate_text);
        assert_eq!(p.adornment_icon, "X");
        assert_eq!(p.adornment_color, Some(Color::Warning));
        assert!(matches!(p.picker_variant, PickerVariant::Inline));
        assert_eq!(p.elevation, Some(4));
        assert!(p.rounded && p.square);
        assert!(!p.modal);
        assert!(matches!(p.overflow_behavior, OverflowBehavior::Reposition));
        assert!(!p.show_toolbar);
        assert!(p.am_pm && p.auto_close);
        assert_eq!(p.closing_delay, 500);
        assert_eq!(p.minute_selection_step, 15);
        assert!(matches!(p.open_to, OpenTo::Minutes));
        assert_eq!(p.time_edit_mode, TimeEditMode::OnlyHours);
        assert_eq!(p.time_format.as_deref(), Some("HH:mm:ss"));
        assert!(p.picker_actions.is_some());
        assert!(p.on_picker_opened.is_some());
        assert!(p.on_picker_closed.is_some());
        assert!(p.on_click.is_some());
        assert!(p.base.has_class("cls"));
    }

    #[test]
    fn minute_selection_step_clamps_to_at_least_one() {
        let p = MudTimePicker::new().minute_selection_step(0);
        assert_eq!(p.minute_selection_step, 1);
    }

    #[test]
    fn effective_format_matches_mudblazor_defaults() {
        let p = MudTimePicker::new();
        assert_eq!(p.effective_format(), "HH:mm");
        let p = MudTimePicker::new().am_pm(true);
        assert_eq!(p.effective_format(), "hh:mm tt");
        let p = MudTimePicker::new().time_format("HH:mm:ss");
        assert_eq!(p.effective_format(), "HH:mm:ss");
        let p = MudTimePicker::new().time_format("");
        assert_eq!(p.effective_format(), "HH:mm");
    }
}