# MudImage for `mud_egui`

An image control with **async loading**, **online URL support**, fallback
sources, CSS-like `object-fit` / `object-position`, elevation, and fluid
sizing for [egui](https://github.com/emilk/egui). Ported from MudBlazor's
[`MudImage`](https://mudblazor.com/components/image) (MudBlazor 8.13.0).

```rust
MudImage::new("https://picsum.photos/800/400")
    .fallback_src("file://assets/placeholder.png")
    .alt("Example photo")
    .object_fit(ObjectFit::Cover)
    .elevation(2)
    .show(ui, vec2(400.0, 200.0));
```

---

## Contents

1. [Setup for async / online images](#setup-for-async--online-images)
2. [Design principle: API parity, not API invention](#design-principle-api-parity-not-api-invention)
3. [Architecture](#architecture)
4. [API reference](#api-reference)
5. [Examples](#examples)
6. [Async loading model](#async-loading-model)
7. [Object-fit / object-position](#object-fit--object-position)
8. [Performance and memory choices](#performance--memory-choices)
9. [Testing](#testing)
10. [Differences from MudBlazor](#differences-from-mudblazor)

---

## Setup for async / online images

`MudImage` uses egui's built-in loader pipeline. **No custom fetch code is
required** — `ImageSource::Uri` triggers the async pipeline automatically
when the loaders are installed[reference:6][reference:7]. You must do two things
once at startup:

### 1 — Install the loaders

```rust
// In your eframe `App::new`, or in the `eframe::run_native` creator:
fn new(cc: &eframe::CreationContext<'_>) -> Self {
    egui_extras::install_image_loaders(&cc.egui_ctx);
    Self::default()
}
```

Calling this multiple times is safe — duplicate installs are ignored[reference:8].

### 2 — Enable the Cargo features

```toml
[dependencies]
egui_extras = { version = "0.37", features = ["all_loaders"] }
image = { version = "0.25", features = ["jpeg", "png", "webp"] }
```

| Feature flag | What it enables |
|---|---|
| `egui_extras` `all_loaders` | `file://`, `http(s)://`, and the base image loaders[reference:9] |
| `egui_extras` `http` | `http(s)://` only |
| `egui_extras` `image` | PNG / JPEG / GIF decoding |
| `egui_extras` `svg` | SVG decoding |
| `image` features | The concrete formats (`jpeg`, `png`, `webp`, …) you actually use |

Without `all_loaders`, remote URLs will silently fall through to
`TextureLoadResult::Failed` — `MudImage` will then show the fallback (or the
built-in broken-image error placeholder). There is no runtime error; the image
just won't appear.

---

## Design principle: API parity, not API invention

The MudBlazor 8.13.0 `MudImage` surface was verified against DNDocs before
implementation[reference:10].

### Ported parameters

| MudBlazor | Rust method | Default |
|-----------|-------------|---------|
| `Alt` | `alt(s)` | `None` |
| `Elevation` | `elevation(u8)` | `0` |
| `FallbackSrc` | `fallback_src(s)` | `None` |
| `Fluid` | `fluid(bool)` | `false` |
| `Height` | `height(f32)` | `None` |
| `ObjectFit` | `object_fit(ObjectFit)` | `Fill` |
| `ObjectPosition` | `object_position(ObjectPosition)` | `Center` |
| `Src` | `new(s)` / `src(s)` | required |
| `Width` | `width(f32)` | `None` |
| `Class` | `classes(iter)` | `None` |
| `Style` | `frame(egui::Frame)` | `None` |
| `Tag` | `tag(t)` / `tag_ref::<U>()` | `None` |
| `UserAttributes` | **not ported** (no HTML tag) | — |

### Extensions kept from the original port

| Rust | Description |
|---|---|
| `loading_placeholder(FnMut(&mut Ui, Rect))` | Custom placeholder while the image loads |
| `error_placeholder(FnMut(&mut Ui, Rect))` | Custom placeholder when both sources fail |
| `show_default(ui)` | Alias for `show(ui, ui.available_size())` |

### Deliberately not ported

- **`UserAttributes`** — there is no HTML tag to splat attributes onto.
- **`Style` → `frame(..)`** — egui has no inline CSS strings; the typed frame
  is the equivalent.

### Naviation helpers

`MudImage::new_empty()` creates an image with no `Src` (fallback-only). This
mirrors MudBlazor's ability to render `FallbackSrc` alone.

---

## Architecture

```
mud_image.rs
├── MudImage<'a>                 — configuration + render entry point
│   └── show(ui, size) -> Response
├── Pure helpers:
│   ├── resolve_fit(fluid, object_fit)     — Fluid → Contain fallback
│   ├── compute_fit_size(natural, container, fit)
│   ├── compute_position_offset(scaled, container, position)
│   ├── resolve_alloc_size(available, fallback, w, h, fluid)
│   └── elevation_shadow(elevation)
├── poll_source(..)              — ImageSource::load → ImageLoadPhase
├── paint_loaded(..)             — fit + position + optional clip
├── paint_loading(..)            — spinner or custom
├── paint_error(..)              — broken-image or custom
└── tests                        — 25 pure-function tests

image_phase.rs                   — pub(crate) ImageLoadPhase enum
```

**Pure layer.** Every geometric and sizing decision is a free function,
covered by unit tests without an `egui::Context`.

**Loader layer.** `poll_source` is the only place that talks to the egui
loader system. It clones the source, calls `ImageSource::load(..)`, and maps
`TextureLoadResult` to `ImageLoadPhase`.

**Painter layer.** `paint_loaded` is the only place that calls `egui::Image`.

---

## API reference

### `MudImage<'a>`

| Method | Default | Mirrors | Description |
|--------|---------|---------|-------------|
| `new(src)` | — | `Src` | Primary image source. |
| `new_empty()` | — | — | No `Src` (fallback-only). |
| `src(src)` | `None` | `Src` | Replace the primary source. |
| `fallback_src(src)` | `None` | `FallbackSrc` | Fallback source. |
| `alt(s)` | `None` | `Alt` | Hover tooltip / accessible text. |
| `fluid(bool)` | `false` | `Fluid` | Scale to available width. |
| `object_fit(ObjectFit)` | `None` → `Fill` | `ObjectFit` | `Fill` / `Contain` / `Cover` / `None` / `ScaleDown`. |
| `object_position(ObjectPosition)` | `Center` | `ObjectPosition` | 9-way anchor. |
| `width(f32)` | `None` | `Width` | Explicit width in px. |
| `height(f32)` | `None` | `Height` | Explicit height in px. |
| `elevation(u8)` | `0` | `Elevation` | Drop shadow (0–24). |
| `loading_placeholder(FnMut)` | `None` | *(extension)* | Custom loading UI. |
| `error_placeholder(FnMut)` | `None` | *(extension)* | Custom error UI. |
| `flavor(StyleFlavor)` | default | — | Utility-class resolver. |
| `classes(iter)` | none | `Class` | Wrapper utility classes. |
| `frame(egui::Frame)` | `None` | `Style` | Replace the wrapper frame. |
| `tag(t)` / `tag_ref::<U>()` | `None` | `Tag` | Arbitrary user data. |
| `show(ui, size)` | — | — | Paint with an explicit fallback size. |
| `show_default(ui)` | — | — | Paint with `ui.available_size()`. |

### `ObjectFit` mapping

| Variant | CSS equivalent | Behaviour |
|---------|----------------|-----------|
| `Fill` (default) | `object-fit: fill` | Stretch to fill, distorting aspect |
| `Contain` | `object-fit: contain` | Fit inside, letterbox |
| `Cover` | `object-fit: cover` | Fill the container, crop (clipped) |
| `None` | `object-fit: none` | Natural size, no scaling |
| `ScaleDown` | `object-fit: scale-down` | Natural size, shrink only if it would overflow |

### `ObjectPosition` mapping

| Variant | CSS equivalent |
|---------|----------------|
| `TopLeft` | `object-position: top left` |
| `Top` | `object-position: top` |
| `TopRight` | `object-position: top right` |
| `Left` | `object-position: left` |
| `Center` (default) | `object-position: center` |
| `Right` | `object-position: right` |
| `BottomLeft` | `object-position: bottom left` |
| `Bottom` | `object-position: bottom` |
| `BottomRight` | `object-position: bottom right` |

### Geometry constants

| Name | Value |
|------|-------|
| `MAX_ELEVATION` | `24` |

---

## Examples

### Remote image with fallback

```rust
MudImage::new("https://picsum.photos/800/400")
    .fallback_src("file://assets/placeholder.png")
    .alt("Example photo")
    .show(ui, vec2(400.0, 200.0));
```

If the remote image is still loading, a spinner appears. If it fails, the
local placeholder takes over. If both fail, a broken-image glyph is shown.

### Local image, cover-cropped

```rust
MudImage::new("file://assets/hero.jpg")
    .object_fit(ObjectFit::Cover)
    .width(600.0)
    .height(300.0)
    .show_default(ui);
```

### Fluid banner

```rust
MudImage::new("https://cdn.example.com/banner.png")
    .fluid(true)
    .object_fit(ObjectFit::Contain)
    .show(ui, vec2(0.0, 200.0)); // height is the placeholder
```

`Fluid` overrides `Width` and `Height`; the width becomes
`ui.available_width()` and the height is a placeholder that the image scales
within.

### Custom loading / error placeholders

```rust
MudImage::new("https://picsum.photos/400/200")
    .loading_placeholder(|ui, rect| {
        ui.add(MudText::new("Loading…").typo(Typo::Caption));
    })
    .error_placeholder(|ui, rect| {
        ui.add(MudButton::new("Retry").size(Size::Small));
    })
    .show(ui, vec2(300.0, 150.0));
```

### Elevation shadow

```rust
MudImage::new("file://assets/avatar.png")
    .elevation(4)
    .object_fit(ObjectFit::Contain)
    .show(ui, vec2(80.0, 80.0));
```

### Object-position demo

```rust
// A wide image inside a square container, anchored to the top-left.
MudImage::new("file://assets/logo-wide.png")
    .object_fit(ObjectFit::Contain)
    .object_position(ObjectPosition::TopLeft)
    .show(ui, vec2(200.0, 200.0));
```

### Fallback-only (no primary)

```rust
MudImage::new_empty()
    .fallback_src("file://assets/empty.png")
    .show(ui, vec2(120.0, 120.0));
```

### Tag round-trip

```rust
let img = MudImage::new("https://picsum.photos/seed/1/200/200")
    .tag("hero");
assert_eq!(img.tag_ref::<&str>(), Some(&"hero"));
img.show(ui, vec2(200.0, 200.0));
```

---

## Async loading model

```
┌──────────────────────────────────────────────────────────┐
│ MudImage::show                                            │
│                                                            │
│  1. poll_source(primary)                                   │
│     └── ImageSource::load(..) → TextureLoadResult         │
│         ├── Ready   → paint_loaded(primary)               │
│         ├── Pending → paint_loading + request_repaint     │
│         └── Failed  → poll_source(fallback)               │
│                        ├── Ready   → paint_loaded(fallback)
│                        ├── Pending → paint_loading + repaint
│                        └── Failed  → paint_error           │
│  2. alt tooltip                                            │
│  3. return Response                                        │
└──────────────────────────────────────────────────────────┘
```

### Why poll with `ImageSource::load` instead of `egui::Image` directly?

`egui::Image::new(source)` already triggers the load internally, but it does
**not** expose the load state to the caller. To route the *fallback* on
failure, `MudImage` needs to know whether the primary failed, not just whether
it's "not painted yet". `ImageSource::load` returns `TextureLoadResult` which
distinguishes `Pending` from `Failed`[reference:11].

Polling is idempotent: the loader caches its result, so `poll_source` followed
by `paint_loaded` (which calls `load` again internally via `egui::Image`) is
a cache hit, not a second fetch.

### Pending → `request_repaint`

When the phase is `Pending`, `MudImage` calls `ui.ctx().request_repaint()` so
the next frame re-polls. When the loader completes, the phase becomes `Ready`
and the image paints on the following frame.

### Retry on hover?

The original README draft mentioned a retry mechanism; it is deliberately
**not** implemented. egui does not expose a per-URI "forget cache" API in a
stable form, and re-fetching on every hover would be a DoS pattern. If you
need to force a reload, change the URI (e.g. append a cache-busting query
string) — that is exactly what MudBlazor's `NoCache` discussion recommends
for HTML[reference:12], and it works here too.

---

## Object-fit / object-position

### Fit resolution

```
natural = texture.size          // from the loader's Ready payload
container = allocated rect size
scaled = compute_fit_size(natural, container, fit)
offset = compute_position_offset(scaled, container, position)
paint_rect = Rect::from_min_size(container.min + offset, scaled)
```

For `Fill`, `scaled == container` so `offset == (0, 0)`.
For `Cover`, `scaled > container` on one axis so `offset < 0`; the scope is
clipped to `container` so the overflow is invisible.

### Position semantics

| Position | Horizontal fraction | Vertical fraction |
|----------|---------------------|-------------------|
| `TopLeft` | 0.0 | 0.0 |
| `Top` | 0.5 | 0.0 |
| `TopRight` | 1.0 | 0.0 |
| `Left` | 0.0 | 0.5 |
| `Center` | 0.5 | 0.5 |
| `Right` | 1.0 | 0.5 |
| `BottomLeft` | 0.0 | 1.0 |
| `Bottom` | 0.5 | 1.0 |
| `BottomRight` | 1.0 | 1.0 |

`offset = (container - scaled) * fraction`. When `scaled < container`, the
offset is positive (letterboxing). When `scaled > container` (Cover), the
offset is negative (crop).

---

## Performance and memory choices

| Choice | Effect |
|---|---|
| `poll_source` clones the `ImageSource` before calling `load` | `ImageSource: Clone` is a cheap pointer/Cow copy. |
| Loader results are cached by egui | `poll_source` + `paint_loaded` is one loader call, not two. |
| `paint_loaded` reads `tex.size` from `Ready(SizedTexture)` | No separate `texture_size()` round-trip. |
| `Cover` uses `ui.scope` with `set_clip_rect` | One scope per clipped image; no per-pixel work. |
| Pure helpers operate on `Vec2` by value | No heap, no locking. |
| `elevation_shadow` clamps at 24 | Prevents `i8` overflow in `Shadow::offset`. |
| All text through `MudText`, icons through `MudIcon` | Theme/font updates propagate automatically. |
| `Arc<dyn Any + Send + Sync>` for `Tag` | Thread-safe; pointer copy. |
| `loading_placeholder` / `error_placeholder` are `Box<dyn FnMut>` | Zero cost when unused. |

Not done, on purpose: a per-URI retry cache. The loader already caches
successes and failures; a second cache would duplicate that with worse
invalidation rules.

---

## Testing

`cargo test` runs 25 pure-function tests:

| Area | Tests |
|---|---|
| `resolve_fit` | explicit override wins, defaults to `Fill` without `Fluid`, defaults to `Contain` with `Fluid` |
| `compute_fit_size` | Fill, Contain (both constrained axes), Cover, None, ScaleDown (both directions), NaN repair, zero-size repair |
| `compute_position_offset` | Center, TopLeft, BottomRight, Top, Right, negative offset for Cover |
| `resolve_alloc_size` | width/height override, fallback to `size`, `Fluid` uses available width, NaN repair |
| `elevation_shadow` | zero is no shadow, clamp above 24, monotonic scale |
| Builders | defaults match MudBlazor, every field set, negative width/height clamp, tag round-trip |
| `ImageLoadPhase` | `Pending.needs_repaint`, `Ready.is_ready` |

Rendering (`show`) needs a live `egui::Context` and is not unit-tested; the
pure layer is the source of truth.

```bash
cargo test -p mud_egui_components mud_image
```

---

## Differences from MudBlazor

- **`UserAttributes` not ported.** There is no HTML tag.
- **`Style` → `frame(..)`.** egui has no inline CSS strings; use the typed
  frame override or `classes(..)`.
- **`Cover` is clipped manually.** egui's `Image` widget does not clip; the
  port wraps it in `ui.scope(|ui| ui.set_clip_rect(container))` so overflow
  is hidden.
- **`Fluid` allocates `available_width × size.y`.** MudBlazor's `Fluid` lets
  the browser compute `height: auto` from the image's natural aspect ratio.
  In immediate-mode egui the allocation must be decided *before* the image
  loads, so the height is a placeholder; the image scales within it using
  `object-fit`. Pass a sensible `size` to `show` (or set `height`) as the
  placeholder.
- **Failure detection uses `TextureLoadResult::Failed`.** MudBlazor relies on
  the browser's `<img onerror>` event; egui's loader returns `Failed` after
  the HTTP status / decode error[reference:13][reference:14].
- **No `NoCache` parameter.** The MudBlazor `NoCache` proposal (#12681) works
  by appending a cache-busting query string in HTML[reference:15]; the same
  technique works here — pass a URL with `?v=<timestamp>` when you need a
  fresh fetch.