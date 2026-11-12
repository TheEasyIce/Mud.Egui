//! `MudDateRangePicker` — a dual text field with a popover calendar that
//! selects a start and an end date.
//!
//! Port of `MudDateRangePicker.razor.cs` (MudBlazor 8.13.0). Pass 1.
//!
//! | MudBlazor parameter | Rust field / method |
//! |---------------------|---------------------|
//! | `AllowDisabledDatesInCount` | [`MudDateRangePicker::allow_disabled_dates_in_count`] (default `true`) |
//! | `AllowDisabledDatesInRange` | [`MudDateRangePicker::allow_disabled_dates_in_range`] (default `false`) |
//! | `AutoClose` | [`MudDateRangePicker::auto_close`] (default `true`) |
//! | `DateRange` | [`DateRangePickerState::range`] (caller-owned) |
//! | `DateRangeChanged` | [`DateRangePickerResponse::action`] (`RangeSelected`) |
//! | `MaxDays` | [`MudDateRangePicker::max_days`] |
//! | `MinDays` | [`MudDateRangePicker::min_days`] |
//! | `PlaceholderEnd` | [`MudDateRangePicker::placeholder_end`] |
//! | `PlaceholderStart` | [`MudDateRangePicker::placeholder_start`] |
//! | `SeparatorIcon` | [`MudDateRangePicker::separator_icon`] (default `ARROW_RIGHT_ALT`) |
//! | *base* `AdornmentIcon` / `AdornmentColor` | stored — `MudRangeInput` has no adornment slot |
//! | *base* `Placeholder` | stored — overridden by the two placeholders |
//! | *base* `TitleDateFormat` / `ShowToolbar` | stored — the toolbar is not painted |
//! | *base* `Clearable` … `Variant` | the shared picker surface |
//! | `Style` / `Tag` / `UserAttributes` | not ported: no CSS strings, no HTML tag |
//! | `Mask` / typed parsing / `Inline` / `Static` | deferred to pass 2 |
//!
//! # Composition, not reimplementation
//!
//! | Primitive | Role |
//! |-----------|------|
//! | [`MudTextField`] ×2 | The start and end triggers. |
//! | [`MudPopover`] | The calendar dropdown. |
//! | [`MudIcon`] | The separator glyph and the calendar chevrons. |
//! | [`MudText`] | The label, the helper/error row, and every numeral. |
//!
//! `DatePickerView`, `SimpleDate`, `is_selectable`, `rotate_day_names` and the
//! calendar geometry constants are **reused from `MudDatePicker`** — the range
//! picker only adds the range decoration and the two-click state machine.
//!
//! # Two-click selection
//!
//! | Click | Effect |
//! |-------|--------|
//! | First | Stores the day in [`DateRangePickerState::pending_start`]; the calendar shows it as a single-day range and starts a hover preview |
//! | Second | Orders the pair so `start <= end`, checks `MinDays` / `MaxDays` / disabled days, commits, and closes when `auto_close` or no `picker_actions` |
//! | Third | Starts a fresh selection (the first click rule) |
//!
//! MudBlazor's rule — "if `_secondDate` is already set, the clicked date
//! becomes `_firstDate` and `_secondDate` is cleared" — falls out of
//! `pending_start` being cleared on every commit.
//!
//! # Validation
//!
//! | Parameter | Effect |
//! |-----------|--------|
//! | `min_days` / `max_days` | Inclusive day-count bounds, checked on the second click |
//! | `allow_disabled_dates_in_count` | When `false`, disabled days do not count toward those bounds |
//! | `allow_disabled_dates_in_range` | When `false` (default), a range containing a disabled day is rejected |
//!
//! # State
//!
//! [`DateRangePickerState::text_start`] / `text_end` are **not** re-synced from
//! `range` every frame — `commit_range()` / `clear()` / `with_range()` keep
//! them in sync, so typed edits in `editable(true)` mode survive.

use egui::{
    pos2, vec2, Color32, CornerRadius, CursorIcon, Layout, Pos2, Rect, Response, Sense,
    Stroke, Ui, UiBuilder, Vec2,
};
use mud_egui_core::base::component_base::ComponentBase;
use mud_egui_core::enums::{
    AnchorOrigin, BackdropMode, Color, Margin, OpenTo, OverflowBehavior, PickerVariant,
    Size, StyleFlavor, TransformOrigin, Typo, Variant,
};
use mud_egui_icons_material::filled;
use mud_egui_theming::models::palettes::palette::Palette;
use mud_egui_theming::theme::MudTheme;

use crate::data_display::icons::mud_icon::MudIcon;
use crate::feedback::popover::MudPopover;
use crate::inputs::pickers::mud_date_picker::{
    is_selectable, rotate_day_names, DatePickerView, SimpleDate, CALENDAR_PADDING,
    CALENDAR_WIDTH, DAY_CELL, DEFAULT_DAY_NAMES, HEADER_HEIGHT, MONTH_NAMES,
};
use crate::inputs::text_field::MudTextField;
use crate::typography::mud_text::MudText;

use super::date_range::{inclusive_days, DateRange};
use super::date_range_picker_action::DateRangePickerAction;
use super::date_range_picker_state::DateRangePickerState;
use super::range_position::RangePosition;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Side of the separator glyph between the two trigger fields.
pub const SEPARATOR_SIZE: f32 = 20.0;

/// Gap on each side of the separator glyph.
pub const SEPARATOR_GAP: f32 = 4.0;

/// Corner radius of a range endpoint pill.
const CELL_RADIUS: u8 = 18;

/// Alpha of the band between two committed endpoints.
const BAND_ALPHA: u8 = 60;

/// Alpha of the band shown while hovering the second date.
const PREVIEW_ALPHA: u8 = 32;

/// Number of years displayed per decade in the year grid.
const YEARS_PER_DECADE: i32 = 10;

// ---------------------------------------------------------------------------
// Response
// ---------------------------------------------------------------------------

/// The result of one [`MudDateRangePicker::show`] frame.
#[derive(Debug)]
pub struct DateRangePickerResponse {
    /// The start field's own `Response`.
    pub trigger_start: Response,
    /// The end field's own `Response`.
    pub trigger_end: Response,
    /// Committed range after this frame.
    pub range: DateRange,
    /// What the picker did this frame.
    pub action: DateRangePickerAction,
}

// ---------------------------------------------------------------------------
// MudDateRangePicker
// ---------------------------------------------------------------------------

