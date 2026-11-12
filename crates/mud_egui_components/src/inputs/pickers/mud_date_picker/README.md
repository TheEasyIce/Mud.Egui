# MudDatePicker for `mud_egui`

A text field with a popover calendar for choosing a single date, for
[egui](https://github.com/emilk/egui). Ported from MudBlazor's
`MudDatePicker` / `MudBaseDatePicker` / `MudPicker<T>` (MudBlazor 8.13.0).

```rust
let mut state = DatePickerState::new("birthday", SimpleDate::new(2024, 1, 1));

MudDatePicker::new()
    .label("Birthday")
    .variant(Variant::Outlined)
    .min_date(SimpleDate::new(1900, 1, 1))
    .max_date(SimpleDate::new(2100, 12, 31))
    .show(ui, &mut state);
```

The trigger reuses **`MudTextField`** (label, placeholder, helper, error,
variant, margin, size, calendar adornment — all free). The dropdown reuses
**`MudPopover`** (anchor / flip / backdrop / Escape — all free). All
calendar text renders through **`MudText`**; navigation chevrons through
**`MudIcon`**.

**No `chrono` dependency.** The port ships a small pure `SimpleDate` type
that covers everything the calendar needs (leap years, weekday via
Sakamoto's algorithm, month arithmetic, ISO formatting).

---

## Contents

1. [Design principle](#design-principle)
2. [Architecture](#architecture)
3. [API reference](#api-reference)
4. [Examples](#examples)
5. [Calendar views](#calendar-views)
6. [Date arithmetic](#date-arithmetic)
7. [Performance and memory choices](#performance--memory-choices)
8. [Testing](#testing)
9. [Differences from MudBlazor](#differences-from-mudblazor)

---

## Design principle

**Composition, not reimplementation.** `MudDatePicker` follows the same
pattern as `MudSelect` / `MudAutocomplete` / `MudMenu`:

| Primitive | Role |
|-----------|------|
| [`MudTextField`](../../inputs/text_field/README.md) | The trigger field, `readonly` unless `editable(true)`. |
| [`MudPopover`](../../overlays/popover/README.md) | The calendar dropdown. |

Only new code is the **calendar body** (three zoom levels + navigation) and
the **commit / clear logic**.

### Ported parameters

**Base (from `MudPicker<T>`):**

| MudBlazor | Rust method | Default |
|-----------|-------------|---------|
| `Adornment` + `AdornmentIcon` + `AdornmentColor` | `adornment_icon(s)` / `adornment_color(Color)` | `CALENDAR_MONTH` / `None` |
| `AnchorOrigin` | `anchor_origin(..)` | `None` (bottom-left) |
| `Clearable` | `clearable(bool)` | `false` |
| `Color` | `color(Color)` | `Primary` |
| `Disabled` | `disabled(bool)` | `false` |
| `Editable` | `editable(bool)` | `false` |
| `Elevation` | `elevation(u8)` | `None` |
| `HelperText` | `helper_text(s)` | `None` |
| `HelperTextOnFocus` | `helper_text_on_focus(bool)` | `false` |
| `IconSize` | `icon_size(Size)` | `Medium` |
| `ImmediateText` | `immediate_text(bool)` | `false` |
| `Label` | `label(s)` | `None` |
| `Margin` | `margin(Margin)` | `None` |
| `Modal` | `modal(bool)` | `true` |
| `OnClick` | `on_click(f)` | `None` |
| `OverflowBehavior` | `overflow_behavior(..)` | `Flip` |
| `PickerActions` | `picker_actions(f)` | `None` |
| `PickerClosed` | `on_picker_closed(f)` | `None` |
| `PickerOpened` | `on_picker_opened(f)` | `None` |
| `PickerVariant` | `picker_variant(..)` | `Dialog` |
| `Placeholder` | `placeholder(s)` | `None` |
| `ReadOnly` | `readonly(bool)` | `false` |
| `Rounded` | `rounded(bool)` | `false` |
| `ShowToolbar` | `show_toolbar(bool)` | `true` |
| `Square` | `square(bool)` | `false` |
| `Variant` | `variant(Variant)` | `Text` |

**Date-specific (from `MudBaseDatePicker` + `MudDatePicker`):**

| MudBlazor | Rust method | Default |
|-----------|-------------|---------|
| `Date` | `DatePickerState::value` (`Option<SimpleDate>`) | `None` |
| `DateFormat` | `date_format(f)` | ISO 8601 |
| `FirstDayOfWeek` | `first_day_of_week(u8)` | `0` (Monday) |
| `IsDateDisabledFunc` | `is_date_disabled(f)` | `None` |
| `MaxDate` | `max_date(SimpleDate)` | `None` |
| `MinDate` | `min_date(SimpleDate)` | `None` |
| `OpenTo` | `open_to(OpenTo)` | `Day` |
| `PickerMonth` | `DatePickerState::view_month` | initial month |

### Deferred to pass 2

- **`Mask`** — character-by-character input masking.
- **`Editable` typing / input parsing** — the field accepts typed text but
  does not parse it back into a `SimpleDate` yet. Use the calendar.
- **`FixDay` / `FixMonth`** — lock the day or month portion of the value.
- **`RelativeWidth` / `Orientation`** — popover width / orientation options.
- **`ShowToolbar` rendering** — the toolbar (which in MudBlazor shows the
  currently hovered date) is not painted; the parameter is stored.
- **`Inline` / `Static` picker variants** — only `Dialog` renders the
  calendar; `Inline` and `Static` are stored for parity.
- **`PickerActions` custom buttons** — the closure exists and is called
  below the calendar, but pass 1 does not render the standard OK/Cancel
  pair.
- **`TextUpdateSuppression` / `KeyDownPreventDefault`** — browser-specific.
- **`ShrinkLabel` / `Underline` / `ToolbarClass`** — cosmetic / CSS-only.

### Not ported

- **`Class` / `Style` / `Tag` / `UserAttributes`** — no CSS strings or HTML
  tags in egui.

---

## Architecture

```
inputs/picker/
└── simple_date.rs  — SimpleDate + pure calendar helpers

inputs/date_picker/
├── date_picker_state.rs  — DatePickerState (value + text + view + popover)
├── date_picker_view.rs   — public DatePickerView (Day / Month / Year)
├── mud_date_picker.rs    — MudDatePicker + DatePickerResponse + tests
└── mod.rs

mud_egui_core/src/enums/
├── picker_variant.rs  — PickerVariant (Inline / Dialog / Static)
└── open_to.rs         — OpenTo (Day / Month / Year / Hours / Minutes)
```

**Pure layer.** `SimpleDate` (calendar arithmetic), `is_selectable`,
`decade_start`, `rotate_day_names`, `header_title`, `navigate` — free
functions and methods, unit-tested without `egui::Context`.

**State layer.** `DatePickerState` owns the value, text, popover state, view
month, and view year. Persist across frames — this is what makes the picker
usable without callbacks.

**Component layer.** `show` renders the trigger, handles open / close, and
delegates positioning to `MudPopover`. The calendar body is a pure
`render_calendar_body` free function.

---

## API reference

### `MudDatePicker`

| Method | Type | Default | Description |
|--------|------|---------|-------------|
| `new()` | — | — | Default picker. |
| `label(s)` / `placeholder(s)` / `helper_text(s)` | `impl Into<String>` | `None` | Trigger text slots. |
| `helper_text_on_focus(bool)` | `bool` | `false` | Show helper only on focus. |
| `variant(Variant)` / `margin(Margin)` / `color(Color)` / `size(Size)` / `icon_size(Size)` | enum | `Text` / `None` / `Primary` / `Medium` / `Medium` | Trigger appearance. |
| `clearable(bool)` / `readonly(bool)` / `disabled(bool)` / `editable(bool)` | `bool` | `false` | Interaction flags. |
| `immediate_text(bool)` | `bool` | `false` | Stored; no-op. |
| `adornment_icon(s)` / `adornment_color(Color)` | `impl Into<String>` / `Color` | `CALENDAR_MONTH` / `None` | Trigger adornment. |
| `picker_variant(PickerVariant)` | `PickerVariant` | `Dialog` | Inline / Dialog / Static. |
| `elevation(u8)` / `rounded(bool)` / `square(bool)` / `modal(bool)` | `u8` / `bool` | `None` / `false` / `false` / `true` | Popover appearance. |
| `overflow_behavior(OverflowBehavior)` | `OverflowBehavior` | `Flip` | Flip vs reposition. |
| `show_toolbar(bool)` | `bool` | `true` | Stored; no toolbar rendered. |
| `open_to(OpenTo)` | `OpenTo` | `Day` | Initial zoom level. |
| `max_height(f32)` | `f32` | `None` | Popover max height. |
| `min_date(SimpleDate)` / `max_date(SimpleDate)` | `SimpleDate` | `None` | Range clamp. |
| `first_day_of_week(u8)` | `u8` | `0` (Monday) | 0 = Monday … 6 = Sunday. |
| `date_format(f)` | `Fn(&SimpleDate) -> String` | ISO 8601 | Display format. |
| `is_date_disabled(f)` | `Fn(&SimpleDate) -> bool` | `None` | Extra disable predicate. |
| `picker_actions(f)` | `FnMut(&mut Ui, &mut DatePickerState)` | `None` | Custom footer row. |
| `on_picker_opened(f)` / `on_picker_closed(f)` / `on_click(f)` | `FnMut()` | `None` | Callbacks. |
| `classes(iter)` / `flavor(StyleFlavor)` | — | — | Utilities. |
| `show(ui, state)` | `(&mut Ui, &mut DatePickerState)` | — | Returns `DatePickerResponse`. |

### `DatePickerState`

| Field / method | Description |
|----------------|-------------|
| `value: Option<SimpleDate>` | Committed date. |
| `text: String` | Trigger text (kept in sync). |
| `popover: PopoverState` | Popover state. |
| `view_month: SimpleDate` | First day of the month being displayed. |
| `view_year_decade: i32` | Start year of the decade grid. |
| `view: DatePickerView` | Current zoom level. |
| `new(id_salt, initial_view_month)` | Closed state. |
| `with_value(value, text)` | Seed the value. |
| `is_open()` / `open()` / `close()` / `toggle()` | Popover control. |
| `commit(value, text)` | Set value and close. |
| `clear()` | Reset value, close. |

### `DatePickerView`

| Variant | Zoom level |
|---------|------------|
| `Day` (default) | Calendar month grid. |
| `Month` | 12 months of one year. |
| `Year` | A decade of years. |

### `DatePickerResponse`

| Field | Type | Description |
|-------|------|-------------|
| `trigger` | `egui::Response` | Trigger response. |
| `value` | `Option<SimpleDate>` | Committed value after this frame. |
| `text` | `String` | Trigger text after this frame. |
| `action` | `DatePickerAction` | What happened this frame. |

### `DatePickerAction`

| Variant | Description |
|---------|-------------|
| `None` | No interaction. |
| `Opened` / `Closed` | Popover opened / closed. |
| `DateSelected(SimpleDate)` | A day was picked and committed. |
| `Cleared` | The value was cleared. |

Helpers: `changed_value()`, `toggled_open()`.

### `SimpleDate`

| Member | Description |
|--------|-------------|
| `new(y, m, d)` | Clamped constructor. |
| `first_of_month(y, m)` | Constructs `YYYY-MM-01`. |
| `is_leap_year(y)` | Static leap-year check. |
| `days_in_month(y, m)` | Days in a month. |
| `weekday_monday_based()` | 0 = Monday … 6 = Sunday. |
| `add_days(n)` / `add_months(n)` / `add_years(n)` | Roll-over arithmetic. |
| `first_of_this_month()` / `last_of_this_month()` | Bounds. |
| `is_before(o)` / `is_after(o)` / `in_range(min, max)` / `clamped(min, max)` | Comparisons. |
| `format_iso()` / `to_string()` | `"YYYY-MM-DD"`. |
| `calendar_grid(y, m, first_day_of_week)` | 42-day grid for the calendar. |
| `same_month(o)` | Same year + month. |

---

## Examples

### Basic picker

```rust
let mut state = DatePickerState::new("birthday", SimpleDate::new(2024, 1, 1));
MudDatePicker::new()
    .label("Birthday")
    .show(ui, &mut state);
```

### Range clamping

```rust
MudDatePicker::new()
    .min_date(SimpleDate::new(1900, 1, 1))
    .max_date(SimpleDate::new(2100, 12, 31))
    .show(ui, &mut state);
```

### Custom formatter (`DD/MM/YYYY`)

```rust
MudDatePicker::new()
    .date_format(|d| format!("{:02}/{:02}/{:04}", d.day, d.month, d.year))
    .show(ui, &mut state);
```

### Weekend-disabled picker

```rust
MudDatePicker::new()
    .is_date_disabled(|d| d.weekday_monday_based() >= 5)
    .show(ui, &mut state);
```

### Sunday-first weeks

```rust
MudDatePicker::new()
    .first_day_of_week(6)
    .show(ui, &mut state);
```

### Open on year view

```rust
MudDatePicker::new()
    .open_to(OpenTo::Year)
    .show(ui, &mut state);
```

### Clearable

```rust
MudDatePicker::new()
    .clearable(true)
    .show(ui, &mut state);
// Clicking the clear icon sets `state.value = None`.
```

### Custom footer actions

```rust
MudDatePicker::new()
    .picker_actions(|ui, state| {
        ui.horizontal(|ui| {
            if MudButton::new("Today").size(Size::Small).show(ui).clicked() {
                let today = SimpleDate::new(2024, 6, 15); // caller-chosen
                state.commit(today, "2024-06-15");
            }
        });
    })
    .show(ui, &mut state);
```

### Read the outcome

```rust
let resp = MudDatePicker::new().show(ui, &mut state);
match resp.action {
    DatePickerAction::DateSelected(d) => log::info!("picked {d}"),
    DatePickerAction::Cleared => log::info!("cleared"),
    DatePickerAction::Opened => log::info!("opened"),
    DatePickerAction::Closed => log::info!("closed"),
    DatePickerAction::None => {}
}
```

### Programmatic selection

```rust
state.commit(SimpleDate::new(2024, 12, 25), "2024-12-25");
state.clear();
```

---

## Calendar views

The calendar has three zoom levels, navigable by clicking the header title
(zoom out) or picking a cell (zoom in):

| View | Grid shows | `<` / `>` navigates | Clicking a cell |
|------|------------|---------------------|-----------------|
| `Day` (default) | 6 weeks × 7 days of one month | One month | Commits the day |
| `Month` | 4 rows × 3 months of one year | One year | Zooms to `Day` on that month |
| `Year` | 2 rows × 5 years of a decade | One decade | Zooms to `Month` on that year |

Clicking the header title zooms out one level. `DatePickerView` is the
public enum — `DatePickerState::view` tracks the current level, so callers
can preset it.

### Selection and disabled state

- The current `value` is highlighted in the accent colour.
- Days outside `min_date` / `max_date` are rendered in `text_disabled`.
- Days matching `is_date_disabled(date)` are also disabled.
- Disabled cells still render (so the grid layout is stable) but ignore
  clicks.

---

## Date arithmetic

`SimpleDate` implements everything the calendar needs without a
dependency:

- **Leap years** — full Gregorian rule (`year % 4 == 0 && year % 100 != 0 ||
  year % 400 == 0`).
- **`weekday_monday_based()`** — Sakamoto's algorithm. Constant time, no
  lookup beyond 12 entries.
- **`add_days` / `add_months` / `add_years`** — proper month and year
  roll-over, day clamping (31 Jan + 1 month → 28 Feb or 29 Feb).
- **`calendar_grid(y, m, first_day_of_week)`** — 42-day grid (6 weeks), so
  the calendar height never shifts between months.

All arithmetic is unit-tested with fixed dates (2024-01-01 = Monday,
2025-01-01 = Wednesday, leap-year Feb 29 handling, decade boundaries).

**No "today" concept** — without a system clock, "today" is caller-owned.
Pass `SimpleDate::new(...)` from your caller for the initial view month,
or wire `std::time::SystemTime` to a `SimpleDate` manually.

---

## Performance and memory choices

| Choice | Effect |
|--------|--------|
| `SimpleDate` is `Copy` (12 bytes) | Zero allocation for date values. |
| Pure calendar arithmetic | Testable without `egui::Context`. |
| `calendar_grid` returns 42 `Copy` dates | Fits in 1 stack-allocated vec. |
| `Header / grid rendering use `allocate_exact_size` + child `Ui`s | No per-cell layout passes. |
| `MudIcon` reuse across chevrons | Free thanks to its transform cache. |
| `MudText` for all calendar text | Theme and typography propagate. |
| `MudTextField` / `MudPopover` reuse | Zero duplication of trigger chrome or positioning. |
| `PickerPalette` resolved once per frame | One palette lookup regardless of grid size. |

---

## Testing

`cargo test` runs 30+ pure-function tests across three files:

| Area | Tests |
|------|-------|
| `SimpleDate` | clamping, leap years, days_in_month, weekday (known dates), add_days / add_months / add_years roll-over, in_range, clamped, format_iso, calendar_grid (42 days, correct start), same_month |
| `DatePickerView` | default, zoom_out chain, zoom_in chain |
| `DatePickerState` | new / with_value, open_resyncs_view, commit, clear, toggle |
| `PickerVariant` / `OpenTo` | default, name, is_dialog / is_date_variant predicates |
| Picker helpers | `variant_opens_popover`, `is_selectable` (min/max/predicate), `decade_start`, `rotate_day_names`, `header_title`, `navigate` per view |
| Defaults / builders | every field |
| `DatePickerAction` | changed_value / toggled_open |

Rendering (`show`) needs a live `egui::Context`; the pure layer is the
source of truth.

```bash
cargo test -p mud_egui_components mud_date_picker
cargo test -p mud_egui_components simple_date
cargo test -p mud_egui_core picker_variant open_to
```

---

## Differences from MudBlazor

- **No `chrono`.** The `SimpleDate` type is a small pure date with just
  enough arithmetic. Callers needing timezone handling or `DateTime` need
  their own bridge.
- **No `MudColorPicker` / `MudTimePicker` yet.** Those ship in the next
  two messages, reusing the same `PickerVariant`, `OpenTo`, and shared
  `MudPicker<T>` structure.
- **`Inline` / `Static` picker variants are stored but not rendered.** Only
  `Dialog` shows the calendar; the other variants are accepted for API
  parity.
- **`Mask` / `Editable` typing are deferred.** The field accepts typed
  text but does not parse it back into `SimpleDate`.
- **`FixDay` / `FixMonth` deferred.**
- **`Text` is caller-owned via `DatePickerState::text`.** MudBlazor's
  `Text` is bidirectionally bound; the Rust port stores it in the state.
- **`Style` / `Tag` / `UserAttributes` not ported.** No CSS strings or HTML
  tags.