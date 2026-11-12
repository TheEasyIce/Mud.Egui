# MudStack (egui port)

An immediate-mode port of MudBlazor's [`MudStack`](https://mudblazor.com/components/stack) — a
single-axis layout container that arranges its children in a row or column with a uniform gap,
optional main-axis justification, cross-axis alignment, wrapping, reversal, responsive
row/column collapse, and leftover-space distribution.

It targets [`egui`](https://github.com/emilk/egui) and is built to feel the way `MudStack` feels
in Blazor, while being honest about where an immediate-mode, single-pass UI framework cannot
reproduce retained-mode/CSS flexbox behavior exactly. Those divergences are documented, not
hidden — see [How it differs from CSS flexbox](#how-it-differs-from-css-flexbox).

> **Scope note.** `MudStack` in real MudBlazor is a pure layout primitive: it has no visual
> surface, no click target, no elevation, and no adornment slots. This port deliberately does
> **not** add `Variant`, `Color`, `Elevation`, `OnClick`/`OnFocus`/`OnKeyDown`, `ReadOnly`,
> ripple/focus rings, or `AdornmentStart`/`AdornmentEnd` — those parameters belong to MudBlazor's
> buttons, inputs, and papers, not to `MudStack`. Every parameter below was checked against
> MudBlazor's actual, merged source and docs before being ported.

---

## Table of contents

- [Quick start](#quick-start)
- [Architecture](#architecture)
- [API reference](#api-reference)
- [Examples](#examples)
- [Performance and memory layout](#performance-and-memory-layout)
- [How it differs from CSS flexbox](#how-it-differs-from-css-flexbox)
- [Testing](#testing)

## Quick start

```rust
// Plain usage: `StackUi` derefs to `Ui`, so just draw children.
MudStack::new().spacing(2).show(ui, |ui| {
ui.label("one");
ui.label("two");
});
```

```rust
// Justify::Space* and StretchItems need to know where each item begins and ends,
// so wrap each child in `StackUi::item` inside the same `show` call.
MudStack::horizontal()
    .justify(Justify::SpaceBetween)
    .align_items(AlignItems::Center)
    .classes(["p-2", "bg-primary"])
    .show(ui, |stack| {
        stack.item(|ui| { ui.label("left"); });
        stack.item(|ui| { ui.label("middle"); });
        stack.item(|ui| { ui.button("right"); });
    });
```

## Architecture

The file is deliberately split into two layers:

```
┌─────────────────────────────────────────────────────────────┐
│  Render path (needs a live egui::Context)                    │
│  MudStack::show → run → run_inner                     │
│  StackUi::item                                                │
│                                                                │
│  Gathers inputs from the Ui (available rect, previous-frame   │
│  Measure, previous-frame item extents), calls the pure        │
│  helpers below for every decision, and applies the results    │
│  (Layout, Rect, item_spacing) to a scoped child Ui.            │
└─────────────────────────────────────────────────────────────┘
                              │ calls
                              ▼
┌─────────────────────────────────────────────────────────────┐
│  Pure helpers (no Ui, no Context — unit-tested directly)      │
│  resolve_is_row    · plan_rect        · plan_offsets          │
│  cross_align       · build_layout     · item_spacing          │
│  should_break_line · advance_line     · extents_approx_eq     │
│  stretch_targets   · plan_stretch     · stretched_item_rect   │
└─────────────────────────────────────────────────────────────┘
```

This keeps every piece of layout arithmetic testable without constructing an `egui::Context`,
and keeps the render path itself thin — it's mostly plumbing between the `Ui` and the helpers.

**Component (`MudStack`)** — the builder. Holds configuration only; `#[must_use]` so a stack
built and never `.show()`n is a compile-time warning, not a silent no-op.

**`StackUi`** — the handle passed into `show`. Wraps the stack's child `Ui` (and derefs to it,
so plain widgets just work) and accumulates the main-axis extent of every
[`StackUi::item`](#stackuiitem), which is what makes `Justify::Space*` and `StretchItems`
possible — both need item *boundaries*, which widgets added directly to the `Ui` don't expose.

**`Measure`** — a small `Copy` struct persisted in egui's temp memory between frames (content
extent, item-sum extent, item count). This is the mechanism behind every "needs the previous
frame's size" feature: `Center`/`End`/`Space*` justification, reversed-column anchoring,
wrap-aware line breaking, and `StretchItems`.

## API reference

### `MudStack` — builder parameters

| Parameter | Type | Default | Builder method | Notes |
|---|---|---|---|---|
| `base` | `ComponentBase` | `ComponentBase::new()` | `.classes(...)` | Utility classes (`p-*`, `m-*`, `bg-<role>`); theme/padding are only resolved if at least one is recognised. |
| `row` | `bool` | `false` | `.row(bool)` | Row vs. column. `MudStack::horizontal()` / `::vertical()` are shorthands. |
| `reverse` | `bool` | `false` | `.reverse(bool)` | `row-reverse` / `column-reverse`. |
| `spacing` | `u8` | `3` (12px) | `.spacing(u8)` | Gap in 4px units, clamped to `0..=20` (both in the builder and again at render time). |
| `align_items` | `AlignItems` | `Stretch` | `.align_items(...)` | Cross-axis alignment: `Start`, `Center`, `End`, `Stretch`. |
| `justify` | `Justify` | `Start` | `.justify(...)` | Main-axis distribution: `Start`, `Center`, `End`, `SpaceBetween`, `SpaceAround`, `SpaceEvenly`. Anything but `Start` needs a known main length (a row's available width, or a column's `.height(..)`) and, for `Center`/`End`/`Space*`, a previous-frame measurement. |
| `wrap` | `Wrap` | `NoWrap` | `.wrap(...)` | `NoWrap` / `Wrap`. Disables `Justify` other than `Start` while active. |
| `breakpoint` | `Breakpoint` | `None` | `.breakpoint(...)` | A row collapses to a column once the container is narrower than this breakpoint. Matches MudBlazor's real, merged semantics: "the state of `Row` is reversed based on the defined screen size breakpoint." |
| `height` | `Option<f32>` | `None` | `.height(f32)` | Fixed height. Required for a column to have main-axis free space to justify, wrap, or stretch into. |
| `stretch_items` | `StretchItems` | `None` | `.stretch_items(...)` | Which items absorb leftover main-axis space. Only affects children added with `StackUi::item`. See [`StretchItems`](#stretchitems). |
| `id_salt` | `Option<Id>` | `None` | `.id_salt(impl Hash + Debug)` | Disambiguates the memory key two stacks would otherwise share (and therefore the measurements they remember). |

### Constructors

| Function | Equivalent to |
|---|---|
| `MudStack::new()` | `MudStack::default()` — vertical, un-reversed, `spacing = 3`, `Stretch`, `Start`. |
| `MudStack::horizontal()` | `MudStack::new().row(true)` |
| `MudStack::vertical()` | `MudStack::new().row(false)` (same as `new()`) |

### Rendering

| Method | Signature | Use when |
|---|---|---|
| `show` | `fn show<R>(self, ui: &mut Ui, add_children: impl FnOnce(&mut StackUi<'_>) -> R) -> InnerResponse<R>` | Always (MudBlazor's `ChildContent`). Add plain widgets directly, or wrap them in `StackUi::item` when you need `Space*` justification, `StretchItems`, or wrapping of composite children. |
| `ui` (via `impl Widget for MudStack`) | — | Rendering an *empty* stack, e.g. purely for its utility-class frame (a coloured spacer). |

### `StackUi<'a>`

| Method | Returns | Notes |
|---|---|---|
| `item(add_contents)` | `InnerResponse<R>` | Adds one child (which may itself contain several widgets) as a single measured, unbreakable unit. While wrapping, starts a new line first if the previous frame's extent says it no longer fits. While `stretch_items` is active, widens the item's `Ui` before drawing it so content that fills `available_width()/height()` can grow into the leftover space. |
| `is_row()` | `bool` | Whether the stack resolved to a row after any breakpoint collapse. |
| `Deref<Target = Ui>` | `&Ui` / `&mut Ui` | `StackUi` derefs to the child `Ui`, so `stack.label(..)` and `render(stack, ..)` work. Widgets added this way are **not** tracked as items. |
| `ui()` | `&mut Ui` | Explicit escape hatch (same as deref); widgets added this way are **not** tracked as items (no `Space*`/`StretchItems` participation). |

### `StretchItems`

Port of MudBlazor's `StretchItems` ([`MudBlazor#8545`](https://github.com/MudBlazor/MudBlazor/pull/8545)).

| Variant | Behavior |
|---|---|
| `None` *(default)* | No item receives extra space. |
| `Start` | Only the first item grows. |
| `End` | Only the last item grows. |
| `StartAndEnd` | First and last items each grow by half the leftover space. |
| `Middle` | Every item except the first and last grows, sharing the leftover evenly. A no-op with 0, 1, or 2 items (there is no middle). |
| `All` | Every item grows, sharing the leftover evenly. |

The leftover space is computed from the *previous* frame's per-item extents (the same
frame-delay `Justify::Space*` already relies on), divided evenly among the flagged positions,
and handed to each flagged item as extra room in its `Ui`'s `max_rect` before it's drawn. What
the item does with that room is up to it: a child that fills `ui.available_width()` (or
`available_height()` in a column) visibly grows; a fixed-size child — a plain `MudButton`, say —
simply ignores the slack. This matches MudBlazor's real behavior, where stretching is likewise
opt-in per child rather than forced.

Falls back to a clean no-op (nothing resized) when: it's the first frame (nothing measured yet),
the stack is wrapping (no single leftover to distribute across multiple lines), the main length
is unknown or infinite, or the content already overflows the available space.

## Examples

### Toolbar: pinned ends, stretched middle

```rust
MudStack::horizontal()
    .stretch_items(StretchItems::Middle)
    .show(ui, |stack| {
        stack.item(|ui| { ui.heading("Inbox"); });
        stack.item(|ui| { ui.add_space(ui.available_width()); }); // grows to fill
        stack.item(|ui| { ui.button("Compose"); });
    });
```

### Responsive row that collapses to a column on narrow containers

```rust
MudStack::horizontal()
    .breakpoint(Breakpoint::Md)
    .spacing(4)
    .align_items(AlignItems::Center)
    .show(ui, |ui| {
        ui.label("Narrow? I'm a column now.");
        ui.button("Action");
    });
```

### Wrapping a row of cards

```rust
MudStack::horizontal()
    .wrap(Wrap::Wrap)
    .spacing(4)
    .show(ui, |stack| {
        for card in &cards {
            stack.item(|ui| render_card(ui, card));
        }
    });
```

### Centered, fixed-height column

```rust
MudStack::vertical()
    .height(240.0)
    .justify(Justify::Center)
    .align_items(AlignItems::Center)
    .show(ui, |ui| {
        ui.label("Centered both ways.");
    });
```

## Performance and memory layout

- **No heap allocation on the hot path.** `Measure` is a small `Copy` struct stored in egui's
  temp memory — reading and writing it costs a handful of words, not an allocation. No
  intermediate collections are built for a plain `show` call.
- **`classes(...)` is opt-in cost.** Utility-class parsing and theme lookup only run when
  `classes(..)` produced at least one recognised utility; the zero-utility case is a direct tail
  call into the render path, skipping the `Frame`/theme machinery entirely.
- **Wrapping and `StretchItems` pay only when used.** The previous-frame per-item extents
  (`Vec<f32>`) are only read/cloned when `wrap` or `stretch_items` is active; a plain non-item,
  non-wrapping stack never touches that memory key.
- **One repaint, not every frame.** Measurements are compared with an epsilon
  (`MEASURE_EPSILON` / `LINE_EPSILON`) rather than exact equality, so sub-pixel float noise from
  egui's layout doesn't trigger a repaint loop — a changed measurement requests exactly one
  corrective repaint.
- **Arithmetic is isolated and `Copy`-friendly.** Every layout decision — rect planning, offset
  planning, line-break decisions, stretch-space division — is a pure function over small `Copy`
  values, which both keeps them cheap and makes them unit-testable without a live `Ui`.

## How it differs from CSS flexbox

- **Frame-delayed measuring.** `Center`, `End`, `Space*` justification, and `StretchItems` all
  need the size of the content, which egui only knows *after* drawing it. The stack remembers
  the previous frame's size and requests one repaint when it changes; the first frame lays out
  as `Start` / unstretched.
- **Main-axis free space.** A row fills the available width. A column has no free space unless
  given `.height(..)`, so `Justify` other than `Start` (and `StretchItems`) need a height there.
- **Wrapping.** `Wrap::Wrap` uses egui's wrapping layout; justification is ignored while
  wrapping. egui only starts a new line for *leaf* widgets — a composite child (a `Frame` /
  `MudPaper`) would otherwise be squeezed into the current line's leftover width instead of
  moving down. `StackUi::item` works around this by measuring each item as an unbreakable unit
  and calling `end_row()` before one that no longer fits, based on the previous frame's extent.
- **`Reverse`** uses egui's right-to-left / bottom-up layouts. A reversed column without
  `.height(..)` needs one hidden (invisible) sizing frame before it has something to anchor to.
- **`StretchItems`** hands leftover main-axis space to specific positions (first/last/middle/
  all), computed a frame late, the same way `Space*` is. It only affects children added with `StackUi::item`.
- **`FillIndex`** (arbitrary per-item proportional flex-grow weights) is **not** supported.
  Unlike `StretchItems`'s fixed start/end/middle/all split, arbitrary per-item weights would
  need a real multi-pass constraint solver — egui's immediate-mode, single-pass layout has no
  room for that.

## Testing

All arithmetic lives in small, pure, `#[cfg(test)]`-covered functions at the bottom of the file
— no `egui::Context` is constructed in tests. Coverage includes:

- `plan_rect` / `plan_offsets` across row/column, reversed/forward, wrapped/unwrapped, and every
  `Justify` variant, including overflow (never produces a negative offset).
- Wrapping line-break decisions (`should_break_line`, `advance_line`) and a full
  five-items-onto-two-lines scenario.
- `Measure` equality under the epsilon tolerance.
- `StretchItems`: `stretch_targets` index selection for every variant (including the 0/1/2-item
  edge cases for `Middle`), and `plan_stretch` space division, overflow fallback, infinite/
  unknown main length, and per-variant leftover splits.

Run with:

```sh
cargo test
```