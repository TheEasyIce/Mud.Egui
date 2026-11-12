# MudDataGrid for `mud_egui`

A sortable, filterable, paginated data grid with row selection for
[egui](https://github.com/emilk/egui), ported from MudBlazor's
[`MudDataGrid<T>`](https://mudblazor.com/components/datagrid) and `Column<T>`.

```rust
let mut state = DataGridState::<Person>::new();

MudDataGrid::new()
    .filterable(true)
    .multi_selection(true)
    .striped(true)
    .show(ui, &people, &mut state, |grid| {
        grid.text_column("name", "Name", |p| p.name.clone())
            .sort_by(|a, b| a.name.cmp(&b.name));
        grid.text_column("city", "City", |p| p.city.clone());
        grid.column(
            Column::new("age", "Age", |ui, p| {
                ui.add(MudText::new(p.age.to_string()).monospace(true));
            })
            .sort_by(|a, b| a.age.cmp(&b.age))
            .align(ColumnAlign::Right)
            .width(100.0),
        );
    });
```

The module is one file (`mud_data_grid.rs`) built from three layers:

| Layer | Contents |
|-------|----------|
| **Data** | `Column<'a, T>`, `SortDefinition`, `ColumnFilter`, `DataGridState<T>` |
| **Pure layout** | `sort_indices`, `filter_indices`, `resolve_column_widths`, `page_range`, `clamp_page`, `toggle_sort`, `toggle_selection`, `breakpoint_px`, `is_stacked` |
| **Component** | `MudDataGrid<'a, T>`, `GridUi<'a, T>`, header / filter-row / body / pager rendering |

---

## Contents

1. [What maps to MudBlazor, and what does not](#what-maps-to-mudblazor-and-what-does-not)
2. [Architecture](#architecture)
3. [API reference](#api-reference)
4. [Examples](#examples)
5. [Layout and data pipeline](#layout-and-data-pipeline)
6. [Performance and memory layout](#performance-and-memory-layout)
7. [Design rationale](#design-rationale)
8. [Differences from MudBlazor](#differences-from-mudblazor)
9. [Testing](#testing)

---

## What maps to MudBlazor, and what does not

The public API was checked against the MudBlazor 8.x `MudDataGrid<T>` and
`Column<T>` parameter lists before implementing. The grid is a combination of
the `MudGrid` layout discipline and the `MudDivider` rendering discipline:
pure layout math first, thin egui painting afterwards.

### Ported parameters

| MudBlazor | This crate | Notes |
|-----------|-----------|-------|
| `SortMode` | `sort_mode(SortMode)` | `None`, `Single`, `Multiple` (default `Multiple`) |
| `AllowUnsorted` | `allow_unsorted(bool)` | Third toggle state `None` |
| `Filterable` | `filterable(bool)` | Default `false` |
| `FilterMode` | `filter_mode(FilterMode)` | Only `ColumnFilterRow` has an egui equivalent |
| `FilterCaseSensitivity` | `filter_case_sensitive(bool)` | Default insensitive |
| `MultiSelection` | `multi_selection(bool)` | Default `false` |
| `SelectOnRowClick` | `select_on_row_click(bool)` | Default `true` |
| `SelectedItems` | `DataGridState::selected: Vec<T>` | Caller-owned binding |
| `SortDefinitions` | `DataGridState::sorts` | Caller-owned binding |
| `FilterDefinitions` | `DataGridState::filters` | Caller-owned binding |
| `CurrentPage` | `current_page(usize)` / state | Parameter or state |
| `RowsPerPage` | `rows_per_page(usize)` / state | Parameter or state |
| `Bordered` | `bordered(bool)` | Column dividers |
| `Outlined` | `outlined(bool)` | Outer border |
| `Hover` | `hover(bool)` | Row hover fill |
| `Striped` | `striped(bool)` | Alternating row fill |
| `Dense` | `dense(bool)` | Compact heights |
| `Elevation` | `elevation(u8)` | Shadow depth (default `1`) |
| `Square` | `square(bool)` | Removes corner rounding |
| `Breakpoint` | `breakpoint(Breakpoint)` | Responsive stacking |
| `Height` | `height(f32)` | Enables a vertical scroll area |
| `Loading` | `loading(bool)` | Overlay + spinner |
| `LoadingProgressColor` | `loading_progress_color(Color)` | Default `Info` |
| `NoRecordsContent` | `no_records_text(impl Into<String>)` | Text form |
| `RowClass` | `row_class(Fn(&T, usize) -> String)` | Resolved through the flavour |
| `RowClick` | `row_click(FnMut(&T, usize))` | |
| `QuickFilter` | `quick_filter(Fn(&T) -> bool)` | Predicate form |
| `ToolBarContent` | `toolbar(FnMut(&mut Ui))` | |
| `Class` | `classes([...])` | Bootstrap / Tailwind resolver |
| `Style` | `frame(egui::Frame)` | egui has no CSS strings |
| `Tag` | `tag(T)` / `tag_ref::<U>()` | `Arc<dyn Any + Send + Sync>` |

### Column parameters

| MudBlazor `Column<T>` | This crate | Notes |
|------------------------|-----------|-------|
| `Title` | `Column::new(id, title, cell)` | Required |
| `Sortable` (`bool?`) | `sortable(bool)` | `None` inherits; requires a comparator |
| `Filterable` (`bool?`) | `filterable(bool)` | `None` inherits the grid flag |
| `Hidden` | `hidden(bool)` | Hidden columns take no width |
| `Align` | `align(ColumnAlign)` | `Left` / `Center` / `Right` |
| `Width` | `width(f32)` | Fixed pixel width |
| `CellTemplate` | the `cell` closure | `Fn(&mut Ui, &T)` |
| `Property` (PropertyColumn) | `Column::text(id, title, get)` | Extractor powers cell + filter |
| `Comparer` | `sort_by(Fn(&T, &T) -> Ordering)` | Explicit comparator |
| `Filterable` text source | `filter_text(Fn(&T) -> String)` | Extractor for filters / search |

### Deliberately not ported

These require DOM/browser features or server-side plumbing that have no
faithful egui equivalent in this crate. Adding no-op builders would be API
invention, which contradicts the parity principle:

- `ServerData`, `VirtualizeServerData`, `Virtualize`, `OverscanCount`,
  `RowLoadingContent` — server data and windowed rendering.
- `EditMode`, `ReadOnly`, `CanCancelEdit`, `CloneStrategy`,
  `StartedEditingItem`, `CommittedItemChanges`, `CanceledEditingItem`,
  `FormFieldChanged`, `Validator` — inline/form editing.
- `Groupable`, `GroupDefinitions`, `IsGrouped`, group expansion — grouping.
- `ColumnResizeMode`, `DragDropColumnReordering`, `ColumnsPanelReordering`,
  `ShowColumnOptions`, `ShowMenuIcon`, `ColumnsPanel` — column chrome.
- `FilterMode::Simple` and `FilterMode::ColumnFilterMenu` — popover filtering.
- `ColGroup`, `ChildRowContent`, `HierarchyColumn`, `ExpandSingleRow` —
  structural row content.

`MudDataGrid` also exposes egui-friendly extensions: `show_search(bool)` (a
built-in search box) and `DataGridState::last_row_rects` for hit-testing.

---

## Architecture

```
mud_data_grid.rs
├── Column<'a, T>              — id, title, flags, align, width,
│                                cell, comparator, filter extractor
├── SortDefinition             — column id + direction
├── ColumnFilter               — column id + text
├── DataGridState<T>           — sorts, filters, page, selection, outputs
├── GridUi<'a, T>              — collector for Column values
├── MudDataGrid<'a, T>         — configuration + rendering
│   └── show(ui, items, state, |grid| ..) -> (Response, R)
├── Pure helpers               — sort_indices, filter_indices,
│                                resolve_column_widths, page_range,
│                                clamp_page, toggle_sort, toggle_selection,
│                                breakpoint_px, is_stacked, row_height, …
└── Render helpers             — draw_header, draw_filter_row, draw_pager,
                                 paint_checkbox, row_fill
```

**Pure layer.** Every decision that can be made from data is a free function.
`sort_indices` and `filter_indices` operate on slices and index lists, so they
are covered by ordinary unit tests and can be reused headlessly (for exports,
snapshot tests, or tooling).

**State layer.** `DataGridState<T>` is caller-owned. It is the egui analogue
of MudBlazor's bindable parameters, and it also carries read-only outputs
(`last_row_rects`, `last_filtered_count`, `last_total_count`) for
introspection.

**Component layer.** `show` resolves the theme and wrapper frame, calls the
pure layer, allocates exact rectangles for header / filter row / body / pager,
and paints. It never measures text to place rows, which keeps layout
deterministic.

---

## API reference

### `MudDataGrid<'a, T>`

| Method | Default | Mirrors | Description |
|--------|---------|---------|-------------|
| `new()` | — | — | MudBlazor defaults. |
| `sort_mode(SortMode)` | `Multiple` | `SortMode` | `None` / `Single` / `Multiple`. |
| `allow_unsorted(bool)` | `false` | `AllowUnsorted` | Adds a third “no sort” toggle state. |
| `filterable(bool)` | `false` | `Filterable` | Enables the inline filter row. |
| `filter_mode(FilterMode)` | `ColumnFilterRow` | `FilterMode` | Only the inline row mode is ported. |
| `filter_case_sensitive(bool)` | `false` | `FilterCaseSensitivity` | String filter matching. |
| `show_search(bool)` | `false` | *(extension)* | Built-in search box writing `state.quick_filter_text`. |
| `multi_selection(bool)` | `false` | `MultiSelection` | Multiple selected rows. |
| `select_on_row_click(bool)` | `true` | `SelectOnRowClick` | Row click toggles selection. |
| `bordered(bool)` | `false` | `Bordered` | Column divider lines. |
| `outlined(bool)` | `false` | `Outlined` | Outer border. |
| `hover(bool)` | `false` | `Hover` | Row hover fill. |
| `striped(bool)` | `false` | `Striped` | Alternating row fill. |
| `dense(bool)` | `false` | `Dense` | Compact heights. |
| `square(bool)` | `false` | `Square` | No corner rounding. |
| `elevation(u8)` | `1` | `Elevation` | Shadow depth (clamped to 24). |
| `breakpoint(Breakpoint)` | `Xs` | `Breakpoint` | Stacks below this breakpoint. |
| `height(f32)` | `None` | `Height` | Enables a scrollable body. |
| `current_page(usize)` | `None` | `CurrentPage` | Overrides state for one call. |
| `rows_per_page(usize)` | `None` | `RowsPerPage` | Overrides state for one call. |
| `show_pager(bool)` | `true` | presence of `PagerContent` | Shows the footer pager. |
| `frame(egui::Frame)` | `None` | `Style` | Replaces the class-resolved frame. |
| `loading(bool)` | `false` | `Loading` | Overlay + spinner. |
| `loading_progress_color(Color)` | `Info` | `LoadingProgressColor` | Spinner tint. |
| `no_records_text(s)` | `"No records to display"` | `NoRecordsContent` | Empty-state text. |
| `aria_label(s)` | `None` | — | Published via `WidgetInfo`. |
| `row_class(f)` | `None` | `RowClass` | Per-row classes resolved through the flavour. |
| `row_click(f)` | `None` | `RowClick` | `FnMut(&T, usize)`. |
| `quick_filter(f)` | `None` | `QuickFilter` | `Fn(&T) -> bool`. |
| `toolbar(f)` | `None` | `ToolBarContent` | `FnMut(&mut Ui)`. |
| `flavor(StyleFlavor)` | default | — | Bootstrap or Tailwind resolver. |
| `classes(iter)` | empty | `Class` | Wrapper utility classes. |
| `tag(T)` / `tag_ref::<U>()` | `None` | `Tag` | Arbitrary user data. |
| `show(ui, items, state, \|grid\| ..)` | — | — | Draws and returns `(Response, R)`. |

### `Column<'a, T>`

| Method | Default | Mirrors | Description |
|--------|---------|---------|-------------|
| `new(id, title, cell)` | — | `Title` + `CellTemplate` | Custom cell renderer. |
| `text(id, title, get)` | — | `PropertyColumn` | Text cell + filter extractor. |
| `sortable(bool)` | inherit | `Sortable` | Requires a comparator to take effect. |
| `filterable(bool)` | inherit | `Filterable` | Requires a filter extractor. |
| `hidden(bool)` | `false` | `Hidden` | Excluded from layout. |
| `align(ColumnAlign)` | `Left` | `Align` | Header and cell alignment. |
| `width(f32)` | flex | `Width` | Fixed pixel width. |
| `sort_by(Fn(&T, &T) -> Ordering)` | `None` | `Comparer` | Sorting comparator. |
| `filter_text(Fn(&T) -> String)` | `None` | filter value | Filter/search extractor. |
| `is_sortable()` | — | — | `sortable.unwrap_or(true) && comparator.is_some()`. |
| `is_filterable(grid_flag)` | — | — | `filterable.unwrap_or(grid_flag) && extractor.is_some()`. |

### `DataGridState<T>`

| Field | Default | Description |
|-------|---------|-------------|
| `sorts` | `[]` | Active sort definitions, priority order. |
| `filters` | `[]` | Active per-column filters. |
| `current_page` | `0` | Zero-based page. |
| `rows_per_page` | `10` | `0` means all rows on one page. |
| `selected` | `[]` | Selected rows. |
| `quick_filter_text` | `""` | Search box contents. |
| `last_row_rects` | `[]` | Screen-space row rectangles from the last frame. |
| `last_filtered_count` | `0` | Rows after filtering in the last frame. |
| `last_total_count` | `0` | Rows passed to `show` in the last frame. |

### Enums

| Type | Variants | Default |
|------|----------|---------|
| `SortDirection` | `None`, `Ascending`, `Descending` | `None` |
| `SortMode` | `None`, `Single`, `Multiple` | `Multiple` |
| `ColumnAlign` | `Left`, `Center`, `Right` | `Left` |
| `FilterMode` | `None`, `ColumnFilterRow` | `ColumnFilterRow` |

---

## Examples

### Basic sortable grid

```rust
let mut state = DataGridState::<Person>::new();

MudDataGrid::new()
    .sort_mode(SortMode::Single)
    .show(ui, &people, &mut state, |grid| {
        grid.column(
            Column::text("name", "Name", |p| p.name.clone())
                .sort_by(|a, b| a.name.cmp(&b.name)),
        );
        grid.column(
            Column::text("age", "Age", |p| p.age.to_string())
                .sort_by(|a, b| a.age.cmp(&b.age))
                .align(ColumnAlign::Right)
                .width(80.0),
        );
    });
```

### Multi-select with a pager

```rust
MudDataGrid::new()
    .multi_selection(true)
    .rows_per_page(5)
    .striped(true)
    .hover(true)
    .show(ui, &people, &mut state, |grid| {
        for col in columns() { grid.column(col); }
    });

for person in &state.selected {
    ui.label(format!("selected: {}", person.name));
}
```

### Filter row + search box

```rust
MudDataGrid::new()
    .filterable(true)
    .show_search(true)
    .show(ui, &people, &mut state, |grid| {
        grid.text_column("name", "Name", |p| p.name.clone());
        grid.text_column("city", "City", |p| p.city.clone());
    });
```

Typing in the search box matches any filterable column; typing in a column's
filter row input narrows that column only. Both compose with AND.

### Responsive stacking

```rust
MudDataGrid::new()
    .breakpoint(Breakpoint::Md) // stack below 960px
    .show(ui, &people, &mut state, |grid| {
        for col in columns() { grid.column(col); }
    });
```

Below the breakpoint each row becomes a card with `title: value` pairs, which
is how MudBlazor's mobile layout behaves for data grids.

### Loading and empty states

```rust
MudDataGrid::new()
    .loading(fetch_in_flight)
    .no_records_text("Nothing here yet")
    .show(ui, &filtered, &mut state, |grid| {
        /* … */
    });
```

### Headless data shaping

The pure layer is public, so you can reuse the exact sort/filter/page logic
without drawing:

```rust
let filtered = filter_indices(&people, &state.filters, &cols, "", None, false);
let sorted = sort_indices(&people, &state.sorts, &cols, &filtered);
let (start, end) = page_range(sorted.len(), 0, 10);
let page = &sorted[start..end];
```

### Row click + custom row classes

```rust
MudDataGrid::new()
    .row_click(|person, idx| println!("clicked {} at row {}", person.name, idx))
    .row_class(|person, _| {
        if person.age > 30 { "bg-warning".into() } else { String::new() }
    })
    .show(ui, &people, &mut state, |grid| { /* … */ });
```

---

## Layout and data pipeline

```
items ──► filter_indices ──► sort_indices ──► page_range ──► visible rows
            │                   │                │
            filters             sorts            current_page / rows_per_page
            quick text          comparators      (clamped)
            quick predicate
```

1. **Filter.** Every active `ColumnFilter` must match (AND). The search box
   matches any filterable column. An optional `quick_filter` predicate is
   ANDed on top. Empty filter values are inactive.
2. **Sort.** The filtered index list is sorted with a stable multi-key
   comparator. Each `SortDefinition` contributes one key; columns without a
   comparator are skipped. `Descending` reverses the comparator result for
   that key only.
3. **Page.** `clamp_page` keeps `current_page` in range after the filter
   changes; `page_range` returns a half-open `[start, end)` slice.
4. **Widths.** `resolve_column_widths` reserves the selection column and all
   fixed widths, then splits the remainder equally among flexible columns.
   Hidden columns contribute nothing.
5. **Draw.** Header, optional filter row, body (scrollable when `height` is
   set), and pager are each allocated as exact rectangles. Cells are child
   `Ui`s clipped to their cell rectangle, so content cannot bleed across
   columns.

Rows are **not** measured: fixed row heights make the planner deterministic
and keep scroll offsets stable. Content taller than a row is clipped, matching
a table-like contract. If you need auto-height rows, use the responsive
stacked mode or a different container.

---

## Performance and memory layout

| Choice | Effect |
|--------|--------|
| `Column` is built once per frame from borrowed closures | No retained render tree. |
| `sort_indices` and `filter_indices` return `Vec<usize>` index lists | Sorting/filtering never moves `T` values; rows stay in place. |
| Multi-key sort uses `sort_by` with short-circuiting keys | One pass of comparisons, stable ordering. |
| `resolve_column_widths` is a single linear pass | No per-column allocation beyond the result vector. |
| Rows use `allocate_exact_size` + absolute child `Ui`s | No per-row layout passes; scroll height is known before drawing. |
| Filter inputs write into `state.filters` in place | No temporary filter map per frame. |
| Selection uses `Vec<T>` + `PartialEq` | No `Hash`/`Eq` bound on `T`; order is preserved. |
| `RowClass` resolution only runs when the callback is set | No resolver work in the common case. |
| `DataGridState` is caller-owned | Sorting/filtering state is not rebuilt each frame. |

Not done, on purpose: a column-width cache and a row-height cache. Both would
require invalidation rules that belong to a layout engine, not a single
widget; the grid stays deterministic instead.

---

## Design rationale

- **Pure data shaping.** Sorting, filtering, pagination, and width resolution
  are pure functions. This is what makes the component testable without a
  window and what keeps `show` a thin painter.
- **Explicit comparators and extractors.** Requiring `T: Ord` or
  `T: Display` would make the grid unusable for domain types. MudBlazor
  solves this with `Comparer` and `PropertyColumn.Property`; this port solves
  it with `sort_by` and `filter_text`.
- **No invented MudBlazor API.** Editing, grouping, virtualization, server
  data, and popover filtering are not present because their MudBlazor
  counterparts depend on DOM features. See the parity section.
- **Bindings become state.** `SelectedItems`, `SortDefinitions`, and
  `FilterDefinitions` are two-way bindings in Blazor; in egui they become
  fields on `DataGridState<T>` that the caller owns and the grid mutates.
- **One filter mode.** `ColumnFilterRow` is the only MudBlazor filter mode
  that does not need a popover, so it is the only one implemented. A column
  without a filter extractor is treated as non-filterable even if the flag is
  set.
- **Responsive stacking, not horizontal scrolling.** Below the configured
  breakpoint rows become label/value cards, which is what MudBlazor's
  `Breakpoint` parameter is for.
- **Never panic in a UI path.** Non-finite widths fall back to 1280 px,
  pages clamp, empty filter strings are inactive, and a missing comparator or
  extractor simply disables the feature instead of panicking.

---

## Differences from MudBlazor

- **`Style` is `frame(egui::Frame)`.** There are no inline CSS strings in
  egui; the typed frame is the equivalent.
- **`NoRecordsContent` and `ToolBarContent` are closures/text, not render
  fragments with parameters.** `toolbar` receives `&mut Ui`; the empty state
  is a text string.
- **`QuickFilter` is a predicate**, and the built-in search box is an
  extension that writes `state.quick_filter_text`. MudBlazor has no built-in
  search box; it expects you to wire `QuickFilter` yourself.
- **Only `FilterMode::ColumnFilterRow` exists.** `Simple` and
  `ColumnFilterMenu` need popovers; they are documented as not ported rather
  than stubbed.
- **Selection is `Vec<T>`, not `HashSet<T>`.** This keeps `T: PartialEq`
  instead of `T: Eq + Hash` and preserves click order for multi-selection.
- **`Breakpoint` uses Bootstrap pixel thresholds** (`Sm` = 600, `Md` = 960,
  `Lg` = 1280, `Xl` = 1920), matching MudBlazor's responsive behaviour.
- **Column `Sortable` / `Filterable` are `Option<bool>`** so `None` inherits
  from the grid, mirroring MudBlazor's nullable column overrides.
- **No column resizing, reordering, or visibility menu.** These are DOM /
  pointer-chrome features and are explicitly out of scope.

---

## Testing

`cargo test` runs the module's tests. Coverage by area:

| Area | Tests |
|------|-------|
| Sort direction | cycle with/without `AllowUnsorted`, `reversed` involution |
| `toggle_sort` | single replace, single clear on third toggle, multiple append, multiple remove, `SortMode::None` no-op |
| `sort_indices` | ascending, descending, multi-key city→age, missing comparator, empty sorts |
| `filter_indices` | case-insensitive, case-sensitive, empty value inactive, AND semantics, quick text over all filterable columns, quick predicate |
| Pagination | `page_count` (zero rows / zero size), `clamp_page`, `page_range` |
| Widths | equal split, selection column sum, hidden skip, non-finite repair |
| Selection | multi toggle preserves order, single replaces |
| Breakpoints | pixel mapping, active edge, strict stacking below |
| Heights | dense vs default row and header |
| Defaults | grid defaults match MudBlazor, column defaults |
| Builders | text column extractor, alignment mapping, tag round-trip |
| State | defaults, `with_rows_per_page`, filter-entry creation |

Rendering itself (`show`) needs a live `egui::Context` and is not unit-tested;
the pure layer is the source of truth for layout and data behaviour, exactly
as in `MudGrid`.

```bash
cargo test -p mud_egui_components mud_data_grid
```