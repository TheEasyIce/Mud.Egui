//! `MudDatePicker` — a text field with a popover calendar.
//!
//! Port of `MudDatePicker.razor.cs` + `MudBaseDatePicker.razor.cs` +
//! `MudPicker<T>.razor.cs` (MudBlazor 8.13.0). Pass 1.
//!
//! The public parameter surface mirrors the Blazor component exactly:
//!
//! | MudBlazor parameter | Rust field / method |
//! |---------------------|---------------------|
//! | `Adornment` (base) | fixed to `End` |
//! | `AdornmentColor` | [`MudDatePicker::adornment_color`] |
//! | `AdornmentIcon` | [`MudDatePicker::adornment_icon`] (default `CALENDAR_MONTH`) |
//! | `AnchorOrigin` | fixed to `Bottom` via [`picker_origins`] |
//! | `Clearable` | [`MudDatePicker::clearable`] |
//! | `Color` | [`MudDatePicker::color`] (default `Primary`) |
//! | `Date` | [`DatePickerState::value`] (caller-owned) |
//! | `DateFormat` | [`MudDatePicker::date_format`] (default ISO 8601) |
//! | `Disabled` | [`MudDatePicker::disabled`] |
//! | `Editable` | [`MudDatePicker::editable`] |
//! | `Elevation` | [`MudDatePicker::elevation`] |
//! | `FirstDayOfWeek` | [`MudDatePicker::first_day_of_week`] (default `0` = Monday) |
//! | `HelperText` | [`MudDatePicker::helper_text`] |
//! | `HelperTextOnFocus` | [`MudDatePicker::helper_text_on_focus`] |
//! | `IconSize` | [`MudDatePicker::icon_size`] |
//! | `ImmediateText` | [`MudDatePicker::immediate_text`] — stored, no-op |
//! | `IsDateDisabledFunc` | [`MudDatePicker::is_date_disabled`] |
//! | `Label` | [`MudDatePicker::label`] |
//! | `Margin` | [`MudDatePicker::margin`] |
//! | `MaxDate` | [`MudDatePicker::max_date`] |
//! | `MinDate` | [`MudDatePicker::min_date`] |
//! | `Modal` | [`MudDatePicker::modal`] |
//! | `OnClick` | [`MudDatePicker::on_click`] |
//! | `OpenTo` | [`MudDatePicker::open_to`] |
//! | `OverflowBehavior` | [`MudDatePicker::overflow_behavior`] |
//! | `PickerActions` | [`MudDatePicker::picker_actions`] |
//! | `PickerClosed` | [`MudDatePicker::on_picker_closed`] |
//! | `PickerOpened` | [`MudDatePicker::on_picker_opened`] |
//! | `PickerVariant` | [`MudDatePicker::picker_variant`] (default `Dialog`) |
//! | `Placeholder` | [`MudDatePicker::placeholder`] |
//! | `ReadOnly` | [`MudDatePicker::readonly`] |
//! | `Rounded` | [`MudDatePicker::rounded`] |
//! | `ShowToolbar` | [`MudDatePicker::show_toolbar`] — stored, no toolbar rendered |
//! | `Size` | [`MudDatePicker::size`] |
//! | `Square` | [`MudDatePicker::square`] |
//! | `Text` | [`DatePickerState::text`] (caller-owned) |
//! | `Variant` | [`MudDatePicker::variant`] (default `Text`) |
//! | `Class` (base) | [`MudDatePicker::classes`] — stored, not resolved |
//! | `Mask` / `FixDay` / `FixMonth` / `RelativeWidth` / `Orientation` / `Inline` | deferred to pass 2 |
//! | `Style` / `Tag` / `UserAttributes` | not ported: no CSS strings, no HTML tag |
//!
//! # Composition, not reimplementation
//!
//! | Primitive | Role |
//! |-----------|------|
//! | [`MudTextField`] | The trigger (adorned with a calendar icon). |
//! | [`MudPopover`] | The calendar dropdown. |
//!
//! Only new code is the **calendar body** (day grid, month grid, year grid,
//! navigation) and the **commit / clear logic**. [`SimpleDate`] supplies all
//! calendar arithmetic.
//!
//! # Text and icons
//!
//! Trigger label / placeholder / helper / error and the calendar's day
//! numbers / month names / year numbers render through [`MudText`].
//! Chevrons and the calendar adornment render through [`MudIcon`].
//!
//! # Interaction
//!
//! | Trigger click | Effect |
//! |---------------|--------|
//! | On the field body | Toggles the popover (when `picker_variant.is_dialog()`) |
//! | On the clear button | Clears `state.value` and `state.text`, closes the popover |
//!
//! The clear-button check runs **before** the popover-toggle check, so a
//! clear click does not also open the calendar. A field-erased empty string
//! in `editable(true)` mode is treated the same way (empty text = no date).
//!
//! # Calendar views
//!
//! Three zoom levels, navigable by clicking the header title (zoom out) or
//! picking a cell (zoom in):
//!
//! | View | Grid shows | `<` / `>` navigates | Clicking a cell |
//! |------|------------|---------------------|-----------------|
//! | `Day` (default) | 6 weeks × 7 days of one month | One month | Commits the day |
//! | `Month` | 4 rows × 3 months of one year | One year | Zooms to `Day` |
//! | `Year` | 2 rows × 5 years of a decade | One decade | Zooms to `Month` |
//!
//! # State
//!
//! [`DatePickerState::text`] is **not** re-synced from
//! [`DatePickerState::value`] on every frame — `commit()` / `clear()` /
//! `with_value()` keep the two in sync, so typed edits in `editable(true)`
//! mode are preserved. Call `state.commit(..)` to change the value
//! programmatically.
//!
//! # Performance
//!
//! - Theme fetched once per frame; the eight palette colours used by the
//!   calendar are resolved once into [`PickerPalette`].
//! - One `MudPopover` render per open frame; the calendar body is a
//!   single allocation of `Ui` per row / cell — no `Vec`, no `String`
//!   beyond what `MudText` and `SimpleDate::calendar_grid` allocate.
//! - All hit-test rects are computed from `CalendarPainter` constants; no
//!   per-frame text measurement.
//! - Pure helpers for visibility, view navigation and date range checks, all
//!   `#[inline]` where the compiler allows.

