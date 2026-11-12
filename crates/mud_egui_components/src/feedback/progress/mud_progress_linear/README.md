# MudProgressLinear for `mud_egui`

A line-shaped indicator of progress for
[egui](https://github.com/emilk/egui), ported from MudBlazor's
[`MudProgressLinear`](https://mudblazor.com/components/progress) (MudBlazor 8.13.0).

```rust
// Determinate, 45 % complete.
ui.add(
    MudProgressLinear::new()
        .value(45.0)
        .color(Color::Primary),
);

// Indeterminate: a segment slides back and forth.
ui.add(
    MudProgressLinear::new()
        .indeterminate(true)
        .color(Color::Info)
        .size(Size::Small),
);
```

The bar paints no text of its own unless you supply a `label` closure. A label's
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
implementation. `MudProgressLinear` has **exactly twelve parameters**, and all
twelve are ported.

### Ported parameters

| MudBlazor | Type | Default | Rust mapping |
|---|---|---|---|
| `Buffer` | `bool` | `false` | `buffer(bool)` |
| `BufferValue` | `double` | `0.0` | `buffer_value(f64)` |
| `ChildContent` | `RenderFragment?` | `null` | `label(FnMut(&mut Ui))` |
| `Color` | `Color` | `Default` | `color(Color)` |
| `Indeterminate` | `bool` | `false` | `indeterminate(bool)` |
| `Max` | `double` | `100.0` | `max(f64)` |
| `Min` | `double` | `0.0` | `min(f64)` |
| `Rounded` | `bool` | `false` | `rounded(bool)` |
| `Size` | `Size` | `Medium` | `size(Size)` |
| `Striped` | `bool` | `false` | `striped(bool)` |
| `Value` | `double` | `0.0` | `value(f64)` |
| `Vertical` | `bool` | `false` | `vertical(bool)` |
| `Class` | `string?` | `null` | `classes(iter)` — stored, not resolved |
| `Tag` | `object?` | `null` | `tag(t)` / `tag_ref::<U>()` |

### Deliberately not ported

| MudBlazor | Reason |
|---|---|
| `Style` | egui has no inline CSS strings. Use `length` for the one thing `Style` was routinely used for on this component (`Style="width: …"`). |
| `UserAttributes` | There is no HTML tag to splat attributes onto. |
| `Class` resolution | Stored on `base` for parity only. The bar's fill and track come from `color` and `Size`; a utility-class frame would composite a second fill. |

### `ChildContent` is the label, not a wrapper

In MudBlazor, `ChildContent` is rendered **inside** the bar, in the
`.mud-progress-linear-content` overlay. It is not the bar's parent. The Rust
mapping is `label(closure)`, drawn in a child `Ui` laid out over the whole bar,
centred both ways.

### Extensions

| Extension | Type | Default | Why |
|---|---|---|---|
| `length(f32)` | `f32` | `None` | The only common use of `Style` on this component is a width override. `length` is the typed equivalent. |

`length` sets the bar's extent along its axis: width for a horizontal bar,
height for a vertical one. Without it, a horizontal bar fills the available
width (clamped to `DEFAULT_HORIZONTAL_LENGTH`) and a vertical one is
`DEFAULT_VERTICAL_LENGTH` tall.

---

## Architecture

```
feedback/progress_linear/
└── mud_progress_linear.rs
    ├── constants        track_thickness, DEFAULT_HORIZONTAL_LENGTH 240,
    │                    DEFAULT_VERTICAL_LENGTH 160, TRACK_ALPHA 60,
    │                    INDETERMINATE_SEGMENT_FRACTION 0.35,
    │                    LOOP_SECONDS 1.8, STRIPE_PERIOD 12, STRIPE_WIDTH 6,
    │                    STRIPE_ALPHA 90
    ├── pure helpers     fraction, indeterminate_phase,
    │                    indeterminate_fractions, bar_radius, radius_u8,
    │                    finite_or, with_alpha
    ├── BarGeometry      slice(a, b), label_rect()
    ├── MudProgressLinear  builder + show
    └── impl Widget      ui.add(MudProgressLinear::new())
```

**Pure layer.** The value fraction, the indeterminate phase and segment, the
corner radius and the alpha helper are free functions over `f64` and `f32`.
None touch `egui::Context`, so they are unit-tested without a window.

**Geometry layer.** `BarGeometry` is a small `Copy` struct that turns `(a, b)`
fractions into a `Rect`, handling both orientations. One code path serves the
value fill, the buffer fill and the indeterminate segment.

**Component layer.** `show` fetches the theme once, resolves one accent colour,
allocates the bar rect, and paints track → buffer → value → stripes → label.

### Why the label is a closure, not a wrapper

MudBlazor's progress bar is a single HTML element; `ChildContent` is absolutely
positioned over it. A closure that receives a `Ui` already sized and centred on
the bar is the direct equivalent. The closure runs exactly once per frame.

---

## API reference

| Method | Type | Default | Mirrors | Description |
|---|---|---|---|---|
| `new()` | `Self` | — | — | Value `0`, range `0..=100`, `Default`, `Medium`, horizontal. |
| `color(c)` | `Color` | `Default` | `Color` | Accent of the fill. |
| `size(s)` | `Size` | `Medium` | `Size` | Bar thickness: 4 / 6 / 10 pt. |
| `value(v)` | `f64` | `0.0` | `Value` | Current progress. Clamped into the range. |
| `min(v)` | `f64` | `0.0` | `Min` | Lower bound. |
| `max(v)` | `f64` | `100.0` | `Max` | Upper bound. |
| `indeterminate(b)` | `bool` | `false` | `Indeterminate` | Sliding segment, ignores `value`. |
| `buffer(b)` | `bool` | `false` | `Buffer` | Overlay ahead of the value. |
| `buffer_value(v)` | `f64` | `0.0` | `BufferValue` | Buffer amount. |
| `striped(b)` | `bool` | `false` | `Striped` | Diagonal stripes on the fill. |
| `rounded(b)` | `bool` | `false` | `Rounded` | Capsule ends. |
| `vertical(b)` | `bool` | `false` | `Vertical` | Run bottom-to-top. |
| `length(px)` | `f32` | `None` | *(extension)* | Explicit length along the axis. |
| `label(f)` | `FnMut(&mut Ui) + 'static` | `None` | `ChildContent` | Overlay content. |
| `classes(iter)` | `iter of AsRef<str>` | none | `Class` | Stored on `ComponentBase`. |
| `tag(t)` / `tag_ref::<U>()` | `T: Any + Send + Sync` | `None` | `Tag` | Arbitrary user data. |
| `show(ui)` | `&mut Ui` | — | — | Returns the bar's `Response`. |

### `Size` mapping

| `Size` | Thickness |
|---|---|
| `ExtraSmall` | 4 pt |
| `Small` | 4 pt |
| `Medium` (default) | 6 pt |
| `Large` | 10 pt |
| `ExtraLarge` | 10 pt |

`ExtraSmall` and `ExtraLarge` fold into `Small` and `Large`. MudBlazor's
stylesheet does the same for this component.

### Constants

| Name | Value |
|---|---|
| `DEFAULT_HORIZONTAL_LENGTH` | `240.0` |
| `DEFAULT_VERTICAL_LENGTH` | `160.0` |
| `TRACK_ALPHA` | `60` |
| `INDETERMINATE_SEGMENT_FRACTION` | `0.35` |
| `LOOP_SECONDS` | `1.8` |
| `STRIPE_PERIOD` | `12.0` |
| `STRIPE_WIDTH` | `6.0` |
| `STRIPE_ALPHA` | `90` |

---

## Examples

### Basic determinate bar

```rust
MudProgressLinear::new()
    .value(72.0)
    .color(Color::Primary)
    .show(ui);
```

### With a percentage label

```rust
MudProgressLinear::new()
    .value(72.0)
    .label(|ui| {
        ui.add(MudText::caption("72%"));
    })
    .show(ui);
```

The closure gets a `Ui` sized to the whole bar and centred both ways. A
`MudText::caption` with a contrasting colour reads well over the fill.

### Custom range

```rust
MudProgressLinear::new()
    .min(0.0)
    .max(1024.0)
    .value(512.0) // half full
    .color(Color::Success)
    .show(ui);
```

### Buffer

```rust
MudProgressLinear::new()
    .value(30.0)          // solid fill
    .buffer(true)
    .buffer_value(60.0)   // faint fill ahead of the solid fill
    .color(Color::Info)
    .show(ui);
```

The buffer fill is drawn first at `TRACK_ALPHA + 60` alpha, so it reads as
"the bar will go this far". The value fill is painted on top at full alpha.

### Indeterminate

```rust
MudProgressLinear::new()
    .indeterminate(true)
    .color(Color::Warning)
    .size(Size::Small)
    .show(ui);
```

The bar requests a repaint every frame while indeterminate. `value`, `buffer`
and `striped` are ignored.

### Striped

```rust
MudProgressLinear::new()
    .value(80.0)
    .striped(true)
    .color(Color::Secondary)
    .show(ui);
```

### Rounded

```rust
MudProgressLinear::new()
    .value(60.0)
    .rounded(true)
    .size(Size::Large)
    .show(ui);
```

The corner radius is half the thickness, so a `Large` bar is a 10 pt capsule.

### Vertical

```rust
MudProgressLinear::new()
    .value(40.0)
    .vertical(true)
    .length(200.0) // 200 pt tall
    .color(Color::Success)
    .show(ui);
```

`Value = Min` sits at the bottom. The indeterminate segment also reverses, so it
enters from the top and exits at the bottom.

### Fixed width

```rust
MudProgressLinear::new()
    .value(30.0)
    .length(160.0)   // not the available width
    .show(ui);
```

### Tag round-trip

```rust
let bar = MudProgressLinear::new().value(50.0).tag("upload");
assert_eq!(bar.tag_ref::<&str>(), Some(&"upload"));
bar.show(ui);
```

---

## Layout and rendering

### Horizontal bar

```
LTR:  [ ████████████░░░░░░░░░░░░ ]   ← value fill (accent) over track (60 α)
        ↑
        slice(0.0, value_fraction)
```

- Width: `ui.available_width()`, clamped to `length` when set.
- Height: `track_thickness(size)` — 4, 6 or 10 pt.
- Track: the whole rect, accent at 60 alpha.
- Buffer: `slice(0.0, buffer_fraction)`, accent at 120 alpha.
- Value: `slice(0.0, value_fraction)`, accent at full alpha.
- Stripes: white at 90 alpha over the value fill.
- Label: child `Ui` over the whole rect, centred both ways.

### Vertical bar

```
   [ █ ]  ← top (Max)
   [ █ ]
   [ ░ ]
   [ ░ ]  ← bottom (Min)
```

- Width: `track_thickness(size)`.
- Height: `length` when set, else `DEFAULT_VERTICAL_LENGTH` (160 pt).
- `slice(a, b)` measures from the **bottom**, so `Value = Min` is at the bottom.

### Indeterminate

```
phase 0.0:  [ ████░░░░░░░░░░░░░░░░ ]
phase 0.5:  [ ░░░░░░░░░████░░░░░░░░ ]
phase 1.0:  [ ░░░░░░░░░░░░░░░░░████ ]
```

The segment is `INDETERMINATE_SEGMENT_FRACTION` (35 %) of the bar length and
travels from `-width` to `1.0`, so it is fully visible in the middle of the
loop and enters/exits at the edges. The phase is `(time mod 1.8) / 1.8`.

### Degenerate inputs

| Input | Behaviour |
|---|---|
| `max <= min` | Value and buffer fractions are `0.0`; nothing fills. |
| Non-finite `value` / `min` / `max` | Treated as `0.0` for the fraction; nothing fills. |
| Non-finite `length` | Falls back to the default length for the orientation. |
| Negative `length` | Clamped to `1.0`. |
| `indeterminate` + `buffer` | Buffer is ignored, matching MudBlazor. |
| `indeterminate` + `striped` | Stripes are ignored, matching MudBlazor. |

---

## Performance and memory choices

| Choice | Effect |
|---|---|
| Theme fetched once per frame | One `MudTheme::fetch` and one `Color::resolve`. |
| Pure helpers on `Copy` values | Value fraction, phase, segment and radius are arithmetic; unit-testable without a `Context`. |
| One `BarGeometry` for all paths | Value, buffer and indeterminate share `slice(a, b)`; no duplicated orientation branches. |
| No `Vec` or `String` allocation | The label closure is the only place user code runs, and it runs once. |
| Stripe pattern is a loop over 12 pt periods | Same cost as a plain bar for any realistic width; no shader, no shape list. |
| Repaint only when indeterminate | A determinate bar requests no repaints; an indeterminate one does, as it must. |
| `Sense::hover()` | The bar is not interactive. This matches MudBlazor, which has no click handler on it. |
| `allocate_exact_size` | One rect, one response; the caller can wrap it in a `MudStack` or `MudPaper` without side effects. |

---

## Testing

`cargo test -p mud_egui_components mud_progress_linear` runs 18 pure-function
tests:

| Area | Tests |
|---|---|
| Thickness | every `Size`; monotonicity across small/medium/large |
| `fraction` | unit range, clamping, degenerate `max <= min`, non-finite inputs |
| Indeterminate phase | periodicity across two loops; `NaN` / `∞` repair |
| Indeterminate segment | starts at `0.0`; ends at `1.0`; monotonic start across the loop; never leaves `0..=1` for any phase |
| `BarGeometry` | horizontal left-to-right slice; vertical bottom-to-top slice; `slice` clamps and swaps inverted inputs |
| Radius | half thickness when rounded; zero when not; `radius_u8` clamping |
| Builders | defaults match MudBlazor; every field set; tag round-trip |

No test creates an `egui::Context`. `show` needs a live `Ui` (layout, painting,
theme lookup, time) and is therefore left to integration tests; everything it
delegates to is covered above.

Suggested integration tests (`egui_kittest` or a headless `Context::run`
harness):

- A bar with `value(50)` in range `0..=100` fills exactly half the rect.
- A buffer overlay is drawn behind the value fill and reaches further.
- An indeterminate bar requests a repaint and its segment position advances
  with time.
- A vertical bar's fill starts at the bottom.
- A rounded `Large` bar has a corner radius of 5.

```bash
cargo test -p mud_egui_components mud_progress_linear
```

---

## Differences from MudBlazor

- **`ChildContent` becomes `label(FnMut(&mut Ui))`.** The closure runs once
  per frame in a `Ui` sized to the whole bar and centred both ways. This is the
  direct equivalent of `.mud-progress-linear-content`.
- **`length(f32)` is an extension.** MudBlazor relies on CSS `width`; `length`
  is the typed equivalent, and is the only reason to reach for `Style` on this
  component.
- **`Class` is stored but not resolved.** The bar's fill and track come from
  `color` and `Size`; a utility-class frame would composite a second fill.
- **`Style` and `UserAttributes` are not ported.** No CSS strings, no HTML tag.
- **`Size` folds `ExtraSmall` into `Small` and `ExtraLarge` into `Large`.** That
  is what MudBlazor's stylesheet does; the thickness table is identical.
- **`Rounded` sets a corner radius of half the thickness.** MudBlazor sets
  `border-radius` to the theme's default; half-thickness is what a capsule
  actually looks like, and is what Material's linear progress uses.
- **The stripe pattern is static, not animated.** MudBlazor animates
  `.mud-progress-linear-striped` with a CSS keyframe. egui has no CSS
  animations, and an animated stripe would need a repaint every frame for a
  purely decorative effect. The stripes are drawn as white at 90 alpha over the
  fill; if you need motion, layer a `MudProgressLinear::indeterminate` on top or
  animate the `length` yourself.
- **The indeterminate loop is 1.8 s.** MudBlazor's transition duration is fixed
  in CSS; this port exposes it as `LOOP_SECONDS` so a theme can be adapted by
  changing one constant.
- **`value` and `buffer_value` are `f64`, not `Option`.** MudBlazor exposes
  them as `double` with `ValueChanged` absent from the API page, confirming
  this is a display-only component. No two-way binding is invented.