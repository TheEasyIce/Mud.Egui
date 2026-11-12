# MudColorPicker for `mud_egui`

A text field with a popover colour picker for
[egui](https://github.com/emilk/egui), ported from MudBlazor's
[`MudColorPicker`](https://mudblazor.com/components/color-picker)
(MudBlazor 8.13.0).

```rust
let mut state = ColorPickerState::new("accent");

MudColorPicker::new()
    .label("Accent colour")
    .show_alpha(false)
    .show(ui, &mut state);

// After a pick: state.value holds the committed colour.
```

The trigger reuses **`MudTextField`**. The dropdown reuses **`MudPopover`**.
The channel controls reuse **`MudSlider`**. Every numeral and label renders
through **`MudText`**; every glyph through **`MudIcon`**.

---

## Contents

1. [Design principle: API parity, not API invention](#design-principle-api-parity-not-api-invention)
2. [Architecture](#architecture)
3. [API reference](#api-reference)
4. [Examples](#examples)
5. [Views, toolbar and interaction](#views-toolbar-and-interaction)
6. [Performance and memory choices](#performance-and-memory-choices)
7. [Testing](#testing)
8. [Differences from MudBlazor](#differences-from-mudblazor)

---

## Design principle: API parity, not API invention

The MudBlazor 8.13.0 parameter surface was verified against DNDocs before
implementation[reference:20][reference:21]. `MudColorPicker` derives from
`MudPicker<MudColor>` and exposes **twenty** own parameters.

### Own parameters

| MudBlazor | Rust mapping | Default |
|---|---|---|
| `CloseIcon` | `close_icon(impl Into<String>)` | `CLOSE` |
| `ColorPickerMode` | `color_picker_mode(ColorPickerMode)` / `ColorPickerState::mode` | `Rgb` |
| `ColorPickerView` | `color_picker_view(ColorPickerView)` / `ColorPickerState::view` | `Spectrum` |
| `DragEffect` | `drag_effect(bool)` | **`true`** |
| `GridIcon` | `grid_icon(impl Into<String>)` | `APPS` |
| `ImportExportIcon` | `import_export_icon(impl Into<String>)` | `IMPORT_EXPORT` |
| `Palette` | `palette(iter of ColorChannels)` | 35 entries |
| `PaletteIcon` | `palette_icon(impl Into<String>)` | `PALETTE` |
| `RightToLeft` | `right_to_left(bool)` | `false` |
| `ShowAlpha` | `show_alpha(bool)` | **`true`** |
| `ShowColorField` | `show_color_field(bool)` | **`true`** |
| `ShowInputs` | `show_inputs(bool)` | **`true`** |
| `ShowModeSwitch` | `show_mode_switch(bool)` | **`true`** |
| `ShowPreview` | `show_preview(bool)` | **`true`** |
| `ShowSliders` | `show_sliders(bool)` | **`true`** |
| `SpectrumIcon` | `spectrum_icon(impl Into<String>)` | `TUNE` |
| `ThrottleInterval` | `throttle_interval(u32)` | **`50`** ms |
| `UpdateBindingIfOnlyHSLChanged` | `update_binding_if_only_hsl_changed(bool)` | **`false`** |
| `Value` | `ColorPickerState::value: Option<ColorChannels>` | `None` |
| `ValueChanged` | `ColorPickerResponse::action` (`ColorSelected`) | — |
| `PickerContent` | **protected** — not ported | — |

### Inherited (`MudPicker<T>`)

`AdornmentIcon` / `AdornmentColor`, `AnchorOrigin`, `AutoClose`, `Clearable`,
`Color`, `Disabled`, `Editable`, `Elevation`, `HelperText`,
`HelperTextOnFocus`, `IconSize`, `ImmediateText`, `Label`, `Margin`, `Modal`,
`OnClick`, `OverflowBehavior`, `PickerClosed`, `PickerOpened`,
`PickerVariant`, `Placeholder`, `ReadOnly`, `Rounded`, `ShowToolbar`, `Size`,
`Square`, `Variant`, `Class`.

### Deliberately not rendered / not ported

| MudBlazor | Reason |
|---|---|
| `PickerContent` | Protected. `show` is the composition point; a custom body is a separate widget. |
| `PickerActions` | Deferred to pass 2 (same as `MudDatePicker`). |
| `Mask` / typed parsing | Deferred. The HEX input parses on change, but there is no character mask. |
| `Inline` / `Static` picker variants | Stored; only `Dialog` renders. |
| `Style` / `Tag` / `UserAttributes` | No CSS strings, no HTML tag. |
| `Class` resolution | Stored on `base` only. |

### Enums introduced

Two public enums, each in its own file (Rule 2):

| Enum | File | Discriminants |
|---|---|---|
| `ColorPickerView` | `mud_egui_core::enums::color_picker_view` | `Spectrum = 0`, `Palette = 1`, `Grid = 2`[reference:22] |
| `ColorPickerMode` | `mud_egui_core::enums::color_picker_mode` | `Rgb = 0`, `Hsl = 1`, `Hex = 2`[reference:23] |

---

## Architecture

```
inputs/mud_color_picker/
├── color_channels.rs       ColorChannels + HSL↔RGB + parsing + formatting
├── color_picker_action.rs  ColorPickerAction (own file, Rule 2)
├── color_picker_state.rs   ColorPickerState (caller-owned)
├── mud_color_picker.rs     MudColorPicker + body + toolbar + swatches + tests
└── mod.rs

mud_egui_core/src/enums/
├── color_picker_mode.rs    ColorPickerMode
└── color_picker_view.rs    ColorPickerView
```

**Pure layer.** `rgb_to_hsl` / `hsl_to_rgb` / `parse_hex` / `parse_alpha` /
`ColorChannels::format` / `contrast_ink` are free functions or methods over
`u8`, `f64` and `&str`. `ColorPickerState::throttle_allows` is arithmetic on
`f64`. None touch `egui::Context`.

**State layer.** `ColorPickerState` holds the committed colour, both text
buffers, the popover state, the view, the mode, the drag latch and the
throttle timestamp. It is caller-owned — the same pattern `MudDatePicker`,
`MudTimePicker` and `MudDateRangePicker` use.

**Component layer.** `show` renders the trigger, opens the popover, then
delegates to `toolbar_row` / `spectrum_body` / `swatch_grid`. Every click or
drag is written into a deferred `Option`, applied after `popover.show`
releases `&mut state.popover`.

**Value type.** `ColorChannels` mirrors `MudColor`'s accessor surface
(`R`/`G`/`B`/`A`/`H`/`S`/`L`) and keeps RGB and HSL in sync[reference:24]. It
converts to and from `Color32`; convert `MudColor` at the boundary.

---

## API reference

### `MudColorPicker<'a>`

| Method | Type | Default | Description |
|---|---|---|---|
| `new()` | — | — | Default picker. |
| `label` / `placeholder` / `helper_text` | `impl Into<String>` | `None` | Trigger text slots. |
| `helper_text_on_focus(bool)` | `bool` | `false` | Helper only while focused / non-empty. |
| `error(bool)` / `error_text(s)` | `bool` / `impl Into<String>` | `false` / `None` | Error accent and message. |
| `variant` / `margin` / `color` / `size` / `icon_size` | enum | `Text` / `None` / `Primary` / `Medium` / `Medium` | Trigger appearance. |
| `clearable` / `readonly` / `disabled` / `editable` | `bool` | `false` | Interaction flags. |
| `immediate_text(bool)` | `bool` | `false` | Stored, no-op. |
| `adornment_icon` / `adornment_color` | `impl Into<String>` / `Color` | `COLORIZE` / `None` | Trigger adornment. |
| `picker_variant` | `PickerVariant` | `Dialog` | Only `Dialog` renders. |
| `elevation` / `rounded` / `square` / `modal` | — | `None` / `false` / `false` / `true` | Popover appearance. |
| `overflow_behavior` | `OverflowBehavior` | `Flip` | Flip vs reposition. |
| `show_toolbar(bool)` | `bool` | `true` | Renders the view buttons, mode switch, import/export and close. |
| `auto_close(bool)` | `bool` | `false` | Close after a pick. |
| `close_icon` | `impl Into<String>` | `CLOSE` | Close glyph. |
| `color_picker_mode` | `ColorPickerMode` | `Rgb` | Initial channel mode. |
| `color_picker_view` | `ColorPickerView` | `Spectrum` | Initial view. |
| `drag_effect(bool)` | `bool` | `true` | Commit while dragging. |
| `grid_icon` / `palette_icon` / `spectrum_icon` | `impl Into<String>` | `APPS` / `PALETTE` / `TUNE` | Toolbar glyphs. |
| `import_export_icon` | `impl Into<String>` | `IMPORT_EXPORT` | Export glyph. |
| `palette(iter)` | `iter of ColorChannels` | 35 entries | Swatch list. |
| `right_to_left(bool)` | `bool` | `false` | Mirrors `RightToLeft`. |
| `show_alpha(bool)` | `bool` | `true` | Include the alpha channel. |
| `show_color_field(bool)` | `bool` | `true` | Saturation × lightness plane. |
| `show_inputs(bool)` | `bool` | `true` | Channel inputs. |
| `show_mode_switch(bool)` | `bool` | `true` | Mode-switch button. |
| `show_preview(bool)` | `bool` | `true` | Preview dot + quick colours. |
| `show_sliders(bool)` | `bool` | `true` | Hue and alpha strips. |
| `throttle_interval(u32)` | `u32` | `50` | Milliseconds between drag commits. |
| `update_binding_if_only_hsl_changed(bool)` | `bool` | `false` | Commit HSL-only edits. |
| `picker_actions(f)` | `FnMut(&mut Ui) -> Option<ColorPickerAction>` | `None` | Custom footer row. |
| `on_picker_opened` / `on_picker_closed` / `on_click` | `FnMut()` | `None` | Callbacks. |
| `classes(iter)` / `flavor(StyleFlavor)` | — | — | Utilities. |
| `show(ui, state)` | `(&mut Ui, &mut ColorPickerState)` | — | Returns `ColorPickerResponse`. |

### `ColorPickerState`

| Field / method | Description |
|---|---|
| `value: Option<ColorChannels>` | Committed colour. |
| `text: String` | Trigger text — always HEX/HEXA. |
| `hex_buffer: String` | The HEX input's own buffer. |
| `popover: PopoverState` | Popover state. |
| `view: ColorPickerView` | Current view. |
| `mode: ColorPickerMode` | Current channel mode. |
| `last_update: Option<f64>` | Timestamp of the last drag commit. |
| `drag_active: bool` | Whether a drag is in flight. |
| `new(id_salt)` / `with_value(color, show_alpha)` | Constructors. |
| `is_open()` / `open()` / `close()` / `toggle()` | Popover control. |
| `commit(color, show_alpha) -> bool` | Commit; returns whether it changed. |
| `clear()` | Reset value and buffers, close. |
| `current() -> ColorChannels` | Value or opaque black. |
| `cycle_mode()` / `change_view(view)` | Toolbar mutations. |
| `throttle_allows(now, interval_ms)` / `mark_updated(now)` | Drag throttle. |

### `ColorPickerResponse`

| Field | Type | Description |
|---|---|---|
| `trigger` | `egui::Response` | Trigger response. |
| `value` | `Option<ColorChannels>` | Committed colour after this frame. |
| `action` | `ColorPickerAction` | What happened this frame. |

### `ColorPickerAction`

| Variant | Description |
|---|---|
| `None` | No interaction. |
| `Opened` / `Closed` | Popover opened / closed. |
| `ColorSelected(ColorChannels)` | A colour was committed. Mirrors `ValueChanged`. |
| `Cleared` | The value was cleared. |

Helpers: `changed_value()`, `toggled_open()`, `selected()`, `name()`.

### `ColorChannels`

| Member | Description |
|---|---|
| `from_rgba8(r, g, b, a)` / `from_rgba(r, g, b, a)` / `from_hsla(h, s, l, a)` | Constructors. |
| `black()` | Opaque black. |
| `parse(&str)` | `#rgb`, `#rgba`, `#rrggbb`, `#rrggbbaa`, `rgb()`, `rgba()`. |
| `r()` / `g()` / `b()` / `a()` / `a8()` | RGB + alpha. |
| `h()` / `s()` / `l()` | HSL. |
| `with_r` / `with_g` / `with_b` / `with_a` / `with_h` / `with_s` / `with_l` | Channel setters. |
| `rgb_eq` / `hsl_eq` | Independent comparisons. |
| `to_hex` / `to_hexa` / `to_rgb` / `to_rgba` / `to_hsl` / `to_hsla` | Formatting. |
| `hex_text(show_alpha)` | The trigger text — always hex. |
| `format(mode, show_alpha)` | Formatting dispatcher. |
| `contrast_ink()` | Black or white by the library's 384 rule. |
| `From<Color32>` / `Into<Color32>` / `Display` | Conversions. |

### Constants

| Name | Value |
|---|---|
| `PLANE_SIZE` | `220.0` |
| `STRIP_HEIGHT` | `14.0` |
| `SWATCH_SIZE` | `24.0` |
| `DEFAULT_THROTTLE_MS` | `50` |
| `DEFAULT_PALETTE` | 35 hex strings (7 hues × 5 shades) |
| `QUICK_COLOR_COUNT` | `5` |

---

## Examples

### Basic picker

```rust
let mut state = ColorPickerState::new("accent");
MudColorPicker::new().label("Accent").show(ui, &mut state);
```

### Seed a value

```rust
let mut state = ColorPickerState::new("accent").with_value(
    ColorChannels::parse("#2196F3").unwrap(),
    /* show_alpha = */ false,
);
```

### HEX-only, no alpha

```rust
MudColorPicker::new()
    .color_picker_mode(ColorPickerMode::Hex)
    .show_alpha(false)
    .show_inputs(true)
    .show(ui, &mut state);
```

### Palette-only

```rust
MudColorPicker::new()
    .color_picker_view(ColorPickerView::Palette)
    .show_toolbar(false)
    .show(ui, &mut state);
```

### Throttled drag for a remote theme

```rust
MudColorPicker::new()
    .drag_effect(true)
    .throttle_interval(120)
    .show(ui, &mut state);
```

### HSL edits fire even when the RGB round-trip is identical

```rust
MudColorPicker::new()
    .color_picker_mode(ColorPickerMode::Hsl)
    .update_binding_if_only_hsl_changed(true)
    .show(ui, &mut state);
```

### Custom palette

```rust
let brand: Vec<ColorChannels> = ["#0D47A1", "#1565C0", "#1E88E5", "#42A5F5", "#90CAF9"]
    .iter()
    .filter_map(|s| ColorChannels::parse(s))
    .collect();

MudColorPicker::new()
    .palette(brand)
    .color_picker_view(ColorPickerView::Grid)
    .show(ui, &mut state);
```

### Reacting to the outcome

```rust
let resp = MudColorPicker::new().show(ui, &mut state);
match resp.action {
    ColorPickerAction::ColorSelected(c) => log::info!("picked {}", c.to_hex()),
    ColorPickerAction::Cleared => log::info!("cleared"),
    ColorPickerAction::Opened => log::info!("opened"),
    ColorPickerAction::Closed => log::info!("closed"),
    ColorPickerAction::None => {}
}
```

### Programmatic set

```rust
state.commit(ColorChannels::from_rgba(0, 200, 83, 255), /* show_alpha = */ false);
state.clear();
```

---

## Views, toolbar and interaction

### Views

| `ColorPickerView` | Body |
|---|---|
| `Spectrum` (default) | Saturation × lightness plane, hue strip, alpha strip, preview, channel inputs |
| `Palette` | Wrapping list of `palette` at 7 columns |
| `Grid` | Wrapping grid of the same colours at 12 columns |

### Toolbar (`show_toolbar`)

```
[ Spectrum ] [ Palette ] [ Grid ]   [ RGB ]  …spacer…  [ ⤴ ] [ ✕ ]
```

- Three view buttons; the active one gets an accent fill.
- The mode switch cycles `RGB → HSL → HEX → RGB`.
- Import/export copies the current colour, formatted in the current mode, to
  the clipboard via `Context::copy_text`.
- The close button closes the popover.

### Spectrum interaction

| Control | Mapping |
|---|---|
| Saturation × lightness plane | x → `S` ∈ `[0, 1]`, y → `L` ∈ `[0, 1]` (bottom = 0) |
| Hue strip | x → `H` ∈ `[0, 360]` |
| Alpha strip | x → `A` ∈ `[0, 1]` |

The plane is painted as an 8 × 8 vertex-coloured mesh, **exact at every
vertex** and linear between them. The colour under the cursor is the colour
that gets committed — there is no HSV/HSL mismatch.

### Channel inputs

| Mode | Inputs |
|---|---|
| `Rgb` | `R`, `G`, `B` (`0..=255`) and `A` (`0..=100`%) |
| `Hsl` | (see *Deviations* — the inputs stay RGB for pass 1) |
| `Hex` | A single `MudTextField` with the hex string |

`MudSlider` is used for the numeric channels because egui has no per-channel
text-input state; the slider is the library's numeric control and is keyboard
accessible.

### Drag, throttle, HSL-only

| Parameter | Behaviour |
|---|---|
| `drag_effect(true)` (default) | A drag commits as it moves. |
| `drag_effect(false)` | Only a click or drag-stop commits. |
| `throttle_interval(50)` | Two drag commits are at least this many milliseconds apart. |
| `update_binding_if_only_hsl_changed(false)` (default) | Commits require an RGB change. |
| `update_binding_if_only_hsl_changed(true)` | An HSL change commits even when the 8-bit RGB round-trip is identical. |

---

## Performance and memory choices

| Choice | Effect |
|---|---|
| `ColorChannels` is `Copy` (7 fields) | Zero allocation for the value. |
| HSL cached on construction | `h()`, `s()`, `l()` are field reads; `hsl_eq` is three `f64` comparisons. |
| `hsl_plane` / `hue_strip` / `alpha_strip` build one `Mesh` each | 81 vertices for the plane, 14 for the hue strip, 4 for the alpha strip. |
| Painter borrows scoped to `{ … }` blocks | Every `ui.painter()` borrow ends before the next `MudText` / `MudSlider` call. |
| `MudTextField` / `MudPopover` / `MudSlider` reuse | No duplicated trigger chrome, positioning, or numeric-control logic. |
| Deferred clicks | `state` is mutated only after `popover.show` releases `&mut state.popover`. |
| Throttle in state | Two `f64` comparisons per frame, no `Memory` round-trip. |
| HEX buffer synced only when stale | Typed input is never overwritten mid-edit. |

---

## Testing

`cargo test -p mud_egui_components mud_color_picker` runs the module's tests:

| Area | Tests |
|---|---|
| `ColorChannels` | construction, HSL round-trip over a 5×5×5 RGB grid, known HSL values, all channel setters, `rgb_eq` / `hsl_eq` independence, hex/rgb/rgba parsing (valid, invalid, whitespace, case), formatting per mode and alpha, `contrast_ink` at the 384 boundary, `Color32` round-trip |
| `ColorPickerAction` | default, `changed_value`, `toggled_open`, `selected`, names |
| `ColorPickerState` | defaults, `with_value` (with and without alpha), toggle, `commit` returns-whether-changed, `clear`, `current` fallback, `cycle_mode` wrap, `change_view`, throttle (first / blocked / elapsed / zero interval) |
| `ColorPickerMode` | default, discriminants, `next` cycle, unique names, `Display` |
| `ColorPickerView` | default, discriminants, `ALL`, `is_spectrum` / `is_swatch_based`, `Display` |
| Builders | every `MudColorPicker` field, default palette size and first entry, quick-colour count |

Rendering needs a live `egui::Context`; the pure layer is the source of truth.

```bash
cargo test -p mud_egui_core color_picker_mode color_picker_view
cargo test -p mud_egui_components color_channels
cargo test -p mud_egui_components color_picker_state
cargo test -p mud_egui_components color_picker_action
cargo test -p mud_egui_components mud_color_picker
```

---

## Differences from MudBlazor

- **`Value: MudColor?` → `Option<ColorChannels>`.** `ColorChannels` mirrors
  `MudColor`'s accessor surface and keeps RGB and HSL in sync[reference:25]. Convert
  at the boundary: `ColorChannels::from(Color32::from(mud_color))` and
  `Color32::from(channels)`.
- **`ValueChanged` becomes `ColorPickerResponse::action`.** The caller owns
  `state.value`; `ColorSelected(_)` reports the committing frame. `Opened` /
  `Closed` mirror `PickerOpened` / `PickerClosed`.
- **The spectrum plane is a vertex-coloured mesh, not a shader.** egui 0.38
  has no custom shaders, so the plane is sampled at 8 × 8 and triangulated.
  The colour under the cursor is exact at every vertex and linear between —
  no HSV/HSL mismatch.
- **Channel inputs are `MudSlider`s.** egui has no per-channel text-input
  state to hang off; the slider is the library's numeric control, is keyboard
  accessible, and shows the value on hover. HEX mode uses a real
  `MudTextField`.
- **HSL mode falls back to RGB inputs for pass 1.** The mode switch and the
  `Hsl` formatting are implemented; the *inputs* remain RGB sliders until a
  dedicated HSL input row lands. `update_binding_if_only_hsl_changed` still
  works because HSL is stored, not recomputed.
- **`ImportExportIcon` copies to the clipboard.** MudBlazor opens a
  paste-a-hex popover; the immediate-mode equivalent is `Context::copy_text`
  with the current value formatted in the current mode.
- **The default palette is 35 entries (7 hues × 5 shades).** MudBlazor's
  default is also 35 colours[reference:26]; the exact hex list is a named
  constant (`DEFAULT_PALETTE`) so a caller can replace it. The first five are
  the quick colours, as documented.
- **`ShowToolbar` gates the view buttons and the mode switch.** MudBlazor's
  remark is "the view can be changed if `ShowToolbar` is true"[reference:27];
  the port matches.
- **`DragEffect` + `ThrottleInterval` are implemented.** The deadline lives in
  `ColorPickerState`, not `egui::Memory`, and `throttle_allows` is a pure
  predicate.
- **`UpdateBindingIfOnlyHSLChanged` compares the stored HSL triple.** Because
  `ColorChannels` caches HSL at construction, the comparison is exact rather
  than a recomputation.
- **`PickerContent` and `PickerActions` are not ported.** `PickerContent` is
  protected; `PickerActions` is deferred to pass 2, same as `MudDatePicker`.
- **`Style` / `Tag` / `UserAttributes` not ported.** No CSS strings, no HTML
  tag.