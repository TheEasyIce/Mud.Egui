# MudRadio + MudRadioGroup for `mud_egui`

A mutually exclusive option set for [egui](https://github.com/emilk/egui),
ported from MudBlazor’s [`MudRadio<T>`](https://mudblazor.com/components/radio)
and `MudRadioGroup<T>` (MudBlazor 8.13.0).

```rust
let mut selected: Option<&str> = None;

MudRadioGroup::new(&mut selected).show(ui, |g| {
    g.radio("a", "Option A");
    g.radio("b", "Option B");
    g.radio("c", "Option C");
});
```

Every label renders through **`MudText`**, every glyph through **`MudIcon`**.

---

## Contents

1. [Design principle: API parity, not API invention](#design-principle-api-parity-not-api-invention)
2. [Architecture](#architecture)
3. [API reference — MudRadioGroup](#api-reference--mudradiogroup)
4. [API reference — MudRadio](#api-reference--mudradio)
5. [Examples](#examples)
6. [Binding model](#binding-model)
7. [Layout and rendering](#layout-and-rendering)
8. [Performance and memory choices](#performance-and-memory-choices)
9. [Testing](#testing)
10. [Differences from MudBlazor](#differences-from-mudblazor)

---

## Design principle: API parity, not API invention

Both components were verified against DNDocs for MudBlazor 8.13.0.

### `MudRadio<T>` ported parameters

| MudBlazor | Type | Default | Rust mapping |
|---|---|---|---|
| `CheckedIcon` | `string` | `RadioButtonChecked` | `checked_icon(s)` |
| `Dense` | `bool` | `false` | `dense(bool)` |
| `IndeterminateIcon` | `string` | `IndeterminateCheckBox` | `indeterminate_icon(s)` — stored, **never painted** |
| `UncheckedColor` | `Color?` | `null` | `unchecked_color(Color)` |
| `UncheckedIcon` | `string` | `RadioButtonUnchecked` | `unchecked_icon(s)` |
| `ChildContent` | `RenderFragment` | — | `label(s)` / `content(f)` |
| `Disabled` | `bool` | `false` | `disabled(bool)` |
| `ReadOnly` | `bool` | `false` | `read_only(bool)` |
| `LabelPlacement` | `Placement` | `End` | `label_placement(Placement)` |
| `Color` | `Color` | `Default` → `Primary` | `color(Color)` |
| `Size` | `Size` | `Medium` | `size(Size)` |
| `Ripple` | `bool` | `true` | `ripple(bool)` |
| `Class` | `string?` | `null` | `classes(iter)` — stored |
| `Tag` | `object?` | `null` | `tag(t)` / `tag_ref::<U>()` |

### `MudRadioGroup<T>` ported parameters

| MudBlazor | Type | Default | Rust mapping |
|---|---|---|---|
| `ChildContent` | `RenderFragment` | — | closure to `show(ui, \|g\| ..)` |
| `Value` / `ValueChanged` | `T?` / callback | — | caller-owned `&mut Option<T>` |
| `ReadOnly` | `bool` | `false` | `read_only(bool)` |
| `Disabled` | `bool` | `false` | `disabled(bool)` |
| `Name` | `string?` | `null` | `name(s)` — stored, **inert** |
| `Class` | `string?` | `null` | `classes(iter)` — stored |
| `Tag` | `object?` | `null` | `tag(t)` / `tag_ref::<U>()` |

### Deliberately not ported

| MudBlazor | Reason |
|---|---|
| `Style` / `UserAttributes` (both) | No CSS strings, no HTML tag. |
| `Class` resolution (both) | Stored for parity. The radio’s colour comes from `Color` / `UncheckedColor`; a utility-class frame would composite a second fill. |
| Form-validation parameters (`Required`, `Error`, `ErrorText`, `HelperText`, …) | Inherited from `MudFormComponent`; no egui form system. |

`IndeterminateIcon` is inherited from `MudBooleanInput` but **never applies to a radio**: a radio is either checked or unchecked. It is stored so a Blazor call site that sets it compiles, and documented as inert.

`Name` is HTML form semantics; egui has no form submission, so it is stored and documented as inert.

---

## Architecture

```
inputs/radio/
├── mud_radio.rs
│   ├── constants        ICON_SMALL 18, ICON_MEDIUM 20, ICON_LARGE 24,
│   │                    GAP 8, DENSE_GAP 4, RIPPLE_DURATION 0.35,
│   │                    DISABLED_OPACITY 0.38, UNCHECKED_OUTLINE_ALPHA 160
│   ├── pure helpers     icon_side, is_vertical, label_leads,
│   │                    layout_size, icon_rect, label_rect, measure_label,
│   │                    with_alpha
│   ├── MudRadio<T>      builder + show(ui, &mut Option<T>)
│   ├── RadioResponse    response + selected
│   └── ripple           start_ripple, paint_ripple
└── mud_radio_group.rs
    ├── MudRadioGroup<T> builder + show(ui, |g| ..)
    ├── RadioGroupUi<T>  collector: radio / radio_full
    └── RadioGroupResponse  response
```

**Pure layer.** Icon size, layout geometry, placement helpers and label measurement are free functions over `f32` and `Placement`. None touch `egui::Context`.

**Collector layer.** `RadioGroupUi` records `MudRadio` values; the group renders them in a wrapping row after the closure returns.

**Component layer.** `MudRadio::show` measures the label, allocates the row, applies a click or `Space`/`Enter`, paints the ripple and the icon, and draws the label through `MudText`.

---

## API reference — `MudRadioGroup`

| Method | Type | Default | Mirrors | Description |
|---|---|---|---|---|
| `new(selected)` | `&'a mut Option<T>` | — | `Value` | Binds the group to a selection. |
| `disabled(b)` | `bool` | `false` | `Disabled` | Disables every radio. |
| `read_only(b)` | `bool` | `false` | `ReadOnly` | Prevents selection without dimming. |
| `name(s)` | `impl Into<String>` | `None` | `Name` | Stored, inert. |
| `classes(iter)` | `iter of AsRef<str>` | none | `Class` | Stored on `ComponentBase`. |
| `tag(t)` / `tag_ref::<U>()` | `T: Any + Send + Sync` | `None` | `Tag` | Arbitrary user data. |
| `show(ui, \|g\| ..)` | `&mut Ui, FnOnce(&mut RadioGroupUi<T>) -> R` | — | `ChildContent` | Returns `RadioGroupResponse`. |

### `RadioGroupUi<T>`

| Method | Description |
|---|---|
| `radio(option, label)` | Adds a radio for `option` with `label`. |
| `radio_full(radio)` | Adds a fully configured `MudRadio`. |
| `len()` / `is_empty()` | Number of radios registered so far. |

### `RadioGroupResponse`

| Field | Type | Description |
|---|---|---|
| `response` | `Response` | The group’s row. Deref target. |
| `inner` | `Option<()>` | Reserved; the group has no closure return value. |

---

## API reference — `MudRadio`

| Method | Type | Default | Mirrors | Description |
|---|---|---|---|---|
| `new(option)` | `T` | — | `Value` | Creates a radio for `option`. |
| `labelled(option, label)` | `(T, impl Into<String>)` | — | `ChildContent` | Convenience constructor. |
| `label(s)` | `impl Into<String>` | `None` | `ChildContent` | Label text. |
| `content(f)` | `FnMut(&mut Ui) + 'static` | `None` | `ChildContent` | Custom label content. |
| `label_placement(p)` | `Placement` | `End` | `LabelPlacement` | `Start` / `End` / `Top` / `Bottom`. |
| `dense(b)` | `bool` | `false` | `Dense` | Compact gap (4 pt). |
| `color(c)` | `Color` | `Primary` | `Color` | Accent when checked. |
| `unchecked_color(c)` | `Color` | `None` | `UncheckedColor` | Outline when unchecked. |
| `size(s)` | `Size` | `Medium` | `Size` | Icon side: 18 / 20 / 24 pt. |
| `ripple(b)` | `bool` | `true` | `Ripple` | Click ripple. |
| `disabled(b)` | `bool` | `false` | `Disabled` | Dims and disables. |
| `read_only(b)` | `bool` | `false` | `ReadOnly` | Disables without dimming. |
| `checked_icon(s)` | `impl Into<String>` | `RadioButtonChecked` | `CheckedIcon` | Checked glyph. |
| `unchecked_icon(s)` | `impl Into<String>` | `RadioButtonUnchecked` | `UncheckedIcon` | Unchecked glyph. |
| `indeterminate_icon(s)` | `impl Into<String>` | `IndeterminateCheckBox` | `IndeterminateIcon` | Stored, never painted. |
| `classes(iter)` | `iter of AsRef<str>` | none | `Class` | Stored. |
| `tag(t)` / `tag_ref::<U>()` | `T: Any + Send + Sync` | `None` | `Tag` | Arbitrary user data. |
| `show(ui, &mut Option<T>)` | `&mut Ui, &mut Option<T>` | — | — | Returns `RadioResponse`. |

### `RadioResponse`

| Field | Type | Description |
|---|---|---|
| `response` | `Response` | The radio’s row. Deref target. |
| `selected` | `bool` | `true` when this radio selected itself this frame. |

### `Size` mapping

| `Size` | Icon side |
|---|---|
| `ExtraSmall` / `Small` | 18 pt |
| `Medium` | 20 pt |
| `Large` / `ExtraLarge` | 24 pt |

### Constants

| Name | Value |
|---|---|
| `ICON_SMALL` / `ICON_MEDIUM` / `ICON_LARGE` | `18.0` / `20.0` / `24.0` |
| `GAP` / `DENSE_GAP` | `8.0` / `4.0` |
| `RIPPLE_DURATION` | `0.35` s |
| `RIPPLE_MAX_RADIUS_FACTOR` | `1.2` |
| `RIPPLE_ALPHA` | `60` |
| `DISABLED_OPACITY` | `0.38` |
| `UNCHECKED_OUTLINE_ALPHA` | `160` |

---

## Examples

### Basic group

```rust
let mut selected: Option<&str> = None;
MudRadioGroup::new(&mut selected).show(ui, |g| {
    g.radio("a", "Option A");
    g.radio("b", "Option B");
});
```

### Disabled radio inside an enabled group

```rust
MudRadioGroup::new(&mut selected).show(ui, |g| {
    g.radio("a", "Available");
    g.radio_full(MudRadio::labelled("b", "Unavailable").disabled(true));
});
```

### Read-only group

```rust
MudRadioGroup::new(&mut selected)
    .read_only(true)
    .show(ui, |g| {
        g.radio("a", "Option A");
        g.radio("b", "Option B");
    });
```

The radios are not dimmed, but clicking them does not change the selection.

### Disabled group

```rust
MudRadioGroup::new(&mut selected)
    .disabled(true)
    .show(ui, |g| {
        g.radio("a", "Option A");
    });
```

### Label before the icon

```rust
g.radio_full(
    MudRadio::labelled("a", "Option A").label_placement(Placement::Start),
);
```

### Custom icons

```rust
g.radio_full(
    MudRadio::labelled("star", "Star")
        .checked_icon(filled::STAR)
        .unchecked_icon(filled::STAR_BORDER)
        .color(Color::Warning),
);
```

### Unchecked outline colour

```rust
g.radio_full(
    MudRadio::labelled("a", "Option A").unchecked_color(Color::Error),
);
```

### Custom label content

```rust
g.radio_full(MudRadio::new("pro").content(|ui| {
    ui.add(MudText::body1("Pro").bold(true));
    ui.add(MudText::caption("$9 / month").color(Color::TextSecondary));
}));
```

### Standalone radios sharing a binding

```rust
let mut selected: Option<&str> = None;
MudRadio::labelled("a", "A").show(ui, &mut selected);
MudRadio::labelled("b", "B").show(ui, &mut selected);
```

### Tag round-trip

```rust
let mut selected: Option<&str> = None;
let group = MudRadioGroup::new(&mut selected).tag("plan");
assert_eq!(group.tag_ref::<&str>(), Some(&"plan"));
```

---

## Binding model

The group owns `&mut Option<T>`. A radio is checked when
`selected.as_ref() == Some(&radio.option)`; clicking or pressing `Space`/`Enter`
on a focused radio writes `Some(option.clone())`.

```rust
let mut selected: Option<String> = None;
MudRadioGroup::new(&mut selected).show(ui, |g| {
    g.radio("small".to_string(), "Small");
    g.radio("large".to_string(), "Large");
});
```

`ValueChanged` is not a callback; `RadioResponse::selected` is a field on the
frame’s result. This is the same split `MudAlert`, `MudCollapse` and
`MudCheckBox` use.

### Click and keyboard

| Input | Effect |
|---|---|
| Click | Selects the radio (if enabled). |
| `Space` / `Enter` (focused) | Selects the radio (if enabled). |
| `ArrowLeft` / `ArrowRight` | **Not implemented**; use Tab between radios. |

Arrow-key navigation across a radio group is documented as a limitation rather
than half-implemented: it requires tracking focus across sibling widgets, which
egui does not provide out of the box.

---

## Layout and rendering

### Horizontal

```
End:    [ ◉ ][ 8 pt ][ Label … ]
Start:  [ Label … ][ 8 pt ][ ◉ ]
```

### Vertical

```
Top:    [ Label … ]
        [ 4/8 pt ]
        [ ◉ ]
```

The group lays radios out in a wrapping row (`ui.horizontal_wrapped`). Use a
`MudStack` or `MudGrid` around the group if you need a fixed column.

### Paint

| State | Glyph | Colour |
|---|---|---|
| Checked | `CheckedIcon` or `RADIO_BUTTON_CHECKED` | `Color` |
| Unchecked | `UncheckedIcon` or `RADIO_BUTTON_UNCHECKED` | `UncheckedColor`, or `Color` at 160 alpha |

Disabled radios are gamma-multiplied by `DISABLED_OPACITY` (0.38).
`read_only(true)` does **not** dim; it only blocks selection.

### Ripple

A filled circle of radius `icon_side × 0.5 × 1.2` expands from the icon centre
over `RIPPLE_DURATION` (0.35 s), then fades out. The start time is stored in
`egui::Memory` keyed by the radio’s `Response::id`.

---

## Performance and memory choices

| Choice | Effect |
|---|---|
| Theme fetched once per radio | One `MudTheme::fetch` and up to two `Color::resolve` calls per radio. |
| Pure helpers on `Copy` values | Icon size, layout and placement are arithmetic; unit-testable without a `Context`. |
| No `Vec` or `String` allocation in the paint path | The label is measured with `layout_no_wrap` and moved into `MudText`. |
| Collector pattern | Radios are collected before rendering, so the group can wrap them without measuring twice. |
| Ripple in `egui::Memory` | No state on the radio; the builder stays stateless. |
| Disabled path skips click and keyboard | `Sense::hover()` for disabled and read-only; no interaction registration. |

---

## Testing

`cargo test -p mud_egui_components mud_radio` runs the module’s pure-function
tests:

| Area | Tests |
|---|---|
| `MudRadio` metrics | icon size per `Size`; placement helpers |
| `MudRadio` layout | `layout_size` horizontal / vertical |
| `MudRadio` builders | defaults; every field set |
| `MudRadioGroup` builders | defaults; every field set; tag round-trip |

No test creates an `egui::Context`. `show` needs a live `Ui` and is left to
integration tests.

Suggested integration tests:

- Clicking an unchecked radio selects it and deselects the previously selected
  radio.
- Clicking the already-selected radio is a no-op.
- A disabled radio ignores clicks; a disabled group ignores every click.
- `read_only(true)` blocks selection without dimming.
- `Space` / `Enter` on a focused radio selects it.
- Custom `checked_icon` / `unchecked_icon` are painted.

```bash
cargo test -p mud_egui_components mud_radio
```

---

## Differences from MudBlazor

- **`Value` / `ValueChanged` become a caller-owned `&mut Option<T>` plus
  `RadioResponse::selected`.** The caller owns the value; the response reports
  the frame it moved. This is the same split `MudCheckBox` and `MudCollapse` use.
- **The group uses the collector pattern**, the same as `MudToggleGroup`. In
  Blazor, `MudRadioGroup` is a container with `ChildContent`; in egui, a closure
  that receives `RadioGroupUi` is the idiomatic equivalent and lets the group
  wrap the radios without measuring twice.
- **`IndeterminateIcon` is stored but never painted.** A radio is binary;
  MudBlazor exposes the parameter because `MudRadio` inherits from
  `MudBooleanInput`, not because it has a visual meaning for radios.
- **`Name` is stored but inert.** HTML form submission has no egui equivalent.
- **Arrow-key navigation is not implemented.** Tab moves between radios; `Space`
  and `Enter` select the focused radio. Full roving-tabindex arrow navigation
  would need cross-widget focus tracking that egui does not provide.
- **The group lays radios out in a wrapping row.** MudBlazor’s radios are
  `inline-block`, which flows the same way. Wrap the group in a `MudStack` or
  `MudGrid` for a fixed column.
- **`Size` folds `ExtraSmall` into `Small` and `ExtraLarge` into `Large`.**
  Consistent with `MudCheckBox` and `MudSwitch`.
- **The default `Color` is `Primary`, not `Default`.** MudBlazor’s
  `MudBooleanInput` default resolves to `Primary` for this component; pinning it
  makes the default visible in the builder table.