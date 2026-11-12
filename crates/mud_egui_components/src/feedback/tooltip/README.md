# MudTooltip for `mud_egui`

A small popup that shows additional information when the pointer hovers, the
child gains focus, or the child is clicked, for
[egui](https://github.com/emilk/egui). Ported from MudBlazor's
`MudTooltip` (MudBlazor 8.13.0).

```rust
let mut tip = TooltipState::new("save-tip");

MudTooltip::new()
    .text("Save the file (Ctrl+S)")
    .placement(Placement::Top)
    .show(ui, &mut tip, |ui| {
        ui.add(MudButton::new("Save"))
    });
```

The trigger is whatever you draw — `MudTooltip` does not impose a widget
style. Tooltip text renders through **`MudText`**; custom bodies via
`tooltip_content` should use `MudText` / `MudIcon` per project convention.

---

## Contents

1. [Design principle: composition + caller-owned state](#design-principle-composition--caller-owned-state)
2. [Architecture](#architecture)
3. [API reference](#api-reference)
4. [Examples](#examples)
5. [Trigger semantics](#trigger-semantics)
6. [Animation and delays](#animation-and-delays)
7. [Performance and memory choices](#performance--memory-choices)
8. [Testing](#testing)
9. [Differences from MudBlazor](#differences-from-mudblazor)

---

## Design principle: composition + caller-owned state

`MudTooltip` is built from one other primitive:

| Primitive | Role |
|-----------|------|
| [`MudPopover`](../popover/README.md) | The positioned tooltip body. Anchor attachment, flip-when-overflowing, and z-ordering are all delegated. |

Unlike `MudSelect` / `MudAutocomplete` / `MudMenu`, **`MudPopover`'s dismiss
hooks are disabled**. A tooltip is display-only — the component itself decides
when to show and hide (hover delay + blur + click-toggle). If the popover's
`dismiss_on_outside_click` were enabled, clicking anywhere would prematurely
close the tooltip while the pointer is still on the trigger.

### Ported parameters

| MudBlazor | Rust method | Default |
|-----------|-------------|---------|
| `Arrow` | `arrow(bool)` | `false` |
| `ChildContent` | `child` argument of `show` | required |
| `Color` | `color(Color)` | `Default` |
| `Delay` | `delay_ms(u32)` | `0` |
| `Disabled` | `disabled(bool)` | `false` |
| `Duration` | `duration_ms(u32)` | `251` |
| `Inline` | `inline(bool)` | `true` (stored; no-op) |
| `Placement` | `placement(Placement)` | `Bottom` |
| `RightToLeft` | `right_to_left(bool)` | `false` |
| `RootClass` | `classes(iter)` | `None` |
| `RootStyle` | *(use `classes`)* | — |
| `ShowOnClick` | `show_on_click(bool)` | `false` |
| `ShowOnFocus` | `show_on_focus(bool)` | `true` |
| `ShowOnHover` | `show_on_hover(bool)` | `true` |
| `Text` | `text(s)` | `None` |
| `TooltipContent` | `tooltip_content(f)` | `None` |
| `Visible` | `TooltipState::visible` | `false` |
| `VisibleChanged` | `on_visible_changed(f)` | `None` |

### Extensions (not in MudBlazor)

| Extension | Description |
|-----------|-------------|
| `TooltipState` | Caller-owned visibility + timers. |
| `TooltipResponse` | Frame's `child` response, resulting `visible`, and whether it changed. |
| `compute_opacity` / `resolve_placement` / `placement_to_origins` | Pure helpers, unit-tested. |

### Deferred to pass 2

- **`Inline`** — MudBlazor's block/inline toggle affects how the *trigger*
  flows in the surrounding layout. egui tooltips are always overlaid on top
  of the parent layout, so the parameter has no equivalent. Stored for parity.
- **`RootStyle`** — egui has no inline CSS strings; use `classes(..)`.
- **Protruding arrow** — pass 1 draws the arrow as a small triangle *inside*
  the tooltip paper, on the anchor-facing edge. MudBlazor's arrow protrudes
  *outside* the paper and touches the anchor. The Rust port's arrow is
  functionally equivalent (it points toward the anchor and expands the visual
  silhouette) but is a few pixels smaller. A proper outdenting caret via a
  second `Area` is deferred to pass 2.

### Not ported

- **`Class` / `Style` / `Tag` / `UserAttributes`** — no CSS strings or HTML
  tags in egui.

---

## Architecture

```
overlays/tooltip/
├── tooltip_state.rs  — TooltipState (visibility + timers + click latch)
├── mud_tooltip.rs    — MudTooltip + TooltipResponse + pure helpers + tests
└── mod.rs
```

**Pure layer.** `resolve_placement`, `placement_to_origins`,
`compute_opacity`, `desired_visibility`, `resolve_tooltip_colors`,
`with_opacity` — free functions, unit-tested without `egui::Context`.

**State layer.** `TooltipState` holds the four timing variables (hover,
show, hide, click-latch) plus the underlying `PopoverState`.

**Component layer.** `show` renders the child, runs the trigger state
machine, computes the fade opacity, and delegates positioning to
`MudPopover`.

---

## API reference — `MudTooltip`

| Method | Type | Default | Description |
|--------|------|---------|-------------|
| `new()` | — | — | Default tooltip. |
| `arrow(bool)` | `bool` | `false` | Draw a small triangle on the anchor-facing edge. |
| `color(Color)` | `Color` | `Default` | Tooltip accent colour. |
| `delay_ms(u32)` | `u32` | `0` | Delay before showing, in ms. |
| `duration_ms(u32)` | `u32` | `251` | Fade in / out duration, in ms. |
| `disabled(bool)` | `bool` | `false` | Suppresses every trigger. |
| `inline(bool)` | `bool` | `true` | Stored for parity; no-op in egui. |
| `placement(Placement)` | `Placement` | `Bottom` | Tooltip side. |
| `right_to_left(bool)` | `bool` | `false` | RTL flag for `Start` / `End`. |
| `show_on_click(bool)` | `bool` | `false` | Click-toggle on the trigger. |
| `show_on_focus(bool)` | `bool` | `true` | Show when the trigger has focus. |
| `show_on_hover(bool)` | `bool` | `true` | Show on pointer hover. |
| `text(s)` | `impl Into<String>` | `None` | Tooltip text. |
| `tooltip_content(f)` | `FnMut(&mut Ui)` | `None` | Custom body, overrides `text`. |
| `on_visible_changed(f)` | `FnMut(bool)` | `None` | Fires when visibility changes. |
| `classes(iter)` | `iter of AsRef<str>` | none | Body utility classes. |
| `flavor(StyleFlavor)` | `StyleFlavor` | `Bootstrap` | Utility resolver. |
| `show(ui, state, child)` | `(&mut Ui, &mut TooltipState, FnOnce(&mut Ui) -> Response)` | — | Returns `TooltipResponse`. |

### `TooltipState`

| Field / method | Description |
|----------------|-------------|
| `visible: bool` | Logical visibility. |
| `popover: PopoverState` | Underlying popover state. |
| `hover_started_at` / `show_started_at` / `hide_started_at` | Timers. |
| `click_open: bool` | `ShowOnClick` latch. |
| `new(id_salt)` / `is_visible()` | Constructors + accessor. |
| `show(now)` / `hide(now)` / `toggle(now)` | Programmatic control. |

### `TooltipResponse`

| Field | Type | Description |
|-------|------|-------------|
| `child` | `egui::Response` | The trigger's response. |
| `visible` | `bool` | Logical visibility after this frame. |
| `changed` | `bool` | True when `visible` flipped. |

---

## Examples

### Hover tooltip

```rust
MudTooltip::new()
    .text("Deletes the selected row")
    .show(ui, &mut state, |ui| ui.add(MudButton::new("Delete")));
```

### Focus-triggered (keyboard navigation)

```rust
MudTooltip::new()
    .text("Press Enter to confirm")
    .placement(Placement::Top)
    .show(ui, &mut state, |ui| ui.add(MudButton::new("Confirm")));
```

### Click-toggle (sticky tooltip)

```rust
MudTooltip::new()
    .text("Click again to dismiss")
    .show_on_click(true)
    .show_on_hover(false)
    .show_on_focus(false)
    .show(ui, &mut state, |ui| ui.add(MudIconButton));
```

### Delay before showing

```rust
MudTooltip::new()
    .text("Appears after 500 ms")
    .delay_ms(500)
    .show(ui, &mut state, |ui| ui.add(MudButton::new("Help")));
```

### Arrow indicator

```rust
MudTooltip::new()
    .text("With arrow")
    .arrow(true)
    .placement(Placement::Top)
    .show(ui, &mut state, |ui| ui.add(MudIcon::new(filled::INFO)));
```

### Custom body

```rust
MudTooltip::new()
    .tooltip_content(|ui| {
        ui.add(MudText::new("Custom").bold(true));
        ui.add(MudText::new("Anything you can draw"));
    })
    .show(ui, &mut state, |ui| ui.add(MudButton::new("Details")));
```

### Listen for visibility changes

```rust
MudTooltip::new()
    .on_visible_changed(|v| log::info!("tooltip visible: {v}"))
    .text("With callback")
    .show(ui, &mut state, |ui| ui.add(MudButton::new("Open")));
```

### Programmatic control

```rust
state.show(ui.input(|i| i.time));
state.hide(ui.input(|i| i.time));
state.toggle(ui.input(|i| i.time));
```

---

## Trigger semantics

The tooltip's visible state is the OR of three independent triggers:

| Trigger | When it fires | Parameter |
|---------|--------------|-----------|
| Hover   | Pointer has been over the trigger for `delay_ms` | `show_on_hover` |
| Focus   | Trigger has keyboard focus | `show_on_focus` |
| Click   | Trigger was clicked once (toggle) | `show_on_click` |

Whichever trigger wins, the tooltip stays visible until **all three** are
false. `disabled(true)` suppresses every trigger regardless of the other
flags.

### Hover delay

`delay_ms > 0` starts a timer when the pointer first enters the trigger. The
tooltip opens when the timer elapses. Leaving the trigger before the timer
elapses cancels the pending open. `delay_ms == 0` opens immediately.

The component calls `ctx.request_repaint_after(delay_ms)` so the tooltip
opens at the right moment even if the app is otherwise idle.

### Click latch

`show_on_click(true)` flips a per-tooltip latch on every trigger click.
Combined with `show_on_hover(false)` this gives a "sticky tooltip" UX. The
latch is stored in `TooltipState::click_open` and can be reset by the caller
at any time.

---

## Animation and delays

The tooltip fades in over `duration_ms` when it becomes visible, and fades
out over `duration_ms` when it becomes invisible. Opacity is computed by the
pure `compute_opacity` helper:

| State | Opacity |
|-------|---------|
| Idle | `0.0` |
| Fading in | `(now - show_started) / duration` |
| Fading out | `1 - (now - hide_started) / duration` |
| Fully visible | `1.0` |

While opacity is between `0.0` and `1.0` (or while fading), the component
calls `ctx.request_repaint()` so the fade advances smoothly.

`duration_ms == 0` snaps between `0.0` and `1.0` with no animation.

When the fade-out completes, `TooltipState::reset_animation` clears the
timers and closes the popover so the next `show` starts cleanly.

---

## Performance and memory choices

| Choice | Effect |
|--------|--------|
| Pure `resolve_placement` / `compute_opacity` / `desired_visibility` | No `egui::Context`; unit-tested. |
| `TooltipState` is one `Clone` struct | Zero per-frame allocation. |
| Timers are `Option<f64>` seconds since start | No datetime arithmetic; direct comparison. |
| Fade animation calls `request_repaint` only while animating | No idle frames. |
| Hover delay calls `request_repaint_after(delay)` | Precise open timing without busy-looping. |
| `MudPopover` reuse | Zero duplication of anchor / flip / z-order logic. |
| Fade applies alpha to fill + ink simultaneously | Text and background fade together. |
| Popover paper drawn once per frame | No per-frame allocations for the fill. |

---

## Testing

`cargo test` runs 18 pure-function tests:

| Area | Tests |
|------|-------|
| `resolve_placement` | Top/Bottom pass-through, Start↔End under RTL |
| `placement_to_origins` | every variant |
| `compute_opacity` | idle, zero-duration snap, fade-in linear, fade-out linear |
| `desired_visibility` | disabled blocks all, each trigger independently |
| `with_opacity` | identity, transparent, alpha scaling |
| Defaults | every field matches MudBlazor |
| Builders | every field set |
| `TooltipState` | start hidden, show/hide idempotence, toggle, reset_animation |

Rendering (`show`) needs a live `egui::Context`; the pure layer is the source
of truth.

```bash
cargo test -p mud_egui_components mud_tooltip
cargo test -p mud_egui_components tooltip_state
```

---

## Differences from MudBlazor

- **State is caller-owned (`TooltipState`).** MudBlazor uses the `Visible` /
  `VisibleChanged` binding; the port puts both in the state struct. Same
  caller-owned-state pattern as the rest of the batch.
- **`Inline` is stored but not rendered.** MudBlazor's block/inline toggle
  affects the trigger's flow in the surrounding layout. egui tooltips are
  always overlaid, so there is no equivalent.
- **Arrow drawn inside the paper.** MudBlazor's arrow protrudes outside the
  paper. Pass 1 draws a smaller triangle inside the paper on the
  anchor-facing edge — functionally equivalent (points toward the anchor,
  expands the silhouette) but not pixel-perfect. Deferred to pass 2.
- **`Style` / `Tag` / `UserAttributes` not ported.** No CSS strings or HTML
  tags in egui.
- **Popover dismissals disabled.** `dismiss_on_escape` and
  `dismiss_on_outside_click` are both `false` so the trigger state machine
  is the sole arbiter of visibility. Pressing Escape does not close a
  tooltip unless you wire it into `on_visible_changed` or drive
  `TooltipState::hide` yourself.