# MudPopover for `mud_egui`

The anchored overlay primitive for [egui](https://github.com/emilk/egui),
ported from MudBlazor's `MudPopover` (MudBlazor 8.13.0). This is the
foundation that `MudSelect`, `MudAutocomplete`, the picker family,
`MudMenu`, and `MudTooltip` all sit on top of.

```rust
let anchor_rect = ui
    .add(egui::Button::new("Open menu"))
    .rect;

let (resp, _) = MudPopover::new()
    .anchor_origin(AnchorOrigin::Bottom)
    .transform_origin(TransformOrigin::TopLeft)
    .match_anchor_width(true)
    .show(ui, &mut self.menu_state, anchor_rect, |ui| {
        ui.add(MudText::new("Item 1"));
        ui.add(MudText::new("Item 2"));
    });
```

The popover itself renders no text or icons — the content closure does. It
is expected to use `MudText` / `MudIcon` per project convention.

---

## Contents

1. [Design principle: API parity, not API invention](#design-principle-api-parity-not-api-invention)
2. [Architecture](#architecture)
3. [Positioning model](#positioning-model)
4. [API reference](#api-reference)
5. [Examples](#examples)
6. [Performance and memory choices](#performance--memory-choices)
7. [Testing](#testing)
8. [Differences from MudBlazor](#differences-from-mudblazor)

---

## Design principle: API parity, not API invention

MudBlazor's `MudPopover` parameter surface was verified against DNDocs before
this port. A few MudBlazor parameters have no egui equivalent (see
"Deliberately not ported") and are documented as such.

### Ported parameters

| MudBlazor | Rust method | Default |
|-----------|-------------|---------|
| `AnchorOrigin` | `anchor_origin(AnchorOrigin)` | `Bottom` |
| `TransformOrigin` | `transform_origin(TransformOrigin)` | `TopLeft` |
| `OverflowBehavior` | `overflow_behavior(OverflowBehavior)` | `Flip` |
| `Paper` | `paper(bool)` | `true` |
| `Elevation` | `elevation(u8)` | `8` |
| `Square` | `square(bool)` | `false` |
| `Open` | `PopoverState::open` | `false` |
| `Class` | `classes(iter)` | `None` |
| `Style` | *(use `classes`)* | — |
| `ChildContent` | closure to `show` | required |

### Extensions (not in MudBlazor)

| Extension | Description |
|-----------|-------------|
| `backdrop(BackdropMode)` | Opt-in backdrop. Default `None` (no backdrop, drops clicks). `Transparent` catches clicks. `Dimmed(alpha)` paints a semi-transparent overlay. |
| `dismiss_on_escape(bool)` | Escape dismisses (default `true`). |
| `dismiss_on_outside_click(bool)` | Outside click dismisses (default `true`). |
| `offset(f32)` | Gap between anchor and popover. |
| `min_width(f32)` / `max_width(f32)` | Explicit clamps. |
| `match_anchor_width(bool)` | Sets `min_width = anchor_rect.width()` when the caller has not set it. |
| `right_to_left(bool)` | Resolves `AnchorOrigin::Start` / `End`. |
| `size_hint(Vec2)` | Initial size guess used on the first open frame. |
| `PopoverResponse` | Rich return value with `open` and `response`. |

### Deliberately not ported

- **`Fixed`** — egui's `Area` is always screen-fixed. The parameter would be
  a no-op.
- **`Style`** — egui has no inline CSS strings; use `classes(..)`.
- **`OverflowBehavior::Shrink`** — needs to know the popover size before
  placing it. Deferred to a future pass (see "Size cache" below for the
  current approximation).

### `AnchorOrigin` (renamed from MudBlazor's `Anchor`)

| Variant | Meaning |
|---------|---------|
| `Top` | Popover above the anchor |
| `Bottom` (default) | Popover below the anchor |
| `Left` | Popover to the left |
| `Right` | Popover to the right |
| `Start` | Logical left in LTR, right in RTL |
| `End` | Logical right in LTR, left in RTL |

### `TransformOrigin` (MudBlazor's `Origin`)

Nine values: `TopLeft` (default), `TopCenter`, `TopRight`, `CenterLeft`,
`Center`, `CenterRight`, `BottomLeft`, `BottomCenter`, `BottomRight`.

The pair `AnchorOrigin::Bottom + TransformOrigin::TopLeft` places the
popover's top-left corner at the anchor's bottom-center — the standard
dropdown layout.

### `OverflowBehavior`

| Variant | Behaviour |
|---------|-----------|
| `Flip` (default) | Flip to opposite side when the preferred position overflows; clamp as last resort |
| `Reposition` | Clamp into screen bounds without flipping |
| `Shrink` | Treated as `Flip` in pass 1; caller must set `max_height` manually |

### `BackdropMode`

| Variant | Behaviour |
|---------|-----------|
| `None` (default) | No backdrop; outside clicks still dismiss if `dismiss_on_outside_click` is on, but the parent UI is interactive |
| `Transparent` | Full-screen click-catcher; parent UI blocked, nothing painted |
| `Dimmed(f32)` | Full-screen overlay at `alpha`; parent UI blocked, semi-transparent black painted |

---

## Architecture

```
overlays/popover/
├── popover_state.rs    — PopoverState { open, id }
├── popover_response.rs — PopoverResponse { open, response }
├── mud_popover.rs      — MudPopover + pure geometry + tests
└── mod.rs              — re-exports

mud_egui_core/src/enums/
├── anchor_origin.rs
├── backdrop_mode.rs
├── overflow_behavior.rs
└── transform_origin.rs
```

**Pure geometry.** `anchor_point`, `transform_offset`, `offset_vec`,
`compute_position`, `flip_vertical`, `flip_horizontal`, `resolve_overflow`
are free functions. All positioning is unit-tested without `egui::Context`.

**State layer.** `PopoverState { open: bool, id: Id }` — the egui equivalent
of MudBlazor's `MudPopover.Open` parameter plus a stable id used for the
size cache.

**Component layer.** `show` computes the target position, paints any
backdrop, renders the popover content into a `Foreground`-ordered
`egui::Area`, caches the actual size for the next frame, and applies
dismissal rules.

---

## Positioning model

```
               anchor_rect
          ┌────────────────────┐
          │                    │
          │                    │  ◄── AnchorOrigin::Top    = top-center
          │                    │
          │                    │  ◄── AnchorOrigin::Left   = left-center
          │                    │       AnchorOrigin::Right = right-center
          │                    │
          │                    │  ◄── AnchorOrigin::Bottom = bottom-center
          └────────────────────┘

          transform_origin determines which corner of the popover
          snaps to that anchor point.
```

### The math

```
anchor_point = point on anchor_rect (top-center, bottom-center, etc.)
offset_vec   = small push away from anchor (offset parameter)
transform    = shift by (transform_origin fraction of popover size)
position     = anchor_point + offset_vec + transform
```

### Example: standard dropdown

| Input | Value |
|-------|-------|
| `anchor_rect` | `Rect { min: (100, 200), size: (200, 40) }` (a text field) |
| `anchor_origin` | `Bottom` |
| `transform_origin` | `TopLeft` |
| `offset` | `4.0` |
| popover size | `(200, 300)` |

`anchor_point = (200, 240)` (bottom-center of the field)  
`offset_vec = (0, 4)` (push down 4 px)  
`transform = (0, 0)` (top-left at anchor point)  
**final position = (200, 244)**

### Flip logic

When the initial position overflows the screen (8 px margin), the port
attempts a flip:

1. If the vertical axis overflows → try `flip_vertical(anchor, transform)`.
2. If the horizontal axis overflows → try `flip_horizontal(anchor, transform)`.
3. After flipping, if it still overflows → clamp to `screen - 8 px`.

`OverflowBehavior::Reposition` skips the flip attempt and goes straight to
clamping.

### Size cache — the chicken-and-egg problem

egui is immediate-mode; the port cannot know the popover size before it is
rendered. It cannot know where to render without the size. The port solves
this with a **size cache in `egui::Memory`**, keyed by `PopoverState::id`:

| Frame | What happens |
|-------|--------------|
| **Open (1st frame)** | `read_cached_size` returns `None`. Use `size_hint` (or `DEFAULT_SIZE_HINT = vec2(200, 200)`). Compute position from that hint. Render. Cache the actual size. |
| **Open (2nd frame)** | `read_cached_size` returns the actual size from frame 1. Compute position from that. Render. Re-cache. |
| **Closed** | No rendering, no caching. |

The one-frame lag is invisible in practice (dropdowns are opened by a
click; the popover appears and settles into place within a frame or two).
If you care about the *first* frame being pixel-perfect, set `size_hint`
to the approximate size of the popover content:

```rust
MudPopover::new()
    .size_hint(vec2(280.0, 300.0))
    .show(ui, &mut self.menu_state, anchor_rect, |ui| {
        for item in &self.items { /* … */ }
    });
```

---

## API reference

### `MudPopover<'a>`

| Method | Type | Default | Description |
|--------|------|---------|-------------|
| `new()` | — | — | Default popover. |
| `anchor_origin(AnchorOrigin)` | `AnchorOrigin` | `Bottom` | Which edge of the anchor. |
| `transform_origin(TransformOrigin)` | `TransformOrigin` | `TopLeft` | Which corner of the popover. |
| `overflow_behavior(OverflowBehavior)` | `OverflowBehavior` | `Flip` | Flip / reposition / shrink. |
| `paper(bool)` | `bool` | `true` | Draw paper background. |
| `elevation(u8)` | `u8` | `8` | Drop-shadow depth. |
| `square(bool)` | `bool` | `false` | Removes paper corner rounding. |
| `backdrop(BackdropMode)` | `BackdropMode` | `None` | Backdrop. |
| `dismiss_on_escape(bool)` | `bool` | `true` | Escape dismisses. |
| `dismiss_on_outside_click(bool)` | `bool` | `true` | Outside click dismisses. |
| `offset(f32)` | `f32` | `0.0` | Gap from anchor. |
| `min_width(f32)` | `f32` | `None` | Explicit minimum width. |
| `max_width(f32)` | `f32` | `None` | Explicit maximum width. |
| `match_anchor_width(bool)` | `bool` | `false` | Sets `min_width = anchor_rect.width()`. |
| `right_to_left(bool)` | `bool` | `false` | Resolves `Start` / `End`. |
| `size_hint(Vec2)` | `Vec2` | `None` | Initial size guess. |
| `classes(iter)` | `iter of AsRef<str>` | none | Paper utility classes. |
| `flavor(StyleFlavor)` | `StyleFlavor` | `Bootstrap` | Utility resolver. |
| `show(ui, state, anchor, \|ui\| ..)` | `(&mut Ui, &mut PopoverState, Rect, FnOnce(&mut Ui) -> R)` | — | Returns `(PopoverResponse, Option<R>)`. |

### `PopoverState`

| Field / method | Description |
|----------------|-------------|
| `open: bool` | Whether the popover is open. |
| `id: Id` | Stable identity for the Area and the size cache. |
| `new(id_salt)` | Closed state with a stable id. |
| `open()` / `close()` / `toggle()` | Mutators. |

### `PopoverResponse`

| Field | Type | Description |
|-------|------|-------------|
| `open` | `bool` | Open state **after** this frame's dismissal handling. |
| `response` | `egui::Response` | Popover's response; zero-sized when closed. |
| `is_closed()` | `bool` | Convenience. |

### `OverflowResult`

Return value of `resolve_overflow`:

| Field | Description |
|-------|-------------|
| `pos` | Final top-left position. |
| `anchor` | Final (possibly flipped) anchor edge. |
| `transform` | Final (possibly flipped) transform origin. |

### Free geometry helpers

| Function | Signature |
|----------|-----------|
| `anchor_point` | `(Rect, AnchorOrigin) -> Pos2` |
| `transform_offset` | `(TransformOrigin, Vec2) -> Vec2` |
| `offset_vec` | `(AnchorOrigin, f32) -> Vec2` |
| `compute_position` | `(Rect, AnchorOrigin, TransformOrigin, Vec2, f32) -> Pos2` |
| `flip_vertical` | `(AnchorOrigin, TransformOrigin) -> Option<(AnchorOrigin, TransformOrigin)>` |
| `flip_horizontal` | `(AnchorOrigin, TransformOrigin) -> Option<(AnchorOrigin, TransformOrigin)>` |
| `resolve_overflow` | `(Rect, AnchorOrigin, TransformOrigin, Vec2, f32, Rect, OverflowBehavior) -> OverflowResult` |

---

## Examples

### Standard dropdown (below a text field)

```rust
let resp = MudTextField::new(&mut self.filter)
    .adornment_end(AdornmentEnd::icon(filled::ARROW_DROP_DOWN))
    .readonly(true)
    .show(ui);

if resp.response.clicked() {
    self.popover_state.toggle();
}

MudPopover::new()
    .anchor_origin(AnchorOrigin::Bottom)
    .transform_origin(TransformOrigin::TopLeft)
    .match_anchor_width(true)
    .offset(2.0)
    .show(ui, &mut self.popover_state, resp.response.rect, |ui| {
        // dropdown items
    });
```

### Tooltip (above, centered)

```rust
MudPopover::new()
    .anchor_origin(AnchorOrigin::Top)
    .transform_origin(TransformOrigin::BottomCenter)
    .paper(true)
    .elevation(4)
    .offset(6.0)
    .dismiss_on_outside_click(false)  // tooltips stay open on outside click
    .show(ui, &mut self.tooltip_state, widget_rect, |ui| {
        ui.add(MudText::new("Tooltip text").font_size(12.0));
    });
```

### Submenu (to the right, vertically centered)

```rust
MudPopover::new()
    .anchor_origin(AnchorOrigin::Right)
    .transform_origin(TransformOrigin::CenterLeft)
    .offset(4.0)
    .show(ui, &mut self.submenu_state, parent_item_rect, |ui| {
        // submenu items
    });
```

### Modal-style popover with dimmed backdrop

```rust
MudPopover::new()
    .anchor_origin(AnchorOrigin::Bottom)
    .transform_origin(TransformOrigin::TopCenter)
    .backdrop(BackdropMode::Dimmed(0.4))
    .paper(true)
    .elevation(12)
    .show(ui, &mut self.state, anchor_rect, |ui| {
        // modal content
    });
```

### Popover with explicit width

```rust
MudPopover::new()
    .min_width(280.0)
    .max_width(400.0)
    .show(ui, &mut self.state, anchor_rect, |ui| {
        // content
    });
```

### Manual open/close

```rust
// From anywhere in your app:
self.popover_state.open();
self.popover_state.close();
self.popover_state.toggle();

// Or set directly:
self.popover_state.open = true;
```

### First-frame size hint

```rust
MudPopover::new()
    .size_hint(vec2(280.0, 350.0))
    .show(ui, &mut self.state, anchor_rect, |ui| {
        // sizeable content
    });
```

---

## Performance and memory choices

| Choice | Effect |
|--------|--------|
| Pure geometry helpers | No `egui::Context`; unit-tested. |
| `PopoverState` is `Copy` | Zero-cost snapshot. |
| Size cache in `egui::Memory` | Persistent across frames, keyed by `PopoverState::id`, no caller-side storage needed. |
| Content closure runs **once** per frame | No double-render for measure; the size cache does the job. |
| `Area` at `Order::Foreground` | Renders above parent UI. |
| Backdrop `Area` at `Order::Middle` | Blocks parent, sits below the popover. |
| `content` closure never runs when closed | Idle popovers cost one `allocate_response(Vec2::ZERO, Sense::hover())`. |
| `paper` frame is resolved once per frame | `BsStyle` / `TwStyle` called once. |
| `resolve_overflow` is a single pass | No iterative fitting. |

---

## Testing

`cargo test` runs 30+ pure-function tests:

| Area | Tests |
|------|-------|
| `anchor_point` | Top / Bottom / Left / Right on a known rect |
| `transform_offset` | TopLeft / TopCenter / BottomRight / Center |
| `offset_vec` | Top / Bottom / Left / Right |
| `compute_position` | dropdown below left / centered, tooltip above, submenu right |
| `flip_vertical` | Bottom→Top, Top→Bottom, horizontal anchor returns None, centered transform returns None |
| `flip_horizontal` | Right→Left, vertical anchor returns None |
| `resolve_overflow` | fits, flips above, reposition clamps, horizontal flip, clamp-when-flip-fails |
| `elevation_shadow` | zero, clamped above 24 |
| Defaults | every `MudPopover` field matches MudBlazor |
| Builders | every field set |

Rendering (`show`) needs a live `egui::Context`; the pure layer is the source
of truth.

```bash
cargo test -p mud_egui_components mud_popover
cargo test -p mud_egui_core anchor_origin
cargo test -p mud_egui_core transform_origin
cargo test -p mud_egui_core overflow_behavior
cargo test -p mud_egui_core backdrop_mode
```

---

## Differences from MudBlazor

- **`Anchor` renamed to `AnchorOrigin`.** The popover has both an "anchor
  origin" (which edge of the anchor) and a "transform origin" (which corner
  of the popover). MudBlazor calls the first one `Anchor` — the port uses a
  more descriptive name that makes the relationship obvious at call sites.
- **`Anchor` variants are 6, not 9.** MudBlazor's `Anchor` has
  `Top / Bottom / Left / Right / Start / End` — the horizontal alignment is
  delegated to `TransformOrigin`. The port matches.
- **`Fixed` not ported.** egui's `Area` is always screen-fixed; the
  parameter would be a no-op.
- **`Shrink` overflow behaviour treated as `Flip`.** egui cannot measure the
  popover size before placing it; `Shrink` needs the caller to set
  `max_height` manually. Reserved for a future pass where popovers can
  measure their content cheaply.
- **Size cache replaces MudBlazor's DOM-based measurement.** MudBlazor
  measures the popover's DOM element before positioning it; egui's
  immediate-mode model has no equivalent. The port caches the actual size in
  `egui::Memory` and uses the previous frame's size for positioning — one
  frame of lag, invisible in practice.
- **`BackdropMode` is an extension.** MudBlazor's `MudPopover` has no
  backdrop parameter; dismissal is handled by the host's outside-click
  listener. The port exposes a `BackdropMode` because the same primitive
  will be reused by the dialog host later.
- **`size_hint` is an extension.** Not in MudBlazor (DOM measurement makes it
  unnecessary there). Provided so callers who care about pixel-perfect
  first-frame rendering can pre-announce the content size.
- **`Style` → `classes(..)`.** egui has no inline CSS strings.