# MudToggleIconButton for `mud_egui`

An icon button that switches between two visual states when clicked, ported
from MudBlazor's
[`MudToggleIconButton`](https://mudblazor.com/components/button) (MudBlazor
8.13.0).

```rust
let mut is_favorite = false;
ui.add(
    MudToggleIconButton::new(&mut is_favorite, filled::FAVORITE_BORDER, filled::FAVORITE)
        .color(Color::Default)
        .toggled_color(Color::Error)
        .title("Add to favourites")
);
```

The toggle state is **caller-owned** (`&mut bool`), matching the pattern used
by every other component in this workspace.

---

## Contents

1. [Design principle](#design-principle)
2. [Architecture](#architecture)
3. [API reference](#api-reference)
4. [Examples](#examples)
5. [Toggle resolution](#toggle-resolution)
6. [Performance and memory choices](#performance-and-memory-choices)
7. [Testing](#testing)
8. [Differences from MudBlazor](#differences-from-mudblazor)

---

## Design principle

| MudBlazor | Rust mapping | Default |
|---|---|---|
| `Icon` | `new(_, icon, _)` | required |
| `ToggledIcon` | `toggled_icon(impl Into<String>)` | required (2nd arg) |
| `Color` | `color(Color)` | `Default` |
| `ToggledColor` | `toggled_color(Color)` | `None` (keeps `color`) |
| `Size` | `size(Size)` | `Medium` |
| `ToggledSize` | `toggled_size(Size)` | `None` |
| `Variant` | `variant(Variant)` | `Text` |
| `ToggledVariant` | `toggled_variant(Variant)` | `None` |
| `IconColor` | `icon_color(Color)` | `None` |
| `ToggledIconColor` | `toggled_icon_color(Color)` | `None` |
| `Title` | `title(impl Into<String>)` | `None` |
| `ToggledTitle` | `toggled_title(impl Into<String>)` | `None` |
| `Disabled` | `disabled(bool)` | `false` |
| `Class` (base) | `classes(iter)` | `None` |
| `Style` / `Tag` / `UserAttributes` | **not ported** | — |

### Extension

`label(impl Into<String>)` shows text beside the icon button — not a
`MudBlazor` parameter (MudBlazor's `MudToggleIconButton` is icon-only).

No public enums are introduced.

---

## Architecture

```
buttons/toggle_icon_button/
├── mud_toggle_icon_button.rs   MudToggleIconButton + ToggleVisualState + tests
└── mod.rs
```

`show` resolves the current visual state (icon + colour + size + variant +
title) from the toggle flag, builds a `MudButton` with those values, renders
it horizontally with an optional `MudText` label, and flips the flag on click.

The `ToggleVisualState` struct is a plain value: it is not an enum, so Rule 2
does not apply.

---

## API reference

| Method | Type | Default | Description |
|---|---|---|---|
| `new(toggled, icon, toggled_icon)` | `(&mut bool, impl Into<String>, impl Into<String>)` | — | Constructor. |
| `color` / `toggled_color` | `Color` | `Default` / `None` | Normal / toggled colours. |
| `size` / `toggled_size` | `Size` | `Medium` / `None` | Normal / toggled sizes. |
| `variant` / `toggled_variant` | `Variant` | `Text` / `None` | Normal / toggled variants. |
| `icon_color` / `toggled_icon_color` | `Color` | `None` / `None` | Icon colour overrides. |
| `title` / `toggled_title` | `impl Into<String>` | `None` | Tooltip. |
| `disabled(bool)` | `bool` | `false` | Disabled state. |
| `label(impl Into<String>)` | `String` | `None` | Text beside the button (extension). |
| `classes(iter)` | `iter of AsRef<str>` | none | Stored. |
| `show(ui)` / `ui.add(..)` | — | — | Renders, flips flag on click. |

---

## Examples

### Favourite toggle

```rust
let mut fav = false;
ui.add(
    MudToggleIconButton::new(&mut fav, filled::FAVORITE_BORDER, filled::FAVORITE)
        .toggled_color(Color::Error),
);
```

### Different variant when toggled

```rust
ui.add(
    MudToggleIconButton::new(&mut on, filled::PLAY_ARROW, filled::PAUSE)
        .variant(Variant::Text)
        .toggled_variant(Variant::Filled)
        .color(Color::Primary)
        .toggled_color(Color::Error),
);
```

### With a text label

```rust
ui.add(
    MudToggleIconButton::new(&mut muted, filled::VOLUME_OFF, filled::VOLUME_UP)
        .label("Mute"),
);
```

---

## Toggle resolution

State precedence when toggled:

| Field | Source |
|---|---|
| `icon` | `toggled_icon` (or `icon` if empty) |
| `color` | `toggled_color` (or `color`) |
| `size` | `toggled_size` (or `size`) |
| `variant` | `toggled_variant` (or `variant`) |
| `icon_color` | `toggled_icon_color` (or `icon_color`) |
| `title` | `toggled_title` (or `title`; empty falls back) |

Clicking flips `*toggled`, then the state is re-read on the next frame.

---

## Performance and memory choices

| Choice | Effect |
|--------|--------|
| `ToggleVisualState` is a plain `Copy`-shaped value | No enum dispatch, no allocation. |
| Underlying `MudButton` | One render path — no icon-button logic duplicated. |
| `MudText::body1(..)` for the label | Rule 3 compliant. |

---

## Testing

`cargo test -p mud_egui_components mud_toggle_icon_button`:

- resolve to normal state when not toggled
- resolve to toggled state when toggled
- toggled icon empty → falls back to normal icon

---

## Differences from MudBlazor

- **`label` is an extension.** MudBlazor's `MudToggleIconButton` is icon-only.
- **`Style` / `Tag` / `UserAttributes` not ported.**
- **Toggle state is `&mut bool`.** MudBlazor uses `@bind-Toggled`; egui has no
  two-way binding, so the caller owns the flag.