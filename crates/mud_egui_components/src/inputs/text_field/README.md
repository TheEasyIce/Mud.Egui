# MudTextField for `mud_egui`

A themed single-line text field for [egui](https://github.com/emilk/egui),
ported from MudBlazor's `MudTextField<T>` (MudBlazor 8.13.0).

```rust
let mut name = String::new();
let resp = MudTextField::new(&mut name)
    .label("Full name")
    .placeholder("Type here…")
    .helper_text("As it appears on your ID")
    .variant(Variant::Outlined)
    .show(ui);

if resp.changed {
    log::info!("value is now {name:?}");
}
```

The label, helper text, error text, and text-form adornment render through
**`MudText`**; icon-form adornments and the clear button render through
**`MudIcon`**. The editable text itself is rendered by `egui::TextEdit`
(its glyphs are egui's responsibility, exactly as MudBlazor delegates to
`<input>`).

---

## Contents

1. [Design principle: API parity, not API invention](#design-principle-api-parity-not-api-invention)
2. [Pass 1 scope and pass 2 deferral](#pass-1-scope-and-pass-2-deferral)
3. [Architecture](#architecture)
4. [API reference](#api-reference)
5. [Examples](#examples)
6. [Layout and styling](#layout-and-styling)
7. [Performance and memory choices](#performance--memory-choices)
8. [Testing](#testing)
9. [Differences from MudBlazor](#differences-from-mudblazor)

---

## Design principle: API parity, not API invention

MudBlazor's `MudTextField<T>` parameter surface was verified against
[DNDocs](https://docs.dndocs.com/n/MudBlazor/8.13.0/api/MudBlazor.MudTextField-1.html)
before this port. The two-way binding (`Value` / `ValueChanged` /
`ValueExpression`) is unified into a **caller-owned `&'a mut String`** passed
to [`MudTextField::new`] — the same pattern used by `MudTable`,
`MudTreeView`, `MudChipSet`, `MudNavMenu`, `MudPagination`, `MudStepper`,
and `MudTabs`. There is no `ValueChanged` callback; the caller reads the
string after [`show`](MudTextField::show) returns.

`MudTextField` also does not implement `egui::Widget` because a
`Widget::ui(self)` implementation cannot return the mutated `String`
alongside the `Response` that the trait requires. Use `show(ui)` directly.

### Ported parameters (pass 1)

| MudBlazor | Rust method | Default |
|-----------|-------------|---------|
| `Value` / `ValueChanged` | `value: &'a mut String` argument of `new` | required |
| `Label` | `label(s)` | `None` |
| `Placeholder` | `placeholder(s)` | `None` |
| `HelperText` | `helper_text(s)` | `None` |
| `Error` | `error(bool)` | `false` |
| `ErrorText` | `error_text(s)` | `None` |
| `Variant` | `variant(Variant)` | `Text` |
| `Margin` | `margin(Margin)` | `None` |
| `Color` | `color(Color)` | `Primary` |
| `Size` | `size(Size)` | `Medium` |
| `Disabled` | `disabled(bool)` | `false` |
| `ReadOnly` | `readonly(bool)` | `false` |
| `Clearable` | `clearable(bool)` | `false` |
| `Adornment` + `AdornmentIcon` + `AdornmentText` | `adornment_end(AdornmentEnd)` | `None` |
| `AdornmentColor` | `adornment_end_color(Color)` | `None` |
| `OnAdornmentClick` | `on_adornment_click(f)` | `None` |
| `Class` (base) | `classes(iter)` | `None` |
| `Style` (base) | *(use `classes`)* | — |
| `Tag` (base) | **not ported** (no HTML tag) | — |
| `UserAttributes` | **not ported** | — |

### Extensions (not in MudBlazor)

| Extension | Description |
|-----------|-------------|
| `id(Id)` | Stable interaction id salt. |
| `aria_label(s)` | Accessible label overriding the caption. |
| `clear_icon(s)` | Glyph used by the clear button. |
| `TextFieldResponse` | Rich return value with `changed` / `lost_focus` / `has_focus` / `submitted`. |

---

## Pass 1 scope and pass 2 deferral

Pass 1 ships everything the **overlay family** needs to render an
input-with-a-dropdown-chevron on top of it: label, helper/error, all three
variants, all three margin values, all five sizes, adornment-end as icon or
text, clear button, focus ring, hover state, and the `TextFieldResponse`
with `changed` / `lost_focus` / `has_focus` / `submitted`.

The following MudBlazor parameters are **deferred to pass 2** and are
documented here so the deferral is visible:

| Deferred parameter | Why pass 2 |
|--------------------|------------|
| `InputType` | Password masking, email validation display, number formatting are their own concern; they do not affect overlay rendering. |
| `Mask` | Character-by-character input masking is a substantial subsystem. |
| `Lines` / `AutoGrow` / `MaxLines` | Multiline layout changes the whole field height model. |
| `Required` / `RequiredError` | Form-level validation display; sits on top of `error` / `error_text`. |
| `Counter` / `MaxLength` | Counter goes in the helper row; needs the helper-row rework. |
| `DebounceInterval` / `Immediate` | Debouncing `changed` requires timer state in `egui::Memory` — fine to add later. |
| `AdornmentStart` | Leading adornment is a mirror of the trailing one; nothing structural. |
| Floating-label-with-border-cutout | MudBlazor draws the label *into* the border when focused/valued. The pass-1 renderer draws it as a static caption above the field. Visually close, structurally simpler. |
| `Typo` on the input body | Applies a typography scale to the `TextEdit`; nothing structural. |

These deferrals do not block `MudSelect`, `MudAutocomplete`, or any picker.
They will be needed by fully-fledged forms, but the forms layer is a separate
work item.

### Deliberately not ported

- **`Tag` / `UserAttributes`** — no HTML tag.
- **`Style`** — no inline CSS strings; use `classes(..)`.
- **`For` / `Validation` / `Validation`** — this is a form-level concept;
  the port uses `error` / `error_text` directly.

---

## Architecture

```
text_field/
├── adornment_end.rs   — AdornmentEnd enum (Icon | Text)
├── mud_text_field.rs  — MudTextField + TextFieldResponse + pure helpers
└── mod.rs             — re-exports

mud_egui_core/src/enums/
└── margin.rs          — Margin enum (None | Dense | Normal)
```

**Pure layer.** `height_for`, `font_for`, `padding_for`, `resolve_color`,
`with_alpha` — free functions, unit-tested without `egui::Context`.

**State layer.** The caller owns `&'a mut String` — the field writes to it
in place, and no separate state struct is needed.

**Component layer.** `show` resolves the theme and class frame, then
`render_inner` draws the label, the field body, the `TextEdit`, the trailing
adornment, the clear button, and the helper/error row.

---

## API reference

### `MudTextField<'a>`

| Method | Type | Default | Description |
|--------|------|---------|-------------|
| `new(value)` | `&'a mut String` | — | Field editing `value` in place. |
| `label(s)` | `impl Into<String>` | `None` | Caption above the field. |
| `placeholder(s)` | `impl Into<String>` | `None` | Hint shown when empty. |
| `helper_text(s)` | `impl Into<String>` | `None` | Below the field. |
| `error(bool)` | `bool` | `false` | Switch accent to error colour. |
| `error_text(s)` | `impl Into<String>` | `None` | Shown instead of `helper_text` when `error` is `true`. |
| `variant(Variant)` | `Variant` | `Text` | `Text` (underline) / `Filled` (grey bg) / `Outlined` (border). |
| `margin(Margin)` | `Margin` | `None` | `None` / `Dense` (−4 px) / `Normal` (+4 px). |
| `color(Color)` | `Color` | `Primary` | Focus / underline accent. |
| `size(Size)` | `Size` | `Medium` | 32 / 40 / 48 px field height. |
| `disabled(bool)` | `bool` | `false` | Prevents interaction. |
| `readonly(bool)` | `bool` | `false` | Prevents editing but keeps visuals. |
| `clearable(bool)` | `bool` | `false` | Shows a clear button when non-empty. |
| `clear_icon(s)` | `impl Into<String>` | `CLEAR` | Glyph used by the clear button. |
| `adornment_end(AdornmentEnd)` | `AdornmentEnd` | `None` | Trailing icon or text. |
| `adornment_end_color(Color)` | `Color` | `None` | Adornment colour override. |
| `on_adornment_click(f)` | `FnMut()` | `None` | Fires when the adornment is clicked. |
| `id(Id)` | `impl Hash` | `None` | Stable interaction id salt. |
| `aria_label(s)` | `impl Into<String>` | `None` | Accessible label; falls back to `label`. |
| `classes(iter)` | `iter of AsRef<str>` | none | Wrapper utility classes. |
| `flavor(StyleFlavor)` | `StyleFlavor` | `Bootstrap` | Utility resolver. |
| `show(ui)` | `&mut Ui` | — | Returns a `TextFieldResponse`. |

### `TextFieldResponse`

| Field | Type | Description |
|-------|------|-------------|
| `response` | `egui::Response` | Union of the field rect and the inner `TextEdit`. |
| `changed` | `bool` | The user edited the value this frame. |
| `lost_focus` | `bool` | The field lost focus this frame. |
| `has_focus` | `bool` | The field has focus. |
| `submitted` | `bool` | Enter was pressed while focused. |

### `AdornmentEnd`

| Variant | Description |
|---------|-------------|
| `Icon(String)` | SVG glyph. |
| `Text(String)` | Short text label (a unit, a hint). |

Constructors: `AdornmentEnd::icon(s)`, `AdornmentEnd::text(s)`.
Predicates: `is_icon()`, `content()`.

### `Margin`

| Variant | Effect |
|---------|--------|
| `None` (default) | Standard field height. |
| `Dense` | −4 px vertical. |
| `Normal` | +4 px vertical. |

### `Variant` (from `mud_egui_core::enums`)

| Variant | Visual |
|---------|--------|
| `Text` (default) | Underline only. |
| `Filled` | Grey background, underline. |
| `Outlined` | Full 1 px border (2 px when focused). |

---

## Examples

### Basic

```rust
let mut name = String::new();
MudTextField::new(&mut name).show(ui);
```

### With label and helper text

```rust
MudTextField::new(&mut email)
    .label("Email")
    .helper_text("We'll never share it.")
    .show(ui);
```

### Outlined variant

```rust
MudTextField::new(&mut name)
    .variant(Variant::Outlined)
    .label("Name")
    .show(ui);
```

### Filled variant

```rust
MudTextField::new(&mut name)
    .variant(Variant::Filled)
    .show(ui);
```

### Dense margin

```rust
MudTextField::new(&mut value)
    .margin(Margin::Dense)
    .show(ui);
```

### Error state

```rust
MudTextField::new(&mut email)
    .label("Email")
    .error(!is_valid_email(&email))
    .error_text("Please enter a valid email.")
    .show(ui);
```

### Clear button

```rust
let resp = MudTextField::new(&mut query)
    .clearable(true)
    .placeholder("Search…")
    .show(ui);
if resp.changed {
    /* query has been updated (or cleared) */
}
```

### Trailing icon adornment (the pattern `MudSelect` will use)

```rust
let mut selected = "Choose…".to_string();
MudTextField::new(&mut selected)
    .label("Country")
    .adornment_end(AdornmentEnd::icon(filled::ARROW_DROP_DOWN))
    .on_adornment_click(|| self.select_open = true)
    .readonly(true) // the field is a trigger, not free text
    .show(ui);
```

### Trailing text adornment (a unit)

```rust
MudTextField::new(&mut weight)
    .label("Weight")
    .adornment_end(AdornmentEnd::text("kg"))
    .show(ui);
```

### Read-only display

```rust
MudTextField::new(&mut id_display)
    .label("Account ID")
    .readonly(true)
    .variant(Variant::Outlined)
    .show(ui);
```

### Submitting on Enter

```rust
let resp = MudTextField::new(&mut query).show(ui);
if resp.submitted {
    run_search(&query);
}
```

### Accessible labelling

```rust
MudTextField::new(&mut amount)
    .aria_label("Transaction amount in US dollars")
    .show(ui);
```

---

## Layout and styling

### Dimensions

| `Size` | Height | Font | Horizontal padding |
|--------|--------|------|--------------------|
| `Small` | 32 px | 12 pt | 8 px |
| `Medium` | 40 px | 14 pt | 12 px |
| `Large` | 48 px | 16 pt | 16 px |

`Margin::Dense` subtracts 4 px; `Margin::Normal` adds 4 px.

### Adornment / clear-button widths

- `AdornmentEnd::Icon` — 50% of field height.
- `AdornmentEnd::Text` — measured text width + 4 px.
- Clear button (only when `adornment_end` is `None`) — 42% of field height + 4 px.
- The edit area is `field_rect` minus horizontal padding on each side, minus
  the adornment width (or clear-button width, whichever is present).

### Focus ring

- **Outlined** variant: the whole border switches to the accent colour and
  grows to 2 px.
- **Text** / **Filled** variants: the underline switches to the accent colour
  and grows to 2 px.

Focus is tracked from `egui::Memory` (previous frame), because
`TextEdit`'s internal id lives there. This is the standard pattern for
composing custom chrome around `TextEdit` and is documented in the source.

### Click-to-focus anywhere

Clicking anywhere in the field box — including the padding and the empty
adornment area — requests focus on the inner `TextEdit`. This matches
MudBlazor, where the whole `<MudInput>` wrapper is clickable.

### Hover state

- The underline / border darkens slightly on hover.
- The trailing adornment and clear button switch to a pointing-hand cursor.

---

## Performance and memory choices

| Choice | Effect |
|--------|--------|
| `height_for` / `font_for` / `padding_for` are pure | No `egui::Context`; unit-tested. |
| `MudTextField` holds `&'a mut String` | Zero-allocation; the caller owns the buffer. |
| `TextEdit` runs with `frame(false)` | The port paints its own frame; `TextEdit` handles only the glyphs and editing. |
| `TextEdit` runs with `margin(ZERO)` | No internal padding, so the padding the port paints is authoritative. |
| Focus state read from `egui::Memory` | One-frame lag but no plumbing needed and no duplicated state. |
| `clip_rect` set on the edit child | Prevents the input text from spilling over the adornment. |
| `push_id(edit_id, …)` on the `TextEdit` | Stable focus tracking across frames even if the field moves in layout. |
| Label / helper / error rendered through `MudText` | Theme and typography propagate; no per-field font configuration. |
| Adornment and clear glyphs rendered through `MudIcon` | Free thanks to the icon transform cache. |
| `clone()` on `placeholder` for `.hint_text(..)` | One small allocation per frame; required because `hint_text` takes `impl Into<WidgetText>` and `Option<&String>` does not implement it. Could be avoided with `as_deref()` but the ergonomics suffer. |

---

## Testing

`cargo test` runs 15 pure-function tests:

| Area | Tests |
|------|-------|
| `height_for` | every `Size` × `Margin` combination, unknown `Size` fallback, extreme-dense clamp |
| `font_for` | every `Size` |
| `padding_for` | every `Size` |
| Defaults | every `MudTextField` field matches MudBlazor |
| Builders | every field set |
| `AdornmentEnd` | `icon` / `text` constructors, `is_icon`, `content` |
| `resolve_color` | Default / Inherit fallback, explicit palette mapping |
| `with_alpha` | identity, transparent, fractional scaling |

Rendering (`show`) needs a live `egui::Context`; the pure layer is the source
of truth.

```bash
cargo test -p mud_egui_components mud_text_field
cargo test -p mud_egui_core margin
```

---

## Differences from MudBlazor

- **Value is caller-owned (`&'a mut String`).** MudBlazor uses the bindable
  `Value` / `ValueChanged` / `ValueExpression` triple; the egui equivalent
  is a mutable reference the caller owns and passes to `new`. Read the
  string after `show` returns to observe changes.
- **No `Widget` impl.** `MudTextField` cannot implement `egui::Widget`
  cleanly because `Widget::ui(self)` cannot return the mutated `String`
  alongside the `Response`.
- **Label rendered as a static caption above the field.** MudBlazor's label
  *floats into* the border when the field is focused or has a value. Pass 2
  will implement the floating label with the border cutout; pass 1 draws
  the caption above the field for structural simplicity. This is the only
  visible deviation and it does not affect the overlay family (whose labels
  sit above the field anyway).
- **Focus tracking has one frame of lag.** Because `TextEdit`'s id lives in
  `egui::Memory` and is only known after the `TextEdit` has run at least
  once, the focus ring follows the previous frame's focus state. In practice
  the lag is invisible.
- **`Variant` defaults to `Text`.** This matches MudBlazor's
  `MudTextField` default (`MudBlazor.Variant.Text`). Note that this differs
  from `MudButton`'s default (`MudBlazor.Variant.Text` too but visually
  different in that component's context).
- **`Margin::None` is the default.** Matches MudBlazor.
- **`InputType` / `Mask` / `Lines` / `AutoGrow` / `Required` / `Counter` /
  `DebounceInterval` / `AdornmentStart` / floating-label-with-cutout /
  `Typo` are deferred to pass 2.** See [Pass 1 scope and pass 2
  deferral](#pass-1-scope-and-pass-2-deferral).
- **`Tag` / `UserAttributes` not ported.** No HTML tag.
- **`Style` → `classes(..)`.** egui has no inline CSS strings.
- **`For` / `Validation` not ported.** Form-level concepts; the port uses
  `error` / `error_text` directly.