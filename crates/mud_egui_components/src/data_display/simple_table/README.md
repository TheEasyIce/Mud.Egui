# MudSimpleTable for `mud_egui`

A lightweight, non-data-bound table for [egui](https://github.com/emilk/egui),
ported from MudBlazor's [`MudSimpleTable`](https://mudblazor.com/components/simpletable)
(MudBlazor 8.13.0). Draws striped rows, hover highlighting, borders, elevation,
dense rows, outlined/square variants, and a fixed header — without any data
model, sorting, filtering, or pagination.

```rust
MudSimpleTable::new()
    .striped(true)
    .hover(true)
    .bordered(true)
    .show(ui, |t| {
        t.header(|h| {
            h.cell("Name");
            h.cell_width("Age", 80.0);
            h.cell("City");
        });
        for person in &people {
            t.row(|r| {
                r.cell(person.name.clone());
                r.cell(person.age.to_string());
                r.cell(person.city.clone());
            });
        }
    });
```

Every text cell renders through **`MudText`**, so typography and theme updates
propagate automatically.

---

## Contents

1. [Design principle: API parity, not API invention](#design-principle-api-parity-not-api-invention)
2. [Architecture](#architecture)
3. [API reference](#api-reference)
4. [Examples](#examples)
5. [Layout and rendering](#layout-and-rendering)
6. [Performance and memory choices](#performance-and-memory-choices)
7. [Testing](#testing)
8. [Differences from MudBlazor](#differences-from-mudblazor)

---

## Design principle: API parity, not API invention

`MudSimpleTable` is a **styled wrapper around raw table markup**, not a data
grid. The MudBlazor component's `ChildContent` is documented as using HTML
table tags (`<thead>`, `<tbody>`, `<tr>`, `<th>`, `<td>`), and the official
examples use those tags directly. `MudTh` / `MudTd` / `MudTr` belong to
`MudTable<T>` and are **not** part of `MudSimpleTable`.

The egui port mirrors this with a closure-based builder that collects header,
body, and footer rows. No new public enums are introduced; the only enum in the
module (`RowKind`, `CellContent`) is `pub(crate)` and lives in its own file.

### Ported parameters

| MudBlazor | Rust method | Default |
|-----------|-------------|---------|
| `Bordered` | `bordered(bool)` | `false` |
| `ChildContent` | closure to `show` | required |
| `Dense` | `dense(bool)` | `false` |
| `Elevation` | `elevation(u8)` | `1` |
| `FixedHeader` | `fixed_header(bool)` | `false` |
| `Hover` | `hover(bool)` | `false` |
| `Outlined` | `outlined(bool)` | `false` |
| `Square` | `square(bool)` | `false` |
| `Striped` | `striped(bool)` | `false` |
| `Class` (base) | `classes(iter)` | none |
| `Style` (base) | *(use `classes`)* | — |
| `Tag` (base) | `tag(t)` / `tag_ref::<U>()` | `None` |
| `UserAttributes` (base) | **not ported** (no HTML tag) | — |

### Deliberately not ported

- **`Style`** — egui has no inline CSS strings; use `classes(..)`.
- **`UserAttributes`** — there is no HTML tag to splat attributes onto.
- **`MudTh` / `MudTd` / `MudTr` sub-components** — these are for `MudTable<T>`,
  not `MudSimpleTable`. The table builder's `header` / `row` / `footer`
  closures replace them.
- **CSS `height` on `FixedHeader`** — replaced by the explicit
  `height(f32)` builder, which maps directly to egui's
  `ScrollArea::max_height`.

---

## Architecture

```
mud_simple_table.rs
├── MudSimpleTable<'a>       — configuration + render entry point
│   └── show(ui, |t| ..) -> (Response, R)
├── SimpleTableUi<'a>        — collector: header / row / footer
├── RowUi<'a>                — per-row cell builder
├── TableCell<'a>            — (content, optional width)
├── Pure helpers             — row_height, resolve_column_widths,
│                             column_count, column_hints, elevation_shadow
└── Tests                    — 20+ pure-function tests

simple_table_internal.rs      — private CellContent / RowKind enums
```

**Pure layer.** `resolve_column_widths`, `column_hints`, `column_count`,
`row_height`, and `elevation_shadow` are free functions, covered by unit tests,
exactly as in `MudDataGrid` and `MudGrid`.

**Collector layer.** `SimpleTableUi` records header, body, and footer rows
before any painting. The table knows every cell's width hint before it
allocates a single rectangle.

**Component layer.** `show` resolves the theme and wrapper frame, calls the
pure layer, allocates one rectangle per row, and paints through child `Ui`s.

---

## API reference

### `MudSimpleTable<'a>`

| Method | Default | Mirrors | Description |
|--------|---------|---------|-------------|
| `new()` | — | — | MudBlazor defaults. |
| `bordered(bool)` | `false` | `Bordered` | Left/right cell borders. |
| `dense(bool)` | `false` | `Dense` | Compact row heights. |
| `elevation(u8)` | `1` | `Elevation` | Drop shadow (0–24). |
| `fixed_header(bool)` | `false` | `FixedHeader` | Sticky header (requires `height`). |
| `hover(bool)` | `false` | `Hover` | Row hover highlight. |
| `outlined(bool)` | `false` | `Outlined` | Outer border. |
| `square(bool)` | `false` | `Square` | Zero corner radius. |
| `striped(bool)` | `false` | `Striped` | Alternating row fills. |
| `height(f32)` | `None` | `Style="height:…"` | Scrollable body height. |
| `classes(iter)` | none | `Class` | Wrapper utility classes. |
| `flavor(StyleFlavor)` | default | — | Bootstrap / Tailwind. |
| `tag(t)` / `tag_ref::<U>()` | `None` | `Tag` | Arbitrary user data. |
| `show(ui, \|t\| ..)` | — | `ChildContent` | Draws and returns `(Response, R)`. |

### `SimpleTableUi<'a>`

| Method | Description |
|--------|-------------|
| `header(\|h\| ..)` | Sets the header row (replaces any previous). |
| `row(\|r\| ..)` | Adds a body row. |
| `footer(\|f\| ..)` | Sets the footer row (replaces any previous). |
| `row_count()` | Number of body rows collected. |
| `is_empty()` | Whether no body rows have been collected. |

### `RowUi<'a>`

| Method | Description |
|--------|-------------|
| `cell(text)` | Text cell, rendered through `MudText`. |
| `cell_width(text, px)` | Text cell with an explicit width. |
| `cell_ui(f)` | Custom-UI cell; `f: FnMut(&mut Ui)`. |
| `cell_ui_width(f, px)` | Custom-UI cell with an explicit width. |
| `len()` / `is_empty()` | Cell count. |

### Geometry constants

| Name | Value |
|------|-------|
| `ROW_HEIGHT` | `48.0` |
| `DENSE_ROW_HEIGHT` | `32.0` |
| `CELL_PADDING_X` | `12.0` |
| `CELL_PADDING_Y` | `8.0` |
| `MAX_ELEVATION` | `24` |
| `FALLBACK_WIDTH` | `1280.0` |

---

## Examples

### Basic table

```rust
MudSimpleTable::new().show(ui, |t| {
    t.header(|h| {
        h.cell("Name");
        h.cell("Age");
    });
    t.row(|r| {
        r.cell("Alice");
        r.cell("30");
    });
    t.row(|r| {
        r.cell("Bob");
        r.cell("35");
    });
});
```

### Striped + hover + bordered

```rust
MudSimpleTable::new()
    .striped(true)
    .hover(true)
    .bordered(true)
    .show(ui, |t| {
        t.header(|h| {
            h.cell("ID");
            h.cell("Name");
            h.cell("Email");
        });
        for (i, person) in people.iter().enumerate() {
            t.row(|r| {
                r.cell(i.to_string());
                r.cell(person.name.clone());
                r.cell(person.email.clone());
            });
        }
    });
```

### Explicit column widths

```rust
MudSimpleTable::new().show(ui, |t| {
    t.header(|h| {
        h.cell_width("ID", 60.0);
        h.cell("Name");
        h.cell_width("Actions", 120.0);
    });
    t.row(|r| {
        r.cell("1");
        r.cell("Alice");
        r.cell_ui_width(|ui| {
            ui.add(MudButton::new("Edit").size(Size::Small));
        }, 120.0);
    });
});
```

### Dense table

```rust
MudSimpleTable::new().dense(true).show(ui, |t| {
    t.header(|h| h.cell("Key"));
    t.row(|r| r.cell("Value"));
});
```

### Elevation and rounded corners

```rust
MudSimpleTable::new()
    .elevation(4)
    .show(ui, |t| { /* … */ });

MudSimpleTable::new()
    .square(true)   // no rounded corners
    .elevation(0)   // no shadow
    .show(ui, |t| { /* … */ });
```

### Outlined table

```rust
MudSimpleTable::new()
    .outlined(true)
    .show(ui, |t| { /* … */ });
```

### Fixed header with a scrollable body

```rust
MudSimpleTable::new()
    .fixed_header(true)
    .height(300.0)
    .show(ui, |t| {
        t.header(|h| {
            h.cell("Name");
            h.cell("Age");
        });
        for person in &people {
            t.row(|r| {
                r.cell(person.name.clone());
                r.cell(person.age.to_string());
            });
        }
    });
```

`fixed_header(true)` is a no-op unless `height` is also set, matching
MudBlazor's requirement that CSS `height` be set when `FixedHeader` is true.

### Custom-UI cells

```rust
MudSimpleTable::new().show(ui, |t| {
    t.header(|h| {
        h.cell("Product");
        h.cell("Actions");
    });
    for product in &products {
        t.row(|r| {
            r.cell_ui(|ui| {
                ui.horizontal(|ui| {
                    ui.add(MudIcon::new(filled::INVENTORY));
                    ui.add(MudText::new(product.name.clone()));
                });
            });
            r.cell_ui(|ui| {
                ui.horizontal(|ui| {
                    ui.add(MudButton::new("Edit").size(Size::Small));
                    ui.add(MudButton::new("Delete").size(Size::Small).color(Color::Error));
                });
            });
        });
    }
});
```

### Footer row

```rust
MudSimpleTable::new().show(ui, |t| {
    t.header(|h| {
        h.cell("Item");
        h.cell("Qty");
    });
    for line in &cart {
        t.row(|r| {
            r.cell(line.name.clone());
            r.cell(line.qty.to_string());
        });
    }
    t.footer(|f| {
        f.cell("Total");
        f.cell(total_qty.to_string());
    });
});
```

### Tag round-trip

```rust
let table = MudSimpleTable::new()
    .tag("schedule")
    .show(ui, |t| { /* … */ });
```

---

## Layout and rendering

### Column widths

The table collects all rows first, then resolves column widths from the
**first explicit width hint** per column:

1. Scan the header, then every body row, then the footer.
2. For each column, take the first `Some(px)` encountered.
3. Columns with no hint are “flexible”.
4. The remaining space is split equally among flexible columns.
5. If every column is fixed, the result is the sum of the explicit widths.

Explicit widths are set via `cell_width(text, px)` or
`cell_ui_width(closure, px)`.

### Row heights

| Density | Row height |
|---------|------------|
| Normal  | `48.0` |
| Dense   | `32.0` |

### Fixed header

When `fixed_header(true)` **and** `height(..)` are both set:

- The header is drawn once, outside the scroll area.
- The body rows scroll inside a `ScrollArea::vertical().max_height(…)`.
- The footer, if present, is drawn after the scroll area.

When only `height(..)` is set, the entire table (header + body + footer)
scrolls together.

### Borders and stripes

- `bordered(true)` draws a vertical line between each pair of cells.
- A horizontal line is always drawn under every row (header, body, footer).
- `striped(true)` fills odd body rows with `faint_bg_color`.
- `hover(true)` fills the hovered body row with `hovered.weak_bg_fill`.
- `outlined(true)` draws a 1 px rounded outline around the whole table.

### Elevation

`elevation(n)` maps to `egui::epaint::Shadow` on the wrapper `Frame`, using
the same scale as `MudAvatar` / `MudBadge` / `MudDataGrid`. Elevation `0`
produces no shadow. Values above `24` are clamped.

### Text styling

- Header and footer cells: `MudText` with `Typo::Subtitle2` and `bold(true)`.
- Body cells: `MudText` with `Typo::Body2`.
- Custom-UI cells: the closure is responsible for using `MudText` / `MudIcon`.

---

## Performance and memory choices

| Choice | Effect |
|--------|--------|
| Rows are collected once per frame and rendered once | No retained render tree. |
| `resolve_column_widths` is a single linear pass | No per-column allocation beyond the result vector. |
| Rows use `allocate_exact_size` + absolute child `Ui`s | No per-row layout pass; heights are known before painting. |
| Header / footer are stored as `Option<Vec<TableCell>>` | Only one of each per frame. |
| Cell text is moved into the render path, not cloned | Each `TableCell` is single-use per frame. |
| All text goes through `MudText` | Theme/font updates propagate automatically. |
| `fixed_header` uses `ScrollArea::max_height` | Only the body is virtualised, matching MudBlazor. |

Not done, on purpose: automatic content measurement for column widths. Doing
so would require every cell closure to run twice per frame (measure + draw),
which is a significant cost for large tables. The explicit-width mechanism
gives callers the same control without that cost.

---

## Testing

`cargo test` runs the module's tests:

| Area | Tests |
|------|-------|
| `resolve_column_widths` | all-flexible, all-fixed, mixed, over-constrained, non-finite available, empty input |
| `column_count` | widest row, missing sections |
| `column_hints` | first non-`None` wins, per-column |
| `row_height` | dense vs normal |
| `elevation_shadow` | zero, clamped, monotonic |
| Defaults | match MudBlazor |
| Builders | every field, negative `height` clamps to `0` |
| Tag | round-trips by type |
| Collector | preserves row order, header replacement |

Rendering itself (`show`) needs a live `egui::Context` and is not unit-tested;
the pure layer is the source of truth for layout.

```bash
cargo test -p mud_egui_components mud_simple_table
```

---

## Differences from MudBlazor

- **No HTML tags.** The `header` / `row` / `footer` closures replace
  `<thead>` / `<tbody>` / `<tfoot>`, and `cell` / `cell_ui` replace
  `<th>` / `<td>`. This is the idiomatic egui equivalent.
- **No `MudTh` / `MudTd` / `MudTr`.** Those belong to `MudTable<T>`, not
  `MudSimpleTable`; they are not ported here.
- **Explicit column widths.** MudBlazor relies on CSS auto-layout; egui has
  no equivalent without measuring every cell. `cell_width` / `cell_ui_width`
  give callers the same control.
- **`Style` → `classes(..)`.** egui has no inline CSS strings.
- **`UserAttributes` not ported.** There is no HTML tag.
- **`FixedHeader` requires `height(..)`.** MudBlazor requires CSS `height` to
  be set; this port requires the egui equivalent, a numeric height.
- **`Elevation` is `u8`, not `int`.** Values above `24` are clamped,
  matching the other `mud_egui` components.