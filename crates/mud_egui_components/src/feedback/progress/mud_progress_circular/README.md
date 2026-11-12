# MudProgressCircular for `mud_egui`

A circle-shaped indicator of progress for
[egui](https://github.com/emilk/egui), ported from MudBlazor's
[`MudProgressCircular`](https://mudblazor.com/components/progress) (MudBlazor 8.13.0).

```rust
// Determinate, 72 % complete.
ui.add(
    MudProgressCircular::new()
        .value(72.0)
        .color(Color::Primary),
);

// Indeterminate: a rotating, breathing arc.
ui.add(
    MudProgressCircular::new()
        .indeterminate(true)
        .color(Color::Info)
        .size(Size::Small),
);
```

The ring paints no text of its own unless you supply a `label` closure. A label's
contents should go through **`MudText`**, and any glyph through **`MudIcon`**.

---

## Contents

1. [Design principle: API parity, not API invention](#design-principle-api-parity-not-api-invention)
2. [Architecture](#architecture)
3. [API reference](#api-reference)
4. [Examples](#examples)
5. [Layout and rendering](#layout-and-rendering)
6. [Performance and memory choices](#performance-and-memory-choices)
7. [Testing](#testing)
8. [Differences from MudBlazor](#differences-from-mudblazor)

---

## Design principle: API parity, not API invention

The MudBlazor 8.13.0 parameter surface was verified against DNDocs before
implementation. `MudProgressCircular` has **exactly nine parameters**, and all
nine are ported.

### Ported parameters

| MudBlazor | Type | Default | Rust mapping |
|---|---|---|---|
| `ChildContent` | `RenderFragment?` | `null` | `label(FnMut(&mut Ui))` |
| `Color` | `Color` | `Default` | `color(Color)` |
| `Indeterminate` | `bool` | `false` | `indeterminate(bool)` |
| `Max` | `double` | `100.0` | `max(f64)` |
| `Min` | `double` | `0.0` | `min(f64)` |
| `Rounded` | `bool` | `false` | `rounded(bool)` |
| `Size` | `Size` | `Medium` | `size(Size)` |
| `StrokeWidth` | `int` | `3` | `stroke_width(u8)` |
| `Value` | `double` | `0.0` | `value(f64)` |
| `Class` | `string?` | `null` | `classes(iter)` — stored, not resolved |
| `Tag` | `object?` | `null` | `tag(t)` / `tag_ref::<U>()` |

### Deliberately not ported

| MudBlazor | Reason |
|---|---|
| `Style` | egui has no inline CSS strings. |
| `UserAttributes` | There is no HTML tag to splat attributes onto. |
| `Class` resolution | Stored on `base` for parity only. The ring's colour comes from `color`; a utility-class frame would composite a second fill. |

There is **no** `ValueChanged`. The DNDocs page lists `Value` as a plain
`double` parameter, and the component inherits `IMudStateHasChanged` but
exposes no two-way binding. This is a display-only component, and the port does
not invent a callback.

### `ChildContent` is the percentage overlay, not a wrapper

In MudBlazor, `ChildContent` is rendered inside
`.mud-progress-circular-percentage`, an absolutely positioned flex box that
covers the whole circle and centres its content[reference:6]. The Rust mapping
is `label(closure)`: it runs in a child `Ui` sized to the circle and centred
both ways.

---

## Architecture

```
feedback/progress_circular/
└── mud_progress_circular.rs
    ├── constants        ARC_SEGMENTS 64, LOOP_SECONDS 1.4,
    │                    MIN_SWEEP 30°, MAX_SWEEP 288°, TRACK_ALPHA 60,
    │                    MIN_STROKE 1, START_ANGLE −90°
    ├── pure helpers     diameter, effective_stroke_width, fraction,
    │                    indeterminate_phase, rotation_for, sweep_for,
    │                    with_alpha, arc_points
    ├── paint_arc        polyline + optional round caps
    ├── MudProgressCircular  builder + show
    └── impl Widget      ui.add(MudProgressCircular::new())
```

**Pure layer.** The diameter, the stroke clamp, the value fraction, the
indeterminate phase / rotation / sweep and the arc polyline are free functions
over `f32`, `f64` and `Pos2`. None touch `egui::Context`, so they are
unit-tested without a window.

**Geometry layer.** `arc_points` turns `(center, radius, start, sweep)` into a
`Vec<Pos2>` of `ARC_SEGMENTS + 1` points. `paint_arc` submits that polyline as
one `Shape::line` and, when rounded, adds two `circle_filled` caps.

**Component layer.** `show` fetches the theme once, resolves one accent colour,
allocates a square rect, paints the track, the arc and the label, and requests a
repaint only when indeterminate.

---

## API reference

| Method | Type | Default | Mirrors | Description |
|---|---|---|---|---|
| `new()` | `Self` | — | — | Value `0`, range `0..=100`, `Default`, `Medium`, stroke `3`. |
| `color(c)` | `Color` | `Default` | `Color` | Accent of the arc. |
| `size(s)` | `Size` | `Medium` | `Size` | Diameter: 24 / 40 / 56 pt. |
| `stroke_width(w)` | `u8` | `3` | `StrokeWidth` | Ring thickness; clamped to `1..=diameter / 2`. |
| `value(v)` | `f64` | `0.0` | `Value` | Current progress; ignored when indeterminate. |
| `min(v)` | `f64` | `0.0` | `Min` | Lower bound. |
| `max(v)` | `f64` | `100.0` | `Max` | Upper bound. |
| `indeterminate(b)` | `bool` | `false` | `Indeterminate` | Rotating, breathing arc. |
| `rounded(b)` | `bool` | `false` | `Rounded` | Round the arc ends. |
| `label(f)` | `FnMut(&mut Ui) + 'static` | `None` | `ChildContent` | Centred overlay content. |
| `classes(iter)` | `iter of AsRef<str>` | none | `Class` | Stored on `ComponentBase`. |
| `tag(t)` / `tag_ref::<U>()` | `T: Any + Send + Sync` | `None` | `Tag` | Arbitrary user data. |
| `show(ui)` | `&mut Ui` | — | — | Returns the circle's `Response`. |

### `Size` mapping

| `Size` | Diameter |
|---|---|
| `ExtraSmall` | 24 pt |
| `Small` | 24 pt |
| `Medium` (default) | 40 pt |
| `Large` | 56 pt |
| `ExtraLarge` | 56 pt |

`ExtraSmall` and `ExtraLarge` fold into `Small` and `Large`, matching
`.mud-progress-small` and `.mud-progress-large` in MudBlazor's SCSS[reference:7].

### Constants

| Name | Value |
|---|---|
| `ARC_SEGMENTS` | `64` |
| `LOOP_SECONDS` | `1.4` |
| `MIN_SWEEP` | `π / 6` (30°) |
| `MAX_SWEEP` | `1.6π` (288°) |
| `TRACK_ALPHA` | `60` |
| `MIN_STROKE` | `1.0` |

---

## Examples

### Basic determinate circle

```rust
MudProgressCircular::new()
    .value(72.0)
    .color(Color::Primary)
    .show(ui);
```

### With a percentage label

```rust
MudProgressCircular::new()
    .value(72.0)
    .size(Size::Large)
    .label(|ui| {
        ui.add(MudText::body2("72%"));
    })
    .show(ui);
```

The closure runs once per frame in a `Ui` sized to the circle and centred both
ways.

### Indeterminate spinner

```rust
MudProgressCircular::new()
    .indeterminate(true)
    .color(Color::Info)
    .size(Size::Small)
    .show(ui);
```

### Inline spinner next to text

```rust
ui.horizontal(|ui| {
    ui.add(
        MudProgressCircular::new()
            .indeterminate(true)
            .size(Size::Small),
    );
    ui.add(MudText::body1("Loading…"));
});
```

### Rounded arc

```rust
MudProgressCircular::new()
    .value(60.0)
    .rounded(true)
    .stroke_width(6)
    .size(Size::Large)
    .show(ui);
```

`rounded(true)` paints a cap of radius `stroke_width / 2` at each end of the
arc, which is MudBlazor's `stroke-linecap: round`.

### Thick ring

```rust
MudProgressCircular::new()
    .value(80.0)
    .stroke_width(8)
    .size(Size::Large)
    .show(ui);
```

### Custom range

```rust
MudProgressCircular::new()
    .min(0.0)
    .max(1024.0)
    .value(512.0) // half full
    .show(ui);
```

### Zero value (empty ring)

```rust
MudProgressCircular::new().value(0.0).show(ui);
```

The track is still drawn, so the circle is visible; only the accent arc is
absent. MudBlazor behaves the same way.

### Tag round-trip

```rust
let ring = MudProgressCircular::new().value(50.0).tag("spinner");
assert_eq!(ring.tag_ref::<&str>(), Some(&"spinner"));
ring.show(ui);
```

---

## Layout and rendering

### Anatomy

```
        ┌─────────────┐
        │   ╭─────╮   │   ← track (accent at 60 α)
        │  ╱       ╲  │
        │ │         │ │   ← arc (accent, 3 pt by default)
        │  ╲       ╱  │
        │   ╰─────╯   │   ← 12 o'clock start, clockwise
        └─────────────┘
        │   [label]   │   ← centred overlay (ChildContent)
```

- **Diameter:** `diameter(size)` — 24, 40 or 56 pt.
- **Radius:** `(diameter − stroke_width) / 2`, so the stroke never escapes the
  allocated square.
- **Start angle:** −90° (12 o'clock), matching the SCSS's
  `transform: rotate(-90deg)`[reference:8].
- **Direction:** clockwise, because the arc is sampled with increasing angle in
  egui's screen space (y grows down).
- **Track:** `painter.circle_stroke(center, radius, Stroke::new(stroke, track))`.
- **Arc:** one `Shape::line` of `ARC_SEGMENTS + 1` points.
- **Rounded caps:** one `circle_filled` at each end, radius `stroke_width / 2`.

### Indeterminate model

```
phase 0.00:  arc at 12 o'clock, 30° wide
phase 0.25:  arc at  3 o'clock, 159° wide
phase 0.50:  arc at  6 o'clock, 288° wide
phase 0.75:  arc at  9 o'clock, 159° wide
phase 1.00:  arc at 12 o'clock, 30° wide
```

`phase = (time mod 1.4) / 1.4`. The rotation is `START_ANGLE + phase × TAU`; the
sweep is `MIN_SWEEP + (MAX_SWEEP − MIN_SWEEP) × (0.5 − 0.5 × cos(phase × TAU))`.
The result is a single arc that makes one full turn per loop while its length
breathes in and out once per loop.

MudBlazor achieves the same look with two CSS animations — the whole SVG rotates
one way, and the dash pattern rotates the other way — plus a `stroke-dasharray`
`80px 200px`[reference:9]. The Rust port collapses the two rotations into one and
replaces the CSS `ease-in-out` dash with a sinusoidal sweep. At a glance the
two read the same.

### Degenerate inputs

| Input | Behaviour |
|---|---|
| `max <= min` | `fraction` returns `0.0`; only the track is drawn. |
| Non-finite `value` / `min` / `max` | Treated as `0.0`; only the track is drawn. |
| `stroke_width > diameter / 2` | Clamped to `diameter / 2`; the radius is never negative. |
| `stroke_width == 0` | Clamped to `1`; the ring is always visible. |
| `indeterminate` + `value` | `value` is ignored, matching MudBlazor. |

---

## Performance and memory choices

| Choice | Effect |
|---|---|
| Theme fetched once per frame | One `MudTheme::fetch` and one `Color::resolve`. |
| Pure helpers on `Copy` values | Diameter, stroke clamp, fraction, phase, rotation and sweep are arithmetic; unit-testable without a `Context`. |
| One `Shape::line` per frame | The arc is a single polyline, not 64 separate line segments. |
| `Vec<Pos2>` capacity is `ARC_SEGMENTS + 1` | One allocation, bounded and predictable. |
| Track is one `circle_stroke` | No per-segment work for the background ring. |
| `rounded` adds two `circle_filled` | Only when the flag is on; a flat-cap arc costs nothing extra. |
| Label closure runs once | No double-run measure/draw pass. |
| Repaint only when indeterminate | A determinate circle requests no repaints; an indeterminate one does, as it must. |
| `Sense::hover()` | The ring is not interactive, matching MudBlazor. |

---

## Testing

`cargo test -p mud_egui_components mud_progress_circular` runs 18 pure-function
tests:

| Area | Tests |
|---|---|
| Diameter | every `Size`; monotonicity across small/medium/large |
| Stroke width | clamping between `1` and `diameter / 2`; degenerate diameter still yields `1` |
| `fraction` | unit range, clamping, degenerate `max <= min`, non-finite inputs |
| Indeterminate phase | periodicity across two loops; `NaN` / `∞` repair |
| Rotation | completes exactly one full turn per loop |
| Sweep | reaches `MIN_SWEEP` at 0 and 1, `MAX_SWEEP` at 0.5; never leaves the band |
| Arc points | starts at 12 o'clock; runs clockwise; `n + 1` points; every point lies on the circle; a full turn closes on itself |
| Builders | defaults match MudBlazor; every field set; tag round-trip |

No test creates an `egui::Context`. `show` needs a live `Ui` (layout, painting,
theme lookup, time) and is therefore left to integration tests; everything it
delegates to is covered above.

Suggested integration tests (`egui_kittest` or a headless `Context::run`
harness):

- A determinate circle at `value(50)` draws an arc covering half the ring.
- `value(0)` draws only the track.
- `rounded(true)` adds two caps; `rounded(false)` adds none.
- An indeterminate circle requests a repaint and its rotation advances with
  time.
- A label closure runs exactly once per frame.

```bash
cargo test -p mud_egui_components mud_progress_circular
```

---

## Differences from MudBlazor

- **`ChildContent` becomes `label(FnMut(&mut Ui))`.** The closure runs once per
  frame in a `Ui` sized to the circle and centred both ways, the direct
  equivalent of `.mud-progress-circular-percentage`.
- **The arc is a polyline, not an SVG path.** `ARC_SEGMENTS = 64` makes the
  approximation invisible at every `Size`. If you need a mathematically exact
  arc, replace the `Shape::line` call in `paint_arc` with an `epaint::PathShape`
  of cubic Béziers — nothing else changes.
- **`Rounded` draws caps instead of setting `stroke-linecap`.** egui has no
  stroke cap; a filled circle of radius `stroke_width / 2` at each endpoint is
  the same silhouette.
- **The indeterminate animation is a single time-based arc.** MudBlazor's
  `rotate` + `dash` CSS animations are approximated by one rotating arc whose
  sweep breathes sinusoidally. The loop is still 1.4 s and the visual rhythm
  matches.
- **`Size` folds `ExtraSmall` into `Small` and `ExtraLarge` into `Large`.** The
  SCSS only defines `small`, `medium` and `large`; folding the edges is what
  MudBlazor does.
- **`StrokeWidth` is `u8`, not `int`.** Clamped to `1..=diameter / 2` at paint
  time so the radius can never invert. The default is still `3`.
- **`Class` is stored but not resolved.** The ring's colour comes from `color`;
  a utility-class frame would composite a second fill.
- **`Style` and `UserAttributes` are not ported.** No CSS strings, no HTML tag.
- **`value` is `f64`, not `Option<double>`.** The DNDocs page lists it as a
  plain `double` and the component exposes no `ValueChanged`; this is a
  display-only component, and no two-way binding is invented.