use egui::{
    pos2, vec2, Color32, CornerRadius, CursorIcon, Layout, Rect, Response, Sense,
    Stroke, Ui, UiBuilder, Vec2,
};
use mud_egui_bootstrap::bs_style::BsStyle;
use mud_egui_core::base::component_base::ComponentBase;
use mud_egui_core::enums::{
    AnchorOrigin, BackdropMode, Color, Margin, OpenTo, OverflowBehavior, PickerVariant, Size,
    StyleFlavor, TransformOrigin, Typo, Variant,
};
use mud_egui_icons_material::filled;
use mud_egui_tailwind::tw_style::{TwState, TwStyle};
use mud_egui_theming::models::palettes::palette::Palette;
use mud_egui_theming::theme::MudTheme;

use crate::data_display::icons::mud_icon::MudIcon;
use crate::feedback::popover::MudPopover;
use crate::inputs::text_field::{AdornmentEnd, MudTextField};
use crate::typography::mud_text::MudText;

use super::date_picker_action::DatePickerAction;
use super::date_picker_state::DatePickerState;
use super::date_picker_view::DatePickerView;
pub(crate) use super::simple_date::SimpleDate;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Base rem size passed to the Tailwind resolver.
const REM_SIZE_PT: f32 = 14.0;

/// Side length of one day cell in the calendar grid.
pub const DAY_CELL: f32 = 36.0;

/// Height of the calendar header row (chevrons + title).
pub const HEADER_HEIGHT: f32 = 40.0;

/// Horizontal / vertical padding inside the calendar body.
pub const CALENDAR_PADDING: f32 = 8.0;

/// Corner radius of a highlighted day / month / year cell.
const CELL_RADIUS: u8 = 18;

/// Approximate calendar width (7 cells × `DAY_CELL` + 2 × `CALENDAR_PADDING`).
pub const CALENDAR_WIDTH: f32 = 7.0 * DAY_CELL + 2.0 * CALENDAR_PADDING;

/// Number of years displayed per decade in the year grid.
const YEARS_PER_DECADE: i32 = 10;

/// Day-name column headers (Monday-first; rotated by `first_day_of_week`).
pub const DEFAULT_DAY_NAMES: [&str; 7] = ["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"];

/// Month names shown in the month grid.
pub const MONTH_NAMES: [&str; 12] = [
    "January", "February", "March", "April", "May", "June",
    "July", "August", "September", "October", "November", "December",
];

// ---------------------------------------------------------------------------
// DatePickerResponse
// ---------------------------------------------------------------------------

