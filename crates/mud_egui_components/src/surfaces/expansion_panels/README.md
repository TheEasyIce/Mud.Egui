# MudExpansionPanels + MudExpansionPanel for `mud_egui`

An accordion container and its collapsible panel for
[egui](https://github.com/emilk/egui), ported from MudBlazor's
[`MudExpansionPanels`](https://mudblazor.com/components/expansion-panel) and
`MudExpansionPanel` (MudBlazor 8.13.0).

```rust
use mud_egui_components::surfaces::expansion_panels::{
    MudExpansionPanel, MudExpansionPanels,
};

MudExpansionPanels::new().show(ui, &mut self.state, |panels| {
    panels.add(MudExpansionPanel::new("Shipping"), |ui| {
        ui.add(MudText::body2("Arrives in 2–3 business days."));
    });
    panels.add(MudExpansionPanel::new("Returns"), |ui| {
        ui.add(MudText::body2("30 days, unused condition."));
    });
});
```

Every string renders through **`MudText`**, every glyph through **`MudIcon`**.

---

## Contents

1. [Design principle: API parity, not API invention](#design-principle-api-parity-not-api-invention)
2. [Architecture](#architecture)
3. [API reference — MudExpansionPanels](#api-reference--mudexpansionpanels)
4. [API reference — MudExpansionPanel](#api-reference--mudexpansionpanel)
5. [Examples](#examples)
6. [Cascading and state](#cascading-and-state)
7. [Layout and rendering](#layout-and-rendering)
8. [Performance and memory choices](#performance-and-memory-choices)
9. [Testing](#testing)
10. [Differences from MudBlazor](#differences-from-mudblazor)

---

## Design principle: API parity, not API invention

Both components were verified against DNDocs for MudBlazor 8.13.0 before
implementation. Between them they expose **nineteen** parameters, and all
nineteen are ported.

### `MudExpansionPanels`

| MudBlazor | Type | Default | Rust mapping |
|---|---|---|---|
| `ChildContent` | `RenderFragment?` | `null` | closure to `show(ui, state, \|panels\| ..)` |
| `Dense` | `bool` | `false` | `dense(bool)` |
| `Elevation` | `int` | **`1`** | `elevation(u8)` (clamped 0–24) |
| `Gutters` | `bool` | `true` | `gutters(bool)` |
| `MultiExpansion` | `bool` | `false` | `multi_expansion(bool)` |
| `Outlined` | `bool` | **`true`** | `outlined(bool)` |
| `Square` | `bool` | `false` | `square(bool)` |
| `Class` | `string?` | `null` | `classes(iter)` — stored |
| `Tag` | `object?` | `null` | `tag(t)` / `tag_ref::<U>()` |

### `MudExpansionPanel`

| MudBlazor | Type | Default | Rust mapping |
|---|---|---|---|
| `ChildContent` | `RenderFragment?` | `null` | closure to `show` / `ExpansionPanelsUi::add` |
| `Dense` | `bool` | `false` | `dense(bool)` |
| `Disabled` | `bool` | `false` | `disabled(bool)` |
| `Expanded` | `bool` | `false` | caller-owned `&mut bool` |
| `ExpandedChanged` | `EventCallback<bool>` | — | `PanelResponse::expanded_changed` |
| `Gutters` | `bool` | `true` | `gutters(bool)` |
| `HeaderClass` | `string?` | `null` | `header_class(s)` — stored |
| `HideIcon` | `bool` | `false` | `hide_icon(bool)` |
| `Icon` | `string` | `ExpandMore` | `icon(s)` |
| `KeepContentAlive` | `bool` | — | `keep_content_alive(bool)` — **inert** |
| `MaxHeight` | `int?` | `null` | `max_height(f32)` → `Option<f32>` |
| `Text` | `string?` | `null` | `new(text)` / `text(s)` |
| `TitleContent` | `RenderFragment?` | `null` | `title_content(f)` — overrides `Text` |
| `Class` | `string?` | `null` | `classes(iter)` — stored |
| `Tag` | `object?` | `null` | `tag(t)` / `tag_ref::<U>()` |

### Deliberately not ported

| MudBlazor | Reason |
|---|---|
| `Style` (both) | egui has no inline CSS strings. |
| `UserAttributes` (both) | There is no HTML tag to splat attributes onto. |
| `Class` / `HeaderClass` resolution | Stored for parity only. The container owns fill, border, shadow and radius; the panel owns padding and its divider. Resolving a second utility frame would composite two fills or two margins. |
| `Panels` (container) | It is a read-only view of the children. In egui the children are the caller's `&mut Vec<bool>`. |
| Protected `Classname` / `HeaderClassname` / `PanelContentClassname` | Internal CSS class getters with no egui analogue. |
| `NextPanelExpanded` | An internal bookkeeping flag used to decide border radii between adjacent panels. The container computes it directly from the panel index. |

### Ported as free functions

`ExpandAllAsync`, `CollapseAllAsync` and `CollapseAllExceptAsync` are methods on
the container instance in Blazor, reachable through `@ref`. In immediate mode
there is no instance to hold, so they become functions over the caller's state
slice: `expand_all(&mut [bool])`, `collapse_all(&mut [bool])`,
`collapse_all_except(&mut [bool], usize)`.

### `KeepContentAlive` is inert

In Blazor it keeps the collapsed content in the DOM. In egui the content
closure is **never called** while the panel is collapsed, and any widget state
is keyed by `Id` in `Memory`, so it survives collapse and expand on its own.
There is nothing for the flag to switch off. It is accepted and stored so a
Blazor call site ports without edits, and documented here rather than silently
dropped.

---

## Architecture

```
surfaces/expansion_panels/
├── mud_expansion_panel.rs
│   ├── constants        HEADER_HEIGHT 48, DENSE_HEADER_HEIGHT 32,
│   │                    CONTENT_PADDING_Y 16/8, GUTTER_PX 16,
│   │                    ICON_SIZE 24/20, ICON_GAP 8, PANEL_RADIUS 4,
│   │                    ICON_ROTATION_DEG 180, HAIRLINE 1
│   ├── pure helpers     header_height, content_padding_y, horizontal_padding,
│   │                    icon_size, next_expanded, icon_rotation,
│   │                    radius_u8, padding_i8, placed_width
│   ├── MudExpansionPanel  builder + show (standalone) + render (in container)
│   ├── PanelChrome      container-supplied overrides (Copy)
│   └── PanelResponse    header Response + inner + toggled + expanded_changed
└── mud_expansion_panels.rs
    ├── MudExpansionPanels  builder + show
    ├── ExpansionPanelsUi   collector: add(panel, content)
    ├── PanelEntry<'a>      panel + Box<dyn FnOnce(&mut Ui) + 'a>
    ├── PanelSetResponse    container Response + inner + toggled
    ├── bulk ops            expand_all, collapse_all, collapse_all_except
    └── pure helpers        sync_state_len, container_radius, elevation_shadow
```

**Pure layer.** Header geometry, the click-to-state rule, the icon angle,
state-vector resizing, the accordion rule and the shadow ramp are all free
functions over `f32`, `bool` and `&mut [bool]`. None touch `egui::Context`, so
they are unit-tested without a window.

**State layer.** Expansion lives in the caller's `&mut Vec<bool>`, indexed by
declaration order and resized to the panel count at the top of every frame.

**Component layer.** The container collects panels, resizes state, draws one
`Frame` around the whole stack, then renders each panel with a `PanelChrome`
describing the cascade and whether the panel is last.

**Content layer.** Each panel's content region is delegated to
[`MudCollapse`](../collapse/README.md), so the animation, the height cache and
the `MaxHeight` clamp are shared rather than reimplemented.

### Why the panel has two entry points

| Entry point | Used by | Surface |
|---|---|---|
| `MudExpansionPanel::show` | standalone panels | own fill, own 1 pt outline, 4 pt radius |
| `MudExpansionPanel::render` (`pub(crate)`) | `MudExpansionPanels` | fill only; the container draws the border, shadow and radius; the panel draws a 1 pt divider when it is not last |

A container that drew each panel's full border would show a 2 pt seam where two
panels meet, because both hairlines would be painted. Splitting the surface
this way gives the "one rounded card, internal rules" look that
`.mud-expansion-panels` produces in Blazor.

---

## API reference — `MudExpansionPanels`

| Method | Type | Default | Mirrors | Description |
|---|---|---|---|---|
| `new()` | `Self` | — | — | Accordion, elevation 1, gutters on, outlined, rounded. |
| `dense(b)` | `bool` | `false` | `Dense` | Compact padding on every panel. |
| `elevation(e)` | `u8` | `1` | `Elevation` | Stack shadow, clamped to 24. `0` = flat. |
| `gutters(b)` | `bool` | `true` | `Gutters` | Left/right padding on every panel. |
| `multi_expansion(b)` | `bool` | `false` | `MultiExpansion` | Allow several panels open at once. |
| `outlined(b)` | `bool` | `true` | `Outlined` | 1 pt border around the stack. |
| `square(b)` | `bool` | `false` | `Square` | Zero corner radius on the stack. |
| `classes(iter)` | `iter of AsRef<str>` | none | `Class` | Stored on `ComponentBase`. |
| `tag(t)` / `tag_ref::<U>()` | `T: Any + Send + Sync` | `None` | `Tag` | Arbitrary user data. |
| `show(ui, state, \|panels\| ..)` | `&mut Ui, &mut Vec<bool>, FnOnce(&mut ExpansionPanelsUi) -> R` | — | `ChildContent` | Returns `PanelSetResponse<R>`. |

### `ExpansionPanelsUi`

| Method | Description |
|---|---|
| `add(panel, content)` | Registers a panel and its content. Nothing is drawn until the builder closure returns. |
| `len()` / `is_empty()` | Number of panels registered so far. |

The content closure is `FnOnce(&mut Ui) + 'a`, where `'a` is the lifetime of the
`show` call — it can borrow local state, so it does **not** need to be
`'static`.

### `PanelSetResponse<R>`

| Field | Type | Description |
|---|---|---|
| `response` | `Response` | The whole stack, border included. |
| `inner` | `R` | The builder closure's return value. |
| `toggled` | `Option<usize>` | Index of the panel whose header toggled this frame. |

### Bulk operations

| Function | Mirrors | Description |
|---|---|---|
| `expand_all(&mut [bool])` | `ExpandAllAsync` | Opens every panel. |
| `collapse_all(&mut [bool])` | `CollapseAllAsync` | Closes every panel. |
| `collapse_all_except(&mut [bool], usize)` | `CollapseAllExceptAsync` | Closes all but one; out-of-range indices are ignored. |

---

## API reference — `MudExpansionPanel`

| Method | Type | Default | Mirrors | Description |
|---|---|---|---|---|
| `new(text)` | `impl Into<String>` | `""` | `Text` | Creates a panel with a text header. |
| `text(s)` | `impl Into<String>` | — | `Text` | Replaces the header caption. |
| `title_content(f)` | `FnMut(&mut Ui) + 'static` | `None` | `TitleContent` | Custom header; overrides `Text`. |
| `dense(b)` | `bool` | `false` | `Dense` | 32 pt header, 8 pt content padding, 20 pt icon. |
| `disabled(b)` | `bool` | `false` | `Disabled` | Header ignores clicks. |
| `gutters(b)` | `bool` | `true` | `Gutters` | 16 pt left/right padding. |
| `header_class(s)` | `impl Into<String>` | `None` | `HeaderClass` | Stored, not resolved. |
| `hide_icon(b)` | `bool` | `false` | `HideIcon` | Hides the expand glyph (clicks still work). |
| `icon(s)` | `impl Into<String>` | `ExpandMore` | `Icon` | Expand glyph override. |
| `keep_content_alive(b)` | `bool` | `false` | `KeepContentAlive` | Inert in egui. |
| `max_height(px)` | `f32` | `None` | `MaxHeight` | Content height cap in points. |
| `id_salt(s)` | `impl Hash + Debug` | `None` | — | Stable animation identity. The container sets one automatically. |
| `classes(iter)` | `iter of AsRef<str>` | none | `Class` | Stored on `ComponentBase`. |
| `tag(t)` / `tag_ref::<U>()` | `T: Any + Send + Sync` | `None` | `Tag` | Arbitrary user data. |
| `show(ui, &mut bool, \|ui\| ..)` | `&mut Ui, &mut bool, FnOnce(&mut Ui) -> R` | — | — | Standalone render; returns `PanelResponse<R>`. |

### `PanelResponse<R>`

| Field | Type | Description |
|---|---|---|
| `response` | `Response` | The header. `PanelResponse` derefs to it. |
| `inner` | `Option<R>` | The content closure's value; `None` while fully collapsed. |
| `expanded` | `bool` | The state after this frame. |
| `toggled` | `bool` | `true` when the header toggled the state this frame. |
| `expanded_changed` | `Option<bool>` | `Some(new)` on the toggling frame. Mirrors `ExpandedChanged`. |

### Constants

| Name | Value |
|---|---|
| `HEADER_HEIGHT` / `DENSE_HEADER_HEIGHT` | `48.0` / `32.0` |
| `CONTENT_PADDING_Y` / `DENSE_CONTENT_PADDING_Y` | `16.0` / `8.0` |
| `GUTTER_PX` | `16.0` |
| `ICON_SIZE` / `DENSE_ICON_SIZE` | `24.0` / `20.0` |
| `ICON_GAP` | `8.0` |
| `PANEL_RADIUS` / `CONTAINER_RADIUS` | `4.0` / `4.0` |
| `ICON_ROTATION_DEG` | `180.0` |
| `HAIRLINE` | `1.0` |
| `MAX_ELEVATION` | `24` |

---

## Examples

### Accordion (the default)

```rust
MudExpansionPanels::new().show(ui, &mut self.state, |panels| {
    panels.add(MudExpansionPanel::new("One"), |ui| { ui.label("1"); });
    panels.add(MudExpansionPanel::new("Two"), |ui| { ui.label("2"); });
});
```

Opening `Two` closes `One` automatically.

### Multi-expansion

```rust
MudExpansionPanels::new()
    .multi_expansion(true)
    .show(ui, &mut self.state, |panels| {
        for (i, section) in self.sections.iter().enumerate() {
            panels.add(MudExpansionPanel::new(&section.title), |ui| {
                ui.add(MudText::body2(&section.body));
            });
            let _ = i;
        }
    });
```

### Opening one panel from a button

```rust
if ui.button("Show returns policy").clicked() {
    collapse_all(&mut self.state);
    self.state[2] = true;          // index 2 = "Returns"
}
```

### Expand all / collapse all

```rust
ui.horizontal(|ui| {
    if ui.button("Expand all").clicked() {
        expand_all(&mut self.state);
    }
    if ui.button("Collapse all").clicked() {
        collapse_all(&mut self.state);
    }
});
```

### Flat, square, flush stack

```rust
MudExpansionPanels::new()
    .elevation(0)
    .outlined(false)
    .square(true)
    .gutters(false)
    .show(ui, &mut self.state, |panels| { /* … */ });
```

### Dense stack with a height-capped panel

```rust
MudExpansionPanels::new()
    .dense(true)
    .show(ui, &mut self.state, |panels| {
        panels.add(
            MudExpansionPanel::new("Build log").max_height(160.0),
            |ui| {
                for line in &self.log {
                    ui.add(MudText::monospace(line.clone()));
                }
            },
        );
    });
```

### Custom header

```rust
panels.add(
    MudExpansionPanel::new("")
        .title_content(|ui| {
            ui.add(MudIcon::new(filled::SHIPPING).size_px(18.0));
            ui.add(MudText::subtitle1("Shipping"));
            ui.add(MudText::caption("2–3 days").color(Color::TextSecondary));
        }),
    |ui| { ui.label("…"); },
);
```

### Disabled panel

```rust
panels.add(
    MudExpansionPanel::new("Enterprise (contact sales)").disabled(true),
    |ui| { ui.label("…"); },
);
```

The header still renders and hovers, but the state never changes.

### Custom expand glyph

```rust
MudExpansionPanel::new("Filters").icon(filled::ADD).show(ui, &mut open, |ui| {
    ui.label("…");
});
```

The glyph rotates 180° as the panel expands, so pick a symmetric one.

### Standalone panel (no container)

```rust
let mut open = false;
MudExpansionPanel::new("Standalone").show(ui, &mut open, |ui| {
    ui.label("Own fill, own border, own radius.");
});
```

### Reacting to a toggle

```rust
let out = MudExpansionPanels::new().show(ui, &mut self.state, |panels| { /* … */ });
if let Some(index) = out.toggled {
    log::info!("panel {index} toggled to {}", self.state[index]);
}
```

---

## Cascading and state

### Cascade rules

`Dense` and `Gutters` are non-nullable on both types, so the container folds
onto the panel rather than "filling in" an unset value:

| Container | Panel | Effective |
|---|---|---|
| `dense(false)` | `dense(false)` | standard |
| `dense(true)` | `dense(false)` | **dense** — container wins |
| `dense(false)` | `dense(true)` | **dense** |
| `gutters(true)` | `gutters(true)` | padded |
| `gutters(false)` | `gutters(true)` | **no padding** — container wins |
| `gutters(true)` | `gutters(false)` | **no padding** |

A panel can opt into density and opt out of gutters, but not the reverse. That
is the only reading consistent with MudBlazor's wording for non-nullable flags.

`Outlined`, `Elevation` and `Square` are container-only in 8.13.0 and apply to
the stack, not to individual panels.

### State vector

```rust
let mut state: Vec<bool> = Vec::new();   // grow it however you like
MudExpansionPanels::new().show(ui, &mut state, |panels| { /* … */ });
// After the call, state.len() == number of panels registered.
```

`show` calls `state.resize(count, false)` before drawing, so a panel added this
frame starts collapsed and a removed panel's flag is dropped. Keep the
declaration order stable if you want a given flag to keep belonging to the same
panel.

### Accordion enforcement

When `multi_expansion` is off and a panel reports `toggled` **to** expanded, the
container sets every other flag to `false` in the same frame, before those
panels are rendered — so they draw collapsed immediately, with no one-frame lag.

### `ExpandedChanged` on the panel

`PanelResponse::expanded_changed` is `Some(new)` only on the frame the header
clicked. It is never set on a programmatic change (writing `state[i] = true`
yourself), because the panel sees the flag already set — the same way an
`ExpandedChanged` callback fires in Blazor only from a user interaction or an
explicit `ExpandAsync()` call.

---

## Layout and rendering

### Header

```
LTR:  [ pad_x ][ title area …………………………… ][ gap 8 ][ icon 24 ][ pad_x ]
RTL:  not mirrored (see Differences)
```

| Density | Header height | Icon side |
|---|---|---|
| standard | 48 pt | 24 pt |
| dense | 32 pt | 20 pt |

- Hover paints `widgets.hovered.weak_bg_fill` over the header rect, behind the
  title and the icon.
- A non-disabled header gets `Sense::click()` and a pointing-hand cursor; a
  disabled header gets `Sense::hover()` only.
- `HideIcon(true)` removes both the glyph **and** its reserved column, so the
  title area grows to fill the space. Clicks still toggle.
- The icon rotates `180° × animate_bool_with_time(id, expanded, 0.2)`.

### Content

```
[ header ]
[ content, padding_y 16 (8 dense), delegated to MudCollapse ]
[ 1 pt divider, if inside a container and not last ]
```

The content region is a `MudCollapse`, so:

- it is clipped and faded while animating;
- its natural height is cached and only remeasured when it moves by more than
  0.1 pt;
- `MaxHeight` clamps both the reserved and the visible height;
- the closure is not called while fully collapsed.

### Stack

The container wraps all panels in one `egui::Frame` with:

| Container flag | Frame property |
|---|---|
| (always) | `fill = palette.surface` |
| `outlined(true)` | `stroke = 1 pt, widgets.noninteractive.bg_stroke.color` |
| `elevation > 0` | `shadow = shadow_ramp(elevation)` |
| `square(false)` | `corner_radius = 4 pt` |

Panels inside it paint no surface of their own. A 1 pt divider is drawn under
every panel except the last, in the same colour as the outline, so the stack
reads as one rounded card with internal rules.

---

## Performance and memory choices

| Choice | Effect |
|---|---|
| Two-phase render | Panels are collected before any is drawn, so accordion enforcement and the "is this the last panel" divider rule are both correct within one frame. |
| Non-`'static` content closures | `Box<dyn FnOnce(&mut Ui) + 'a>` borrows for the length of the `show` call, so a panel's content can read `&self` instead of cloning into `move` closures or `Rc<RefCell<_>>`. |
| Collapsed panels are free | `MudCollapse` skips the content closure entirely once settled, so twenty closed panels cost twenty header rows and nothing else. |
| One `Frame` for the whole stack | One fill, one stroke, one shadow — not one per panel. |
| One `MudCollapse` per panel | The height cache, the epsilon and the `MaxHeight` clamp are shared with `MudCollapse` rather than duplicated. |
| Header is one allocation | `allocate_exact_size` for the header rect plus a child `Ui` for the title; no nested layouts, no per-frame `Vec`. |
| `mem::take` for the caption | The `String` is moved into `MudText` rather than cloned. |
| Pure helpers on `Copy` values | Header geometry, the toggle rule, the icon angle and state resizing are arithmetic; unit-testable without a `Context`. |
| `state.resize` is O(delta) | Growing by one panel appends one `false`; the steady state does no work. |

---

## Testing

`cargo test -p mud_egui_components mud_expansion` runs 26 tests:

| File | Area | Tests |
|---|---|---|
| `mud_expansion_panel.rs` | Metrics | header height per density; content padding per density; gutter padding; icon size per density; `radius_u8` / `padding_i8` clamping; `placed_width` |
| | Toggle rule | `next_expanded` truth table; disabled ignores clicks |
| | Icon | rotation spans 0–180; clamps and sanitizes `NaN` |
| | Chrome | `PanelChrome::standalone` has no overrides |
| | Builders | defaults match MudBlazor; every field set; `id_salt` stability; tag round-trip |
| `mud_expansion_panels.rs` | Builders | defaults (elevation **1**, outlined **true**); every field set; tag round-trip |
| | State | `sync_state_len` grows with `false`, shrinks, is idempotent |
| | Bulk ops | `expand_all`, `collapse_all`, `collapse_all_except` (including out-of-range and empty) |
| | Chrome | container radius; shadow `None` at 0 and clamped at 24 |
| | Cascade | container dense forces dense; container gutters-off forces flush; panel can opt in to dense and out of gutters |

No test creates an `egui::Context`. `show` and `render` need a live `Ui` and are
left to integration tests; everything they delegate to is covered above.

Suggested integration tests (`egui_kittest` or a headless `Context::run`
harness):

- Clicking a header flips that panel and, in accordion mode, closes the others
  in the same frame.
- `multi_expansion(true)` lets two panels be open at once.
- A disabled panel's header click leaves the state untouched.
- `HideIcon(true)` widens the title area and still toggles.
- `MaxHeight` caps the expanded content and clips the overflow.
- A stack of three panels draws exactly two dividers.

```bash
cargo test -p mud_egui_components mud_expansion
```

---

## Differences from MudBlazor

- **`Expanded` / `ExpandedChanged` become a `&mut bool` plus a return field.**
  The caller owns the flag; `PanelResponse::expanded_changed` reports the
  toggling frame. This is the same split `MudCollapse` and `MudAlert` use, and
  it means `MudExpansionPanels` never stores hidden state.
- **`ExpandAllAsync` / `CollapseAllAsync` / `CollapseAllExceptAsync` are free
  functions over `&mut [bool]`.** There is no instance to hold in immediate
  mode; the caller owns the vector.
- **`Panels` is the caller's vector.** MudBlazor exposes a read-only list of
  child components; here the children *are* the state.
- **`Class` and `HeaderClass` are stored but not resolved.** The container owns
  fill, border, shadow and radius, and the panel owns padding and its divider.
  Resolving another utility frame would composite a second fill or a second
  margin.
- **`Style` and `UserAttributes` are not ported.** No CSS strings, no HTML tag.
- **`KeepContentAlive` is inert.** In egui the content closure is simply not
  called while collapsed, and widget state survives in `Memory` regardless.
- **`Elevation` and `Outlined` are `u8` / `bool` with the same defaults as
  MudBlazor (`1` and `true`).** Elevation is clamped to `0..=24` like every
  other component in this workspace.
- **`MaxHeight` is `f32` points, not `int` pixels.** Consistent with
  `MudCollapse`, `MudStack` and `MudCardMedia`.
- **No `Ripple` parameter.** MudBlazor 8.13.0's `MudExpansionPanel` does not
  have one; adding it would be API invention. The header's hover wash is the
  only interaction feedback.
- **One `Icon`, not an expand/collapse pair.** 8.13.0 exposes a single `Icon`
  (default `ExpandMore`), which this port rotates 180° when expanded — matching
  `.mud-expand-panel-icon`'s CSS transform. If your MudBlazor build shows a
  static icon, delete the `.rotation(..)` call in `render`; it is one line.
- **No RTL mirroring.** `MudExpansionPanel` has no `RightToLeft` parameter in
  8.13.0 (unlike `MudTimeline`, which does), so the header is always
  left-to-right. Wrap the stack in an RTL-aware parent if you need the mirror.
- **Panel surfaces are split between the container and the panel.** Standalone
  panels draw their own fill, outline and radius; inside a container the
  container draws one border, shadow and radius and the panel draws only a
  divider. This avoids the 2 pt seam you get when two hairlines meet, and
  reproduces MudBlazor's "one rounded card, internal rules" look.
- **Accordion enforcement happens before rendering.** Because panels are
  collected first, closing the others takes effect in the same frame rather
  than one frame later.