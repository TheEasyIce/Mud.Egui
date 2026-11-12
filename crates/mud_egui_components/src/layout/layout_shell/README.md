# MudLayout + MudAppBar + MudDrawer for `mud_egui`

The application shell for [egui](https://github.com/emilk/egui), ported from
MudBlazor's [`MudLayout`](https://mudblazor.com/components/layout),
[`MudAppBar`](https://mudblazor.com/components/appbar) and
[`MudDrawer`](https://mudblazor.com/components/drawer) (MudBlazor 8.13.0).

```rust
struct App {
    layout: LayoutState,
    drawer: DrawerState,
}

MudLayout::new().show(ui, &mut self.layout, |ui, layout| {
    MudAppBar::new().color(Color::Primary).show(ui, layout, |ui| {
        ui.add(MudText::new("My App").typo(Typo::H6));
    });
    ui.horizontal(|ui| {
        MudDrawer::new()
            .variant(DrawerVariant::Responsive)
            .show(ui, &mut self.drawer, layout, |ui| {
                ui.add(MudText::new("Navigation"));
            });
        MudMainContent::new().show(ui, layout, |ui| {
            ui.add(MudText::new("Page body"));
        });
    });
});
```

The shell paints only its own surfaces; every string inside is yours to render
through **`MudText`**, every glyph through **`MudIcon`**.

---

## Contents

1. [Design principle: API parity, not API invention](#design-principle-api-parity-not-api-invention)
2. [Architecture](#architecture)
3. [API reference — MudLayout](#api-reference--mudlayout)
4. [API reference — MudAppBar](#api-reference--mudappbar)
5. [API reference — MudDrawer](#api-reference--muddrawer)
6. [Examples](#examples)
7. [Layout and interaction](#layout-and-interaction)
8. [Performance and memory choices](#performance-and-memory-choices)
9. [Testing](#testing)
10. [Differences from MudBlazor](#differences-from-mudblazor)

---

## Design principle: API parity, not API invention

The MudBlazor 8.13.0 parameter surfaces were verified against DNDocs before
implementation. `MudLayout` has **no own parameters** beyond `ChildContent`;
`MudAppBar` has **ten**; `MudDrawer` has **fifteen**.

### `MudLayout`

| MudBlazor | Rust mapping | Default |
|---|---|---|
| `ChildContent` | `body` argument of `MudLayout::show` | required |
| `Class` (base) | `classes(iter)` — stored | `None` |
| `Style` / `Tag` / `UserAttributes` | **not ported** | — |

### `MudAppBar`

| MudBlazor | Rust mapping | Default |
|---|---|---|
| `Bottom` | `bottom(bool)` | `false` |
| `ChildContent` | `content` argument of `show` | required |
| `Color` | `color(Color)` | `Default` |
| `Dense` | `dense(bool)` | `false` |
| `Elevation` | `elevation(u8)` | `4` |
| `Fixed` | `fixed(bool)` | `true` — **stored** |
| `Gutters` | `gutters(bool)` | `true` |
| `ToolBarClass` | `toolbar_class(s)` — **stored** | `None` |
| `WrapContent` | `wrap_content(bool)` | `false` |
| `Contextual` | **not in 8.13.0** — 8.14+ addition | — |
| `Class` (base) | `classes(iter)` — stored | `None` |
| `Style` / `Tag` / `UserAttributes` | **not ported** | — |

### `MudDrawer`

| MudBlazor | Rust mapping | Default |
|---|---|---|
| `Anchor` | `anchor(AnchorOrigin)` | `Start` |
| `Breakpoint` | `breakpoint(Breakpoint)` | `Md` |
| `ChildContent` | `content` argument of `show` | required |
| `ClipMode` | `clip_mode(DrawerClipMode)` | `Never` |
| `Color` | `color(Color)` | `Default` |
| `Elevation` | `elevation(u8)` | `1` |
| `Fixed` | `fixed(bool)` | `true` — **stored** |
| `Height` | `height(f32)` | `None` → `200.0` |
| `MiniWidth` | `mini_width(f32)` | `None` → `56.0` |
| `Open` | `DrawerState::open` | `false` |
| `OpenChanged` | `DrawerResponse::opened` / `closed` | — |
| `OpenMiniOnHover` | `open_mini_on_hover(bool)` | `false` |
| `Overlay` | `overlay(bool)` | `true` |
| `OverlayAutoClose` | `overlay_auto_close(bool)` | `true` |
| `Variant` | `variant(DrawerVariant)` | `Responsive` |
| `Width` | `width(f32)` | `None` → `240.0` |
| `Class` (base) | `classes(iter)` — stored | `None` |
| `Style` / `Tag` / `UserAttributes` | **not ported** | — |

### `MudDrawerHeader`

| MudBlazor | Rust mapping | Default |
|---|---|---|
| `ChildContent` | `content` argument of `show` | required |
| `Dense` | `dense(bool)` | `false` |
| `Class` (base) | `classes(iter)` — stored | `None` |

### `MudMainContent`

| MudBlazor | Rust mapping | Default |
|---|---|---|
| `ChildContent` | `content` argument of `show` | required |
| `Class` (base) | `classes(iter)` — stored | `None` |

### Enums introduced

Two public enums, each in its own file (Rule 2):

| Enum | File | Discriminants |
|---|---|---|
| `DrawerVariant` | `mud_egui_core::enums::drawer_variant` | `Temporary = 0`, `Responsive = 1`, `Persistent = 2`, `Mini = 3` |
| `DrawerClipMode` | `mud_egui_core::enums::drawer_clip_mode` | `Never = 0`, `Docked = 1`, `Always = 2` |

`AnchorOrigin` (already in `mud_egui_core::enums`) is MudBlazor's `Anchor` —
the popover README documents the rename. `Breakpoint` and `Color` are reused.

### Deliberately not ported

| MudBlazor | Reason |
|---|---|
| `Style` / `Tag` / `UserAttributes` (all four components) | No CSS strings, no HTML tag. |
| `ToolBarClass` resolution | Stored for parity. The bar owns its own fill, height and gutters. |
| `Contextual` on `MudAppBar` | 8.14+ addition (PR #10210). Not in 8.13.0. |
| `Fixed` behaviour | Stored. egui has no free-floating chrome inside a `ScrollArea`; scroll-away behaviour is the caller's `ScrollArea` decision. |

---

## Architecture

```
layout/
├── app_bar/
│   ├── app_bar_response.rs   AppBarResponse
│   ├── mud_app_bar.rs        MudAppBar + pure helpers + tests
│   └── mod.rs
├── drawer/
│   ├── drawer_response.rs    DrawerResponse
│   ├── drawer_state.rs       DrawerState (caller-owned)
│   ├── effective_variant.rs  pub(crate) EffectiveVariant + resolve (Rule 2)
│   ├── mud_drawer.rs         MudDrawer + Side + resolve_side + side_rect + tests
│   ├── mud_drawer_header.rs  MudDrawerHeader
│   └── mod.rs
├── layout_shell/
│   ├── layout_state.rs       LayoutState
│   ├── mud_layout.rs         MudLayout
│   ├── mud_main_content.rs   MudMainContent + compute_main_rect
│   └── mod.rs
└── mod.rs

mud_egui_core/src/enums/
├── drawer_clip_mode.rs       DrawerClipMode
└── drawer_variant.rs         DrawerVariant
```

**Pure layer.** `compute_main_rect`, `resolve_side`, `side_rect`,
`resolve` (effective variant), `is_light`, `LayoutState::top_inset` /
`bottom_inset`, and every `_state` mutator are free functions or methods over
`Rect`, `f32`, enums. None touch `egui::Context`.

**State layer.** `LayoutState` is reset by `MudLayout::show`; `MudAppBar` and
`MudDrawer` publish into it; `MudMainContent` reads it. `DrawerState` is the
caller-owned `Open` binding.

**Component layer.** `MudLayout` hands the caller a full-screen `Ui`; the
caller composes `MudAppBar`, `MudDrawer` and `MudMainContent` inside it.

---

## API reference — `MudLayout`

| Method | Type | Description |
|---|---|---|
| `new()` | — | Default layout. |
| `classes(iter)` | `iter of AsRef<str>` | Stored. |
| `show(ui, state, \|ui, layout\| ..)` | `(&mut Ui, &mut LayoutState, FnOnce(&mut Ui, &mut LayoutState) -> R) -> R` | Resets `state` and hands the caller a screen-sized `Ui`. |

---

## API reference — `MudAppBar`

| Method | Type | Default | Description |
|---|---|---|---|
| `new()` | — | — | Default app bar. |
| `bottom(bool)` | `bool` | `false` | Anchor to the bottom edge. |
| `color(Color)` | `Color` | `Default` | Surface fill. |
| `dense(bool)` | `bool` | `false` | 48 pt instead of 64. |
| `elevation(u8)` | `u8` | `4` | Shadow depth, clamped to 24. |
| `fixed(bool)` | `bool` | `true` | Stored. |
| `gutters(bool)` | `bool` | `true` | 16 pt side padding. |
| `toolbar_class(s)` | `impl Into<String>` | `None` | Stored. |
| `wrap_content(bool)` | `bool` | `false` | Wrap onto more rows. |
| `classes(iter)` | — | — | Stored. |
| `show(ui, layout, \|ui\| ..)` | `(&mut Ui, &mut LayoutState, FnOnce(&mut Ui) -> R)` | — | Returns `InnerResponse<AppBarResponse>`. |

### `AppBarResponse`

| Field | Type | Description |
|---|---|---|
| `response` | `Response` | The bar's own response. |
| `height` | `f32` | 64 normally, 48 when `dense`. |

### Constants

| Name | Value |
|---|---|
| `APP_BAR_HEIGHT` | `64.0` |
| `DENSE_APP_BAR_HEIGHT` | `48.0` |
| `APP_BAR_GUTTER` | `16.0` |
| `APP_BAR_GAP` | `8.0` |

---

## API reference — `MudDrawer`

| Method | Type | Default | Description |
|---|---|---|---|
| `new()` | — | — | Default drawer. |
| `anchor(AnchorOrigin)` | `AnchorOrigin` | `Start` | Edge. |
| `breakpoint(Breakpoint)` | `Breakpoint` | `Md` | Responsive threshold. |
| `clip_mode(DrawerClipMode)` | `DrawerClipMode` | `Never` | Z-order vs app bar. |
| `color(Color)` | `Color` | `Default` | Surface fill. |
| `elevation(u8)` | `u8` | `1` | Shadow depth. |
| `fixed(bool)` | `bool` | `true` | Stored. |
| `height(f32)` | `f32` | `None` → `200` | Top/bottom extent. |
| `mini_width(f32)` | `f32` | `None` → `56` | Collapsed mini extent. |
| `open_mini_on_hover(bool)` | `bool` | `false` | Expand a mini drawer on hover. |
| `overlay(bool)` | `bool` | `true` | Backdrop for overlay variants. |
| `overlay_auto_close(bool)` | `bool` | `true` | Backdrop click closes. |
| `variant(DrawerVariant)` | `DrawerVariant` | `Responsive` | Behaviour. |
| `width(f32)` | `f32` | `None` → `240` | Left/right extent. |
| `right_to_left(bool)` | `bool` | `false` | Resolve `Start` / `End`. |
| `classes(iter)` | — | — | Stored. |
| `show(ui, state, layout, \|ui\| ..)` | `(&mut Ui, &mut DrawerState, &mut LayoutState, FnOnce(&mut Ui) -> R)` | — | Returns `DrawerResponse`. |

### `DrawerState`

| Field / method | Description |
|---|---|
| `open: bool` | Whether the drawer is open. |
| `hover_expanded: bool` | Mini expansion latch. |
| `new()` / `opened()` | Constructors. |
| `is_open()` / `open()` / `close()` / `toggle()` | Mutators. |

### `DrawerResponse`

| Field | Type | Description |
|---|---|---|
| `response` | `Response` | The panel's response. |
| `opened` | `bool` | `true` on the opening frame. |
| `closed` | `bool` | `true` on the closing frame. |
| `visible` | `bool` | Whether anything painted. |
| `reserved_extent` | `f32` | Width / height reserved. `0` for overlay. |

### Effective variant

| Config | Below breakpoint | At / above breakpoint |
|---|---|---|
| `Temporary` | Overlay | Overlay |
| `Persistent` | Push | Push |
| `Responsive` | Overlay | Push |
| `Mini` | Overlay | Mini |

### Constants

| Name | Value |
|---|---|
| `DEFAULT_WIDTH` | `240.0` |
| `DEFAULT_MINI_WIDTH` | `56.0` |
| `DEFAULT_HEIGHT` | `200.0` |
| `DEFAULT_MINI_HEIGHT` | `40.0` |

---

## Examples

### Minimal shell

```rust
let mut layout = LayoutState::default();
let mut drawer = DrawerState::new();

MudLayout::new().show(ui, &mut layout, |ui, layout| {
    MudAppBar::new().show(ui, layout, |ui| {
        ui.add(MudText::new("My App"));
    });
    ui.horizontal(|ui| {
        MudDrawer::new().show(ui, &mut drawer, layout, |ui| {
            ui.add(MudText::new("Nav"));
        });
        MudMainContent::new().show(ui, layout, |ui| {
            ui.add(MudText::new("Body"));
        });
    });
});
```

### Persistent drawer with a toggle button

```rust
MudAppBar::new().show(ui, layout, |ui| {
    if ui.button("☰").clicked() { drawer.toggle(); }
    ui.add(MudText::new("App"));
});

ui.horizontal(|ui| {
    MudDrawer::new()
        .variant(DrawerVariant::Persistent)
        .width(280.0)
        .show(ui, &mut drawer, layout, |ui| {
            MudDrawerHeader::new().show(ui, |ui| {
                ui.add(MudText::new("Navigation").typo(Typo::H6));
            });
            ui.add(MudText::new("Menu"));
        });
    MudMainContent::new().show(ui, layout, |ui| { /* body */ });
});
```

### Temporary drawer with an overlay

```rust
MudDrawer::new()
    .variant(DrawerVariant::Temporary)
    .overlay(true)
    .overlay_auto_close(true)
    .show(ui, &mut drawer, layout, |ui| { /* nav */ });
```

### Mini drawer that expands on hover

```rust
MudDrawer::new()
    .variant(DrawerVariant::Mini)
    .mini_width(56.0)
    .width(240.0)
    .open_mini_on_hover(true)
    .show(ui, &mut drawer, layout, |ui| { /* nav */ });
```

### Bottom app bar

```rust
MudAppBar::new().bottom(true).show(ui, layout, |ui| {
    ui.add(MudText::new("Status bar"));
});
```

### Drawer above the app bar (`ClipMode::Never`)

```rust
MudLayout::new().show(ui, &mut layout, |ui, layout| {
    ui.horizontal(|ui| {
        drawer_show.show(ui, &mut drawer, layout, |ui| { /* nav */ });
        ui.vertical(|ui| {
            MudAppBar::new().show(ui, layout, |ui| {
                ui.add(MudText::new("Title"));
            });
            MudMainContent::new().show(ui, layout, |ui| { /* body */ });
        });
    });
});
```

### Docked drawer (`ClipMode::Docked`)

```rust
MudDrawer::new()
    .clip_mode(DrawerClipMode::Docked)
    .show(ui, &mut drawer, layout, |ui| { /* nav */ });
```

The app bar is declared first; the drawer starts below it.

---

## Layout and interaction

### Structure

```
DrawerClipMode::Docked / Always     DrawerClipMode::Never
┌──────────────────────────────┐   ┌───────┬──────────────────────┐
│ AppBar                        │   │       │ AppBar                │
├───────┬──────────────────────┤   │ Drawer├──────────────────────┤
│Drawer │ MainContent           │   │       │ MainContent           │
│       │                       │   │       │                       │
└───────┴──────────────────────┘   └───────┴──────────────────────┘
```

The nesting is the caller's choice; the diagram above is the canonical shape.

### Sizing

| Component | Extent |
|---|---|
| `MudAppBar` | `available_width × 64` (48 dense) |
| `MudDrawer` (left/right) | `width` (default 240) × `available_height` |
| `MudDrawer` (top/bottom) | `available_width` × `height` (default 200) |
| `MudDrawer` mini (collapsed) | `mini_width` (default 56) × `available_height` |
| `MudMainContent` | `available_rect_before_wrap` minus the insets in `LayoutState` |

### Interaction

| Input | Effect |
|---|---|
| Backdrop click (overlay variant, `overlay_auto_close`) | `state.close()` |
| Pointer over a mini drawer (`open_mini_on_hover`) | `state.hover_expanded = true` for the frame |
| Toggle button in the app bar | Caller calls `state.toggle()` |

`MudAppBar` and `MudDrawer` are static containers; interactive state belongs to
the buttons placed inside them.

### `LayoutState` and declaration order

`MudMainContent` reads `LayoutState` when it is declared. Therefore:

- **`Docked` / `Always`**: declare the app bar first, then the drawer, then the
  main content.
- **`Never`**: declare the drawer first, then the app bar, then the main
  content (the drawer sits to the left of the vertical).

### Overlay z-order

Overlay drawers paint at `Order::Foreground` and their backdrop at
`Order::Middle`. Inline drawers paint at the normal z-order of the parent `Ui`.
`ClipMode::Never` in an inline (push) drawer needs no special handling because
the caller nests the drawer beside the vertical that holds the app bar.

---

## Performance and memory choices

| Choice | Effect |
|---|---|
| `LayoutState` is five `f32` + two `bool` | `Copy`, reset by a single assignment. |
| `DrawerState` is two `bool` | `Copy`, no allocation. |
| `AppBarResponse` is one `Response` + one `f32` | `Copy`-adjacent; no allocation. |
| Shadows are three stacked `rect_filled` calls | No `Frame`, no shader. Visually close to a blurred shadow. |
| `MudMainContent` reads `LayoutState` once | One `Rect` computation per frame. |
| Overlay drawers are `Area`-backed | One `Area::show` per open overlay frame; no per-frame `Frame`. |
| Hover latch on mini drawers is one `bool` | No pointer-hit-test beyond `Response::hovered`. |
| Pure helpers (`side_rect`, `compute_main_rect`, `resolve`) | Arithmetic on `Rect` / `f32` / enums; testable without a `Context`. |

---

## Testing

`cargo test -p mud_egui_components layout` runs the module's tests:

| Area | Tests |
|---|---|
| `DrawerVariant` | default, discriminants, unique names, predicates, `Display` |
| `DrawerClipMode` | default, discriminants, `starts_above_app_bar`, unique names, `Display` |
| `EffectiveVariant::resolve` | temporary always overlay, persistent always push, responsive follows the breakpoint, mini follows the breakpoint, sentinels override width |
| `LayoutState` | default all-zero, `reset`, `top_inset` with top / bottom app bar, `bottom_inset` |
| `compute_main_rect` | default, top app bar, bottom app bar, both drawers, everything composing, oversized insets |
| `resolve_side` | every anchor with and without RTL |
| `side_rect` | left, right, top offset |
| `MudAppBar` | defaults, every builder, height constants, contrast threshold |
| `MudDrawer` | defaults, every builder, default extents |
| `MudDrawerHeader` | default, dense, height constants |
| `MudLayout` | default, classes |

Rendering needs a live `egui::Context`; the pure layer is the source of truth.

```bash
cargo test -p mud_egui_core drawer_variant drawer_clip_mode
cargo test -p mud_egui_components layout
```

---

## Differences from MudBlazor

- **`LayoutState` replaces the CSS variables.** MudBlazor publishes
  `--mud-drawer-width-left`, `--mud-appbar-height` and friends from
  `MudDrawerContainer`; the port uses a caller-owned struct that each child
  writes into as it is declared. `MudMainContent` reads it — the same
  mechanism, typed.
- **`MudLayout` hands the caller a screen-sized `Ui`.** The caller chooses the
  nesting; the port does not auto-compose the shell. This is the immediate-mode
  equivalent of MudBlazor's CSS-driven placement.
- **`Open` / `OpenChanged` become `DrawerState` + `DrawerResponse`.** The caller
  owns the flag; `opened` / `closed` report the transition frame. Same split as
  every other component in this workspace.
- **`Width` / `MiniWidth` / `Height` are `f32` points, not CSS strings.**
  `300px` and `30%` have no egui equivalent; compute a percentage from
  `ui.available_width()` before calling the builder.
- **`Overlay` / `OverlayAutoClose` are honoured.** A temporary or
  responsive-below-breakpoint drawer paints a screen backdrop at `Order::Middle`
  and closes on click when `overlay_auto_close`.
- **`ClipMode` affects z-order and top offset, not push behaviour.** The
  effective variant decides whether the drawer reserves width. `Never` runs
  full height above the app bar; `Docked` / `Always` start below it.
- **`Fixed` is stored.** egui has no free-floating chrome inside a
  `ScrollArea`; scroll-away behaviour is the caller's decision.
- **`MudAppBar` does not delegate to `MudToolBar`.** MudToolBar would paint a
  second frame around the content. The app bar paints one frame and lays the
  content out inline with the same 16 pt gutters and 64 / 48 pt height.
- **`ToolBarClass` is stored, not resolved.** No CSS class system.
- **`Contextual` on `MudAppBar` is not ported.** It is a 8.14+ addition
  (PR #10210), not present in 8.13.0.
- **`Style` / `Tag` / `UserAttributes` are not ported.** No CSS strings, no
  HTML tag.