# `MudContainer` for `mud_egui`

A responsive, centered, width-limiting container for [egui](https://github.com/emilk/egui),
ported from [MudBlazor](https://mudblazor.com)'s `MudContainer`.

It caps its content at a Material Design breakpoint, centers that content
horizontally, applies responsive side gutters, and resolves Bootstrap- or
Tailwind-style utility classes into an `egui::Frame`.

> **Status note.** This file set was written without access to a Rust toolchain
> or to the private `mud_egui_*` crates, so neither the source nor its test
> suite has been compiled or executed by the author. Everything that touches
> those crates (`ComponentBase`, `BsStyle`, `TwStyle`, `TwState`, `MudTheme`,
> `MaxWidth`) uses only the signatures already present in the original file.
> Run `cargo test` before shipping. See [Known limitations](#known-limitations).

---

## Table of contents

1. [What MudBlazor's `MudContainer` actually has](#what-mudblazors-mudcontainer-actually-has)
2. [Quick start](#quick-start)
3. [Architecture](#architecture)
4. [API reference](#api-reference)
5. [Breakpoints and gutters](#breakpoints-and-gutters)
6. [Styling pipeline and visual behaviour](#styling-pipeline-and-visual-behaviour)
7. [Events, state and thread-safe handles](#events-state-and-thread-safe-handles)
8. [Real-world examples](#real-world-examples)
9. [Performance choices and design rationale](#performance-choices-and-design-rationale)
10. [Testing](#testing)
11. [Changes from the previous version](#changes-from-the-previous-version)
12. [Known limitations](#known-limitations)

---

## What MudBlazor's `MudContainer` actually has

Before extending the component, the published MudBlazor API reference was
checked ([mudblazor.com/api/MudContainer](https://mudblazor.com/api/MudContainer)).
`MudContainer` is a layout primitive with exactly these parameters:

| MudBlazor parameter | Type              | Default          | Meaning                                                                 |
|---------------------|-------------------|------------------|-------------------------------------------------------------------------|
| `MaxWidth`          | `MaxWidth`        | `MaxWidth.Large` | Max width; grows with screen size; `false` disables it                  |
| `Fixed`             | `bool`            | `false`          | Set max width to the min width of the current breakpoint                |
| `Gutters`           | `bool`            | `true`           | Side padding (added in MudBlazor v7, replacing `DisableGutters`)        |
| `ChildContent`      | `RenderFragment`  | `null`           | The content                                                             |
| `Class`             | `string`          | `null`           | User class names, space separated                                       |
| `Style`             | `string`          | `null`           | User styles applied on top of the component's own                       |
| `Tag`               | `object`          | `null`           | Arbitrary user data                                                     |
| `UserAttributes`    | `Dictionary`      | empty            | Extra attributes splatted onto the HTML tag (ARIA, custom `id`, ...)    |

It has **no** `Value` / `ValueChanged` / `ValueExpression`, `Variant`, `Color`,
`Size`, `ReadOnly`, adornment slots, or input events (`OnClick`, `OnBlur`,
`OnKeyDown`, `OnChange`). Those belong to input and button components.
Because the goal was to avoid adding what the real component does not have,
this port does **not** invent them either.

Everything beyond the table above is explicitly an **extension** and is marked
as such in the [API reference](#api-reference).

---

## Quick start

```rust
use mud_egui_core::enums::max_width::MaxWidth;

// Keep the container in your app state so caches and animations persist.
struct Page {
    container: MudContainer,
}

impl Page {
    fn new() -> Self {
        Self {
            container: MudContainer::new()
                .max_width(MaxWidth::Md)
                .classes(["bg-surface", "p-4", "rounded-lg", "elevation-2"]),
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        self.container.show(ui, |ui| {
            ui.heading("Centered, at most 960 px wide");
            ui.label("Gutters are 24 px on wide screens, 16 px below 600 px.");
        });
    }
}
```

`ui.add(MudContainer::new())` also works through the `Widget` impl, but it adds
an *empty* container and re-resolves styles each frame. Use `show` for content.

---

## Architecture

```
mud_container.rs
├── Pure layout layer (no egui state; fully unit-testable)
│   ├── MudContainerMetrics   breakpoint table, gutters, snap, resolve, centering
│   ├── LayoutSpec            Copy descriptor: max_width / fluid / fixed / gutters
│   └── HorizontalLayout      Copy result: gutter, content, outer, effective_max, breakpoint
│
├── Style layer
│   ├── StyleFlavor           Bootstrap | Tailwind
│   ├── StyleOverrides        typed "Style" parameter (fill, stroke, padding, outer margin)
│   └── fade_frame / focus ring / gutter composition (private helpers)
│
├── State & events layer
│   ├── InteractionState      previous-frame hover / active / focus / dark / disabled
│   ├── Edge                  Steady | Rising | Falling
│   ├── EventPlan             pure "which callbacks fire" decision
│   └── ContainerHandle       lock-free, Send + Sync live-state window
│       └── ContainerSnapshot
│
└── MudContainer              builder + show / show_with_layout glue
    └── tests                 pure math, planning, handle, builders, real egui passes
```

### Frame pipeline (`show_with_layout`)

1. Read `available_rect_before_wrap` and compute a `HorizontalLayout` (pure math).
2. Compare the viewport breakpoint with the previous frame and fire
   `on_breakpoint_change` if it changed (or this is the first frame).
3. Create a child `Ui` whose rect is horizontally offset to center the content.
   (Offsetting X is deliberate: `ui.add_space` inserts *vertical* space in a
   top-down parent.)
4. Choose the container's own `Sense` (see [Click passthrough](#click-passthrough)).
5. Resolve the frame: **classes → `StyleOverrides` → disabled fade → focus ring →
   responsive gutters.**
6. Run the user's content with the content column pinned to `content_width`
   (skipped when unbounded), disabled via `Ui::disable` if needed.
7. Reserve a full-width row in the parent using a **sanitized** rect.
8. Publish an ARIA label to egui's accessibility tree, attach an optional
   tooltip, plan and fire callbacks from state edges, record interaction state
   for the next frame, and publish to any live `ContainerHandle`.

### Click passthrough

egui hit-tests widgets in registration order and a later registration wins.
The container reserves its own rect *after* its children are drawn, so an
interactive `Sense` on that rect would sit on top of everything inside it and
steal clicks and keyboard focus from children such as text fields.

| Registered callbacks                         | Container `Sense`  | Children receive input? |
|----------------------------------------------|--------------------|-------------------------|
| none, or only `on_hover` / `on_unhover` / `tooltip` | `Sense::hover()`   | yes                     |
| any of `on_click`, `on_focus`, `on_blur`     | `Sense::click()`   | **no**: container is the click target |
| `disabled(true)`                             | `Sense::hover()`   | children are disabled   |

The container never senses **drag**, so it cannot interfere with drag-to-scroll
in an enclosing `ScrollArea`. If you need a clickable region *and* typable
children, nest two containers.

---

## API reference

### Builder methods and public fields

"MudBlazor" column: **✔** = exists on the real `MudContainer`; **ext** = Rust
extension; **n/a** = no equivalent concept.

| Member (builder = field)                | Type                                   | Default        | MudBlazor | Description |
|-----------------------------------------|----------------------------------------|----------------|-----------|-------------|
| `max_width(MaxWidth)`                   | `MaxWidth`                             | `Lg`           | ✔ `MaxWidth` | Breakpoint ceiling. Ignored when `fluid` or `fixed` is on. |
| `fixed(bool)`                           | `bool`                                 | `false`        | ✔ `Fixed`    | Snap ceiling to the breakpoint of the offered width. |
| `gutters(bool)`                         | `bool`                                 | `true`         | ✔ `Gutters`  | 16 px below 600 px offered width, 24 px otherwise. |
| `show(ui, \|ui\| ...)`                  | closure                                | n/a            | ✔ `ChildContent` | Draws the container and its content. |
| `class(&str)` / `classes(iter)`         | `ComponentBase` classes                | empty          | ✔ `Class`   | Space-separated utility classes; duplicates ignored. |
| `style(StyleOverrides)` / `fill` / `stroke` / `padding` | `StyleOverrides`       | none           | ✔ `Style` (typed) | Overrides applied after class resolution. |
| `tag(value)` / `tag_ref::<T>()`         | `Option<Box<dyn Any + Send + Sync>>`   | `None`         | ✔ `Tag`      | Arbitrary user data. |
| `attr(k, v)` / `attrs(iter)` / `user_attribute(k)` | `BTreeMap<String, String>`  | empty          | ✔ `UserAttributes` | Extra attributes; `"aria-label"` doubles as label fallback. |
| `fluid(bool)`                           | `bool`                                 | `false`        | ext (≈ `MaxWidth.False`) | Fill the full offered width. Wins over `fixed`. |
| `disabled(bool)`                        | `bool`                                 | `false`        | ext       | Disables children, fades frame, silences callbacks. |
| `elevation(u8)`                         | `u8` (0–24, clamped)                   | `0`            | ext       | Forwarded to `ComponentBase::set_elevation`. |
| `flavor(StyleFlavor)`                   | `StyleFlavor`                          | `Bootstrap`    | ext       | Bootstrap or Tailwind class resolution. |
| `id(impl Into<Id>)`                     | `Option<Id>`                           | `None`         | ext (≈ `UserAttributes["id"]`) | Salts the child `Ui` id. |
| `aria_label(impl Into<String>)`         | `Option<String>`                       | `None`         | ext (≈ `UserAttributes["aria-label"]`) | Published to egui's accessibility tree (not a visible tooltip). |
| `tooltip(impl Into<String>)`            | `Option<String>`                       | `None`         | ext       | Visible hover tooltip. |
| `focus_ring(bool)`                      | `bool`                                 | `true`         | ext       | Animated focus ring on interactive containers. |
| `memoize_style(bool)`                   | `bool`                                 | `false`        | ext       | Opt-in cache of the class-resolved `Frame`. |
| `on_hover(f)`                           | `FnMut() + Send + Sync`                | `None`         | ext       | Pointer entered (rising edge). |
| `on_unhover(f)`                         | `FnMut() + Send + Sync`                | `None`         | ext       | Pointer left (falling edge). |
| `on_click(f)`                           | `FnMut() + Send + Sync`                | `None`         | ext       | Primary click. Upgrades `Sense` to click. |
| `on_focus(f)`                           | `FnMut() + Send + Sync`                | `None`         | ext       | Gained keyboard focus (rising edge). Upgrades `Sense`. |
| `on_blur(f)`                            | `FnMut() + Send + Sync`                | `None`         | ext       | Lost keyboard focus (falling edge). Upgrades `Sense`. |
| `on_breakpoint_change(f)`               | `FnMut(MaxWidth) + Send + Sync`        | `None`         | ext       | Fires on first frame and whenever the viewport bucket changes. |
| `base` (public field)                   | `ComponentBase`                        | empty          | ✔ (`Class` storage) | Direct access for extra mutations (`add_class`, `set_elevation`, ...). |
| `interaction` (public field)            | `InteractionState`                     | all `false`    | ext       | Previous-frame flags used by Tailwind variants. |
| `bs_style` / `tw_style` (public fields) | `BsStyle` / `TwStyle`                  | default        | ext       | Resolver caches. |

### Constructors and introspection

| Method | Returns | Description |
|--------|---------|-------------|
| `MudContainer::new()` | `Self` | Defaults: `Lg`, gutters on, Bootstrap. |
| `for_breakpoint(MaxWidth)` | `Self` | Container capped at a breakpoint. |
| `full_width()` | `Self` | `MaxWidth::False`; gutters stay on (like MudBlazor). |
| `from_spec(LayoutSpec)` | `Self` | Build from a `Copy` layout descriptor. |
| `layout_spec()` | `LayoutSpec` | Current layout-affecting settings. |
| `preview_layout(offered)` | `HorizontalLayout` | What `show` *would* compute, with no egui state. |
| `last_layout()` | `Option<HorizontalLayout>` | Layout from the most recent frame. |
| `handle()` | `ContainerHandle` | Lazily-created thread-safe live-state handle. |
| `effective_aria_label()` | `Option<&str>` | Explicit label, else `"aria-label"` attribute. |
| `invalidate_style_cache()` | `()` | Drop the memoized frame and bump the epoch. |
| `reset_interaction()` | `()` | Clear hover/active/focus flags and ring animation. |
| `show(ui, add_contents)` | `InnerResponse<R>` | Draw; closure receives `&mut Ui`. |
| `show_with_layout(ui, add_contents)` | `InnerResponse<R>` | Draw; closure also receives `&HorizontalLayout`. |

### `HorizontalLayout`

| Field / method | Type | Description |
|----------------|------|-------------|
| `gutter` | `f32` | Side gutter per edge (px). |
| `content_width` | `f32` | Width for child content (infinite for fluid in an unbounded parent). |
| `outer_width` | `f32` | Content plus both gutters; never exceeds a finite offer. |
| `effective_max` | `MaxWidth` | Ceiling after fluid/fixed resolution. |
| `breakpoint` | `MaxWidth` | Bucket of the *offered* width (`Xs` for non-finite offers). |
| `gutters_total()` | `f32` | `2 × gutter`. |
| `is_bounded()` | `bool` | `outer_width.is_finite()`. |
| `offset_in(offered)` | `f32` | Centering offset. |

### `StyleOverrides`

| Field | Type | Replaces |
|-------|------|----------|
| `fill` | `Option<Color32>` | `Frame::fill` |
| `stroke` | `Option<Stroke>` | `Frame::stroke` |
| `padding` | `Option<Margin>` | `Frame::inner_margin` (gutters are added afterward) |
| `outer_margin` | `Option<Margin>` | `Frame::outer_margin` |

### Visual variants

`MudContainer` has no `Color` / `Variant` / `Size` parameters, exactly like
MudBlazor. Its visual variations come from:

| Axis | Values | Source |
|------|--------|--------|
| Width ceiling | `Xs 444`, `Sm 600`, `Md 960`, `Lg 1280`, `Xl 1920`, `Xxl 2560`, `False ∞` | `MaxWidth` |
| Fixed mode | off / snap to viewport breakpoint | `fixed` |
| Gutters | none / 16 px / 24 px (automatic) | `gutters` |
| Surface (colour, radius, elevation) | any palette colour / utility class | `classes`, `elevation`, `StyleOverrides` |
| Style system | Bootstrap / Tailwind (with `hover:`, `active:`, `dark:`) | `flavor` |
| State | idle / hover (Tailwind) / focused ring / disabled fade | interaction + `disabled` |

### Binding slots

| MudBlazor concept | Present on `MudContainer`? | Rust equivalent |
|-------------------|----------------------------|-----------------|
| `Value` / `ValueChanged` / `ValueExpression` | No | none (a container has no value) |
| `ChildContent` | Yes | the closure passed to `show` |
| `AdornmentStart` / `AdornmentEnd` | No | none |

### Free types

| Type | Purpose |
|------|---------|
| `MudContainerMetrics` | Namespace of pure layout functions: `to_pixels`, `label`, `snap_breakpoint`, `gutter_for`, `resolve`, `centering_offset`, plus the pixel constants. |
| `LayoutSpec` | `Copy` struct of the four layout settings with `resolve(offered)`. |
| `HorizontalLayout` | `Copy` result of a layout resolution. |
| `StyleFlavor` | `Bootstrap` (default) or `Tailwind`. |
| `StyleOverrides` | Typed equivalent of MudBlazor's `Style`. |
| `InteractionState` | Previous-frame flags; `to_tw_state()` maps to `TwState` and respects `disabled`. |
| `Edge` | `Steady` / `Rising` / `Falling`; `Edge::detect(prev, now)`. |
| `EventPlan` | Which callbacks fire this frame; `EventPlan::compute(...)`. |
| `ContainerHandle` | Cloneable, `Send + Sync` window into live state. |
| `ContainerSnapshot` | Plain-data copy of a handle's state. |

---

## Breakpoints and gutters

```
offered width →   0 ─────── 600 ─────── 960 ─────── 1280 ────── 1920 ────── 2560 ──▶
viewport bucket →     Xs         Sm          Md          Lg          Xl         Xxl
ceiling (Max…)  →    444        600         960        1280        1920        2560
gutter          →     16 px        24 px (from 600 px upward)
```

Resolution rules (all enforced by `MudContainerMetrics::resolve`):

* `NaN` and negative offers are treated as `0`.
* An infinite offer yields the breakpoint cap, or an unbounded width for
  `fluid` / `MaxWidth::False`.
* `fixed` replaces the ceiling with the viewport bucket's ceiling.
  `fluid` beats `fixed`.
* Gutters never exceed half of a finite offer, so tiny viewports cannot overflow.
* For finite offers, `outer_width <= offered` always holds.
* `Xs` is **444**, not 0, matching MudBlazor's CSS.

Worked example, `MaxWidth::Lg`, gutters on:

| Offered | Gutter | Content | Outer | Notes |
|--------:|-------:|--------:|------:|-------|
| 500     | 16     | 468     | 500   | small gutter |
| 800     | 24     | 752     | 800   | below the cap |
| 2000    | 24     | 1232    | 1280  | capped, centered with a 360 px offset |
| 10      | 5      | 0       | 10    | gutter clamped to half the offer |

---

## Styling pipeline and visual behaviour

Order of operations for the frame:

```
utility classes (Bootstrap | Tailwind)
   → StyleOverrides (fill / stroke / padding / outer margin)
   → disabled fade (fill and stroke alpha × 0.6)
   → focus ring (interactive containers only)
   → responsive gutters added to inner_margin left/right
```

* **Padding composes with gutters.** A `p-4` class plus a 24 px gutter yields
  `p-4 + 24` on left and right; top and bottom are untouched.
* **Disabled.** Children are disabled through egui's own disabled styling
  (non-interactive and faded), the frame fill/stroke are faded, the container
  senses hover only, and no callbacks fire.
* **Focus ring.** For containers with a click-class callback, a 2 pt stroke in
  the palette's primary colour (`"MudPalettePrimary"`, falling back to egui's
  selection stroke colour) fades in and out over 120 ms. The stroke width is
  reserved even when the ring is idle, so focusing never shifts layout.
  Disable with `.focus_ring(false)`.
* **Tailwind variants.** Hover/active/dark flags are the *previous* frame's
  (immediate mode cannot know this frame's hover before drawing), which is
  imperceptible at normal frame rates.
* **Shadows, radii and ripple** are produced by your utility classes through
  `ComponentBase` and the style resolvers (for example `elevation-N`,
  `rounded-lg`). `MudContainer` itself adds none, just like MudBlazor, whose
  container has no inherent elevation or border radius.

---

## Events, state and thread-safe handles

### Edge-triggered callbacks

Hover and focus callbacks are derived from level changes between frames:

```rust
let plan = EventPlan::compute(
    prev_hovered, prev_focused,   // last frame
    hovered, focused, clicked,    // this frame
    disabled,
);
```

Because planning is a pure function, the firing rules are covered by tests
without egui. A disabled container always returns an empty plan.

### Breakpoint callbacks

```rust
let mut page = MudContainer::new().on_breakpoint_change(|bp| {
    println!("breakpoint is now {}", MudContainerMetrics::label(bp));
});
```

Fires on the first frame and on every bucket crossing. Unlike `effective_max`
(constant for a capped container), `breakpoint` tracks the *viewport*, so it is
the right signal for adaptive layouts.

### `ContainerHandle`: reading live state from anywhere

```rust
let mut page = MudContainer::new().max_width(MaxWidth::Lg);
let handle = page.handle();                 // cheap; clone it freely

// Another component, another thread, a test:
let snap = handle.snapshot();
if snap.outer_width < 700.0 || snap.viewport_breakpoint == MaxWidth::Xs {
    /* collapse the sidebar */
}
```

* The container performs the atomic stores **only if** a handle exists.
* `f32` values are stored as `AtomicU32` bit patterns; booleans and enums as
  `AtomicBool` / `AtomicU8`; the frame counter as `AtomicU64`.
* The frame counter uses `Release` (writer) / `Acquire` (reader); all other
  stores are `Relaxed`. A snapshot is field-wise atomic and is **eventually
  consistent** if a different thread is mid-frame; it is exact on the render
  thread.

---

## Real-world examples

### 1. Page shell with a responsive sidebar

```rust
struct App {
    page: MudContainer,
    sidebar_open: bool,
}

impl App {
    fn new() -> Self {
        Self {
            page: MudContainer::new().max_width(MaxWidth::Xl),
            sidebar_open: true,
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        let sidebar_open = &mut self.sidebar_open;
        self.page.show_with_layout(ui, |ui, layout| {
            // Collapse the sidebar on narrow content columns.
            *sidebar_open = layout.content_width >= 720.0;
            ui.horizontal(|ui| {
                if *sidebar_open {
                    ui.vertical(|ui| { ui.set_width(220.0); ui.label("Navigation"); });
                }
                ui.vertical(|ui| { ui.label("Main content"); });
            });
        });
    }
}
```

### 2. Fixed-breakpoint design

```rust
// Content width jumps between 600 / 960 / 1280 ... instead of flowing.
let mut c = MudContainer::new().fixed(true);
```

### 3. Full-bleed hero with a constrained inner column

```rust
let mut hero = MudContainer::full_width().gutters(false).classes(["bg-primary"]);
let mut inner = MudContainer::for_breakpoint(MaxWidth::Md);

hero.show(ui, |ui| {
    inner.show(ui, |ui| {
        ui.heading("Welcome");
    });
});
```

### 4. Tailwind variants and typed style overrides

```rust
let mut card = MudContainer::new()
    .flavor(StyleFlavor::Tailwind)
    .classes(["bg-surface", "hover:bg-primary/10", "rounded-lg", "p-4"])
    .style(StyleOverrides {
        stroke: Some(egui::Stroke::new(1.0, egui::Color32::from_gray(90))),
        ..Default::default()
    });
```

### 5. A clickable, keyboard-focusable card

```rust
let mut card = MudContainer::new()
    .max_width(MaxWidth::Sm)
    .classes(["bg-surface", "p-4", "rounded-lg"])
    .aria_label("Open settings")
    .tooltip("Settings")
    .on_click(|| println!("open settings"))
    .on_focus(|| println!("focused"))
    .on_blur(|| println!("blurred"));
// Children do not receive clicks here; the card itself is the target.
```

### 6. Metadata with `Tag` and `UserAttributes`

```rust
#[derive(Debug, PartialEq)]
struct PageMeta { route: &'static str }

let c = MudContainer::new()
    .tag(PageMeta { route: "/settings" })
    .attr("data-testid", "settings-page")
    .attr("aria-label", "Settings page");

assert_eq!(c.tag_ref::<PageMeta>(), Some(&PageMeta { route: "/settings" }));
assert_eq!(c.effective_aria_label(), Some("Settings page"));
```

### 7. Responsive-design tooling without egui

```rust
for width in [400.0, 800.0, 1500.0, 2600.0] {
    let l = MudContainer::new().max_width(MaxWidth::Lg).preview_layout(width);
    println!("{width:>6} → outer {:>6.0}  bucket {}",
        l.outer_width, MudContainerMetrics::label(l.breakpoint));
}
```

### 8. Cached style resolution for a hot, static container

```rust
let mut c = MudContainer::new()
    .classes(["bg-surface", "p-4"])
    .memoize_style(true);

// After you mutate the MudTheme palette outside egui's visuals:
c.invalidate_style_cache();
```

---

## Performance choices and design rationale

All claims below describe *structure* of the implementation; no benchmark
numbers are asserted, because none were measured.

| Choice | Rationale |
|--------|-----------|
| **Pure, `Copy` layout layer** (`MudContainerMetrics`, `LayoutSpec`, `HorizontalLayout`) | Layout is arithmetic on a few `f32`s and two one-byte enums: no allocation, no `egui` state, trivially testable and reusable (`preview_layout`). |
| **Narrow `Sense`** | The container only asks for `click` when a click-class callback exists, and never asks for drag. This avoids stealing input from children and from scroll-area drag. |
| **Lazy `ContainerHandle`** | The `Arc` is allocated on first `handle()` call; without a handle `show` does no atomic work and no extra allocation. Readers never take a lock. |
| **Edge-planned events** | `EventPlan` separates *deciding* from *firing*; both are cheap and decisions are pure. Callbacks are `Option<Box<dyn FnMut>>`, so unused events cost one `is_some` check. |
| **Opt-in frame memoization** | Resolving classes requires fetching the theme and building a `Frame`. With `memoize_style(true)` a 64-bit FNV-1a key (flavour, classes, elevation, ripple flag, dark mode, a small visuals fingerprint, Tailwind variant bits, epoch) is compared and the cached `Frame` is cloned on a hit. Hashing the class list is O(total class characters), so the win is largest when theme resolution dominates. It is **opt-in** because `MudTheme` values stored outside egui's visuals cannot be observed; call `invalidate_style_cache()` after such changes. |
| **FNV-1a for keys** | Deterministic across runs and platforms (no random `SipHash` seed), allocation-free, adequate for non-adversarial cache keys. |
| **Sanitized reserved rect** | A single `NaN`/`inf` rect would poison an enclosing `ScrollArea`'s measured size; `used_rect` and `finite_or` make that impossible. |
| **`floor` for gutter margins** | `egui::Margin` is integral; flooring guarantees the margins never exceed the float gutter budget on tiny viewports. |
| **Self-contained focus animation** | The fade state lives in the container (`f32`, stepped by `stable_dt`), so there is no `Id` collision when several unnamed containers share a `Ui`. A repaint is requested only while the animation is in flight. |
| **Constant reserved ring stroke** | Changing a frame's stroke width changes its total size in egui; reserving the width up front means focusing never causes layout shift. |
| **`ui.disable()` for disabled** | Delegates to egui's built-in disabled behaviour instead of re-implementing it. |

Memory layout: `HorizontalLayout` is four `f32` + two one-byte enums,
`InteractionState` is five `bool`s, `StyleOverrides` is four `Option`s of
small `Copy` types, and `ContainerSnapshot` is plain data; all are `Copy`. The
container itself owns heap data only for what you opt into (callbacks, tag,
attributes, label, handle, cached frame).

---

## Testing

The `mod tests` block at the bottom of `mud_container.rs` contains:

* **Pure metrics**: breakpoint table, monotonicity, snap thresholds, gutters,
  fluid/fixed/infinite/NaN/negative/tiny offers, and a brute-force sweep of
  `resolve` invariants across offers × gutters × fluid × fixed × all breakpoints.
* **Helpers**: sanitizers, margin arithmetic, `used_rect`, `approach`,
  `fade_frame`, FNV-1a, visuals fingerprint, `MaxWidth` ↔ `u8` round trip.
* **Events**: `Edge`, `EventPlan` (edges, clicks, disabled), and `fire`
  invoking exactly the planned callbacks once.
* **Builders and state**: defaults, presets, `LayoutSpec` round trips,
  `Tag`, `UserAttributes`, ARIA fallback, class parsing/dedup, elevation clamp,
  `StyleOverrides`, interaction state, `reset_interaction`.
* **Cache keys**: stability, sensitivity to classes/elevation/dark/visuals/
  flavour/epoch, Tailwind hover sensitivity vs. Bootstrap insensitivity.
* **Handle**: zeroed start, publish/snapshot round trip, shared clones, lazy
  creation, `Send + Sync`, cross-thread read.
* **Real egui passes** (via `egui::Context::run` and a `CentralPanel`):
  closure value, resolved layout, full-width reservation, fixed and fluid
  modes, handle frame counting, breakpoint callback sequence, disabled and
  enabled children, memoization reuse/invalidate, the `Widget` impl.

Run with:

```bash
cargo test -p <crate-containing-mud_container>
```

Pointer- and keyboard-driven callbacks are tested through `EventPlan` and
`fire` rather than synthetic egui input events.

---

## Changes from the previous version

| Area | Before | Now |
|------|--------|-----|
| MudBlazor parity | `Class`, `Gutters`, `MaxWidth`, `Fixed`, `ChildContent` | Adds `Style` (typed), `Tag`, `UserAttributes` |
| `aria_label` | Shown as a **hover tooltip** | Published to egui's accessibility tree; visible text moved to a separate `tooltip` |
| `disabled` | Silenced callbacks only | Also disables children and fades the frame |
| Container `Sense` | `click_and_drag` for any callback | `hover` for hover/tooltip, `click` only for click/focus/blur; never drag |
| Blur detection | `lost_focus()` or inferred | Pure focus-level edge detection |
| Layout result | 4 fields | Adds `breakpoint`; `show_with_layout` hands it to the closure |
| Live state | none | `ContainerHandle` / `ContainerSnapshot` |
| Style resolution | Every frame | Every frame by default; opt-in memoization |
| Focus visuals | none | Animated focus ring on interactive containers |
| Events | 5 | Adds `on_breakpoint_change` |

---

## Known limitations

* **Not compiled or run by the author** (no Rust toolchain was available).
  Treat the first `cargo test` as part of review.
* **Depends on the private `mud_egui_*` crates.** The code assumes these
  already-used APIs: `ComponentBase::{new, classes, class_names, has_class,
  add_class, set_elevation, elevation_value, ripple_enabled}`,
  `BsStyle::build_frame`, `TwStyle::build_frame`, `TwState { hovered, active,
  dark }`, `MudTheme::{fetch, current_palette}`, `palette.resolve(&str)`, and
  `MaxWidth` being `Copy + Debug + PartialEq` with variants
  `Xs/Sm/Md/Lg/Xl/Xxl/False`. `class_names()` must be iterable with `.iter()`
  and hashable items. `elevation_value()` must be an integer.
* **egui version.** The code assumes the `i8`-field `Margin` and `UiBuilder`
  era of egui (0.31 or later). Version-fragile APIs (such as `Shadow` and
  `rect_stroke`) were deliberately avoided; `Frame`'s `fill`, `stroke`,
  `inner_margin` and `outer_margin` fields, `Ui::disable`,
  `Response::widget_info`, and `Context::run` / `CentralPanel` in tests are
  used. Tests also depend on `Ui::set_max_width` behaving as documented.
* **Frame memoization cannot observe `MudTheme` changes** that do not alter
  egui's visuals; call `invalidate_style_cache()`.
* **Interactive containers reserve a 2 pt stroke** for the focus ring
  (`focus_ring(false)` removes it).
* **Callback-bearing containers capture input from children** by design; see
  [Click passthrough](#click-passthrough).
* **Unbounded offers report `Xs`** as the viewport bucket, because there is no
  meaningful bucket for an infinite width.
* **Not part of MudBlazor's `MudContainer`, and intentionally absent:**
  `Value`/`ValueChanged`/`ValueExpression`, `Variant`, `Color`, `Size`,
  `ReadOnly`, `AdornmentStart`/`AdornmentEnd`, `OnKeyDown`, `OnChange`.

## License

Use the license of the surrounding `mud_egui` workspace.