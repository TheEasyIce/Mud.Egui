# MudTable for `mud_egui`

A sortable, filterable, paginated table with multiselection, responsive
stacking, child rows and a built-in pager for
[egui](https://github.com/emilk/egui), ported from MudBlazor's
[`MudTable<T>`](https://mudblazor.com/components/table) (MudBlazor 8.13.0).

```rust
let mut state = TableState::<Person>::new();

MudTable::new()
    .hover(true)
    .striped(true)
    .multi_selection(true)
    .show(ui, &people, &mut state, |t| {
        t.header(|h| {
            h.sort_th("Name", "name", |a, b| a.name.cmp(&b.name));
            h.sort_th("Age",  "age",  |a, b| a.age.cmp(&b.age));
        });
        t.row_template(|r, p| {
            r.td(Td::text(p.name.clone()).label("Name"));
            r.td(Td::text(p.age.to_string()).label("Age").align(ColumnAlign::Right));
        });
    });
```

Every text goes through **`MudText`**, every glyph through **`MudIcon`**, and
the pager reuses **`MudButton`** / **`MudToggleGroup`**.

---

## Contents

1. [Design principle: API parity, not API invention](#design-principle-api-parity-not-api-invention)
2. [Architecture](#architecture)
3. [API reference](#api-reference)
4. [Examples](#examples)
5. [Data pipeline](#data-pipeline)
6. [Performance and memory choices](#performance-and-memory-choices)
7. [Testing](#testing)
8. [Differences from MudBlazor](#differences-from-mudblazor)

---

## Design principle: API parity, not API invention

MudBlazor's `MudTable<T>` was checked against DNDocs and the source before
implementing. It is **row-template driven**: you supply `RowTemplate` (and
usually `RowTemplate` with `MudTd` cells), plus optional `HeaderContent` /
`FooterContent`, and the table handles sorting, filtering, pagination and
selection.

That is a different model from `MudDataGrid<T>` (column descriptors). This port
follows the `MudTable` model, and reuses the existing
`SortDirection` / `ColumnAlign` / `Breakpoint` / `SelectionMode` enums — no new
public enums are introduced. The only enums in the module (`CellContent`,
`RowKind`) are `pub(crate)` and live in `table_internal.rs`.

### Deliberately not ported

| MudBlazor | Why |
|-----------|-----|
| Inline editing (`Editable`, `RowEditingTemplate`, `EditTrigger`, `ApplyButtonPosition`, `CommitEditIcon`, `CancelEditIcon`, `RowEditPreview/Commit/Cancel`, `IsEditRowSwitchingBlocked`, `ReadOnly`) | Requires an edit-state machine and form validation. Out of scope for a display table. |
| Grouping (`GroupBy`, `GroupHeaderTemplate`, `GroupFooterTemplate`, `Group*Class`, `Group*Style`, `CollapseAllGroups`, `ExpandAllGroups`) | Hierarchical grouping is a separate widget; adding it here would balloon the surface. |
| `ServerData`, `TotalItems`, `Virtualize`, `OverscanCount`, `IsVirtualized` | Server paging and windowed rendering need async plumbing and a virtualised `Ui`. Use the built-in paging. |
| `Comparer` | Rust `T: PartialEq` is the idiomatic equivalent. |
| `ColGroup` | Replaced by [`MudTable::column_widths`] — the same purpose (per-column sizing) expressed in egui terms. |
| `Style`, `RowStyle`, `ContainerStyle`, `CellClass` | egui has no inline CSS strings. `row_class` / `row_class_func` / `header_classes` / `footer_classes` / `container_classes` are resolved through the Bootstrap or Tailwind flavour instead. |
| `UserAttributes` | There is no HTML tag to splat attributes onto. |
| `CustomHeader` / `CustomFooter` flags | Not needed: calling [`TableUi::header`] / [`TableUi::footer`] more than once produces multi-row headers/footers directly. |

---

## Architecture

```
mud_table.rs
├── MudTable<'a, T>            — configuration + render entry point
│   └── show(ui, items, state, |t| ..) -> (Response, R)
├── TableState<T>              — sorts, page, rows-per-page, selection
├── Td<'a>                     — body/footer cell (text | ui, label, align, width)
├── Th<'a, T>                  — header cell (text | ui, sort key + comparator)
├── RowUi<'a>                  — per-row cell collector
├── HeaderUi<'a, T>            — per-header-row cell collector
├── FooterUi<'a>               — per-footer-row cell collector
├── TableUi<'a, T>             — top-level collector (headers, footers,
│                                row_template, child_row)
├── Pure helpers               — row_height, effective_row_height,
│                                active_breakpoint, is_stacked, toggle_sort,
│                                sort_indices_by, page_count, clamp_page,
│                                page_range, resolve_column_widths,
│                                resolve_column_count, resolve_column_hints,
│                                elevation_shadow, toggle_selection
├── draw_cell_in_rect / draw_cell_content / paint_checkbox
└── draw_table_pager           — MudTablePager equivalent

table_internal.rs              — pub(crate) CellContent / RowKind
```

**Pure layer.** Everything decidable from numbers is a free function, unit
tested without an `egui::Context`.

**State layer.** `TableState<T>` is caller-owned — the egui equivalent of
`SelectedItem`, `SelectedItems`, `CurrentPage`, `RowsPerPage`, `SortLabel` and
`SortDirection` bindings.

**Component layer.** `show` filters → sorts → pages → runs the row template
once per visible row → resolves widths → paints.

---

## API reference

### `MudTable<'a, T>`

| Method | Default | Mirrors | Description |
|--------|---------|---------|-------------|
| `new()` | — | — | MudBlazor defaults. |
| `allow_unsorted(bool)` | `true` | `AllowUnsorted` | Allows the third `None` sort state. |
| `filter(Fn(&T) -> bool)` | `None` | `Filter` | Row predicate. |
| `pager(bool)` | `true` | `PagerContent` | Built-in pager. |
| `current_page(usize)` | `None` | `CurrentPage` | One-call override. |
| `rows_per_page(usize)` | `None` | `RowsPerPage` | One-call override. |
| `page_size_options(iter)` | `[10,25,50,100]` | `PageSizeOptions` | Pager sizes. |
| `rows_per_page_string(s)` | `"Rows per page:"` | `RowsPerPageString` | Pager label. |
| `all_items_text(s)` | `"All"` | `AllItemsText` | Label for `usize::MAX`. |
| `hide_rows_per_page(bool)` | `false` | `HideRowsPerPage` | Hides the size selector. |
| `multi_selection(bool)` | `false` | `MultiSelection` | Checkbox column. |
| `select_on_row_click(bool)` | `true` | `SelectOnRowClick` | Row click toggles. |
| `selection_changeable(bool)` | `true` | `SelectionChangeable` | Freezes selection. |
| `bordered(bool)` | `false` | `Bordered` | Cell dividers. |
| `dense(bool)` | `false` | `Dense` | Compact rows. |
| `elevation(u8)` | `1` | `Elevation` | Shadow (0–24). |
| `hover(bool)` | `false` | `Hover` | Row hover fill. |
| `outlined(bool)` | `false` | `Outlined` | Outer border. |
| `square(bool)` | `false` | `Square` | No rounding. |
| `striped(bool)` | `false` | `Striped` | Alternating fills. |
| `breakpoint(Breakpoint)` | `Xs` | `Breakpoint` | Stacking threshold. |
| `item_size(f32)` | `None` | `ItemSize` | Explicit row height. |
| `height(f32)` | `None` | `Height` | Scrollable table. |
| `fixed_header(bool)` | `false` | `FixedHeader` | Sticky header. |
| `fixed_footer(bool)` | `false` | `FixedFooter` | Sticky footer. |
| `horizontal_scrollbar(bool)` | `false` | `HorizontalScrollbar` | Horizontal scroll. |
| `column_widths(iter)` | `None` | `ColGroup` | Per-column widths. |
| `row_class(s)` | `None` | `RowClass` | Row utility classes. |
| `row_class_func(f)` | `None` | `RowClassFunc` | Per-row classes. |
| `on_row_click(f)` | `None` | `OnRowClick` | `FnMut(&T, usize)`. |
| `on_row_mouse_enter(f)` | `None` | `OnRowMouseEnter` | Hover-enter. |
| `on_row_mouse_leave(f)` | `None` | `OnRowMouseLeave` | Hover-leave. |
| `toolbar(f)` | `None` | `ToolBarContent` | Above the table. |
| `loading(bool)` | `false` | `Loading` | Overlay + spinner. |
| `loading_progress_color(Color)` | `Info` | `LoadingProgressColor` | Spinner tint. |
| `no_records_text(s)` | `"No records to display"` | `NoRecordsContent` | Empty text. |
| `loading_text(s)` | `"Loading…"` | `LoadingContent` | Loading text. |
| `aria_label(s)` | `None` | — | Accessible label. |
| `classes(iter)` | none | `TableClass` | Table classes. |
| `container_classes(s)` | `None` | `ContainerClass` | Container classes. |
| `header_classes(s)` | `None` | `HeaderClass` | Header row classes. |
| `footer_classes(s)` | `None` | `FooterClass` | Footer row classes. |
| `flavor(StyleFlavor)` | default | — | Utility resolver. |
| `tag(t)` / `tag_ref::<U>()` | `None` | `Tag` | Arbitrary data. |
| `show(ui, items, state, \|t\| ..)` | — | — | Returns `(Response, R)`. |

### `Td<'a>`

| Method | Mirrors | Description |
|--------|---------|-------------|
| `text(s)` | `MudTd` + text | Text cell via `MudText`. |
| `ui(f)` | `MudTd` + `ChildContent` | Custom cell. |
| `label(s)` | `MudTd.DataLabel` | Responsive-mode label. |
| `hidden(bool)` | `MudTd.Hidden` | Skips the cell. |
| `align(ColumnAlign)` | `MudTd` alignment | Left / Center / Right. |
| `width(f32)` | `MudTd` width | Explicit column hint. |

### `Th<'a, T>`

| Method | Mirrors | Description |
|--------|---------|-------------|
| `text(s)` | `MudTh` | Plain header cell. |
| `ui(f)` | `MudTh` + `ChildContent` | Custom header cell. |
| `sortable(bool)` | `MudTh.Sortable` | Enable/disable sorting. |
| `sort_by(key, cmp)` | `MudTableSortLabel.SortLabel` + `SortBy` | Key + comparator. |
| `hidden(bool)` | `MudTh.Hidden` | Skips the column. |
| `align(ColumnAlign)` | `MudTh` alignment | Alignment. |
| `width(f32)` | `MudTh` width | Explicit column hint. |
| `is_sortable()` | — | Resolved sortability. |

### `TableUi<'a, T>`

| Method | Mirrors | Description |
|--------|---------|-------------|
| `header(\|h\| ..)` | `HeaderContent` / `MudTHeadRow` | Appends a header row. |
| `footer(\|f\| ..)` | `FooterContent` / `MudTFootRow` | Appends a footer row. |
| `row_template(f)` | `RowTemplate` | `FnMut(&mut RowUi, &T)`. |
| `child_row(f)` | `ChildRowContent` | `FnMut(&mut RowUi, &T)`. |

### `TableState<T>`

| Field | Default | Description |
|-------|---------|-------------|
| `sort_label` | `None` | Active sort key. |
| `sort_direction` | `None` | Active direction. |
| `current_page` | `0` | Zero-based page. |
| `rows_per_page` | `10` | `0` = all rows. |
| `selected_item` | `None` | Single selection. |
| `selected_items` | `[]` | Multi selection. |
| `last_row_rects` | `[]` | Row rectangles from the last frame. |
| `last_filtered_count` | `0` | Rows after filtering. |
| `last_total_count` | `0` | Rows passed to `show`. |
| `is_selected(&T)` | — | Mode-aware membership test. |
| `with_rows_per_page(n)` | — | Builder form. |

---

## Examples

### Basic sortable, striped table

```rust
MudTable::new()
    .striped(true)
    .hover(true)
    .show(ui, &people, &mut state, |t| {
        t.header(|h| {
            h.sort_th("Name", "name", |a, b| a.name.cmp(&b.name));
            h.sort_th("Age", "age", |a, b| a.age.cmp(&b.age));
        });
        t.row_template(|r, p| {
            r.td(Td::text(p.name.clone()).label("Name"));
            r.td(Td::text(p.age.to_string()).label("Age"));
        });
    });
```

### Multiselection with checkboxes

```rust
MudTable::new()
    .multi_selection(true)
    .show(ui, &people, &mut state, |t| {
        t.header(|h| { h.text("Name"); h.text("Age"); });
        t.row_template(|r, p| {
            r.td(Td::text(p.name.clone()).label("Name"));
            r.td(Td::text(p.age.to_string()).label("Age"));
        });
    });

for p in &state.selected_items {
    ui.label(&p.name);
}
```

### Filter + pager

```rust
MudTable::new()
    .filter(|p: &Person| p.age > 18)
    .rows_per_page(5)
    .page_size_options([5, 10, 25])
    .show(ui, &people, &mut state, |t| { /* … */ });
```

### Sticky header with a scrollable body

```rust
MudTable::new()
    .height(320.0)
    .fixed_header(true)
    .show(ui, &people, &mut state, |t| { /* … */ });
```

`fixed_header` only takes effect when `height` is set — same rule as
MudBlazor.

### Custom cells and right alignment

```rust
t.row_template(|r, p| {
    r.td(Td::text(p.name.clone()).label("Name"));
    r.td(
        Td::ui(|ui| {
            ui.add(MudButton::new("Edit").size(Size::Small));
        })
        .label("Actions"),
    );
    r.td(Td::text(format!("${}", p.salary)).align(ColumnAlign::Right).label("Salary"));
});
```

### Child rows

```rust
MudTable::new().show(ui, &people, &mut state, |t| {
    t.header(|h| h.text("Name"));
    t.row_template(|r, p| r.text(p.name.clone()));
    t.child_row(|r, p| {
        r.ui(|ui| {
            ui.add(MudText::new(format!("Age: {}", p.age)).typo(Typo::Caption));
        });
    });
});
```

`ChildRowContent` renders under **every** row, exactly as in MudBlazor.

### Multi-row header

```rust
t.header(|h| { h.text("Person"); h.text(""); });
t.header(|h| { h.text("Name");   h.text("Age"); });
```

Equivalent to `CustomHeader="true"` plus two `MudTHeadRow`s.

### Explicit column widths

```rust
MudTable::new()
    .column_widths([Some(60.0), None, Some(120.0)])
    .show(ui, &people, &mut state, |t| { /* … */ });
```

This is the egui form of `ColGroup`.

### Responsive stacking

```rust
MudTable::new()
    .breakpoint(Breakpoint::Md) // stack below 960 px
    .show(ui, &people, &mut state, |t| { /* … */ });
```

Below the breakpoint each row becomes a card, and each `Td` renders as
`label` + value. A cell without `.label(..)` renders as bare content —
MudBlazor behaves the same way when `DataLabel` is omitted.

### Row callbacks and classes

```rust
MudTable::new()
    .on_row_click(|p, idx| println!("{idx}: {}", p.name))
    .row_class_func(|p, _| if p.age > 30 { "bg-warning".into() } else { String::new() })
    .show(ui, &people, &mut state, |t| { /* … */ });
```

### Loading and empty states

```rust
MudTable::new()
    .loading(fetch_in_flight)
    .loading_text("Fetching…")
    .no_records_text("Nothing here yet")
    .show(ui, &rows, &mut state, |t| { /* … */ });
```

---

## Data pipeline

```
items ──► filter ──► sort_indices_by ──► clamp_page / page_range ──► page rows
            │              │                      │
         Filter         sort_label +          current_page +
                        sort_direction        rows_per_page
```

1. **Filter.** `filter` predicate, if set.
2. **Sort.** `sort_indices_by` with the comparator registered on the `Th`
   whose `sort_key` matches `state.sort_label`. Single-column only.
3. **Page.** `clamp_page` keeps `current_page` valid after filtering;
   `page_range` returns `[start, end)`.
4. **Row template.** Runs **once per visible row** into an owned `Vec<Td>`.
5. **Columns.** Count and widths resolved from `column_widths` → header rows →
   footer rows → widest body row. Minimum one column.
6. **Draw.** Header rows → body rows (optional checkbox column) → child rows
   → footer rows → pager.

Running the row template exactly once per row keeps `FnMut` side effects
predictable (no measure/draw double-run).

---

## Performance and memory choices

| Choice | Effect |
|--------|--------|
| Row template runs once per visible row | No double invocation; predictable `FnMut`. |
| One `Vec<Td>` per visible row | Bounded by `rows_per_page` (10–100); freed each frame. |
| Index-based filtering and sorting | `T` values never move; rows stay in place. |
| Stable `sort_indices_by` | Predictable ordering for equal keys. |
| Single-pass `resolve_column_widths` | One allocation for the result. |
| Header rows are `Vec<Vec<Th>>` | Multi-row headers cost nothing extra. |
| All text through `MudText`, icons through `MudIcon` | Theme/font updates propagate; no duplicated style logic. |
| `MudToggleGroup` for page sizes | Reuses the themed component instead of a raw combo box. |
| `ui.scope_id()` for interaction ids | Avoids the deprecated `ui.id()` and keeps ids unique per table. |
| `paint_checkbox` takes `&mut Ui` | `Ui::put` requires it; the glyph is `.to_owned()` to disambiguate `MudIcon::new`. |

Not done, on purpose: a column-width cache and a row-height cache. Both need
invalidation rules that belong to a layout engine; the table stays
deterministic instead.

---

## Testing

`cargo test` runs the module's tests:

| Area | Tests |
|------|-------|
| Row height | density defaults, `item_size` override, non-finite / negative repair |
| Breakpoints | Bootstrap thresholds, active edges, strict stacking below |
| Sorting | `toggle_sort` two-state and three-state cycles, column switching; `sort_indices_by` asc / desc / none |
| Pagination | `page_count` (zero rows / zero size), `clamp_page`, `page_range` |
| Widths | equal split, mixed fixed/flexible, non-finite available, over-constrained |
| Columns | explicit widths win, widest section wins, default of 1, first-hint-wins |
| Elevation | zero is no shadow, clamping |
| Selection | single keeps on re-click, multi toggles membership, `is_selected` covers both shapes |
| Builders | MudBlazor defaults, every field set |
| Cells | `Td` builders, `Th::is_sortable` requires key **and** comparator |
| Collectors | order preserved, multi-row headers append |
| Tag | round-trips by type |

Rendering (`show`) needs a live `egui::Context` and is not unit-tested; the pure
layer is the source of truth.

```bash
cargo test -p mud_egui_components mud_table
```

---

## Differences from MudBlazor

- **No inline editing.** `Editable`, `RowEditingTemplate`, `EditTrigger`,
  commit/cancel icons and the edit callbacks are out of scope; see the
  parity section.
- **No grouping.** `GroupBy` and the group templates are a separate widget.
- **No server data / virtualization.** Paging is local and rows are drawn
  eagerly for the current page.
- **`ColGroup` → `column_widths(iter)`.** Same purpose (per-column sizing),
  expressed as pixel hints because egui has no CSS `<col>`.
- **`Style` / `RowStyle` / `CellClass` → class strings resolved through the
  flavour.** `row_class`, `row_class_func`, `header_classes`,
  `footer_classes`, `container_classes` and `classes` feed the Bootstrap or
  Tailwind resolver. `CellClass` is not ported because `<td>` class semantics
  do not map to a single egui property.
- **`CustomHeader` / `CustomFooter` flags are implicit.** Calling
  `TableUi::header` / `TableUi::footer` more than once produces multi-row
  headers/footers, so the flags are unnecessary.
- **`SelectedItem` and `SelectedItems` are unified in `TableState<T>`.**
  Single selection writes `selected_item`, multi writes `selected_items`; the
  two are kept mutually exclusive by `toggle_selection`.
- **`PagerContent` → `pager(true)`.** The built-in pager reproduces
  `MudTablePager`'s page-size selector, range label and previous/next buttons.
- **`TableState` exposes `last_row_rects`.** An extension for hit-testing or
  overlays; MudBlazor has no equivalent because the DOM provides it.