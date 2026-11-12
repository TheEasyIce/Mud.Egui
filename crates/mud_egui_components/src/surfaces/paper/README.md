# MudPaper for `mud_egui`

A themed surface for grouping other components, for
[egui](https://github.com/emilk/egui). Ported from MudBlazor's
[`MudPaper`](https://mudblazor.com/components/paper) (MudBlazor 8.13.0).

```rust
let mut paper = MudPaper::new()
    .elevation(4)
    .classes(["p-4", "bg-surface"]);

paper.show(ui, |ui| {
    ui.add(MudText::body1("Hello"));
});
```

`MudPaper` paints no text and no glyphs of its own. Every string inside renders
through **`MudText`**, every glyph through **`MudIcon`**.

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
implementation. `MudPaper` has **exactly nine parameters**, and all nine are
ported.

### Ported parameters

| MudBlazor | Type | Default | Rust mapping |
|---|---|---|---|
| `ChildContent` | `RenderFragment?` | `null` | closure to `show(ui, \|ui\| ..)` |
| `Elevation` | `int` | **`1`** | `elevation(u8)` (clamped 0–24) |
| `Height` | `string?` | `null` | `height(f32)` → `Option<f32>` points |
| `MaxHeight` | `string?` | `null` | `max_height(f32)` |
| `MaxWidth` | `string?` | `null` | `max_width(f32)` |
| `MinHeight` | `string?` | `null` | `min_height(f32)` |
| `MinWidth` | `string?` | `null` | `min_width(f32)` |
| `Outlined` | `bool` | `false` | `outlined(bool)` |
| `Square` | `bool` | `false` | `square(bool)` |
| `Width` | `string?` | `null` | `width(f32)` |
| `Class` | `string?` | `null` | `classes(iter)` — resolved through `flavor` |
| `Tag` | `object?` | `null` | `tag(t)` / `tag_ref::<U>()` |

`Width` / `Height` and their `Min*` / `Max*` siblings are CSS strings in
MudBlazor (`"150px"`, `"30%"`). They become `Option<f32>` points here,
consistent with `MudStack`, `MudCardMedia` and `MudCollapse`. Compute a
percentage from `ui.available_width()` / `available_height()` before calling the
builder.

### Extensions (not MudBlazor parameters)

These four exist in the pre-existing port and are kept for two reasons:
`MudSplitPanel` depends on `transparent`, and a surface with no typed escape
hatch cannot express a one-off background or radius that is not in the class
vocabulary.

| Extension | Type | Default | Purpose |
|---|---|---|---|
| `transparent(b)` | `bool` | `false` | Suppresses fill, shadow and outline. |
| `fill(c)` | `Color32` | `None` | Explicit background; beats `bg-*` utilities. |
| `stroke(s)` | `Stroke` | `None` | Explicit outline; only drawn when `outlined`. |
| `corner_radius(r)` | `CornerRadius` | `None` | Explicit radius; `square` wins over it. |
| `flavor(f)` | `StyleFlavor` | `Bootstrap` | Bootstrap or Tailwind class resolver. |

Every one is marked **extension** in the API reference. None of them is a
MudBlazor parameter.

### Deliberately not ported

| MudBlazor | Reason |
|---|---|
| `Style` | egui has no inline CSS strings. Use `classes(..)` and the `fill` / `stroke` / `corner_radius` extensions for the same purpose. |
| `UserAttributes` | There is no HTML tag to splat attributes onto. |
| `Classname` / `Stylename` | Protected getters that expose the computed CSS class list. There is no equivalent object in egui. |

There is **no** `Rounded` parameter in 8.13.0 — `Square` is the only radius
parameter, and `MudGlobal.Rounded` is the theme-wide default this port reads as
the fallback radius.

---

## Architecture

```
surfaces/paper/
└── mud_paper.rs
    ├── constants        MAX_ELEVATION 24, OUTLINE_STROKE_WIDTH 1,
    │                    REM_SIZE_PT 14, FALLBACK_THEME_RADIUS 4
    ├── pure helpers     paper_stroke, paper_shadow, elevation_shadow,
    │                    apply_radius, theme_radius, radius_is_zero, radius_u8,
    │                    ordered_bounds, clamp_to_bounds, apply_axis_bounds
    ├── MudPaper         builder + show(&mut self, ui, add_contents)
    └── impl Widget      ui.add(MudPaper::new())  (empty surface)
```

**Pure layer.** Outline choice, shadow suppression, radius precedence, bound
ordering and bound clamping are free functions over `bool`, `Option<f32>` and
small `Copy` types. None touch `egui::Context`, so they are unit-tested without
a window.

**Style layer.** The class resolver (`BsStyle` / `TwStyle`) is cached on the
instance and consulted once per frame. `tw_state` is refreshed with the current
dark flag before the frame is built, and with the previous frame's hover and
active flags after it is drawn.

**Component layer.** `show` builds one `egui::Frame`, applies the visual
precedence, applies the layout bounds inside the frame closure, and runs the
caller's content.

---

## API reference

### `MudPaper`

| Method | Type | Default | Mirrors | Description |
|---|---|---|---|---|
| `new()` | `Self` | — | — | Elevation 1, rounded, not outlined. |
| `elevation(e)` | `u8` | `1` | `Elevation` | Shadow depth, clamped to 24. `0` = flat. |
| `outlined(b)` | `bool` | `false` | `Outlined` | 1 pt border; **suppresses the shadow**. |
| `square(b)` | `bool` | `false` | `Square` | Zero corner radius. |
| `width(px)` | `f32` | `None` | `Width` | Explicit width in points. |
| `height(px)` | `f32` | `None` | `Height` | Explicit height in points. |
| `min_width(px)` | `f32` | `None` | `MinWidth` | Lower width bound. |
| `min_height(px)` | `f32` | `None` | `MinHeight` | Lower height bound. |
| `max_width(px)` | `f32` | `None` | `MaxWidth` | Upper width bound. |
| `max_height(px)` | `f32` | `None` | `MaxHeight` | Upper height bound. |
| `transparent(b)` | `bool` | `false` | *(extension)* | Suppresses fill, shadow and outline. |
| `fill(c)` | `Color32` | `None` | *(extension)* | Explicit background. |
| `stroke(s)` | `Stroke` | `None` | *(extension)* | Explicit outline (only when `outlined`). |
| `corner_radius(r)` | `CornerRadius` | `None` | *(extension)* | Explicit radius; `square` wins. |
| `flavor(f)` | `StyleFlavor` | `Bootstrap` | — | Utility-class resolver. |
| `classes(iter)` | `iter of AsRef<str>` | none | `Class` | Resolved into the frame. |
| `tag(t)` / `tag_ref::<U>()` | `T: Any + Send + Sync` | `None` | `Tag` | Arbitrary user data. |
| `show(ui, \|ui\| ..)` | `&mut Ui, FnOnce(&mut Ui) -> R` | — | `ChildContent` | Returns `InnerResponse<R>`. |

`show` takes `&mut self` (not `self`) because it writes the Tailwind hover and
active flags back after drawing. Store the paper on a struct field so those
flags survive between frames.

### Constants

| Name | Value |
|---|---|
| `MAX_ELEVATION` | `24` |
| `OUTLINE_STROKE_WIDTH` | `1.0` |
| `REM_SIZE_PT` | `14.0` |
| `FALLBACK_THEME_RADIUS` | `4.0` |

---

## Examples

### Default card

```rust
let mut card = MudPaper::new();   // elevation 1, rounded, content-sized
card.show(ui, |ui| {
    ui.add(MudText::body2("Item 1"));
});
```

### Elevated, floating surface

```rust
let mut panel = MudPaper::new().elevation(6);
panel.show(ui, |ui| {
    ui.add(MudText::body1("Lifted off the page."));
});
```

### Outlined, no shadow

```rust
let mut panel = MudPaper::new().outlined(true);
panel.show(ui, |ui| {
    ui.add(MudText::body2("Bordered, flat."));
});
```

`outlined(true)` suppresses the shadow even when `elevation` is left at its
default of 1. That is MudBlazor's rule: *"Elevation is disabled when a
component supports and uses the `Outlined` property."*

### Fixed-size card

```rust
let mut card = MudPaper::new()
    .width(320.0)
    .height(180.0)
    .classes(["p-4"]);
card.show(ui, |ui| {
    ui.add(MudText::body2("320 × 180 pt."));
});
```

### Constrained, responsive surface

```rust
let mut panel = MudPaper::new()
    .min_width(200.0)
    .max_width(480.0)
    .min_height(120.0)
    .max_height(400.0);
panel.show(ui, |ui| {
    ui.add(MudText::body2("Resizes between 200–480 wide."));
});
```

### Square corners

```rust
let mut tile = MudPaper::new().square(true);
tile.show(ui, |ui| { ui.label("…"); });
```

### Transparent layout wrapper

```rust
// No fill, shadow or outline — just the sizing/layout half of MudPaper.
let mut wrapper = MudPaper::new().transparent(true).width(400.0);
wrapper.show(ui, |ui| {
    render_split_panel_content(ui);
});
```

This is the form `MudSplitPanel` uses internally.

### Typed fill override

```rust
let mut banner = MudPaper::new()
    .fill(Color32::from_rgb(32, 40, 64))
    .corner_radius(CornerRadius::same(12))
    .classes(["p-6"]);
banner.show(ui, |ui| {
    ui.add(MudText::body1("Custom surface"));
});
```

### Tailwind with hover variants (stored instance)

```rust
struct MyPanel {
    paper: MudPaper,
}

impl MyPanel {
    fn show(&mut self, ui: &mut egui::Ui) {
        self.paper.show(ui, |ui| {
            ui.add(MudText::body1("Hover me"));
        });
    }
}

// In the constructor:
let paper = MudPaper::new()
    .flavor(StyleFlavor::Tailwind)
    .classes(["bg-primary/90", "hover:bg-primary", "rounded-lg", "p-4"]);
```

The `hover:` variant is fed from the previous frame's `response.hovered()`,
which is what `tw_state` exists for. A one-shot `ui.add(MudPaper::new())` cannot
carry that state and will always render the resting variant.

### Tag round-trip

```rust
let p = MudPaper::new().tag("settings-page");
assert_eq!(p.tag_ref::<&str>(), Some(&"settings-page"));
```

---

## Layout and rendering

### Resolution order

Every visual property has one precedence rule:

| Property | Highest → lowest |
|---|---|
| Fill | `transparent` → `fill` → `bg-*` class → theme surface |
| Corner radius | `square` → `corner_radius` → `rounded-*` class → theme radius |
| Outline | none unless `outlined`; then `stroke` → themed border colour |
| Shadow | none if `transparent` or `outlined`; else the elevation ramp |

### Bounds

`width` / `height` and the four `min_*` / `max_*` bounds are applied inside the
frame closure, before the caller's content runs. That means the content sees the
final size through `ui.available_width()` / `available_height()`.

An inverted pair is swapped: `min_width(600)` + `max_width(400)` is treated as
`400..=600`. An explicit `width` is clamped into the resolved bounds.

| Builder calls | Effective |
|---|---|
| `width(320)` only | exactly 320 |
| `min_width(200)` + `max_width(480)` | content size, clamped to 200–480 |
| `width(320)` + `min_width(400)` | 400 |
| `width(320)` + `max_width(200)` | 200 |
| `min_width(600)` + `max_width(400)` | bounds become 400–600 |

### Theme radius fallback

When neither `square` nor `corner_radius` is set **and** the class resolver left
the frame's radius at zero, the theme's `default_border_radius` is applied
(`4.0` by default). This reproduces MudBlazor's `var(--mud-default-borderradius)`.

The heuristic cannot distinguish "no class touched the radius" from an explicit
`rounded-none` class. If you need square corners unambiguously, use
`.square(true)` rather than a `rounded-none` class.

### Why `show` takes `&mut self`

`tw_state` carries `hovered`, `active` and `dark` into the class resolver. The
`dark` flag is refreshed before the frame is built (it is known from
`ui.visuals()`), but `hovered` and `active` are only known **after** the frame
has been drawn, because they depend on the frame's own response. `show`
therefore writes them back at the end of the call, and the next frame's
`hover:` / `active:` variants read them.

This is the same trade-off `MudContainer` and the old `MudPaper` document. A
component that is reconstructed every frame cannot carry the state; store it on
a field.

---

## Performance and memory choices

| Choice | Effect |
|---|---|
| Class resolvers cached on the instance | A stored paper re-parses no classes between frames. |
| One `egui::Frame` per call | One fill, one stroke, one shadow — not one per property. |
| Pure precedence helpers | `paper_stroke`, `paper_shadow`, `apply_radius`, `ordered_bounds` and `clamp_to_bounds` are arithmetic over `Copy` values. |
| Bounds applied before content | Content sees the final size, so no second layout pass is needed. |
| `Shadow::NONE` / `Stroke::NONE` guards | A flat or outlined paper submits no shadow and no border shape. |
| Theme fetched once | One `MudTheme::fetch` and one palette lookup closure, shared by the frame builder and the border colour. |
| No `Vec` or `String` allocation | Beyond what the class resolver and the content itself do. |

Not done, on purpose: caching the resolved `Frame` across frames. `Frame`
contains `Color32` and `Stroke`, both cheap to rebuild, and the class resolver
is the only expensive part — which is already cached on the instance.

---

## Testing

`cargo test -p mud_egui_components mud_paper` runs 17 pure-function tests:

| Area | Tests |
|---|---|
| Shadow | outlined returns `Shadow::NONE`; elevation 0 returns `Shadow::NONE`; elevated scales; clamping at 24 |
| Outline | not outlined returns `None`; custom stroke wins; fallback uses the border colour and 1 pt width |
| Radius | `square` wins over an override; explicit override beats the theme; theme fills a zero radius; a class-set radius is not overwritten; `radius_u8` / `radius_is_zero` sanitize |
| Bounds | `ordered_bounds` passes through and swaps inverted pairs; `clamp_to_bounds` respects both ends and `None` |
| Builders | defaults match MudBlazor (elevation **1**); every field set; tag round-trip |

No test creates an `egui::Context`. `MudPaper::show` needs a live `Ui` (class
resolution, frame painting, theme lookup) and is therefore left to integration
tests; everything it delegates to is covered above.

Suggested integration tests (`egui_kittest` or a headless `Context::run`
harness):

- A paper with `width(320)` and `height(180)` occupies exactly that rectangle.
- `min_width` / `max_width` clamp a content that would otherwise be wider.
- `outlined(true)` draws a border and no shadow.
- `transparent(true)` draws neither.
- A stored paper with a `hover:` Tailwind class picks up the variant on the
  second frame of hover.

```bash
cargo test -p mud_egui_components mud_paper
```

---

## Differences from MudBlazor

- **`Width` / `Height` / `Min*` / `Max*` are `f32` points, not CSS strings.**
  MudBlazor accepts `"150px"`, `"30%"` or any CSS length. egui has no CSS
  parser; compute a percentage from `ui.available_width()` before calling the
  builder. The defaults are still `None`, matching MudBlazor's `null`.
- **`Elevation` is `u8`, clamped to `0..=24`.** MudBlazor accepts `int` and
  relies on CSS classes; the port clamps like every other component in this
  workspace. The default is still `1`.
- **`Elevation` is suppressed by `Outlined`, structurally.** `paper_shadow`
  returns `Shadow::NONE` whenever `outlined` is set. This matches MudBlazor's
  documented rule exactly, rather than relying on CSS cascade order.
- **The theme's `default_border_radius` is a heuristic fallback.** It is applied
  only when no `square`, no `corner_radius` override and no class-set radius is
  present. An explicit `rounded-none` class is overridden by the theme default;
  use `.square(true)` for unambiguous square corners.
- **`Style` and `UserAttributes` are not ported.** No inline CSS strings, no
  HTML tag. `classes(..)` covers the utility-class case; the four typed
  extensions cover the rest.
- **`transparent`, `fill`, `stroke` and `corner_radius` are extensions, not
  MudBlazor parameters.** They are clearly marked in the API reference.
  `transparent` is required by `MudSplitPanel`; the other three are typed
  escape hatches.
- **`show` takes `&mut self`.** The pre-existing port used the same signature
  for the same reason: `tw_state` must persist between frames for Tailwind
  `hover:` / `active:` variants. Every other component that needs no persistent
  state uses `self` or `mut self`.
- **No `Rounded` parameter.** 8.13.0 does not have one; `Square` is the only
  radius parameter, and the theme radius is the fallback.