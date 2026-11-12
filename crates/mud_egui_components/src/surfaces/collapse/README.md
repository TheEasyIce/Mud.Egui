# MudCollapse for `mud_egui`

A container that animates its height and opacity to hide or reveal its content,
for [egui](https://github.com/emilk/egui). Ported from MudBlazor's
[`MudCollapse`](https://mudblazor.com/api/collapse) (MudBlazor 8.13.0).

```rust
let out = MudCollapse::new()
    .expanded(self.open)
    .id_salt("faq-1")
    .show(ui, |ui| {
        ui.add(MudText::body2("Hidden until expanded."));
    });

if let Some(open) = out.expanded_changed {
    log::info!("expanded changed to {open}");
}
if out.animation_ended {
    self.ready_to_scroll = true;
}
```

Every string inside renders through **`MudText`**, every glyph through
**`MudIcon`**.

---

## Contents

1. [Design principle: API parity, not API invention](#design-principle-api-parity-not-api-invention)
2. [Architecture](#architecture)
3. [API reference](#api-reference)
4. [Events in an immediate-mode UI](#events-in-an-immediate-mode-ui)
5. [Layout and rendering](#layout-and-rendering)
6. [Examples](#examples)
7. [Performance and memory choices](#performance-and-memory-choices)
8. [Testing](#testing)
9. [Differences from MudBlazor](#differences-from-mudblazor)

---

## Design principle: API parity, not API invention

The MudBlazor 8.13.0 parameter surface was verified against DNDocs before
implementation. `MudCollapse` has **exactly five parameters**, and all five are
ported.

### Ported parameters

| MudBlazor | Type | Default | Rust mapping |
|---|---|---|---|
| `ChildContent` | `RenderFragment?` | `null` | closure to `show(ui, \|ui\| ..)` |
| `Expanded` | `bool` | `false` | `expanded(bool)` |
| `ExpandedChanged` | `EventCallback<bool>` | — | `CollapseResponse::expanded_changed` |
| `MaxHeight` | `int?` | `null` | `max_height(f32)` → `Option<f32>` |
| `OnAnimationEnd` | `EventCallback` | — | `CollapseResponse::animation_ended` |
| `Class` | `string?` | `null` | `classes(iter)` — resolved through `flavor` |
| `Tag` | `object?` | `null` | `tag(t)` / `tag_ref::<U>()` |

### Deliberately not ported

| MudBlazor | Reason |
|---|---|
| `Style` | egui has no inline CSS strings. Utility classes are resolved through the Bootstrap or Tailwind flavour; use `classes(..)` and the typed `flavor(..)` for the same purpose. |
| `UserAttributes` | There is no HTML tag to splat attributes onto. |

`MaxHeight` becomes `Option<f32>` because CSS `int?` (pixels) and egui points
are the same unit at scale 1; `None` means "no cap", which is MudBlazor's
`null`.

---

## Architecture

```
mud_collapse.rs
├── constants          DEFAULT_DURATION, HEIGHT_EPSILON, MAX_NATURAL_HEIGHT,
│                      REM_SIZE_PT
├── pure helpers       sane_duration, sane_length, sane_max_height,
│                      is_animating, detect_animation_end,
│                      detect_expanded_change, visible_height,
│                      needs_height_update, placed_width
├── compute_progress   animate_bool_with_time, or a direct 0/1 for duration 0
├── paint_frame_bg     shadow + fill + stroke over the reserved rect
├── MudCollapse        builder
│   └── show           progress → events → one of three paths
├── CollapseResponse   response + inner + progress + event fields (Deref<Response>)
├── impl Widget        ui.add(MudCollapse::new())  (empty content)
└── mod tests          16 pure-function tests
```

**Pure layer.** Every decision that can be made from numbers — sanitizing the
duration and the cap, whether we are animating, whether the animation just
ended, whether `expanded` changed, how tall the visible strip is, whether the
cached height is stale — is a free function over `f32` and `bool`. None of them
touch `egui::Context`, so they are unit-tested without a window.

**State layer.** Three values live in `egui::Memory`, keyed by
`ui.make_persistent_id(id_salt)`:

| Key | Type | Purpose |
|---|---|---|
| `"height"` | `f32` | Last measured natural height |
| `"prev_expanded"` | `bool` | Previous frame's `expanded` |
| `"animating"` | `bool` | Whether we were animating last frame |

`id_salt` disambiguates collapses that share a parent, the same way
`MudToggleGroup` salts its ripple keys.

**Component layer.** `show` reads progress, computes the two events, picks one
of three paths, draws the content **exactly once**, and reserves the correct
height.

### The three paths

| Path | Condition | Behaviour |
|---|---|---|
| Collapsed | `!expanded && progress <= 0` | Zero-sized response; the closure is **not called**; the height cache is dropped. |
| Settled expanded | `expanded && progress >= 1` | Content drawn plainly, measured, exactly its height (capped by `max_height`) reserved. |
| Animating | otherwise | Content drawn clipped to `cached × progress`, opacity `= progress`, `cached × progress` reserved. |

---

## API reference

### `MudCollapse`

| Method | Type | Default | Mirrors | Description |
|---|---|---|---|---|
| `new()` | `Self` | — | — | Collapsed, 0.2 s, no cap. |
| `expanded(b)` | `bool` | `false` | `Expanded` | Whether the content is shown. |
| `duration(s)` | `f32` | `0.2` | *(CSS transition)* | Animation seconds; `0.0` is instant. |
| `max_height(px)` | `f32` | `None` | `MaxHeight` | Visible height cap in points. |
| `id_salt(s)` | `impl Hash + Debug` | `None` | — | Stable identity for two collapses in one parent. |
| `flavor(f)` | `StyleFlavor` | `Bootstrap` | — | Utility-class resolver. |
| `classes(iter)` | `iter of AsRef<str>` | none | `Class` | Utility classes resolved into the frame. |
| `tag(t)` / `tag_ref::<U>()` | `T: Any + Send + Sync` | `None` | `Tag` | Arbitrary user data. |
| `show(ui, \|ui\| ..)` | `&mut Ui, FnOnce(&mut Ui) -> R` | — | `ChildContent` | Returns `CollapseResponse<R>`. |

### `CollapseResponse<R>`

| Field / method | Type | Description |
|---|---|---|
| `response` | `Response` | The reserved area. `CollapseResponse` derefs to it. |
| `inner` | `Option<R>` | The closure's value; `None` while fully collapsed. |
| `progress` | `f32` | `0.0` collapsed … `1.0` expanded. |
| `expanded_changed` | `Option<bool>` | `Some(new)` on the frame `expanded` changed; `None` on the first frame and on steady frames. |
| `animation_ended` | `bool` | `true` for exactly one frame when progress reaches its target. |
| `is_fully_expanded()` | `bool` | `progress >= 1`. |
| `is_fully_collapsed()` | `bool` | `progress <= 0`. |

### Constants

| Name | Value |
|---|---|
| `DEFAULT_DURATION` | `0.2` s |
| `HEIGHT_EPSILON` | `0.1` pt |
| `MAX_NATURAL_HEIGHT` | `10_000.0` pt |

---

## Events in an immediate-mode UI

egui has no event-callback objects, so the two MudBlazor callbacks are reported
as **data about this frame**:

```rust
let out = MudCollapse::new().expanded(open).id_salt("faq-1").show(ui, |ui| {
    ui.label("…");
});

if let Some(now_open) = out.expanded_changed { /* ExpandedChanged */ }
if out.animation_ended                     { /* OnAnimationEnd  */ }
```

- `expanded_changed` is `Some(new)` only when the `expanded` argument differs
  from the previous frame's. It is **never** set on the first frame (no change
  has happened yet).
- `animation_ended` is `true` for **exactly one frame**, when progress reaches
  the target (`1.0` when expanding, `0.0` when collapsing). With
  `duration(0.0)` no animation is ever in progress, so it never fires.
- If the collapse is not shown for a frame (an ancestor is hidden), the stored
  state is not updated; events resume correctly when it is shown again.

---

## Layout and rendering

| Aspect | Behaviour |
|---|---|
| Animation | Linear progress from egui's `animate_bool_with_time`; height `= natural_height × progress`, opacity `= progress`. |
| Natural height | Measured from the child `Ui` (frame margins included) and cached; remeasured only when it changes by more than `HEIGHT_EPSILON`. |
| `max_height` | The reserved and visible height never exceed it; the content is clipped to it. |
| Width | The available width. |
| Frame | Resolved from utility classes (Bootstrap or Tailwind) and wrapped around the content: shadow, fill and stroke are painted manually over the reserved rect so the height is exact. |
| Interaction | Static container (`Sense::hover()`); hover/focus/disabled styling belongs to the content. |

### First expansion, one frame late

The natural height is learned from the previous frame. On the **first** frame
of the **first** expansion there is no cached height, so the content is drawn
invisibly (clip height 0, opacity 0), measured, and cached; the animation then
starts from a known height on the next frame. The delay is one frame (≈16 ms),
imperceptible, and after the first expansion the collapse animates correctly
forever.

### Why the frame is painted manually

`egui::Frame::show` reserves the rect its content used, which is the *natural*
height — wrong for the animating path, where only `natural × progress` should
be reserved, and wrong for `max_height`, where the content must be clipped. So
`MudCollapse` allocates exactly the visible rect first, then paints the frame's
shadow, fill and stroke into it, then clips the content child to the same rect.

---

## Examples

### Basic

```rust
ui.checkbox(&mut self.open, "Show details");
MudCollapse::new()
    .expanded(self.open)
    .id_salt("details")
    .show(ui, |ui| {
        ui.add(MudText::body2("Hidden until expanded."));
    });
```

### Reacting to events

```rust
let out = MudCollapse::new()
    .expanded(self.open)
    .id_salt("details")
    .show(ui, |ui| {
        ui.add(MudText::body2("Details"));
    });

if let Some(open) = out.expanded_changed {
    log::info!("expanded changed to {open}");
}
if out.animation_ended {
    self.scroll_into_view = true;
}
```

### Limited height (tall content stays clipped)

```rust
MudCollapse::new()
    .expanded(true)
    .max_height(120.0)
    .id_salt("log")
    .show(ui, |ui| {
        for line in &self.lines {
            ui.add(MudText::monospace(line.clone()));
        }
    });
```

### Accordion: header + collapse

```rust
for (i, item) in self.items.iter().enumerate() {
    if ui.button(&item.title).clicked() {
        self.open = if self.open == Some(i) { None } else { Some(i) };
    }
    MudCollapse::new()
        .expanded(self.open == Some(i))
        .id_salt(("faq", i))
        .classes(["p-2"])
        .show(ui, |ui| {
            ui.add(MudText::body2(&item.body));
        });
}
```

### Instant switch

```rust
MudCollapse::new()
    .expanded(self.open)
    .duration(0.0)     // no animation, no OnAnimationEnd
    .id_salt("instant")
    .show(ui, |ui| { ui.label("…"); });
```

### Tailwind surface

```rust
MudCollapse::new()
    .expanded(self.open)
    .flavor(StyleFlavor::Tailwind)
    .classes(["bg-surface", "rounded-lg", "p-4", "hover:bg-surface/90"])
    .id_salt("tw")
    .show(ui, |ui| { ui.label("…"); });
```

The `hover:` variant is fed from the previous frame's state, exactly as
`MudPaper` documents.

### Tag round-trip

```rust
let c = MudCollapse::new().duration(0.0).tag("advanced-options");
assert_eq!(c.tag_ref::<&str>(), Some(&"advanced-options"));
c.expanded(true).id_salt("adv").show(ui, |ui| { ui.label("…"); });
```

---

## Performance and memory choices

| Choice | Effect |
|---|---|
| Collapsed path skips the closure | A hidden panel costs one animation lookup and a zero-size allocation. |
| One measurement per frame | The closure is `FnOnce` and is called exactly once; heights are cached between frames. |
| Noise-tolerant cache updates | Heights are only rewritten (and a repaint requested) when they change by more than `HEIGHT_EPSILON`, so sub-pixel float noise cannot cause a repaint loop. |
| Three scal