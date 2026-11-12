# MudCheckBox for `mud_egui`

A checkbox for boolean or tri-state values, for
[egui](https://github.com/emilk/egui), ported from MudBlazor's
[`MudCheckBox<T>`](https://mudblazor.com/components/checkbox) (MudBlazor 8.13.0).

```rust
let mut accepted = false;
ui.add(
    MudCheckBox::new(&mut accepted)
        .label("I agree to the terms")
        .color(Color::Primary),
);
if accepted {
    // …
}
```

Every label renders through **`MudText`**, every glyph through **`MudIcon`**.

---

## Contents

1. [Design principle: API parity, not API invention](#design-principle-api-parity-not-api-invention)
2. [Architecture](#architecture)
3. [API reference](#api-reference)
4. [Examples](#examples)
5. [Binding model](#binding-model)
6. [Layout and rendering](#layout-and-rendering)
7. [Performance and memory choices](#performance-and-memory-choices)
8. [Testing](#testing)
9. [Differences from MudBlazor](#differences-from-mudblazor)

---

## Design principle: API parity, not API invention

The MudBlazor 8.13.0 parameter surface was verified against DNDocs before
implementation. `MudCheckBox<T>` inherits from `MudBooleanInput<T>`, so its
surface combines check-box-specific appearance parameters with the shared
boolean-input parameters.

### Direct `MudCheckBox` parameters

| MudBlazor | Type | Default | Rust mapping |
|---|---|---|---|
| `CheckedIcon` | `string` | `CheckBox` | `checked_icon(s)` |
| `Dense` | `bool` | `false` | `dense(bool)` |
| `IndeterminateIcon` | `string` | `IndeterminateCheckBox` | `indeterminate_icon(s)` |
| `KeyboardEnabled` | `bool` | `true` | `keyboard_enabled(bool)` |
| `TriState` | `bool` | `false` | `tri_state(bool)` |
| `UncheckedColor` | `Color?` | `null` | `unchecked_color(Color)` |
| `UncheckedIcon` | `string` | `CheckBoxOutlineBlank` | `unchecked_icon(s)` |

### Inherited from `MudBooleanInput<T>`

| MudBlazor | Type | Default | Rust mapping |
|---|---|---|---|
| `Value` | `T?` | — | caller-owned `&mut T` via [`CheckBoxValue`] |
| `ValueChanged` | `EventCallback<T?>` | — | `CheckBoxResponse::changed` |
| `Label` | `string?` | `null` | `label(s)` |
| `LabelPlacement` | `Placement` | `End` | `label_placement(Placement)` |
| `Ripple` | `bool` | `true` | `ripple(bool)` |
| `Size` | `Size` | `Medium` | `size(Size)` |
| `Color` | `Color` | `Default` → `Primary` | `color(Color)` |
| `Disabled` | `bool` | `false` | `disabled(bool)` |
| `ReadOnly` | `bool` | `false` | `read_only(bool)` |
| `Class` | `string?` | `null` | `classes(iter)` — stored, not resolved |
| `Tag` | `object?` | `null` | `tag(t)` / `tag_ref::<U>()` |

### Deliberately not ported

| MudBlazor | Reason |
|---|---|
| `Style` | egui has no inline CSS strings. |
| `UserAttributes` | There is no HTML tag to splat attributes onto. |
| `Class` resolution | Stored on `base` for parity only. The checkbox's colour comes from `Color` / `UncheckedColor`; a utility-class frame would composite a second fill. |
| Form-validation parameters (`Required`, `Error`, `ErrorText`, `HelperText`, `ConversionError`, `Strict`, `Immediate`, `DebounceInterval`) | Inherited from `MudFormComponent` / `MudBaseInput`; they have no egui equivalent without a form system. |
| `Typo` | **Not a parameter on `MudCheckBox` in 8.13.0.** The label uses the theme's default typography (`Body1`). Adding a `Typo` builder would be API invention. |
| `ChildContent` (from `MudBooleanInput`) | Not exposed by `MudCheckBox`’s DNDocs page; the custom-icon use case is served by `CheckedIcon` / `UncheckedIcon` / `IndeterminateIcon`. |

---

## Architecture

```
inputs/check_box/
└── mud_check_box.rs
    ├── CheckBoxValue       trait; impl for bool and Option<bool>
    ├── MudCheckBox<'a, T>  builder + show
    ├── CheckBoxResponse    response + changed
    ├── PaintState          Checked | Unchecked | Indeterminate
    ├── pure helpers        icon_side, next_value, layout_size,
    │                       is_vertical, label_leads, icon_rect, label_rect,
    │                       paint_state, resolve_icon, with_alpha, apply_keys
    ├── ripple              start_ripple, paint_ripple (memory-backed)
    └── impl Widget         ui.add(MudCheckBox::new(&mut v))
```

**Pure layer.** The icon size, the value cycle (boolean and tri-state), the
row layout, the paint state, the glyph resolution and the keyboard mapping are
free functions over `Option<bool>`, `f32` and `Placement`. None touch
`egui::Context`.

**Binding layer.** `CheckBoxValue` is a two-method trait implemented for `bool`
and `Option<bool>`. One `MudCheckBox` serves both modes; the trait is the
seam.

**Component layer.** `show` measures the label, allocates the row, applies a
click or key press, paints the ripple and the icon, and draws the label through
`MudText`.

---

## API reference

### `MudCheckBox<'a, T: CheckBoxValue>`

| Method | Type | Default | Mirrors | Description |
|---|---|---|---|---|
| `new(value)` | `&'a mut T` | — | `Value` | Binds the checkbox to a `bool` or `Option<bool>`. |
| `label(s)` | `impl Into<String>` | `None` | `Label` | Label text. |
| `label_placement(p)` | `Placement` | `End` | `LabelPlacement` | `Start` / `End` / `Top` / `Bottom` (plus `Left` / `Right`). |
| `dense(b)` | `bool` | `false` | `Dense` | Compact gap (4 pt instead of 8 pt). |
| `color(c)` | `Color` | `Primary` | `Color` | Accent of the checked and indeterminate states. |
| `unchecked_color(c)` | `Color` | `None` | `UncheckedColor` | Outline colour of the unchecked state. |
| `size(s)` | `Size` | `Medium` | `Size` | Icon side: 18 / 24 / 32 pt. |
| `ripple(b)` | `bool` | `true` | `Ripple` | Click ripple. |
| `disabled(b)` | `bool` | `false` | `Disabled` | Dims and disables. |
| `read_only(b)` | `bool` | `false` | `ReadOnly` | Disables without dimming. |
| `tri_state(b)` | `bool` | `false` | `TriState` | Enables the indeterminate state. |
| `keyboard_enabled(b)` | `bool` | `true` | `KeyboardEnabled` | Keyboard control. |
| `checked_icon(s)` | `impl Into<String>` | `CheckBox` | `CheckedIcon` | Checked glyph. |
| `unchecked_icon(s)` | `impl Into<String>` | `CheckBoxOutlineBlank` | `UncheckedIcon` | Unchecked glyph. |
| `indeterminate_icon(s)` | `impl Into<String>` | `IndeterminateCheckBox` | `IndeterminateIcon` | Indeterminate glyph. |
| `classes(iter)` | `iter of AsRef<str>` | none | `Class` | Stored on `ComponentBase`. |
| `tag(t)` / `tag_ref::<U>()` | `T: Any + Send + Sync` | `None` | `Tag` | Arbitrary user data. |
| `show(ui)` | `&mut Ui` | — | — | Returns `CheckBoxResponse`. |

### `CheckBoxResponse`

| Field | Type | Description |
|---|---|---|
| `response` | `Response` | The whole row (icon + label). Deref target. |
| `changed` | `bool` | `true` on the frame the value changed. |

`CheckBoxResponse` implements `Deref<Target = Response>`, so `out.clicked()`,
`out.hovered()` and `out.rect` work directly.

### `CheckBoxValue`

| Method | Description |
|---|---|
| `get() -> Option<bool>` | Current value; `None` means indeterminate. |
| `set(Option<bool>)` | Replaces the value. |

Implemented for `bool` and `Option<bool>`. A custom type can implement it to
adapt its own storage.

### `Placement`

`Start` (default for `MudSwitch`, `End` for `MudCheckBox`), `End`, `Top`,
`Bottom`, plus the `Left` / `Right` aliases. `Start` / `Left` place the label
before the icon; `End` / `Right` after it. `Top` / `Bottom` stack vertically.

### `Size` mapping

| `Size` | Icon side |
|---|---|
| `ExtraSmall` | 18 pt |
| `Small` | 18 pt |
| `Medium` | 24 pt |
| `Large` | 32 pt |
| `ExtraLarge` | 32 pt |

`ExtraSmall` folds into `Small` and `ExtraLarge` into `Large`, matching the
other input components in this workspace.

### Constants

| Name | Value |
|---|---|
| `ICON_SMALL` / `ICON_MEDIUM` / `ICON_LARGE` | `18.0` / `24.0` / `32.0` |
| `GAP` / `DENSE_GAP` | `8.0` / `4.0` |
| `RIPPLE_DURATION` | `0.35` s |
| `RIPPLE_MAX_RADIUS_FACTOR` | `1.2` |
| `RIPPLE_ALPHA` | `60` |
| `DISABLED_OPACITY` | `0.38` |
| `UNCHECKED_OUTLINE_ALPHA` | `160` |

---

## Examples

### Boolean checkbox

```rust
let mut accepted = false;
ui.add(
    MudCheckBox::new(&mut accepted)
        .label("I agree to the terms")
        .color(Color::Primary),
);
```

### Tri-state checkbox

```rust
let mut state: Option<bool> = None; // indeterminate

ui.add(
    MudCheckBox::new(&mut state)
        .tri_state(true)
        .label("Select all")
        .indeterminate_icon(filled::INDETERMINATE_CHECK_BOX),
);

match state {
    None => log::info!("some children selected"),
    Some(true) => log::info!("all selected"),
    Some(false) => log::info!("none selected"),
}
```

### Label before the icon

```rust
MudCheckBox::new(&mut checked)
    .label("Remember me")
    .label_placement(Placement::Start)
    .show(ui);
```

### Vertical label

```rust
MudCheckBox::new(&mut checked)
    .label("Above")
    .label_placement(Placement::Top)
    .show(ui);

MudCheckBox::new(&mut checked)
    .label("Below")
    .label_placement(Placement::Bottom)
    .show(ui);
```

### Dense row in a table toolbar

```rust
MudCheckBox::new(&mut self.select_all)
    .label("Select all")
    .dense(true)
    .size(Size::Small)
    .show(ui);
```

### Read-only

```rust
MudCheckBox::new(&mut checked)
    .label("Locked value")
    .read_only(true)
    .show(ui);
```

The value cannot be changed, but the checkbox is not dimmed.

### Disabled

```rust
MudCheckBox::new(&mut checked)
    .label("Unavailable")
    .disabled(true)
    .show(ui);
```

The checkbox is dimmed to 38 % opacity and ignores clicks and keys.

### Custom glyphs (star checkbox)

```rust
MudCheckBox::new(&mut starred)
    .checked_icon(filled::STAR)
    .unchecked_icon(filled::STAR_BORDER)
    .color(Color::Warning)
    .show(ui);
```

### Unchecked colour

```rust
MudCheckBox::new(&mut checked)
    .unchecked_color(Color::Error)
    .show(ui);
```

The outline of the unchecked box is red; the checked fill is still `Primary`.

### Reacting to a change

```rust
let out = MudCheckBox::new(&mut self.accepted)
    .label("Accept")
    .show(ui);
if out.changed {
    log::info!("accepted = {}", self.accepted);
}
```

### Keyboard-only form

```rust
// With `keyboard_enabled(true)` (the default), Tab to the checkbox and use:
//   Space     cycle
//   Enter     set true
//   Delete    set false
//   Backspace set indeterminate (only with tri_state)
```

### Tag round-trip

```rust
let cb = MudCheckBox::new(&mut checked).tag("terms");
assert_eq!(cb.tag_ref::<&str>(), Some(&"terms"));
```

---

## Binding model

`MudCheckBox` is generic over [`CheckBoxValue`]:

```rust
impl CheckBoxValue for bool       // boolean mode
impl CheckBoxValue for Option<bool> // tri-state mode
```

The caller owns the value and passes `&mut T` to `new`:

```rust
let mut a = false;          // bool
let mut b: Option<bool> = None; // Option<bool>

MudCheckBox::new(&mut a).show(ui);
MudCheckBox::new(&mut b).tri_state(true).show(ui);
```

`ValueChanged` is not a callback; it is `CheckBoxResponse::changed`, a field on
the frame’s result. This is the same split `MudAlert` and `MudCollapse` use.

### Tri-state cycle

| Current | Click / `Space` |
|---|---|
| `None` | `Some(true)` |
| `Some(true)` | `Some(false)` |
| `Some(false)` | `None` |

With `tri_state(false)`, the value toggles between `true` and `false`; a `None`
binding becomes `Some(true)` on the first click.

### Keyboard mapping

| Key | Result |
|---|---|
| `Space` | Same as a click |
| `Enter` | `Some(true)` |
| `Delete` | `Some(false)` |
| `Backspace` | `None` (only when `tri_state`) |

---

## Layout and rendering

### Horizontal

```
End:    [ ✓ ][  8 pt  ][ Label … ]
Start:  [ Label … ][ 8 pt ][ ✓ ]
```

- Icon side: 18 / 24 / 32 pt.
- Row height: `max(icon_side, label_height)`.
- Gap: 8 pt, or 4 pt when `dense`.

### Vertical

```
Top:    [ Label … ]
        [  4/8 pt ]
        [ ✓ ]

Bottom: [ ✓ ]
        [  4/8 pt ]
        [ Label … ]
```

The row width is `max(icon_side, label_width)`; the height is
`icon_side + gap + label_height`.

### Paint

| State | Glyph | Colour |
|---|---|---|
| Checked | `CheckedIcon` or `CHECK_BOX` | `Color` |
| Unchecked | `UncheckedIcon` or `CHECK_BOX_OUTLINE_BLANK` | `UncheckedColor`, or `Color` at 160 alpha |
| Indeterminate | `IndeterminateIcon` or `INDETERMINATE_CHECK_BOX` | `Color` |

Disabled checkboxes are gamma-multiplied by `DISABLED_OPACITY` (0.38).
`read_only(true)` does **not** dim; it only blocks interaction.

### Ripple

A filled circle of radius `icon_side × 0.5 × 1.2` expands from the icon centre
over `RIPPLE_DURATION` (0.35 s), then fades out. The start time is stored in
`egui::Memory` keyed by the row’s `Response::id`. A repaint is requested only
while the ripple is in flight.

---

## Performance and memory choices

| Choice | Effect |
|---|---|
| Theme fetched once per frame | One `MudTheme::fetch` and up to two `Color::resolve` calls. |
| Pure helpers on `Copy` values | Size, cycle, layout, state and keyboard are arithmetic; unit-testable without a `Context`. |
| No `Vec` or `String` allocation | The label is measured with `layout_no_wrap` and moved into `MudText`; no per-frame clone. |
| One `allocate_exact_size` | The whole row is one rect and one `Response`. |
| Ripple in `egui::Memory` | No state on the component; the builder stays stateless. |
| `bool::then` / `Option::filter` | No `Option<Option<_>>`, no branches in the paint path. |
| Keyboard only read when focused | `apply_keys` runs only when `keyboard_enabled && enabled && has_focus`. |
| Disabled path skips click and keys | `Sense::hover()` for disabled and read-only; no interaction registration. |

---

## Testing

`cargo test -p mud_egui_components mud_check_box` runs 14 pure-function tests:

| Area | Tests |
|---|---|
| Icon size | every `Size`; edge folding |
| Boolean cycle | toggle, `None → Some(true)` |
| Tri-state cycle | full `None → true → false → None` loop; one cycle returns to indeterminate |
| Placement | `is_vertical` and `label_leads` for every variant |
| Layout | `layout_size` horizontal / vertical / no label; `icon_rect` and `label_rect` follow placement |
| Paint state | boolean and tri-state mapping |
| Glyph | override wins; default is non-empty |
| `CheckBoxValue` | `bool` and `Option<bool>` round-trip |
| Builders | defaults match MudBlazor; every field set; tag round-trip |

No test creates an `egui::Context`. `show` needs a live `Ui` (layout, painting,
theme lookup, input) and is therefore left to integration tests; everything it
delegates to is covered above.

Suggested integration tests (`egui_kittest` or a headless `Context::run`
harness):

- Clicking toggles a boolean checkbox and sets `changed`.
- Clicking a tri-state checkbox cycles `None → true → false → None`.
- `read_only(true)` blocks clicks without dimming.
- `disabled(true)` blocks clicks and paints at 38 % opacity.
- `Space` toggles a focused checkbox; `Enter` / `Delete` / `Backspace` set
  `true` / `false` / `None` (the last only with `tri_state`).
- The ripple starts on click and requests a repaint for at most 0.35 s.

```bash
cargo test -p mud_egui_components mud_check_box
```

---

## Differences from MudBlazor

- **`Value` / `ValueChanged` become a caller-owned `&mut T` plus
  `CheckBoxResponse::changed`.** The caller owns the value; `changed` is
  `true` on the frame it moved. This is the same split `MudAlert` and
  `MudCollapse` use.
- **`CheckBoxValue` is a new public trait.** It is the seam that lets one
  component serve `bool` and `Option<bool>`. MudBlazor achieves the same with
  its generic `MudCheckBox<T>` and `BoolConverter<T>`; a trait is the idiomatic
  Rust equivalent.
- **`Class` is stored but not resolved.** The checkbox’s colour comes from
  `Color` and `UncheckedColor`; a utility-class frame would composite a second
  fill.
- **`Style` and `UserAttributes` are not ported.** No CSS strings, no HTML tag.
- **Form-validation parameters are not ported.** `Required`, `Error`,
  `ErrorText`, `HelperText`, `ConversionError`, `Strict`, `Immediate` and
  `DebounceInterval` belong to `MudFormComponent` / `MudBaseInput`. Without a
  form system in egui there is nothing for them to drive.
- **`Typo` is not a parameter.** 8.13.0’s `MudCheckBox` does not expose one;
  the label uses the theme’s `Body1`. A `Typo` builder would be API invention.
- **`LabelPlacement` uses the existing `Placement` enum.** `Start` / `Left`
  place the label before the icon; `End` / `Right` after. `Top` / `Bottom`
  stack vertically. The default is `End`, matching MudBlazor.
- **`Size` folds `ExtraSmall` into `Small` and `ExtraLarge` into `Large`.**
  Consistent with `MudSwitch` and `MudSlider`.
- **The ripple is memory-backed.** Start time lives in `egui::Memory`; the
  component itself is stateless between frames.
- **The default `Color` is `Primary`, not `Default`.** MudBlazor’s
  `MudBooleanInput` default is `Default`, which resolves to `Primary` for
  this component. Pinning it to `Primary` makes the default visible in the
  builder table; a caller can still set `Color::Default` explicitly if a
  theme maps it to something else.