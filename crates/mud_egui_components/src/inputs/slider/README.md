# MudSlider for `mud_egui`

A single-value slider for [egui](https://github.com/emilk/egui), ported from
MudBlazor's [`MudSlider<T>`](https://mudblazor.com/components/slider)
(MudBlazor 8.13.0).

```rust
let mut volume = 50.0_f32;

ui.add(
    MudSlider::new(&mut volume, 0.0, 100.0)
        .color(Color::Primary)
        .show_value(true)
        .value_format("{:.0}%"),
);
```

The label, tick labels and value bubble all render through **`MudText`**, so
the active `MudTheme` typography and colours propagate to every visible string
in the component. The rail, fill, ticks, halo, thumb and bubble background are
painted directly with `egui::Painter`.

---

## Contents

1. [Design principle: API parity, not API invention](#design-principle-api-parity-not-api-invention)
2. [Architecture](#architecture)
3. [API reference](#api-reference)
4. [Examples](#examples)
5. [Layout and rendering](#layout-and-rendering)
6. [Performance and memory choices](#performance--memory-choices)
7. [Testing](#testing)
8. [Differences from MudBlazor](#differences-from-mudblazor)

---

## Design principle: API parity, not API invention

The MudBlazor 8.13.0 `MudSlider<T>` parameter surface was verified against the
published API reference before this port. Fourteen parameters are ported; five
are deliberately omitted; one Rust-only extension is added.

### Ported parameters

| MudBlazor | Type | Default | Rust mapping |
|---|---|---|---|
| `Value` | `T` | — | bound `&mut f32` |
| `ValueChanged` | `EventCallback<T>` | — | `Response::changed` on the returned `Response` |
| `Min` | `double` | `0.0` | `MudSlider::new` `min` |
| `Max` | `double` | `100.0` | `MudSlider::new` `max` |
| `Step` | `double` | `1.0` | `step(f32)` |
| `Vertical` | `bool` | `false` | `vertical(bool)` |
| `Disabled` | `bool` | `false` | `disabled(bool)` |
| `Color` | `Color` | `Primary` | `color(Color)` |
| `Size` | `Size` | `Small` | `size(Size)` |
| `Variant` | `Variant` | `Text` | `variant(Variant)` |
| `TickMarks` | `bool` | `false` | `ticks(bool)` |
| `MarkerLabels` | `string[]?` | `null` | `tick_labels(&[&str])` |
| `ValueLabel` | `bool` | `false` | `show_value(bool)` |
| `ValueLabelFormat` | `Func<double, string>` | `null` | `value_format(&str)` — template string form |
| `ChildContent` (label) | `RenderFragment` | — | `label(&str)` — string form |

`ValueLabelFormat` is a `Func<double, string>` in Blazor. A Rust port cannot
take a generic `Fn(f64) -> String` on a `#[must_use]` builder without
introducing a trait object lifetime, so it accepts a **template string**
instead: `"{}"`, `"{:.2}"`, `"${:.2}"`, `"{} px"`. The common cases all work;
a dynamic format (e.g. `"{} / {}"` filled with two values) is not supported and
should be expressed by the caller after reading `*value`.

### Deliberately not ported

| MudBlazor | Reason |
|---|---|
| `Range` | Two-thumb range mode is a separate widget shape; the interaction model (which thumb is being dragged), the layout, and the display all differ from the single-thumb slider. Adding it would roughly double the surface, and it deserves its own component (`MudRangeSlider`) rather than a boolean mode. |
| `RangeValue` / `RangeValueChanged` | Paired with `Range`; same reason. |
| `Immediate` | MudBlazor defaults to firing `ValueChanged` on release; in an immediate-mode UI every frame is a “release”, so `Response::changed` fires every frame the value moves. This port therefore behaves as `Immediate = true` always. |
| `Culture` | Rust's `format!` is not culture-aware. Use a `value_format(..)` template, or format the value yourself after reading `*value`. |
| `LabelAttributes` / `InputAttributes` | No HTML attributes in egui. |
| `Class` / `Style` / `Tag` / `UserAttributes` | No CSS strings, no HTML tag. `Tag` is not exposed because there is no `Any`-typed slot on the builder; callers can wrap the slider in their own struct if they need one. |

### Extension

| Extension | Type | Default | Purpose |
|---|---|---|---|
| `length(f32)` | `f32` | `None` | Explicit length along the slider's axis, in points. MudBlazor relies on CSS width; `length` is the typed equivalent and the only reason a caller would reach for `Style` on this component. |

### Rule 3 compliance

Every visible string — the label, tick labels, and value-bubble text — goes
through `MudText`. No user-facing text uses `painter.text` or `painter.galley`;
only the rail, fill, ticks, halo, thumb and bubble **background** are painted
directly. This means the active `MudTheme` typography and colours apply to
every label, exactly as they do for `MudRating`, `MudCheckBox` and the other
input components.

The label above the slider is drawn with `MudText::body1`. The tick labels and
the value bubble use `MudText` with `Typo::Body2` and a `font_size` override of
12 pt, matching MudBlazor's `.mud-slider-marker-label` and value-label
typography.

---

## Architecture

```
inputs/slider/
└── mud_slider.rs
    ├── constants            MIN_TICKS 2, MAX_TICKS 30, HIT_THICKNESS 28,
    │                        LABEL_ROW_EXTRA 12, TICK_LABEL_OFFSET 18,
    │                        TICK_OVERHANG 2, DEFAULT_VERTICAL_LENGTH 160,
    │                        FALLBACK_LENGTH 200, TRACK_ALPHA 0.38,
    │                        DISABLED_FACTOR 0.5, HALO_HOVER_EXTRA 8,
    │                        HALO_PRESS_EXTRA 6, HALO_ALPHA 0.16,
    │                        ANIM_TIME 0.12, LABEL_FONT_SIZE 12,
    │                        BUBBLE_* geometry, CONTRAST_LUMINANCE_THRESHOLD 0.45
    ├── SizeMetrics          rail / thumb per Size
    ├── Axis                 start..end line, point_at / fraction_of
    ├── RenderOpts           resolved inputs for render_slider
    ├── render_slider        allocation, interaction, painting
    ├── draw_value_bubble    background + pointer + MudText for text
    ├── pure helpers         size_metrics, value_to_fraction, fraction_to_value,
    │                        snap_to_step, step_decimals, format_value,
    │                        format_with_template, tick_count, tick_fraction,
    │                        is_ambient_color, contrast_text, usable_or,
    │                        label_font, keyboard_adjust, resolve_color
    └── impl Widget          ui.add(MudSlider::new(..))
```

**Pure layer.** Every geometric and formatting decision is a free function
over `f32`, `Rect` and `Pos2`. `Axis` wraps the line the thumb travels along;
it works for both orientations so the interaction math is not duplicated.

**Component layer.** `show` resolves theme, colour and metrics once, then
delegates to `render_slider` for allocation and painting. `render_slider` is a
free function so it can borrow `value: &mut f32` directly and does not need
`&mut self`.

**Text layer.** All three text sites — the top label, tick labels, and value
bubble — go through `MudText`. Tick labels and bubble text are placed in a
child `Ui` sized to the measured text, so a `MudText::inline(true)` fills its
rect exactly and the text appears centred on the target point.

---

## API reference

| Method | Type | Default | Mirrors | Description |
|---|---|---|---|---|
| `new(value, min, max)` | `&'a mut f32, f32, f32` | — | `Value` + `Min` + `Max` | Constructs a slider. |
| `step(s)` | `f32` | `1.0` | `Step` | Snap increment. `<= 0` is continuous. |
| `variant(v)` | `Variant` | `Text` | `Variant` | `Filled` fills the rail up to the thumb. |
| `color(c)` | `Color` | `Primary` | `Color` | Rail / thumb / tick / bubble colour. |
| `size(s)` | `Size` | `Small` | `Size` | Rail thickness + thumb diameter. |
| `label(s)` | `impl Into<&'a str>` | `None` | `ChildContent` (string) | Text label above (horizontal) or beside (vertical). |
| `disabled(b)` | `bool` | `false` | `Disabled` | Dims to 50 % and disables interaction. |
| `vertical(b)` | `bool` | `false` | `Vertical` | Runs bottom-to-top instead of left-to-right. |
| `show_value(b)` | `bool` | `false` | `ValueLabel` | Shows the value bubble on hover / press / focus. |
| `value_format(fmt)` | `impl Into<&'a str>` | `None` | `ValueLabelFormat` | Template for the bubble value. |
| `ticks(b)` | `bool` | `false` | `TickMarks` | Draws ticks on the rail. |
| `tick_labels(labels)` | `&'a [&'a str]` | `None` | `MarkerLabels` | Per-tick labels (horizontal only). |
| `length(px)` | `f32` | `None` | *(extension)* | Explicit length along the axis. |
| `show(ui)` | `&mut Ui` | — | — | Returns the slider's `Response`. |
| `ui.add(..)` | — | — | — | Same as `show`. |

`Response::changed()` is `true` on the frame the bound value changed.

### `Size` mapping

| `Size` | Rail thickness | Thumb diameter |
|---|---|---|
| `ExtraSmall` / `Small` | 2 pt | 16 pt |
| `Medium` | 4 pt | 20 pt |
| `Large` / `ExtraLarge` | 6 pt | 24 pt |

`ExtraSmall` folds into `Small` and `ExtraLarge` into `Large`, matching
MudBlazor's three-slider-size model.

### Constants

| Name | Value |
|---|---|
| `HIT_THICKNESS` | `28.0` |
| `LABEL_ROW_EXTRA` | `12.0` |
| `TICK_LABEL_OFFSET` | `18.0` |
| `TICK_OVERHANG` | `2.0` |
| `DEFAULT_VERTICAL_LENGTH` | `160.0` |
| `FALLBACK_LENGTH` | `200.0` |
| `TRACK_ALPHA` | `0.38` |
| `DISABLED_FACTOR` | `0.5` |
| `HALO_HOVER_EXTRA` / `HALO_PRESS_EXTRA` / `HALO_ALPHA` | `8` / `6` / `0.16` |
| `ANIM_TIME` | `0.12` s |
| `LABEL_FONT_SIZE` | `12.0` |
| `MIN_TICKS` / `MAX_TICKS` | `2` / `30` |
| `CONTRAST_LUMINANCE_THRESHOLD` | `0.45` |

---

## Examples

### Basic

```rust
let mut volume = 50.0_f32;
ui.add(MudSlider::new(&mut volume, 0.0, 100.0));
```

### Filled variant

```rust
ui.add(
    MudSlider::new(&mut volume, 0.0, 100.0)
        .variant(Variant::Filled)
        .color(Color::Primary),
);
```

### Larger size

```rust
ui.add(MudSlider::new(&mut volume, 0.0, 100.0).size(Size::Large));
```

### With a top label

```rust
ui.add(
    MudSlider::new(&mut volume, 0.0, 100.0)
        .label("Volume")
        .show_value(true)
        .value_format("{:.0}%"),
);
```

### With ticks and labels

```rust
const LABELS: &[&str] = &["Poor", "Fair", "Good", "Great", "Excellent"];
let mut rating = 3.0_f32;

ui.add(
    MudSlider::new(&mut rating, 1.0, 5.0)
        .step(1.0)
        .ticks(true)
        .tick_labels(LABELS),
);
```

### Vertical with an explicit length

```rust
let mut level = 10.0_f32;

ui.add(
    MudSlider::new(&mut level, 0.0, 20.0)
        .vertical(true)
        .length(200.0)
        .label("Level")
        .show_value(true),
);
```

### Disabled

```rust
ui.add(
    MudSlider::new(&mut volume, 0.0, 100.0)
        .disabled(true)
        .label("Locked"),
);
```

### Continuous (no step)

```rust
ui.add(MudSlider::new(&mut volume, 0.0, 1.0).step(0.0));
```

### Custom template

```rust
ui.add(
    MudSlider::new(&mut price, 0.0, 100.0)
        .show_value(true)
        .value_format("${:.2}"),
);
```

### Reading the changed flag

```rust
let response = MudSlider::new(&mut volume, 0.0, 100.0).show(ui);
if response.changed() {
    log::info!("volume = {volume}");
}
```

### Keyboard

With the slider focused:

| Key | Effect |
|---|---|
| `→` / `↑` | `+= step` |
| `←` / `↓` | `-= step` |
| `PageUp` | `+= 10 × step` |
| `PageDown` | `-= 10 × step` |
| `Home` | Set to `min` |
| `End` | Set to `max` |

---

## Layout and rendering

### Horizontal slider

```
[ label           ]   ← MudText::body1 above the strip
[ ●─────────────── ]   ← rail (translucent) + fill up to thumb (Filled)
   Poor  Fair ...      ← tick labels, centred under each tick
```

- **Height:** 28 pt (`HIT_THICKNESS`), plus 12 pt for the tick-label row when
  labels are shown.
- **Width:** `ui.available_width()`, clamped to `length` when set.
- **Rail:** 2 / 4 / 6 pt tall (per `Size`), translucent at `0.38` α.
- **Fill:** up to the thumb, only for `Variant::Filled`.
- **Halo:** `HALO_HOVER_EXTRA` while hovered / pressed, `HALO_PRESS_EXTRA`
  additional while pressed.
- **Thumb:** circular, `SizeMetrics.thumb` diameter.

### Vertical slider

```
[    ]   [ label    ]
[ ●  ]
[    ]
[    ]
```

The track runs bottom-to-top; `min` sits at the bottom. The label is placed to
the left (horizontal container), same as `MudSlider`'s Blazor layout for
`Vertical`.

### Value bubble

Painted only while the slider is hovered, pressed, or keyboard-focused. It sits
`HALO_HOVER_EXTRA + BUBBLE_GAP + BUBBLE_ARROW_HEIGHT` above the thumb centre
(to the right of it when vertical) with a 5 pt pointer aimed at the thumb.

The bubble **background and pointer** are painted on the slider's own layer
via `ui.painter()`; the bubble **text** goes through `MudText`. This means the
bubble is guaranteed to be on top of the slider itself, but not guaranteed to
escape a clip imposed by an ancestor (e.g. a small `ScrollArea`). See the
[`Rule 3 compliance`](#rule-3-compliance) section above.

### Degenerate inputs

| Input | Behaviour |
|---|---|
| `max <= min` | The slider is inert; the thumb sits at `min`, no interaction. |
| Non-finite `value` | Clamped to `min`; the caller's value is untouched unless the user drags. |
| Non-finite `length` | Falls back to the available width or `DEFAULT_VERTICAL_LENGTH`. |
| `step <= 0` | Continuous drag; no snapping. |
| Tick count outside `2..=30` | Ticks are silently skipped rather than drawn illegibly dense. |

---

## Performance and memory choices

| Choice | Effect |
|---|---|
| One `Painter` clone per `render_slider` | Releases the immutable `ui` borrow so child `Ui`s (for `MudText`) can open later. Cheap: clones an `Arc` and copies two scalar fields. |
| Theme fetched once per frame | One `MudTheme::fetch` and up to one `Color::resolve`. |
| One `allocate_exact_size` for the strip | The whole control is one rect and one `Response`. |
| Pure helpers on `Copy` values | Rail thickness, thumb diameter, value→fraction, tick count/fraction and template formatting are arithmetic; unit-testable without a `Context`. |
| No `Vec`, no `String` on the hot path | The label and tick labels borrow from the caller; the bubble string is built once per active frame. |
| `MudText` in a child `Ui` per label | One `Ui` per label/tick, but each is scoped to its measured rect and discarded after. |
| Halo and press fades use `animate_bool_with_time` | Idle at zero cost when nothing changes. |
| `painter.layout_no_wrap` for measurement only | Used to size the child `Ui` rect so `MudText::inline(true)` fills it exactly; never used to paint text. |

---

## Testing

`cargo test -p mud_egui_components mud_slider` runs 30+ pure-function tests:

| Area | Tests |
|---|---|
| `size_metrics` | every `Size`; edge sizes fold inwards; largest thumb fits the strip |
| `value_to_fraction` | maps, clamps, degenerate range |
| `fraction_to_value` / `snap_to_step` | snap to step, clamp at max, non-positive step is continuous, offset from `min` |
| `Axis` | horizontal / vertical mapping, zero-length safety |
| `format_with_template` | bare `{}`, `{:.N}`, literal text preserved, unmatched brace, non-numeric precision, only first placeholder |
| `step_decimals` / `format_value` | step-derived precision; f32 noise hidden; templates applied |
| `tick_count` | invalid step, `max <= min`, degenerate, dense, normal, inclusive boundaries |
| `tick_fraction` | even spacing, drops ticks past `max`, float-error tolerance |
| `is_ambient_color` / `contrast_text` | every ambient variant; dark-on-light and white-on-brand |
| `usable_or` | valid, zero, negative, non-finite |

No test creates an `egui::Context`. `MudSlider::show`, `render_slider`,
`draw_value_bubble`, `keyboard_adjust`, `label_font` and `resolve_color` need a
live `Ui` and are left to integration tests.

Suggested integration tests (`egui_kittest` or a headless `Context::run`):

- Dragging the thumb to the middle of a `0..=100` slider sets the value to
  `50`.
- Every visible string (label, tick labels, bubble text) is rendered by
  `MudText` — assert by mutating the theme's typography and confirming the
  rendered font changes.
- The bubble is shown only while the slider is hovered / pressed / focused.
- `Home` / `End` jump to the ends when the slider is focused.
- A disabled slider ignores pointer and keyboard.

```bash
cargo test -p mud_egui_components mud_slider
```

---

## Differences from MudBlazor

- **Every visible string goes through `MudText`.** The pre-refactor port drew
  tick labels and the value bubble with `painter.text` / `painter.galley` and
  `TextStyle::Body.resolve(ui.style())`, bypassing the active `MudTheme`. They
  now render through `MudText` and inherit the theme's `Typo::Body2`
  typography at a 12 pt override, matching MudBlazor's caption scale.
- **The value bubble lives on the slider's own layer, not the tooltip layer.**
  egui offers no way to construct a `Ui` on an arbitrary layer, and `MudText`
  needs a `Ui`; a bubble that satisfies Rule 3 therefore cannot live on the
  tooltip layer. The bubble is drawn last on the current layer, so it is on
  top of the slider itself; a tight ancestor clip (a small `ScrollArea`) may
  still clip it, where the previous implementation would have escaped. This is
  a deliberate Rule 3 trade-off, documented here rather than silently dropped.
- **`ValueLabelFormat` is a template string, not a `Func<double, string>`.**
  The common `{}` / `{:.N}` placeholder forms are supported by
  `format_with_template`; anything else should be formatted by the caller
  after reading `*value`. The template is applied by `format_value` after
  rounding to the step's precision, so f32 noise never shows.
- **`Immediate` is not a parameter.** This port behaves as `Immediate = true`
  always, because in immediate mode `Response::changed` fires every frame the
  value moves. `Immediate = false` (fire on release) has no meaning here.
- **`Range` / `RangeValue` / `RangeValueChanged` are not ported.** Range
  sliders need a separate component (`MudRangeSlider`) with a different
  interaction model.
- **`Culture` is not ported.** Rust's `format!` is not culture-aware; use
  `value_format(..)` or format the value yourself.
- **`length(f32)` is an extension.** MudBlazor relies on CSS width; `length`
  is the typed equivalent. Vertical sliders default to
  `DEFAULT_VERTICAL_LENGTH` (160 pt) when `length` is unset, because there is
  no “available height” to stretch to.
- **`WidgetInfo::slider` is called with the correct two-argument form.**
  egui’s `slider(value, label)` constructor does not take an `enabled` flag;
  `enabled` is set on the returned `WidgetInfo` afterwards, so a disabled
  slider is reported as disabled to assistive technology.