# MudChip / MudChipSet for `mud_egui`

A compact, selectable, closable token and its container, ported from
MudBlazor's [`MudChip<T>`](https://mudblazor.com/components/chip) and
[`MudChipSet<T>`](https://mudblazor.com/components/chipset) (MudBlazor
8.13.0), for [egui](https://github.com/emilk/egui).

```rust
let mut set = MudChipSet::new()
    .selection_mode(SelectionMode::Multi)
    .color(Color::Primary)
    .check_mark(true)
    .all_closable(true);

set.show(ui, |ctx| {
    ctx.chip(MudChip::new("Rust").value("rs"));
    ctx.chip(MudChip::new("Go").value("go"));
    ctx.chip(MudChip::new("Zig").value("zig"));
});
```

Every label renders through **`MudText`**, every glyph through **`MudIcon`**.

---

## Contents

1. [Design principle: API parity, not API invention](#design-principle-api-parity-not-api-invention)
2. [Architecture](#architecture)
3. [API reference — MudChip](#api-reference--mudchip)
4. [API reference — MudChipSet](#api-reference--mudchipset)
5. [Examples](#examples)
6. [Selection and cascade semantics](#selection-and-cascade-semantics)
7. [Performance and memory choices](#performance-and-memory-choices)
8. [Testing](#testing)
9. [Differences from MudBlazor](#differences-from-mudblazor)

---

## Design principle: API parity, not API invention

The MudBlazor 8.13.0 parameter surface for `MudChip<T>` and `MudChipSet<T>`
was verified against DNDocs before this port.

### Ported `MudChip` parameters

| MudBlazor | Rust method | Default |
|-----------|-------------|---------|
| `CheckMark` | `check_mark(bool)` | `false` |
| `CheckedIcon` | `checked_icon(s)` | `filled::CHECK` |
| `ChildContent` | `new(s)` / `text(s)` | required |
| `CloseIcon` | `close_icon(s)` | `filled::CANCEL` |
| `Color` | `color(Color)` | `Color::Default` |
| `Disabled` | `disabled(bool)` | `false` |
| `Icon` | `icon(s)` | `None` |
| `IconColor` | `icon_color(Color)` | `Color::Inherit` |
| `Label` | `label(bool)` | `false` |
| `OnClick` | `on_click(f)` | `None` |
| `OnClose` | `on_closed(f)` | `None` |
| `Selected` | `selected(bool)` | `false` |
| `SelectedChanged` | `on_selected_changed(f)` | `None` |
| `SelectedColor` | `selected_color(Color)` | `None` |
| `Size` | `size(Size)` | `Size::Medium` |
| `Variant` | `variant(Variant)` | `Variant::Filled` |
| `Class` (base) | `classes(iter)` | `None` |
| `Id` (base) | `id(s)` | `None` |
| `Style` (base) | *(use `classes`)* | — |
| `UserAttributes` | **not ported** (no HTML tag) | — |

### Extensions (not in MudBlazor)

| Extension | Description |
|-----------|-------------|
| `value(s)` | Stable selection key (MudBlazor's `Value` is generic; the port uses `String`). |
| `closable(bool)` | Explicit close-button toggle. MudBlazor derives it from `OnClose.HasDelegate`. |
| `elevation(u8)` | Drop-shadow depth (0–24). Not a `MudChip` parameter. |
| `aria_label(s)` | Accessible label. |
| `flavor(StyleFlavor)` | Bootstrap or Tailwind resolver. |

### Ported `MudChipSet` parameters

| MudBlazor | Rust method | Default |
|-----------|-------------|---------|
| `SelectionMode` | `selection_mode(SelectionMode)` | `Single` |
| `SelectedValue` | `selected_value(s)` / field | `None` |
| `SelectedValues` | `selected_values(iter)` / field | `{}` |
| `Mandatory` | `mandatory(bool)` | `false` |
| `Disabled` | `disabled(bool)` | `false` |
| `ReadOnly` | `read_only(bool)` | `false` |
| `AllClosable` | `all_closable(bool)` | `false` |
| `Color` | `color(Color)` | `None` |
| `IconColor` | `icon_color(Color)` | `None` |
| `SelectedColor` | `selected_color(Color)` | `None` |
| `Variant` | `variant(Variant)` | `None` |
| `Size` | `size(Size)` | `None` |
| `Label` | `label(bool)` | `None` |
| `CheckMark` | `check_mark(bool)` | `false` |
| `CheckedIcon` | `checked_icon(s)` | `None` |
| `CloseIcon` | `close_icon(s)` | `None` |
| `OnSelectedValueChanged` | `on_selected_value_changed(f)` | `None` |
| `OnSelectedValuesChanged` | `on_selected_values_changed(f)` | `None` |
| `OnClose` | `on_close(f)` | `None` |

### Deliberately not ported

- **`Comparer`** — Rust `PartialEq` on the key is the equivalent.
- **`Href` / `Target` / `ForceLoad` / `Rel`** on `MudChip` — use
  [`MudLink`](../link/README.md) for hyperlink behaviour. The port keeps
  `MudChip` as a purely interactive token.
- **`Style` / `UserAttributes`** — no CSS strings or HTML tags in egui.

---

## Architecture

```
mud_chip.rs
├── Pure helpers
│   ├── with_alpha(c, alpha)
│   ├── resolve_color(c, theme, default)
│   ├── effective_variant(selected, variant)
│   └── chip_text_color(disabled, variant, accent, palette)
├── ChipMetrics
│   ├── resolve(size, label)
│   └── close_icon_size()
├── MudChip       — struct + builders + show
└── mod tests     — 20+ pure-function tests

mud_chipset.rs
├── MudChipSet    — struct + builders + show
├── MudChipSetCtx — collector with `chip(..)` helper
└── mod tests     — 6 pure-function tests
```

**Pure layer.** `with_alpha`, `resolve_color`, `effective_variant`,
`chip_text_color`, `ChipMetrics::resolve`, and `ChipMetrics::close_icon_size`
are free functions or methods that do not touch `egui::Context` and are
covered by unit tests.

**State layer.** `MudChipSet` is caller-owned; its fields are the equivalent
of MudBlazor's two-way-bound `SelectedValue` / `SelectedValues`.

**Component layer.** `MudChip::show` measures the label, allocates the chip
rect, paints the background + border + focus ring + elevation, then paints
the leading icon (`MudIcon`), label (`MudText`), and close button (`MudIcon`).

---

## API reference — `MudChip`

| Method | Type | Default | Description |
|--------|------|---------|-------------|
| `new(text)` | `impl Into<String>` | — | Full constructor. |
| `text(s)` | `impl Into<String>` | `""` | Label text. |
| `value(s)` | `impl Into<String>` | `None` | Selection key. |
| `selected(b)` | `bool` | `false` | Selected state. |
| `disabled(b)` | `bool` | `false` | Disabled state. |
| `color(c)` | `Color` | `Default` | Accent colour. |
| `icon_color(c)` | `Color` | `Inherit` | Leading-icon colour. |
| `selected_color(c)` | `Color` | `None` | Accent override when selected. |
| `variant(v)` | `Variant` | `Filled` | Base variant. |
| `size(s)` | `Size` | `Medium` | Size preset. |
| `label(b)` | `bool` | `false` | Small-radius chip. |
| `icon(s)` | `impl Into<String>` | `None` | Leading icon. |
| `checked_icon(s)` | `impl Into<String>` | `CHECK` | Check glyph. |
| `close_icon(s)` | `impl Into<String>` | `CANCEL` | Close-button glyph. |
| `check_mark(b)` | `bool` | `false` | Show check when selected. |
| `closable(b)` | `bool` | `false` | Show close button. |
| `elevation(e)` | `u8` | `1` / `3` | Drop shadow (filled only). |
| `id(s)` | `impl Into<String>` | `None` | Stable identity. |
| `aria_label(s)` | `impl Into<String>` | `None` | Accessible label. |
| `classes(iter)` | `iter of AsRef<str>` | none | Utility classes. |
| `flavor(f)` | `StyleFlavor` | `Bootstrap` | Utility resolver. |
| `on_click(f)` | `FnMut() + 'static` | `None` | Body-click callback. |
| `on_closed(f)` | `FnMut() + 'static` | `None` | Close callback. |
| `on_selected_changed(f)` | `FnMut(bool) + 'static` | `None` | Selection callback. |
| `show(ui)` | `&mut Ui` | — | Returns `(Response, bool)`. |

### `show` return value

```rust
let (body_resp, close_clicked) = chip.show(ui);
if body_resp.clicked() { /* chip body clicked */ }
if close_clicked { /* close button clicked */ }
```

`body_resp` is the chip body's hit-rect response — **not** the wrapper
frame's hover-only response, so `.clicked()` reflects real clicks.

---

## API reference — `MudChipSet`

| Method | Type | Default | Description |
|--------|------|---------|-------------|
| `new()` | — | — | Default set (`Single` mode). |
| `selection_mode(m)` | `SelectionMode` | `Single` | `Single` / `Toggle` / `Multi`. |
| `selected_value(s)` | `impl Into<String>` | `None` | Single-selection value. |
| `selected_values(iter)` | `iter of Into<String>` | `{}` | Multi-selection values. |
| `mandatory(b)` | `bool` | `false` | Prevents deselecting the last chip. |
| `disabled(b)` | `bool` | `false` | Disables all chips. |
| `read_only(b)` | `bool` | `false` | Prevents selection changes. |
| `all_closable(b)` | `bool` | `false` | Cascades `closable` to every chip. |
| `color(c)` | `Color` | `None` | Cascades to `MudChip::color`. |
| `icon_color(c)` | `Color` | `None` | Cascades. |
| `selected_color(c)` | `Color` | `None` | Cascades. |
| `variant(v)` | `Variant` | `None` | Cascades. |
| `size(s)` | `Size` | `None` | Cascades. |
| `label(b)` | `bool` | `None` | Cascades. |
| `check_mark(b)` | `bool` | `false` | Cascades. |
| `checked_icon(s)` | `impl Into<String>` | `None` | Cascades. |
| `close_icon(s)` | `impl Into<String>` | `None` | Cascades. |
| `on_selected_value_changed(f)` | `FnMut(Option<String>)` | `None` | Fired on single-selection change. |
| `on_selected_values_changed(f)` | `FnMut(HashSet<String>)` | `None` | Fired on multi-selection change. |
| `on_close(f)` | `FnMut(String)` | `None` | Fired when a chip closes. |
| `show(ui, \|ctx\| ..)` | `&mut Ui, FnOnce(&mut MudChipSetCtx) -> R` | — | Returns `R`. |

### `MudChipSetCtx`

| Method | Description |
|--------|-------------|
| `chip(chip)` | Cascades defaults, renders, handles selection/close. Returns `true` if state changed. |
| `selected_value()` | `Option<&String>` |
| `selected_values()` | `&HashSet<String>` |
| `set` / `ui` | Public fields exposing the underlying state and `Ui`. |

---

## Examples

### Basic chip

```rust
ui.add(MudChip::new("Rust"));
```

### With icon and closable

```rust
MudChip::new("Filter: active")
    .icon(filled::FILTER_ALT)
    .closable(true)
    .on_closed(|| println!("closed"))
    .show(ui);
```

### Selected with check mark

```rust
MudChip::new("Selected")
    .selected(true)
    .check_mark(true)
    .show(ui);
```

### Outlined / text variants

```rust
MudChip::new("Outlined").variant(Variant::Outlined).show(ui);
MudChip::new("Text").variant(Variant::Text).show(ui);
```

### Label style (small radius)

```rust
MudChip::new("LABEL").label(true).show(ui);
```

### Sizes

```rust
MudChip::new("Small").size(Size::Small).show(ui);
MudChip::new("Medium").size(Size::Medium).show(ui);
MudChip::new("Large").size(Size::Large).show(ui);
```

### Chip set — single selection (default)

```rust
let mut set = MudChipSet::new();
set.show(ui, |ctx| {
    for tag in ["all", "active", "done"] {
        ctx.chip(MudChip::new(tag).value(tag));
    }
});
// After the frame, `set.selected_value` holds the clicked tag.
```

### Chip set — multi selection

```rust
let mut set = MudChipSet::new().selection_mode(SelectionMode::Multi);
set.show(ui, |ctx| {
    for tag in ["rust", "go", "zig"] {
        ctx.chip(MudChip::new(tag));
    }
});
for tag in &set.selected_values {
    println!("selected: {tag}");
}
```

### Chip set — toggle with mandatory

```rust
let mut set = MudChipSet::new()
    .selection_mode(SelectionMode::Toggle)
    .mandatory(true)
    .selected_value("default");

set.show(ui, |ctx| {
    for tag in ["default", "custom"] {
        ctx.chip(MudChip::new(tag).value(tag));
    }
});
```

Clicking the currently selected chip does **not** clear the selection.

### Chip set — cascading defaults

```rust
let mut set = MudChipSet::new()
    .color(Color::Secondary)
    .size(Size::Small)
    .check_mark(true)
    .all_closable(true);

set.show(ui, |ctx| {
    // Every chip inherits color=Secondary, size=Small, check_mark, closable.
    ctx.chip(MudChip::new("A"));
    // Override per-chip:
    ctx.chip(MudChip::new("B").color(Color::Error));
});
```

### Chip set — change callbacks

```rust
let mut set = MudChipSet::new()
    .on_selected_value_changed(|value| {
        log::info!("selected: {value:?}");
    })
    .on_close(|value| {
        log::info!("closed chip: {value}");
    });
```

---

## Selection and cascade semantics

### Selection modes

| Mode | Click unselected | Click selected | `mandatory` interaction |
|------|------------------|----------------|-------------------------|
| `Single` | selects it | replaces the selection | `mandatory` does **not** apply (there is always a selection) |
| `Toggle` | selects it | deselects it | `mandatory(true)` prevents deselecting the last chip |
| `Multi` | adds it | removes it | `mandatory` does not apply |

`read_only(true)` freezes selection but still fires `on_click` and
`on_closed` — matching MudBlazor's `ReadOnly` semantics.

### Cascading defaults

`MudChipSetCtx::chip` fills in the chip's optional fields from the set when
they are `None` / `false`. The precedence is:

```
chip.field  >  set.field  >  MudChip default
```

| Field | Chip-set → chip cascade |
|-------|-------------------------|
| `color` | When `chip.color.is_none()` |
| `icon_color` | When `chip.icon_color.is_none()` |
| `selected_color` | When `chip.selected_color.is_none()` |
| `variant` | When `chip.variant.is_none()` |
| `size` | When `chip.size.is_none()` |
| `label` | When `chip.label.is_none()` |
| `check_mark` | When `chip.show_check_mark == false` |
| `checked_icon` | When `chip.checked_icon.is_none()` |
| `close_icon` | When `chip.close_icon.is_none()` |
| `closable` | When `chip.closable == false` |
| `disabled` | OR-ed with `set.disabled` |

### Callback ordering

For one chip-set frame:

1. Chip body click → `chip.on_click` (if any) → chip internal toggle →
   `chip.on_selected_changed` (if any, with the new chip-local value).
2. Then the chipset updates `selected_value` / `selected_values` and fires
   `set.on_selected_value_changed` / `set.on_selected_values_changed`.
3. Close button click → `chip.on_closed` → then `set.on_close`.

Chips that live inside a set should not rely on their own
`on_selected_changed` value; use the set's callback instead.

---

## Performance and memory choices

| Choice | Effect |
|--------|--------|
| Label width measured once per frame via `layout_no_wrap` | No text layout twice; the same `FontId` is reused. |
| Single `MudText` for the label | No per-chip copy of the theme font. |
| Pure `ChipMetrics::resolve` | No per-frame `match` on `Size` inside the paint path. |
| `effective_variant` is a pure `match` | Fast, easy to test. |
| `with_alpha` early-outs on `alpha >= 1.0` and `<= 0.0` | Avoids the `from_rgba_unmultiplied` call for identity. |
| `MudChipSet` uses `HashSet<String>` for Multi | O(1) membership; matches MudBlazor's semantics. |
| `MudChipSet::chip` returns `bool` | Cheap to ignore or inspect. |
| Chip's `id` feeds `ui.scope_id().with(id)` | Stable interaction ids; no collisions between chips. |
| Close interaction is scoped to its own rect | Clicking close does not also register as a body click. |

---

## Testing

`cargo test` runs 20+ pure-function tests in `mud_chip.rs`:

| Area | Tests |
|------|-------|
| `with_alpha` | full alpha identity, zero → transparent, fractional scaling |
| `resolve_color` | `Default` / `Inherit` → fallback, `Primary` → palette primary |
| `effective_variant` | unselected → base, selected `Filled↔Text`, `Outlined` unchanged, `None` → `Text` |
| `chip_text_color` | disabled → `DISABLED_ALPHA`, filled → contrast, outlined/text → accent |
| `ChipMetrics` | Small / Medium / Large exact values, label forces radius 4, unknown size → Medium |
| `close_icon_size` | scales and clamps to `CLOSE_ICON_MIN` |
| Defaults | match MudBlazor |
| Builders | every field set, `flavor` |

Plus 6 pure-function tests in `mud_chipset.rs`:

| Area | Tests |
|------|-------|
| Defaults | `selection_mode == Single`, empty state |
| Builders | selection state, cascade defaults |
| `mandatory` | flag is set |
| Cascading | `color` / `size` / `variant` are `Some` |

Rendering needs a live `egui::Context`; the pure layer is the source of
truth.

```bash
cargo test -p mud_egui_components mud_chip
```

---

## Differences from MudBlazor

- **`value` is `String`** rather than generic `T`. MudBlazor's `MudChip<T>`
  is generic; the Rust port uses `String` because `MudChipSet` uses string
  keys for its `HashSet`. If you need typed values, store the mapping
  externally.
- **`closable` is explicit.** MudBlazor shows the close button when
  `OnClose.HasDelegate` is true. The port lets the caller force it with
  `closable(true)`, which is what `MudChipSet::all_closable` needs.
- **`elevation` is an extension.** Not a `MudChip` parameter; the port
  defaults filled chips to 1 on rest and 3 on hover, matching
  `MudBadge`'s scale.
- **`Href` / `Target` / `ForceLoad` / `Rel` not ported.** Use `MudLink`.
- **`Style` → `classes(..)`.** egui has no inline CSS strings.
- **`UserAttributes` not ported.** No HTML tag.
- **`MudChip::show` returns `(Response, bool)`.** The `Response` is the body
  hit-rect (so `.clicked()` reflects clicks) and `bool` is `close_clicked`.
  `ui.add(MudChip::new(..))` still works via the `Widget` impl and discards
  the `close_clicked` flag.