/// The result of one [`MudDatePicker::show`] frame.
#[derive(Debug)]
pub struct DatePickerResponse {
    /// The trigger's own `Response`.
    pub trigger: Response,
    /// Committed value after this frame.
    pub value: Option<SimpleDate>,
    /// Trigger text after this frame.
    pub text: String,
    /// What the picker did this frame.
    pub action: DatePickerAction,
}

// ---------------------------------------------------------------------------
// MudDatePicker
// ---------------------------------------------------------------------------

/// A text field with a popover calendar.
///
/// Mirrors `MudDatePicker` in MudBlazor 8.13.0. The Blazor two-way bindings
/// `Date` / `DateChanged` become a caller-owned [`DatePickerState::value`]
/// plus a [`DatePickerResponse::action`] report — the same split
/// `MudRating`, `MudCheckBox` and `MudRadio` use.
#[must_use = "MudDatePicker does nothing until you call `.show(ui, state)`"]
pub struct MudDatePicker<'a> {
    // ── Trigger text field ─────────────────────────────────────────────────
    /// Caption above the trigger. Mirrors `Label`.
    pub label: Option<String>,
    /// Hint shown when the trigger is empty. Mirrors `Placeholder`.
    pub placeholder: Option<String>,
    /// Caption below the trigger. Mirrors `HelperText`.
    pub helper_text: Option<String>,
    /// Show `helper_text` only while the field is focused or has content.
    /// Mirrors `HelperTextOnFocus`.
    pub helper_text_on_focus: bool,
    /// Trigger variant (Text / Outlined / Filled). Mirrors `Variant`.
    pub variant: Variant,
    /// Vertical density of the trigger. Mirrors `Margin`.
    pub margin: Margin,
    /// Trigger accent colour. Mirrors `Color`.
    pub color: Color,
    /// Trigger height / font size. Mirrors `Size`.
    pub size: Size,
    /// Calendar adornment icon size. Mirrors `IconSize`.
    pub icon_size: Size,
    /// Shows a clear button on the trigger. Mirrors `Clearable`.
    pub clearable: bool,
    /// Prevents editing without dimming. Mirrors `ReadOnly`.
    pub readonly: bool,
    /// Prevents all interaction and dims. Mirrors `Disabled`.
    pub disabled: bool,
    /// Allows typing into the trigger. Mirrors `Editable`.
    pub editable: bool,
    /// Stored for parity. Mirrors `ImmediateText`.
    pub immediate_text: bool,

    // ── Adornment ──────────────────────────────────────────────────────────
    /// Trailing icon glyph. Mirrors `AdornmentIcon`.
    pub adornment_icon: String,
    /// Trailing icon colour override. Mirrors `AdornmentColor`.
    pub adornment_color: Option<Color>,

    // ── Picker variant / display ───────────────────────────────────────────
    /// `Inline` / `Dialog` / `Static`. Mirrors `PickerVariant`.
    pub picker_variant: PickerVariant,
    /// Popover shadow depth. Mirrors `Elevation`.
    pub elevation: Option<u8>,
    /// Rounded popover corners. Mirrors `Rounded`.
    pub rounded: bool,
    /// Square popover corners. Mirrors `Square`.
    pub square: bool,
    /// Dimmed backdrop blocks the parent UI. Mirrors `Modal`.
    pub modal: bool,
    /// Flip / reposition / shrink. Mirrors `OverflowBehavior`.
    pub overflow_behavior: OverflowBehavior,
    /// Stored for parity. Mirrors `ShowToolbar`.
    pub show_toolbar: bool,
    /// Initial zoom level. Mirrors `OpenTo`.
    pub open_to: OpenTo,
    /// Popover height cap. Mirrors `RelativeWidth` (stored only).
    pub max_height: Option<f32>,

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

    // ── Customisation ──────────────────────────────────────────────────────
    /// Custom footer row inside the calendar. Returns `Some(action)` to
    /// request a state change — the picker applies it after the popover
    /// closure returns.
    pub picker_actions:
        Option<Box<dyn FnMut(&mut Ui) -> Option<DatePickerAction> + 'a>>,
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
    /// Bootstrap resolver cache.
    pub bs_style: BsStyle,
    /// Tailwind resolver cache.
    pub tw_style: TwStyle,
    /// Tailwind variant state (hovered / active / dark).
    pub tw_state: TwState,
}

