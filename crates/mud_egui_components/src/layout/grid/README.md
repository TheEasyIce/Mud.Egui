# MudGrid for `mud_egui`

A twelve-track, mobile-first responsive layout grid for [egui](https://github.com/emilk/egui),
ported from MudBlazor's `MudGrid` / `MudItem`.

```rust
MudGrid::new()
    .spacing(3)
    .show(ui, |grid| {
        grid.item(MudItem::new().xs(12).md(6), |ui| { ui.label("Left");  });
        grid.item(MudItem::new().xs(12).md(6), |ui| { ui.label("Right"); });
    });
```

Three source files make up the module:

| File | Contents |
|------|----------|
| `mud_grid_span.rs` | `MudGridSpan`: the size one item asks for at one breakpoint (`Tracks`, `Star`, `Auto`, `Unset`), plus parsing, conversions and presets |
| `mud_item.rs` | `MudItem`: six spans (`xs`..`xxl`) and the mobile-first cascade |
| `mud_grid.rs` | `MudGrid`, `GridUi`, `ItemStyle`, the pure layout math and the headless planner |

---

## Contents

1. [What maps to MudBlazor, and what does not](#what-maps-to-mudblazor-and-what-does-not)
2. [Architecture](#architecture)
3. [API reference](#api-reference)
4. [Examples](#examples)
5. [Layout algorithm](#layout-algorithm)
6. [Performance and memory layout](#performance-and-memory-layout)
7. [Design rationale](#design-rationale)
8. [Behaviour changes from the previous version](#behaviour-changes-from-the-previous-version)
9. [Testing](#testing)

---

## What maps to MudBlazor, and what does not

The public MudBlazor API was checked before extending anything. `MudGrid` exposes `Spacing`
(gap in 4px steps, documented maximum 16, default 6) and `Justify`, and `MudItem` exposes
`xs`, `sm`, `md`, `lg`, `xl`, `xxl`, plus the usual `Class` / `Style` / `ChildContent`
inherited from the component base.

| MudBlazor | This crate | Notes |
|-----------|-----------|-------|
| `MudGrid.Spacing` | `MudGrid::spacing(u8)` | 4px per unit, clamped to **16** |
| `MudGrid.Justify` | `MudGrid::justify(Justify)` | `Start`, `Center`, `End`, `SpaceBetween`, `SpaceAround`, `SpaceEvenly` |
| `MudGrid.Class` | `MudGrid::classes([...])` | Resolved by the Bootstrap or Tailwind flavor |
| `MudGrid.Style` | `MudGrid::frame(Frame)` | egui has no CSS string; an explicit `egui::Frame` is the equivalent |
| `MudGrid.ChildContent` | closure passed to `MudGrid::show` | |
| `MudItem.xs` ... `xxl` | `MudItem::xs(..)` ... `xxl(..)` | Integers, `"2*"`, `"auto"` |
| `MudItem.Class` / `Style` | `ItemStyle` via `GridUi::styled_item` | Classes or an explicit `Frame` |
| `MudItem.ChildContent` | closure passed to `GridUi::item` | |

**Deliberately not added.** The checklist in `REFACTOR.md` lists `Variant`, `Color`, `Size`,
`Elevation`, `Disabled`, `ReadOnly`, `Value` / `ValueChanged` / `ValueExpression`,
`OnClick` / `OnFocus` / `OnBlur` / `OnKeyDown` / `OnChange`, `AriaLabel` and
`AdornmentStart` / `AdornmentEnd`. Those belong to input and button components. MudBlazor's
grid has none of them, and the instructions at the top of `REFACTOR.md` say not to add what
the real component lacks, so they are not invented here. The grid does expose the egui
equivalent of what its children need: the response covers the whole grid, and
`cell_rects()` gives every item's rectangle for your own hit-testing or focus handling.

---

## Architecture

```
┌──────────────────────────────────────────────────────────────┐
│ mud_grid.rs                                                  │
│                                                              │
│  GridUi ── collects (MudItem, ItemStyle?, content closure)   │
│     │                                                        │
│     ▼                                                        │
│  MudGrid::show                                               │
│    1. resolve frames   (BsStyle / TwStyle, grid + items)     │
│    2. resolve spans    (MudItem::resolve_span at breakpoint) │
│    3. measure Auto     (Probe | Cached)                      │
│    4. plan_into        ◄── pure math, no egui                │
│    5. draw row by row  (egui child Uis, unbounded height)    │
│    6. cache + respond                                        │
└──────────────┬───────────────────────────────┬───────────────┘
               │ uses                          │ uses
               ▼                               ▼
      mud_item.rs  (MudItem)         mud_grid_span.rs (MudGridSpan)
      six spans + cascade            Tracks | Star | Auto | Unset
```

**Pure layer.** `pack_rows_into`, `row_widths_into`, `row_offsets_into` and `plan_into` take
slices and write into caller-owned buffers. They know nothing about egui, which is why
nearly all behaviour is covered by ordinary unit tests and why `MudGrid::plan` can lay out
a grid with no `egui::Context` at all.

**Component layer.** `MudGrid::show` is a thin orchestrator: it resolves styling, calls the
pure layer, and draws.

**Builder layer.** `GridUi` records items first and draws afterwards. Knowing every span
up front is what makes row packing and `Auto` measurement possible.

---

## API reference

### `MudGridSpan`

| Member | Kind | Description |
|--------|------|-------------|
| `Unset` | variant (default) | Not specified here; cascades to the next smaller breakpoint |
| `Tracks(u8)` | variant | Fixed number of tracks, normalized to `1..=12` |
| `Star(f32)` | variant | Proportional share of leftover space; invalid weights become `0.0` |
| `Auto` | variant | Shrink to the content's natural width |
| `FULL`, `HALF`, `THIRD`, `QUARTER`, `SIXTH` | consts | `Tracks(12)`, `(6)`, `(4)`, `(3)`, `(2)` |
| `fixed(u8)` | `const fn` | `Tracks` with the count clamped to `1..=12` |
| `star(f32)` | fn | `Star` constructor |
| `normalized()` | fn | Never returns `Unset`; clamps tracks; repairs weights; idempotent |
| `or(fallback)` | fn | `self` if set, else `fallback` (one step of the cascade) |
| `is_set()`, `is_tracks()`, `is_star()`, `is_auto()` | `const fn` | Variant tests |
| `tracks()`, `weight()` | fn | Payload accessors (`0` / `0.0` for other variants) |
| `track_need()` | fn | Tracks reserved when packing rows (`Star`/`Auto`/`Unset` reserve 1) |
| `parse(&str)` | fn | `Option` parse: `""`, `"auto"`, `"*"`, `"2.5*"`, `"6"` |
| `FromStr` / `Display` | traits | Strict parse (`ParseSpanError`) and a syntax that round-trips |
| `From<i8,i16,i32,i64,isize,u8,u16,u32,u64,usize>` | traits | Integers clamp into `1..=12`, never panic |
| `From<&str>`, `From<String>`, `From<&String>` | traits | Lenient: garbage becomes `Unset` |
| `TRACK_COUNT` | const | `12` |

### `MudItem`

| Member | Description |
|--------|-------------|
| `new()` | All breakpoints unset (full row everywhere) |
| `full()`, `auto()`, `star(w)`, `equal(n)` | Presets that set every breakpoint |
| `xs(..)`, `sm(..)`, `md(..)`, `lg(..)`, `xl(..)`, `xxl(..)` | Set one breakpoint (accepts anything `Into<MudGridSpan>`) |
| `all(..)` | Set every breakpoint, replacing earlier values |
| `at(Breakpoint, ..)` | Set a breakpoint chosen at run time (`None` writes `xs`) |
| `get(Breakpoint)` | Raw stored span, no cascade |
| `ladder()` / `from_ladder([..;6])` | Convert to and from the six-element array |
| `is_empty()` | True when nothing is set |
| `resolve_span(Breakpoint)` | Cascaded, normalized span for one breakpoint |
| `resolve_all()` | The cascade at every breakpoint at once |
| `From<MudGridSpan>` | A bare span becomes an item that uses it everywhere |
| `Display` | `xs=12 md=6 lg=2*` (set breakpoints only) |

Breakpoint ranges (pixels of available width):

| `xs` | `sm` | `md` | `lg` | `xl` | `xxl` |
|------|------|------|------|------|-------|
| 0 | 600 | 960 | 1280 | 1920 | 2560 |

### `MudGrid`

| Member | Default | Description |
|--------|---------|-------------|
| `new()` / `default()` | | MudBlazor defaults |
| `spacing(u8)` | `6` (24px) | Gap in 4px units, clamped to `MAX_SPACING` (16) |
| `justify(Justify)` | `Start` | Distribution of leftover row space |
| `breakpoint(Breakpoint)` | auto | Pin the breakpoint instead of deriving it from width |
| `classes(iter)` | none | Utility classes for the grid's own frame |
| `flavor(StyleFlavor)` | default | Bootstrap or Tailwind resolver |
| `frame(Frame)` | none | Explicit frame; replaces the class-resolved one |
| `auto_measure(AutoMeasure)` | `Probe` | How `Auto` widths are learned |
| `disabled(bool)` | `false` | Suppresses hover/active in Tailwind variants |
| `show(ui, \|grid\| ..)` | | Draw; returns `InnerResponse<R>` |
| `plan(&items, width, &auto)` | | Headless layout, no egui context needed |
| `gap()` | | Gap in pixels |
| `active_breakpoint()` | | Breakpoint of the last frame |
| `last_width()` | | Width laid out against in the last frame |
| `last_cells()` | | `&[CellBox]` from the last frame |
| `cell_rects()` | | `&[Rect]` (screen space) per item from the last frame |

Public fields: `justify`, `spacing`, `breakpoint`, `base`, `flavor`, `bs_style`, `tw_style`,
`interaction`, `frame_override`, `measure`.

### `GridUi`

| Member | Description |
|--------|-------------|
| `item(span, \|ui\| ..)` | Add one item. `span` is a `MudItem` or a bare `MudGridSpan` |
| `styled_item(span, ItemStyle, \|ui\| ..)` | Add one item with its own classes or frame |
| `each(span, iter, \|ui, &value\| ..)` | One item per element, sharing a span (a Razor `@foreach`) |
| `spacer(span)` | An empty item that only occupies space |
| `len()` / `is_empty()` | Items added so far |

### Supporting types

| Type | Description |
|------|-------------|
| `ItemStyle` | `class(..)`, `classes(..)`, `frame(..)`, `is_empty()`; the per-item `Class` / `Style` |
| `AutoMeasure` | `Probe` (exact, closure runs twice for `Auto` cells) or `Cached` (closure runs once) |
| `CellBox` | `{ index, row, x, width }` plus `right()`; one planned cell |
| `InteractionState` | `hovered`, `active`, `focused`, `dark`, `disabled`; feeds Tailwind variants |
| `ParseSpanError` | Returned by strict `FromStr` |
| Constants | `MAX_SPACING` (16), `SPACING_UNIT` (4.0), `FALLBACK_WIDTH` (1280.0), `BREAKPOINT_SM` .. `BREAKPOINT_XXL` |

### Visual variants

The grid draws no chrome of its own. Its appearance is whatever its frame resolves to:
`classes(..)` through the Bootstrap or Tailwind flavor, or an explicit `frame(..)`. State
variants (hover, active, dark) come from `InteractionState`, captured from the previous
frame's response.

---

## Examples

### Responsive cards

```rust
MudGrid::new().spacing(4).show(ui, |grid| {
    for n in 0..6 {
        grid.item(MudItem::new().xs(12).sm(6).lg(4), move |ui| {
            ui.group(|ui| { ui.label(format!("Card {n}")); });
        });
    }
});
```

Full width on phones, two across from 600px, three across from 1280px.

### Sidebar plus content using stars

```rust
MudGrid::new().show(ui, |grid| {
    grid.item(MudItem::new().xs(12).md(3), |ui| { ui.label("Sidebar"); });
    grid.item(MudItem::new().xs(12).md("*"), |ui| { ui.label("Content"); });
});
```

On `md` and up the sidebar takes 3 tracks and the content takes everything left.

### Styled items

```rust
let card = ItemStyle::new().classes(["bg-surface", "p-4", "rounded-lg"]);

MudGrid::new().flavor(StyleFlavor::Tailwind).show(ui, |grid| {
    grid.styled_item(MudItem::new().xs(12).md(6), card.clone(), |ui| { ui.label("A"); });
    grid.styled_item(MudItem::new().xs(12).md(6), card,         |ui| { ui.label("B"); });
});
```

### Iterating data

```rust
let people = ["Ada", "Grace", "Linus"];
MudGrid::new().show(ui, |grid| {
    grid.each(MudItem::new().xs(12).sm(4), people, |ui, name| { ui.label(*name); });
});
```

### Centering a row of fixed-width buttons

```rust
MudGrid::new().justify(Justify::Center).show(ui, |grid| {
    grid.item(MudItem::auto(), |ui| { ui.button("Save"); });
    grid.item(MudItem::auto(), |ui| { ui.button("Cancel"); });
});
```

### Reacting to the active breakpoint

```rust
let mut grid = MudGrid::new();           // keep it across frames
grid.show(ui, |g| { /* ... */ });
if matches!(grid.active_breakpoint(), Some(Breakpoint::Xs)) {
    // render a compact header
}
```

### Headless layout (tests, tooling, snapshotting)

```rust
let items = [MudItem::new().xs(12).md(6); 2];
let cells = MudGrid::new().plan(&items, 1200.0, &[]);
assert_eq!((cells[1].row, cells[1].x, cells[1].width), (0, 612.0, 588.0));
```

### Parsing spans from configuration

```rust
let span: MudGridSpan = "2*".parse()?;               // strict: Err on garbage
let lenient = MudItem::new().md(some_user_string);   // lenient: garbage is Unset
```

### Opting into cached `Auto` measurement

```rust
MudGrid::new().auto_measure(AutoMeasure::Cached).show(ui, |grid| {
    grid.item(MudItem::auto(), |ui| { ui.button("Expensive to build"); });
    grid.item(MudItem::star(1.0), |ui| { ui.label("Fills the rest"); });
});
```

---

## Layout algorithm

For a grid `W` pixels wide with gap `g` (`spacing * 4`):

1. **Track width.** `t = max(0, (W - 11g) / 12)`. A cell spanning `n` tracks is
   `n*t + (n-1)*g` wide, so twelve tracks plus eleven gaps fill `W` exactly.
2. **Resolve spans.** Each item resolves its span at the active breakpoint (below), walking
   *down* the ladder to the first set value; nothing set means 12 tracks.
3. **Pack rows.** Items fill a row until the next would exceed 12 tracks. `Star` and `Auto`
   reserve one track each. Every row holds at least one item.
4. **Size each row.** `Tracks` and `Auto` cells are fixed first. The remainder
   (`W` minus fixed widths minus gaps, never negative) is split among `Star` cells by weight,
   or equally when every weight is zero.
5. **Justify each row.** Leftover space moves cells according to `Justify`
   (`Center` = half, `End` = all, `SpaceBetween` = spread between, `SpaceAround` /
   `SpaceEvenly` as in CSS flexbox).
6. **Draw.** Cells are child `Ui`s with unbounded height; a row is as tall as its tallest
   cell, and rows are separated by `g`.

The active breakpoint comes from the width **available to the grid**, not the OS window, so
a grid inside a narrow side panel behaves as a narrow grid. A non-finite available width
(e.g. inside a horizontal scroll area) falls back to 1280px.

---

## Performance and memory layout

| Choice | Effect |
|--------|--------|
| `MudGridSpan` is `Copy`, 8 bytes (`u8`/`f32` payload plus tag) | Passed by value, no refcounts |
| `MudItem` is `Copy`, six spans = 48 bytes | Cheap to build every frame |
| Cascade resolution walks a fixed `[MudGridSpan; 6]` on the stack | Resolving a span never allocates |
| Layout math writes into caller-owned `Vec`s (`*_into` functions) | No per-row allocation |
| `MudGrid` keeps a private scratch arena (spans, auto widths, measured widths, rows, widths, offsets, cell boxes, rectangles) reused every frame | Steady-state frames do not re-grow those buffers |
| Pure layout core with no egui types | Unit-testable and usable headless |
| `AutoMeasure::Cached` | `Auto` closures run once per frame instead of twice |

**What still allocates per frame.** The item list is rebuilt every frame because immediate-mode
closures cannot be retained across frames: that is one `Box` per item, the `Vec` holding them,
and the `Vec` of resolved item frames. Styled items additionally build a small
`ComponentBase` while their frame is resolved. Everything else is reused.

**About `Cached`.** In this mode a cell is probed only when no previous width is stored. After
each frame the real widths are compared with the cache; a change larger than half a pixel is
stored and triggers one repaint, so content that resizes lags by at most one frame. Cached
widths live in egui's temp data keyed by the grid's `Ui` id. `Probe` remains the default
because it is always exact.

**Threads.** egui is single-threaded immediate mode and `MudGrid::show` takes `&mut self`, so
no internal locking exists or is needed. `MudItem`, `MudGridSpan` and `CellBox` are plain
`Copy` data.

These figures describe structure, not measurements: no benchmarks were run.

---

## Design rationale

- **Gaps instead of negative margins.** Current MudBlazor implements spacing with CSS `gap`
  (formerly negative margins plus padding). Gaps are the natural model for egui too: cells get
  exact rectangles and nothing overflows the container.
- **Spans as an enum, not a number.** `Tracks`, `Star` and `Auto` are different sizing
  disciplines. An enum makes illegal states unrepresentable and keeps the layout `match`
  exhaustive.
- **Never panic in a UI path.** Out-of-range integers clamp, bad strings become `Unset`, NaN
  and infinite weights become `0.0`, non-finite widths fall back to 1280px, non-finite heights
  count as `0`. Strict parsing (`FromStr`) is available when you want errors.
- **Lenient builders, strict parsers.** `item.md("garbage")` does not stop a frame from
  drawing; `"garbage".parse::<MudGridSpan>()` tells you.
- **Plan, then draw.** Separating planning from drawing is what allows headless tests,
  `cell_rects()`, and a single pass over the pure math per frame.
- **Container breakpoints.** Panels, docks and split views are common in egui, so the
  default follows the grid's own width. `breakpoint(..)` pins it when you want
  window-style behaviour.
- **No invented MudBlazor API.** See the first section.

---

## Behaviour changes from the previous version

- **`MAX_SPACING` is now 16** (was 20), matching MudBlazor's documented maximum. A grid
  asking for `spacing(20)` is now clamped to 16 (64px).
- **`disabled` is no longer reset every frame.** The old code wrote `disabled = false` at the
  end of each `show`, so the flag could never take effect. It now persists until changed.
- **The grid's response rectangle ends at `min.x + width`**, not the raw available right edge,
  so an unbounded parent no longer produces an infinite rectangle.
- **`GridUi::item` accepts `impl Into<MudItem>`.** Existing calls passing a `MudItem` compile
  unchanged.
- **`ladder_rank` is `pub(crate)`** rather than private, and `ParseSpanError`, `MudGridSpan`
  and `MudItem` keep their existing behaviour and tests.
- **New integer conversions** (`i8`, `i16`, `i64`, `isize`, `u64`) and `String` /
  `&String` conversions on `MudGridSpan`.

---

## Testing

Each file ends with a `#[cfg(test)] mod tests` block.

| File | Covers |
|------|--------|
| `mud_grid_span.rs` | defaults, presets, `fixed`, `or`, `track_need`, normalization (idempotent, never `Unset`, NaN), parsing (valid, garbage, overflow, mixed), `Display` round-trips, every integer and string conversion |
| `mud_item.rs` | cascade rules, normalization, every setter, `all`, `at`, `get`, ladder round-trip, presets, `resolve_all` agreeing with `resolve_span`, `Display` |
| `mud_grid.rs` | breakpoint edges, track and gap math, row packing, star/auto sizing, degenerate inputs, justification, buffer reuse, the planner (`plan_into`, `MudGrid::plan`) across breakpoints, justify, spacing, fallback width and invariants (no overlap, no overflow, in-order cells), `GridUi` / `ItemStyle` builders, `MudGrid` builders and defaults, interaction state |

```text
cargo test
```

The planner tests are the "visual layout" tests: they assert exact x positions and widths
for each breakpoint and justify mode. Drawing itself (`MudGrid::show`) needs a live egui
context and is not unit-tested.