/// A dual text field with a popover calendar that selects a date range.
///
/// Mirrors `MudDateRangePicker` in MudBlazor 8.13.0. The Blazor two-way
/// binding `DateRange` / `DateRangeChanged` becomes a caller-owned
/// [`DateRangePickerState::range`] plus a
/// [`DateRangePickerResponse::action`] report — the same split
/// `MudDatePicker`, `MudTimePicker` and `MudRating` use.
#[must_use = "MudDateRangePicker does nothing until you call `.show(ui, state)`"]
pub struct MudDateRangePicker<'a> {
    // ── Trigger fields ─────────────────────────────────────────────────────
    /// Caption above the two fields. Mirrors `Label`.
    pub label: Option<String>,
    /// Base placeholder. Stored — overridden by the two specific ones.
    pub placeholder: Option<String>,
    /// Placeholder for the start field. Mirrors `PlaceholderStart`.
    pub placeholder_start: Option<String>,
    /// Placeholder for the end field. Mirrors `PlaceholderEnd`.
    pub placeholder_end: Option<String>,
    /// Caption below the pair. Mirrors `HelperText`.
    pub helper_text: Option<String>,
    /// Show `helper_text` only while focused or non-empty. Mirrors `HelperTextOnFocus`.
    pub helper_text_on_focus: bool,
    /// Error accent. Mirrors `Error`.
    pub error: bool,
    /// Error text shown instead of `helper_text`. Mirrors `ErrorText`.
    pub error_text: Option<String>,
    /// Field variant (Text / Outlined / Filled). Mirrors `Variant`.
    pub variant: Variant,
    /// Field density. Mirrors `Margin`.
    pub margin: Margin,
    /// Accent colour. Mirrors `Color`.
    pub color: Color,
    /// Field height / font size. Mirrors `Size`.
    pub size: Size,
    /// Calendar icon size. Mirrors `IconSize` — stored, not rendered.
    pub icon_size: Size,
    /// Shows a clear button on both fields. Mirrors `Clearable`.
    pub clearable: bool,
    /// Prevents editing without dimming. Mirrors `ReadOnly`.
    pub readonly: bool,
    /// Prevents all interaction and dims. Mirrors `Disabled`.
    pub disabled: bool,
    /// Allows typing into either field. Mirrors `Editable`.
    pub editable: bool,
    /// Stored for parity. Mirrors `ImmediateText`.
    pub immediate_text: bool,

    // ── Adornment (inert on this component) ────────────────────────────────
    /// Stored, not rendered: `MudRangeInput` has no adornment slot.
    pub adornment_icon: String,
    /// Stored, not rendered.
    pub adornment_color: Option<Color>,

    // ── Separator ──────────────────────────────────────────────────────────
    /// Glyph between the two fields. Mirrors `SeparatorIcon`.
    pub separator_icon: String,

    // ── Picker variant / display ───────────────────────────────────────────
    /// `Inline` / `Dialog` / `Static`. Only `Dialog` renders.
    pub picker_variant: PickerVariant,
    /// Popover shadow depth. Mirrors `Elevation`.
    pub elevation: Option<u8>,
    /// Rounded popover corners. Mirrors `Rounded` — stored; the popover's
    /// paper is already rounded by default, so the only corner control is
    /// `square(true)`.
    pub rounded: bool,
    /// Square popover corners. Mirrors `Square`.
    pub square: bool,
    /// Transparent backdrop blocks the parent UI. Mirrors `Modal`.
    pub modal: bool,
    /// Flip / reposition / shrink. Mirrors `OverflowBehavior`.
    pub overflow_behavior: OverflowBehavior,
    /// Stored — the toolbar is not painted. Mirrors `ShowToolbar`.
    pub show_toolbar: bool,
    /// Initial zoom level. Mirrors `OpenTo`.
    pub open_to: OpenTo,
    /// Toolbar title template. Stored — the toolbar is not painted.
    pub title_date_format: Option<String>,
    /// Popover size-hint override in points. Stored. Mirrors `RelativeWidth`.
    pub max_height: Option<f32>,
    /// Close after the range is committed. Mirrors `AutoClose` (default `true`).
    pub auto_close: bool,

    // ── Date-specific ──────────────────────────────────────────────────────
    /// Lower bound for selectable days. Mirrors `MinDate`.
    pub min_date: Option<SimpleDate>,
    /// Upper bound for selectable days. Mirrors `MaxDate`.
    pub max_date: Option<SimpleDate>,
    /// 0 = Monday … 6 = Sunday. Mirrors `FirstDayOfWeek`.
    pub first_day_of_week: u8,
    /// Custom display formatter. Mirrors `DateFormat`.
    pub date_format: Option<Box<dyn Fn(&SimpleDate) -> String + 'a>>,
    /// Extra disable predicate for individual days. Mirrors `IsDateDisabledFunc`.
    pub is_date_disabled: Option<Box<dyn Fn(&SimpleDate) -> bool + 'a>>,

    // ── Range validation ───────────────────────────────────────────────────
    /// Minimum inclusive day count. Mirrors `MinDays`.
    pub min_days: Option<u32>,
    /// Maximum inclusive day count. Mirrors `MaxDays`.
    pub max_days: Option<u32>,
    /// Count disabled days toward `min_days` / `max_days`. Mirrors
    /// `AllowDisabledDatesInCount` (default `true`).
    pub allow_disabled_dates_in_count: bool,
    /// Allow a committed range to contain disabled days. Mirrors
    /// `AllowDisabledDatesInRange` (default `false`).
    pub allow_disabled_dates_in_range: bool,

    // ── Customisation ──────────────────────────────────────────────────────
    /// Custom footer row inside the popover. Returns `Some(action)` to
    /// request a state change — applied after the popover closure returns.
    pub picker_actions:
        Option<Box<dyn FnMut(&mut Ui) -> Option<DateRangePickerAction> + 'a>>,
    /// Fires on open. Mirrors `PickerOpened`.
    pub on_picker_opened: Option<Box<dyn FnMut() + 'a>>,
    /// Fires on close. Mirrors `PickerClosed`.
    pub on_picker_closed: Option<Box<dyn FnMut() + 'a>>,
    /// Fires on trigger click. Mirrors `OnClick`.
    pub on_click: Option<Box<dyn FnMut() + 'a>>,

    // ── Utilities ──────────────────────────────────────────────────────────
    /// Shared component state (utility classes). Stored for parity.
    pub base: ComponentBase,
    /// Bootstrap vs Tailwind resolver. Not a MudBlazor parameter.
    pub flavor: StyleFlavor,
}

impl<'a> Default for MudDateRangePicker<'a> {
    fn default() -> Self {
        Self {
            label: None,
            placeholder: None,
            placeholder_start: None,
            placeholder_end: None,
            helper_text: None,
            helper_text_on_focus: false,
            error: false,
            error_text: None,
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
            adornment_icon: filled::CALENDAR_MONTH.to_owned(),
            adornment_color: None,
            separator_icon: filled::ARROW_RIGHT_ALT.to_owned(),
            picker_variant: PickerVariant::Dialog,
            elevation: None,
            rounded: false,
            square: false,
            modal: true,
            overflow_behavior: OverflowBehavior::Flip,
            show_toolbar: true,
            open_to: OpenTo::Day,
            title_date_format: None,
            max_height: None,
            auto_close: true,
            min_date: None,
            max_date: None,
            first_day_of_week: 0,
            date_format: None,
            is_date_disabled: None,
            min_days: None,
            max_days: None,
            allow_disabled_dates_in_count: true,
            allow_disabled_dates_in_range: false,
            picker_actions: None,
            on_picker_opened: None,
            on_picker_closed: None,
            on_click: None,
            base: ComponentBase::new(),
            flavor: StyleFlavor::default(),
        }
    }
}

impl<'a> MudDateRangePicker<'a> {
    /// Creates a picker with MudBlazor's default settings.
    pub fn new() -> Self {
        Self::default()
    }

    // ── Builders ───────────────────────────────────────────────────────────