impl<'a> Default for MudDatePicker<'a> {
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
            adornment_icon: filled::CALENDAR_MONTH.to_owned(),
            adornment_color: None,
            picker_variant: PickerVariant::Dialog,
            elevation: None,
            rounded: false,
            square: false,
            modal: true,
            overflow_behavior: OverflowBehavior::Flip,
            show_toolbar: true,
            open_to: OpenTo::Day,
            max_height: None,
            min_date: None,
            max_date: None,
            first_day_of_week: 0,
            date_format: None,
            is_date_disabled: None,
            picker_actions: None,
            on_picker_opened: None,
            on_picker_closed: None,
            on_click: None,
            base: ComponentBase::new(),
            flavor: StyleFlavor::default(),
            bs_style: BsStyle::default(),
            tw_style: TwStyle::default(),
            tw_state: TwState::default(),
        }
    }
}

impl<'a> MudDatePicker<'a> {
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
    #[inline] pub fn open_to(mut self, o: OpenTo) -> Self { self.open_to = o; self }
    #[inline] pub fn max_height(mut self, px: f32) -> Self { self.max_height = Some(px); self }
    #[inline] pub fn min_date(mut self, d: SimpleDate) -> Self { self.min_date = Some(d); self }
    #[inline] pub fn max_date(mut self, d: SimpleDate) -> Self { self.max_date = Some(d); self }
    #[inline] pub fn first_day_of_week(mut self, d: u8) -> Self { self.first_day_of_week = d.min(6); self }
    #[inline] pub fn date_format(mut self, f: impl Fn(&SimpleDate) -> String + 'a) -> Self { self.date_format = Some(Box::new(f)); self }
    #[inline] pub fn is_date_disabled(mut self, f: impl Fn(&SimpleDate) -> bool + 'a) -> Self { self.is_date_disabled = Some(Box::new(f)); self }
    #[inline] pub fn picker_actions(mut self, f: impl FnMut(&mut Ui) -> Option<DatePickerAction> + 'a) -> Self { self.picker_actions = Some(Box::new(f)); self }
    #[inline] pub fn on_picker_opened(mut self, f: impl FnMut() + 'a) -> Self { self.on_picker_opened = Some(Box::new(f)); self }
    #[inline] pub fn on_picker_closed(mut self, f: impl FnMut() + 'a) -> Self { self.on_picker_closed = Some(Box::new(f)); self }
    #[inline] pub fn on_click(mut self, f: impl FnMut() + 'a) -> Self { self.on_click = Some(Box::new(f)); self }
    #[inline] pub fn classes<S: AsRef<str>>(mut self, classes: impl IntoIterator<Item = S>) -> Self { self.base = self.base.classes(classes); self }
    #[inline] pub fn flavor(mut self, f: StyleFlavor) -> Self { self.flavor = f; self }

    // ── Rendering ──────────────────────────────────────────────────────────

