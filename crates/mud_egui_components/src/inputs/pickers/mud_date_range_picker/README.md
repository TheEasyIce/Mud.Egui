# MudDateRangePicker for `mud_egui`

A pair of text fields with a popover calendar that selects a **start** and an
**end** date, for [egui](https://github.com/emilk/egui). Ported from
MudBlazor's [`MudDateRangePicker`](https://mudblazor.com/components/datepicker)
(MudBlazor 8.13.0).

```rust
let mut state = DateRangePickerState::new("booking", SimpleDate::new(2024, 1, 1));

MudDateRangePicker::new()
    .label("Stay")
    .placeholder_start("Arrival")
    .placeholder_end("Departure")
    .min_days(1)
    .max_days(30)
    .show(ui, &mut state);

// After a completed selection: state.range is start..=end, ordered.
```

The trigger is two **`MudTextField`**s separated by a **`MudIcon`** — the egui
equivalent of `MudRangeInput`. The dropdown reuses **`MudPopover`**. Every
numeral and every label renders through **`MudText`**.

**No `chrono` dependency.** `SimpleDate` (from `MudDatePicker`) supplies the
calendar arithmetic; `DateRange` is a small pair of optional dates.

---

## Contents

1. [Design principle: API parity, not API invention](#design-principle-api-parity-not-api-invention)
2. [Architecture](#architecture)
3. [API reference](#api-reference)
4. [Examples](#examples)
5. [Two-click selection and validation](#two-click-selection-and-validation)
6. [Performance and memory choices](#performance-and-memory-choices)
7. [Testing](#testing)
8. [Differences from MudBlazor](#differences-from-mudblazor)

---

## Design principle: API parity, not API invention

The MudBlazor 8.13.0 parameter surface was verified against DNDocs before
implementation. `MudDateRangePicker` has **ten own parameters** plus the
inherited `MudBaseDatePicker` + `MudPicker<T>` surface, and every one is
accounted for.

### Own parameters

| MudBlazor | Rust method | Default |
|---|---|---|
| `AllowDisabledDatesInCount` | `allow_disabled_dates_in_count(bool)` | **`true`** |
| `AllowDisabledDatesInRange` | `allow_disabled_dates_in_range(bool)` | **`false`** |
| `AutoClose` | `auto_close(bool)` | **`true`** |
| `DateRange` | `DateRangePickerState::range` | empty |
| `DateRangeChanged` | `DateRangePickerResponse::action` | — |
| `MaxDays` | `max_days(u32)` | `None` |
| `MinDays` | `min_days(u32)` | `None` |
| `PlaceholderEnd` | `placeholder_end(s)` | `None` |
| `PlaceholderStart` | `placeholder_start(s)` | `None` |
| `SeparatorIcon` | `separator_icon(s)` | `ARROW_RIGHT_ALT` |
| `InputContent` / `IsRange` | **protected** — not ported | — |

### Inherited parameters

`AdornmentIcon`, `AdornmentColor`, `AnchorOrigin`, `Clearable`, `Color`,
`Disabled`, `Editable`, `Elevation`, `HelperText`, `HelperTextOnFocus`,
`IconSize`, `ImmediateText`, `Label`, `Margin`, `Modal`, `OnClick`,
`OverflowBehavior`, `PickerActions`, `PickerClosed`, `PickerOpened`,
`PickerVariant`, `Placeholder`, `ReadOnly`, `Rounded`, `ShowToolbar`, `Size`,
`Square`, `Variant`, `Class`, `DateFormat`, `FirstDayOfWeek`,
`IsDateDisabledFunc`, `MaxDate`, `MinDate`, `OpenTo`, `PickerMonth`,
`TitleDateFormat`.

### Deliberately not rendered

| MudBlazor | Reason |
|---|---|
| `AdornmentIcon` / `AdornmentColor` | `MudRangeInput` has no adornment slot — the separator icon is the only chrome between the fields. Stored so a Blazor call site ports without edits. |
| `Placeholder` | Overridden by `PlaceholderStart` / `PlaceholderEnd` in `MudRangeInput`. Stored. |
| `TitleDateFormat` / `ShowToolbar` | The toolbar is not painted (same deferral as `MudDatePicker`). |
| `IconSize` | The calendar chevrons are a fixed 20 pt, as in `MudDatePicker`. |
| `Style` / `Tag` / `UserAttributes` | No CSS strings, no HTML tag. |
| `Class` resolution | Stored on `base` only. |

### Deferred to pass 2

`Mask`, typed `Editable` parsing, `Inline` / `Static` rendering,
`ClosingDelay` timing (commit closes in the same frame), `FixDay` /
`FixMonth`, `ShrinkLabel`, `Underline`.

---

## Architecture

```
inputs/date_range_picker/
├── date_range.rs                DateRange + inclusive_days
├── range_position.rs            pub(crate) RangePosition (own file, Rule 2)
├── date_range_picker_action.rs  DateRangePickerAction (own file, Rule 2)
├── date_range_picker_state.rs   DateRangePickerState (caller-owned)
├── mud_date_range_picker.rs     MudDateRangePicker + response + helpers + tests
└── mod.rs
```

**Reused from `MudDatePicker`** (imported, not duplicated):
`SimpleDate`, `DatePickerView`, `is_selectable`, `rotate_day_names`, and the
calendar geometry constants (`DAY_CELL`, `HEADER_HEIGHT`, `CALENDAR_PADDING`,
`CALENDAR_WIDTH`, `MONTH_NAMES`, `DEFAULT_DAY_NAMES`). Requires one extra
`pub use simple_date::SimpleDate;` in `date_picker/mod.rs`.

**Pure layer.** `inclusive_days`, `DateRange::*`, `second_click_valid`,
`span_count`, `range_contains_disabled`, `classify`, `preview_range`,
`day_at`, `header_title`, `navigate` — free functions or methods over
`SimpleDate`, `DateRange`, `Pos2` and `Rect`. None touch `egui::Context`.

**State layer.** `DateRangePickerState { range, text_start, text_end, popover,
view_month, view_year_decade, view, pending_start }` is caller-owned.
`pending_start` is the egui equivalent of MudBlazor's private `_firstDate`.

**Component layer.** `show` renders the label, the two fields and the
separator, the helper/error row, then delegates the calendar to
`render_calendar_body` and applies the deferred clicks after
`popover.show` releases `&mut state.popover`.

---

## API reference

### `MudDateRangePicker<'a>`

| Method | Type | Default | Description |
|---|---|---|---|
| `new()` | — | — | Default picker. |
| `label(s)` | `impl Into<String>` | `None` | Caption above the pair. |
| `placeholder(s)` | `impl Into<String>` | `None` | Base placeholder — stored, overridden by the two below. |
| `placeholder_start(s)` / `placeholder_end(s)` | `impl Into<String>` | `None` | Per-field hints. |
| `helper_text(s)` | `impl Into<String>` | `None` | Caption below the pair. |
| `helper_text_on_focus(bool)` | `bool` | `false` | Helper only while focused / non-empty. |
| `error(bool)` / `error_text(s)` | `bool` / `impl Into<String>` | `false` / `None` | Error accent and message. |
| `variant(Variant)` / `margin(Margin)` / `color(Color)` / `size(Size)` / `icon_size(Size)` | enum | `Text` / `None` / `Primary` / `Medium` / `Medium` | Field appearance. |
| `clearable` / `readonly` / `disabled` / `editable` | `bool` | `false` | Interaction flags. |
| `immediate_text(bool)` | `bool` | `false` | Stored, no-op. |
| `adornment_icon(s)` / `adornment_color(Color)` | — | `CALENDAR_MONTH` / `None` | **Stored, not rendered** (`MudRangeInput` has no adornment slot). |
| `separator_icon(s)` | `impl Into<String>` | `ARROW_RIGHT_ALT` | Glyph between the fields. |
| `picker_variant(PickerVariant)` | — | `Dialog` | Only `Dialog` renders. |
| `elevation(u8)` / `rounded(bool)` / `square(bool)` / `modal(bool)` | — | `None` / `false` / `false` / `true` | Popover appearance. |
| `overflow_behavior(OverflowBehavior)` | — | `Flip` | Flip vs reposition. |
| `show_toolbar(bool)` / `title_date_format(s)` | — | `true` / `None` | Stored — the toolbar is not painted. |
| `open_to(OpenTo)` | `OpenTo` | `Day` | Initial zoom level. |
| `max_height(f32)` | `f32` | `None` | Stored (popover size hint). |
| `auto_close(bool)` | `bool` | `true` | Close after the range is committed. |
| `min_date(SimpleDate)` / `max_date(SimpleDate)` | `SimpleDate` | `None` | Selectable range clamp. |
| `first_day_of_week(u8)` | `u8` | `0` (Monday) | 0 = Monday … 6 = Sunday. |
| `date_format(f)` | `Fn(&SimpleDate) -> String` | ISO 8601 | Display format for both fields. |
| `is_date_disabled(f)` | `Fn(&SimpleDate) -> bool` | `None` | Extra disable predicate. |
| `min_days(u32)` / `max_days(u32)` | `u32` | `None` | Inclusive day-count bounds; clamped to ≥ 1. |
| `allow_disabled_dates_in_count(bool)` | `bool` | `true` | Count disabled days toward the bounds. |
| `allow_disabled_dates_in_range(bool)` | `bool` | `false` | Allow a committed range containing a disabled day. |
| `picker_actions(f)` | `FnMut(&mut Ui) -> Option<DateRangePickerAction>` | `None` | Custom footer row. |
| `on_picker_opened` / `on_picker_closed` / `on_click` | `FnMut()` | `None` | Callbacks. |
| `classes(iter)` / `flavor(StyleFlavor)` | — | — | Utilities. |
| `show(ui, state)` | `(&mut Ui, &mut DateRangePickerState)` | — | Returns `DateRangePickerResponse`. |

### `DateRangePickerState`

| Field / method | Description |
|---|---|
| `range: DateRange` | Committed range. |
| `text_start` / `text_end: String` | Trigger texts (kept in sync by `commit_range` / `clear` / `with_range`). |
| `popover: PopoverState` | Popover state. |
| `view_month: SimpleDate` | First day of the displayed month. |
| `view_year_decade: i32` | Start year of the decade grid. |
| `view: DatePickerView` | Current zoom level. |
| `pending_start: Option<SimpleDate>` | First click of the two-click selection. |
| `new(id_salt, initial_view_month)` | Closed state. |
| `with_range(range, text_start, text_end)` | Seed (normalises). |
| `is_open()` / `open()` / `close()` / `toggle()` | Popover control. |
| `commit_range(range, text_start, text_end)` | Commit and close. |
| `clear()` | Reset range and texts, close. |
| `display_range()` | `pending_start`-aware range for the calendar. |
| `first_click()` | The pending start, if any. |

### `DateRangePickerResponse`

| Field | Type | Description |
|---|---|---|
| `trigger_start` | `egui::Response` | Start field response. |
| `trigger_end` | `egui::Response` | End field response. |
| `range` | `DateRange` | Committed range after this frame. |
| `action` | `DateRangePickerAction` | What happened this frame. |

### `DateRangePickerAction`

| Variant | Description |
|---|---|
| `None` | No interaction. |
| `Opened` / `Closed` | Popover opened / closed. |
| `RangeSelected(DateRange)` | A complete range was committed. Mirrors `DateRangeChanged`. |
| `Cleared` | The range was cleared. |

Helpers: `changed_value()`, `toggled_open()`, `selected()`, `name()`.

### `DateRange`

| Member | Description |
|---|---|
| `new(start, end)` / `empty()` / `single(date)` | Constructors. |
| `is_empty()` / `is_complete()` / `is_same_day()` | Predicates. |
| `normalized()` | Orders `start <= end`. |
| `contains(&date)` | Inclusive membership (half-open = start only). |
| `day_count()` | Inclusive day count. |
| `formatted(format)` | `(String, String)` for the two fields. |
| `Display` | `start ..= end`, `—` for a missing end. |

### Free helpers

| Function | Signature | Description |
|---|---|---|
| `inclusive_days` | `(SimpleDate, SimpleDate) -> i32` | Inclusive span, either order. |
| `second_click_valid` | `(first, second, min, max, allow_in_count, disabled) -> bool` | Second-click validation. |
| `span_count` | `(first, second, allow_disabled, disabled) -> i32` | Inclusive count, optionally skipping disabled days. |
| `range_contains_disabled` | `(&DateRange, &dyn Fn(&SimpleDate) -> bool) -> bool` | Any disabled day in the span. |
| `day_at` | `(Pos2, Rect, &[SimpleDate]) -> Option<SimpleDate>` | Grid hit-test. |
| `format_one` | `(Option<SimpleDate>, format) -> String` | One field's text. |

### Constants

| Name | Value |
|---|---|
| `SEPARATOR_SIZE` | `20.0` |
| `SEPARATOR_GAP` | `4.0` |

---

## Examples

### Basic range picker

```rust
let mut state = DateRangePickerState::new("range", SimpleDate::new(2024, 1, 1));
MudDateRangePicker::new().label("Period").show(ui, &mut state);
```

### Arrival / departure with a 30-day cap

```rust
MudDateRangePicker::new()
    .label("Stay")
    .placeholder_start("Arrival")
    .placeholder_end("Departure")
    .min_days(1)
    .max_days(30)
    .show(ui, &mut state);
```

Days that would exceed the cap are greyed out immediately after the first
click.

### Weekends excluded from the day count

```rust
let weekend = |d: &SimpleDate| d.weekday_monday_based() >= 5;

MudDateRangePicker::new()
    .is_date_disabled(weekend)
    .allow_disabled_dates_in_count(false) // 5 business days may span 7 calendar days
    .max_days(5)
    .show(ui, &mut state);
```

### Reject ranges that cross a disabled day

```rust
MudDateRangePicker::new()
    .is_date_disabled(|d| d == SimpleDate::new(2024, 12, 25))
    .allow_disabled_dates_in_range(false) // default
    .show(ui, &mut state);
```

A range that spans Christmas Day is rejected; the first click stays pending.

### Keep the popover open for multiple picks

```rust
MudDateRangePicker::new()
    .auto_close(false)
    .show(ui, &mut state);
```

### Custom separator glyph

```rust
MudDateRangePicker::new()
    .separator_icon(filled::REMOVE)
    .show(ui, &mut state);
```

### Custom date format

```rust
MudDateRangePicker::new()
    .date_format(|d| format!("{:02}/{:02}/{:04}", d.day, d.month, d.year))
    .show(ui, &mut state);
// Both fields and the committed range use it.
```

### Reacting to the outcome

```rust
let resp = MudDateRangePicker::new().show(ui, &mut state);
match resp.action {
    DateRangePickerAction::RangeSelected(r) => log::info!("{r} ({} days)", r.day_count().unwrap()),
    DateRangePickerAction::Cleared => log::info!("cleared"),
    DateRangePickerAction::Opened => log::info!("opened"),
    DateRangePickerAction::Closed => log::info!("closed"),
    DateRangePickerAction::None => {}
}
```

### Live preview from the first click

```rust
let resp = MudDateRangePicker::new().show(ui, &mut state);
if let Some(first) = state.first_click() {
    // The user has clicked once; the calendar is previewing first ..= hover.
    log::debug!("selection started at {first}");
}
```

### Programmatic range

```rust
state.commit_range(
    DateRange::new(Some(SimpleDate::new(2024, 6, 1)), Some(SimpleDate::new(2024, 6, 30))),
    "2024-06-01",
    "2024-06-30",
);
state.clear();
```

---

## Two-click selection and validation

### Click flow

| Click | Effect |
|---|---|
| First | `pending_start = Some(day)`; the calendar shows it as a single-day range and previews `day ..= hover` |
| Second | Orders the pair, checks `MinDays` / `MaxDays` / disabled days, commits `DateRange`, closes if `auto_close` or no `picker_actions` |
| Third | Starts a fresh selection (`pending_start` is cleared on every commit) |

The committed range is **always ordered** so `start <= end`, whatever the
click order — MudBlazor's `_firstDate` / `_secondDate` normalisation.

### Validation

| Parameter | Effect | Default |
|---|---|---|
| `min_days(n)` | Rejects a span shorter than `n` days, inclusive | none |
| `max_days(n)` | Rejects a span longer than `n` days, inclusive | none |
| `allow_disabled_dates_in_count(b)` | When `false`, disabled days are skipped in the count | `true` |
| `allow_disabled_dates_in_range(b)` | When `false`, a span containing a disabled day is rejected | `false` |

`MinDays` / `MaxDays` are **inclusive of the selected date**: a same-day
range has a count of `1`, so `max_days(1)` allows exactly one day and
`min_days(2)` forbids it.

A rejected second click leaves `pending_start` in place, so the user can pick
a different end without restarting.

### Visual states

| State | Fill |
|---|---|
| `Start` / `End` | Accent pill, rounded on the outer edge only |
| `SingleDay` | Accent pill, fully rounded |
| `Between` | Accent at 60 α, full-width band |
| Preview (hovering the second date) | Accent at 32 α |
| Hover on a selectable day | `widgets.hovered.weak_bg_fill` |

Ink is `accent_contrast_text` on endpoints, `text_primary` elsewhere, and
`text_disabled` for unselectable or out-of-month days.

---

## Performance and memory choices

| Choice | Effect |
|---|---|
| `DateRange` is `Copy` (two `Option<SimpleDate>`) | Zero allocation for the value. |
| `inclusive_days` uses a Julian-day number | `O(1)` day counting, no `add_days` loops. |
| Pure validation helpers | `second_click_valid` / `span_count` / `classify` are testable without a `Context`. |
| One interaction region for the day grid | One `allocate_exact_size` + one `Response` for 42 cells; the hover preview is computed *before* painting. |
| `day_at` is a pure grid hit-test | No per-cell `interact` calls. |
| Numerals through `MudText` in child `Ui`s | Rule 3 compliant; no `painter.text`. |
| Bands and pills through `ui.painter()` | Geometry only — no user-facing text bypasses `MudText`. |
| Theme fetched once per frame | One `MudTheme::fetch`; the palette is resolved once. |
| `MudTextField` / `MudPopover` / `MudDatePicker` reuse | No duplicated trigger chrome, positioning, or calendar arithmetic. |
| Text snapshotted before the field borrow | `state.text_*` is read before `MudTextField::new(&mut state.text_*)` takes it. |

---

## Testing

`cargo test -p mud_egui_components mud_date_range_picker` runs the module's
tests:

| Area | Tests |
|---|---|
| `DateRange` | empty/default, `new` preserves order, `single`, completeness, `is_same_day`, `normalized` (ordering, idempotence, partial), `contains` (inclusive, reversed, half-open, empty), `day_count`, `formatted` (closure, ISO fallback, blank end), `Display` |
| `inclusive_days` | same day, adjacent, full month, month and year boundaries, leap years, a ten-year span |
| `second_click_valid` | no bounds, `max_days` inclusive both directions, `min_days` inclusive, both composing, disabled-day counting on/off |
| `span_count` | matches `inclusive_days` without a predicate; skips disabled days with one |
| `range_contains_disabled` | inside, absent, reversed range, half-open range |
| `classify` | every position, single day, half-open, reversed, empty |
| `preview_range` | missing ends, same day, reversed pair normalised |
| `day_at` | every cell of the 7 × 6 grid, outside the grid, short cell list |
| `header_title` / `navigate` | all three views |
| `RangePosition` | `is_endpoint`, `is_band`, unique names |
| `DateRangePickerAction` | default, `changed_value`, `toggled_open`, `selected`, names |
| `DateRangePickerState` | defaults, `with_range` (seed + normalise), `open` / `close` / `toggle`, `open` drops the pending start, `commit_range`, `clear`, `display_range`, `first_click` |
| Builders | defaults match MudBlazor, every field set, `min_days` / `max_days` clamp, `first_day_of_week` clamp |

Rendering needs a live `egui::Context`; the pure layer is the source of truth.

```bash
cargo test -p mud_egui_components mud_date_range_picker
cargo test -p mud_egui_components date_range
cargo test -p mud_egui_components date_range_picker_state
cargo test -p mud_egui_components date_range_picker_action
```

---

## Differences from MudBlazor

- **No `chrono`.** `SimpleDate` is a small pure date; `DateRange` holds two
  `Option<SimpleDate>`.
- **`DateRange` is a `Copy` value type with `Option` endpoints**, mirroring
  `Range<DateTime?>`. v9 makes `Start` / `End` init-only; the port keeps them
  mutable, matching 8.13.
- **`DateRangeChanged` becomes `DateRangePickerResponse::action`.** The
  caller owns `state.range`; `RangeSelected(_)` reports the committing frame.
  `Opened` / `Closed` mirror `PickerOpened` / `PickerClosed`. Same split as
  `DatePickerAction` and `MudRating`.
- **The trigger is two `MudTextField`s + a `MudIcon`**, the egui equivalent
  of `MudRangeInput`. The label and the helper/error row are painted once
  above/below the pair, not per field.
- **`AdornmentIcon` / `AdornmentColor` / `Placeholder` are stored, not
  rendered.** `MudRangeInput` overrides `InputContent` and has no adornment
  slot; the separator icon is the only chrome between the fields.
- **`AutoClose` is ported and defaults to `true`.** MudBlazor also closes when
  no `PickerActions` are defined, which the port reproduces
  (`auto_close || picker_actions.is_none()`).
- **`MinDays` / `MaxDays` are validated per second click, not via cached
  bounds.** Semantically identical to MudBlazor's `_minValidDate` /
  `_maxValidDate` recalculation; the predicate form is simpler and testable.
- **`allow_disabled_dates_in_count` defaults to `true`**, so disabled days
  count toward the bounds unless the caller opts out — matching the DNDocs
  remark.
- **`allow_disabled_dates_in_range` defaults to `false`**, so a range
  spanning a disabled day is rejected. The rejected click leaves the pending
  start in place.
- **`Inline` / `Static` picker variants are stored but not rendered.** Only
  `Dialog` shows the calendar.
- **`ClosingDelay` timing is not implemented**; a commit closes in the same
  frame.
- **`Style` / `Tag` / `UserAttributes` not ported.** No CSS strings or HTML
  tags.