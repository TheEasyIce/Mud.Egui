# MudSwitch for `mud_egui`

A two-state toggle control for [egui](https://github.com/emilk/egui), ported
from MudBlazor's [`MudSwitch<T>`](https://mudblazor.com/components/switch)
(MudBlazor 8.13.0).

```rust
let mut notifications = true;

ui.add(
    MudSwitch::new(&mut notifications)
        .label("Email notifications")
        .label_placement(Placement::End)
        .color(Color::Success),
);

if !notifications {
    mute();
}
```

Every string renders through **`MudText`**; the thumb glyph renders through
**`MudIcon`**. The track and thumb themselves are painted directly.

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

The MudBlazor 8.13.0 parameter surface was verified against the DNDocs pages
for `MudSwitch<T>` and its parent `MudBooleanInput<T>` before implementation.

### Switch-specific parameters

| MudBlazor | Type | Default | Rust mapping |
|---|---|---|---|
| `ThumbIcon` | `string?` | `null` | `thumb_icon(s)` |
| `ThumbIconColor` | `Color` | `Default` | `thumb_icon_color(Color)` — ignored when `thumb_icon` is unset |
| `UncheckedColor` | `Color` | `Default` | `unchecked_color(Color)` |

### Inherited from `MudBooleanInput<T>`

| MudBlazor | Type | Default | Rust mapping |
|---|---|---|---|
| `Value` | `T` | — | caller-owned `&'a mut bool` |
| `ValueChanged` | `EventCallback<T>` | — | `SwitchResponse::changed` |
| `Label` | `string?` | `null` | `label(s)` |
| `LabelPlacement` | `Placement` | `End` | `label_placement(Placement)` |
| `Color` | `Color` | `Default` | `color(Color)` |
| `Size` | `Size` | `Medium` | `size(Size)` |
| `Disabled` | `bool` | `false` | `disabled(bool)` |
| `ReadOnly` | `bool` | `false` | `read_only(bool)` |
| `Ripple` | `bool` | `true` | `ripple(bool)` |
| `Class` | `string?` | `null` | `classes(iter)` — stored, not resolved |
| `Tag` | `object?` | `null` | `tag(t)` / `tag_ref::<U>()` |

### Deliberately not ported

| MudBlazor | Reason |
|---|---|
| Generic `T` + `BoolConverter<T>` | MudBlazor converts arbitrary `T` (typically `bool`) through a pluggable `BoolConverter<T>`. egui has no converter pipeline; the port binds `&'a mut bool`, which is the overwhelmingly common case. Porting the generic form would be inventing a trait that MudBlazor does not require. |
| `ChildContent` | MudBlazor's `ChildContent` is a `RenderFragment` for a custom label; `label(s)` is the string form. A custom-content closure would require a lifetime-bearing closure on the builder and is not needed for the switch's documented use. |
| Validation members (`Required`, `Error`, `ErrorText`, `HelperText`, `ConversionError`, `Strict`, `Immediate`, `DebounceInterval`) | Inherited from `MudFormComponent` / `MudBaseInput`; they have no egui equivalent without a form system. |
| `Style` | egui has no inline CSS strings. |
| `UserAttributes` | There is no HTML tag to splat attributes onto. |
| `Class` resolution | Stored on `base` for parity only. The track and thumb colours come from `color` / `unchecked_color` / `thumb_icon_color`; a utility-class frame would composite a second fill. |

No enums are introduced: `Placement`, `Color`, and `Size` already exist in
`mud_egui_core::enums`.

### `ThumbIconColor` is conditional

`ThumbIconColor` **only applies when `ThumbIcon` is set**. That is the
documented MudBlazor remark on both parameters, and the port enforces it
structurally: `thumb_icon_color` is simply never read unless
`thumb_icon.is_some()`.

---

## Architecture

```
inputs/switch/
└── mud_switch.rs
    ├── constants        SWITCH_ANIM_SECONDS 0.15, RIPPLE_DURATION 0.35,
    │                    RIPPLE_MAX_RADIUS_FACTOR 1.6, RIPPLE_ALPHA 60,
    │                    UNCHECKED_TRACK_ALPHA 120, DISABLED_OPACITY 0.38,
    │                    LABEL_GAP 8, THUMB_OVERHANG 2,
    │                    THUMB_ICON_FRACTION 0.6, THUMB_STROKE 1
    ├── SwitchMetrics    track_w / track_h / thumb per Size
    ├── pure helpers     is_vertical, label_leads, row_size, track_rect,
    │                    label_rect, thumb_center, next_value_from_keys,
    │                    measure_label, is_ambient, lerp_color, with_alpha
    ├── MudSwitch<'a>    builder + show(ui) -> SwitchResponse
    ├── SwitchResponse   response + changed
    └── ripple           start_ripple, paint_ripple (memory-backed)
```

**Pure layer.** Size metrics, row/track/label geometry, the thumb's rest
positions, the keyboard mapping, colour blending and alpha helpers are free
functions or small `Copy` structs. None touch `egui::Context`; the whole
non-painting surface is unit-testable.

**Binding layer.** The caller owns the `&'a mut bool`; the switch writes it when
a click or key toggles it, and reports `SwitchResponse::changed` for the frame
it moved. This is the same split `MudCheckBox`, `MudRadio` and `MudCollapse`
use.

**Component layer.** `show` measures the label, allocates the row, applies a
click or key press, then paints the track, thumb, thumb icon, label and ripple.

---

## API reference

### `MudSwitch<'a>`

| Method | Type | Default | Mirrors | Description |
|---|---|---|---|---|
| `new(value)` | `&'a mut bool` | — | `Value` | Binds the switch. |
| `label(s)` | `impl Into<String>` | `None` | `Label` | Label text. |
| `label_placement(p)` | `Placement` | `End` | `LabelPlacement` | `Start` / `End` / `Top` / `Bottom` (plus `Left` / `Right`). |
| `color(c)` | `Color` | `Default` | `Color` | Checked-track accent. |
| `unchecked_color(c)` | `Color` | `Default` | `UncheckedColor` | Unchecked-track colour. |
| `size(s)` | `Size` | `Medium` | `Size` | Track and thumb dimensions. |
| `thumb_icon(s)` | `impl Into<String>` | `None` | `ThumbIcon` | Glyph on the thumb. |
| `thumb_icon_color(c)` | `Color` | `Default` | `ThumbIconColor` | Thumb glyph colour; ignored without `thumb_icon`. |
| `ripple(b)` | `bool` | `true` | `Ripple` | Click ripple. |
| `disabled(b)` | `bool` | `false` | `Disabled` | Dims and disables. |
| `read_only(b)` | `bool` | `false` | `ReadOnly` | Disables without dimming. |
| `classes(iter)` | `iter of AsRef<str>` | none | `Class` | Stored on `ComponentBase`. |
| `tag(t)` / `tag_ref::<U>()` | `T: Any + Send + Sync` | `None` | `Tag` | Arbitrary user data. |
| `show(ui)` | `&mut Ui` | — | — | Returns `SwitchResponse`. |

### `SwitchResponse`

| Field | Type | Description |
|---|---|---|
| `response` | `Response` | The switch row (track + label). Deref target. |
| `changed` | `bool` | `true` on the frame the bound value moved. Mirrors `ValueChanged`. |

`SwitchResponse` implements `Deref<Target = Response>`, so `out.clicked()`,
`out.hovered()` and `out.rect` work directly.

### `Size` mapping

| `Size` | Track (w × h) | Thumb |
|---|---|---|
| `ExtraSmall` / `Small` | 34 × 14 pt | 18 pt |
| `Medium` | 40 × 16 pt | 22 pt |
| `Large` / `ExtraLarge` | 50 × 20 pt | 28 pt |

The thumb is deliberately larger than the track, and overhangs it by
`THUMB_OVERHANG` (2 pt) at each end; that is the switch's defining silhouette.

### Colour resolution

| State | Role | Fallback when `Default` / `Inherit` |
|---|---|---|
| Checked track | `color` | `ui.visuals().selection.bg_fill` |
| Unchecked track | `unchecked_color` | `ui.visuals().text_color()` at `UNCHECKED_TRACK_ALPHA` (120) |
| Thumb icon | `thumb_icon_color` | `ui.visuals().text_color()` |

During the animation the checked and unchecked track colours are linearly
interpolated by `progress`, so toggling fades between them rather than
snapping.

### Keyboard

| Key | Effect |
|---|---|
| `ArrowLeft` / `Delete` | Sets `false` |
| `ArrowRight` / `Enter` (incl. numpad) | Sets `true` |
| `Space` | Toggles |

Priority, when multiple keys are reported pressed in one frame:
`uncheck > check > space`. That is deterministic and matches the order in
which MudBlazor's `HandleKeyDownAsync` evaluates its branches.

### Constants

| Name | Value |
|---|---|
| `SWITCH_ANIM_SECONDS` | `0.15` s |
| `RIPPLE_DURATION` | `0.35` s |
| `RIPPLE_MAX_RADIUS_FACTOR` | `1.6` |
| `RIPPLE_ALPHA` | `60` |
| `UNCHECKED_TRACK_ALPHA` | `120` |
| `DISABLED_OPACITY` | `0.38` |
| `LABEL_GAP` | `8.0` |
| `THUMB_OVERHANG` | `2.0` |
| `THUMB_ICON_FRACTION` | `0.6` |
| `THUMB_STROKE` | `1.0` |

---

## Examples

### Basic

```rust
let mut enabled = true;
ui.add(MudSwitch::new(&mut enabled).label("Enabled"));
```

### Coloured, with a different unchecked colour

```rust
ui.add(
    MudSwitch::new(&mut notifications)
        .label("Notifications")
        .color(Color::Success)
        .unchecked_color(Color::Error),
);
```

`color` is the on-track colour; `unchecked_color` is the off-track colour. Both
default to `Color::Default`, which resolves to the theme's accent (on) and a
muted text colour (off).

### Label before the track

```rust
ui.add(
    MudSwitch::new(&mut enabled)
        .label("Compact mode")
        .label_placement(Placement::Start),
);
```

### Vertical label

```rust
MudSwitch::new(&mut enabled)
    .label("Above")
    .label_placement(Placement::Top)
    .show(ui);

MudSwitch::new(&mut enabled)
    .label("Below")
    .label_placement(Placement::Bottom)
    .show(ui);
```

### Thumb icon

```rust
MudSwitch::new(&mut airplane_mode)
    .thumb_icon(filled::AIRPLANE)
    .thumb_icon_color(Color::Primary)
    .show(ui);
```

`thumb_icon_color` is only read when `thumb_icon` is set.

### Dense row in a toolbar

```rust
MudSwitch::new(&mut self.compact)
    .label("Compact")
    .size(Size::Small)
    .show(ui);
```

### Read-only

```rust
MudSwitch::new(&mut self.locked)
    .label("Locked")
    .read_only(true)
    .show(ui);
```

The switch is not dimmed and its state cannot be toggled by pointer or
keyboard; the caller can still change `*value` directly.

### Disabled

```rust
MudSwitch::new(&mut self.locked)
    .label("Unavailable")
    .disabled(true)
    .show(ui);
```

Dims the track, thumb and label to 38 % opacity and blocks all input.

### Reacting to a toggle

```rust
let out = MudSwitch::new(&mut notifications).label("Notify").show(ui);
if out.changed {
    log::info!("notifications = {notifications}");
}
```

### Keyboard-only form

```rust
// With the switch focused:
//   ArrowLeft / Delete  → off
//   ArrowRight / Enter  → on
//   Space               → toggle
```

### Tag round-trip

```rust
let s = MudSwitch::new(&mut enabled).tag("airplane");
assert_eq!(s.tag_ref::<&str>(), Some(&"airplane"));
```

---

## Layout and rendering

### Horizontal (`Start` / `End`)

```
End:    [ ( )───── ]  Label …
Start:  Label …  [ ( )───── ]
```

- Track: `switch_metrics(size)` pill, radius = half the track height.
- Thumb: circle of `metrics.thumb`, centred on the track's vertical centre and
  overhanging `THUMB_OVERHANG` at the active end.
- Gap between track and label: `LABEL_GAP` (8 pt).
- Row height: `max(thumb, track_h)`.

### Vertical (`Top` / `Bottom`)

```
Top:    [ Label … ]
        [ ( )───── ]
Bottom: [ ( )───── ]
        [ Label … ]
```

The track is horizontally centred under/over the label; the row width is
`max(track_w, label_width)`.

### Animation

```
progress = animate_bool_with_time(id, *value, SWITCH_ANIM_SECONDS)
thumb_center = lerp(off_center, on_center, progress)
track_fill   = lerp(unchecked_color, checked_color, progress)
```

The thumb interpolates linearly between its two rest positions; the track
colour fades between unchecked and checked. Both are driven by the same
`progress`, so they stay in sync.

### Ripple

When `ripple(true)` and a toggle happens, a filled circle expands from the
thumb's centre with radius `thumb/2 × RIPPLE_MAX_RADIUS_FACTOR` over
`RIPPLE_DURATION`, then fades out. Start time lives in `egui::Memory` keyed by
the switch's `Response::id`, so the component stays stateless.

### Degenerate inputs

| Input | Behaviour |
|---|---|
| Empty label | The label column is zero-width; only the track is drawn. |
| `disabled` | Track, thumb and label are gamma-multiplied by 38 %; no interaction, no cursor change beyond `NotAllowed`. |
| `read_only` | Full opacity; pointer clicks and keys are ignored; cursor stays default. |
| `!ui.is_enabled()` (parent disabled) | Treated as `disabled` for interaction. |
| `thumb_icon_color` set without `thumb_icon` | Ignored — no icon is drawn. |

---

## Performance and memory choices

| Choice | Effect |
|---|---|
| Theme fetched once per frame | One `MudTheme::fetch`; the surface colour is read from it once, and the two accent colours fall back to `ui.visuals()` when ambient (so no palette walk in the common `Color::Default` case). |
| One `allocate_exact_size` for the row | The whole switch is one rect and one `Response`. |
| Pure helpers on `Copy` values | Metrics, geometry, keyboard mapping and colour blending are arithmetic; unit-testable without a `Context`. |
| No `Vec`, no `String` beyond the label | The label is measured with `layout_no_wrap` and moved into `MudText`; the thumb glyph is moved into `MudIcon`. |
| `animate_bool_with_time` | Idle at zero cost when the value does not change; the animation requests its own repaints only while in flight. |
| Ripple in `egui::Memory` | One `f64` per in-flight ripple; zero cost when `ripple(false)` or disabled. |
| `Deref` on `SwitchResponse` | Callers that only need `.clicked()` do not pay for naming the struct. |

---

## Testing

`cargo test -p mud_egui_components mud_switch` runs 18 pure-function tests:

| Area | Tests |
|---|---|
| Metrics | `ExtraSmall`→`Small` and `ExtraLarge`→`Large` folding; monotonicity of track width, height and thumb; thumb is larger than track height for every size |
| Placement | `is_vertical` and `label_leads` for every `Placement` variant |
| Row geometry | `row_size` horizontal and vertical; `track_rect` centres vertically and follows placement; `label_rect` follows placement |
| Thumb | rest positions at both ends; overhang applied; vertical centre preserved |
| Keyboard | uncheck beats check beats space; space toggles; no key → `None` |
| Colour | ambient roles; `lerp_color` endpoints and midpoint; `with_alpha` preserves RGB |
| Builders | defaults match MudBlazor; every field set; tag round-trip |

No test creates a live `egui::Context`. `MudSwitch::show` needs a live `Ui`
(layout, painting, animation, theme lookup) and is therefore left to
integration tests; everything it delegates to is covered above.

Suggested integration tests (`egui_kittest` or a headless `Context::run`
harness):

- A click toggles the bound value and sets `changed` for exactly one frame.
- `ArrowLeft` / `ArrowRight` / `Enter` / `Space` on a focused switch produce
  the documented values.
- A disabled switch ignores pointer and keyboard input.
- A read-only switch is not dimmed but cannot be toggled.
- The thumb's X position advances monotonically with `animate_bool` progress.
- A `thumb_icon` renders through `MudIcon`, and is absent when unset.

```bash
cargo test -p mud_egui_components mud_switch
```

---

## Differences from MudBlazor

- **`Value` / `ValueChanged` become a caller-owned `&'a mut bool` plus
  `SwitchResponse::changed`.** The caller owns the value; the response reports
  the frame it moved. This is the same split `MudCheckBox`, `MudRadio` and
  `MudCollapse` use.
- **The generic `MudSwitch<T>` is bound to `bool`.** MudBlazor converts
  arbitrary `T` through `BoolConverter<T>`; egui has no converter pipeline.
  The port would rather bind the common case honestly than invent a
  converter trait that has no Blazor counterpart.
- **`ChildContent` is not ported.** The label is the string form; a custom
  label closure has no MudBlazor switch analogue that would justify a
  lifetime-bearing field on the builder.
- **`ThumbIconColor` is structurally conditional.** It is only read when
  `thumb_icon.is_some()`, matching the documented remark rather than relying
  on the caller to respect it.
- **Colour interpolation is explicit.** MudBlazor's SCSS animates the track
  colour with a CSS transition; the port lerps between the resolved unchecked
  and checked colours by the same `progress` that drives the thumb, which is
  the immediate-mode equivalent and keeps both transitions in lockstep.
- **The unchecked-track fallback is a muted text colour at 120 alpha.** This is
  an approximation of `--mud-palette-text-secondary`; it is named at the top of
  the file (`UNCHECKED_TRACK_ALPHA`) so it can be tuned without touching the
  render path.
- **The switch metrics are constants derived from MudBlazor's SCSS.** They are
  deliberately named and grouped in `switch_metrics` so a theme that wants
  different proportions can adjust them in one place, rather than scattering
  magic numbers through the painter.
- **Keyboard priority is documented and deterministic.** When several keys are
  reported pressed in one frame, `uncheck` wins over `check`, which wins over
  `space`. egui reports key presses independently, so this is a real
  possibility and worth pinning down in a unit test.
- **`Class` is stored but not resolved.** The switch's visual identity comes
  from `color`, `unchecked_color`, `thumb_icon_color` and `size`; a
  utility-class frame would composite a second fill over the track.
- **`Style` and `UserAttributes` are not ported.** egui has no inline CSS
  strings and no HTML tag to splat attributes onto.