    /// Draws the trigger and, when open, the calendar popover.
    pub fn show(mut self, ui: &mut Ui, state: &mut DatePickerState) -> DatePickerResponse {
        // 1. Render the trigger.
        //
        // `MudTextField::new(&mut state.text)` holds a **mutable** borrow of
        // `state.text` until `.show(ui)` consumes `trigger`. We therefore
        // snapshot every field of `state` we need while that borrow is live
        // (here: `state.text.is_empty()` and `state.popover.open`) before
        // constructing the trigger.
        let was_open = state.popover.open;
        let text_is_empty = state.text.is_empty();
        let show_helper_text = self.helper_text.is_some()
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
        if show_helper_text {
            if let Some(s) = self.helper_text.clone() { trigger = trigger.helper_text(s); }
        }
        if self.clearable { trigger = trigger.clearable(true); }
        let trigger_resp = trigger.show(ui);

        // 2. Resolve interaction.
        //
        // `trigger.show(ui)` consumed the builder, so the mutable borrow of
        // `state.text` has been released; reading `state.text` here is safe.
        let mut action = DatePickerAction::None;
        let clear_clicked = self.clearable
            && trigger_resp.changed
            && state.text.is_empty()
            && state.value.is_some();

        if !clear_clicked
            && trigger_resp.response.clicked()
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
                action = DatePickerAction::Opened;
            } else {
                if let Some(cb) = self.on_picker_closed.as_mut() { cb(); }
                action = DatePickerAction::Closed;
            }
        }

        if clear_clicked {
            state.clear();
            action = DatePickerAction::Cleared;
        }

        // 3. Render the popover — unchanged from your version.
        if state.popover.open && self.picker_variant.is_dialog() {
            let theme = MudTheme::fetch(ui.ctx()).unwrap_or_default();
            let palette = theme.current_palette();
            let picker_palette = resolve_palette(self.color, palette, ui);

            let (anchor, transform) = picker_origins();

            let mut popover = MudPopover::new()
                .anchor_origin(anchor)
                .transform_origin(transform)
                .paper(true)
                .elevation(self.elevation.unwrap_or(8))
                .overflow_behavior(self.overflow_behavior)
                .size_hint(vec2(CALENDAR_WIDTH, 320.0))
                .match_anchor_width(false);

            if self.modal {
                popover = popover.backdrop(BackdropMode::Transparent);
            }

            let child_rect = trigger_resp.response.rect;

            let mut clicked_date: Option<SimpleDate> = None;
            let mut view_clicked: Option<DatePickerView> = None;
            let mut deferred_action: Option<DatePickerAction> = None;

            let mut picker_actions = self.picker_actions.take();

            let first_day_of_week = self.first_day_of_week;
            let min_date = self.min_date;
            let max_date = self.max_date;
            let user_disabled = self.is_date_disabled.as_deref();

            let value = state.value;
            let view = state.view;

            let _ = popover.show(ui, &mut state.popover, child_rect, |pop_ui| {
                render_calendar_body(
                    pop_ui,
                    &mut state.view_month,
                    &mut state.view_year_decade,
                    view,
                    value,
                    &mut picker_actions,
                    &picker_palette,
                    first_day_of_week,
                    min_date.as_ref(),
                    max_date.as_ref(),
                    user_disabled,
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
            if let Some(date) = clicked_date {
                let text = apply_format(&date, formatter);
                state.commit(date, text);
                action = DatePickerAction::DateSelected(date);
                if let Some(cb) = self.on_picker_closed.as_mut() { cb(); }
            } else if let Some(deferred) = deferred_action {
                match deferred {
                    DatePickerAction::DateSelected(date) => {
                        let text = apply_format(&date, formatter);
                        state.commit(date, text);
                        action = DatePickerAction::DateSelected(date);
                        if let Some(cb) = self.on_picker_closed.as_mut() { cb(); }
                    }
                    DatePickerAction::Cleared => {
                        state.clear();
                        action = DatePickerAction::Cleared;
                        if let Some(cb) = self.on_picker_closed.as_mut() { cb(); }
                    }
                    other => action = other,
                }
            }
        }

        DatePickerResponse {
            trigger: trigger_resp.response,
            value: state.value,
            text: state.text.clone(),
            action,
        }
    }
}

// ---------------------------------------------------------------------------
// Pure helpers (no `egui::Ui` required)
// ---------------------------------------------------------------------------

/// Opens a picker's popover if the picker variant allows it.
#[inline]
pub fn variant_opens_popover(variant: PickerVariant) -> bool {
    variant.is_dialog()
}

/// `true` when `date` can be selected given the current `min` / `max` /
/// user-supplied `is_date_disabled` predicate.
pub fn is_selectable(
    date: &SimpleDate,
    min: Option<&SimpleDate>,
    max: Option<&SimpleDate>,
    user_disabled: Option<&dyn Fn(&SimpleDate) -> bool>,
) -> bool {
    if !date.in_range(min, max) {
        return false;
    }
    if let Some(pred) = user_disabled {
        if pred(date) {
            return false;
        }
    }
    true
}

/// Decade start (rounds down to the nearest multiple of ten).
#[inline]
pub fn decade_start(year: i32) -> i32 {
    year.div_euclid(YEARS_PER_DECADE) * YEARS_PER_DECADE
}

/// Day-name list rotated to start on `first_day_of_week`.
pub fn rotate_day_names(first_day_of_week: u8) -> [&'static str; 7] {
    let mut out: [&'static str; 7] = DEFAULT_DAY_NAMES;
    if first_day_of_week > 0 && first_day_of_week < 7 {
        out.rotate_left(first_day_of_week as usize);
    }
    out
}

/// Bottom-left anchor pair for the picker popover.
#[inline]
pub fn picker_origins() -> (AnchorOrigin, TransformOrigin) {
    (AnchorOrigin::Bottom, TransformOrigin::TopLeft)
}

/// ISO 8601 — `YYYY-MM-DD`. Used when `DateFormat` is not set.
fn default_date_format(d: &SimpleDate) -> String {
    d.format_iso()
}

/// Applies `f` if set, else the ISO 8601 default.
fn apply_format(
    d: &SimpleDate,
    f: Option<&dyn Fn(&SimpleDate) -> String>,
) -> String {
    match f {
        Some(f) => f(d),
        None => default_date_format(d),
    }
}

// ---------------------------------------------------------------------------
// Palette resolution
// ---------------------------------------------------------------------------

/// The eight colours the calendar body needs, resolved once per frame.
struct PickerPalette {
    text_primary: Color32,
    text_secondary: Color32,
    text_disabled: Color32,
    accent: Color32,
    accent_text: Color32,
    hover_fill: Color32,
    today_outline: Color32,
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
        today_outline: accent,
        divider: palette.divider.into(),
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
    value: Option<SimpleDate>,
    picker_actions: &mut Option<Box<dyn FnMut(&mut Ui) -> Option<DatePickerAction> + '_>>,
    p: &PickerPalette,
    first_day_of_week: u8,
    min: Option<&SimpleDate>,
    max: Option<&SimpleDate>,
    user_disabled: Option<&dyn Fn(&SimpleDate) -> bool>,
    clicked_date: &mut Option<SimpleDate>,
    view_clicked: &mut Option<DatePickerView>,
    deferred_action: &mut Option<DatePickerAction>,
) {
    render_header(ui, view_month, view_year_decade, view, p, view_clicked);

    match view {
        DatePickerView::Day => render_day_grid(
            ui,
            *view_month,
            value,
            p,
            first_day_of_week,
            min,
            max,
            user_disabled,
            clicked_date,
        ),
        DatePickerView::Month => render_month_grid(ui, view_month, value, p, view_clicked),
        DatePickerView::Year => render_year_grid(
            ui,
            view_month,
            *view_year_decade,
            value,
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

    // Previous.
    let prev_id = ui.scope_id().with("datepicker_prev");
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

    // Next.
    let next_id = ui.scope_id().with("datepicker_next");
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

    // Title.
    let title = header_title(*view_month, *view_year_decade, view);
    let title_rect = Rect::from_min_max(
        pos2(prev_rect.right() + 4.0, rect.top()),
        pos2(next_rect.left() - 4.0, rect.bottom()),
    );
    let title_id = ui.scope_id().with("datepicker_title");
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

fn header_title(view_month: SimpleDate, view_year_decade: i32, view: DatePickerView) -> String {
    match view {
        DatePickerView::Day => {
            let name = MONTH_NAMES[(view_month.month - 1) as usize];
            format!("{name} {}", view_month.year)
        }
        DatePickerView::Month => format!("{}", view_month.year),
        DatePickerView::Year => {
            let start = view_year_decade;
            let end = start + YEARS_PER_DECADE - 1;
            format!("{start} – {end}")
        }
    }
}

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

fn render_day_grid(
    ui: &mut Ui,
    view_month: SimpleDate,
    value: Option<SimpleDate>,
    p: &PickerPalette,
    first_day_of_week: u8,
    min: Option<&SimpleDate>,
    max: Option<&SimpleDate>,
    user_disabled: Option<&dyn Fn(&SimpleDate) -> bool>,
    clicked_date: &mut Option<SimpleDate>,
) {
    let names = rotate_day_names(first_day_of_week);

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        for name in names {
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

    let grid = SimpleDate::calendar_grid(view_month.year, view_month.month, first_day_of_week)
        .unwrap_or_default();
    let current_month = view_month.month;
    let current_year = view_month.year;

    for row in 0..6 {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            for col in 0..7 {
                let idx = row * 7 + col;
                if let Some(date) = grid.get(idx) {
                    let is_other_month =
                        date.month != current_month || date.year != current_year;
                    let is_selected = value.as_ref() == Some(date);
                    let selectable = is_selectable(date, min, max, user_disabled);
                    render_day_cell(
                        ui,
                        *date,
                        is_other_month,
                        is_selected,
                        selectable,
                        p,
                        clicked_date,
                    );
                } else {
                    let _ =
                        ui.allocate_exact_size(vec2(DAY_CELL, DAY_CELL), Sense::hover());
                }
            }
        });
    }
}

#[allow(clippy::too_many_arguments)]
fn render_day_cell(
    ui: &mut Ui,
    date: SimpleDate,
    is_other_month: bool,
    is_selected: bool,
    selectable: bool,
    p: &PickerPalette,
    clicked_date: &mut Option<SimpleDate>,
) {
    let (rect, resp) = ui.allocate_exact_size(
        Vec2::splat(DAY_CELL),
        if selectable { Sense::click() } else { Sense::hover() },
    );
    let inner = rect.shrink(2.0);
    let radius = CornerRadius::same(CELL_RADIUS);

    let text_color = if !selectable || is_other_month {
        p.text_disabled
    } else if is_selected {
        p.accent_text
    } else {
        p.text_primary
    };

    if is_selected {
        ui.painter().rect_filled(inner, radius, p.accent);
    } else if resp.hovered() && selectable {
        ui.painter().rect_filled(inner, radius, p.hover_fill);
    }
    if resp.hovered() && selectable {
        ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
    }
    if resp.clicked() && selectable {
        *clicked_date = Some(date);
    }

    let mut child = ui.new_child(
        UiBuilder::new()
            .max_rect(inner)
            .layout(Layout::centered_and_justified(egui::Direction::LeftToRight)),
    );
    child.add(
        MudText::new(date.day.to_string())
            .typo(Typo::Body2)
            .custom_color(text_color),
    );
}

fn render_month_grid(
    ui: &mut Ui,
    view_month: &mut SimpleDate,
    value: Option<SimpleDate>,
    p: &PickerPalette,
    view_clicked: &mut Option<DatePickerView>,
) {
    let year = view_month.year;
    for row in 0..4 {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            for col in 0..3 {
                let month_idx = row * 3 + col;
                let month_name = MONTH_NAMES[month_idx];
                let month_num = (month_idx + 1) as u8;
                let is_selected = value
                    .map(|v| v.year == year && v.month == month_num)
                    .unwrap_or(false);
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
                    *view_month = SimpleDate::first_of_month(year, month_num);
                    *view_clicked = Some(DatePickerView::Day);
                }
                let text_color = if is_selected { p.accent_text } else { p.text_primary };
                let mut child = ui.new_child(
                    UiBuilder::new()
                        .max_rect(inner)
                        .layout(Layout::centered_and_justified(
                            egui::Direction::LeftToRight,
                        )),
                );
                child.add(
                    MudText::new(month_name)
                        .typo(Typo::Body2)
                        .custom_color(text_color),
                );
            }
        });
    }
}

fn render_year_grid(
    ui: &mut Ui,
    view_month: &mut SimpleDate,
    view_year_decade: i32,
    value: Option<SimpleDate>,
    p: &PickerPalette,
    view_clicked: &mut Option<DatePickerView>,
) {
    let start = view_year_decade;
    for row in 0..2 {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            for col in 0..5 {
                let year = start + row * 5 + col;
                let is_selected = value.map(|v| v.year == year).unwrap_or(false);
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
                let text_color = if is_selected { p.accent_text } else { p.text_primary };
                let mut child = ui.new_child(
                    UiBuilder::new()
                        .max_rect(inner)
                        .layout(Layout::centered_and_justified(
                            egui::Direction::LeftToRight,
                        )),
                );
                child.add(
                    MudText::new(year.to_string())
                        .typo(Typo::Body2)
                        .custom_color(text_color),
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

    // --- variant ----------------------------------------------------------

    #[test]
    fn variant_opens_popover_only_for_dialog() {
        assert!(!variant_opens_popover(PickerVariant::Inline));
        assert!(variant_opens_popover(PickerVariant::Dialog));
        assert!(!variant_opens_popover(PickerVariant::Static));
    }

    // --- selectable -------------------------------------------------------

    #[test]
    fn is_selectable_respects_min_max() {
        let min = d(2024, 1, 1);
        let max = d(2024, 12, 31);
        assert!(is_selectable(&d(2024, 6, 15), Some(&min), Some(&max), None));
        assert!(!is_selectable(&d(2023, 12, 31), Some(&min), Some(&max), None));
        assert!(!is_selectable(&d(2025, 1, 1), Some(&min), Some(&max), None));
        assert!(is_selectable(&d(2024, 6, 15), None, None, None));
    }

    #[test]
    fn is_selectable_respects_user_predicate() {
        let disabled = |date: &SimpleDate| date.weekday_monday_based() >= 5;
        assert!(is_selectable(&d(2024, 1, 1), None, None, Some(&disabled)));
        assert!(!is_selectable(&d(2024, 1, 6), None, None, Some(&disabled)));
    }

    // --- decade / day names -----------------------------------------------

    #[test]
    fn decade_start_rounds_down() {
        assert_eq!(decade_start(2024), 2020);
        assert_eq!(decade_start(2020), 2020);
        assert_eq!(decade_start(2019), 2010);
        assert_eq!(decade_start(0), 0);
        assert_eq!(decade_start(-5), -10);
    }

    #[test]
    fn rotate_day_names_shifts_left() {
        assert_eq!(rotate_day_names(0), ["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"]);
        assert_eq!(rotate_day_names(1)[0], "Tu");
        assert_eq!(rotate_day_names(6)[0], "Su");
    }

    // --- origins ----------------------------------------------------------

    #[test]
    fn picker_origins_are_bottom_left() {
        let (a, t) = picker_origins();
        assert_eq!(a, AnchorOrigin::Bottom);
        assert_eq!(t, TransformOrigin::TopLeft);
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
        navigate(&mut vm, &mut vyd, DatePickerView::Day, -1);
        assert_eq!(vm, d(2024, 3, 1));

        navigate(&mut vm, &mut vyd, DatePickerView::Month, 1);
        assert_eq!(vm, d(2025, 3, 1));

        navigate(&mut vm, &mut vyd, DatePickerView::Year, 1);
        assert_eq!(vyd, 2030);
        navigate(&mut vm, &mut vyd, DatePickerView::Year, -1);
        assert_eq!(vyd, 2020);
    }

    // --- formatting -------------------------------------------------------

    #[test]
    fn default_date_format_is_iso() {
        assert_eq!(default_date_format(&d(2024, 3, 14)), "2024-03-14");
    }

    #[test]
    fn apply_format_uses_custom_or_default() {
        let custom: &dyn Fn(&SimpleDate) -> String =
            &|dd: &SimpleDate| format!("{:02}/{:02}/{:04}", dd.day, dd.month, dd.year);
        assert_eq!(apply_format(&d(2024, 3, 14), Some(custom)), "14/03/2024");
        assert_eq!(apply_format(&d(2024, 3, 14), None), "2024-03-14");
    }

    // --- builders ---------------------------------------------------------

    #[test]
    fn defaults_match_mudblazor() {
        let p: MudDatePicker = MudDatePicker::new();
        assert!(p.label.is_none());
        assert!(p.placeholder.is_none());
        assert!(p.helper_text.is_none());
        assert!(!p.helper_text_on_focus);
        assert!(matches!(p.variant, Variant::Text));
        assert!(matches!(p.margin, Margin::None));
        assert!(matches!(p.color, Color::Primary));
        assert!(matches!(p.size, Size::Medium));
        assert!(!p.clearable);
        assert!(!p.readonly && !p.disabled && !p.editable);
        assert_eq!(p.adornment_icon, filled::CALENDAR_MONTH);
        assert!(p.adornment_color.is_none());
        assert!(matches!(p.picker_variant, PickerVariant::Dialog));
        assert!(p.elevation.is_none());
        assert!(!p.rounded && !p.square);
        assert!(p.modal);
        assert!(matches!(p.overflow_behavior, OverflowBehavior::Flip));
        assert!(p.show_toolbar);
        assert!(matches!(p.open_to, OpenTo::Day));
        assert!(p.max_height.is_none());
        assert!(p.min_date.is_none());
        assert!(p.max_date.is_none());
        assert_eq!(p.first_day_of_week, 0);
        assert!(p.date_format.is_none());
        assert!(p.is_date_disabled.is_none());
        assert!(p.picker_actions.is_none());
        assert!(p.on_picker_opened.is_none());
        assert!(p.on_picker_closed.is_none());
        assert!(p.on_click.is_none());
    }

    #[test]
    fn builders_set_every_field() {
        let p: MudDatePicker = MudDatePicker::new()
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
            .open_to(OpenTo::Year)
            .max_height(400.0)
            .min_date(d(2024, 1, 1))
            .max_date(d(2024, 12, 31))
            .first_day_of_week(3)
            .date_format(|dd| format!("{}/{}/{}", dd.month, dd.day, dd.year))
            .is_date_disabled(|_| false)
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
        assert!(matches!(p.open_to, OpenTo::Year));
        assert_eq!(p.max_height, Some(400.0));
        assert_eq!(p.min_date, Some(d(2024, 1, 1)));
        assert_eq!(p.max_date, Some(d(2024, 12, 31)));
        assert_eq!(p.first_day_of_week, 3);
        assert!(p.date_format.is_some());
        assert!(p.is_date_disabled.is_some());
        assert!(p.picker_actions.is_some());
        assert!(p.on_picker_opened.is_some());
        assert!(p.on_picker_closed.is_some());
        assert!(p.on_click.is_some());
        assert!(p.base.has_class("cls"));
    }

    #[test]
    fn first_day_of_week_clamps() {
        let p = MudDatePicker::new().first_day_of_week(99);
        assert_eq!(p.first_day_of_week, 6);
    }
}