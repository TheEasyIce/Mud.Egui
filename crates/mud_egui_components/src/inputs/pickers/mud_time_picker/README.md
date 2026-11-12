# MudTimePicker for `mud_egui`

A text field with a popover clock dial for choosing a time of day, for
[egui](https://github.com/emilk/egui). Ported from MudBlazor's
[`MudTimePicker`](https://mudblazor.com/components/time-picker)
(MudBlazor 8.13.0).

```rust
let mut state = TimePickerState::new("meeting");

MudTimePicker::new()
.label("Meeting time")
.am_pm(true)
.minute_selection_step(15)
.show(ui, &mut state);
```

The trigger reuses **`MudTextField`**. The dropdown reuses **`MudPopover`**.
Every numeral and every AM/PM marker renders through **`MudText`**; the
adornment and the clear button render through **`MudIcon`**.

**No `chrono` dependency.** A small pure `SimpleTime` type covers everything
the dial needs.

---

## Contents

1. [Design principle](#design-principle)
2. [Architecture](#architecture)
3. [API reference](#api-reference)
4. [Examples](#examples)
5. [Clock views and commit flow](#clock-views-and-commit-flow)
6. [Formatting and arithmetic](#formatting-and-arithmetic)
7. [Performance and memory choices](#performance-and-memory-choices)
8. [Testing](#testing)
9. [Differences from MudBlazor](#differences-from-mudblazor)

---

## Design principle

**Composition, not reimplementation.** `MudTimePicker` follows the same
pattern as `MudDatePicker` / `MudSelect` / `MudMenu`:

| Primitive | Role |
|-----------|------|
| [`MudTextField`](../../inputs/text_field/README.md) | The trigger, `readonly` unless `editable(true)`. |
| [`MudPopover`](../../overlays/popover/README.md) | The clock dropdown. |

Only the **clock dial** (ring outlines, hand, numerals, AM/PM markers) and the
**hour → minute commit logic** are new.

### Ported parameters

**Time-specific:**

| MudBlazor | Rust method | Default |
|-----------|-------------|---------|
| `AmPm` | `am_pm(bool)` | `false` |
| `AutoClose` | `auto_close(bool)` | `false` |
| `ClosingDelay` | `closing_delay(u32)` | `200` ms |
| `MinuteSelectionStep` | `minute_selection_step(u8)` | `1` |
| `OpenTo` | `open_to(OpenTo)` | `OpenTo::Hours` |
| `Time` | `TimePickerState::value` | `None` |
| `TimeChanged` | `TimePickerResponse::action` | — |
| `TimeEditMode` | `time_edit_mode(TimeEditMode)` | `Normal` |
| `TimeFormat` | `time_format(&str)` | `hh:mm tt` / `HH:mm` |

**Base (`MudPicker<T>`):** `Adornment` + `AdornmentIcon` + `AdornmentColor`,
`AnchorOrigin`, `Clearable`, `Color`, `Disabled`, `Editable`, `Elevation`,
`HelperText`, `HelperTextOnFocus`, `IconSize`, `ImmediateText`, `Label`,
`Margin`, `Modal`, `OnClick`, `OverflowBehavior`, `PickerActions`,
`PickerClosed`, `PickerOpened`, `PickerVariant`, `Placeholder`, `ReadOnly`,
`Rounded`, `ShowToolbar`, `Size`, `Square`, `Variant`, `Class`.

**Pointer state:** `PointerMoving` is a public parameter in 8.13.0
(`public bool PointerMoving { get; set; }`)[reference:7]. The port derives it
from the dial's `Response` and uses it to suppress the hand animation.

### Deferred to pass 2

- **`Mask`** — character-by-character input masking.
- **`Editable` typed parsing** — the field accepts text but does not parse
  it back into a `SimpleTime` yet. Use the dial.
- **`Inline` / `Static` picker variants** — stored; only `Dialog` renders.
- **`ShowToolbar` rendering** — stored; no toolbar is painted.
- **`TextUpdateSuppression` / `KeyDownPreventDefault`** — browser-specific.
- **`ShrinkLabel` / `Underline` / `ToolbarClass`** — cosmetic.

### Not ported

- **`Class` resolution** — stored on `base` for parity only.
- **`Style` / `Tag` / `UserAttributes`** — no CSS strings, no HTML tag.

---

## Architecture

```
inputs/time_picker/
├── simple_time.rs          SimpleTime + formatting + arithmetic
├── time_picker_view.rs     TimePickerView (Hours | Minutes)
├── time_picker_action.rs   TimePickerAction (own file, Rule 2)
├── time_picker_state.rs    TimePickerState (caller-owned)
├── mud_time_picker.rs      MudTimePicker + TimePickerResponse + dial + tests
└── mod.rs

mud_egui_core/src/enums/
└── time_edit_mode.rs       TimeEditMode (Normal | OnlyMinutes | OnlyHours)
```

**Pure layer.** `snap_minute`, `clock_angle`, `clock_index`,
`outer_hour_for_index`, `inner_hour_for_index`, `is_outer_ring`,
`numeral_pos`, `hour_at`, `minute_at`, `flip_am_pm`, `view_for_open` and
every `SimpleTime` method are free functions over `f32`, `u8`, `Pos2` and
`SimpleTime`. None touch `egui::Context`.

**State layer.** `TimePickerState { value, text, popover, view, pending_hour }`
is caller-owned. `pending_hour` bridges the hour and minute steps of one
selection so the commit can happen on the minute click.

**Component layer.** `show` renders the trigger, honours any `AutoClose`
deadline, opens/closes the popover, renders the dial, then applies the
deferred clicks to `state` after `popover.show` releases
`&mut state.popover`.

---

## API reference

### `MudTimePicker<'a>`

| Method | Type | Default | Description |
|--------|------|---------|-------------|
| `new()` | — | — | Default picker. |
| `label` / `placeholder` / `helper_text` | `impl Into<String>` | `None` | Trigger text slots. |
| `helper_text_on_focus(bool)` | `bool` | `false` | Helper only while focused / non-empty. |
| `variant(Variant)` / `margin(Margin)` / `color(Color)` / `size(Size)` / `icon_size(Size)` | enum | `Text` / `None` / `Primary` / `Medium` / `Medium` | Trigger appearance. |
| `clearable` / `readonly` / `disabled` / `editable` | `bool` | `false` | Interaction flags. |
| `immediate_text(bool)` | `bool` | `false` | Stored, no-op. |
| `adornment_icon` / `adornment_color` | `impl Into<String>` / `Color` | `ACCESS_TIME` / `None` | Trigger adornment. |
| `picker_variant(PickerVariant)` | `PickerVariant` | `Dialog` | Only `Dialog` renders. |
| `elevation(u8)` / `rounded(bool)` / `square(bool)` / `modal(bool)` | — | `None` / `false` / `false` / `true` | Popover appearance. |
| `overflow_behavior(OverflowBehavior)` | — | `Flip` | Flip vs reposition. |
| `show_toolbar(bool)` | `bool` | `true` | Stored; no toolbar rendered. |
| `am_pm(bool)` | `bool` | `false` | 12-hour clock + markers. |
| `auto_close(bool)` | `bool` | `false` | Close after the value is set. |
| `closing_delay(u32)` | `u32` | `200` | Milliseconds before closing. |
| `minute_selection_step(u8)` | `u8` | `1` | Dial minute granularity; clamped to ≥ 1. |
| `open_to(OpenTo)` | `OpenTo` | `Hours` | Initial dial. |
| `time_edit_mode(TimeEditMode)` | `TimeEditMode` | `Normal` | Which parts are editable. |
| `time_format(&str)` | `&str` | `None` | Display template. |
| `picker_actions(f)` | `FnMut(&mut Ui) -> Option<TimePickerAction>` | `None` | Custom footer row. |
| `on_picker_opened` / `on_picker_closed` / `on_click` | `FnMut()` | `None` | Callbacks. |
| `classes(iter)` / `flavor(StyleFlavor)` | — | — | Utilities. |
| `show(ui, state)` | `(&mut Ui, &mut TimePickerState)` | — | Returns `TimePickerResponse`. |

### `TimePickerState`

| Field / method | Description |
|----------------|-------------|
| `value: Option<SimpleTime>` | Committed value. |
| `text: String` | Trigger text (kept in sync by `commit` / `clear` / `with_value`). |
| `popover: PopoverState` | Popover state. |
| `view: TimePickerView` | Dial currently shown. |
| `pending_hour: Option<u8>` | Hour chosen in the hour step, awaiting a minute. |
| `new(id_salt)` / `with_value(time, text)` | Constructors. |
| `open(OpenTo)` / `close()` / `toggle(OpenTo)` | Popover control. |
| `commit(time, text)` / `clear()` | Value mutation. |
| `effective_hour()` | `pending_hour` → value's hour → `0`. |

### `TimePickerResponse`

| Field | Type | Description |
|-------|------|-------------|
| `trigger` | `egui::Response` | Trigger response. |
| `value` | `Option<SimpleTime>` | Committed value after this frame. |
| `text` | `String` | Trigger text after this frame. |
| `action` | `TimePickerAction` | What happened this frame. |

### `TimePickerAction`

| Variant | Description |
|---------|-------------|
| `None` | No interaction. |
| `Opened` / `Closed` | Popover opened / closed. |
| `TimeSelected(SimpleTime)` | A time was committed. Mirrors `TimeChanged`. |
| `Cleared` | The value was cleared. |

Helpers: `changed_value()`, `toggled_open()`, `name()`.

### `TimePickerView`

| Variant | Description |
|---------|-------------|
| `Hours` (default) | Hour ring. |
| `Minutes` | Minute ring. |

Helpers: `name()`, `toggle()`, `is_hours()`.

### `TimeEditMode` (`mud_egui_core::enums`)

| Variant | Discriminant | Behaviour |
|---------|-------------|-----------|
| `Normal` (default) | `0` | Hours then minutes. |
| `OnlyMinutes` | `1` | Opens on the minute ring; hour comes from the value. |
| `OnlyHours` | `2` | Commits on the hour click; minute comes from the value. |

Helpers: `ALL`, `name()`, `commits_after_hour()`, `opens_on_minutes()`,
`Display`.

### `SimpleTime`

| Member | Description |
|--------|-------------|
| `new(h, m, s)` / `hm(h, m)` / `midnight()` | Clamped constructors. |
| `is_am()` / `is_pm()` / `hour_12()` | 12-hour helpers. |
| `total_minutes()` / `from_total_minutes(n)` | Minutes since midnight. |
| `add_minutes(delta)` | Wrap-around shift. |
| `with_hours` / `with_minutes` / `toggle_am_pm` | Field replacement. |
| `format_hhmm()` / `format_default(am_pm)` / `format(template)` | Display. |
| `Display` | `HH:MM`. |

### Free helpers

| Function | Signature | Description |
|----------|-----------|-------------|
| `snap_minute` | `(u8, u8) -> u8` | Nearest multiple of the step, wrapping. |
| `clock_angle` | `(Pos2, Pos2) -> f32` | Radians, `0` = 12 o'clock, clockwise. |
| `hour_at` | `(Pos2, Pos2, bool) -> Option<u8>` | Click → hour (12h / 24h). |
| `minute_at` | `(Pos2, Pos2, u8) -> Option<u8>` | Click → minute, snapped. |
| `numeral_pos` | `(Pos2, f32, u8) -> Pos2` | Position of clock index `i`. |
| `view_for_open` | `(OpenTo) -> TimePickerView` | Initial dial. |

### Constants

| Name | Value |
|------|-------|
| `DIAL_SIZE` | `240.0` |
| `OUTER_RADIUS` | `105.6` |
| `INNER_RADIUS` | `72.0` |
| `AM_PM_ROW_HEIGHT` | `32.0` |
| `DEFAULT_CLOSING_DELAY_MS` | `200` |

---

## Examples

### Basic 24-hour picker

```rust
let mut state = TimePickerState::new("start");
MudTimePicker::new().label("Start time").show(ui, &mut state);
```

### 12-hour with AM/PM

```rust
MudTimePicker::new()
.label("Alarm")
.am_pm(true)
.show(ui, &mut state);
```

### 15-minute granularity, auto-close

```rust
MudTimePicker::new()
.label("Slot")
.minute_selection_step(15)
.auto_close(true)
.closing_delay(300)
.show(ui, &mut state);
```

### Open on the minute ring

```rust
MudTimePicker::new()
.open_to(OpenTo::Minutes)
.show(ui, &mut state);
```

### Hours only (e.g. a start-hour field)

```rust
MudTimePicker::new()
.time_edit_mode(TimeEditMode::OnlyHours)
.time_format("HH时")
.show(ui, &mut state);
```

### Minutes only (e.g. a minute offset)

```rust
MudTimePicker::new()
.time_edit_mode(TimeEditMode::OnlyMinutes)
.open_to(OpenTo::Minutes)
.show(ui, &mut state);
```

### Custom format

```rust
MudTimePicker::new()
.am_pm(true)
.time_format("h:mm tt")     // → "6:32 PM"
.show(ui, &mut state);

MudTimePicker::new()
.time_format("HH:mm:ss")    // → "18:32:07"
.show(ui, &mut state);
```

### Clearable

```rust
MudTimePicker::new().clearable(true).show(ui, &mut state);
```

### Custom footer actions

```rust
MudTimePicker::new()
.picker_actions(|ui| {
ui.horizontal(|ui| {
if MudButton::new("Now").size(Size::Small).show(ui).clicked() {
return Some(TimePickerAction::TimeSelected(SimpleTime::hm(9, 0)));
}
None
}).inner
})
.show(ui, &mut state);
```

### Read the outcome

```rust
let resp = MudTimePicker::new().show(ui, &mut state);
match resp.action {
TimePickerAction::TimeSelected(t) => log::info!("picked {t}"),
TimePickerAction::Cleared => log::info!("cleared"),
TimePickerAction::Opened => log::info!("opened"),
TimePickerAction::Closed => log::info!("closed"),
TimePickerAction::None => {}
}
```

### Programmatic selection

```rust
state.commit(SimpleTime::hm(18, 30), "18:30");
state.clear();
```

---

## Clock views and commit flow

### 24-hour dial (`am_pm(false)`)

```
         12
    23        1
 22              2
11                3
 10              4
     9        5
          8
```

The outer ring is `1..=12`; the inner ring is `13..=23` and `00` at the 12
o'clock slot. Clicking near the centre is ignored.

### 12-hour dial (`am_pm(true)`)

```
          12
     11        1
   10            2
   9             3
      8        4
           5
   [ AM ]  [ PM ]
```

A single ring, plus two `MudText` markers. Clicking the inactive marker
flips the hour across noon (`09:xx ↔ 21:xx`).

### Minute dial

Labels at `0, 5, …, 55`. `minute_selection_step(n)` snaps every click to the
nearest multiple of `n` (wrapping, so `59` with a step of `15` becomes `0`).

### Commit flow

| `TimeEditMode` | Hour click | Minute click |
|----------------|------------|--------------|
| `Normal` | Store `pending_hour`, switch to the minute ring | Commit `pending_hour : minute` |
| `OnlyHours` | Commit `hour : current_minute` | never shown |
| `OnlyMinutes` | never shown | Commit `current_hour : minute` |

`AutoClose` + `ClosingDelay` are honoured: a committed time sets a deadline
in `egui::Memory`, the popover stays open until it elapses, then closes.

---

## Formatting and arithmetic

`SimpleTime::format(template)` evaluates a MudBlazor-style `TimeFormat`:

| Token | Output |
|-------|--------|
| `HH` / `H` | 24-hour, padded / unpadded |
| `hh` / `h` | 12-hour, padded / unpadded |
| `mm` / `m` | minutes, padded / unpadded |
| `ss` / `s` | seconds, padded / unpadded |
| `tt` | `AM` / `PM` |

Unknown characters are copied verbatim, so `"HH:mm"`, `"h:mm tt"` and
`"HH时mm分"` all work. An empty template falls back to the default
(`hh:mm tt` when `am_pm`, else `HH:mm`) — MudBlazor's documented defaults
[reference:8].

All arithmetic (`total_minutes`, `from_total_minutes`, `add_minutes`,
`with_hours`, `toggle_am_pm`) is pure and unit-tested.

**No "now" concept** — without a system clock, "now" is caller-owned. Seed
`TimePickerState` with `with_value(..)` if you want a starting time.

---

## Performance and memory choices

| Choice | Effect |
|--------|--------|
| `SimpleTime` is `Copy` (3 bytes) | Zero allocation for time values. |
| Pure dial geometry | `clock_angle` / `hour_at` / `minute_at` are arithmetic; testable without a `Context`. |
| Theme fetched once per frame | One `MudTheme::fetch`; eight palette colours resolved once into `PickerPalette`. |
| Numerals through `MudText` in child `Ui`s | Rule 3 compliant; no `painter.text`. |
| Ring outlines and hand through `ui.painter()` | Geometry only — no user-facing text bypasses `MudText`. |
| `animate_value_with_time` only when not dragging | `PointerMoving` suppresses it, matching the parameter's purpose[reference:9]. |
| `AutoClose` deadline in `egui::Memory` | No caller-side storage; no busy-looping. |
| `pending_hour` in state | No shadow copy on the builder between frames. |
| Deferred clicks | `state` is mutated only after `popover.show` releases its borrow. |

---

## Testing

`cargo test -p mud_egui_components mud_time_picker` runs the module's tests:

| Area | Tests |
|------|-------|
| `SimpleTime` | clamping, `hm`, AM/PM split, `hour_12`, total-minute round-trip, `add_minutes` wrap, `with_*`, `toggle_am_pm`, all format tokens, defaults |
| `TimePickerView` | default, toggle, involution, names |
| `TimeEditMode` | default, discriminants, `ALL` uniqueness, `Display`, predicates |
| `TimePickerAction` | default, `changed_value`, `toggled_open`, names |
| Dial geometry | `clock_angle` at 12/3/6/9, `clock_index`, both hour maps, `hour_at` on both rings and in both modes, dead zone, `minute_at` with and without a step |
| `snap_minute` | step 0/1, rounding, wrap |
| State | defaults, `with_value`, `open` resets view and pending hour, `commit` / `clear`, `effective_hour`, `view_for_open` |
| Builders | defaults match MudBlazor, every field set, step clamp, effective format |

Rendering needs a live `egui::Context`; the pure layer is the source of
truth.

```bash
cargo test -p mud_egui_components mud_time_picker
cargo test -p mud_egui_components simple_time
cargo test -p mud_egui_components time_picker_state
cargo test -p mud_egui_core time_edit_mode
```

---

## Differences from MudBlazor

- **No `chrono`.** `SimpleTime` is a small pure time-of-day. Callers needing
  timezone handling need their own bridge.
- **`TimeFormat` is a template string, not a `Func`.** The token set
  (`HH`/`H`/`hh`/`h`/`mm`/`m`/`ss`/`s`/`tt`) covers MudBlazor's documented
  examples; an unknown character is copied through.
- **`ClosingDelay` is honoured via a `Memory` deadline.** MudBlazor keeps the
  DOM element around for the delay; egui closes the popover when the
  deadline elapses.
- **`PointerMoving` is derived from the dial's `Response`**, not a separate
  parameter, and is used to suppress the hand animation.
- **The hand animates with `animate_value_with_time`**, not CSS. The
  duration is a constant; `PointerMoving` snaps it.
- **`Inline` / `Static` picker variants are stored but not rendered.** Only
  `Dialog` shows the clock.
- **`Editable` typing is deferred.** The field accepts typed text but does
  not parse it back into `SimpleTime`.
- **`Mask` / `FixDay`-style constraints are deferred.**
- **`Text` is caller-owned via `TimePickerState::text`.**
- **`Style` / `Tag` / `UserAttributes` not ported.** No CSS strings or HTML
  tags.
- **`TimePickerAction`, `TimePickerView` and `TimeEditMode` live in their
  own files**, per Rule 2.