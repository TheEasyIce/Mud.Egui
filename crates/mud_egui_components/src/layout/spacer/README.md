# MudSpacer for egui

A zero-content filler for [egui](https://github.com/emilk/egui), ported from MudBlazor's
`MudSpacer`. It expands into the free space of the current layout so that widgets on
either side are pushed apart, like a `flex-grow: 1` element in a CSS flex container.

## Contents

1. [Design principle: API parity, not API invention](#design-principle-api-parity-not-api-invention)
2. [Architecture](#architecture)
3. [API reference](#api-reference)
4. [Layout behaviour](#layout-behaviour)
5. [Examples](#examples)
6. [Performance and memory choices](#performance-and-memory-choices)
7. [Testing](#testing)
8. [Differences from MudBlazor](#differences-from-mudblazor)

---

## Design principle: API parity, not API invention

`MudSpacer` is a special case in MudBlazor: it has **no documented parameters of its
own**. An upstream issue (MudBlazor #5577) notes that the component and its API are not
documented at all and that IDE IntelliSense shows nothing beyond the inherited members.
The only members it has are therefore those of `MudComponentBase`: `Class`, `Style`,
`Tag` and `UserAttributes`. I could not retrieve its source file, so this statement
rests on that issue and on how MudBlazor examples use it (a filler between siblings).

Because of that, this port deliberately has **no** `Value` / `ValueChanged`, no
`OnClick` / `OnFocus` / `OnKeyDown`, no `Variant`, `Color`, `Size`, `Elevation`,
`Disabled`, `ReadOnly`, no `ChildContent` and no `AdornmentStart` / `AdornmentEnd`. A
spacer has no content, no value and nothing to interact with.

## Architecture

```
mud_spacer.rs
├── pure helpers       sane_length, sane_fraction
├── MudSpacer          public builder struct
│   ├── builder        reserve, fraction, min_size, share, flavor, classes, tag
│   ├── sizing math    resolve_extent, equal_share_fraction (pure, pub)
│   │                  plan_size (pure, private)
│   └── rendering      show
├── impl Widget        ui.add(MudSpacer::new())
└── mod tests          pure tests + headless egui frame tests
```

Render flow for `show(ui)`:

1. Fetch the theme and resolve the utility-class `Frame` (Bootstrap or Tailwind).
2. Inside the frame, read the layout's main direction.
3. `plan_size` calls `resolve_extent` for the main axis; the cross axis is `0`.
4. `allocate_response(size, Sense::hover())`.

## API reference

### Members that exist in MudBlazor

| Rust | MudBlazor | Description |
|------|-----------|-------------|
| `.classes([..])` | `Class` | Utility classes, resolved by the selected flavor. |
| `.tag(T)` / `.tag_ref::<T>()` | `Tag` | Attach and read back user data (`T: Any + Send + Sync`). |

### Extensions (needed because egui has no flex solver)

| Rust | Type | Default | Description |
|------|------|---------|-------------|
| `.reserve(f32)` | `f32` | `0.0` | Points to leave free after the spacer for following widgets. |
| `.fraction(f32)` | `f32` | `1.0` | Share of the remaining space to take, clamped to `0..=1` (`NaN` → `1`). |
| `.min_size(f32)` | `f32` | `0.0` | Minimum extent along the main axis. |
| `.share(index, count)` | — | — | The `index`-th of `count` spacers splitting the free space equally (sets `fraction`). **New.** |
| `.flavor(StyleFlavor)` | `StyleFlavor` | default | Bootstrap or Tailwind utility resolver. |

### Associated functions

| Function | Description |
|----------|-------------|
| `MudSpacer::resolve_extent(available, item_spacing, reserve, fraction, min_size)` | The sizing formula, public so you can reuse it. |
| `MudSpacer::equal_share_fraction(index, count)` | Fraction for the `index`-th of `count` equal spacers: `1 / (count - index)`. |

### Not applicable in egui

| MudBlazor | Why |
|-----------|-----|
| `Style` | No inline CSS strings in egui; use `.classes(..)`. |
| `UserAttributes` | No HTML tag to splat attributes onto. |

## Layout behaviour

```
extent = max(min_size, (available - item_spacing - reserve).max(0) * fraction)
```

- In a **horizontal** layout the spacer's size is `(extent, 0)`; in a **vertical** layout
  it is `(0, extent)`.
- When the available length is **not finite** (the scroll direction of a `ScrollArea`),
  there is no free space to share and the spacer returns `min_size`.
- `item_spacing` is subtracted once because egui adds one gap after every widget.
- Several spacers in one row: egui lays out one widget at a time, so a spacer cannot
  know how many siblings follow. The first one would take everything. Use
  `.share(index, count)` to split the space equally; the split is exact when item
  spacing and `reserve` are zero and approximate otherwise.
- The spacer is non-interactive (`Sense::hover()`), so it has no hover, focus or
  disabled styling.

## Examples

### Toolbar: title left, actions right

```rust
ui.horizontal(|ui| {
    ui.heading("Inbox");
    MudSpacer::new().show(ui);
    ui.button("Compose");
});
```

### Keep room for trailing widgets

```rust
ui.horizontal(|ui| {
    ui.label("Status");
    ui.add(MudSpacer::new().reserve(80.0).min_size(8.0));
    ui.label("Right side, about 80 pt wide");
});
```

### Three groups with equal gaps

```rust
ui.horizontal(|ui| {
    ui.label("A");
    ui.add(MudSpacer::new().share(0, 2));
    ui.label("B");
    ui.add(MudSpacer::new().share(1, 2));
    ui.label("C");
});
```

### Push a footer to the bottom of a column

```rust
ui.vertical(|ui| {
    ui.label("Content");
    ui.add(MudSpacer::new().reserve(32.0));
    ui.button("Footer");
});
```

### Utility classes and user data

```rust
ui.add(
    MudSpacer::new()
        .flavor(StyleFlavor::Tailwind)
        .classes(["px-2"])
        .tag("toolbar-gap"),
);
```

## Performance and memory choices

A spacer is about as cheap as a widget gets; the work is removing avoidable cost, not
inventing complexity.

- **Pure sizing on `f32`.** `resolve_extent` and `equal_share_fraction` are `#[inline]`,
  allocation-free and lock-free. No state is shared between frames or threads.
- **Stack-only builder.** Apart from the optional tag (`Arc<dyn Any + Send + Sync>`,
  allocated only when you call `.tag`), the struct holds only scalars and the style
  resolver state.
- **Single-axis plan.** `plan_size` computes only the main-axis extent; the cross axis
  is a constant `0`.
- **Dead state removed.** The previous version wrote `hovered` / `active` into a
  `TwState` after the spacer had already been consumed, which had no effect; that code
  is gone.
- **Sanitized inputs.** Negative, `NaN` and infinite values can never produce a negative
  or non-finite size.

Not done: caching the resolved utility `Frame`. The style resolvers live in other crates
and own their invalidation rules.

## Testing

`cargo test` covers:

- **Sanitizers** and every branch of `resolve_extent`: default fill, reserve, fraction,
  `min_size` floor, unbounded space, negative availability, bad inputs.
- **Equal shares:** the fraction sequence, edge cases, and a simulated split of 300 pt
  across three spacers.
- **Builder:** defaults, input sanitizing, `share`, tag round trips, `plan_size` axis
  selection.
- **Headless frames:** horizontal and vertical spacers fill their layout, `reserve`
  shrinks the extent by exactly that amount, `fraction(0.5)` takes about half, and the
  `Widget` impl works.

There are no event-callback tests because a spacer has no events.

## Differences from MudBlazor

- `reserve`, `fraction`, `min_size` and `share` have no MudBlazor counterpart; they stand
  in for the free-space resolution that a CSS flex container does automatically.
- `Style` and `UserAttributes` are not ported (no CSS or HTML in egui).
- Several MudBlazor spacers split space by `flex-grow`; here you opt in with `share`.