    #[inline] pub fn label(mut self, s: impl Into<String>) -> Self { self.label = Some(s.into()); self }
    #[inline] pub fn placeholder(mut self, s: impl Into<String>) -> Self { self.placeholder = Some(s.into()); self }
    #[inline] pub fn placeholder_start(mut self, s: impl Into<String>) -> Self { self.placeholder_start = Some(s.into()); self }
    #[inline] pub fn placeholder_end(mut self, s: impl Into<String>) -> Self { self.placeholder_end = Some(s.into()); self }
    #[inline] pub fn helper_text(mut self, s: impl Into<String>) -> Self { self.helper_text = Some(s.into()); self }
    #[inline] pub fn helper_text_on_focus(mut self, b: bool) -> Self { self.helper_text_on_focus = b; self }
    #[inline] pub fn error(mut self, b: bool) -> Self { self.error = b; self }
    #[inline] pub fn error_text(mut self, s: impl Into<String>) -> Self { self.error_text = Some(s.into()); self }
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
    #[inline] pub fn separator_icon(mut self, s: impl Into<String>) -> Self { self.separator_icon = s.into(); self }
    #[inline] pub fn picker_variant(mut self, v: PickerVariant) -> Self { self.picker_variant = v; self }
    #[inline] pub fn elevation(mut self, e: u8) -> Self { self.elevation = Some(e); self }
    #[inline] pub fn rounded(mut self, b: bool) -> Self { self.rounded = b; self }
    #[inline] pub fn square(mut self, b: bool) -> Self { self.square = b; self }
    #[inline] pub fn modal(mut self, b: bool) -> Self { self.modal = b; self }
    #[inline] pub fn overflow_behavior(mut self, b: OverflowBehavior) -> Self { self.overflow_behavior = b; self }
    #[inline] pub fn show_toolbar(mut self, b: bool) -> Self { self.show_toolbar = b; self }
    #[inline] pub fn open_to(mut self, o: OpenTo) -> Self { self.open_to = o; self }
    #[inline] pub fn title_date_format(mut self, s: impl Into<String>) -> Self { self.title_date_format = Some(s.into()); self }
    #[inline] pub fn max_height(mut self, px: f32) -> Self { self.max_height = Some(px); self }
    #[inline] pub fn auto_close(mut self, b: bool) -> Self { self.auto_close = b; self }
    #[inline] pub fn min_date(mut self, d: SimpleDate) -> Self { self.min_date = Some(d); self }
    #[inline] pub fn max_date(mut self, d: SimpleDate) -> Self { self.max_date = Some(d); self }
    #[inline] pub fn first_day_of_week(mut self, d: u8) -> Self { self.first_day_of_week = d.min(6); self }
    #[inline] pub fn date_format(mut self, f: impl Fn(&SimpleDate) -> String + 'a) -> Self { self.date_format = Some(Box::new(f)); self }
    #[inline] pub fn is_date_disabled(mut self, f: impl Fn(&SimpleDate) -> bool + 'a) -> Self { self.is_date_disabled = Some(Box::new(f)); self }
    /// Sets `MinDays`, clamped to at least `1` (a range is never zero days).
    #[inline] pub fn min_days(mut self, n: u32) -> Self { self.min_days = Some(n.max(1)); self }
    /// Sets `MaxDays`, clamped to at least `1`.
    #[inline] pub fn max_days(mut self, n: u32) -> Self { self.max_days = Some(n.max(1)); self }
    #[inline] pub fn allow_disabled_dates_in_count(mut self, b: bool) -> Self { self.allow_disabled_dates_in_count = b; self }
    #[inline] pub fn allow_disabled_dates_in_range(mut self, b: bool) -> Self { self.allow_disabled_dates_in_range = b; self }
    #[inline] pub fn picker_actions(mut self, f: impl FnMut(&mut Ui) -> Option<DateRangePickerAction> + 'a) -> Self { self.picker_actions = Some(Box::new(f)); self }
    #[inline] pub fn on_picker_opened(mut self, f: impl FnMut() + 'a) -> Self { self.on_picker_opened = Some(Box::new(f)); self }
    #[inline] pub fn on_picker_closed(mut self, f: impl FnMut() + 'a) -> Self { self.on_picker_closed = Some(Box::new(f)); self }
    #[inline] pub fn on_click(mut self, f: impl FnMut() + 'a) -> Self { self.on_click = Some(Box::new(f)); self }
    #[inline] pub fn classes<S: AsRef<str>>(mut self, classes: impl IntoIterator<Item = S>) -> Self { self.base = self.base.classes(classes); self }
    #[inline] pub fn flavor(mut self, f: StyleFlavor) -> Self { self.flavor = f; self }

    // ── Rendering ──────────────────────────────────────────────────────────

