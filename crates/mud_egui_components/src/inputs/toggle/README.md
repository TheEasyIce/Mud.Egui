# MudToggleGroup for egui

A production-grade, MudBlazor‑faithful port of `MudToggleGroup<T>` and
`MudToggleItem<T>` (MudBlazor 8.13.0) for the [`egui`](https://github.com/emilk/egui)
immediate-mode GUI library.

The component reproduces MudBlazor’s segmented-control aesthetic—shared
outline, square inner corners, accent under-fill, check-mark support, ripple,
and precise size presets—while remaining allocation-light and idiomatic Rust.

---

## Table of contents

1. [Architecture](#architecture)
2. [API reference](#api-reference)
   - [`MudToggleGroup`](#mudtogglegroup)
   - [`MudToggleItem`](#mudtoggleitem)
   - [`SelectionMode`](#selectionmode)
   - [`Size` and `Color`](#size-and-color)
3. [Real-world examples](#real-world-examples)
   - [Single selection (radio-like)](#single-selection-radio-like)
   - [Multi selection (checkbox-like)](#multi-selection-checkbox-like)
   - [Toggle selection (deselectable)](#toggle-selection-deselectable)
   - [Vertical group with custom content](#vertical-group-with-custom-content)
   - [Check marks and fixed content](#check-marks-and-fixed-content)
4. [Performance & design rationale](#performance--design-rationale)
5. [Visual parity notes](#visual-parity-notes)
6. [Testing](#testing)
7. [License](#license)

---

## Architecture

```
mud_toggle_group.rs
├── MudToggleItem<T>        — single-item description (value, text, icons, content)
├── MudToggleGroup          — group configuration and rendering engine
│   ├── show()              — main entry point; collects items, renders, applies click
│   ├── render_item()       — dispatch: custom vs standard
│   ├── render_custom()     — user-provided closure path
│   ├── render_standard()   — MudButton path
│   └── make_button()       — zero-clone button construction
├── ToggleGroupUi<'a, T>    — collector handle passed to the show() closure
├── Pure helpers            — sizing, corners, selection, icons, theme decisions
└── Ripple subsystem        — memory-backed, per-item expanding-circle animation
```

The component follows the same **per-frame temporary** model as MudBlazor’s
render tree: items are collected into a `Vec` on every frame, rendered once,
and dropped. The selection itself is not owned by the group; it lives in the
caller’s `Vec<T>`, which is the egui equivalent of Blazor’s `Value` /
`Values` two-way binding.

---

## API reference

### `MudToggleGroup`

| Method / field | Type | Default | MudBlazor equivalent | Description |
|---|---|---|---|---|
| `new()` | `Self` | — | — | Creates a group with defaults. |
| `selection_mode(m)` | `SelectionMode` | `Single` | `SelectionMode` | Selection behaviour. |
| `check_mark(b)` | `bool` | `false` | `CheckMark` | Show leading check glyphs. |
| `check_mark_class(s)` | `impl Into<String>` | `None` | `CheckMarkClass` | CSS classes for check icons (parity hook). |
| `selected_class(s)` | `impl Into<String>` | `None` | `SelectedClass` | CSS classes for selected items (parity hook). |
| `fixed_content(b)` | `bool` | `false` | `FixedContent` | Reserve check-glyph space when unselected. |
| `disabled(b)` | `bool` | `false` | `Disabled` | Disable the whole group. |
| `vertical(b)` | `bool` | `false` | `Vertical` | Stack items vertically. |
| `right_to_left(b)` | `bool` | `false` | `RightToLeft` (cascading) | Reverse horizontal order. |
| `delimiters(b)` | `bool` | `true` | `Delimiters` | Draw lines between flat items. |
| `outlined(b)` | `bool` | `true` | `Outlined` | Draw the shared segmented border. |
| `color(c)` | `Color` | `Primary` | `Color` | Accent / border colour. |
| `size(s)` | `Size` | `Medium` | `Size` | Cell height preset. |
| `ripple(b)` | `bool` | `true` | `Ripple` | Enable click ripple. |
| `classes(iter)` | `impl IntoIterator<Item = impl AsRef<str>>` | — | `Class` (base) | Utility classes on `ComponentBase`. |
| `show(ui, &mut Vec<T>, closure)` | `Response` | — | `Value` / `Values` binding | Renders the group and mutates the selection. |

> **MudBlazor parity note:** `FullWidth`, `CornerRadius`, and a plain
> `MudToggleItem::Icon` parameter do **not** exist in MudBlazor 8.13.0 and are
> therefore **not** part of the public API. Layout is content-sized except for
> vertical groups and groups containing custom content, which stretch to the
> available width.

### `MudToggleItem`

| Method / field | Type | Default | MudBlazor equivalent | Description |
|---|---|---|---|---|
| `new(value, text)` | `(T, impl Into<String>)` | — | `Value`, `Text` | Creates an item. |
| `selected_icon(s)` | `impl Into<String>` | `None` (falls back to Material check) | `SelectedIcon` | Glyph when selected and `check_mark` is on. |
| `unselected_icon(s)` | `impl Into<String>` | `None` | `UnselectedIcon` | Glyph when unselected and `check_mark` is on. |
| `disabled(b)` | `bool` | `false` | `Disabled` | Disable this item only. |
| `content(f)` | `FnMut(&mut Ui, bool)` | `None` | `ChildContent` (`RenderFragment<bool>`) | Full custom cell renderer. |

### `SelectionMode`

| Variant | Behaviour |
|---|---|
| `Single` | Exactly one item selected; re-clicking the selected item does nothing. |
| `Toggle` | One item selected; re-clicking the selected item clears the selection. |
| `Multi` | Multiple items; clicking toggles membership. |

### `Size` and `Color`

`Size` maps to MudBlazor’s height presets:

| `Size` | Cell height |
|---|---|
| `ExtraSmall` | 24 px |
| `Small` | 32 px |
| `Medium` | 40 px |
| `Large` | 48 px |
| `ExtraLarge` | 56 px |

`Color` accepts the standard MudBlazor palette (`Primary`, `Secondary`,
`Info`, `Success`, `Warning`, `Error`, `Default`, `Custom`).

---

## Real-world examples

### Single selection (radio-like)

```rust
use mud_egui_components::mud_toggle_group::MudToggleGroup;
use mud_egui_core::enums::{Color, SelectionMode};

let mut alignment: Vec<&'static str> = vec!["left"];

MudToggleGroup::new()
    .selection_mode(SelectionMode::Single)
    .color(Color::Primary)
    .show(ui, &mut alignment, |group| {
        group.item("left", "Left");
        group.item("center", "Center");
        group.item("right", "Right");
    });

// `alignment` is now `vec!["left"]`, `vec!["center"]`, or `vec!["right"]`.
```

### Multi selection (checkbox-like)

```rust
let mut toppings: Vec<String> = vec!["cheese".into()];

MudToggleGroup::new()
    .selection_mode(SelectionMode::Multi)
    .outlined(false)
    .delimiters(true)
    .show(ui, &mut toppings, |group| {
        group.item("cheese".to_string(), "Cheese");
        group.item("ham".to_string(), "Ham");
        group.item("olives".to_string(), "Olives");
    });
```

### Toggle selection (deselectable)

```rust
let mut filter: Vec<&'static str> = vec!["active"];

MudToggleGroup::new()
    .selection_mode(SelectionMode::Toggle)
    .show(ui, &mut filter, |group| {
        group.item("active", "Active");
        group.item("archived", "Archived");
    });
```

Clicking `Active` while it is selected clears `filter` to `vec![]`.

### Vertical group with custom content

```rust
use egui::Widget;

let mut view: Vec<u8> = vec![0];

MudToggleGroup::new()
    .vertical(true)
    .show(ui, &mut view, |group| {
        group.item_full(
            MudToggleItem::new(0, "")
                .content(|ui, selected| {
                    ui.horizontal(|ui| {
                        ui.label("📊");
                        ui.label(if selected { "Bold Chart" } else { "Chart" });
                    });
                }),
        );
        group.item_full(
            MudToggleItem::new(1, "")
                .content(|ui, selected| {
                    ui.horizontal(|ui| {
                        ui.label("📋");
                        ui.label(if selected { "Bold Table" } else { "Table" });
                    });
                }),
        );
    });
```

Custom content closures run twice per frame (measure, then draw) and must be
side-effect-free.

### Check marks and fixed content

```rust
MudToggleGroup::new()
    .check_mark(true)
    .fixed_content(true)
    .show(ui, &mut selected, |group| {
        group.item_full(
            MudToggleItem::new("a", "Option A")
                .selected_icon("✅")
                .unselected_icon("⬜"),
        );
        group.item("b", "Option B"); // uses default check / hidden placeholder
    });
```

With `fixed_content(true)` and no explicit `unselected_icon`, an invisible
check glyph reserves space so the text does not shift when selection changes.

---

## Performance & design rationale

| Technique | Benefit |
|---|---|
| **String moves via `mem::take`** | No `Clone` on the hot path. Items are single-use per frame. |
| **Lazy theme resolution** | `MudTheme::fetch` is called at most once per frame, and only when a border, divider, or selection fill actually needs a colour. |
| **Allocation-free sizing** | `cell_width`, `separator_count`, `display_index`, and corner calculations use saturating integer arithmetic; no temporary `Vec`s. |
| **Conditional height buffer** | Custom-content heights are stored in a reusable `Vec` only when at least one item has custom content. Plain button groups allocate nothing for heights. |
| **Memory-backed ripple** | Ripple start times live in `egui::Memory` keyed by item index. The caller does not need to own ripple state, and the group remains stateless between frames. |
| **Invisible probes** | Measurement runs in a separate `Ui` that never allocates parent space, paints nothing, and registers no interaction. |
| **Finite guards** | `finite_or_zero` prevents NaN/∞ values from unbounded scroll areas from poisoning layout arithmetic. |

---

## Visual parity notes

- **Shared outline** — Outlined groups draw one rounded border around the
  entire group and square inner corners on individual cells.
- **Accent under-fill** — Selected standard buttons receive a square-cornered
  accent fill underneath the button, preventing the button’s rounded fill from
  leaving notches at cell boundaries.
- **Hover overlay** — Custom-content cells receive a subtle white overlay
  (`HOVER_OVERLAY_ALPHA = 10`) on hover, matching MudButton feedback.
- **Focus ring** — Custom-content cells draw a 2 px accent ring when focused.
- **Ripple** — A 350 ms expanding circle is painted on click for both standard
  and custom cells.
- **Typography** — Text rendering is delegated to `MudButton`, which already
  implements MudBlazor’s size and colour hierarchy.
- **Disabled state** — Disabled groups and items suppress clicks, disable
  custom content, and rely on `MudButton`’s muted visuals.

---

## Testing

The source file contains an extensive `#[cfg(test)] mod tests` block covering:

- Size-preset height mapping
- Radius clamping and truncation
- Finite-value repair
- Stretch-layout truth tables
- Cell-width arithmetic (including non-finite inputs)
- Vertical cell-width behaviour
- Custom-content padding and minimum height
- Separator counts and RTL index mapping
- Corner rounding for horizontal, vertical, and single-cell groups
- Theme/accent decision helpers
- Selection semantics for `Single`, `Toggle`, and `Multi`
- Icon resolution (check mark, fixed content, hidden placeholders)
- Builder defaults and field assignments
- Collector insertion order

Run with:

```bash
cargo test
```

---

## License

This component is provided under the same license as the surrounding
`mud_egui` project. See the repository root for details.