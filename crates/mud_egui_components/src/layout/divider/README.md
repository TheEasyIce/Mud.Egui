# MudDivider for egui

A horizontal or vertical rule for [egui](https://github.com/emilk/egui), ported from
[MudBlazor's `MudDivider`](https://mudblazor.com/components/divider). It draws with the
theme palette, supports MudBlazor's `Inset` / `Middle` margins, light and absolute
variants, vertical flex stretching, and (as an extension) a centered text label.

## Contents

1. [Design principle: API parity, not API invention](#design-principle-api-parity-not-api-invention)
2. [Architecture](#architecture)
3. [API reference](#api-reference)
4. [Visual behaviour](#visual-behaviour)
5. [Examples](#examples)
6. [Performance and memory choices](#performance-and-memory-choices)
7. [Testing](#testing)
8. [Differences from MudBlazor](#differences-from-mudblazor)

---

## Design principle: API parity, not API invention

The MudBlazor API for `MudDivider` was checked against the published API docs, the
component source (`MudDivider.razor.cs`) and the stylesheet (`_divider.scss`) before
this refactor. The real component has exactly five parameters plus the shared
`MudComponentBase` members:

| MudBlazor parameter | Type          | Default     |
|---------------------|---------------|-------------|
| `Absolute`          | `bool`        | `false`     |
| `DividerType`       | `DividerType` | `FullWidth` |
| `FlexItem`          | `bool`        | `false`     |
| `Light`             | `bool`        | `false`     |
| `Vertical`          | `bool`        | `false`     |
| `Class` (base)      | `string`      | `null`      |
| `Style` (base)      | `string`      | `null`      |
| `Tag` (base)        | `object`      | `null`      |
| `UserAttributes` (base) | `Dictionary<string, object>` | — |

A divider has no value, no click or focus behaviour, no elevation and no variants.
Therefore this port deliberately has **no** `Value` / `ValueChanged` /
`ValueExpression`, no `OnClick` / `OnBlur` / `OnFocus` / `OnKeyDown` / `OnChange`,
no `Variant`, `Size`, `Elevation`, `Disabled`, `ReadOnly`, and no
`AdornmentStart` / `AdornmentEnd`. Adding them would create API that MudBlazor does
not have and that would do nothing.

## Architecture

```
mud_divider.rs
├── constants            INSET_START, MIDDLE_SIDE, TEXT_GAP, LIGHT_FACTOR, ...
├── pure helpers         sanitize_*, resolve_line_color, inset_rect,
│                        absolute_endpoints, label_row_height
├── MudDivider           public builder struct (all fields public)
│   ├── builder methods  vertical, divider_type, flex_item, light, absolute,
│   │                    color, text, thickness, height, flavor, classes, tag
│   ├── pure helpers     margins_for, margins_for_dir, line_segment_width,
│   │                    vertical_extent
│   └── rendering        show -> render_content / render_absolute
│                        render_vertical / render_horizontal / draw_labelled
├── impl Widget          ui.add(MudDivider::new())
└── mod tests            pure-function tests + headless egui frame tests
```

The rule of the module: **every decision that can be made from numbers is a pure
function**. Rendering methods only fetch the theme, call those functions, allocate
space and paint. This keeps almost all behaviour testable without a GPU or window.

Render flow for `show(ui)`:

1. Fetch the `MudTheme` and palette from the egui context.
2. Choose the line color (`resolve_line_color`): default → palette divider color,
   semantic color → resolved palette color (fallback: divider color), then `light`
   fades it.
3. If `absolute`: paint on the parent's edge, allocate no space, return.
4. Otherwise resolve the utility-class `Frame` (Bootstrap or Tailwind flavor) and,
   inside it, apply the divider-type margins and draw the vertical, horizontal or
   labelled line.

## API reference

### Parameters that exist in MudBlazor

| Rust | MudBlazor | Type | Default | Description |
|------|-----------|------|---------|-------------|
| `.vertical(bool)` | `Vertical` | `bool` | `false` | Draw a vertical line. |
| `.divider_type(DividerType)` | `DividerType` | `DividerType` | `FullWidth` | `FullWidth`: no margin. `Inset`: 72 pt start margin. `Middle`: 16 pt left and right. |
| `.flex_item(bool)` | `FlexItem` | `bool` | `false` | Vertical dividers stretch to the available height. |
| `.light(bool)` | `Light` | `bool` | `false` | Lighter line color. |
| `.absolute(bool)` | `Absolute` | `bool` | `false` | Pin to the bottom (horizontal) or right (vertical) edge of the parent rect; takes no layout space. |
| `.classes([..])` | `Class` | iterator of `AsRef<str>` | empty | Utility classes, resolved by the selected flavor. |
| `.tag(T)` / `.tag_ref::<T>()` | `Tag` | `T: Any + Send + Sync` | `None` | Attach and read back user data. |

### Extensions kept from the original port

| Rust | Type | Default | Description |
|------|------|---------|-------------|
| `.color(Color)` | `Color` | `Default` | Semantic color role of the line. |
| `.text(impl Into<Arc<str>>)` | `Option<Arc<str>>` | `None` | Centered label (horizontal dividers only). Empty string clears it. |
| `.thickness(f32)` | `f32` | `1.0` | Line thickness in points, sanitized (min `0.5`, non-finite → `1.0`). |
| `.height(f32)` | `f32` | `24.0` | Height of a non-flex vertical divider (negative → `0`, non-finite → `24`). |
| `.flavor(StyleFlavor)` | `StyleFlavor` | default | Bootstrap or Tailwind utility resolver. |

### Not applicable in egui

| MudBlazor | Why it is not ported |
|-----------|----------------------|
| `Style` | There are no inline CSS strings in egui. Use `.classes(..)` and the typed fields. |
| `UserAttributes` | There is no HTML tag to splat attributes onto. |

### Methods

| Method | Description |
|--------|-------------|
| `MudDivider::new()` | Full-width horizontal divider with default settings. |
| `show(self, ui) -> Response` | Render and return the line's `Response`. |
| `ui.add(divider)` | Same as `show` through the `Widget` impl. |

## Visual behaviour

Values follow `_divider.scss` from MudBlazor:

| Case | Behaviour |
|------|-----------|
| Default | 1 pt line in the palette's divider color, spanning the available width. |
| `Inset` | 72 pt start margin (`margin-inline-start`): mirrored to the right in right-to-left layouts. |
| `Middle` | 16 pt left and right margin, no vertical margin. |
| `Light` | Line color multiplied by `0.5` (see below). |
| `Vertical` | Fixed `height` (default 24 pt) or, with `flex_item`, the available height. |
| `Absolute` | Drawn on the parent's bottom edge (horizontal) or right edge (vertical), inset by the divider-type margins. |
| Labelled | Two line segments with the label centered between them and an 8 pt gap; label uses the theme's `body2` font and the weak text color. If the label is wider than the row, the segments shrink to zero rather than going negative. |

A divider is a static, non-interactive element, so it has no hover, focus, active or
disabled styling. The `Response` uses `Sense::hover()`; labelled dividers also expose
the label to AccessKit through `WidgetInfo`.

## Examples

### Basic

```rust
MudDivider::new().show(ui);
ui.add(MudDivider::new().thickness(2.0));
```

### Between list items (Inset)

```rust
for (i, item) in items.iter().enumerate() {
    list_row(ui, item);
    if i + 1 < items.len() {
        MudDivider::new().divider_type(DividerType::Inset).show(ui);
    }
}
```

### Labelled divider

```rust
ui.add(MudDivider::new().text("OR").color(Color::Primary));
```

### Vertical divider in a toolbar

```rust
ui.horizontal(|ui| {
    ui.button("Cut");
    MudDivider::new().vertical(true).flex_item(true).show(ui);
    ui.button("Copy");
});
```

### Pinned to the bottom of a card

```rust
egui::Frame::group(ui.style()).show(ui, |ui| {
    ui.set_min_height(120.0);
    ui.label("Card body");
    MudDivider::new().absolute(true).light(true).show(ui);
});
```

### Utility classes and user data

```rust
let divider = MudDivider::new()
    .flavor(StyleFlavor::Tailwind)
    .classes(["my-4"])
    .tag("section-break");

assert_eq!(divider.tag_ref::<&str>(), Some(&"section-break"));
divider.show(ui);
```

## Performance and memory choices

Being honest about scale: a divider is one of the cheapest widgets in a UI, so the
choices below remove avoidable work rather than chase micro-benchmarks.

- **No per-frame clone of the label.** The original cloned the `String` label into a
  `move` closure every frame. Rendering now borrows the label, and the label is stored
  as `Arc<str>`, so copying a divider specification around is a pointer copy.
  (egui's `layout_no_wrap` takes an owned `String`, so one allocation per labelled
  frame remains; egui's galley cache absorbs the layout cost.)
- **No per-frame closure boxing.** The old horizontal path built a `move` closure with
  captured clones even for unlabelled dividers; the new path is a straight `match`.
- **Pure functions on `Copy` data.** All geometry and color decisions take and return
  `f32`, `Margin`, `Rect` and `Color32` by value: no heap, no locking, nothing to
  synchronize.
- **Thread safety by construction.** The only shared structure is the optional tag,
  held as `Arc<dyn Any + Send + Sync>`.
- **Absolute dividers skip the utility frame** entirely, because they take no layout
  space.
- **Sanitization at the edges.** NaN, infinity and negative values can never reach the
  painter, so a bad number cannot cause layout explosions or invisible lines.

Not done, on purpose: a cache for the resolved frame. The `ComponentBase` / style
resolver APIs were not changed in this refactor, and invalidation rules for them are
owned by those crates.

## Testing

`cargo test` runs the module's tests:

- **Pure functions:** sanitizers, color choice (including `light`), margins for every
  `DividerType` (LTR and RTL), segment widths, vertical extents, inset rectangles and
  absolute-line endpoints.
- **Builder:** MudBlazor defaults, every parameter setter, number sanitizing, empty-label
  clearing, and tag round trips by type.
- **Headless frames:** a real `egui::Context` runs one 400×300 frame to check that
  horizontal dividers span the width, vertical dividers honor `height` or fill the
  space with `flex_item`, labelled dividers are taller than the line, `Inset` and
  `Middle` leave their margins, absolute dividers take no layout space, and the
  `Widget` impl works.

There are no event-callback tests because a divider has no events.

## Differences from MudBlazor

- **`Light`** derives its color by fading the divider color instead of reading a
  `DividerLight` palette entry. MudBlazor's stock palette makes "light" look darker
  than the default (upstream issue #5669); this port always makes it lighter.
- **Non-flex vertical height** is a fixed `height` (24 pt default) because CSS's
  `height: 100%` has no direct egui equivalent without a parent size.
- **Margins changed from the previous Rust port** to match the stylesheet:
  `Inset` is 72 pt (was 16) and `Middle` has no vertical margin (was 8).
- **`child_content`** changed from `Option<String>` to `Option<Arc<str>>`.
- **Absolute dividers** ignore the label and utility classes.