    /// Draws the two trigger fields and, when open, the range calendar.
    pub fn show(mut self, ui: &mut Ui, state: &mut DateRangePickerState) -> DateRangePickerResponse {
        // 0. Snapshot everything the trigger needs before it takes
        //    `&mut state.text_start` / `&mut state.text_end`.
        let was_open = state.popover.open;
        let committed = state.range;
        let pending_start = state.pending_start;
        let text_empty = state.text_start.is_empty() && state.text_end.is_empty();
        let show_helper = !self.error
            && self.helper_text.is_some()
            && (!self.helper_text_on_focus || !text_empty || was_open);
        let has_actions = self.picker_actions.is_some();

        // 1. Label above the pair (MudRangeInput has one label, not two).
        let theme = MudTheme::fetch(ui.ctx()).unwrap_or_default();
        let palette = theme.current_palette();
        let p = resolve_palette(self.color, palette, ui);
        if let Some(label) = self.label.clone() {
            ui.add(
                MudText::new(label)
                    .typo(Typo::Body2)
                    .custom_color(p.text_secondary),
            );
            ui.add_space(2.0);
        }

        // 2. The two fields and the separator.
        let field_w = ((ui.available_width() - SEPARATOR_SIZE - 2.0 * SEPARATOR_GAP).max(0.0))
            * 0.5;
        let start_id = state.popover.id.with("start_field");
        let end_id = state.popover.id.with("end_field");

        let variant = self.variant;
        let margin = self.margin;
        let size = self.size;
        let color = self.color;
        let disabled = self.disabled;
        let readonly = self.readonly || !self.editable;
        let clearable = self.clearable;
        let placeholder_start = self.placeholder_start.clone();
        let placeholder_end = self.placeholder_end.clone();
        let clear_icon = filled::CLEAR.to_owned();
        let separator_icon = self.separator_icon.clone();
        let separator_color = p.text_secondary;

        let pair = ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = SEPARATOR_GAP;

            let start = ui
                .allocate_ui(vec2(field_w, 0.0), |ui| {
                    let mut f = MudTextField::new(&mut state.text_start)
                        .id(start_id)
                        .disabled(disabled)
                        .readonly(readonly)
                        .variant(variant)
                        .margin(margin)
                        .color(color)
                        .size(size)
                        .clear_icon(clear_icon.clone())
                        .clearable(clearable);
                    if let Some(s) = placeholder_start.clone() { f = f.placeholder(s); }
                    f.show(ui)
                })
                .inner;

            let (sep_rect, _) =
                ui.allocate_exact_size(Vec2::splat(SEPARATOR_SIZE), Sense::hover());
            ui.put(
                sep_rect,
                MudIcon::new(separator_icon)
                    .size_px(SEPARATOR_SIZE)
                    .custom_color(separator_color),
            );

            let end = ui
                .allocate_ui(vec2(field_w, 0.0), |ui| {
                    let mut f = MudTextField::new(&mut state.text_end)
                        .id(end_id)
                        .disabled(disabled)
                        .readonly(readonly)
                        .variant(variant)
                        .margin(margin)
                        .color(color)
                        .size(size)
                        .clear_icon(clear_icon)
                        .clearable(clearable);
                    if let Some(s) = placeholder_end.clone() { f = f.placeholder(s); }
                    f.show(ui)
                })
                .inner;

            (start, end)
        })
            .inner;

        let start_resp = pair.0;
        let end_resp = pair.1;
        let trigger_clicked = start_resp.response.clicked() || end_resp.response.clicked();
        let trigger_changed = start_resp.changed || end_resp.changed;

        // 3. Helper / error row below the pair.
        if self.error {
            if let Some(msg) = self.error_text.clone() {
                ui.add(MudText::new(msg).typo(Typo::Caption).custom_color(p.error));
            } else {
                ui.add_space(2.0);
            }
        } else if show_helper {
            if let Some(msg) = self.helper_text.clone() {
                ui.add(
                    MudText::new(msg)
                        .typo(Typo::Caption)
                        .custom_color(p.text_secondary),
                );
            }
        }

        // 4. Trigger interaction. Clearing runs first so a clear click does
        //    not also open the popover. Clearing either field clears both
        //    texts and the range, mirroring `ClearAsync`.
        let mut action = DateRangePickerAction::None;
        let clear_clicked = self.clearable
            && trigger_changed
            && state.text_start.is_empty()
            && state.text_end.is_empty()
            && !committed.is_empty();

        if !clear_clicked
            && trigger_clicked
            && !self.disabled
            && self.picker_variant.is_dialog()
        {
            state.toggle();
            if let Some(cb) = self.on_click.as_mut() { cb(); }
        }

        let now_open = state.popover.open;
        if now_open != was_open {
            if now_open {
                if let Some(cb) = self.on_picker_opened.as_mut() { cb(); }
                action = DateRangePickerAction::Opened;
            } else {
                if let Some(cb) = self.on_picker_closed.as_mut() { cb(); }
                action = DateRangePickerAction::Closed;
            }
        }

        if clear_clicked {
            state.clear();
            action = DateRangePickerAction::Cleared;
        }

        // 5. Popover + calendar.
        if state.popover.open && self.picker_variant.is_dialog() {
            // `MudPopover` exposes `square(bool)` only; the paper is rounded
            // by default, so `self.rounded` is stored but has no separate
            // effect.
            let mut popover = MudPopover::new()
                .anchor_origin(AnchorOrigin::Bottom)
                .transform_origin(TransformOrigin::TopLeft)
                .paper(true)
                .elevation(self.elevation.unwrap_or(8))
                .overflow_behavior(self.overflow_behavior)
                .size_hint(vec2(CALENDAR_WIDTH, 360.0))
                .match_anchor_width(false)
                .square(self.square);
            if self.modal {
                popover = popover.backdrop(BackdropMode::Transparent);
            }

            let child_rect = start_resp.response.rect.union(end_resp.response.rect);

            // Deferred interactions: collected inside the closure, applied
            // after `popover.show` releases `&mut state.popover`.
            let mut clicked_date: Option<SimpleDate> = None;
            let mut view_clicked: Option<DatePickerView> = None;
            let mut deferred_action: Option<DateRangePickerAction> = None;

            let mut picker_actions = self.picker_actions.take();

            let display = state.display_range();
            let view = state.view;
            let first_day_of_week = self.first_day_of_week;
            let min_date = self.min_date;
            let max_date = self.max_date;
            let user_disabled = self.is_date_disabled.as_deref();
            let min_days = self.min_days;
            let max_days = self.max_days;
            let allow_in_count = self.allow_disabled_dates_in_count;

            let _ = popover.show(ui, &mut state.popover, child_rect, |pop_ui| {
                render_calendar_body(
                    pop_ui,
                    &mut state.view_month,
                    &mut state.view_year_decade,
                    view,
                    display,
                    pending_start,
                    &p,
                    first_day_of_week,
                    min_date.as_ref(),
                    max_date.as_ref(),
                    user_disabled,
                    min_days,
                    max_days,
                    allow_in_count,
                    &mut picker_actions,
                    &mut clicked_date,
                    &mut view_clicked,
                    &mut deferred_action,
                );
            });

            self.picker_actions = picker_actions;

            let formatter = self.date_format.as_deref();
            if let Some(v) = view_clicked {
                state.view = v;
            }

            // 6. Apply the calendar interaction.
            let mut committed_range: Option<DateRange> = None;

            if let Some(date) = clicked_date {
                match pending_start {
                    None => {
                        // First click: hold the start, drop any half-built end.
                        state.pending_start = Some(date);
                        state.view = DatePickerView::Day;
                    }
                    Some(first) => {
                        let span_ok = second_click_valid(
                            first,
                            date,
                            min_days,
                            max_days,
                            allow_in_count,
                            user_disabled,
                        );
                        if span_ok {
                            let ordered = DateRange::new(Some(first), Some(date)).normalized();
                            let contains_disabled = match user_disabled {
                                Some(pred) => range_contains_disabled(&ordered, &|d| pred(d)),
                                None => false,
                            };
                            if self.allow_disabled_dates_in_range || !contains_disabled {
                                committed_range = Some(ordered);
                            }
                        }
                    }
                }
            }

            if let Some(range) = committed_range {
                let (t_start, t_end) = range.formatted(formatter);
                let close_now = self.auto_close || !has_actions;
                state.range = range;
                state.text_start = t_start;
                state.text_end = t_end;
                state.pending_start = None;
                if close_now {
                    state.popover.open = false;
                    if let Some(cb) = self.on_picker_closed.as_mut() { cb(); }
                }
                action = DateRangePickerAction::RangeSelected(range);
            } else if let Some(deferred) = deferred_action {
                match deferred {
                    DateRangePickerAction::RangeSelected(range) => {
                        let (t_start, t_end) = range.formatted(formatter);
                        state.commit_range(range, t_start, t_end);
                        action = DateRangePickerAction::RangeSelected(range);
                        if let Some(cb) = self.on_picker_closed.as_mut() { cb(); }
                    }
                    DateRangePickerAction::Cleared => {
                        state.clear();
                        action = DateRangePickerAction::Cleared;
                        if let Some(cb) = self.on_picker_closed.as_mut() { cb(); }
                    }
                    other => action = other,
                }
            }
        }

        DateRangePickerResponse {
            trigger_start: start_resp.response,
            trigger_end: end_resp.response,
            range: state.range,
            action,
        }
    }
}

// ---------------------------------------------------------------------------
// Pure helpers
// ---------------------------------------------------------------------------

/// `true` when `second` is an acceptable endpoint for a range starting at
/// `first`, given `MinDays` / `MaxDays` and the disabled-day counting mode.
///
/// The count is inclusive of both ends, matching MudBlazor's
/// "Inclusive of the selected date" remarks.
pub fn second_click_valid(
    first: SimpleDate,
    second: SimpleDate,
    min_days: Option<u32>,
    max_days: Option<u32>,
    allow_disabled_in_count: bool,
    disabled: Option<&dyn Fn(&SimpleDate) -> bool>,
) -> bool {
    let count = span_count(first, second, allow_disabled_in_count, disabled);
    if let Some(max) = max_days {
        if count > max as i32 {
            return false;
        }
    }
    if let Some(min) = min_days {
        if count < min as i32 {
            return false;
        }
    }
    true
}

/// Inclusive day count between `first` and `second`, optionally skipping
/// disabled days (`AllowDisabledDatesInCount = false`).
pub fn span_count(
    first: SimpleDate,
    second: SimpleDate,
    allow_disabled: bool,
    disabled: Option<&dyn Fn(&SimpleDate) -> bool>,
) -> i32 {
    if allow_disabled {
        return inclusive_days(first, second);
    }
    let (lo, hi) = if first <= second { (first, second) } else { (second, first) };
    let mut n = 0;
    let mut d = lo;
    while d <= hi {
        let skip = disabled.map(|f| f(&d)).unwrap_or(false);
        if !skip {
            n += 1;
        }
        d = d.add_days(1);
    }
    n
}

/// `true` when any day in `range` is disabled by `disabled`.
pub fn range_contains_disabled(
    range: &DateRange,
    disabled: &dyn Fn(&SimpleDate) -> bool,
) -> bool {
    let (Some(start), Some(end)) = (range.start, range.end) else {
        return false;
    };
    let (lo, hi) = if start <= end { (start, end) } else { (end, start) };
    let mut d = lo;
    while d <= hi {
        if disabled(&d) {
            return true;
        }
        d = d.add_days(1);
    }
    false
}

/// Where `date` sits relative to `range`.
pub(crate) fn classify(date: &SimpleDate, range: &DateRange) -> RangePosition {
    match (range.start, range.end) {
        (Some(s), Some(e)) => {
            let (s, e) = if s <= e { (s, e) } else { (e, s) };
            if *date == s && *date == e {
                RangePosition::SingleDay
            } else if *date == s {
                RangePosition::Start
            } else if *date == e {
                RangePosition::End
            } else if *date > s && *date < e {
                RangePosition::Between
            } else {
                RangePosition::Outside
            }
        }
        (Some(s), None) if *date == s => RangePosition::SingleDay,
        _ => RangePosition::Outside,
    }
}

