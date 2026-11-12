# MudToolBar (egui port)

A horizontal toolbar panel for [egui](https://github.com/emilk/egui), ported from MudBlazor's
[`MudToolBar`](https://mudblazor.com/api/mudtoolbar). It gives its children a Material-style
row: 64 pt tall (48 pt dense), 16 pt gutters, vertically centered, optionally wrapping. Layout
of the children is delegated to [`MudStack`](./README.md).

## Contents

1. [Design principle: API parity, not API invention](#design-principle-api-parity-not-api-invention)
2. [Architecture](#architecture)
3. [API reference](#api-reference)
4. [Rendering behaviour](#rendering-behaviour)
5. [Examples](#examples)
6. [Performance and memory choices](#performance-and-memory-choices)
7. [Testing](#testing)
8. [Differences from MudBlazor](#differences-from-mudblazor)

---

## Design principle: API parity, not API invention

The current MudBlazor `MudToolBar` API (checked against the published API docs) is:

| MudBlazor | Type | Default | Description |
|-----------|------|---------|-------------|
| `ChildContent` | `RenderFragment` | — | The toolbar content, typically icon buttons. |
| `Dense` | `bool` | `false` | Compact vertical padding. |
| `Gutters` | `bool` | `true` | Left and right padding. (Older versions called it `DisableGutters`, with the opposite meaning.) |
| `WrapContent` | `bool` | `false` | Lets the content wrap. |
| `Class`, `Style`, `Tag`, `UserAttributes`, `FieldId` | base | — | Inherited from `MudComponentBase`. |

A toolbar has no value, no click target, no variants and no elevation (elevation is an
`MudAppBar` parameter). This port therefore has **no** `Value` / `ValueChanged`, no
`OnClick` / `OnFocus` / `OnKeyDown`, no `Variant`, `Color`, `Size`, `Elevation`,
`Disabled`, `ReadOnly` and no `AdornmentStart` / `AdornmentEnd`.

## Architecture

```
mud_tool_bar.rs
├── pure helpers       toolbar_height, resolve_inner_margin, content_min_height,
│                      used_width, used_height, vertical_extent_of
├── MudToolBar         builder: dense, gutters, wrap_content, justify_space_between,
│   │                  flavor, classes, tag
│   ├── show           plain closure       ──┐
│   ├── show_items     measured StackUi items ├──► render
│   ├── build_stack    MudStack::horizontal() config
│   └── render         frame → min size → MudStack → allocate in parent
├── impl Widget        ui.add(MudToolBar::new())  (empty toolbar)
└── mod tests
```

Render flow:

1. Fetch the theme and resolve the utility-class `Frame` (Bootstrap or Tailwind).
2. `resolve_inner_margin`: add the 16 pt gutters unless the classes already resolved
   to an inner margin.
3. Create a child `Ui` over the available rectangle (so the toolbar can be sized
   after drawing).
4. Inside the frame: force full width and `content_min_height`, then call `draw`, which is
   either `MudStack::show` or `MudStack::show_items`.
5. Allocate `used_width × used_height` in the parent and return that `Response`.

### How it uses `MudStack`

The toolbar builds exactly this stack and nothing else:

```rust
MudStack::horizontal()
    .justify(if justify_space_between { Justify::SpaceBetween } else { Justify::Start })
    .wrap(if wrap_content { Wrap::Wrap } else { Wrap::NoWrap })
    .align_items(AlignItems::Center)
    .spacing(2)            // 2 × 4 px = 8 px gap
```

Only the documented `MudStack` API is used (`horizontal`, `justify`, `wrap`,
`align_items`, `spacing`, `show`, `show_items`, `StackUi::item`). Consequences taken from the
stack README:

- `Justify::Space*` needs item boundaries, so `justify_space_between` only works through
  `MudToolBar::show_items` with `stack.item(..)`.
- `Justify::Space*` is ignored while wrapping.
- Space distribution is computed from the previous frame, so the first frame lays out as
  `Start`.

## API reference

### Members that exist in MudBlazor

| Rust | MudBlazor | Type | Default | Description |
|------|-----------|------|---------|-------------|
| `.dense(bool)` | `Dense` | `bool` | `false` | 48 pt instead of 64 pt. |
| `.gutters(bool)` | `Gutters` | `bool` | `true` | 16 pt left and right padding, skipped if the utility classes already set an inner margin. |
| `.wrap_content(bool)` | `WrapContent` | `bool` | `false` | Children wrap onto more rows (the toolbar grows). |
| closure of `show` / `show_items` | `ChildContent` | closure | — | The toolbar content. |
| `.classes([..])` | `Class` | iterator of `AsRef<str>` | none | Utility classes. |
| `.tag(T)` / `.tag_ref::<T>()` | `Tag` | `T: Any + Send + Sync` | `None` | User data. |

### Extension kept from the original port

| Rust | Type | Default | Description |
|------|------|---------|-------------|
| `.justify_space_between(bool)` | `bool` | `false` | Distribute free space between children. Requires `show_items`. |
| `.flavor(StyleFlavor)` | `StyleFlavor` | default | Bootstrap or Tailwind utility resolver. |

### Not applicable in egui

| MudBlazor | Why |
|-----------|-----|
| `Style` | No inline CSS strings in egui; use `.classes(..)`. |
| `UserAttributes`, `FieldId` | No HTML tag to splat attributes onto or label for. |

### Rendering methods

| Method | Signature | Use when |
|--------|-----------|----------|
| `show` | `fn show<R>(self, ui, impl FnOnce(&mut Ui) -> R) -> InnerResponse<R>` | The common case: draw widgets left to right. |
| `show_items` | `fn show_items<R>(self, ui, impl FnOnce(&mut StackUi<'_>) -> R) -> InnerResponse<R>` | You need `justify_space_between`. |
| `ui.add(toolbar)` | `Widget` impl | An empty toolbar, e.g. only for its background. |

The returned `InnerResponse::response` is the response of the **whole toolbar rectangle**
(full width, full height), not of the inner stack.

## Rendering behaviour

| Aspect | Behaviour |
|--------|-----------|
| Height | `max(content, 64)` pt, or `max(content, 48)` when dense. Wrapped content grows the toolbar. |
| Width | The available width (or the content width when the available width is unbounded). |
| Gutters | 16 pt left and right; none when `gutters(false)`; skipped when classes provide padding. |
| Vertical alignment | `AlignItems::Center` on the stack. |
| Gap | 8 pt between children. |
| Background / borders | Come from utility classes via the frame and cover the whole toolbar (see below). |

A toolbar is a static container, so it has no hover, focus, active or disabled styling of
its own. Interactive state belongs to the buttons placed inside it.

## Examples

### Basic toolbar

```rust
MudToolBar::new().show(ui, |ui| {
    ui.heading("Inbox");
    ui.button("Compose");
});
```

### Dense toolbar with a background utility class

```rust
MudToolBar::new()
    .dense(true)
    .classes(["bg-primary"])
    .show(ui, |ui| {
        ui.label("Selected: 3");
    });
```

### Title left, actions right (space-between)

```rust
MudToolBar::new()
    .justify_space_between(true)
    .show_items(ui, |stack| {
        stack.item(|ui| { ui.heading("Inbox"); });
        stack.item(|ui| {
            ui.horizontal(|ui| {
                ui.button("Search");
                ui.button("Compose");
            });
        });
    });
```

### Title left, actions right (spacer)

```rust
MudToolBar::new().show(ui, |ui| {
    ui.heading("Inbox");
    ui.add(MudSpacer::new().reserve(100.0));
    ui.button("Compose");
});
```

### Wrapping toolbar of chips

```rust
MudToolBar::new().wrap_content(true).show(ui, |ui| {
    for tag in &tags {
        ui.label(tag);
    }
});
```

### No gutters, user data

```rust
let bar = MudToolBar::new().gutters(false).tag("editor-toolbar");
assert_eq!(bar.tag_ref::<&str>(), Some(&"editor-toolbar"));
bar.show(ui, |ui| { ui.button("Bold"); });
```

## Performance and memory choices

- **One pass, no intermediate collections.** The toolbar adds no per-frame `Vec`s or
  strings; all numeric decisions are pure `f32` / `Margin` functions.
- **Frame resolved once per call, not per child.** Utility classes resolve into a single
  `Frame`; the stack handles child layout.
- **No dead state.** The old version wrote `hovered` / `active` into `TwState` after the
  toolbar had been consumed (no effect); that code is removed.
- **No duplicate response work.** The toolbar allocates its rectangle once and returns that
  response; the old code discarded it.
- **Sanitized geometry.** Non-finite widths or strokes never reach `allocate_rect`.

Not done: caching the resolved frame. The style resolvers live in other crates and own
their invalidation rules; `MudStack` remains responsible for its own measurement memory.

## Testing

`cargo test` covers:

- **Pure helpers:** heights, gutter resolution (with and without explicit padding),
  vertical extents, content minimum height (including negative and `NaN` strokes), used
  width / height.
- **Builder:** defaults, every setter, tag round trips.
- **Headless frames:** standard toolbar reserves 64 pt and the full width; dense reserves
  48 pt; `show` and `show_items` return the closure value; gutters move the first child
  inwards; a heavily wrapped toolbar is never shorter than standard; the `Widget` impl
  renders an empty toolbar.

There are no event-callback tests because a toolbar has no events.

## Differences from MudBlazor

- **Bug fixed:** `justify_space_between` previously did nothing, because it was only ever
  combined with the plain `show`; it now works through `show_items`.
- **Bug fixed:** utility-class backgrounds previously covered only the content height while
  the toolbar reserved 64 / 48 pt. The frame content now spans the full width and height.
- **Returned response** is now the toolbar's full rectangle rather than the inner stack's.
- **Vertical centering** inside the minimum height relies on `MudStack`'s
  `AlignItems::Center`; the toolbar guarantees the frame is that tall, but the placement of
  children inside it is the stack's responsibility.
- `Style`, `UserAttributes` and `FieldId` are not ported (no CSS or HTML in egui).
- `justify_space_between` has no MudBlazor counterpart.