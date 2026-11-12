# MudTimeline for `mud_egui`

A chronological timeline for [egui](https://github.com/emilk/egui), ported
from MudBlazor's [`MudTimeline`](https://mudblazor.com/components/timeline)
and `MudTimelineItem` (MudBlazor 8.13.0).

```rust
let mut state = TimelineState::new();

MudTimeline::new()
.timeline_position(TimelinePosition::Alternate)
.timeline_orientation(TimelineOrientation::Vertical)
.show(ui, &mut state, |t| {
t.item(TimelineItem::text("Order placed").icon(filled::SHOPPING_CART));
t.item(
TimelineItem::text("Shipped")
.color(Color::Success)
.variant(Variant::Filled),
);
t.item(TimelineItem::text("Delivered").icon(filled::CHECK));
});
```

Every string renders through **`MudText`**, every glyph through **`MudIcon`**.

---

## Contents

1. [Design principle: API parity, not API invention](#design-principle-api-parity-not-api-invention)
2. [Architecture](#architecture)
3. [API reference](#api-reference)
4. [Examples](#examples)
5. [Layout and rendering](#layout-and-rendering)
6. [Performance and memory choices](#performance--memory-choices)
7. [Testing](#testing)
8. [Differences from MudBlazor](#differences-from-mudblazor)

---

## Design principle: API parity, not API invention

The MudBlazor 8.13.0 parameter surface was verified against the DNDocs pages
for `MudTimeline`, `MudTimelineItem`, `TimelineAlign`, `TimelineOrientation`,
and `TimelinePosition` before implementation.

Three new public enums (`TimelineAlign`, `TimelineOrientation`,
`TimelinePosition`) live in `mud_egui_core::enums`, each in its own file,
matching the project convention. No other enums are introduced.

### Ported `MudTimeline` parameters

| MudBlazor | Rust | Default |
|---|---|---|
| `Modifiers` | `modifiers(bool)` | `true` |
| `Reverse` | `reverse(bool)` | `false` |
| `RightToLeft` (cascading) | `right_to_left(bool)` | `false` |
| `SelectedIndex` | `TimelineState::selected_index` | `None` |
| `SelectedIndexChanged` | `on_selected_index_change(FnMut(usize))` | `None` |
| `TimelineAlign` | `timeline_align(TimelineAlign)` | `Default` |
| `TimelineOrientation` | `timeline_orientation(TimelineOrientation)` | `Vertical` |
| `TimelinePosition` | `timeline_position(TimelinePosition)` | `Alternate` |
| `ChildContent` | closure to `show` | required |
| `Class` | `classes(iter)` | `None` |
| `Style` | `frame(egui::Frame)` | `None` |
| `Tag` | `tag(t)` / `tag_ref::<U>()` | `None` |
| `UserAttributes` | **not ported** (no HTML tag) | — |

### Ported `MudTimelineItem` parameters

| MudBlazor | Rust | Default |
|---|---|---|
| `Color` | `TimelineItem::color(Color)` | `Default` |
| `Variant` | `TimelineItem::variant(Variant)` | `Outlined` |
| `Size` | `TimelineItem::size(Size)` | `Small` |
| `HideDot` | `TimelineItem::hide_dot(bool)` | `false` |
| `Icon` | `TimelineItem::icon(svg)` | `None` |
| `ItemDot` | `TimelineItem::dot(FnMut(&mut Ui))` | `None` |
| `ItemContent` | `TimelineItem::new(FnMut(&mut Ui))` | required |
| `ItemOpposite` | `TimelineItem::opposite(FnMut(&mut Ui))` | `None` |
| `TimelineAlign` | `TimelineItem::timeline_align(TimelineAlign)` | `None` (inherit) |
| `DotStyle` / `DotClass` | **not ported** (no CSS strings) | — |

### Enums

**`TimelineAlign`** — `Default` (centered), `Start`, `End`.

**`TimelineOrientation`** — `Vertical` (default), `Horizontal`.

**`TimelinePosition`** — `Alternate` (default), `Top`, `Bottom`, `Left`,
`Right`, `Start`, `End`.

Valid combinations:

| Orientation | Valid positions |
|---|---|
| `Vertical` | `Left`, `Right`, `Alternate`, `Start`, `End` |
| `Horizontal` | `Top`, `Bottom`, `Alternate` |

Cross-axis values are coerced to `Alternate` by `resolve_position`.

### Deliberately not ported

- **`Modifiers`** is stored for parity but has no visual effect (egui has no
  CSS pseudo-elements).
- **`DotStyle` / `DotClass`** — use `TimelineItem::dot(..)` for a fully
  custom dot renderer.
- **`UserAttributes`** — there is no HTML tag.

### Navigation helpers

`MudTimeline::move_to(state, index, count)`, `MudTimeline::next(state, count)`,
`MudTimeline::previous(state, count)` mirror `MudBaseItemsControl`'s
`MoveTo(int)`, `Next()`, `Previous()`.

---

## Architecture

```
mud_timeline.rs
├── MudTimeline<'a>        — configuration + render entry point
│   └── show(ui, state, |t| ..) -> (Response, R)
├── Cfg                    — immutable per-frame config
├── render_vertical(..)    — vertical axis renderer
├── render_horizontal(..)  — horizontal axis renderer
├── paint_dot(..)          — default / custom dot
├── Pure helpers           — dot_diameter, dot_span, alternate_side,
│                            item_sides, resolve_position, effective_align,
│                            dot_y_offset, dot_x_offset
└── tests                  — 23 pure-function tests

timeline_item.rs           — TimelineItem<'a> (data model)
timeline_state.rs          — TimelineState (caller-owned selection)
timeline_ui.rs             — TimelineUi<'a> (collector)
```

**Pure layer.** Every layout decision is a free function, unit-tested without
an `egui::Context`.

**State layer.** `TimelineState` is caller-owned — the egui equivalent of
`SelectedIndex`.

**Component layer.** `show` resolves the theme and frame, calls the pure layer,
and paints.

---

## API reference

### `MudTimeline<'a>`

| Method | Default | Mirrors | Description |
|---|---|---|---|
| `new()` | — | — | MudBlazor defaults. |
| `modifiers(bool)` | `true` | `Modifiers` | Stored for parity; no visual effect. |
| `reverse(bool)` | `false` | `Reverse` | Flip the alternate side order. |
| `right_to_left(bool)` | `false` | `RightToLeft` | Resolve `Start`/`End` for RTL. |
| `timeline_position(TimelinePosition)` | `Alternate` | `TimelinePosition` | Where items sit relative to the axis. |
| `timeline_orientation(TimelineOrientation)` | `Vertical` | `TimelineOrientation` | Axis direction. |
| `timeline_align(TimelineAlign)` | `Default` | `TimelineAlign` | Default dot alignment. |
| `selectable(bool)` | `true` | — | Rows write `selected_index`. |
| `on_selected_index_change(FnMut(usize))` | `None` | `SelectedIndexChanged` | Click callback. |
| `on_item_click(FnMut(usize))` | alias | — | Alias for the above. |
| `height(f32)` | `None` | `Style="max-height:…"` | Vertical scroll cap. |
| `item_width(f32)` | `240.0` | — | Horizontal item width. |
| `item_height(f32)` | `200.0` | — | Horizontal item height. |
| `classes(iter)` | none | `Class` | Wrapper utility classes. |
| `flavor(StyleFlavor)` | default | — | Bootstrap / Tailwind. |
| `frame(egui::Frame)` | `None` | `Style` | Replace the wrapper frame. |
| `aria_label(s)` | `None` | — | Accessible label. |
| `tag(t)` / `tag_ref::<U>()` | `None` | `Tag` | Arbitrary data. |
| `show(ui, state, \|t\| ..)` | — | — | Returns `(Response, R)`. |

### `TimelineItem<'a>`

| Method | Mirrors | Description |
|---|---|---|
| `new(FnMut(&mut Ui))` | `ItemContent` | Required content closure. |
| `text(s)` | `ItemContent` (text) | Convenience text content. |
| `color(Color)` | `Color` | Dot accent. |
| `variant(Variant)` | `Variant` | `Outlined` / `Filled` / `Text`. |
| `size(Size)` | `Size` | `Small` (12px) / `Medium` (16px) / `Large` (20px). |
| `hide_dot(bool)` | `HideDot` | Hide the dot, keep the line. |
| `icon(svg)` | `Icon` | Icon inside the dot. |
| `dot(FnMut(&mut Ui))` | `ItemDot` | Custom dot renderer. |
| `content(FnMut(&mut Ui))` | `ItemContent` | Replace the content. |
| `opposite(FnMut(&mut Ui))` | `ItemOpposite` | Opposite-side content (Alternate only). |
| `timeline_align(TimelineAlign)` | `TimelineAlign` | Per-item alignment override. |
| `with_check_icon()` | — | Convenience: Material check icon. |

### `TimelineState`

| Field | Description |
|---|---|
| `selected_index: Option<usize>` | Currently selected item. |
| `last_item_rects: Vec<Rect>` | Item row rectangles from the last frame. |
| `last_dot_centers: Vec<Pos2>` | Dot centres from the last frame. |

### Geometry constants

| Name | Value |
|---|---|
| `DOT_SMALL` | `12.0` |
| `DOT_MEDIUM` | `16.0` |
| `DOT_LARGE` | `20.0` |
| `DOT_PADDING` | `8.0` |
| `ITEM_GAP` | `8.0` |
| `CONNECTOR_THICKNESS` | `2.0` |
| `DEFAULT_ITEM_WIDTH` | `240.0` |
| `DEFAULT_ITEM_HEIGHT` | `200.0` |

---

## Examples

### Vertical, left-aligned

```rust
MudTimeline::new()
    .timeline_position(TimelinePosition::Left)
    .show(ui, &mut state, |t| {
        t.item(TimelineItem::text("Step 1").color(Color::Success).variant(Variant::Filled));
        t.item(TimelineItem::text("Step 2").color(Color::Success).variant(Variant::Filled));
        t.item(TimelineItem::text("Step 3"));
    });
```

### Vertical, alternating

```rust
MudTimeline::new()
    .timeline_position(TimelinePosition::Alternate)
    .show(ui, &mut state, |t| {
        t.item(TimelineItem::text("Jan 2024").opposite(|ui| {
            ui.add(MudText::new("Order placed").typo(Typo::Body2));
        }));
        t.item(TimelineItem::text("Feb 2024").opposite(|ui| {
            ui.add(MudText::new("Shipped").typo(Typo::Body2));
        }));
    });
```

### Horizontal

```rust
MudTimeline::new()
    .timeline_orientation(TimelineOrientation::Horizontal)
    .timeline_position(TimelinePosition::Top)
    .item_width(220.0)
    .item_height(160.0)
    .show(ui, &mut state, |t| {
        t.item(TimelineItem::text("Q1"));
        t.item(TimelineItem::text("Q2"));
        t.item(TimelineItem::text("Q3"));
    });
```

### Reverse alternate

```rust
MudTimeline::new()
    .timeline_position(TimelinePosition::Alternate)
    .reverse(true) // first item goes right instead of left
    .show(ui, &mut state, |t| { /* … */ });
```

### Sizes and variants

```rust
TimelineItem::text("Small outlined")   // defaults: Size::Small, Variant::Outlined
TimelineItem::text("Medium filled").size(Size::Medium).variant(Variant::Filled)
TimelineItem::text("Large").size(Size::Large).icon(filled::STAR)
```

### Custom dot

```rust
TimelineItem::text("Custom dot").dot(|ui| {
    ui.add(MudAvatar::new().text("A").size(Size::Small));
})
```

### HideDot (continuation line)

```rust
TimelineItem::text("continued...").hide_dot(true)
```

### Scrollable vertical timeline

```rust
MudTimeline::new()
.height(400.0)
.show(ui, &mut state, |t| { /* many items */ });
```

### Selection callback

```rust
MudTimeline::new()
.on_selected_index_change(|i| println!("selected {i}"))
.show(ui, &mut state, |t| { /* … */ });

// After the frame, `state.selected_index` holds the clicked index.
```

### Per-item alignment

```rust
MudTimeline::new()
    .timeline_align(TimelineAlign::Default)
    .show(ui, &mut state, |t| {
        t.item(TimelineItem::text("centered")); // inherits Default
        t.item(TimelineItem::text("top").timeline_align(TimelineAlign::Start));
        t.item(TimelineItem::text("bottom").timeline_align(TimelineAlign::End));
    });
```

---

## Layout and rendering

### Vertical

Each item is a horizontal band:

| Position | Row layout |
|---|---|
| `Left` / `Start` | `[ dot ][ content ]` |
| `Right` / `End` | `[ content ][ dot ]` |
| `Alternate` | `[ content \| opposite ][ dot ][ opposite \| content ]` |

Content height is natural (measured by egui). The dot's Y position within the
row is controlled by `TimelineAlign`:

| Align | Dot Y |
|---|---|
| `Default` | row centre |
| `Start` | `DOT_PADDING + diameter/2` from the top |
| `End` | `DOT_PADDING + diameter/2` from the bottom |

### Horizontal

Each item is a fixed-size box (`item_width` × `item_height`):

| Position | Item layout |
|---|---|
| `Top` | `[ dot row ][ content ]` |
| `Bottom` | `[ content ][ dot row ]` |
| `Alternate` | `[ content \| opposite ][ dot row ][ opposite \| content ]` |

The dot's X position within the box is controlled by `TimelineAlign`:

| Align | Dot X |
|---|---|
| `Default` | box centre |
| `Start` | `DOT_PADDING + diameter/2` from the left |
| `End` | `DOT_PADDING + diameter/2` from the right |

### Connector

A single straight line runs through every dot centre, from the first to the
last. It is painted **behind** items by reserving a shape slot with
`ui.painter().add(Shape::Noop)` before rendering and filling it with
`ui.painter().set(slot, Shape::line_segment(..))` afterwards. This is the same
pattern used by `MudToggleGroup`'s underline fill and `MudButton`'s underfill.

### Dot rendering

| Variant | Appearance |
|---|---|
| `Outlined` (default) | surface fill + 2 px accent stroke |
| `Filled` | accent fill, no stroke |
| `Text` | accent fill (no text inside a dot) |

`Icon` renders inside the dot, colour = contrast text (Filled/Text) or accent
(Outlined). `ItemDot` overrides the entire default circle.

### `ItemOpposite`

Only rendered when `TimelinePosition::Alternate`, matching MudBlazor issue
#11799. In non-alternate positions the opposite closure is ignored.

---

## Performance and memory choices

| Choice | Effect |
|---|---|
| Items are collected once per frame and rendered once | No retained render tree. |
| `TimelineState` is caller-owned | Selection survives frames. |
| Connector uses a single reserved `Shape` slot | One shape instead of N segments. |
| Dots are painted after the row layout | No chicken-and-egg for alignment. |
| `dot_span` is a pure function | Column width is fixed before rendering. |
| All text through `MudText`, icons through `MudIcon` | Theme/font updates propagate. |
| Horizontal items use fixed boxes | Deterministic connector Y. |
| No per-frame `Vec` beyond `items` and `last_*` | Steady-state frames allocate nothing else. |

Not done, on purpose: natural-width horizontal items. Doing so would require
measuring every item's content before laying out, which is what
`MudGrid::AutoMeasure::Probe` does for grid cells; horizontal timelines in
MudBlazor are effectively fixed-size carousels, so fixed boxes are the
faithful choice.

---

## Testing

`cargo test` runs 23 pure-function tests:

| Area | Tests |
|---|---|
| Dot geometry | `dot_diameter` for every `Size`, `dot_span` adds padding |
| Alternate sides | alternation, reverse flips |
| Item sides | vertical Left/Right, horizontal Top/Bottom, alternate opposites |
| Position resolution | cross-axis coercion, `Start`/`End` normalisation with RTL |
| Alignment | `effective_align` prefers item override, `dot_y_offset`/`dot_x_offset` for every align |
| Defaults | match MudBlazor for every field |
| Builders | every field set, `item_width` clamp, `height` clamp |
| Navigation | `move_to` clamps, `next`/`previous` wrap |
| Tag | round-trips by type |

Rendering (`show`) needs a live `egui::Context` and is not unit-tested; the
pure layer is the source of truth.

```bash
cargo test -p mud_egui_components mud_timeline
```

---

## Differences from MudBlazor

- **`Modifiers` has no effect.** egui has no CSS pseudo-elements; the
  parameter is stored for parity.
- **`DotStyle` / `DotClass` not ported.** Use `TimelineItem::dot(..)`.
- **Horizontal items are fixed-size.** MudBlazor's horizontal timeline is a
  CSS flex row where items size to content; egui cannot measure all items
  before laying out without a two-pass probe. `item_width` / `item_height`
  give the caller the same control without the cost.
- **Selection is opt-in.** MudBlazor's `SelectedIndex` is a bindable
  parameter; this port writes it on click when `selectable(true)` (default).
  Set `selectable(false)` to make the timeline display-only.
- **`ItemOpposite` is only rendered in `Alternate`.** Same as MudBlazor
  (issue #11799).
- **Cross-axis positions are coerced to `Alternate`.** MudBlazor renders an
  empty or broken timeline; the port normalises to `Alternate` so the result
  is always usable.
- **Connector line is always drawn.** MudBlazor's `HideDot` hides the dot but
  keeps the line; the port matches that.