/// The hover preview range: `pending_start ..= hovered`, normalised, or
/// `None` when either end is missing or they are the same day.
pub(crate) fn preview_range(
    pending_start: Option<SimpleDate>,
    hovered: Option<SimpleDate>,
) -> Option<DateRange> {
    match (pending_start, hovered) {
        (Some(a), Some(b)) if a != b => {
            Some(DateRange::new(Some(a), Some(b)).normalized())
        }
        _ => None,
    }
}

/// The day under `pos` in a 7 × 6 grid, or `None` outside it.
pub fn day_at(pos: Pos2, grid: Rect, cells: &[SimpleDate]) -> Option<SimpleDate> {
    if !grid.contains(pos) {
        return None;
    }
    let col = ((pos.x - grid.left()) / DAY_CELL).floor();
    let row = ((pos.y - grid.top()) / DAY_CELL).floor();
    if col < 0.0 || row < 0.0 {
        return None;
    }
    let idx = row as usize * 7 + col as usize;
    cells.get(idx).copied()
}

/// Header title for the current view. Mirrors `MudDatePicker`'s private
/// helper; duplicated rather than widening that module's surface.
fn header_title(view_month: SimpleDate, view_year_decade: i32, view: DatePickerView) -> String {
    match view {
        DatePickerView::Day => {
            let name = MONTH_NAMES[(view_month.month - 1) as usize];
            format!("{name} {}", view_month.year)
        }
        DatePickerView::Month => format!("{}", view_month.year),
        DatePickerView::Year => {
            let end = view_year_decade + YEARS_PER_DECADE - 1;
            format!("{view_year_decade} – {end}")
        }
    }
}

/// Navigates the calendar one step in `direction`. Mirrors `MudDatePicker`'s
/// private helper.
fn navigate(
    view_month: &mut SimpleDate,
    view_year_decade: &mut i32,
    view: DatePickerView,
    direction: i32,
) {
    match view {
        DatePickerView::Day => *view_month = view_month.add_months(direction),
        DatePickerView::Month => *view_month = view_month.add_years(direction),
        DatePickerView::Year => *view_year_decade += direction * YEARS_PER_DECADE,
    }
}

/// Replaces the alpha channel, keeping the unmultiplied RGB.
#[inline]
fn with_alpha(c: Color32, alpha: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), alpha)
}

// ---------------------------------------------------------------------------
// Palette
// ---------------------------------------------------------------------------

struct PickerPalette {
    text_primary: Color32,
    text_secondary: Color32,
    text_disabled: Color32,
    accent: Color32,
    accent_text: Color32,
    hover_fill: Color32,
    divider: Color32,
    error: Color32,
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
        error: palette.error.into(),
    }
}

// ---------------------------------------------------------------------------
// Calendar rendering
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
fn render_calendar_body(
    ui: &mut Ui,
    view_month: &mut SimpleDate,
    view_year_decade: &mut i32,
    view: DatePickerView,
    display: DateRange,
    pending_start: Option<SimpleDate>,
    p: &PickerPalette,
    first_day_of_week: u8,
    min: Option<&SimpleDate>,
    max: Option<&SimpleDate>,
    user_disabled: Option<&dyn Fn(&SimpleDate) -> bool>,
    min_days: Option<u32>,
    max_days: Option<u32>,
    allow_in_count: bool,
    picker_actions: &mut Option<Box<dyn FnMut(&mut Ui) -> Option<DateRangePickerAction> + '_>>,
    clicked_date: &mut Option<SimpleDate>,
    view_clicked: &mut Option<DatePickerView>,
    deferred_action: &mut Option<DateRangePickerAction>,
) {
    render_header(ui, view_month, view_year_decade, view, p, view_clicked);

    match view {
        DatePickerView::Day => render_day_grid(
            ui,
            *view_month,
            display,
            pending_start,
            p,
            first_day_of_week,
            min,
            max,
            user_disabled,
            min_days,
            max_days,
            allow_in_count,
            clicked_date,
        ),
        DatePickerView::Month => render_month_grid(ui, view_month, display, p, view_clicked),
        DatePickerView::Year => render_year_grid(
            ui,
            view_month,
            *view_year_decade,
            display,
            p,
            view_clicked,
        ),
    }

    if let Some(actions) = picker_actions.as_mut() {
        ui.add_space(CALENDAR_PADDING);
        let cursor = ui.cursor().min;
        ui.painter().line_segment(
            [
                pos2(cursor.x, cursor.y),
                pos2(cursor.x + ui.available_width(), cursor.y),
            ],
            Stroke::new(1.0, p.divider),
        );
        if let Some(a) = actions(ui) {
            *deferred_action = Some(a);
        }
    }
}

fn render_header(
    ui: &mut Ui,
    view_month: &mut SimpleDate,
    view_year_decade: &mut i32,
    view: DatePickerView,
    p: &PickerPalette,
    view_clicked: &mut Option<DatePickerView>,
) {
    let width = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(vec2(width, HEADER_HEIGHT), Sense::hover());

    let btn_size = HEADER_HEIGHT - 8.0;
    let prev_rect = Rect::from_center_size(
        pos2(rect.left() + CALENDAR_PADDING + btn_size * 0.5, rect.center().y),
        Vec2::splat(btn_size),
    );
    let next_rect = Rect::from_center_size(
        pos2(rect.right() - CALENDAR_PADDING - btn_size * 0.5, rect.center().y),
        Vec2::splat(btn_size),
    );

    let prev_id = ui.scope_id().with("range_prev");
    let prev_resp = ui.interact(prev_rect, prev_id, Sense::click());
    if prev_resp.hovered() {
        ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
        ui.painter().rect_filled(prev_rect, CornerRadius::same(6), p.hover_fill);
    }
    if prev_resp.clicked() {
        navigate(view_month, view_year_decade, view, -1);
    }
    ui.put(
        prev_rect,
        MudIcon::new(filled::CHEVRON_LEFT.to_owned())
            .size_px(20.0)
            .custom_color(p.text_primary),
    );

    let next_id = ui.scope_id().with("range_next");
    let next_resp = ui.interact(next_rect, next_id, Sense::click());
    if next_resp.hovered() {
        ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
        ui.painter().rect_filled(next_rect, CornerRadius::same(6), p.hover_fill);
    }
    if next_resp.clicked() {
        navigate(view_month, view_year_decade, view, 1);
    }
    ui.put(
        next_rect,
        MudIcon::new(filled::CHEVRON_RIGHT.to_owned())
            .size_px(20.0)
            .custom_color(p.text_primary),
    );

    let title = header_title(*view_month, *view_year_decade, view);
    let title_rect = Rect::from_min_max(
        pos2(prev_rect.right() + 4.0, rect.top()),
        pos2(next_rect.left() - 4.0, rect.bottom()),
    );
    let title_id = ui.scope_id().with("range_title");
    let title_resp = ui.interact(title_rect, title_id, Sense::click());
    if title_resp.hovered() {
        ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
        ui.painter().rect_filled(title_rect, CornerRadius::same(6), p.hover_fill);
    }
    if title_resp.clicked() && view != DatePickerView::Year {
        *view_clicked = Some(view.zoom_out());
    }

    let mut child = ui.new_child(
        UiBuilder::new()
            .max_rect(title_rect)
            .layout(Layout::centered_and_justified(egui::Direction::LeftToRight)),
    );
    child.add(
        MudText::new(title)
            .typo(Typo::Subtitle2)
            .custom_color(p.text_primary),
    );
}

