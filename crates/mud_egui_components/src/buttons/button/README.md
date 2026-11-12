# MudButton for `mud_egui`

A themed button for [egui](https://github.com/emilk/egui), ported from
MudBlazor's [`MudButton`](https://mudblazor.com/components/button)
(MudBlazor 8.13.0). Supports variants (Text, Outlined, Filled), sizes,
start/end icons, disabled state, hover/pressed feedback, focus ring and a
theme-driven drop shadow.

```rust
let response = MudButton::new("Save")
    .variant(Variant::Filled)
    .color(Color::Primary)
    .size(Size::Large)
    .start_icon(filled::CHECK)
    .show(ui);

if response.clicked() {
    save();
}
```

Every string goes through **`MudText`**; every icon through **`MudIcon`**.

---

## Contents

1. [Design principle](#design-principle)
2. [Architecture](#architecture)
3. [API reference](#api-reference)
4. [Examples](#examples)
5. [Rendering pipeline](#rendering-pipeline)
6. [Performance and memory choices](#performance-and-memory-choices)
7. [Testing](#testing)
8. [Differences from MudBlazor](#differences-from-mudblazor)

---

## Design principle

Verified against MudBlazor 8.13.0 DNDocs.

| MudBlazor | Rust mapping | Default |
|---|---|---|
| `Variant` | `variant(Variant)` | `Text` |
| `Color` | `color(Color)` | `Primary` |
| `Size` | `size(Size)` | `Medium` |
| `Disabled` | `disabled(bool)` | `false` |
| `FullWidth` | `full_width(bool)` | `false` |
| `DropShadow` | `drop_shadow(bool)` | `true` |
| `StartIcon` | `start_icon(impl Into<String>)` | `None` |
| `EndIcon` | `end_icon(impl Into<String>)` | `None` |
| `IconColor` | `icon_color(Color)` | `None` |
| `ChildContent` | `new(text)` | required |
| `Class` (base) | `classes(iter)` — stored | `None` |
| `OnClick` | `Response::clicked()` | — |
| `Href` / `Target` / `Rel` | **not ported** — use `MudLink` | — |
| `OnClickStopPropagation` | **not ported** — no DOM | — |
| `Style` / `Tag` / `UserAttributes` | **not ported** | — |

### Extensions

| Extension | Purpose |
|---|---|
| `uppercase(bool)` | Overrides `Typo::Button`'s default uppercase |
| `icon_size(u8)` | Explicit icon bucket (`0..=4`) |

No public enums are introduced.

---

## Architecture

```
buttons/button/
├── mud_button.rs   MudButton + pure helpers + tests
└── mod.rs
```

Pure layer: `size_to_metrics`, `icon_extent`, `looks_like_svg_path`,
`resolve_uppercase`, `display_label`, `icon_kind_for`, `sanitize`,
`contrast_color`, `pick_base`, `mix_color`, `resolve_colors`,
`icon_color_for`, `interaction_strength`, `state_fill`, `plan_layout` — free
functions over `Size`, `Color32`, `f32`. None touch `&Ui`.

Render layer: `show` fetches the theme, calls the pure layer, allocates one
rect, paints shadow/fill/border/focus-ring, then paints the label through
`MudText` and the icons through `MudIcon`.

---

## API reference

| Method | Type | Default | Description |
|---|---|---|---|
| `new(text)` | `impl Into<String>` | — | Label. |
| `variant(Variant)` | `Variant` | `Text` | Variant. |
| `color(Color)` | `Color` | `Primary` | Semantic colour. |
| `size(Size)` | `Size` | `Medium` | Size preset. |
| `disabled(bool)` | `bool` | `false` | Disabled state. |
| `full_width(bool)` | `bool` | `false` | Stretch to available width. |
| `drop_shadow(bool)` | `bool` | `true` | Shadow on Filled buttons. |
| `start_icon(s)` / `end_icon(s)` | `impl Into<String>` | `None` | Icon slots. |
| `icon_color(Color)` | `Color` | `None` | Icon colour override. |
| `uppercase(bool)` | `bool` | `None` | Force uppercase. |
| `icon_size(u8)` | `u8` | `None` | Icon bucket (`0..=4`). |
| `classes(iter)` | `iter of AsRef<str>` | none | Stored. |
| `show(ui)` | `&mut Ui` | — | Returns `Response`. |
| `ui.add(..)` | `Widget` | — | Same as `show`. |

---

## Examples

### Variants and colours

```rust
MudButton::new("Text").show(ui);
MudButton::new("Outlined").variant(Variant::Outlined).show(ui);
MudButton::new("Filled").variant(Variant::Filled).color(Color::Primary).show(ui);
MudButton::new("Inherit").color(Color::Inherit).show(ui);
```

### Sizes

```rust
MudButton::new("Tiny").size(Size::ExtraSmall).show(ui);
MudButton::new("Huge").size(Size::ExtraLarge).variant(Variant::Filled).show(ui);
```

### Icons

```rust
MudButton::new("Download").start_icon(filled::DOWNLOAD).show(ui);
MudButton::new("Next").end_icon(filled::CHEVRON_RIGHT).icon_color(Color::Primary).show(ui);
MudButton::new("").start_icon(filled::STAR).show(ui); // icon-only, no phantom gap
```

### Full width / disabled

```rust
MudButton::new("Submit").variant(Variant::Filled).full_width(true).show(ui);
MudButton::new("Unavailable").variant(Variant::Outlined).disabled(true).show(ui);
```

### Forcing case

```rust
MudButton::new("preserve").uppercase(false).show(ui); // stays lowercase
MudButton::new("force").uppercase(true).show(ui);      // FORCE
```

---

## Rendering pipeline

```
Measure label (layout_no_wrap) → plan_layout → allocate_exact_size
     │                                              │
     ▼                                              ▼
size (plan.size)                            paint order:
                                            1. shadow (Filled only)
                                            2. fill (with hover/press overlay)
                                            3. border (Outlined only)
                                            4. focus ring (has_focus && !disabled)
                                            5. label (MudText::button)
                                            6. start icon (MudIcon)
                                            7. end icon (MudIcon)
```

Label goes through `MudText::button(..).uppercase(false)` because
`display_label` already applied the case transform.

---

## Performance and memory choices

| Choice | Effect |
|---|---|
| One `layout_no_wrap` for the label | Decides size before allocating. |
| One `allocate_exact_size` | No `egui::Button`, no extra scope. |
| `MudText` cache hit | Same font family + size as the measurement, so the second galley is a lookup. |
| Paint culling | Content is not painted when scrolled out of view. |
| Lazy widget info | `widget_info` closure only runs when accesskit is active. |
| Pure helpers | No `&Ui`; all arithmetic is testable. |

---

## Testing

`cargo test -p mud_egui_components mud_button` runs ~35 tests:

| Area | Tests |
|---|---|
| Size metrics | every `Size`, monotonicity |
| Uppercase | override beats default, buffer preserved when no transform |
| Icons | bucket extent, SVG path detection, kind resolution |
| Sanitization | NaN / ∞ / negative → 0 |
| Contrast | 384 boundary, black/white selection |
| Colour | base pick, mix endpoints, per-variant enabled / disabled |
| Interaction | press beats hover, state fills |
| Layout | text-only, start+text, text+end, icon-only, two-icons-no-text, full width, non-finite fallback, bad measurement repair |
| Builders | defaults, every setter |

Rendering requires a live `egui::Context` and is not unit-tested.

```bash
cargo test -p mud_egui_components mud_button
```

---

## Differences from MudBlazor

- **`Href` / `Target` / `Rel` not ported.** MudBlazor's `MudButton` can be a
  link; the port keeps it as a pure button and defers link behaviour to
  `MudLink`.
- **`OnClickStopPropagation` not ported.** No DOM event bubbling.
- **`uppercase` and `icon_size` are extensions.** MudBlazor does not expose
  them; the label follows `Typo::Button`'s default case and icons follow the
  button size.
- **`Style` / `Tag` / `UserAttributes` not ported.** No CSS strings, no HTML
  tag.
- **Label goes through `MudText` (Rule 3).** The previous version painted the
  galley directly via `painter.galley(..)`. The current version measures once
  and passes the text to `MudText::button(..)` inside a child `Ui` — same
  layout, compliant with the MudText rule, and honouring the theme's button
  typography automatically.
- **The shadow is a theme lookup + `as_shape`.** Same as `MudBlazor`'s
  `DropShadow` — it is not painted at all when `disabled` or non-`Filled`.