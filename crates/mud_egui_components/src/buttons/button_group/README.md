# MudButtonGroup for `mud_egui`

A horizontal or vertical group of buttons sharing style, ported from
MudBlazor's [`MudButtonGroup`](https://mudblazor.com/components/button)
(MudBlazor 8.13.0).

```rust
MudButtonGroup::new()
.variant(Variant::Filled)
.color(Color::Primary)
.show(ui, |group| {
group.button(MudButton::new("Reply"));
group.button(MudButton::new("Reply All"));
group.button(MudButton::new("Forward"));
});
```

Every button inside the group is a `MudButton`, so text goes through
**`MudText`** and icons through **`MudIcon`**.

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
| `Color` | `color(Color)` | `Primary` |
| `Size` | `size(Size)` | `Medium` |
| `Variant` | `variant(Variant)` | `Text` |
| `Vertical` | `vertical(bool)` | `false` |
| `FullWidth` | `full_width(bool)` | `false` |
| `DropShadow` | `drop_shadow(bool)` | `true` |
| `RightToLeft` | `right_to_left(bool)` | `false` |
| `OverrideStyles` | `override_styles(bool)` | `true` |
| `CornerRadius` | `corner_radius(f32)` — **stored, not yet resolved** | `4.0` |
| `ChildContent` | `button` closure via `ButtonGroupUi` | required |
| `Class` (base) | `classes(iter)` — stored | `None` |
| `Style` / `Tag` / `UserAttributes` | **not ported** | — |

No public enums are introduced.

---

## Architecture

```
buttons/button_group/
├── mud_button_group.rs   MudButtonGroup + ButtonGroupUi + tests
└── mod.rs
```

`show` runs the caller's closure once to **collect** every `MudButton`, then
renders them in one pass. This is the same pattern `MudGrid` uses. Two reasons:

1. `full_width` distribution needs to know the button count.
2. Separator placement needs to know which buttons are interior.

`override_styles(true)` overwrites each button's `variant` / `color` / `size`
/ `drop_shadow` with the group's values; `false` leaves them untouched.

---

## API reference

| Method | Type | Default | Description |
|---|---|---|---|
| `new()` | — | — | Default group. |
| `variant(Variant)` | `Variant` | `Text` | Group variant. |
| `color(Color)` | `Color` | `Primary` | Group colour. |
| `size(Size)` | `Size` | `Medium` | Group size. |
| `vertical(bool)` | `bool` | `false` | Stack vertically. |
| `full_width(bool)` | `bool` | `false` | Fill available width. |
| `drop_shadow(bool)` | `bool` | `true` | Shadows on children. |
| `right_to_left(bool)` | `bool` | `false` | Reverse horizontal order. |
| `override_styles(bool)` | `bool` | `true` | Apply group styles to every button. |
| `corner_radius(f32)` | `f32` | `4.0` | Stored. |
| `classes(iter)` | `iter of AsRef<str>` | none | Stored. |
| `id_salt(salt)` | `impl Hash + Debug` | `None` | Stable interaction ids. |
| `show(ui, \|group\| ..)` | `FnOnce(&mut ButtonGroupUi)` | — | Renders. |

### `ButtonGroupUi`

| Method | Description |
|---|---|
| `button(MudButton)` | Registers a button (applies group styles if `override_styles`). |
| `len()` / `is_empty()` | Introspection. |

---

## Examples

### Filled horizontal

```rust
MudButtonGroup::new().variant(Variant::Filled).show(ui, |g| {
    g.button(MudButton::new("A"));
    g.button(MudButton::new("B"));
    g.button(MudButton::new("C"));
});
```

### Outlined — no separators

```rust
MudButtonGroup::new()
    .variant(Variant::Outlined) // Outlined draws no separators
    .show(ui, |g| { /* … */ });
```

### Vertical

```rust
MudButtonGroup::new().vertical(true).show(ui, |g| { /* … */ });
```

### Full-width, evenly split

```rust
MudButtonGroup::new().full_width(true).show(ui, |g| { /* … */ });
```

### Preserve per-button styling

```rust
MudButtonGroup::new()
    .override_styles(false)
    .show(ui, |g| {
        g.button(MudButton::new("Text").variant(Variant::Text));
        g.button(MudButton::new("Filled").variant(Variant::Filled));
    });
```

---

## Layout

```
Horizontal (natural):      [A][sep][B][sep][C]
Horizontal (full_width):   [──── A ────][sep][──── B ────][sep][──── C ────]
Vertical (auto-width):     [A]
                           [sep]
                           [B]
                           [sep]
                           [C]
```

Separator is 1 pt; `Outlined` groups omit it (rely on border overlap). Full-
width horizontal splits `(available − sep_total)` evenly across buttons.

---

## Performance and memory choices

| Choice | Effect |
|---|---|
| One closure invocation | No double-run of the caller's `button(..)` calls. |
| Buttons collected into a `Vec` | Bounded by button count; freed at end of `show`. |
| No dynamic dispatch inside render | `MudButton` is monomorphic. |

---

## Testing

`cargo test -p mud_egui_components mud_button_group`:

- defaults, every builder
- `apply_styles` with override on/off
- `needs_separators` for all three variants
- `classes` accumulation, `id_salt` determinism

Rendering requires a live `Context`.

---

## Differences from MudBlazor

- **`Color` / `Size` / `Variant` are not nullable.** MudBlazor uses `Color?`,
  `Size?`, `Variant?` and only applies them when the caller set them; the port
  requires concrete values and always applies them when `override_styles` is
  `true`. If you want MudBlazor's "only override what you set" behaviour, use
  `override_styles(false)` and configure each button explicitly.
- **`corner_radius` is stored, not resolved.** `MudButton` does not currently
  expose a per-instance corner radius; the group cannot distribute it.
- **`Style` / `Tag` / `UserAttributes` not ported.**