#[allow(clippy::too_many_arguments)]
fn render_day_grid(
    ui: &mut Ui,
    view_month: SimpleDate,
    display: DateRange,
    pending_start: Option<SimpleDate>,
    p: &PickerPalette,
    first_day_of_week: u8,
    min: Option<&SimpleDate>,
    max: Option<&SimpleDate>,
    user_disabled: Option<&dyn Fn(&SimpleDate) -> bool>,
    min_days: Option<u32>,
    max_days: Option<u32>,
    allow_in_count: bool,
    clicked_date: &mut Option<SimpleDate>,
) {
    // Day-name row.
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        for name in rotate_day_names(first_day_of_week) {
            let (rect, _) = ui.allocate_exact_size(vec2(DAY_CELL, 24.0), Sense::hover());
            let mut child = ui.new_child(
                UiBuilder::new()
                    .max_rect(rect)
                    .layout(Layout::centered_and_justified(egui::Direction::LeftToRight)),
            );
            child.add(
                MudText::new(name)
                    .typo(Typo::Caption)
                    .custom_color(p.text_secondary),
            );
        }
    });

    let cells = SimpleDate::calendar_grid(view_month.year, view_month.month, first_day_of_week)
        .unwrap_or_default();

    // One interaction region for the whole 7 × 6 grid, so the hover preview
    // can be computed *before* any cell is painted.
    let grid_size = vec2(DAY_CELL * 7.0, DAY_CELL * 6.0);
    let (grid_rect, grid_resp) = ui.allocate_exact_size(grid_size, Sense::click());

    let hovered = grid_resp.hover_pos().and_then(|pos| day_at(pos, grid_rect, &cells));
    let preview = preview_range(pending_start, hovered);

    let clicked = if grid_resp.clicked() {
        grid_resp
            .interact_pointer_pos()
            .and_then(|pos| day_at(pos, grid_rect, &cells))
    } else {
        None
    };

    for (idx, date) in cells.iter().enumerate() {
        let col = (idx % 7) as f32;
        let row = (idx / 7) as f32;
        let rect = Rect::from_min_size(
            grid_rect.min + vec2(col * DAY_CELL, row * DAY_CELL),
            Vec2::splat(DAY_CELL),
        );
        let other_month = date.month != view_month.month || date.year != view_month.year;

        let base_selectable = is_selectable(date, min, max, user_disabled);
        let selectable = if let Some(first) = pending_start {
            base_selectable
                && second_click_valid(first, *date, min_days, max_days, allow_in_count, user_disabled)
        } else {
            base_selectable
        };

        let position = classify(date, &display);
        let in_preview = preview
            .as_ref()
            .map(|r| r.contains(date) && !position.is_endpoint())
            .unwrap_or(false);

        paint_range_day(
            ui,
            rect,
            *date,
            other_month,
            selectable,
            position,
            in_preview,
            p,
        );
    }

    if let Some(date) = clicked {
        let base_ok = is_selectable(&date, min, max, user_disabled);
        let second_ok = match pending_start {
            None => true,
            Some(first) => {
                second_click_valid(first, date, min_days, max_days, allow_in_count, user_disabled)
            }
        };
        if base_ok && second_ok {
            *clicked_date = Some(date);
        }
    }

    if grid_resp.hovered() && hovered.is_some() {
        ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
    }
}

#[allow(clippy::too_many_arguments)]
fn paint_range_day(
    ui: &mut Ui,
    rect: Rect,
    date: SimpleDate,
    other_month: bool,
    selectable: bool,
    position: RangePosition,
    in_preview: bool,
    p: &PickerPalette,
) {
    // Geometry: a full-width band, and an inset pill for endpoints.
    let band = Rect::from_min_max(
        pos2(rect.left(), rect.top() + 2.0),
        pos2(rect.right(), rect.bottom() - 2.0),
    );
    let pill = rect.shrink(2.0);
    let r = CELL_RADIUS;

    // 1. Band / preview fills (never through `MudText`, so a scoped painter
    //    borrow is fine).
    if position.is_band() {
        ui.painter()
            .rect_filled(band, CornerRadius::ZERO, with_alpha(p.accent, BAND_ALPHA));
    }
    if in_preview {
        ui.painter().rect_filled(
            band,
            CornerRadius::ZERO,
            with_alpha(p.accent, PREVIEW_ALPHA),
        );
    }

    // 2. Endpoint pills, rounded on the outer edge only.
    match position {
        RangePosition::SingleDay => {
            ui.painter().rect_filled(pill, CornerRadius::same(r), p.accent);
        }
        RangePosition::Start => {
            ui.painter().rect_filled(
                pill,
                CornerRadius { nw: r, sw: r, ne: 0, se: 0 },
                p.accent,
            );
        }
        RangePosition::End => {
            ui.painter().rect_filled(
                pill,
                CornerRadius { nw: 0, sw: 0, ne: r, se: r },
                p.accent,
            );
        }
        RangePosition::Between | RangePosition::Outside => {}
    }

    // 3. Ink.
    let ink = if !selectable || other_month {
        p.text_disabled
    } else if position.is_endpoint() {
        p.accent_text
    } else {
        p.text_primary
    };

    // 4. Numeral through MudText (Rule 3).
    let mut child = ui.new_child(
        UiBuilder::new()
            .max_rect(pill)
            .layout(Layout::centered_and_justified(egui::Direction::LeftToRight)),
    );
    child.add(
        MudText::new(date.day.to_string())
            .typo(Typo::Body2)
            .custom_color(ink),
    );
}

fn render_month_grid(
    ui: &mut Ui,
    view_month: &mut SimpleDate,
    display: DateRange,
    p: &PickerPalette,
    view_clicked: &mut Option<DatePickerView>,
) {
    let year = view_month.year;
    for row in 0..4 {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            for col in 0..3 {
                let month_idx = row * 3 + col;
                let month_num = (month_idx + 1) as u8;
                let first = SimpleDate::first_of_month(year, month_num);
                let is_selected = display.start == Some(first) || display.end == Some(first);
                let (rect, resp) = ui.allocate_exact_size(
                    vec2(ui.available_width() / 3.0, DAY_CELL * 1.4),
                    Sense::click(),
                );
                let inner = rect.shrink(4.0);
                if is_selected {
                    ui.painter().rect_filled(inner, CornerRadius::same(6), p.accent);
                } else if resp.hovered() {
                    ui.painter().rect_filled(inner, CornerRadius::same(6), p.hover_fill);
                    ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
                }
                if resp.clicked() {
                    *view_month = first;
                    *view_clicked = Some(DatePickerView::Day);
                }
                let ink = if is_selected { p.accent_text } else { p.text_primary };
                let mut child = ui.new_child(
                    UiBuilder::new()
                        .max_rect(inner)
                        .layout(Layout::centered_and_justified(egui::Direction::LeftToRight)),
                );
                child.add(
                    MudText::new(MONTH_NAMES[month_idx])
                        .typo(Typo::Body2)
                        .custom_color(ink),
                );
            }
        });
    }
}

