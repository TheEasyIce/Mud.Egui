# MudPagination for `mud_egui`

A horizontal row of clickable page numbers with optional first / previous /
next / last navigation controls for [egui](https://github.com/emilk/egui),
ported from MudBlazor's [`MudPagination`](https://mudblazor.com/components/pagination)
(MudBlazor 8.13.0).

```rust
let mut state = PaginationState::new();

MudPagination::new(10)
    .color(Color::Primary)
    .show_first_button(true)
    .show_last_button(true)
    .show(ui, &mut state);
```

Page numbers render through **`MudText`**; control icons and ellipses render
through **`MudIcon`**.

---

## Contents

1. [Design principle: API parity, not API invention](#design-principle-api-parity-not-api-invention)
2. [Architecture](#architecture)
3. [API reference — MudPagination](#api-reference--mudpagination)
4. [API reference — PaginationState](#api-reference--paginationstate)
5. [API reference — PaginationAction & PageControl](#api-reference--paginationaction--pagecontrol)
6. [Examples](#examples)
7. [Layout algorithm](#layout-algorithm)
8. [Performance and memory choices](#performance-and-memory-choices)
9. [Testing](#testing)
10. [Differences from MudBlazor](#differences-from-mudblazor)

---

## Design principle: API parity, not API invention

The MudBlazor 8.13.0 `MudPagination` surface was verified against
[DNDocs](https://docs.dndocs.com/n/MudBlazor/8.13.0/api/MudBlazor.MudPagination.html)
before this refactor.

### Ported parameters

| MudBlazor | Rust method | Default |
|-----------|-------------|---------|
| `BeforeIcon` | `before_icon(s)` | `NAVIGATE_BEFORE` |
| `BoundaryCount` | `boundary_count(u32)` | `1` |
| `Color` | `color(Color)` | `Primary` |
| `ControlButtonClicked` | `on_control_clicked(f)` / `PaginationResponse.action` | — |
| `Count` | `new(n)` / `count(u32)` | `1` |
| `Disabled` | `disabled(bool)` | `false` |
| `DropShadow` | `drop_shadow(bool)` | `true` |
| `FirstIcon` | `first_icon(s)` | `FIRST_PAGE` |
| `LastIcon` | `last_icon(s)` | `LAST_PAGE` |
| `MiddleCount` | `middle_count(u32)` | `1` |
| `NextIcon` | `next_icon(s)` | `NAVIGATE_NEXT` |
| `Rectangular` | `rectangular(bool)` | `false` |
| `RightToLeft` | `right_to_left(bool)` | `false` |
| `Selected` | `PaginationState::selected` | `1` |
| `SelectedChanged` | `on_selected_changed(f)` / `PaginationResponse.selected` | — |
| `ShowFirstButton` | `show_first_button(bool)` | `false` |
| `ShowLastButton` | `show_last_button(bool)` | `false` |
| `ShowNextButton` | `show_next_button(bool)` | `true` |
| `ShowPageButtons` | `show_page_buttons(bool)` | `true` |
| `ShowPreviousButton` | `show_previous_button(bool)` | `true` |
| `Size` | `size(Size)` | `Medium` |
| `Variant` | `variant(Variant)` | `Text` |
| `Class` (base) | `classes(iter)` | `None` |
| `Style` (base) | *(use `classes`)* | — |
| `Tag` (base) | **not ported** (no HTML tag) | — |
| `UserAttributes` | **not ported** | — |

### Extensions (not in MudBlazor)

| Extension | Description |
|-----------|-------------|
| `id_salt(s)` | Stable interaction ids. |
| `PaginationState` | Caller-owned selection state. |
| `PaginationResponse` | Rich return value with `selected` and `action`. |
| `paginate_layout` | Pure layout function (internal, unit-tested). |

### Deliberately not ported

- **`Tag` / `UserAttributes`** — no HTML tag.
- **`Style`** — no inline CSS strings; use `classes(..)`.
- **`NavigateToAsync(Page)` / `NavigateToAsync(int)`** — the caller owns
  `PaginationState`, so navigation is a direct field write. See the
  [0-based vs 1-based note](#api-reference--paginationstate).

### Note on `Page` → `PageControl`

MudBlazor's `Page` enum (`First` / `Previous` / `Next` / `Last`) is ported as
`PageControl` because `Page` is already the trait name in
`mud_egui_navigation::page`. The variants and semantics are identical.

---

## Architecture

```
pagination/
├── paginate_item.rs       — pub(crate) PaginateItem (Page | Ellipsis)
├── pagination_action.rs   — pub PaginationAction (None | Page | Control)
├── pagination_state.rs    — pub PaginationState { selected }
├── mud_pagination.rs      — MudPagination + PaginationResponse + pure helpers
└── mod.rs

mud_egui_core/src/enums/
└── page_control.rs        — pub PageControl (First | Previous | Next | Last)
```

**Pure layer.** `paginate_layout`, `button_size`, `icon_size`, `font_size`,
`resolve_color`, `resolve_variant` are free functions or `pub(crate)`
helpers, unit-tested without `egui::Context`.

**State layer.** `PaginationState { selected: u32 }` is caller-owned. This is
the egui equivalent of MudBlazor's bindable `Selected` parameter.

**Component layer.** `show` resolves the theme and frame, computes the layout,
renders the row (respecting `right_to_left`), applies the click to `state`,
and returns a `PaginationResponse`.

---

## API reference — `MudPagination`

| Method | Type | Default | Description |
|--------|------|---------|-------------|
| `new(count)` | `u32` | — | Total page count (clamped to ≥ 1). |
| `first_icon(s)` | `impl Into<String>` | `FIRST_PAGE` | First-page icon. |
| `before_icon(s)` | `impl Into<String>` | `NAVIGATE_BEFORE` | Previous-page icon. |
| `next_icon(s)` | `impl Into<String>` | `NAVIGATE_NEXT` | Next-page icon. |
| `last_icon(s)` | `impl Into<String>` | `LAST_PAGE` | Last-page icon. |
| `boundary_count(n)` | `u32` | `1` | Pages always shown at each end. |
| `middle_count(n)` | `u32` | `1` | Pages shown around the selected one. |
| `rectangular(b)` | `bool` | `false` | Square buttons instead of round. |
| `drop_shadow(b)` | `bool` | `true` | Shadow behind buttons. |
| `right_to_left(b)` | `bool` | `false` | RTL layout. |
| `show_first_button(b)` | `bool` | `false` | Show the "first page" control. |
| `show_last_button(b)` | `bool` | `false` | Show the "last page" control. |
| `show_next_button(b)` | `bool` | `true` | Show the "next page" control. |
| `show_previous_button(b)` | `bool` | `true` | Show the "previous page" control. |
| `show_page_buttons(b)` | `bool` | `true` | Show numeric page buttons. |
| `count(n)` | `u32` | `1` | Total pages (clamped to ≥ 1). |
| `disabled(b)` | `bool` | `false` | Prevents all clicks. |
| `color(c)` | `Color` | `Primary` | Accent colour. |
| `size(s)` | `Size` | `Medium` | Button size. |
| `variant(v)` | `Variant` | `Text` | Variant for non-selected buttons. |
| `id_salt(s)` | `impl Into<String>` | `None` | Stable interaction ids. |
| `on_selected_changed(f)` | `FnMut(u32)` | `None` | Fired when selection changes. |
| `on_control_clicked(f)` | `FnMut(PageControl)` | `None` | Fired on control click. |
| `classes(iter)` | `iter of AsRef<str>` | none | Wrapper utility classes. |
| `flavor(f)` | `StyleFlavor` | `Bootstrap` | Utility resolver. |
| `show(ui, state)` | `(&mut Ui, &mut PaginationState)` | — | Returns `PaginationResponse`. |

### `PaginationResponse`

| Field | Type | Description |
|-------|------|-------------|
| `response` | `egui::Response` | The wrapper's response. |
| `selected` | `u32` | Page after the click was applied (1-based). |
| `action` | `PaginationAction` | What was clicked this frame. |

---

## API reference — `PaginationState`

| Field / method | Description |
|----------------|-------------|
| `selected: u32` | Currently selected page (1-based). |
| `new()` | `selected = 1`. |
| `with_selected(n)` | `selected = max(n, 1)`. |
| `navigate_to(page, count)` | Clamps into `1..=count`; returns `true` on change. |
| `next(count)` | `selected + 1`, clamped at `count`. |
| `previous(count)` | `selected - 1`, clamped at `1`. |

### 0-based vs 1-based

MudBlazor has two `NavigateToAsync` overloads:

- `NavigateToAsync(Page page)` — takes the `Page` enum.
- `NavigateToAsync(int pageIndex)` — takes a **0-based** index.

The port only exposes 1-based methods because the `Selected` parameter
itself is 1-based, and keeping the state model uniform is more important
than mirroring the dual overload. Callers who want to mirror the 0-based API
can write `state.selected = index + 1`.

---

## API reference — `PaginationAction` & `PageControl`

### `PaginationAction`

| Variant | Description |
|---------|-------------|
| `None` | No click this frame. |
| `Page(u32)` | A page number button was clicked (1-based). |
| `Control(PageControl)` | A first/previous/next/last control was clicked. |

Helpers: `is_change()`, `is_control()`, `target_page()` (only for `Page(_)`).

### `PageControl`

| Variant | Description |
|---------|-------------|
| `First` | First-page control. |
| `Previous` | Previous-page control. |
| `Next` | Next-page control. |
| `Last` | Last-page control. |

Renamed from MudBlazor's `Page` to avoid clashing with
`mud_egui_navigation::page::Page`.

---

## Examples

### Basic pagination

```rust
let mut state = PaginationState::new();

MudPagination::new(10).show(ui, &mut state);
// Clicking page 5 → state.selected == 5
```

### With first/last controls

```rust
MudPagination::new(20)
    .show_first_button(true)
    .show_last_button(true)
    .show(ui, &mut state);
```

### Larger boundary and middle ranges

```rust
MudPagination::new(50)
    .boundary_count(2)
    .middle_count(3)
    .show(ui, &mut state);
// → 1 2 ... 24 25 26 ... 49 50
```

### Rectangular variant

```rust
MudPagination::new(10)
    .rectangular(true)
    .show(ui, &mut state);
```

### Filled variant

```rust
MudPagination::new(5)
    .variant(Variant::Filled)
    .show(ui, &mut state);
// All pages rendered as solid coloured buttons.
```

### Outlined variant

```rust
MudPagination::new(5)
    .variant(Variant::Outlined)
    .show(ui, &mut state);
// Non-selected pages outlined; selected page filled.
```

### Disabled

```rust
MudPagination::new(5)
    .disabled(true)
    .show(ui, &mut state);
// No button responds.
```

### Without page buttons (only controls)

```rust
MudPagination::new(10)
    .show_page_buttons(false)
    .show_first_button(true)
    .show_last_button(true)
    .show(ui, &mut state);
// Only ⏮ ◀ ▶ ⏭ are rendered.
```

### RTL

```rust
MudPagination::new(10)
    .right_to_left(true)
    .show(ui, &mut state);
// The row is laid out right-to-left, so the "first" control appears on
// the right of the row.
```

### Reading the outcome

```rust
let resp = MudPagination::new(10).show(ui, &mut state);
match resp.action {
    PaginationAction::Page(n) => log::info!("jumped to page {n}"),
    PaginationAction::Control(PageControl::Next) => log::info!("next"),
    PaginationAction::Control(PageControl::Last) => log::info!("last"),
    PaginationAction::Control(PageControl::First) => log::info!("first"),
    PaginationAction::Control(PageControl::Previous) => log::info!("previous"),
    PaginationAction::None => {}
}
```

### Callbacks

```rust
MudPagination::new(10)
    .on_selected_changed(|page| log::info!("page changed to {page}"))
    .on_control_clicked(|ctl| log::info!("control clicked: {ctl:?}"))
    .show(ui, &mut state);
```

### Programmatic navigation

```rust
// In response to an external event:
state.navigate_to(5, 10);  // clamps to 5
state.next(10);            // 6
state.previous(10);        // 5
```

### Overriding icons

```rust
MudPagination::new(10)
    .first_icon(filled::KEYBOARD_DOUBLE_ARROW_LEFT)
    .before_icon(filled::KEYBOARD_ARROW_LEFT)
    .next_icon(filled::KEYBOARD_ARROW_RIGHT)
    .last_icon(filled::KEYBOARD_DOUBLE_ARROW_RIGHT)
    .show(ui, &mut state);
```

---

## Layout algorithm

`paginate_layout(count, boundary_count, middle_count, selected)` returns a
`Vec` of `PaginateItem`s:

- The first `boundary_count` pages are always shown.
- The last `boundary_count` pages are always shown.
- The `middle_count` pages centred on `selected` are always shown
  (`middle_count >= 1` so the selected page is never hidden).
- Any gap of one or more hidden pages becomes one `Ellipsis`.

### Truth table

| `count` | `boundary` | `middle` | `selected` | Result |
|---------|------------|----------|------------|--------|
| 9 | 1 | 1 | 5 | `1 … 5 … 9` |
| 9 | 1 | 3 | 5 | `1 … 4 5 6 … 9` |
| 9 | 2 | 1 | 5 | `1 2 … 5 … 8 9` |
| 9 | 2 | 3 | 5 | `1 2 … 4 5 6 … 8 9` |
| 5 | 2 | 1 | 3 | `1 2 3 4 5` (contiguous) |
| 5 | 1 | 1 | 3 | `1 … 3 … 5` |
| 1 | 1 | 1 | 1 | `1` |
| 0 | 1 | 1 | 1 | `1` (count clamped) |
| 9 | 0 | 1 | 5 | `… 5` (leading ellipsis; no trailing) |

### Clamping

- `count < 1` → treated as `1`.
- `selected < 1` → clamped to `1`.
- `selected > count` → clamped to `count`.
- `middle_count < 1` → treated as `1` (guarantees the selected page is shown).

### Ellipsis semantics

An ellipsis is emitted **before** the next shown page, so a **trailing**
ellipsis is never produced (there is always a page after it). A **leading**
ellipsis can appear when `boundary_count = 0` and the selected page's middle
range does not start at page 1.

---

## Performance and memory choices

| Choice | Effect |
|--------|--------|
| `paginate_layout` is a pure function | No `egui::Context`; unit-tested. |
| `PaginateItem` is `Copy` | No per-item allocations. |
| Layout returns at most `count` items | Bounded by `count`, typically ≤ 20. |
| Page number rendering through `MudText` | Theme and typography propagate automatically. |
| Control icons through `MudIcon` | Free thanks to the transform cache for repeated icons. |
| `PaginationState` is `Copy` | Zero-cost snapshot; passed as `&mut`. |
| `PaginationAction` is `Copy` | Returned by value; easy to test. |
| Single-pass row layout | No second pass; all buttons are allocated as they render. |
| Callbacks are `Option<Box<dyn FnMut>>` | Zero-cost when `None`. |

---

## Testing

`cargo test` runs 20+ pure-function tests:

| Area | Tests |
|------|-------|
| `button_size` / `icon_size` / `font_size` | every `Size` |
| `resolve_color` | `Default`/`Inherit` → primary, explicit → palette |
| `resolve_variant` | selected always `Filled`, unselected uses base |
| `paginate_layout` | small count, boundary/middle truth table, clamping, ellipsis rules, `middle_count = 0`, `boundary_count = 0` |
| `PaginationState` | default, `with_selected`, `navigate_to`, `next`, `previous`, count-0 handling |
| `PaginationAction` | every variant, `is_change`, `is_control`, `target_page` |
| Defaults | match MudBlazor |
| Builders | every field, `count` clamped to ≥ 1 |

Rendering (`show`) needs a live `egui::Context`; the pure layer is the source
of truth.

```bash
cargo test -p mud_egui_components mud_pagination
cargo test -p mud_egui_core page_control
```

---

## Differences from MudBlazor

- **`Page` → `PageControl`.** Renamed to avoid clashing with
  `mud_egui_navigation::page::Page`.
- **State is caller-owned (`PaginationState`).** MudBlazor uses the bindable
  `Selected` parameter; the egui equivalent is a struct the caller owns and
  passes by `&mut`.
- **`NavigateToAsync` not ported.** Direct field writes on `PaginationState`
  are the equivalent. Only 1-based methods are exposed.
- **`Style` / `Tag` / `UserAttributes` not ported.** No CSS strings or HTML
  tags.
- **`Variant::Filled` quirk preserved.** In MudBlazor, `Variant::Filled`
  makes all page buttons solid (the selected one is still filled with the
  accent, but the non-selected pages look identical). The port reproduces
  this exactly; use `Variant::Text` or `Variant::Outlined` to get the
  familiar "only the selected page is filled" look.
- **`RightToLeft` is a normal parameter.** MudBlazor uses a cascading
  parameter; the port takes it directly and lays the row out RTL via
  `egui::Layout::right_to_left`.