# MudSkeleton for `mud_egui`

A temporary placeholder for content while data is loaded, for
[egui](https://github.com/emilk/egui). Ported from MudBlazor's
[`MudSkeleton`](https://mudblazor.com/components/skeleton) (MudBlazor 8.13.0).

```rust
// A text placeholder.
ui.add(MudSkeleton::new().width(240.0).height(16.0));

// A circle placeholder (avatar loading).
ui.add(
    MudSkeleton::new()
        .skeleton_type(SkeletonType::Circle)
        .width(48.0)
        .height(48.0)
        .animation(Animation::Wave),
);

// A rectangle placeholder (image loading).
ui.add(
    MudSkeleton::new()
        .skeleton_type(SkeletonType::Rectangle)
        .width(320.0)
        .height(180.0)
        .animation(Animation::Off),
);
```

`MudSkeleton` paints no text and no glyphs — it is a solid shape. There is
nothing inside it to route through `MudText` or `MudIcon`.

---

## Contents

1. [Design principle: API parity, not API invention](#design-principle-api-parity-not-api-invention)
2. [Architecture](#architecture)
3. [API reference](#api-reference)
4. [Examples](#examples)
5. [Layout and rendering](#layout-and-rendering)
6. [Animation pipeline](#animation-pipeline)
7. [Performance and memory choices](#performance-and-memory-choices)
8. [Testing](#testing)
9. [Differences from MudBlazor](#differences-from-mudblazor)

---

## Design principle: API parity, not API invention

The MudBlazor 8.13.0 parameter surface was verified against DNDocs before
implementation. `MudSkeleton` has **four parameters**, and all four are ported.

### Ported parameters

| MudBlazor | Type | Default | Rust mapping |
|---|---|---|---|
| `Animation` | `Animation` | `Pulse` | `animation(Animation)` |
| `Height` | `string?` | `null` | `height(f32)` → `Option<f32>` points |
| `SkeletonType` | `SkeletonType` | `Text` | `skeleton_type(SkeletonType)` |
| `Width` | `string?` | `null` | `width(f32)` → `Option<f32>` points |
| `Class` | `string?` | `null` | `classes(iter)` — stored, not resolved |
| `Tag` | `object?` | `null` | `tag(t)` / `tag_ref::<U>()` |

### Deliberately not ported

| MudBlazor | Reason |
|---|---|
| `Style` | egui has no inline CSS strings. The typed [`fill`](MudSkeleton::fill) and [`radius`](MudSkeleton::radius) extensions cover the two properties a caller would routinely set via `Style`. |
| `UserAttributes` | There is no HTML tag to splat attributes onto. |
| `Class` resolution | Stored on `base` for parity. The skeleton's only visual properties are its shape (an enum) and its fill (an extension); a utility-class frame would composite a second fill. |

### Extensions

| Extension | Type | Default | Purpose |
|---|---|---|---|
| `fill(Color32)` | `Color32` | theme tint | Explicit background colour, beats the theme's default skeleton tint. |
| `radius(f32)` | `f32` | per shape | Explicit corner radius in points, overrides the shape's default. |

Both are typed escape hatches for the two properties `Style` was routinely
used for on a skeleton; they are marked as extensions, not as MudBlazor
parameters.

### The `Animation` enum is renamed once

MudBlazor's `Animation.False = 0` variant cannot be named `False` in Rust.
The port maps it to [`Animation::Off`](crate::enums::Animation::Off), which
reads the same and avoids the keyword. The numeric values are preserved in
the doc comments for reference; nothing in the port depends on the numbers.

---

## Architecture

```
feedback/skeleton/
├── mud_skeleton.rs
│   ├── constants        DEFAULT_TEXT_HEIGHT 20, DEFAULT_CIRCLE_DIAMETER 40,
│   │                    DEFAULT_RECTANGLE_HEIGHT 100, TEXT_RADIUS 4,
│   │                    SKELETON_ALPHA 40, PULSE_PERIOD_SECONDS 1.3,
│   │                    PULSE_MIN_ALPHA 0.7, WAVE_PERIOD_SECONDS 1.0,
│   │                    WAVE_BAND_WIDTH 200, WAVE_PEAK_ALPHA 90,
│   │                    WAVE_HIGHLIGHT
│   ├── MudSkeleton      builder + show(ui) + Widget impl
│   ├── pure helpers     skeleton_dimensions, skeleton_radius, sane_positive,
│   │                    radius_u8, with_alpha, pulse_alpha, wave_phase,
│   │                    wave_band_x, paint_wave
│   └── mod tests        27 pure-function tests
└── mod.rs               re-exports

mud_egui_core/src/enums/
├── animation.rs         Animation (Off, Pulse, Wave)
└── skeleton_type.rs     SkeletonType (Text, Circle, Rectangle)
```

**Pure layer.** Dimension resolution, radius resolution, the pulse curve, the
wave phase and the wave band's three x-coordinates are free functions. None
touch `egui::Context`; the whole non-painting surface is unit-testable.

**Rendering layer.** `show` resolves the shape and colour, allocates one rect,
paints the base fill, and (for `Wave`) overlays a three-column vertex-coloured
mesh. `Off` and `Pulse` are a single `rect_filled`.

---

## API reference

| Method | Type | Default | Mirrors | Description |
|---|---|---|---|---|
| `new()` | `Self` | — | — | Text shape, Pulse animation, no explicit size. |
| `skeleton_type(kind)` | `SkeletonType` | `Text` | `SkeletonType` | Shape of the placeholder. |
| `animation(a)` | `Animation` | `Pulse` | `Animation` | Animation style. |
| `width(px)` | `f32` | `None` | `Width` | Explicit width in points. |
| `height(px)` | `f32` | `None` | `Height` | Explicit height in points. |
| `fill(c)` | `Color32` | theme tint | *(extension)* | Explicit background colour. |
| `radius(px)` | `f32` | per shape | *(extension)* | Explicit corner radius. |
| `classes(iter)` | `iter of AsRef<str>` | none | `Class` | Stored on `ComponentBase`. |
| `tag(t)` / `tag_ref::<U>()` | `T: Any + Send + Sync` | `None` | `Tag` | Arbitrary user data. |
| `show(ui)` | `&mut Ui` | — | — | Returns the skeleton's `Response`. |

### `SkeletonType`

| Variant | Width default | Height default | Corner radius |
|---|---|---|---|
| `Text` (default) | available width | 20 pt | 4 pt |
| `Circle` | `height` (forced square) | 40 pt | half the diameter |
| `Rectangle` | available width | 100 pt | 0 |

Helpers: `ALL`, `name()`, `Display`.

### `Animation`

| Variant | Behaviour |
|---|---|
| `Off` | Static fill. No repaints requested. |
| `Pulse` (default) | Opacity fades 1.0 → 0.7 → 1.0 in a 1.3 s loop. |
| `Wave` | A 200 pt gradient band of lighter fill sweeps left-to-right in a 1.0 s loop. |

Helpers: `ALL`, `name()`, `is_animated()`, `Display`.

### Constants

| Name | Value |
|---|---|
| `DEFAULT_TEXT_HEIGHT` | `20.0` pt |
| `DEFAULT_CIRCLE_DIAMETER` | `40.0` pt |
| `DEFAULT_RECTANGLE_HEIGHT` | `100.0` pt |
| `TEXT_RADIUS` | `4.0` pt |
| `SKELETON_ALPHA` | `40` |
| `PULSE_PERIOD_SECONDS` | `1.3` s |
| `PULSE_MIN_ALPHA` | `0.7` |
| `WAVE_PERIOD_SECONDS` | `1.0` s |
| `WAVE_BAND_WIDTH` | `200.0` pt |
| `WAVE_PEAK_ALPHA` | `90` |

---

## Examples

### Text skeleton (default)

```rust
ui.add(MudSkeleton::new());
```

A single 20 pt line filling the available width, pulsing.

### Text skeleton with explicit size

```rust
ui.add(MudSkeleton::new().width(240.0).height(16.0));
```

### Circle (avatar loading)

```rust
ui.add(
    MudSkeleton::new()
        .skeleton_type(SkeletonType::Circle)
        .width(48.0),
);
```

`width(48.0)` alone is enough; `Circle` forces the height to match, so the
result is a 48 pt circle.

### Rectangle (image loading)

```rust
ui.add(
    MudSkeleton::new()
        .skeleton_type(SkeletonType::Rectangle)
        .width(320.0)
        .height(180.0)
        .animation(Animation::Wave),
);
```

### Static skeleton (no animation)

```rust
ui.add(
    MudSkeleton::new()
        .skeleton_type(SkeletonType::Rectangle)
        .width(320.0)
        .height(180.0)
        .animation(Animation::Off),
);
```

No repaints are requested for an `Off` skeleton; it costs nothing per frame
until something else changes.

### Custom fill and radius

```rust
ui.add(
    MudSkeleton::new()
        .skeleton_type(SkeletonType::Rectangle)
        .width(320.0)
        .height(8.0)
        .fill(Color32::from_rgb(80, 80, 100))
        .radius(4.0),
);
```

`fill` and `radius` are extensions — the typed equivalent of `Style="…"` on a
Blazor skeleton.

### Filling the available width

```rust
ui.add(
    MudSkeleton::new()
        .width(ui.available_width())
        .height(20.0),
);
```

`Text` and `Rectangle` default to the available width when `width` is unset,
so this is the same as leaving `width` off.

### Tag round-trip

```rust
let s = MudSkeleton::new().tag("avatar-loading");
assert_eq!(s.tag_ref::<&str>(), Some(&"avatar-loading"));
```

---

## Layout and rendering

### Anatomy

```
Text (4 pt radius):
  ┌────────────────────────────────────────┐ 20 pt
  └────────────────────────────────────────┘
  ┌──────┐
Circle (radius = half):
  ┌──┐
  │  │ 40 pt
  └──┘
Rectangle (0 radius):
  ┌────────────────────────┐
  │                        │ 100 pt
  └────────────────────────┘
```

- **Text**: rounded 4 pt, one `Body1` line-height tall, fills the available
  width. This is the shape of a text line placeholder.
- **Circle**: square (width = height), corner radius = half the diameter, so
  it is a perfect circle. The smaller of `width` and `height` wins when both
  are set, so a `Circle` never becomes an ellipse.
- **Rectangle**: square corners, fills the available width, 100 pt tall. The
  shape of an image or card placeholder.

### Fill

The base fill is `ui.visuals().widgets.noninteractive.bg_fill` at
`SKELETON_ALPHA` (40). That is egui's muted background, which reads as "not
yet loaded" in both light and dark themes, and matches the role
`--mud-palette-action-default-hover` plays in MudBlazor's SCSS.

An explicit `fill(c)` overrides the colour but not the alpha; the alpha stays
at `SKELETON_ALPHA` so a bright custom colour still reads as a placeholder.

### Per-shape defaults

| Shape | When `width` is unset | When `height` is unset |
|---|---|---|
| `Text` | `ui.available_width()` | `DEFAULT_TEXT_HEIGHT` (20 pt) |
| `Circle` | `height`, else `DEFAULT_CIRCLE_DIAMETER` | `width`, else `DEFAULT_CIRCLE_DIAMETER` |
| `Rectangle` | `ui.available_width()` | `DEFAULT_RECTANGLE_HEIGHT` (100 pt) |

### Degenerate inputs

| Input | Behaviour |
|---|---|
| `width = NaN` | Falls back to the shape's default. |
| `width < 0` | Falls back to the shape's default. |
| `Circle` with only `height` set | Width = height, so the result is a square. |
| `Circle` with both set to different values | The smaller wins; the result is a square of that side. |
| `radius = NaN` | Falls back to the shape's default. |
| `fill` alpha | Ignored; the skeleton always uses `SKELETON_ALPHA`. |

---

## Animation pipeline

```
time ──► Pulse?  ──► pulse_alpha(time)   ──► gamma_multiply(alpha)
     │
     └──► Wave?   ──► wave_phase(time)
                   ──► wave_band_x(phase, w, 200)
                   ──► three-column vertex-coloured Mesh
                   ──► painter.add(Shape::mesh(mesh))
```

### Pulse

`pulse_alpha(t) = 0.7 + 0.3 · (1 + cos(2π · (t mod 1.3) / 1.3)) / 2`

At the start and end of the loop the alpha is `1.0`; at the halfway point it is
`0.7`. The fill is gamma-multiplied by that factor, so the whole shape fades
in and out together.

### Wave

`wave_phase(t) = (t mod 1.0) / 1.0`

The wave band is `WAVE_BAND_WIDTH` (200 pt) wide and its left edge travels from
`-WAVE_BAND_WIDTH` to the rect's right edge over one period. The band's three
x-coordinates (left edge, centre, right edge) are clamped to `[0, w]`, so a
band that has partially exited the rect is clipped to it by construction.

The mesh has six vertices — two rows (top and bottom) × three columns — with
vertex colours `transparent → WAVE_HIGHLIGHT → transparent`. Painting that
mesh over the base fill produces a horizontal shine that sweeps across the
shape.

### Repaints

| Animation | Requests repaint |
|---|---|
| `Off` | Never |
| `Pulse` | Every frame |
| `Wave` | Every frame |

`Pulse` and `Wave` call `ui.ctx().request_repaint()` once per frame; a window
with only static `Off` skeletons therefore does not repaint. All skeletons on
a page use the same clock, so they animate in lockstep — which is exactly what
MudBlazor's CSS animation does.

---

## Performance and memory choices

| Choice | Effect |
|---|---|
| Theme fetched only via `ui.visuals()` | No `MudTheme::fetch`; the skeleton colour comes from the active `Visuals`, which is one call. |
| `Off` requests no repaints | A page of static skeletons costs nothing per frame. |
| `Pulse` is one `rect_filled` plus a multiply | No mesh, no extra shapes. |
| `Wave` is one `Mesh` | Six vertices, twelve indices; the cheapest possible gradient band in egui. |
| No `Vec`, no `String` beyond the builder | The builder owns no collections; the mesh is built once per wave frame. |
| Pure helpers on `Copy` values | Dimension resolution, radius, pulse alpha, wave phase and band are arithmetic; unit-testable without a `Context`. |
| No `egui::Memory` | The skeleton is stateless between frames; the animation is driven by the shared clock. |
| `Sense::hover()` | The skeleton is not interactive, matching MudBlazor. |

---

## Testing

`cargo test -p mud_egui_components mud_skeleton` runs 27 pure-function tests:

| Area | Tests |
|---|---|
| Dimensions | `Text` defaults and explicit size; `Circle` defaults and smaller-side-wins; `Circle` with one side set; `Rectangle` defaults; non-finite and negative repair |
| Radius | per-shape defaults (Text 4, Circle half, Rectangle 0); override wins; override sanitization |
| Pulse | full alpha at loop boundaries; `PULSE_MIN_ALPHA` at half period; stays within band across a loop; non-finite repair |
| Wave | phase periodicity; non-finite repair; band fully off left at 0, fully off right at 1, centred at 0.5; band never leaves the rect across the loop |
| Builders | defaults match MudBlazor; every field set; tag round-trip |

Plus 9 tests in `mud_egui_core` for the two enums:

| Enum | Tests |
|---|---|
| `Animation` | default is `Pulse`; `ALL` has 3 unique variants; lowercase unique names; `Display` matches `name`; `is_animated` only false for `Off` |
| `SkeletonType` | default is `Text`; `ALL` has 3 unique variants; lowercase unique names; `Display` matches `name` |

No test creates an `egui::Context`. `MudSkeleton::show` and `paint_wave` need a
live `Ui` and are left to integration tests; everything they delegate to is
covered above.

Suggested integration tests (`egui_kittest` or a headless `Context::run`
harness):

- A `Text` skeleton with no explicit size fills the available width and is
  20 pt tall.
- A `Circle` with `width(48)` occupies a 48 × 48 square and its corner radius
  is 24.
- `Animation::Off` requests no repaint.
- `Animation::Pulse` requests a repaint every frame and its fill alpha varies
  between 0.7 and 1.0 across the loop.
- `Animation::Wave` requests a repaint every frame and the wave mesh's vertices
  move across the loop.
- `fill` and `radius` extensions override the shape's defaults.

```bash
cargo test -p mud_egui_core animation
cargo test -p mud_egui_core skeleton_type
cargo test -p mud_egui_components mud_skeleton
```

---

## Differences from MudBlazor

- **`Animation.False` is renamed to `Animation.Off`.** `False` is a reserved
  keyword in Rust. `Off` reads the same ("no animation") and avoids the
  keyword. The numeric values are documented in the enum but nothing in the
  port depends on them.
- **`Width` / `Height` are `f32` points, not CSS strings.** MudBlazor accepts
  `"300px"` or `"30%"`. egui has no CSS parser; compute a percentage from
  `ui.available_width()` / `ui.available_height()` before calling the builder.
  The defaults are still `None`, matching MudBlazor's `null`.
- **Per-shape defaults are named constants, not CSS defaults.** MudBlazor's
  SCSS gives a skeleton a default height and (for `Text`) rounded corners;
  those come from theme variables there and are hardcoded constants here
  (`DEFAULT_TEXT_HEIGHT`, `TEXT_RADIUS`, …). They are grouped at the top of
  the file so a theme that wants different proportions can adjust them in one
  place.
- **The skeleton fill comes from `ui.visuals()`, not a theme palette entry.**
  MudBlazor uses `--mud-palette-action-default-hover` from its palette; the
  port uses `ui.visuals().widgets.noninteractive.bg_fill`, which plays the
  same role in egui's default `Visuals`. Set `Visuals::widgets` in your theme
  to change it globally, or use the `fill` extension for a one-off override.
- **`fill` and `radius` are extensions.** They are the typed equivalents of
  the `Style` property a Blazor caller would use for the two properties a
  skeleton actually needs to customize. They are documented as extensions, not
  as MudBlazor parameters.
- **`Style` and `UserAttributes` are not ported.** No CSS strings, no HTML
  tag.
- **The wave animation uses a vertex-coloured mesh, not a CSS gradient.** egui
  has no CSS `linear-gradient`; the equivalent is a six-vertex mesh whose
  vertex colours go transparent → highlight → transparent. The visual result
  is the same sweeping shine, and the mesh's three columns come from the same
  `wave_band_x` helper the tests exercise.
- **The pulse animation applies the fade to the whole shape, not to the
  individual colour.** MudBlazor's CSS fades `opacity` on the element, which
  affects the fill and any border together. The port has no border, so
  gamma-multiplying the fill by the pulse factor is equivalent.
- **No `ChildContent`.** `MudSkeleton` in Blazor is a self-closing placeholder;
  there is nothing inside it. The port has no closure parameter, matching the
  component.