fn render_year_grid(
    ui: &mut Ui,
    view_month: &mut SimpleDate,
    view_year_decade: i32,
    display: DateRange,
    p: &PickerPalette,
    view_clicked: &mut Option<DatePickerView>,
) {
    for row in 0..2 {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            for col in 0..5 {
                let year = view_year_decade + row * 5 + col;
                let is_selected = display.start.map(|d| d.year == year).unwrap_or(false)
                    || display.end.map(|d| d.year == year).unwrap_or(false);
                let (rect, resp) = ui.allocate_exact_size(
                    vec2(ui.available_width() / 5.0, DAY_CELL * 1.4),
                    Sense::click(),
                );
                let inner = rect.shrink(4.0);
                if is_selected {
                    ui.painter().rect_filled(inner, CornerRadius::same(6), p.accent);
                } else if resp.hovered() {
                    ui.painter().rect_filled(inner, CornerRadius::same(6), p.hover_fill);
                    ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
                }
                if resp.clicked() {
                    *view_month = SimpleDate::first_of_month(year, view_month.month);
                    *view_clicked = Some(DatePickerView::Month);
                }
                let ink = if is_selected { p.accent_text } else { p.text_primary };
                let mut child = ui.new_child(
                    UiBuilder::new()
                        .max_rect(inner)
                        .layout(Layout::centered_and_justified(egui::Direction::LeftToRight)),
                );
                child.add(
                    MudText::new(year.to_string())
                        .typo(Typo::Body2)
                        .custom_color(ink),
                );
            }
        });
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u8, day: u8) -> SimpleDate {
        SimpleDate::new(y, m, day)
    }

    // --- second-click validation ------------------------------------------

    #[test]
    fn no_bounds_accepts_every_pair() {
        for (a, b) in [
            (d(2024, 1, 1), d(2024, 1, 1)),
            (d(2024, 1, 1), d(2024, 1, 2)),
            (d(2024, 1, 1), d(2025, 1, 1)),
            (d(2025, 1, 1), d(2024, 1, 1)),
        ] {
            assert!(second_click_valid(a, b, None, None, true, None));
        }
    }

    #[test]
    fn max_days_is_inclusive() {
        let first = d(2024, 1, 1);
        assert!(second_click_valid(first, d(2024, 1, 3), None, Some(3), true, None));
        assert!(!second_click_valid(first, d(2024, 1, 4), None, Some(3), true, None));
        assert!(second_click_valid(first, first, None, Some(1), true, None));
    }

    #[test]
    fn max_days_applies_in_either_direction() {
        let first = d(2024, 1, 10);
        assert!(second_click_valid(first, d(2024, 1, 8), None, Some(3), true, None));
        assert!(!second_click_valid(first, d(2024, 1, 6), None, Some(3), true, None));
    }

    #[test]
    fn min_days_is_inclusive() {
        let first = d(2024, 1, 1);
        assert!(!second_click_valid(first, first, Some(3), None, true, None));
        assert!(!second_click_valid(first, d(2024, 1, 2), Some(3), None, true, None));
        assert!(second_click_valid(first, d(2024, 1, 3), Some(3), None, true, None));
    }

    #[test]
    fn min_and_max_compose() {
        let first = d(2024, 1, 1);
        let ok = |b| second_click_valid(first, b, Some(2), Some(4), true, None);
        assert!(!ok(d(2024, 1, 1)));
        assert!(ok(d(2024, 1, 2)));
        assert!(ok(d(2024, 1, 4)));
        assert!(!ok(d(2024, 1, 5)));
    }

    #[test]
    fn disabled_days_are_skipped_when_not_counted() {
        let weekend = |d: &SimpleDate| d.weekday_monday_based() >= 5;
        let first = d(2024, 1, 1);
        let sun = d(2024, 1, 7);
        assert_eq!(span_count(first, sun, true, Some(&weekend)), 7);
        assert_eq!(span_count(first, sun, false, Some(&weekend)), 5);
        assert!(!second_click_valid(first, sun, None, Some(6), true, Some(&weekend)));
        assert!(second_click_valid(first, sun, None, Some(6), false, Some(&weekend)));
    }

    #[test]
    fn span_count_without_predicate_matches_inclusive_days() {
        for (a, b) in [
            (d(2024, 1, 1), d(2024, 1, 1)),
            (d(2024, 1, 1), d(2024, 3, 1)),
            (d(2023, 12, 31), d(2024, 1, 1)),
        ] {
            assert_eq!(span_count(a, b, true, None), inclusive_days(a, b));
            assert_eq!(span_count(a, b, false, None), inclusive_days(a, b));
        }
    }

    // --- range_contains_disabled ------------------------------------------

    #[test]
    fn range_contains_disabled_detects_any_day() {
        let weekend = |d: &SimpleDate| d.weekday_monday_based() >= 5;
        let mon_fri = DateRange::new(Some(d(2024, 1, 1)), Some(d(2024, 1, 5)));
        let mon_sun = DateRange::new(Some(d(2024, 1, 1)), Some(d(2024, 1, 7)));
        assert!(!range_contains_disabled(&mon_fri, &weekend));
        assert!(range_contains_disabled(&mon_sun, &weekend));
    }

    #[test]
    fn range_contains_disabled_handles_reversed_and_half_open() {
        let weekend = |d: &SimpleDate| d.weekday_monday_based() >= 5;
        let reversed = DateRange::new(Some(d(2024, 1, 7)), Some(d(2024, 1, 1)));
        assert!(range_contains_disabled(&reversed, &weekend));
        let half = DateRange::new(Some(d(2024, 1, 6)), None);
        assert!(!range_contains_disabled(&half, &weekend));
    }

    // --- classify ---------------------------------------------------------

    #[test]
    fn classify_maps_every_position() {
        let r = DateRange::new(Some(d(2024, 1, 10)), Some(d(2024, 1, 20)));
        assert_eq!(classify(&d(2024, 1, 10), &r), RangePosition::Start);
        assert_eq!(classify(&d(2024, 1, 20), &r), RangePosition::End);
        assert_eq!(classify(&d(2024, 1, 15), &r), RangePosition::Between);
        assert_eq!(classify(&d(2024, 1, 9), &r), RangePosition::Outside);
        assert_eq!(classify(&d(2024, 1, 21), &r), RangePosition::Outside);
    }

    #[test]
    fn classify_handles_single_day_and_half_open() {
        let single = DateRange::single(d(2024, 1, 10));
        assert_eq!(classify(&d(2024, 1, 10), &single), RangePosition::SingleDay);
        assert_eq!(classify(&d(2024, 1, 11), &single), RangePosition::Outside);

        let half = DateRange::new(Some(d(2024, 1, 10)), None);
        assert_eq!(classify(&d(2024, 1, 10), &half), RangePosition::SingleDay);
        assert_eq!(classify(&d(2024, 1, 11), &half), RangePosition::Outside);

        assert_eq!(classify(&d(2024, 1, 10), &DateRange::empty()), RangePosition::Outside);
    }

    #[test]
    fn classify_orders_a_reversed_range() {
        let r = DateRange::new(Some(d(2024, 1, 20)), Some(d(2024, 1, 10)));
        assert_eq!(classify(&d(2024, 1, 10), &r), RangePosition::Start);
        assert_eq!(classify(&d(2024, 1, 20), &r), RangePosition::End);
        assert_eq!(classify(&d(2024, 1, 15), &r), RangePosition::Between);
    }

    // --- preview ----------------------------------------------------------

    #[test]
    fn preview_requires_both_ends_and_distinct_days() {
        assert!(preview_range(None, Some(d(2024, 1, 5))).is_none());
        assert!(preview_range(Some(d(2024, 1, 1)), None).is_none());
        assert!(preview_range(Some(d(2024, 1, 1)), Some(d(2024, 1, 1))).is_none());
        let p = preview_range(Some(d(2024, 1, 10)), Some(d(2024, 1, 5))).unwrap();
        assert_eq!(p, DateRange::new(Some(d(2024, 1, 5)), Some(d(2024, 1, 10))));
    }

    // --- day_at -----------------------------------------------------------

    fn grid() -> Rect {
        Rect::from_min_size(pos2(10.0, 20.0), vec2(DAY_CELL * 7.0, DAY_CELL * 6.0))
    }

    fn cells() -> Vec<SimpleDate> {
        (0..42).map(|i| d(2024, 1, 1).add_days(i)).collect()
    }

    #[test]
    fn day_at_maps_every_cell() {
        let g = grid();
        let c = cells();
        for idx in 0..42usize {
            let col = (idx % 7) as f32;
            let row = (idx / 7) as f32;
            let pos = g.min + vec2(col * DAY_CELL + DAY_CELL * 0.5, row * DAY_CELL + DAY_CELL * 0.5);
            assert_eq!(day_at(pos, g, &c), Some(c[idx]));
        }
    }

    #[test]
    fn day_at_rejects_positions_outside_the_grid() {
        let g = grid();
        let c = cells();
        assert_eq!(day_at(pos2(0.0, 0.0), g, &c), None);
        assert_eq!(day_at(pos2(g.right() + 1.0, g.center().y), g, &c), None);
        assert_eq!(day_at(pos2(g.center().x, g.bottom() + 1.0), g, &c), None);
    }

    #[test]
    fn day_at_tolerates_a_short_cell_list() {
        let g = grid();
        let short = vec![d(2024, 1, 1)];
        let pos = g.min + vec2(DAY_CELL * 3.0 + 1.0, 1.0);
        assert_eq!(day_at(pos, g, &short), None);
    }

    // --- header / navigation ----------------------------------------------

    #[test]
    fn header_title_reflects_view() {
        let vm = d(2024, 3, 1);
        assert_eq!(header_title(vm, 2020, DatePickerView::Day), "March 2024");
        assert_eq!(header_title(vm, 2020, DatePickerView::Month), "2024");
        assert_eq!(header_title(vm, 2020, DatePickerView::Year), "2020 – 2029");
    }

    #[test]
    fn navigate_changes_the_right_field_per_view() {
        let mut vm = d(2024, 3, 1);
        let mut vyd = 2020;
        navigate(&mut vm, &mut vyd, DatePickerView::Day, 1);
        assert_eq!(vm, d(2024, 4, 1));
        navigate(&mut vm, &mut vyd, DatePickerView::Month, -1);
        assert_eq!(vm, d(2023, 4, 1));
        navigate(&mut vm, &mut vyd, DatePickerView::Year, 1);
        assert_eq!(vyd, 2030);
    }

    // --- builders ---------------------------------------------------------

    #[test]
    fn defaults_match_mudblazor() {
        let p: MudDateRangePicker = MudDateRangePicker::new();
        assert!(p.label.is_none());
        assert!(p.placeholder.is_none());
        assert!(p.placeholder_start.is_none());
        assert!(p.placeholder_end.is_none());
        assert!(p.helper_text.is_none());
        assert!(!p.helper_text_on_focus);
        assert!(!p.error);
        assert!(p.error_text.is_none());
        assert!(matches!(p.variant, Variant::Text));
        assert!(matches!(p.margin, Margin::None));
        assert!(matches!(p.color, Color::Primary));
        assert!(matches!(p.size, Size::Medium));
        assert!(!p.clearable && !p.readonly && !p.disabled && !p.editable);
        assert_eq!(p.adornment_icon, filled::CALENDAR_MONTH);
        assert!(p.adornment_color.is_none());
        assert_eq!(p.separator_icon, filled::ARROW_RIGHT_ALT);
        assert!(matches!(p.picker_variant, PickerVariant::Dialog));
        assert!(p.elevation.is_none());
        assert!(!p.rounded && !p.square);
        assert!(p.modal);
        assert!(matches!(p.overflow_behavior, OverflowBehavior::Flip));
        assert!(p.show_toolbar);
        assert!(matches!(p.open_to, OpenTo::Day));
        assert!(p.title_date_format.is_none());
        assert!(p.max_height.is_none());
        assert!(p.auto_close);
        assert!(p.min_date.is_none());
        assert!(p.max_date.is_none());
        assert_eq!(p.first_day_of_week, 0);
        assert!(p.date_format.is_none());
        assert!(p.is_date_disabled.is_none());
        assert!(p.min_days.is_none());
        assert!(p.max_days.is_none());
        assert!(p.allow_disabled_dates_in_count);
        assert!(!p.allow_disabled_dates_in_range);
        assert!(p.picker_actions.is_none());
        assert!(p.on_picker_opened.is_none());
        assert!(p.on_picker_closed.is_none());
        assert!(p.on_click.is_none());
    }

    #[test]
    fn builders_set_every_field() {
        let p: MudDateRangePicker = MudDateRangePicker::new()
            .label("L")
            .placeholder("P")
            .placeholder_start("PS")
            .placeholder_end("PE")
            .helper_text("H")
            .helper_text_on_focus(true)
            .error(true)
            .error_text("E")
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
            .adornment_icon("A")
            .adornment_color(Color::Warning)
            .separator_icon("S")
            .picker_variant(PickerVariant::Inline)
            .elevation(4)
            .rounded(true)
            .square(true)
            .modal(false)
            .overflow_behavior(OverflowBehavior::Reposition)
            .show_toolbar(false)
            .open_to(OpenTo::Month)
            .title_date_format("MMMM dd")
            .max_height(400.0)
            .auto_close(false)
            .min_date(d(2024, 1, 1))
            .max_date(d(2024, 12, 31))
            .first_day_of_week(3)
            .date_format(|dd| format!("{:02}/{:02}/{:04}", dd.day, dd.month, dd.year))
            .is_date_disabled(|_| false)
            .min_days(2)
            .max_days(14)
            .allow_disabled_dates_in_count(false)
            .allow_disabled_dates_in_range(true)
            .picker_actions(|_ui| None)
            .on_picker_opened(|| {})
            .on_picker_closed(|| {})
            .on_click(|| {})
            .classes(["cls"]);

        assert_eq!(p.label.as_deref(), Some("L"));
        assert_eq!(p.placeholder.as_deref(), Some("P"));
        assert_eq!(p.placeholder_start.as_deref(), Some("PS"));
        assert_eq!(p.placeholder_end.as_deref(), Some("PE"));
        assert_eq!(p.helper_text.as_deref(), Some("H"));
        assert!(p.helper_text_on_focus && p.error);
        assert_eq!(p.error_text.as_deref(), Some("E"));
        assert!(matches!(p.variant, Variant::Outlined));
        assert!(matches!(p.margin, Margin::Dense));
        assert!(matches!(p.color, Color::Secondary));
        assert!(matches!(p.size, Size::Large));
        assert!(matches!(p.icon_size, Size::Small));
        assert!(p.clearable && p.readonly && p.disabled && p.editable);
        assert!(p.immediate_text);
        assert_eq!(p.adornment_icon, "A");
        assert_eq!(p.adornment_color, Some(Color::Warning));
        assert_eq!(p.separator_icon, "S");
        assert!(matches!(p.picker_variant, PickerVariant::Inline));
        assert_eq!(p.elevation, Some(4));
        assert!(p.rounded && p.square);
        assert!(!p.modal);
        assert!(matches!(p.overflow_behavior, OverflowBehavior::Reposition));
        assert!(!p.show_toolbar);
        assert!(matches!(p.open_to, OpenTo::Month));
        assert_eq!(p.title_date_format.as_deref(), Some("MMMM dd"));
        assert_eq!(p.max_height, Some(400.0));
        assert!(!p.auto_close);
        assert_eq!(p.min_date, Some(d(2024, 1, 1)));
        assert_eq!(p.max_date, Some(d(2024, 12, 31)));
        assert_eq!(p.first_day_of_week, 3);
        assert!(p.date_format.is_some());
        assert!(p.is_date_disabled.is_some());
        assert_eq!(p.min_days, Some(2));
        assert_eq!(p.max_days, Some(14));
        assert!(!p.allow_disabled_dates_in_count);
        assert!(p.allow_disabled_dates_in_range);
        assert!(p.picker_actions.is_some());
        assert!(p.on_picker_opened.is_some());
        assert!(p.on_picker_closed.is_some());
        assert!(p.on_click.is_some());
        assert!(p.base.has_class("cls"));
    }

    #[test]
    fn day_bounds_clamp_to_at_least_one() {
        let p = MudDateRangePicker::new().min_days(0).max_days(0);
        assert_eq!(p.min_days, Some(1));
        assert_eq!(p.max_days, Some(1));
    }

    #[test]
    fn first_day_of_week_clamps() {
        assert_eq!(
            MudDateRangePicker::new().first_day_of_week(99).first_day_of_week,
            6
        );
    }
}