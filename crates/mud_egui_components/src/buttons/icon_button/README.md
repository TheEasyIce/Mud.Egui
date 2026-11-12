# MudIconButton for `mud_egui`

A strictly icon-only button, ported from MudBlazor's
[`MudIconButton`](https://mudblazor.com/components/button) (MudBlazor 8.13.0).

```rust
ui.add(
    MudIconButton::new(filled::SETTINGS)
        .variant(Variant::Filled)
        .color(Color::Primary)
        .size(Size::Medium),
);
```

The icon goes through **`MudIcon`** (via `MudButton`); no text is ever drawn.

---

## Contents

1. [Design principle](#design-principle)
2. [Architecture](#architecture)
3. [API reference](#api-reference)
4. [Examples](#examples)
5. [Layout](#layout)
6. [Performance and memory choices](#performance-and-memory-choices)
7. [Testing](#testing)
8. [Differences from MudBlazor](#differences-from-mudblazor)

---

## Design principle

Verified against MudBlazor 8.13.0 DNDocs.

| MudBlazor | Rust mapping | Default |
|---|---|---|
| `Icon` | `new(impl Into<String>)` | required |
| `Variant` | `variant(Variant)` | `Filled` |
| `Color` | `color(Color)` | `Primary` |
| `IconColor` | `icon_color(Color)` | `Color::Default` |
| `Size` | `size(Size)` | `Medium` |
| `Disabled` | `disabled(bool)` | `false` |
| `DropShadow` | `drop_shadow(bool)` | `true` |
| `Class` (base) | `classes(iter)` | `None` |
| `OnClick` | `Response::clicked()` | — |
| `Href` / `Target` | **not ported** — use `MudLink` | — |
| `OnClickStopPropagation` | **not ported** | — |
| `Style` / `Tag` / `UserAttributes` | **not ported** | — |

No public enums are introduced.

---

## Architecture

```
buttons/icon_button/
├── mud_icon_button.rs   MudIconButton + into_button() helper + tests
└── mod.rs
```

`MudIconButton` is a thin wrapper: `.into_button()` builds a `MudButton` with
the resolved tint, and `show` forces a `dim × dim` square. All styling comes
from `MudButton`.

`resolve_tint` mirrors MudBlazor's `ApplyIconTint`:
- `icon_color` non-`Default` → use it
- `Text` / `Outlined` → the button's own colour
- `Filled` → `Color::Inherit` (contrast ink from the fill)

---

## API reference

| Method | Type | Default | Description |
|---|---|---|---|
| `new(icon)` | `impl Into<String>` | — | Icon path. |
| `variant(Variant)` | `Variant` | `Filled` | Variant. |
| `color(Color)` | `Color` | `Primary` | Semantic colour. |
| `icon_color(Color)` | `Color` | `Default` | Icon colour override; `Default` = compute from variant. |
| `size(Size)` | `Size` | `Medium` | Size preset — drives the square footprint. |
| `disabled(bool)` | `bool` | `false` | Disabled state. |
| `drop_shadow(bool)` | `bool` | `true` | Shadow on Filled. |
| `classes(iter)` | `iter of AsRef<str>` | none | Stored. |
| `show(ui)` | `&mut Ui` | — | Returns `Response`. |

### Size → footprint

| Size | Footprint |
|---|---|
| `ExtraSmall` | 32 × 32 |
| `Small` | 40 × 40 |
| `Medium` | 48 × 48 |
| `Large` | 56 × 56 |
| `ExtraLarge` | 64 × 64 |

---

## Examples

### Filled default

```rust
ui.add(MudIconButton::new(filled::SETTINGS));
```

### Outlined with a custom tint

```rust
ui.add(
    MudIconButton::new(filled::EDIT)
        .variant(Variant::Outlined)
        .color(Color::Primary)
        .icon_color(Color::Warning),
);
```

### Text-style

```rust
ui.add(
    MudIconButton::new(filled::MORE_VERT)
        .variant(Variant::Text)
        .color(Color::Inherit),
);
```

### Disabled

```rust
ui.add(MudIconButton::new(filled::DELETE).disabled(true));
```

---

## Layout

`show` forces `dim × dim` via `interact_size.y = dim` and `add_sized(dim, dim)`.
The button's own `full_width(true)` fills that square, and the icon is
centred inside.

---

## Performance and memory choices

| Choice | Effect |
|---|---|
| `into_button()` builds a `MudButton` | One allocation path; no icon-only logic duplicated. |
| `ui.scope(..)` for `interact_size` | Parent spacing not touched. |
| `MudIcon` reuse | Free thanks to its transform cache. |

---

## Testing

`cargo test -p mud_egui_components mud_icon_button`:

- defaults
- builder chain
- `resolve_dimension` for every `Size`
- `resolve_tint` default (per variant) and override

---

## Differences from MudBlazor

- **`Href` / `Target` / `OnClickStopPropagation` not ported.** Same reasons as
  `MudButton`.
- **`Style` / `Tag` / `UserAttributes` not ported.**
- **No `ChildContent`.** Icon-only by construction (no `.label(..)`).