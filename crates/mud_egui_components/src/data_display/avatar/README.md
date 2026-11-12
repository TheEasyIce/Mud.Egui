# MudAvatar + MudAvatarGroup for egui

Production-grade, MudBlazor-faithful ports of `MudAvatar.razor.cs` and
`MudAvatarGroup.razor.cs` (MudBlazor 8.13.0) for the
[`egui`](https://github.com/emilk/egui) immediate-mode GUI library.

Two small but complete building blocks:

- **`MudAvatar`** — a circular or rounded-square badge for a user photo,
  Material icon, initials, or arbitrary custom content.
- **`MudAvatarGroup`** — a horizontally-overlapping stack of avatars with
  a `+N` overflow badge and shared styling.

Both components ship with full API parity to MudBlazor 8.13.0, zero-clone
construction, unit-tested pure helpers, and idiomatic egui builders.

---

## Table of contents

1. [Architecture](#architecture)
2. [API reference — `MudAvatar`](#api-reference--mudavatar)
3. [API reference — `MudAvatarGroup`](#api-reference--mudavatargroup)
4. [Real-world examples](#real-world-examples)
5. [Breaking changes from earlier versions](#breaking-changes-from-earlier-versions)
6. [Performance & design rationale](#performance--design-rationale)
7. [Testing](#testing)
8. [License](#license)

---

## Architecture

```
mud_avatar.rs
├── MudAvatar<'a>           — single avatar
│   ├── svg / image / text / child   — content slots
│   ├── variant / color / size / square / rounded / elevation
│   ├── ringed / ring_color (internal, used by MudAvatarGroup)
│   └── show(self, ui) -> Response
├── AvatarMetrics           — pure size resolver (S/M/L → diameter + font)
├── elevation_shadow(u8)    — pure elevation → egui Shadow
└── resolve_color(Color)    — palette resolver (Default/Inherit fallback)

mud_avatar_group.rs
├── MudAvatarGroup          — group configuration + render engine
│   └── show(self, ui, |ctx| …) -> Response
├── MudAvatarGroupCtx       — builder context for `ctx.item(AvatarItem)`
├── AvatarItem              — one row entry (Svg or Text + optional Color)
├── AvatarContentInput      — Svg(String) | Text(String)
└── Pure helpers            — overlap_for, avatar_cell_for, row_width_for,
                              visible_count_for, ring_extra_for
```

The group is a **layout calculator**: it allocates one row of
`row_width_for(cell, slots, overlap)` and then paints each avatar at a
computed `x` offset. No per-frame allocations beyond the temporary
`Vec<AvatarItem>` inside `show`.

---

## API reference — `MudAvatar`

| Method | Type | Default | MudBlazor equivalent | Description |
|---|---|---|---|---|
| `new()` | `Self` | — | — | Creates an empty avatar. |
| `svg(s)` | `impl Into<String>` | none | `Svg` | Material-style SVG path (e.g. `filled::PERSON`). |
| `icon(s)` | `impl Into<String>` | none | `Svg` | Deprecated alias for `svg`. |
| `image(src)` | `egui::ImageSource<'a>` | none | `Image` | Raster image (profile photo). |
| `text(t)` | `impl Into<String>` | none | `ChildContent` (text) | Short text (initials). |
| `child(f)` | `FnMut(&mut Ui) + 'a` | none | `ChildContent` | Arbitrary custom content. |
| `variant(v)` | `Variant` | `Filled` | `Variant` | `Filled`, `Outlined`, `Text`. |
| `color(c)` | `Color` | `Default` | `Color` | Theme colour. |
| `size(s)` | `Size` | `Medium` | `Size` | `Small` (24 px), `Medium` (40 px), `Large` (56 px). |
| `rounded(b)` | `bool` | `false` | `Rounded` | 4 px corner radius. |
| `square(b)` | `bool` | `false` | `Square` | Zero corner radius. |
| `elevation(e)` | `u8` | `0` | `Elevation` | Drop shadow (0–24; clamped). |
| `flavor(f)` | `StyleFlavor` | `Bootstrap` | — | Utility-class flavour. |
| `classes(iter)` | `iter of AsRef<str>` | — | `Class` | Bootstrap/Tailwind utility classes. |
| `show(ui)` | `&mut Ui` | — | — | Draws and returns a `Response`. |

**Not exposed (documented parity gaps):** `Alt` (no clean `WidgetInfo`
mapping without `MudImage` support) — see [Notes](#notes-on-parity-gaps).

---

## API reference — `MudAvatarGroup`

| Method | Type | Default | MudBlazor equivalent | Description |
|---|---|---|---|---|
| `new()` | `Self` | — | — | Creates a group. |
| `max(n)` | `usize` | `5` | `Max` | Max visible avatars. `0`/`usize::MAX` = show all. |
| `total(n)` | `usize` | `None` | `Total` | Total people (for the `+N` badge). |
| `spacing(units)` | `u8` | `3` | `Spacing` | Overlap units (each = 4 px). |
| `color(c)` | `Color` | `Default` | `Color` | Shared colour. |
| `size(s)` | `Size` | `Medium` | `Size` | Shared size. |
| `variant(v)` | `Variant` | `Filled` | `Variant` | Shared variant. |
| `elevation(e)` | `u8` | `0` | `Elevation` | Shared elevation. |
| `rounded(b)` | `bool` | `false` | `Rounded` | Shared rounded flag. |
| `square(b)` | `bool` | `false` | `Square` | Shared square flag. |
| `show(ui, |ctx| …)` | `&mut Ui, FnOnce(&mut Ctx) -> R` | — | `ChildContent` | Draws the row. |

### `AvatarItem`

| Method | Type | Default | Description |
|---|---|---|---|
| `text(t)` | `impl Into<String>` | — | Initials item. |
| `svg(s)` | `impl Into<String>` | — | SVG path item. |
| `icon(s)` | `impl Into<String>` | — | Deprecated alias for `svg`. |
| `color(c)` | `Color` | group default | Per-item override. |

### `AvatarContentInput`

```rust
pub enum AvatarContentInput {
    Svg(String),
    Text(String),
}
```

### `Size` mapping

| `Size` | Diameter | Font size |
|---|---|---|
| `Small` | 24 px | 12 pt |
| `Medium` | 40 px | 20 pt |
| `Large` | 56 px | 24 pt |

### Spacing → overlap

| `spacing(n)` | Overlap (px) |
|---|---|
| 0 | 0 |
| 1 | 4 |
| 3 (default) | 12 |
| 6 | 24 |

---

## Real-world examples

### Icon avatar

```rust
use mud_egui_components::MudAvatar;
use mud_egui_icons_material::filled;
use mud_egui_core::enums::Color;

MudAvatar::new()
    .svg(filled::PERSON)
    .color(Color::Primary)
    .show(ui);
```

### Photo avatar

```rust
MudAvatar::new()
    .image(egui::include_image!("assets/user.png"))
    .size(Size::Large)
    .show(ui);
```

### Initials avatar

```rust
MudAvatar::new()
    .text("JD")
    .color(Color::Secondary)
    .variant(Variant::Outlined)
    .elevation(2)
    .show(ui);
```

### Custom content

```rust
MudAvatar::new()
    .child(|ui| {
        ui.label("🚀");
    })
    .size(Size::Large)
    .show(ui);
```

### Avatar group with overflow

```rust
use mud_egui_components::data_display::avatar::mud_avatar_group::{
    AvatarItem, MudAvatarGroup,
};

MudAvatarGroup::new()
    .max(5)
    .total(12)
    .color(Color::Primary)
    .show(ui, |g| {
        g.item(AvatarItem::svg(filled::PERSON));
        g.item(AvatarItem::text("AB"));
        g.item(AvatarItem::svg(filled::STAR));
        g.item(AvatarItem::text("CD").color(Color::Secondary));
        g.item(AvatarItem::svg(filled::HOME));
        // …only 5 shown; a `+7` badge appears.
    });
```

### Outlined group

```rust
MudAvatarGroup::new()
    .variant(Variant::Outlined)
    .spacing(2)
    .show(ui, |g| {
        g.item(AvatarItem::text("A"));
        g.item(AvatarItem::text("B"));
        g.item(AvatarItem::text("C"));
    });
// Each avatar gets a 2 px surface-coloured ring, so cells grow by 4 px.
```

---

## Breaking changes from earlier versions

| Old name | New name | Notes |
|---|---|---|
| `MudAvatar::icon(s)` | `MudAvatar::svg(s)` | Old name kept as `#[deprecated]` alias. |
| `AvatarItem::icon(s)` | `AvatarItem::svg(s)` | Old name kept as `#[deprecated]` alias. |
| `AvatarContentInput::Icon` | `AvatarContentInput::Svg` | Enums cannot carry aliases; update call sites. |
| `MudAvatar::size_px(f32)` | **removed** | MudBlazor only defines `Size::{Small, Medium, Large}`. Use a `Size` bucket or `child()` for pixel-exact control. |
| `MudAvatarGroup::max_display(n)` | `MudAvatarGroup::max(n)` | Default changed from `usize::MAX` to **`5`** (MudBlazor parity). Pass `0` or `usize::MAX` to show all. |

### Rendering bug fixes in this refactor

1. **Avatars painted below the row.** `MudAvatarGroup::show` used to paint
   at `ui.cursor().min` *after* `allocate_exact_size`, which pointed below
   the reserved row. Now it paints at `rect.min`.
2. **Ring clipped.** `draw_avatar_at` allocated only `diameter × diameter`
   while `MudAvatar` (when `ringed`) renders `diameter + 4 × 4`. The group
   now allocates `avatar_cell_for(size, variant)` per cell.
3. **Elevation was a no-op.** `MudAvatar::elevation` was stored but never
   applied. It now feeds `Frame::shadow(elevation_shadow(n))`.

---

## Notes on parity gaps

- **`Alt`** — MudBlazor attaches `Alt` to the `<img>` HTML element for
  screen readers. In egui this would map to `response.widget_info(||
  WidgetInfo::labeled(Role::Image, true, alt))`. The `MudImage` widget does
  not currently accept an `alt` parameter, so `MudAvatar::alt` would be a
  no-op stub. It is therefore **not exposed**. If `MudImage::alt(s)` lands,
  add `alt` to `MudAvatar` and forward it.
- **`Style` (typed inline style)** — MudBlazor exposes a `Style` parameter
  for arbitrary CSS. The Rust port relies on `classes()` and the `bs_style`
  / `tw_style` resolvers, which together cover the same ground without a
  typed escape hatch. Use `.classes(["style-prop:value"])` for one-off
  overrides.

---

## Performance & design rationale

| Technique | Benefit |
|---|---|
| **Zero-clone construction** | `svg()`, `text()`, and `image()` take `impl Into<String>`/`ImageSource`; no intermediate allocation. |
| **Pure helpers** | `AvatarMetrics::resolve`, `elevation_shadow`, `overlap_for`, `avatar_cell_for`, `row_width_for`, `visible_count_for` are all `#[inline]` pure fns. Unit-tested without egui context. |
| **Row-level allocation only** | `MudAvatarGroup` allocates exactly one `Response` for the whole row; each avatar is drawn via `ui.new_child(UiBuilder::new().max_rect(rect))`, which does not advance the parent cursor. |
| **Single-pass overlap** | Positions are computed as `x += cell - overlap` in the render loop — no scratch buffer, no reverse iteration. |
| **Ring-aware cell sizing** | `avatar_cell_for(size, variant)` guarantees the ring extension is always accounted for; no post-hoc correction. |
| **Elevation clamps at 24** | Prevents `i8` overflow in `egui::epaint::Shadow::offset` even for user-supplied values. |

---

## Testing

The source files ship with:

- **`mud_avatar.rs`** — 8 tests covering size metrics, colour resolution,
  elevation clamping and monotonicity, builder defaults, content slots,
  and ring math.
- **`mud_avatar_group.rs`** — 12 tests covering overlap scaling, diameter
  mapping, ring extension, cell width, row width (including negative-guard
  and empty-row cases), visible-count logic (`0`/`usize::MAX`/clamped),
  builder defaults, all-field builders, and the `AvatarItem` /
  `MudAvatarGroupCtx` collector.

Run with:

```bash
cargo test -p mud_egui_components mud_avatar
```

No `egui::Context` is required for any test — every assertion targets a
pure helper.

---

## License

Provided under the same license as the surrounding `mud_egui` project.
See the repository root for details.