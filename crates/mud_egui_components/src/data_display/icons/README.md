# MudIcon for `mud_egui`

A themed icon rendered from an SVG path or a text glyph for
[egui](https://github.com/emilk/egui), ported from MudBlazor's
[`MudIcon`](https://mudblazor.com/components/icon) (MudBlazor 8.13.0).
Features two-layer global caching (tessellated geometry + transformed mesh),
CSS-like utility classes with Bootstrap and Tailwind flavours, rotation /
flip transforms, and edge-triggered interaction callbacks.

```rust
ui.add(MudIcon::new(icons::STAR).color(Color::Warning).size_preset(Size::Large));
```

Every glyph is drawn with the active theme's text colour by default; use
`.color(..)` or `.custom_color(..)` to override.

---

## Contents

1. [Design principle: API parity, not API invention](#design-principle-api-parity-not-api-invention)
2. [Architecture](#architecture)
3. [API reference](#api-reference)
4. [Examples](#examples)
5. [Caching strategy](#caching-strategy)
6. [Performance and memory choices](#performance-and-memory-choices)
7. [Testing](#testing)
8. [Differences from MudBlazor](#differences-from-mudblazor)

---

## Design principle: API parity, not API invention

The MudBlazor 8.13.0 `MudIcon` surface was verified against DNDocs before
this refactor.

### Ported parameters

| MudBlazor | Rust method | Default |
|-----------|-------------|---------|
| `ChildContent` | **not ported** (see below) | — |
| `Color` | `color(Color)` | `Color::Inherit` |
| `Disabled` | `disabled(bool)` | `false` |
| `Icon` | `new(s)` / `icon(s)` | required |
| `Size` | `size_preset(Size)` | `Size::Medium` |
| `Title` | `title(s)` | `None` |
| `ViewBox` | `viewbox_size(f32)` | `0.0` (use bounds) |
| `Class` | `classes(iter)` | `None` |
| `Style` | *(use `classes`)* | — |
| `Tag` | **not ported** (no HTML tag) | — |
| `UserAttributes` | **not ported** (no HTML tag) | — |

### Extensions (not in MudBlazor)

| Extension | Description |
|-----------|-------------|
| `kind(IconKind)` | `SvgPath` (default) or `Glyph` (text character). |
| `size(u8)` | Index `0..=4` — maps 1:1 to `Size` variants. Kept for backward compatibility. |
| `size_px(f32)` | Explicit pixel size (bypasses the index table). |
| `custom_color(Color32)` | Explicit colour override (takes precedence over `Color`). |
| `rotation(f32)` | Clockwise rotation in degrees. |
| `flip(IconFlip)` | `None`, `Horizontal`, `Vertical`, `Both`. |
| `id(Id)` | Stable identity for egui memory. |
| `aria_label(s)` | Accessible label (falls back to `title`). |
| `on_hover` / `on_unhover` / `on_click` / `on_focus` / `on_blur` | Edge-triggered callbacks. |

### Deliberately not ported

- **`ChildContent`** — MudBlazor allows arbitrary Razor markup inside the
  icon element. The idiomatic egui equivalent is to put the custom content
  in the parent (a `Frame` + the icon, or a custom widget). Adding a
  lifetime-bound `FnMut(&mut Ui)` field here would force a lifetime on
  `MudIcon` and complicate the builder for a rarely-used hook.
- **`ViewBox` as a string** — MudBlazor accepts `"0 0 24 24"`. egui's icon
  geometry is square, so the port takes a single `f32` side length. `0.0`
  means "use the geometry's own bounds".
- **`Style`** — egui has no inline CSS strings. Use `classes(..)`.
- **`Tag` / `UserAttributes`** — there is no HTML tag to attach them to.

### `Size` mapping

| MudBlazor `Size` | `size(u8)` index | Pixels |
|---|---|---|
| `ExtraSmall` | 0 | 8 |
| `Small` | 1 | 16 |
| `Medium` (default) | 2 | 24 |
| `Large` | 3 | 32 |
| `ExtraLarge` | 4 | 48 |

`size_preset(Size::Large)` is the parity entry point; `size(3)` is the
extension form. Both set the same field.

---

## Architecture

```
mud_icon.rs
├── ICON_SIZES_PX           — index → pixel table
├── size_to_pixels(u8)      — pure
├── size_preset_to_index(Size) — pure
├── PATH_CACHE              — global, path string → RawGeometry
│   └── tessellate_icon(..) — lyon parse + fill tessellation
│   └── boundary_edges_of(..) — silhouette extraction for AA feather
├── TRANSFORM_CACHE         — global, TransformKey → TransformedLocal
│   └── get_transformed_local(..) — scale / flip / rotate / colour / feather
├── InteractionState        — hovered / active / focused / dark / disabled
├── MudIcon                 — builder + rendering
│   ├── show(self, ui)      — stateless (ui.add compatible)
│   ├── show_mut(&mut self) — stateful (persists hover / active)
│   ├── resolve_frame(..)   — BsStyle / TwStyle resolution
│   ├── draw_path_at(..)    — SVG mesh
│   ├── draw_glyph_at(..)   — text glyph
│   ├── resolve_color(..)   — palette + disabled opacity
│   └── dispatch_events(..) — edge-triggered callbacks
└── mod tests               — 20+ pure-function tests
```

**Pure layer.** `size_to_pixels` and `size_preset_to_index` are the only
pure helpers used by the builder; every other helper is either a global
cache function or a rendering method.

**Cache layer.** `PATH_CACHE` and `TRANSFORM_CACHE` are `LazyLock<RwLock<…>>`
globals. Both use `Arc` internally so cloning a hit is a pointer copy.

**Component layer.** `show_mut` is the entry point. It resolves the frame,
allocates the rect, paints the background, paints the icon, and dispatches
callbacks.

---

## API reference

### `MudIcon`

| Method | Type | Default | Mirrors | Description |
|--------|------|---------|---------|-------------|
| `new(icon)` | `impl Into<String>` | — | `Icon` | Create an SVG-path icon. |
| `icon` | *(field)* | — | `Icon` | SVG path string. |
| `kind(IconKind)` | `IconKind` | `SvgPath` | *(extension)* | SVG path or glyph. |
| `size(u8)` | `u8` | `2` | *(extension)* | Index 0..=4. |
| `size_preset(Size)` | `Size` | `Medium` | `Size` | MudBlazor parity. |
| `size_px(f32)` | `f32` | `None` | *(extension)* | Explicit pixels. |
| `color(Color)` | `Color` | `Inherit` | `Color` | Semantic colour. |
| `custom_color(Color32)` | `Color32` | `None` | *(extension)* | Explicit override. |
| `rotation(f32)` | `f32` | `0.0` | *(extension)* | Degrees clockwise. |
| `flip(IconFlip)` | `IconFlip` | `None` | *(extension)* | Mirror. |
| `disabled(bool)` | `bool` | `false` | `Disabled` | Reduced opacity, no interaction. |
| `viewbox_size(f32)` | `f32` | `0.0` | `ViewBox` | Side length; 0 = use bounds. |
| `title(s)` | `impl Into<String>` | `None` | `Title` | Tooltip. |
| `id(Id)` | `Id` | `None` | *(extension)* | Stable egui identity. |
| `aria_label(s)` | `impl Into<String>` | `None` | *(extension)* | Accessible label. |
| `flavor(StyleFlavor)` | `StyleFlavor` | `Bootstrap` | — | Utility resolver. |
| `classes(iter)` | `iter of AsRef<str>` | none | `Class` | Utility classes. |
| `on_hover(f)` | `FnMut() + Send + Sync` | `None` | *(extension)* | Pointer enters. |
| `on_unhover(f)` | `FnMut() + Send + Sync` | `None` | *(extension)* | Pointer leaves. |
| `on_click(f)` | `FnMut() + Send + Sync` | `None` | *(extension)* | Primary click. |
| `on_focus(f)` | `FnMut() + Send + Sync` | `None` | *(extension)* | Gains focus. |
| `on_blur(f)` | `FnMut() + Send + Sync` | `None` | *(extension)* | Loses focus. |
| `show(ui)` | `&mut Ui` | — | — | Stateless render. |
| `show_mut(ui)` | `&mut Ui` | — | — | Stateful render. |

### `IconKind`

| Variant | Description |
|---------|-------------|
| `SvgPath` (default) | Parse + tessellate the string as an SVG path. |
| `Glyph` | Render the string as a text glyph with the proportional font. |

### `IconFlip`

| Variant | Description |
|---------|-------------|
| `None` (default) | No flip. |
| `Horizontal` | Mirror on the X axis. |
| `Vertical` | Mirror on the Y axis. |
| `Both` | Mirror on both axes. |

### `InteractionState`

| Field | Type | Description |
|-------|------|-------------|
| `hovered` | `bool` | Pointer is over the icon. |
| `active` | `bool` | Pointer button is down on the icon. |
| `focused` | `bool` | Icon has keyboard focus. |
| `dark` | `bool` | Dark mode is active. |
| `disabled` | `bool` | Icon is disabled. |

`to_tw_state()` converts this to the `TwState` expected by `TwStyle`.

### Cache introspection

| Function | Description |
|----------|-------------|
| `path_cache_len()` | Number of tessellated geometries. |
| `transform_cache_len()` | Number of transformed meshes. |
| `clear_caches()` | Drop both caches (tests / memory pressure only). |

---

## Examples

### Basic SVG icon

```rust
use mud_egui_icons_material::filled;
use mud_egui_core::enums::{Color, Size};

ui.add(MudIcon::new(filled::STAR).color(Color::Warning).size_preset(Size::Large));
```

### Explicit pixel size

```rust
ui.add(MudIcon::new(filled::SETTINGS).size_px(18.0));
```

### Text glyph icon

```rust
ui.add(
    MudIcon::new("★")
        .kind(IconKind::Glyph)
        .color(Color::Warning)
        .size_px(20.0),
);
```

### Rotation and flip

```rust
ui.add(
    MudIcon::new(filled::ARROW_FORWARD)
        .rotation(90.0)
        .size_preset(Size::Medium),
);
ui.add(MudIcon::new(filled::PERSON).flip(IconFlip::Horizontal));
```

### Disabled state

```rust
ui.add(MudIcon::new(filled::DELETE).disabled(true));
```

### Custom colour override

```rust
ui.add(MudIcon::new(filled::WARNING).custom_color(Color32::from_rgb(255, 140, 0)));
```

### Utility classes (Bootstrap)

```rust
ui.add(
    MudIcon::new(filled::STAR)
        .classes(["bg-primary", "rounded-circle", "p-1"]),
);
```

### Tailwind with hover variant (stateful)

```rust
// In your struct:
struct MyPage {
    settings_icon: MudIcon,
}

impl MyPage {
    fn new() -> Self {
        Self {
            settings_icon: MudIcon::new(filled::SETTINGS)
                .flavor(StyleFlavor::Tailwind)
                .classes(["bg-primary/20", "rounded-lg", "p-2", "hover:bg-primary/40"]),
        }
    }

    fn show(&mut self, ui: &mut egui::Ui) {
        self.settings_icon.show_mut(ui);
    }
}
```

### Callbacks

```rust
ui.add(
    MudIcon::new(filled::CLOSE)
        .on_click(|| println!("closed"))
        .on_hover(|| println!("hovered")),
);
```

### Accessibility

```rust
ui.add(
    MudIcon::new(filled::INFO)
        .title("Information")
        .aria_label("Show details"),
);
```

---

## Caching strategy

### Layer 1 — `PATH_CACHE`

Keyed by the SVG path string. Stores the raw tessellated geometry (positions
in the path's native coordinate space, triangle indices, bounds, and the
silhouette boundary edges used for antialiasing). Tessellation uses `lyon`
with `FillOptions::tolerance(0.01)`.

The cache is a `LazyLock<RwLock<HashMap<Arc<str>, Arc<OnceLock<Option<Arc<RawGeometry>>>>>>>`.
The `OnceLock` ensures a given path is tessellated **at most once**, even
under concurrent readers.

### Layer 2 — `TRANSFORM_CACHE`

Keyed by `(geometry_id, extent, rotation_bucket, flip, color, feather_bucket)`.
Stores the mesh **after** scale / flip / rotate / colour have been applied,
still in icon-local space (origin at top-left of the icon extent).

The key:
- `extent` is `f32::to_bits` so the key is `Eq + Hash`.
- `rotation_bucket` quantises to half-degree buckets (720 per circle).
- `feather_bucket` quantises to 1/20th-pixel buckets, so DPI changes don't
  poison the cache.

The cache is soft-capped at 4096 entries; on overflow it clears entirely
(next frames repopulate the hot set).

### Hot path

On a double cache hit:
1. Build `TransformKey` (a few integer ops).
2. Look up `TransformedLocal` (one `RwLock::read`).
3. Clone the `Mesh` (one `Vec<Vertex>` + one `Vec<u32>`).
4. Translate every vertex by `rect.min` (one pass over `vertices`).
5. `ui.painter().add(Shape::mesh(..))`.

There is **no** per-frame tessellation, no per-frame scale/flip/rotate, and
no per-frame colour multiplication when the parameters match a cached entry.

---

## Performance and memory choices

| Choice | Effect |
|--------|--------|
| Global `PATH_CACHE` keyed by path string | One tessellation per unique icon, across the whole app. |
| Global `TRANSFORM_CACHE` keyed by visual params | One transform per (icon, size, colour, rotation, flip). |
| `Arc<RawGeometry>` / `Arc<TransformedLocal>` | Cache hits are pointer copies. |
| `OnceLock` inside `PATH_CACHE` | A path is tessellated at most once, even under concurrent readers. |
| Rotation quantised to half-degree buckets | Prevents unbounded cache growth from continuous rotation. |
| Feather quantised to 1/20th-pixel buckets | DPI changes don't invalidate the cache. |
| `LazyLock` for global caches | Zero-cost initialisation; no `Option` check on the hot path. |
| `Arc<Vec<u32>>` for shared indices | Two icons with the same path share the index buffer. |
| Antialiasing via boundary-edge strips | Single-pixel feather without egui's tessellator. |
| Soft cap + clear | Bounded memory; no LRU bookkeeping. |

---

## Testing

`cargo test` runs 20+ pure-function tests:

| Area | Tests |
|------|-------|
| Size table | every index → pixels, out-of-range → 24, `size_preset` → index |
| `size_px` | overrides index, negative / NaN / infinity / zero → `None` |
| Defaults | every field matches MudBlazor |
| Builders | `flavor`, `classes`, full builder chain |
| `InteractionState` | `to_tw_state` respects disabled |
| Geometry cache | tessellation succeeds, same path hits the same `Arc`, distinct paths hit distinct `Arc`s |
| Transform cache | same params hit the same `Arc`, different colour → distinct, rotation quantisation |
| Colour | disabled applies `DISABLED_OPACITY` |
| StyleFlavor | default is Bootstrap, `REM_SIZE_PT` is 14 |

Rendering (`show_mut`) needs a live `egui::Context` and is not unit-tested;
the pure layer and the caches are the source of truth.

```bash
cargo test -p mud_egui_components mud_icon
```

---

## Differences from MudBlazor

- **`ChildContent` not ported.** See the parity section.
- **`ViewBox` is a `f32` side length, not a string.** egui icon geometry is
  square.
- **`Size` uses the `u8` index as an extension.** `size_preset(Size)` is the
  parity entry point.
- **`Style` → `classes(..)`.** egui has no inline CSS strings.
- **`Tag` / `UserAttributes` not ported.** There is no HTML tag.
- **`IconKind`, `IconFlip`, `custom_color`, `rotation`, `flip`, `size_px`,
  `id`, `aria_label`, and the five callbacks are extensions.**
- **`Disabled` uses `gamma_multiply(0.38)`**, matching Material Design's
  disabled opacity. MudBlazor uses a dedicated disabled colour token; the
  visual result is equivalent.
- **`Color::Inherit` resolves to `ui.visuals().text_color()`**, so the icon
  picks up the surrounding text colour automatically.