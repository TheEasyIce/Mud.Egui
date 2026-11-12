
===== ./crates/mud_egui_bootstrap/src/README.md =====

# Bootstrap Utilities for Mud.Egui

Bootstrap-style utility classes for egui widgets. Style controls with class names like `bg-primary`, `p-3` and `rounded-pill`, resolved against your active theme.

## Contents

- [Getting Started](#getting-started)
- [Utilities](#utilities)
- [Scales](#scales)
- [Specificity Rules](#specificity-rules)
- [Troubleshooting](#troubleshooting)

## Getting Started

egui is an immediate-mode library, so there is no persistent widget object and no `IsEnabled`-style property. Instead, each component stores a style cache (`BsStyle`) next to its `ComponentBase`. The cache resolves class names once and reuses the result on later frames.

```rust
use mud_egui_components::foundation::base::component_base::ComponentBase;
use mud_egui_bootstrap::bs_style::BsStyle;

pub struct MyCard {
    base: ComponentBase,
    style: BsStyle,
}

impl MyCard {
    pub fn new() -> Self {
        Self {
            base: ComponentBase::new().classes(["bg-primary", "p-3", "rounded-pill"]),
            style: BsStyle::default(),
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        // Resolve the classes into an egui::Frame.
        // 14.0 is the rem size in points.
        let frame = self.style.build_frame(&self.base, ui, 14.0);

        // Draw inside the frame.
        frame.show(ui, |ui| {
            ui.label("Hello Bootstrap");
        });
    }
}
```

**The flow:** declare classes on `ComponentBase` → resolve with `BsStyle::build_frame` → draw inside the returned `egui::Frame`.

Keep `BsStyle` as a field on your component. Creating it inside `ui()` throws the cache away every frame.

## Utilities

| Utility | Controls | Example classes |
| --- | --- | --- |
| [BsBackground](#bsbackground) | `Frame.fill` | `bg-primary`, `bg-body`, `bg-primary-hover` |
| [BsBorder](#bsborder) | `Frame.stroke` | `border-dashed-2`, `border-0` |
| [BsMargin / BsPadding](#bsmargin--bspadding) | `Frame.outer_margin` / `Frame.inner_margin` | `m-3`, `px-2`, `mt-n2`, `mx-auto` |
| [BsRounded](#bsrounded) | `Frame.corner_radius` | `rounded-3`, `rounded-pill`, `rounded-top` |

### BsBackground

Binds `Frame.fill` to a theme palette color.

- **Syntax:** `bg-primary`, `bg-body`, `bg-transparent`, `bg-white`, `bg-black`
- **Hover:** `bg-primary-hover` maps to `MudPalettePrimaryHoverBrush`
- **Aliases:** `bg-body` maps to `MudPaletteBackground`
- **Rule:** the last `bg-*` class written wins

### BsBorder

Sets `Frame.stroke`.

- **Styles:** `border-solid`, `border-dashed`, `border-dotted`
- **Width:** `border-0` to `border-5`
- **Combined:** `border-dashed-2` (dashed style, 2px width)

### BsMargin & BsPadding

Sets `Frame.outer_margin` (margin) and `Frame.inner_margin` (padding).

- **Syntax:** `m-3` (margin), `p-3` (padding), `mx-2` (horizontal), `mt-0` (top)
- **Negative margins:** `mt-n2` (that is, `-0.5rem`)
- **Auto margins:** `mx-auto`, `ms-auto`, `me-auto`

> **Note:** egui has no auto margins. For `*-auto` classes the cache returns an `egui::Align`, which you must apply to your layout yourself.

### BsRounded

Sets `Frame.corner_radius`.

- **Sizes:** `rounded-0` to `rounded-5`, `rounded-pill`, `rounded-circle`
- **Sides:** `rounded-top`, `rounded-end`, `rounded-bottom`, `rounded-start`

## Scales

All values are multiples of the `rem` size you pass to `build_frame` (Bootstrap's usual base is `14.0` here).

### Spacing (margin & padding)

| Step | `0` | `1` | `2` | `3` | `4` | `5` |
| --- | --- | --- | --- | --- | --- | --- |
| rem | 0 | 0.25 | 0.5 | 1 | 1.5 | 3 |

### Corner radius

| Step | `0` | `1` | `2` (default) | `3` | `4` | `5` | `pill` |
| --- | --- | --- | --- | --- | --- | --- | --- |
| rem | 0 | 0.25 | 0.375 | 0.5 | 1 | 2 | 50 |

## Specificity Rules

When several classes touch the same property, the more specific one wins, regardless of order.

| Utility | Order (lowest → highest) |
| --- | --- |
| Margin / padding | `m` < `mx`/`my` < `mt`/`mb`/`ms`/`me` |
| Rounded | `rounded*` < `top` < `end` < `bottom` < `start` |
| Background | last `bg-*` written wins |

Example: `p-0 pt-3` gives zero padding on three sides and `1rem` on top.

## Troubleshooting

| Symptom | Likely cause / fix |
| --- | --- |
| **Classes do nothing** | Pass the `ComponentBase` to `build_frame` and draw inside the returned `Frame`. |
| **`mx-auto` doesn't center** | `auto` returns an `egui::Align`. Apply it to the layout, for example with `ui.with_layout(egui::Layout::right_to_left(...), ...)`. |
| **Padding is huge** | Check the `rem` value you passed. At `16.0`, `p-3` is 16px; at `14.0` it is 14px. A mistaken value such as `48.0` multiplies every scale. |
| **Wrong colors** | The resolver returns raw keys such as `"MudPalettePrimary"`. Make sure your theme lookup can resolve them. |

===== ./crates/mud_egui_components/src/buttons/button_group/README.md =====

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

===== ./crates/mud_egui_components/src/buttons/button/README.md =====

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

===== ./crates/mud_egui_components/src/buttons/fab/README.md =====

# MudFab + MudMenuFab for `mud_egui`

Floating action buttons for [egui](https://github.com/emilk/egui), ported from
MudBlazor's [`MudFab`](https://mudblazor.com/components/buttonfab) and
[`MudMenuFab`](https://mudblazor.com/components/menufab) (MudBlazor 8.13.0).

`MudFab` is a standalone floating action button. `MudMenuFab` is a trigger FAB
that expands into a vertical menu of icon-only actions.

```rust
// Standalone FAB.
ui.add(MudFab::new().icon(filled::ADD).color(Color::Primary));

// Action menu.
let mut menu = MenuFabState::new("compose");
MudMenuFab::new()
    .icon(filled::ADD)
    .fixed(true)
    .bottom(24.0)
    .right(24.0)
    .actions([
        FabMenuAction::new(filled::EDIT).tooltip("Edit").on_click(|| open_editor()),
        FabMenuAction::new(filled::DELETE).tooltip("Delete").on_click(|| confirm_delete()),
    ])
    .show(ui, &mut menu);
```

Every string goes through **`MudText`**, every glyph through **`MudIcon`**.

---

## Contents

1. [Design principle: API parity, not API invention](#design-principle-api-parity-not-api-invention)
2. [Architecture](#architecture)
3. [API reference — MudFab](#api-reference--mudfab)
4. [API reference — MudMenuFab](#api-reference--mudmenufab)
5. [Examples](#examples)
6. [Layout and interaction](#layout-and-interaction)
7. [Performance and memory choices](#performance-and-memory-choices)
8. [Testing](#testing)
9. [Differences from MudBlazor](#differences-from-mudblazor)

---

## Design principle: API parity, not API invention

The MudBlazor 8.13.0 parameter surfaces were verified against DNDocs before
implementation. `MudFab` has **eighteen** own parameters; `MudMenuFab` has
**eleven** plus the `FabMenuAction` record.

### `MudFab` parameters

| MudBlazor | Rust mapping | Default |
|---|---|---|
| `Bottom` | `bottom(f32)` / `Option<f32>` | `None` → `16.0` |
| `ChildContent` | `child(FnMut(&mut Ui))` | `None` |
| `Color` | `color(Color)` | `Default` |
| `Disabled` | `disabled(bool)` | `false` |
| `DropShadow` | `drop_shadow(bool)` | `true` |
| `EndIcon` | `end_icon(impl Into<String>)` | `None` |
| `Fixed` | `fixed(bool)` | `false` |
| `Href` | `href(impl Into<String>)` | `None` |
| `Icon` | `icon(impl Into<String>)` | `None` |
| `Label` | `label(impl Into<String>)` | `None` |
| `OnClick` | `on_click(FnMut())` | `None` |
| `Right` | `right(f32)` / `Option<f32>` | `None` → `16.0` |
| `Size` | `size(Size)` | `Large` |
| `StartIcon` | `start_icon(impl Into<String>)` | `None` |
| `Target` | `target(impl Into<String>)` | `None` |
| `Title` | `title(impl Into<String>)` | `None` |
| `Variant` | `variant(Variant)` | `Filled` |
| `Class` | `classes(iter)` | `None` |
| `Style` / `Tag` / `UserAttributes` | **not ported** | — |

### `MudMenuFab` parameters

| MudBlazor | Rust mapping | Default |
|---|---|---|
| `Actions` | `actions(iter of FabMenuAction)` | required |
| `ChildContent` | `child(FnMut(&mut Ui))` | `None` |
| `Color` | `color(Color)` | `Default` |
| `Disabled` | `disabled(bool)` | `false` |
| `Icon` | `icon(impl Into<String>)` | required |
| `OnClick` | `on_click(FnMut())` | `None` |
| `Size` | `size(Size)` | `Large` |
| `Variant` | `variant(Variant)` | `Filled` |
| `Fixed` / `Bottom` / `Right` | `fixed(bool)` / `bottom` / `right` | from `MudFab` |
| `Class` | `classes(iter)` | `None` |
| `Style` / `Tag` / `UserAttributes` | **not ported** | — |

### `FabMenuAction` fields

| MudBlazor | Rust | Required |
|---|---|---|
| `Icon` | `icon: String` | yes |
| `Tooltip` | `tooltip: Option<String>` | no |
| `OnClick` | `on_click: Option<Box<dyn FnMut() + 'a>>` | no |

### Deliberately not ported

| MudBlazor | Reason |
|---|---|
| `Class` / `Style` / `Tag` / `UserAttributes` resolution | No CSS strings, no HTML tag. The FAB owns its fill, radius and shadow. |

No enums are introduced — `Size`, `Color`, `Variant`, `AnchorOrigin` and
`TransformOrigin` are all reused from `mud_egui_core::enums`.

---

## Architecture

```
buttons/fab/
├── fab_menu_action.rs  FabMenuAction
├── menu_fab_state.rs   MenuFabState (caller-owned)
├── mud_fab.rs          MudFab + FabMetrics + FabColors + helpers
├── mud_menu_fab.rs     MudMenuFab + MenuFabResponse + helpers
└── mod.rs
```

**Pure layer.** `FabMetrics::resolve`, `fab_label_font_size`,
`resolve_colors`, `action_side`, `total_height` are free functions or methods
over `Size`, `Color`, `Variant`, `Palette`. None touch `egui::Context`.

**Paint layer.** `paint_fab_bg` (in `mud_fab.rs`) draws the shadow, the fill
and the stroke. Both `MudFab` and `MudMenuFab`'s action buttons go through
the same helper.

**State layer.** `MenuFabState` wraps a `PopoverState` — the open flag and the
stable id used by `MudPopover`.

**Component layer.** `MudFab::show` is the trigger renderer;
`MudMenuFab::show` composes `MudFab` for the trigger and `MudPopover` for the
action column.

---

## API reference — `MudFab<'a>`

| Method | Type | Default | Description |
|---|---|---|---|
| `new()` | — | — | Default FAB. |
| `icon(s)` | `impl Into<String>` | `None` | Main icon. |
| `label(s)` | `impl Into<String>` | `None` | Pill-shaped label. |
| `start_icon(s)` / `end_icon(s)` | `impl Into<String>` | `None` | Icons around the label. |
| `size(s)` | `Size` | `Large` | 40 / 48 / 56 pt. |
| `color(c)` | `Color` | `Default` | Accent. |
| `variant(v)` | `Variant` | `Filled` | Fill, outline, or text. |
| `disabled(b)` | `bool` | `false` | Dims + blocks. |
| `drop_shadow(b)` | `bool` | `true` | Soft shadow when `Filled`. |
| `title(s)` | `impl Into<String>` | `None` | Hover tooltip. |
| `fixed(b)` | `bool` | `false` | Fixed on-screen position. |
| `bottom(px)` / `right(px)` | `f32` | `None` → `16.0` | Offsets when `fixed`. |
| `href(s)` / `target(s)` | `impl Into<String>` | `None` | URL opened on click. |
| `on_click(f)` | `FnMut()` | `None` | Click callback. |
| `child(f)` | `FnMut(&mut Ui)` | `None` | Custom content; overrides icon + label. |
| `classes(iter)` / `flavor(StyleFlavor)` | — | — | Utilities. |
| `show(ui)` | `&mut Ui` | — | Returns `Response`. |
| `ui.add(..)` | — | — | `Widget` impl. |

### `FabMetrics`

| Field | `Small` | `Medium` | `Large` |
|---|---|---|---|
| `side` | 40 | 48 | 56 |
| `icon_px` | 20 | 24 | 24 |
| `pad_x` | 16 | 20 | 24 |
| `label_gap` | 8 | 8 | 8 |

`ExtraSmall` folds into `Small`, `ExtraLarge` into `Large`.

---

## API reference — `MudMenuFab<'a>`

| Method | Type | Default | Description |
|---|---|---|---|
| `new()` | — | — | Default action menu. |
| `icon(s)` | `impl Into<String>` | `""` | Trigger icon. |
| `actions(iter)` | `iter of FabMenuAction` | empty | Action list, top-to-bottom. |
| `size(s)` / `color(c)` / `variant(v)` | enum | `Large` / `Default` / `Filled` | Trigger appearance. |
| `disabled(b)` | `bool` | `false` | Disables the trigger and every action. |
| `drop_shadow(b)` | `bool` | `true` | Shadow on the trigger and each action. |
| `fixed(b)` / `bottom(px)` / `right(px)` | `bool` / `f32` | `false` / `16.0` | Same as `MudFab`. |
| `on_click(f)` | `FnMut()` | `None` | Trigger-click callback (fires on open *and* close). |
| `child(f)` | `FnMut(&mut Ui)` | `None` | Custom trigger content. |
| `aria_label(s)` | `impl Into<String>` | `None` | Accessible trigger label. |
| `show(ui, state)` | `(&mut Ui, &mut MenuFabState)` | — | Returns `MenuFabResponse`. |

### `MenuFabState`

| Method | Description |
|---|---|
| `new(id_salt)` | Closed menu with a stable id. |
| `is_open()` / `open()` / `close()` / `toggle()` | Popover control. |
| `popover: PopoverState` | The underlying popover state. |

### `MenuFabResponse`

| Field | Type | Description |
|---|---|---|
| `response` | `Response` | The trigger's `Response`. |
| `opened` | `bool` | `true` on the frame the menu opened. |
| `closed` | `bool` | `true` on the frame the menu closed. |
| `triggered` | `Option<usize>` | Index of the action fired this frame. |
| `is_open()` | `bool` | Convenience. |

### `FabMenuAction<'a>`

| Method | Type | Default | Description |
|---|---|---|---|
| `new(icon)` | `impl Into<String>` | — | Required icon. |
| `tooltip(s)` | `impl Into<String>` | `None` | Hover tooltip. |
| `on_click(f)` | `FnMut()` | `None` | Click callback. |

### Constants

| Name | Value |
|---|---|
| `DEFAULT_RIGHT` / `DEFAULT_BOTTOM` | `16.0` |
| `ACTION_GAP` | `8.0` |
| `TRIGGER_OFFSET` | `12.0` |
| `ACTION_INSET` | `8.0` |

---

## Examples

### Icon-only FAB

```rust
ui.add(MudFab::new().icon(filled::ADD).color(Color::Primary));
```

### Labelled FAB

```rust
ui.add(
    MudFab::new()
        .icon(filled::EDIT)
        .label("New document")
        .size(Size::Large),
);
```

### Fixed FAB at the bottom-right

```rust
ui.add(
    MudFab::new()
        .icon(filled::ADD)
        .fixed(true)
        .right(24.0)
        .bottom(24.0),
);
```

### FAB with `href`

```rust
ui.add(
    MudFab::new()
        .icon(filled::OPEN_IN_NEW)
        .href("https://docs.rs")
        .target("_blank"),
);
```

### Disabled FAB

```rust
ui.add(
    MudFab::new()
        .icon(filled::SAVE)
        .disabled(true),
);
```

### FAB menu with actions

```rust
let mut menu = MenuFabState::new("compose");

MudMenuFab::new()
    .icon(filled::ADD)
    .color(Color::Primary)
    .actions([
        FabMenuAction::new(filled::EDIT)
            .tooltip("New document")
            .on_click(|| new_document()),
        FabMenuAction::new(filled::EMAIL)
            .tooltip("New email")
            .on_click(|| new_email()),
        FabMenuAction::new(filled::PHOTO_CAMERA)
            .tooltip("New photo")
            .on_click(|| new_photo()),
    ])
    .show(ui, &mut menu);
```

### Fixed FAB menu at the bottom-right

```rust
MudMenuFab::new()
    .icon(filled::MORE_VERT)
    .fixed(true)
    .right(24.0)
    .bottom(24.0)
    .actions(/* … */)
    .show(ui, &mut menu);
```

### Reacting to a fired action

```rust
let resp = MudMenuFab::new()
    .icon(filled::ADD)
    .actions(/* … */)
    .show(ui, &mut menu);

if let Some(i) = resp.triggered {
    log::info!("action {i} fired");
}
```

### Programmatic open / close

```rust
menu.open();
menu.close();
menu.toggle();
```

---

## Layout and interaction

### Trigger layout

```
Icon-only                       With label
  ┌─────────┐                   ┌───────────────────────────────┐
  │    ★    │                   │ [★]  New document  [→]        │
  └─────────┘                   └───────────────────────────────┘
   side × side                  side tall, content + 2 × pad wide
   radius = side/2              radius = side/2
```

The `child` closure takes over the whole rect when set.

### Fixed positioning

`fixed(true)` renders the FAB inside an `egui::Area` anchored to
`Align2::RIGHT_BOTTOM` at `-(right, bottom)`. The bottom-right corner of the
FAB therefore sits `right` points from the screen's right edge and `bottom`
points from the screen's bottom edge. No space is reserved in the parent `Ui`
— matching CSS `position: fixed`.

### Action menu layout

```
┌────┐  ← action N (top)
└────┘
   ↑ ACTION_GAP
┌────┐  ← action N-1
└────┘
   ⋮
   ↑ TRIGGER_OFFSET
┌────┐  ← trigger
└────┘
```

The popover is anchored at `AnchorOrigin::Top` / `TransformOrigin::BottomCenter`
with `offset(TRIGGER_OFFSET)`, so the column's bottom-center sits 12 points
above the trigger's top-center. Action buttons are `trigger_size − 8` points
across with an 8-point gap.

### Interaction

| Input | Effect |
|---|---|
| Click trigger | Toggles the action menu; fires `on_click` |
| Click action | Fires that action's `on_click`; closes the menu |
| Escape (menu open) | Closes the menu |
| Click outside the trigger and the menu (menu open) | Closes the menu |
| Disabled | Every input is ignored, cursor is `NotAllowed` |

`MudPopover`'s built-in `dismiss_on_outside_click` is disabled so a click on
the trigger cannot race with a background dismissal (the trigger is outside
the popover, so the built-in dismissal would fire *and* the toggle would fire,
netting a no-op). Dismissal is handled in `MudMenuFab::show` with an explicit
"outside both" check.

---

## Performance and memory choices

| Choice | Effect |
|---|---|
| `MudFab` is self-painted | One `allocate_exact_size`, one shadow pass (three `rect_filled`), one fill, one stroke. No `egui::Button`, no extra scope. |
| Label is measured once per frame | `layout_no_wrap` for the size, then `MudText` reuses the galley cache. |
| `FabMetrics::resolve` is pure and `#[inline]` | Arithmetic on five `f32`s; unit-testable without a `Context`. |
| `resolve_colors` is pure | One `Color::resolve` (or zero for `Default` / `Inherit`), one contrast computation, one `gamma_multiply`. |
| `paint_fab_bg` shared between trigger and actions | No duplicated fill / stroke / shadow logic. |
| Shadow is three `rect_filled` calls | No `Frame`, no shader, no `Shape::shadow`. Visually close to a 6 pt blur. |
| Action column is a single popover | One `MudPopover::show` per open frame — sizing, positioning, z-order all come from the shared primitive. |
| `MenuFabState` is a `PopoverState` | Two fields, `Copy`. |
| Actions are `Vec<FabMenuAction>` on the builder | Rebuilt per frame by the caller; no state between frames. |
| Disabled short-circuits interaction | `Sense::hover()` for disabled; no click registration. |

---

## Testing

`cargo test -p mud_egui_components mud_fab mud_menu_fab` runs the module's
tests:

| Area | Tests |
|---|---|
| `FabMetrics::resolve` | every size, edge-size folding |
| `fab_label_font_size` | every size |
| `resolve_colors` | filled contrast ink, outlined transparent fill + stroke, text no fill + stroke, disabled alpha |
| `action_side` | every size, edge-size folding |
| `total_height` | 0 / 1 / N actions, gap accounting |
| `FabMenuAction` | construction, tooltip, `on_click` fires |
| `MenuFabState` | default, `open` / `close` / `toggle`, stable id |
| `MudFab` defaults | every field matches MudBlazor |
| `MudFab` builders | every field set |
| `MudMenuFab` defaults | every field matches MudBlazor |
| `MudMenuFab` builders | every field set |
| `MenuFabResponse::is_open` | opened / closed variants |

Rendering needs a live `egui::Context`; the pure layer is the source of
truth.

```bash
cargo test -p mud_egui_components mud_fab
cargo test -p mud_egui_components mud_menu_fab
cargo test -p mud_egui_components menu_fab_state
cargo test -p mud_egui_components fab_menu_action
```

---

## Differences from MudBlazor

- **`Bottom` / `Right` are `f32`, not `int?`.** Typed and clamped by
  `DEFAULT_BOTTOM` / `DEFAULT_RIGHT` when unset.
- **The `Title` tooltip is `Response::on_hover_text`.** egui has no HTML
  `title` attribute.
- **`Href` / `Target` open the system browser via `egui::Context::open_url`.**
  `"_blank"` maps to `OpenUrl::new_tab`; every other target maps to
  `OpenUrl::same_tab`. Same mapping `MudLink` uses.
- **`ChildContent` replaces the icon-and-label layout.** MudBlazor's
  `ChildContent` behaves the same way for `MudFab`.
- **`MudMenuFab`'s action menu is a `MudPopover`.** The column stacks above
  the trigger and closes on escape, action click, or outside click. The
  `MudPopover` built-in `dismiss_on_outside_click` is disabled because it
  cannot distinguish the trigger from "outside"; dismissal is handled by
  `MudMenuFab::show` explicitly.
- **The shadow is approximated.** MudBlazor uses a CSS `box-shadow`. egui has
  no blurred shadow primitive that can be painted behind an already-allocated
  rect without a second `Frame` pass. Three stacked `rect_filled` calls stand
  in — visually close but not pixel-perfect. If you need a real blur, wrap
  the FAB in a `Frame` at the cost of losing the exact
  `allocate_exact_size` sizing.
- **No `Style` / `Tag` / `UserAttributes`.** No CSS strings, no HTML tag.
- **`class` / `style` on `FabMenuAction`** are stored-but-ignored in
  MudBlazor's JS interop; the port does not expose them (they would be dead
  fields).

===== ./crates/mud_egui_components/src/buttons/icon_button/README.md =====

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

===== ./crates/mud_egui_components/src/buttons/toggle_icon_button/README.md =====

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

===== ./crates/mud_egui_components/src/data_display/avatar/README.md =====

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

===== ./crates/mud_egui_components/src/data_display/badge/README.md =====

# MudBadge for egui

Production-grade, MudBlazor-faithful port of `MudBadge.razor.cs`
(MudBlazor 8.13.0) for the [`egui`](https://github.com/emilk/egui)
immediate-mode GUI library.

`MudBadge` overlays a **dot**, an **icon**, or a **short count** onto any
corner of an arbitrary child widget, with all nine `Origin` anchors,
integer-count capping, optional click handling, elevation shadows, and
utility-class styling.

---

## Table of contents

1. [Architecture](#architecture)
2. [API reference](#api-reference)
3. [Real-world examples](#real-world-examples)
4. [Positioning semantics](#positioning-semantics)
5. [Breaking changes](#breaking-changes)
6. [Performance & design rationale](#performance--design-rationale)
7. [Testing](#testing)
8. [License](#license)

---

## Architecture

```
mud_badge.rs
├── MudBadge                      — configuration + render entry point
│   └── show(ui, child) -> (Response, R, bool)
├── BadgeStyle                    — resolved fill / ink / shadow / stroke
├── resolve_badge_style(..)       — Colour + BadgeClass + Bordered + Elevation
├── badge_center(..)              — pure positioning math (9 origins × overlap)
├── format_count(..)              — pure content capping ("100" → "99+")
├── anchor_sides(..)              — Origin → (horizontal, vertical) direction
└── paint_badge(..)               — shadow + mask + fill + stroke
```

The component is a **thin overlay**: `show` draws the child via
`ui.scope(child)`, reads the child's `Rect`, computes the badge's absolute
rect, and paints + hit-tests the badge **without reserving any additional
parent layout space** (matching MudBlazor's CSS `position: absolute`).
The returned `(child_response, child_return_value, badge_clicked)` tuple lets
callers distinguish a child click from a badge click.

---

## API reference

| Method | Type | Default | MudBlazor equivalent | Description |
|---|---|---|---|---|
| `new()` | `Self` | — | — | Creates a badge with MudBlazor defaults. |
| `color(c)` | `Color` | `Color::Default` | `Color` | Accent colour; `Default` → theme primary. |
| `dot(b)` | `bool` | `false` | `Dot` | Render a small dot instead of content. |
| `content(s)` | `impl Into<String>` | `None` | `Content` (string) | Badge caption. |
| `count(n)` | `i32` | `None` | `Content` (int) | Badge caption from an integer. |
| `icon(svg)` | `impl Into<String>` | `None` | `Icon` | Material SVG path inside the badge. |
| `max(m)` | `i32` | `99` | `Max` | Cap for numeric content (`100` → `99+`). |
| `overlap(b)` | `bool` | `false` | `Overlap` | `true` centres the badge on the edge; `false` pushes it outside. |
| `visible(b)` | `bool` | `true` | `Visible` | Show/hide the badge without removing the child. |
| `bordered(b)` | `bool` | `false` | `Bordered` | Draw a 2 px surface ring around the badge. |
| `origin(o)` | `Origin` | `TopRight` | `Origin` | One of 9 anchor corners. |
| `elevation(e)` | `u8` | `0` | `Elevation` | Drop shadow (0–24; clamped). |
| `badge_class(s)` | `impl Into<String>` | `None` | `BadgeClass` | Utility classes applied to the badge itself. |
| `badge_aria_label(s)` | `impl Into<String>` | `None` | `BadgeAriaLabel` | ARIA label published to accesskit. |
| `on_click(f)` | `FnMut() + 'static` | `None` | `OnClick` | Makes the badge clickable. |
| `flavor(f)` | `StyleFlavor` | `Bootstrap` | — | Utility-class flavour (extension). |
| `classes(iter)` | `iter of AsRef<str>` | — | `Class` | Utility classes on the **wrapper** frame. |
| `show(ui, child)` | `&mut Ui, FnOnce(&mut Ui) -> R` | — | — | Draws the badge and returns `(Response, R, bool)`. |

### `Origin` anchors

| Variant | Horizontal | Vertical |
|---|---|---|
| `TopLeft` | start | top |
| `TopCenter` | centre | top |
| `TopRight` (default) | end | top |
| `CenterLeft` | start | centre |
| `Center` | centre | centre |
| `CenterRight` | end | centre |
| `BottomLeft` | start | bottom |
| `BottomCenter` | centre | bottom |
| `BottomRight` | end | bottom |

### Content precedence

The renderer picks **exactly one** content kind, in this order:

1. `dot(true)` → dot badge (ignores `icon` / `content`)
2. `icon(..)` → icon badge (ignores `content`)
3. `content(..)` → text badge
4. none of the above → no badge is drawn

### `show` return value

```rust
let (child_resp, child_value, badge_clicked) =
    MudBadge::new().content("3").show(ui, |ui| { /* … */ });
```

- `child_resp` — the child's own `Response`.
- `child_value` — whatever the `child` closure returned.
- `badge_clicked` — `true` only if the badge was clicked **and** an
  `on_click` handler is registered.

---

## Real-world examples

### Notification count on an icon button

```rust
use mud_egui_components::MudBadge;
use mud_egui_core::enums::Color;
use mud_egui_icons_material::filled;

let (_, _, clicked) = MudBadge::new()
    .count(5)
    .color(Color::Error)
    .max(99)
    .show(ui, |ui| {
        ui.add(egui::Button::new("🔔"));
    });

if clicked {
    open_notifications();
}
```

### Dot indicator

```rust
MudBadge::new()
    .dot(true)
    .color(Color::Success)
    .origin(Origin::TopRight)
    .show(ui, |ui| {
        ui.add(egui::Label::new("Inbox"));
    });
```

### Icon badge with ARIA label

```rust
MudBadge::new()
    .icon(filled::STAR)
    .badge_aria_label("Featured")
    .show(ui, |ui| {
        ui.add(egui::Button::new("Product"));
    });
```

### Non-overlapping badge (MudBlazor default)

```rust
MudBadge::new()
    .count(12)
    .overlap(false) // badge sits fully outside the child's corner
    .elevation(2)
    .show(ui, |ui| {
        ui.add(egui::Button::new("Messages"));
    });
```

### Overlapping badge

```rust
MudBadge::new()
    .count(3)
    .overlap(true) // badge centre sits on the child's corner
    .show(ui, |ui| {
        ui.add(egui::Button::new("Notifications"));
    });
```

### Bordered badge with a custom class

```rust
MudBadge::new()
    .count(7)
    .bordered(true)
    .badge_class("bg-warning text-black")
    .show(ui, |ui| {
        ui.add(egui::Button::new("Tasks"));
    });
```

---

## Positioning semantics

MudBlazor positions the badge with CSS `transform: translate(…)` applied to
an absolutely-positioned element. The Rust port reproduces the same
semantics with `badge_center`:

| `overlap` | Anchored axis behaviour |
|---|---|
| `true` | Badge centre sits **on** the child's edge (the classic overlap). |
| `false` (default) | Badge is pushed **fully outside** the child by **half its own size** on the anchored axis. |

On the **non-anchored** axis (e.g. the horizontal axis for `TopCenter`),
the badge is always centred on the child's axis.

This means wide text badges (`"99+"`) never clip into the child when
`overlap(false)` — the push-out offset automatically scales with the
badge's measured width.

---

## Breaking changes

| Old API | New API | Notes |
|---|---|---|
| `.clickable(true)` | *(removed)* | The badge is clickable iff `on_click` is set (MudBlazor parity: `OnClick.HasDelegate`). |
| `.color(..)` default `Primary` | default `Color::Default` | Matches MudBlazor's `MudBadge` default. |
| `.overlap(..)` default `true` | default `false` | Matches MudBlazor's `MudBadge` default. |

### Bug fixes in this refactor

1. **Positioning ignored badge size.** `compute_badge_center` previously
   hard-coded a 12 px inward offset and ignored `badge_w` / `badge_h`. Wide
   text badges clipped into the child. Now the offset is the badge's own
   half-size on the anchored axis.
2. **Layout growth.** `ui.allocate_rect` reserved the badge's rect in the
   parent layout, growing the outer `Frame`. The badge now uses
   `ui.interact` on its absolute rect, matching MudBlazor's absolute overlay.
3. **Hit-test used the wrong rect.** The cursor-icon check used
   `MIN_TEXT_EXTENT` for every content kind; it now uses the actual badge
   rect, so dots and wide text badges hit-test correctly.
4. **`dot` / `icon` badges ignored clicks.** They now respect `on_click`
   just like text badges.

---

## Performance & design rationale

| Technique | Benefit |
|---|---|
| **Single style resolution** | `resolve_badge_style` runs once per frame and returns a `BadgeStyle` (`Copy`-like fields); no repeated palette lookups per content kind. |
| **Pure positioning math** | `badge_center`, `format_count`, and `anchor_sides` are `#[inline]` pure functions, fully unit-tested without an `egui::Context`. |
| **No layout side effects** | The badge uses `ui.interact` on an absolute rect, so wrapping a child in a badge never changes the parent's layout (unlike `allocate_rect`). |
| **Class parsing only when needed** | The `BadgeClass` frame is built only when `badge_class` is `Some` and non-empty. |
| **Elevation clamped** | `elevation_shadow` clamps at 24, preventing `i8` overflow in `egui::epaint::Shadow::offset`. |
| **Mask via `rect_filled`** | The 1.5 px surface mask is a single `rect_filled` call; no extra shape allocation. |

---

## Testing

`mud_badge.rs` ships with **17 unit tests**:

- `anchor_sides` covers all 9 origins.
- `badge_center` covers `overlap(true)` vs `overlap(false)` for
  `TopRight`, `TopLeft`, `BottomRight`, `TopCenter`, and `Center`, plus a
  wide-text badge that verifies the push-out offset scales with width.
- `format_count` covers non-numeric pass-through, capping, negative
  values, and `max = 0`.
- Defaults match MudBlazor (`Color::Default`, `Overlap(false)`, `Max = 99`,
  `Origin::TopRight`, `Visible(true)`).
- Builders set every field including `badge_class`, `badge_aria_label`,
  and `on_click`.
- `resolve_badge_style` verifies default → primary fill, `bordered` →
  surface stroke, `elevation` → shadow, and explicit colour override.

Run with:

```bash
cargo test -p mud_egui_components mud_badge
```

---

## License

Provided under the same license as the surrounding `mud_egui` project.
See the repository root for details.

===== ./crates/mud_egui_components/src/data_display/carousel/README.md =====

# MudCarousel + MudCarouselItem for egui

Production-grade, MudBlazor-faithful ports of `MudCarousel<TData>.razor.cs`
and `MudCarouselItem.razor.cs` (MudBlazor 8.13.0) for the
[`egui`](https://github.com/emilk/egui) immediate-mode GUI library.

`MudCarousel` shows one item at a time with **arrows**, **bullets**, **swipe
gestures**, and **auto-cycling**. `MudCarouselItem` wraps each slide's
content and per-slide metadata (colour, transition, visibility).

---

## Table of contents

1. [Architecture](#architecture)
2. [API reference — `MudCarousel`](#api-reference--mudcarousel)
3. [API reference — `MudCarouselItem`](#api-reference--mudcarouselitem)
4. [Real-world examples](#real-world-examples)
5. [Auto-cycle & swipe internals](#auto-cycle--swipe-internals)
6. [Breaking changes & bug fixes](#breaking-changes--bug-fixes)
7. [Performance & design rationale](#performance--design-rationale)
8. [Testing](#testing)
9. [License](#license)

---

## Architecture

```
mud_carousel.rs
├── MudCarousel                       — configuration + render entry point
│   ├── show(ui, |ctx| …) -> (Response, usize, R)
│   ├── next / previous / move_to     — public navigation helpers
│   ├── maybe_auto_advance(..)        — memory-backed auto-cycle
│   └── handle_swipe(..)              — memory-backed total-delta swipe
├── CarouselCtx / CarouselItemEntry   — builder context for slides
├── Pure helpers (unit-tested):
│   ├── arrow_y(..)                   — Position → vertical arrow offset
│   ├── bullet_positions(..)          — Position + RTL → bullet centres
│   ├── swipe_direction(..)           — (dx, dy) → Next / Prev / None
│   ├── next_index / prev_index       — wrap-around
│   ├── clamp_index(..)               — safe range mapping
│   └── visible_indices(..)           — skip hidden items
└── CSS utilities                     — ComponentBase + BS/TW resolvers

mud_carousel_item.rs
├── MudCarouselItem                   — slide metadata + resolve_background
└── Tests                             — defaults, builders, colour resolution
```

The carousel is a **single-pass renderer**: it allocates one interaction
region for the whole slide area, paints the selected slide, then overlays
arrows and bullets with absolute hit-test rects (`ui.interact`), so neither
affects the parent's layout — matching MudBlazor's CSS absolute positioning.

---

## API reference — `MudCarousel`

| Method | Type | Default | MudBlazor equivalent | Description |
|---|---|---|---|---|
| `new()` | `Self` | — | — | MudBlazor defaults. |
| `auto_cycle(b)` | `bool` | `false` | `AutoCycle` | Enable auto-cycling. |
| `auto_cycle_time(t)` | `Duration` | `Duration::ZERO` | `AutoCycleTime` | Interval; `ZERO` disables cycling. |
| `enable_swipe_gesture(b)` | `bool` | `true` | `EnableSwipeGesture` | Horizontal drag navigation. |
| `selected_index(i)` | `usize` | `0` | `SelectedIndex` | Position in the visible list. |
| `show_arrows(b)` | `bool` | `true` | `ShowArrows` | Show prev/next arrows. |
| `show_bullets(b)` | `bool` | `true` | `ShowBullets` | Show bullet strip. |
| `arrows_position(p)` | `Position` | `Center` | `ArrowsPosition` | `Top` / `Center` / `Bottom`. |
| `bullets_position(p)` | `Position` | `Bottom` | `BulletsPosition` | `Bottom` / `Top` / `Left` / `Right` / `Center` / `Start` / `End`. |
| `bullets_color(c)` | `Color` | `None` | `BulletsColor` | Override bullet colour. |
| `bullets_class(s)` | `impl Into<String>` | `None` | `BulletsClass` | Utility classes for bullets. |
| `navigation_buttons_class(s)` | `impl Into<String>` | `None` | `NavigationButtonsClass` | Utility classes for arrows. |
| `bullet_template(f)` | `FnMut(&mut Ui, bool)` | `None` | `BulletTemplate` | Custom bullet renderer. |
| `next_button_template(f)` | `FnMut(&mut Ui)` | `None` | `NextButtonTemplate` | Custom next-arrow renderer. |
| `previous_button_template(f)` | `FnMut(&mut Ui)` | `None` | `PreviousButtonTemplate` | Custom prev-arrow renderer. |
| `checked_icon(s)` | `impl Into<String>` | `RADIO_BUTTON_CHECKED` | `CheckedIcon` | Selected-bullet icon. |
| `unchecked_icon(s)` | `impl Into<String>` | `RADIO_BUTTON_UNCHECKED` | `UncheckedIcon` | Non-selected bullet icon. |
| `previous_icon(s)` | `impl Into<String>` | `NAVIGATE_BEFORE` | `PreviousIcon` | Prev-arrow icon. |
| `next_icon(s)` | `impl Into<String>` | `NAVIGATE_NEXT` | `NextIcon` | Next-arrow icon. |
| `height(h)` | `f32` | `200.0` | *(via `Style`)* | Explicit height. |
| `right_to_left(b)` | `bool` | `false` | `RightToLeft` | RTL arrows + `Start`/`End` bullets. |
| `on_selected_index_change(f)` | `FnMut(usize)` | `None` | `SelectedIndexChanged` | Selection callback (extension). |
| `flavor(f)` | `StyleFlavor` | `Bootstrap` | — | Utility-class resolver. |
| `classes(iter)` | `iter of AsRef<str>` | — | `Class` | Wrapper utility classes. |
| `show(ui, |ctx| …)` | `&mut Ui, FnOnce(&mut CarouselCtx) -> R` | — | `ChildContent` | Render and return `(Response, usize, R)`. |

### Navigation helpers

| Method | Signature | Description |
|---|---|---|
| `move_to` | `(&mut self, index: usize, count: usize)` | Clamped absolute jump. |
| `next` | `(&mut self, count: usize)` | Advance with wrap. |
| `previous` | `(&mut self, count: usize)` | Retreat with wrap. |

### `Position` mapping

| Variant | Arrows | Bullets |
|---|---|---|
| `Top` | top edge | horizontal, top |
| `Center` | vertical middle | horizontal, middle |
| `Bottom` | bottom edge | horizontal, bottom *(default)* |
| `Left` / `Start` | falls back to `Center` | vertical, left *(right in RTL)* |
| `Right` / `End` | falls back to `Center` | vertical, right *(left in RTL)* |

---

## API reference — `MudCarouselItem`

| Method | Type | Default | MudBlazor equivalent | Description |
|---|---|---|---|---|
| `new()` | `Self` | — | — | Defaults: `Color::Default`, `Transition::Slide`, `visible = true`. |
| `color(c)` | `Color` | `Default` | `Color` | Background colour role. |
| `custom_transition_enter(s)` | `impl Into<String>` | `None` | `CustomTransitionEnter` | Enter transition (stored; not painted). |
| `custom_transition_exit(s)` | `impl Into<String>` | `None` | `CustomTransitionExit` | Exit transition (stored; not painted). |
| `transition(t)` | `Transition` | `Slide` | `Transition` | Navigation effect. |
| `right_to_left(b)` | `bool` | `false` | `RightToLeft` (cascading) | RTL content flow. |
| `visible(b)` | `bool` | `true` | *(get-only)* | **Extension**: hide from render/nav/bullets. |
| `resolve_background(palette)` | `() -> Option<Color32>` | — | — | `None` for `Default`/`Inherit`. |

### `show` return value

```rust
let (response, selected_index, user_result) = carousel.show(ui, |ctx| {
    ctx.slide(|ui| { ui.label("Slide 1"); });
    ctx.item(
        MudCarouselItem::new().color(Color::Primary),
        |ui| { ui.label("Slide 2"); },
    );
    42 // returned as user_result
});
```

Persist `selected_index` into your own state (or use
`on_selected_index_change`).

---

## Real-world examples

### Basic carousel with auto-cycle

```rust
use std::time::Duration;
use mud_egui_components::MudCarousel;
use mud_egui_core::enums::Color;

// Persist `selected` somewhere across frames (e.g. a struct field).
let (_, selected, _) = MudCarousel::new()
    .auto_cycle(true)
    .auto_cycle_time(Duration::from_secs(4))
    .height(280.0)
    .show(ui, |c| {
        c.slide(|ui| ui.label("Welcome"));
        c.item(
            MudCarouselItem::new().color(Color::Primary),
            |ui| ui.label("Primary slide"),
        );
        c.item(
            MudCarouselItem::new().color(Color::Secondary),
            |ui| ui.label("Secondary slide"),
        );
    });
self.selected = selected;
```

### Swipe-only (no arrows, no bullets)

```rust
MudCarousel::new()
    .show_arrows(false)
    .show_bullets(false)
    .enable_swipe_gesture(true)
    .show(ui, |c| {
        c.slide(|ui| ui.label("Swipe me"));
        c.slide(|ui| ui.label("Or me"));
    });
```

### Custom bullets and arrows

```rust
MudCarousel::new()
    .bullets_position(Position::Top)
    .bullets_color(Color::Success)
    .checked_icon(filled::STAR)
    .unchecked_icon(filled::STAR_BORDER)
    .next_icon(filled::CHEVRON_RIGHT)
    .previous_icon(filled::CHEVRON_LEFT)
    .show(ui, |c| { /* … */ });
```

### Right-to-left layout

```rust
MudCarousel::new()
    .right_to_left(true)
    .bullets_position(Position::Start) // start edge = right edge in RTL
    .show(ui, |c| { /* … */ });
```

### Custom bullet template

```rust
MudCarousel::new()
    .bullet_template(|ui, is_selected| {
        let color = if is_selected { Color32::WHITE } else { Color32::GRAY };
        ui.painter().circle_filled(
            ui.max_rect().center(),
            4.0,
            color,
        );
    })
    .show(ui, |c| { /* … */ });
```

### Persist state via callback

```rust
let mut page = MyPage {
    carousel: MudCarousel::new().on_selected_index_change(|i| {
        log::info!("carousel moved to {i}");
    }),
    selected: 0,
};
let (_, idx, _) = std::mem::take(&mut page.carousel).show(ui, |c| { /* … */ });
page.selected = idx;
```

---

## Auto-cycle & swipe internals

### Auto-cycle

- Requires **both** `auto_cycle(true)` **and** `auto_cycle_time > Duration::ZERO`
  (MudBlazor behaviour: a zero interval disables cycling).
- Timing is stored in `egui::Memory` under
  `response.id.with("auto_cycle_last")`, so rebuilding the `MudCarousel` every
  frame (the builder pattern) does not reset the clock.
- Calls `request_repaint_after(interval)` so the app wakes up exactly when the
  next advance is due — no idle spinning.

### Swipe

- Tracks the pointer press-origin (`swipe_start`) and the last in-drag
  position (`swipe_last`) in `egui::Memory`.
- On `drag_stopped`, computes the total delta (`last - start`) and compares
  against `SWIPE_THRESHOLD` (50 px).
- The original implementation used `Response::drag_delta()` on
  `drag_stopped()`, which returns the **per-frame** delta — almost always
  near zero on release, so swipes never triggered. This is fixed.

---

## Breaking changes & bug fixes

| Area | Before | After |
|---|---|---|
| Auto-cycle | Fields existed but were never read | Implemented (memory-backed) |
| Swipe | `drag_delta()` on release (per-frame ≈ 0) | Total delta tracking in memory |
| `AutoCycleTime` default | `5s` | `Duration::ZERO` (MudBlazor parity) |
| `ui.id()` | Deprecated | `ui.scope_id()` |
| Arrow/bullet interaction IDs | Shared across carousel instances | Salted from `response.id` |
| `MudCarouselItem::visible` | Ignored | Filters render/nav/bullets |
| Arrows/bullets position | Hard-coded | `Position` enum |
| Utility classes | None | `ComponentBase` + BS/TW resolvers |

---

## Performance & design rationale

| Technique | Benefit |
|---|---|
| **Single interaction region** | One `allocate_exact_size` for the whole carousel; arrows and bullets use `ui.interact` on absolute rects, so no layout side effects. |
| **Pure helpers** | `arrow_y`, `bullet_positions`, `swipe_direction`, `next_index`, `prev_index`, `clamp_index`, `visible_indices` are all `#[inline]` pure functions, unit-tested without `egui::Context`. |
| **Memory-backed state** | Auto-cycle timing and swipe tracking live in `egui::Memory`, so the builder pattern (consuming `self`) works without forcing `&mut self`. |
| **Visible filtering** | `Vec<usize>` of visible indices built once per frame; no per-frame allocation for the common case where all items are visible (still allocates a small `Vec`, but avoids repeated filtering in navigation). |
| **No per-frame clones** | Icons and templates are cloned only when used; templates are boxed `FnMut` and invoked in place. |
| **`request_repaint_after`** | Auto-cycle schedules the next frame exactly when needed; no busy-looping. |

---

## Testing

- **`mud_carousel.rs`** — 20+ tests:
  - `swipe_direction` threshold and dominance.
  - `next_index` / `prev_index` / `clamp_index` wrap and empty cases.
  - `visible_indices` filtering.
  - `arrow_y` for Top / Center / Bottom and fallbacks.
  - `bullet_positions` for Bottom / Top / Left / Right / Start with RTL,
    plus zero-count.
  - Defaults match MudBlazor.
  - Builders set every field.
  - Navigation helpers.
- **`mud_carousel_item.rs`** — 6 tests:
  - Defaults match MudBlazor.
  - Builders set every field.
  - `resolve_background` for `Default`, `Inherit`, and explicit colours.
  - `visible` extension.

Run with:

```bash
cargo test -p mud_egui_components mud_carousel
```

---

## License

Provided under the same license as the surrounding `mud_egui` project.
See the repository root for details.

===== ./crates/mud_egui_components/src/data_display/chip/README.md =====

# MudChip / MudChipSet for `mud_egui`

A compact, selectable, closable token and its container, ported from
MudBlazor's [`MudChip<T>`](https://mudblazor.com/components/chip) and
[`MudChipSet<T>`](https://mudblazor.com/components/chipset) (MudBlazor
8.13.0), for [egui](https://github.com/emilk/egui).

```rust
let mut set = MudChipSet::new()
    .selection_mode(SelectionMode::Multi)
    .color(Color::Primary)
    .check_mark(true)
    .all_closable(true);

set.show(ui, |ctx| {
    ctx.chip(MudChip::new("Rust").value("rs"));
    ctx.chip(MudChip::new("Go").value("go"));
    ctx.chip(MudChip::new("Zig").value("zig"));
});
```

Every label renders through **`MudText`**, every glyph through **`MudIcon`**.

---

## Contents

1. [Design principle: API parity, not API invention](#design-principle-api-parity-not-api-invention)
2. [Architecture](#architecture)
3. [API reference — MudChip](#api-reference--mudchip)
4. [API reference — MudChipSet](#api-reference--mudchipset)
5. [Examples](#examples)
6. [Selection and cascade semantics](#selection-and-cascade-semantics)
7. [Performance and memory choices](#performance-and-memory-choices)
8. [Testing](#testing)
9. [Differences from MudBlazor](#differences-from-mudblazor)

---

## Design principle: API parity, not API invention

The MudBlazor 8.13.0 parameter surface for `MudChip<T>` and `MudChipSet<T>`
was verified against DNDocs before this port.

### Ported `MudChip` parameters

| MudBlazor | Rust method | Default |
|-----------|-------------|---------|
| `CheckMark` | `check_mark(bool)` | `false` |
| `CheckedIcon` | `checked_icon(s)` | `filled::CHECK` |
| `ChildContent` | `new(s)` / `text(s)` | required |
| `CloseIcon` | `close_icon(s)` | `filled::CANCEL` |
| `Color` | `color(Color)` | `Color::Default` |
| `Disabled` | `disabled(bool)` | `false` |
| `Icon` | `icon(s)` | `None` |
| `IconColor` | `icon_color(Color)` | `Color::Inherit` |
| `Label` | `label(bool)` | `false` |
| `OnClick` | `on_click(f)` | `None` |
| `OnClose` | `on_closed(f)` | `None` |
| `Selected` | `selected(bool)` | `false` |
| `SelectedChanged` | `on_selected_changed(f)` | `None` |
| `SelectedColor` | `selected_color(Color)` | `None` |
| `Size` | `size(Size)` | `Size::Medium` |
| `Variant` | `variant(Variant)` | `Variant::Filled` |
| `Class` (base) | `classes(iter)` | `None` |
| `Id` (base) | `id(s)` | `None` |
| `Style` (base) | *(use `classes`)* | — |
| `UserAttributes` | **not ported** (no HTML tag) | — |

### Extensions (not in MudBlazor)

| Extension | Description |
|-----------|-------------|
| `value(s)` | Stable selection key (MudBlazor's `Value` is generic; the port uses `String`). |
| `closable(bool)` | Explicit close-button toggle. MudBlazor derives it from `OnClose.HasDelegate`. |
| `elevation(u8)` | Drop-shadow depth (0–24). Not a `MudChip` parameter. |
| `aria_label(s)` | Accessible label. |
| `flavor(StyleFlavor)` | Bootstrap or Tailwind resolver. |

### Ported `MudChipSet` parameters

| MudBlazor | Rust method | Default |
|-----------|-------------|---------|
| `SelectionMode` | `selection_mode(SelectionMode)` | `Single` |
| `SelectedValue` | `selected_value(s)` / field | `None` |
| `SelectedValues` | `selected_values(iter)` / field | `{}` |
| `Mandatory` | `mandatory(bool)` | `false` |
| `Disabled` | `disabled(bool)` | `false` |
| `ReadOnly` | `read_only(bool)` | `false` |
| `AllClosable` | `all_closable(bool)` | `false` |
| `Color` | `color(Color)` | `None` |
| `IconColor` | `icon_color(Color)` | `None` |
| `SelectedColor` | `selected_color(Color)` | `None` |
| `Variant` | `variant(Variant)` | `None` |
| `Size` | `size(Size)` | `None` |
| `Label` | `label(bool)` | `None` |
| `CheckMark` | `check_mark(bool)` | `false` |
| `CheckedIcon` | `checked_icon(s)` | `None` |
| `CloseIcon` | `close_icon(s)` | `None` |
| `OnSelectedValueChanged` | `on_selected_value_changed(f)` | `None` |
| `OnSelectedValuesChanged` | `on_selected_values_changed(f)` | `None` |
| `OnClose` | `on_close(f)` | `None` |

### Deliberately not ported

- **`Comparer`** — Rust `PartialEq` on the key is the equivalent.
- **`Href` / `Target` / `ForceLoad` / `Rel`** on `MudChip` — use
  [`MudLink`](../link/README.md) for hyperlink behaviour. The port keeps
  `MudChip` as a purely interactive token.
- **`Style` / `UserAttributes`** — no CSS strings or HTML tags in egui.

---

## Architecture

```
mud_chip.rs
├── Pure helpers
│   ├── with_alpha(c, alpha)
│   ├── resolve_color(c, theme, default)
│   ├── effective_variant(selected, variant)
│   └── chip_text_color(disabled, variant, accent, palette)
├── ChipMetrics
│   ├── resolve(size, label)
│   └── close_icon_size()
├── MudChip       — struct + builders + show
└── mod tests     — 20+ pure-function tests

mud_chipset.rs
├── MudChipSet    — struct + builders + show
├── MudChipSetCtx — collector with `chip(..)` helper
└── mod tests     — 6 pure-function tests
```

**Pure layer.** `with_alpha`, `resolve_color`, `effective_variant`,
`chip_text_color`, `ChipMetrics::resolve`, and `ChipMetrics::close_icon_size`
are free functions or methods that do not touch `egui::Context` and are
covered by unit tests.

**State layer.** `MudChipSet` is caller-owned; its fields are the equivalent
of MudBlazor's two-way-bound `SelectedValue` / `SelectedValues`.

**Component layer.** `MudChip::show` measures the label, allocates the chip
rect, paints the background + border + focus ring + elevation, then paints
the leading icon (`MudIcon`), label (`MudText`), and close button (`MudIcon`).

---

## API reference — `MudChip`

| Method | Type | Default | Description |
|--------|------|---------|-------------|
| `new(text)` | `impl Into<String>` | — | Full constructor. |
| `text(s)` | `impl Into<String>` | `""` | Label text. |
| `value(s)` | `impl Into<String>` | `None` | Selection key. |
| `selected(b)` | `bool` | `false` | Selected state. |
| `disabled(b)` | `bool` | `false` | Disabled state. |
| `color(c)` | `Color` | `Default` | Accent colour. |
| `icon_color(c)` | `Color` | `Inherit` | Leading-icon colour. |
| `selected_color(c)` | `Color` | `None` | Accent override when selected. |
| `variant(v)` | `Variant` | `Filled` | Base variant. |
| `size(s)` | `Size` | `Medium` | Size preset. |
| `label(b)` | `bool` | `false` | Small-radius chip. |
| `icon(s)` | `impl Into<String>` | `None` | Leading icon. |
| `checked_icon(s)` | `impl Into<String>` | `CHECK` | Check glyph. |
| `close_icon(s)` | `impl Into<String>` | `CANCEL` | Close-button glyph. |
| `check_mark(b)` | `bool` | `false` | Show check when selected. |
| `closable(b)` | `bool` | `false` | Show close button. |
| `elevation(e)` | `u8` | `1` / `3` | Drop shadow (filled only). |
| `id(s)` | `impl Into<String>` | `None` | Stable identity. |
| `aria_label(s)` | `impl Into<String>` | `None` | Accessible label. |
| `classes(iter)` | `iter of AsRef<str>` | none | Utility classes. |
| `flavor(f)` | `StyleFlavor` | `Bootstrap` | Utility resolver. |
| `on_click(f)` | `FnMut() + 'static` | `None` | Body-click callback. |
| `on_closed(f)` | `FnMut() + 'static` | `None` | Close callback. |
| `on_selected_changed(f)` | `FnMut(bool) + 'static` | `None` | Selection callback. |
| `show(ui)` | `&mut Ui` | — | Returns `(Response, bool)`. |

### `show` return value

```rust
let (body_resp, close_clicked) = chip.show(ui);
if body_resp.clicked() { /* chip body clicked */ }
if close_clicked { /* close button clicked */ }
```

`body_resp` is the chip body's hit-rect response — **not** the wrapper
frame's hover-only response, so `.clicked()` reflects real clicks.

---

## API reference — `MudChipSet`

| Method | Type | Default | Description |
|--------|------|---------|-------------|
| `new()` | — | — | Default set (`Single` mode). |
| `selection_mode(m)` | `SelectionMode` | `Single` | `Single` / `Toggle` / `Multi`. |
| `selected_value(s)` | `impl Into<String>` | `None` | Single-selection value. |
| `selected_values(iter)` | `iter of Into<String>` | `{}` | Multi-selection values. |
| `mandatory(b)` | `bool` | `false` | Prevents deselecting the last chip. |
| `disabled(b)` | `bool` | `false` | Disables all chips. |
| `read_only(b)` | `bool` | `false` | Prevents selection changes. |
| `all_closable(b)` | `bool` | `false` | Cascades `closable` to every chip. |
| `color(c)` | `Color` | `None` | Cascades to `MudChip::color`. |
| `icon_color(c)` | `Color` | `None` | Cascades. |
| `selected_color(c)` | `Color` | `None` | Cascades. |
| `variant(v)` | `Variant` | `None` | Cascades. |
| `size(s)` | `Size` | `None` | Cascades. |
| `label(b)` | `bool` | `None` | Cascades. |
| `check_mark(b)` | `bool` | `false` | Cascades. |
| `checked_icon(s)` | `impl Into<String>` | `None` | Cascades. |
| `close_icon(s)` | `impl Into<String>` | `None` | Cascades. |
| `on_selected_value_changed(f)` | `FnMut(Option<String>)` | `None` | Fired on single-selection change. |
| `on_selected_values_changed(f)` | `FnMut(HashSet<String>)` | `None` | Fired on multi-selection change. |
| `on_close(f)` | `FnMut(String)` | `None` | Fired when a chip closes. |
| `show(ui, \|ctx\| ..)` | `&mut Ui, FnOnce(&mut MudChipSetCtx) -> R` | — | Returns `R`. |

### `MudChipSetCtx`

| Method | Description |
|--------|-------------|
| `chip(chip)` | Cascades defaults, renders, handles selection/close. Returns `true` if state changed. |
| `selected_value()` | `Option<&String>` |
| `selected_values()` | `&HashSet<String>` |
| `set` / `ui` | Public fields exposing the underlying state and `Ui`. |

---

## Examples

### Basic chip

```rust
ui.add(MudChip::new("Rust"));
```

### With icon and closable

```rust
MudChip::new("Filter: active")
    .icon(filled::FILTER_ALT)
    .closable(true)
    .on_closed(|| println!("closed"))
    .show(ui);
```

### Selected with check mark

```rust
MudChip::new("Selected")
    .selected(true)
    .check_mark(true)
    .show(ui);
```

### Outlined / text variants

```rust
MudChip::new("Outlined").variant(Variant::Outlined).show(ui);
MudChip::new("Text").variant(Variant::Text).show(ui);
```

### Label style (small radius)

```rust
MudChip::new("LABEL").label(true).show(ui);
```

### Sizes

```rust
MudChip::new("Small").size(Size::Small).show(ui);
MudChip::new("Medium").size(Size::Medium).show(ui);
MudChip::new("Large").size(Size::Large).show(ui);
```

### Chip set — single selection (default)

```rust
let mut set = MudChipSet::new();
set.show(ui, |ctx| {
    for tag in ["all", "active", "done"] {
        ctx.chip(MudChip::new(tag).value(tag));
    }
});
// After the frame, `set.selected_value` holds the clicked tag.
```

### Chip set — multi selection

```rust
let mut set = MudChipSet::new().selection_mode(SelectionMode::Multi);
set.show(ui, |ctx| {
    for tag in ["rust", "go", "zig"] {
        ctx.chip(MudChip::new(tag));
    }
});
for tag in &set.selected_values {
    println!("selected: {tag}");
}
```

### Chip set — toggle with mandatory

```rust
let mut set = MudChipSet::new()
    .selection_mode(SelectionMode::Toggle)
    .mandatory(true)
    .selected_value("default");

set.show(ui, |ctx| {
    for tag in ["default", "custom"] {
        ctx.chip(MudChip::new(tag).value(tag));
    }
});
```

Clicking the currently selected chip does **not** clear the selection.

### Chip set — cascading defaults

```rust
let mut set = MudChipSet::new()
    .color(Color::Secondary)
    .size(Size::Small)
    .check_mark(true)
    .all_closable(true);

set.show(ui, |ctx| {
    // Every chip inherits color=Secondary, size=Small, check_mark, closable.
    ctx.chip(MudChip::new("A"));
    // Override per-chip:
    ctx.chip(MudChip::new("B").color(Color::Error));
});
```

### Chip set — change callbacks

```rust
let mut set = MudChipSet::new()
    .on_selected_value_changed(|value| {
        log::info!("selected: {value:?}");
    })
    .on_close(|value| {
        log::info!("closed chip: {value}");
    });
```

---

## Selection and cascade semantics

### Selection modes

| Mode | Click unselected | Click selected | `mandatory` interaction |
|------|------------------|----------------|-------------------------|
| `Single` | selects it | replaces the selection | `mandatory` does **not** apply (there is always a selection) |
| `Toggle` | selects it | deselects it | `mandatory(true)` prevents deselecting the last chip |
| `Multi` | adds it | removes it | `mandatory` does not apply |

`read_only(true)` freezes selection but still fires `on_click` and
`on_closed` — matching MudBlazor's `ReadOnly` semantics.

### Cascading defaults

`MudChipSetCtx::chip` fills in the chip's optional fields from the set when
they are `None` / `false`. The precedence is:

```
chip.field  >  set.field  >  MudChip default
```

| Field | Chip-set → chip cascade |
|-------|-------------------------|
| `color` | When `chip.color.is_none()` |
| `icon_color` | When `chip.icon_color.is_none()` |
| `selected_color` | When `chip.selected_color.is_none()` |
| `variant` | When `chip.variant.is_none()` |
| `size` | When `chip.size.is_none()` |
| `label` | When `chip.label.is_none()` |
| `check_mark` | When `chip.show_check_mark == false` |
| `checked_icon` | When `chip.checked_icon.is_none()` |
| `close_icon` | When `chip.close_icon.is_none()` |
| `closable` | When `chip.closable == false` |
| `disabled` | OR-ed with `set.disabled` |

### Callback ordering

For one chip-set frame:

1. Chip body click → `chip.on_click` (if any) → chip internal toggle →
   `chip.on_selected_changed` (if any, with the new chip-local value).
2. Then the chipset updates `selected_value` / `selected_values` and fires
   `set.on_selected_value_changed` / `set.on_selected_values_changed`.
3. Close button click → `chip.on_closed` → then `set.on_close`.

Chips that live inside a set should not rely on their own
`on_selected_changed` value; use the set's callback instead.

---

## Performance and memory choices

| Choice | Effect |
|--------|--------|
| Label width measured once per frame via `layout_no_wrap` | No text layout twice; the same `FontId` is reused. |
| Single `MudText` for the label | No per-chip copy of the theme font. |
| Pure `ChipMetrics::resolve` | No per-frame `match` on `Size` inside the paint path. |
| `effective_variant` is a pure `match` | Fast, easy to test. |
| `with_alpha` early-outs on `alpha >= 1.0` and `<= 0.0` | Avoids the `from_rgba_unmultiplied` call for identity. |
| `MudChipSet` uses `HashSet<String>` for Multi | O(1) membership; matches MudBlazor's semantics. |
| `MudChipSet::chip` returns `bool` | Cheap to ignore or inspect. |
| Chip's `id` feeds `ui.scope_id().with(id)` | Stable interaction ids; no collisions between chips. |
| Close interaction is scoped to its own rect | Clicking close does not also register as a body click. |

---

## Testing

`cargo test` runs 20+ pure-function tests in `mud_chip.rs`:

| Area | Tests |
|------|-------|
| `with_alpha` | full alpha identity, zero → transparent, fractional scaling |
| `resolve_color` | `Default` / `Inherit` → fallback, `Primary` → palette primary |
| `effective_variant` | unselected → base, selected `Filled↔Text`, `Outlined` unchanged, `None` → `Text` |
| `chip_text_color` | disabled → `DISABLED_ALPHA`, filled → contrast, outlined/text → accent |
| `ChipMetrics` | Small / Medium / Large exact values, label forces radius 4, unknown size → Medium |
| `close_icon_size` | scales and clamps to `CLOSE_ICON_MIN` |
| Defaults | match MudBlazor |
| Builders | every field set, `flavor` |

Plus 6 pure-function tests in `mud_chipset.rs`:

| Area | Tests |
|------|-------|
| Defaults | `selection_mode == Single`, empty state |
| Builders | selection state, cascade defaults |
| `mandatory` | flag is set |
| Cascading | `color` / `size` / `variant` are `Some` |

Rendering needs a live `egui::Context`; the pure layer is the source of
truth.

```bash
cargo test -p mud_egui_components mud_chip
```

---

## Differences from MudBlazor

- **`value` is `String`** rather than generic `T`. MudBlazor's `MudChip<T>`
  is generic; the Rust port uses `String` because `MudChipSet` uses string
  keys for its `HashSet`. If you need typed values, store the mapping
  externally.
- **`closable` is explicit.** MudBlazor shows the close button when
  `OnClose.HasDelegate` is true. The port lets the caller force it with
  `closable(true)`, which is what `MudChipSet::all_closable` needs.
- **`elevation` is an extension.** Not a `MudChip` parameter; the port
  defaults filled chips to 1 on rest and 3 on hover, matching
  `MudBadge`'s scale.
- **`Href` / `Target` / `ForceLoad` / `Rel` not ported.** Use `MudLink`.
- **`Style` → `classes(..)`.** egui has no inline CSS strings.
- **`UserAttributes` not ported.** No HTML tag.
- **`MudChip::show` returns `(Response, bool)`.** The `Response` is the body
  hit-rect (so `.clicked()` reflects clicks) and `bool` is `close_clicked`.
  `ui.add(MudChip::new(..))` still works via the `Widget` impl and discards
  the `close_clicked` flag.

===== ./crates/mud_egui_components/src/data_display/data_grid/README.md =====

# MudDataGrid for `mud_egui`

A sortable, filterable, paginated data grid with row selection for
[egui](https://github.com/emilk/egui), ported from MudBlazor's
[`MudDataGrid<T>`](https://mudblazor.com/components/datagrid) and `Column<T>`.

```rust
let mut state = DataGridState::<Person>::new();

MudDataGrid::new()
    .filterable(true)
    .multi_selection(true)
    .striped(true)
    .show(ui, &people, &mut state, |grid| {
        grid.text_column("name", "Name", |p| p.name.clone())
            .sort_by(|a, b| a.name.cmp(&b.name));
        grid.text_column("city", "City", |p| p.city.clone());
        grid.column(
            Column::new("age", "Age", |ui, p| {
                ui.add(MudText::new(p.age.to_string()).monospace(true));
            })
            .sort_by(|a, b| a.age.cmp(&b.age))
            .align(ColumnAlign::Right)
            .width(100.0),
        );
    });
```

The module is one file (`mud_data_grid.rs`) built from three layers:

| Layer | Contents |
|-------|----------|
| **Data** | `Column<'a, T>`, `SortDefinition`, `ColumnFilter`, `DataGridState<T>` |
| **Pure layout** | `sort_indices`, `filter_indices`, `resolve_column_widths`, `page_range`, `clamp_page`, `toggle_sort`, `toggle_selection`, `breakpoint_px`, `is_stacked` |
| **Component** | `MudDataGrid<'a, T>`, `GridUi<'a, T>`, header / filter-row / body / pager rendering |

---

## Contents

1. [What maps to MudBlazor, and what does not](#what-maps-to-mudblazor-and-what-does-not)
2. [Architecture](#architecture)
3. [API reference](#api-reference)
4. [Examples](#examples)
5. [Layout and data pipeline](#layout-and-data-pipeline)
6. [Performance and memory layout](#performance-and-memory-layout)
7. [Design rationale](#design-rationale)
8. [Differences from MudBlazor](#differences-from-mudblazor)
9. [Testing](#testing)

---

## What maps to MudBlazor, and what does not

The public API was checked against the MudBlazor 8.x `MudDataGrid<T>` and
`Column<T>` parameter lists before implementing. The grid is a combination of
the `MudGrid` layout discipline and the `MudDivider` rendering discipline:
pure layout math first, thin egui painting afterwards.

### Ported parameters

| MudBlazor | This crate | Notes |
|-----------|-----------|-------|
| `SortMode` | `sort_mode(SortMode)` | `None`, `Single`, `Multiple` (default `Multiple`) |
| `AllowUnsorted` | `allow_unsorted(bool)` | Third toggle state `None` |
| `Filterable` | `filterable(bool)` | Default `false` |
| `FilterMode` | `filter_mode(FilterMode)` | Only `ColumnFilterRow` has an egui equivalent |
| `FilterCaseSensitivity` | `filter_case_sensitive(bool)` | Default insensitive |
| `MultiSelection` | `multi_selection(bool)` | Default `false` |
| `SelectOnRowClick` | `select_on_row_click(bool)` | Default `true` |
| `SelectedItems` | `DataGridState::selected: Vec<T>` | Caller-owned binding |
| `SortDefinitions` | `DataGridState::sorts` | Caller-owned binding |
| `FilterDefinitions` | `DataGridState::filters` | Caller-owned binding |
| `CurrentPage` | `current_page(usize)` / state | Parameter or state |
| `RowsPerPage` | `rows_per_page(usize)` / state | Parameter or state |
| `Bordered` | `bordered(bool)` | Column dividers |
| `Outlined` | `outlined(bool)` | Outer border |
| `Hover` | `hover(bool)` | Row hover fill |
| `Striped` | `striped(bool)` | Alternating row fill |
| `Dense` | `dense(bool)` | Compact heights |
| `Elevation` | `elevation(u8)` | Shadow depth (default `1`) |
| `Square` | `square(bool)` | Removes corner rounding |
| `Breakpoint` | `breakpoint(Breakpoint)` | Responsive stacking |
| `Height` | `height(f32)` | Enables a vertical scroll area |
| `Loading` | `loading(bool)` | Overlay + spinner |
| `LoadingProgressColor` | `loading_progress_color(Color)` | Default `Info` |
| `NoRecordsContent` | `no_records_text(impl Into<String>)` | Text form |
| `RowClass` | `row_class(Fn(&T, usize) -> String)` | Resolved through the flavour |
| `RowClick` | `row_click(FnMut(&T, usize))` | |
| `QuickFilter` | `quick_filter(Fn(&T) -> bool)` | Predicate form |
| `ToolBarContent` | `toolbar(FnMut(&mut Ui))` | |
| `Class` | `classes([...])` | Bootstrap / Tailwind resolver |
| `Style` | `frame(egui::Frame)` | egui has no CSS strings |
| `Tag` | `tag(T)` / `tag_ref::<U>()` | `Arc<dyn Any + Send + Sync>` |

### Column parameters

| MudBlazor `Column<T>` | This crate | Notes |
|------------------------|-----------|-------|
| `Title` | `Column::new(id, title, cell)` | Required |
| `Sortable` (`bool?`) | `sortable(bool)` | `None` inherits; requires a comparator |
| `Filterable` (`bool?`) | `filterable(bool)` | `None` inherits the grid flag |
| `Hidden` | `hidden(bool)` | Hidden columns take no width |
| `Align` | `align(ColumnAlign)` | `Left` / `Center` / `Right` |
| `Width` | `width(f32)` | Fixed pixel width |
| `CellTemplate` | the `cell` closure | `Fn(&mut Ui, &T)` |
| `Property` (PropertyColumn) | `Column::text(id, title, get)` | Extractor powers cell + filter |
| `Comparer` | `sort_by(Fn(&T, &T) -> Ordering)` | Explicit comparator |
| `Filterable` text source | `filter_text(Fn(&T) -> String)` | Extractor for filters / search |

### Deliberately not ported

These require DOM/browser features or server-side plumbing that have no
faithful egui equivalent in this crate. Adding no-op builders would be API
invention, which contradicts the parity principle:

- `ServerData`, `VirtualizeServerData`, `Virtualize`, `OverscanCount`,
  `RowLoadingContent` — server data and windowed rendering.
- `EditMode`, `ReadOnly`, `CanCancelEdit`, `CloneStrategy`,
  `StartedEditingItem`, `CommittedItemChanges`, `CanceledEditingItem`,
  `FormFieldChanged`, `Validator` — inline/form editing.
- `Groupable`, `GroupDefinitions`, `IsGrouped`, group expansion — grouping.
- `ColumnResizeMode`, `DragDropColumnReordering`, `ColumnsPanelReordering`,
  `ShowColumnOptions`, `ShowMenuIcon`, `ColumnsPanel` — column chrome.
- `FilterMode::Simple` and `FilterMode::ColumnFilterMenu` — popover filtering.
- `ColGroup`, `ChildRowContent`, `HierarchyColumn`, `ExpandSingleRow` —
  structural row content.

`MudDataGrid` also exposes egui-friendly extensions: `show_search(bool)` (a
built-in search box) and `DataGridState::last_row_rects` for hit-testing.

---

## Architecture

```
mud_data_grid.rs
├── Column<'a, T>              — id, title, flags, align, width,
│                                cell, comparator, filter extractor
├── SortDefinition             — column id + direction
├── ColumnFilter               — column id + text
├── DataGridState<T>           — sorts, filters, page, selection, outputs
├── GridUi<'a, T>              — collector for Column values
├── MudDataGrid<'a, T>         — configuration + rendering
│   └── show(ui, items, state, |grid| ..) -> (Response, R)
├── Pure helpers               — sort_indices, filter_indices,
│                                resolve_column_widths, page_range,
│                                clamp_page, toggle_sort, toggle_selection,
│                                breakpoint_px, is_stacked, row_height, …
└── Render helpers             — draw_header, draw_filter_row, draw_pager,
                                 paint_checkbox, row_fill
```

**Pure layer.** Every decision that can be made from data is a free function.
`sort_indices` and `filter_indices` operate on slices and index lists, so they
are covered by ordinary unit tests and can be reused headlessly (for exports,
snapshot tests, or tooling).

**State layer.** `DataGridState<T>` is caller-owned. It is the egui analogue
of MudBlazor's bindable parameters, and it also carries read-only outputs
(`last_row_rects`, `last_filtered_count`, `last_total_count`) for
introspection.

**Component layer.** `show` resolves the theme and wrapper frame, calls the
pure layer, allocates exact rectangles for header / filter row / body / pager,
and paints. It never measures text to place rows, which keeps layout
deterministic.

---

## API reference

### `MudDataGrid<'a, T>`

| Method | Default | Mirrors | Description |
|--------|---------|---------|-------------|
| `new()` | — | — | MudBlazor defaults. |
| `sort_mode(SortMode)` | `Multiple` | `SortMode` | `None` / `Single` / `Multiple`. |
| `allow_unsorted(bool)` | `false` | `AllowUnsorted` | Adds a third “no sort” toggle state. |
| `filterable(bool)` | `false` | `Filterable` | Enables the inline filter row. |
| `filter_mode(FilterMode)` | `ColumnFilterRow` | `FilterMode` | Only the inline row mode is ported. |
| `filter_case_sensitive(bool)` | `false` | `FilterCaseSensitivity` | String filter matching. |
| `show_search(bool)` | `false` | *(extension)* | Built-in search box writing `state.quick_filter_text`. |
| `multi_selection(bool)` | `false` | `MultiSelection` | Multiple selected rows. |
| `select_on_row_click(bool)` | `true` | `SelectOnRowClick` | Row click toggles selection. |
| `bordered(bool)` | `false` | `Bordered` | Column divider lines. |
| `outlined(bool)` | `false` | `Outlined` | Outer border. |
| `hover(bool)` | `false` | `Hover` | Row hover fill. |
| `striped(bool)` | `false` | `Striped` | Alternating row fill. |
| `dense(bool)` | `false` | `Dense` | Compact heights. |
| `square(bool)` | `false` | `Square` | No corner rounding. |
| `elevation(u8)` | `1` | `Elevation` | Shadow depth (clamped to 24). |
| `breakpoint(Breakpoint)` | `Xs` | `Breakpoint` | Stacks below this breakpoint. |
| `height(f32)` | `None` | `Height` | Enables a scrollable body. |
| `current_page(usize)` | `None` | `CurrentPage` | Overrides state for one call. |
| `rows_per_page(usize)` | `None` | `RowsPerPage` | Overrides state for one call. |
| `show_pager(bool)` | `true` | presence of `PagerContent` | Shows the footer pager. |
| `frame(egui::Frame)` | `None` | `Style` | Replaces the class-resolved frame. |
| `loading(bool)` | `false` | `Loading` | Overlay + spinner. |
| `loading_progress_color(Color)` | `Info` | `LoadingProgressColor` | Spinner tint. |
| `no_records_text(s)` | `"No records to display"` | `NoRecordsContent` | Empty-state text. |
| `aria_label(s)` | `None` | — | Published via `WidgetInfo`. |
| `row_class(f)` | `None` | `RowClass` | Per-row classes resolved through the flavour. |
| `row_click(f)` | `None` | `RowClick` | `FnMut(&T, usize)`. |
| `quick_filter(f)` | `None` | `QuickFilter` | `Fn(&T) -> bool`. |
| `toolbar(f)` | `None` | `ToolBarContent` | `FnMut(&mut Ui)`. |
| `flavor(StyleFlavor)` | default | — | Bootstrap or Tailwind resolver. |
| `classes(iter)` | empty | `Class` | Wrapper utility classes. |
| `tag(T)` / `tag_ref::<U>()` | `None` | `Tag` | Arbitrary user data. |
| `show(ui, items, state, \|grid\| ..)` | — | — | Draws and returns `(Response, R)`. |

### `Column<'a, T>`

| Method | Default | Mirrors | Description |
|--------|---------|---------|-------------|
| `new(id, title, cell)` | — | `Title` + `CellTemplate` | Custom cell renderer. |
| `text(id, title, get)` | — | `PropertyColumn` | Text cell + filter extractor. |
| `sortable(bool)` | inherit | `Sortable` | Requires a comparator to take effect. |
| `filterable(bool)` | inherit | `Filterable` | Requires a filter extractor. |
| `hidden(bool)` | `false` | `Hidden` | Excluded from layout. |
| `align(ColumnAlign)` | `Left` | `Align` | Header and cell alignment. |
| `width(f32)` | flex | `Width` | Fixed pixel width. |
| `sort_by(Fn(&T, &T) -> Ordering)` | `None` | `Comparer` | Sorting comparator. |
| `filter_text(Fn(&T) -> String)` | `None` | filter value | Filter/search extractor. |
| `is_sortable()` | — | — | `sortable.unwrap_or(true) && comparator.is_some()`. |
| `is_filterable(grid_flag)` | — | — | `filterable.unwrap_or(grid_flag) && extractor.is_some()`. |

### `DataGridState<T>`

| Field | Default | Description |
|-------|---------|-------------|
| `sorts` | `[]` | Active sort definitions, priority order. |
| `filters` | `[]` | Active per-column filters. |
| `current_page` | `0` | Zero-based page. |
| `rows_per_page` | `10` | `0` means all rows on one page. |
| `selected` | `[]` | Selected rows. |
| `quick_filter_text` | `""` | Search box contents. |
| `last_row_rects` | `[]` | Screen-space row rectangles from the last frame. |
| `last_filtered_count` | `0` | Rows after filtering in the last frame. |
| `last_total_count` | `0` | Rows passed to `show` in the last frame. |

### Enums

| Type | Variants | Default |
|------|----------|---------|
| `SortDirection` | `None`, `Ascending`, `Descending` | `None` |
| `SortMode` | `None`, `Single`, `Multiple` | `Multiple` |
| `ColumnAlign` | `Left`, `Center`, `Right` | `Left` |
| `FilterMode` | `None`, `ColumnFilterRow` | `ColumnFilterRow` |

---

## Examples

### Basic sortable grid

```rust
let mut state = DataGridState::<Person>::new();

MudDataGrid::new()
    .sort_mode(SortMode::Single)
    .show(ui, &people, &mut state, |grid| {
        grid.column(
            Column::text("name", "Name", |p| p.name.clone())
                .sort_by(|a, b| a.name.cmp(&b.name)),
        );
        grid.column(
            Column::text("age", "Age", |p| p.age.to_string())
                .sort_by(|a, b| a.age.cmp(&b.age))
                .align(ColumnAlign::Right)
                .width(80.0),
        );
    });
```

### Multi-select with a pager

```rust
MudDataGrid::new()
    .multi_selection(true)
    .rows_per_page(5)
    .striped(true)
    .hover(true)
    .show(ui, &people, &mut state, |grid| {
        for col in columns() { grid.column(col); }
    });

for person in &state.selected {
    ui.label(format!("selected: {}", person.name));
}
```

### Filter row + search box

```rust
MudDataGrid::new()
    .filterable(true)
    .show_search(true)
    .show(ui, &people, &mut state, |grid| {
        grid.text_column("name", "Name", |p| p.name.clone());
        grid.text_column("city", "City", |p| p.city.clone());
    });
```

Typing in the search box matches any filterable column; typing in a column's
filter row input narrows that column only. Both compose with AND.

### Responsive stacking

```rust
MudDataGrid::new()
    .breakpoint(Breakpoint::Md) // stack below 960px
    .show(ui, &people, &mut state, |grid| {
        for col in columns() { grid.column(col); }
    });
```

Below the breakpoint each row becomes a card with `title: value` pairs, which
is how MudBlazor's mobile layout behaves for data grids.

### Loading and empty states

```rust
MudDataGrid::new()
    .loading(fetch_in_flight)
    .no_records_text("Nothing here yet")
    .show(ui, &filtered, &mut state, |grid| {
        /* … */
    });
```

### Headless data shaping

The pure layer is public, so you can reuse the exact sort/filter/page logic
without drawing:

```rust
let filtered = filter_indices(&people, &state.filters, &cols, "", None, false);
let sorted = sort_indices(&people, &state.sorts, &cols, &filtered);
let (start, end) = page_range(sorted.len(), 0, 10);
let page = &sorted[start..end];
```

### Row click + custom row classes

```rust
MudDataGrid::new()
    .row_click(|person, idx| println!("clicked {} at row {}", person.name, idx))
    .row_class(|person, _| {
        if person.age > 30 { "bg-warning".into() } else { String::new() }
    })
    .show(ui, &people, &mut state, |grid| { /* … */ });
```

---

## Layout and data pipeline

```
items ──► filter_indices ──► sort_indices ──► page_range ──► visible rows
            │                   │                │
            filters             sorts            current_page / rows_per_page
            quick text          comparators      (clamped)
            quick predicate
```

1. **Filter.** Every active `ColumnFilter` must match (AND). The search box
   matches any filterable column. An optional `quick_filter` predicate is
   ANDed on top. Empty filter values are inactive.
2. **Sort.** The filtered index list is sorted with a stable multi-key
   comparator. Each `SortDefinition` contributes one key; columns without a
   comparator are skipped. `Descending` reverses the comparator result for
   that key only.
3. **Page.** `clamp_page` keeps `current_page` in range after the filter
   changes; `page_range` returns a half-open `[start, end)` slice.
4. **Widths.** `resolve_column_widths` reserves the selection column and all
   fixed widths, then splits the remainder equally among flexible columns.
   Hidden columns contribute nothing.
5. **Draw.** Header, optional filter row, body (scrollable when `height` is
   set), and pager are each allocated as exact rectangles. Cells are child
   `Ui`s clipped to their cell rectangle, so content cannot bleed across
   columns.

Rows are **not** measured: fixed row heights make the planner deterministic
and keep scroll offsets stable. Content taller than a row is clipped, matching
a table-like contract. If you need auto-height rows, use the responsive
stacked mode or a different container.

---

## Performance and memory layout

| Choice | Effect |
|--------|--------|
| `Column` is built once per frame from borrowed closures | No retained render tree. |
| `sort_indices` and `filter_indices` return `Vec<usize>` index lists | Sorting/filtering never moves `T` values; rows stay in place. |
| Multi-key sort uses `sort_by` with short-circuiting keys | One pass of comparisons, stable ordering. |
| `resolve_column_widths` is a single linear pass | No per-column allocation beyond the result vector. |
| Rows use `allocate_exact_size` + absolute child `Ui`s | No per-row layout passes; scroll height is known before drawing. |
| Filter inputs write into `state.filters` in place | No temporary filter map per frame. |
| Selection uses `Vec<T>` + `PartialEq` | No `Hash`/`Eq` bound on `T`; order is preserved. |
| `RowClass` resolution only runs when the callback is set | No resolver work in the common case. |
| `DataGridState` is caller-owned | Sorting/filtering state is not rebuilt each frame. |

Not done, on purpose: a column-width cache and a row-height cache. Both would
require invalidation rules that belong to a layout engine, not a single
widget; the grid stays deterministic instead.

---

## Design rationale

- **Pure data shaping.** Sorting, filtering, pagination, and width resolution
  are pure functions. This is what makes the component testable without a
  window and what keeps `show` a thin painter.
- **Explicit comparators and extractors.** Requiring `T: Ord` or
  `T: Display` would make the grid unusable for domain types. MudBlazor
  solves this with `Comparer` and `PropertyColumn.Property`; this port solves
  it with `sort_by` and `filter_text`.
- **No invented MudBlazor API.** Editing, grouping, virtualization, server
  data, and popover filtering are not present because their MudBlazor
  counterparts depend on DOM features. See the parity section.
- **Bindings become state.** `SelectedItems`, `SortDefinitions`, and
  `FilterDefinitions` are two-way bindings in Blazor; in egui they become
  fields on `DataGridState<T>` that the caller owns and the grid mutates.
- **One filter mode.** `ColumnFilterRow` is the only MudBlazor filter mode
  that does not need a popover, so it is the only one implemented. A column
  without a filter extractor is treated as non-filterable even if the flag is
  set.
- **Responsive stacking, not horizontal scrolling.** Below the configured
  breakpoint rows become label/value cards, which is what MudBlazor's
  `Breakpoint` parameter is for.
- **Never panic in a UI path.** Non-finite widths fall back to 1280 px,
  pages clamp, empty filter strings are inactive, and a missing comparator or
  extractor simply disables the feature instead of panicking.

---

## Differences from MudBlazor

- **`Style` is `frame(egui::Frame)`.** There are no inline CSS strings in
  egui; the typed frame is the equivalent.
- **`NoRecordsContent` and `ToolBarContent` are closures/text, not render
  fragments with parameters.** `toolbar` receives `&mut Ui`; the empty state
  is a text string.
- **`QuickFilter` is a predicate**, and the built-in search box is an
  extension that writes `state.quick_filter_text`. MudBlazor has no built-in
  search box; it expects you to wire `QuickFilter` yourself.
- **Only `FilterMode::ColumnFilterRow` exists.** `Simple` and
  `ColumnFilterMenu` need popovers; they are documented as not ported rather
  than stubbed.
- **Selection is `Vec<T>`, not `HashSet<T>`.** This keeps `T: PartialEq`
  instead of `T: Eq + Hash` and preserves click order for multi-selection.
- **`Breakpoint` uses Bootstrap pixel thresholds** (`Sm` = 600, `Md` = 960,
  `Lg` = 1280, `Xl` = 1920), matching MudBlazor's responsive behaviour.
- **Column `Sortable` / `Filterable` are `Option<bool>`** so `None` inherits
  from the grid, mirroring MudBlazor's nullable column overrides.
- **No column resizing, reordering, or visibility menu.** These are DOM /
  pointer-chrome features and are explicitly out of scope.

---

## Testing

`cargo test` runs the module's tests. Coverage by area:

| Area | Tests |
|------|-------|
| Sort direction | cycle with/without `AllowUnsorted`, `reversed` involution |
| `toggle_sort` | single replace, single clear on third toggle, multiple append, multiple remove, `SortMode::None` no-op |
| `sort_indices` | ascending, descending, multi-key city→age, missing comparator, empty sorts |
| `filter_indices` | case-insensitive, case-sensitive, empty value inactive, AND semantics, quick text over all filterable columns, quick predicate |
| Pagination | `page_count` (zero rows / zero size), `clamp_page`, `page_range` |
| Widths | equal split, selection column sum, hidden skip, non-finite repair |
| Selection | multi toggle preserves order, single replaces |
| Breakpoints | pixel mapping, active edge, strict stacking below |
| Heights | dense vs default row and header |
| Defaults | grid defaults match MudBlazor, column defaults |
| Builders | text column extractor, alignment mapping, tag round-trip |
| State | defaults, `with_rows_per_page`, filter-entry creation |

Rendering itself (`show`) needs a live `egui::Context` and is not unit-tested;
the pure layer is the source of truth for layout and data behaviour, exactly
as in `MudGrid`.

```bash
cargo test -p mud_egui_components mud_data_grid
```

===== ./crates/mud_egui_components/src/data_display/highlighter/README.md =====

# MudHighlighter for `mud_egui`

A themed text highlighter for [egui](https://github.com/emilk/egui), ported
from MudBlazor's [`MudHighlighter`](https://mudblazor.com/components/highlighter).
Renders a block of text with one or more search fragments visually emphasised,
using the existing **`MudText`** widget for every fragment so typography and
theme updates propagate automatically.

```rust
MudHighlighter::new("The quick brown fox jumps over the lazy dog")
    .highlighted_text("fox")
    .show(ui);
```

---

## Contents

1. [Design principle: API parity, not API invention](#design-principle-api-parity-not-api-invention)
2. [Architecture](#architecture)
3. [API reference](#api-reference)
4. [Visual behaviour](#visual-behaviour)
5. [Examples](#examples)
6. [Matching algorithm](#matching-algorithm)
7. [Performance and memory choices](#performance-and-memory-choices)
8. [Testing](#testing)
9. [Differences from MudBlazor](#differences-from-mudblazor)

---

## Design principle: API parity, not API invention

The MudBlazor `MudHighlighter` API was checked against the published API docs
and the source (`MudHighlighter.razor.cs`) before this port. The real component
has these parameters plus the shared `MudComponentBase` members:

| MudBlazor parameter | Type | Default |
|---------------------|------|---------|
| `Text` | `string?` | `null` |
| `HighlightedText` | `string?` | `null` |
| `HighlightedTexts` | `IEnumerable<string>?` | `null` |
| `CaseSensitive` | `bool` | `false` |
| `UntilNextBoundary` | `bool` | `false` |
| `Markup` | `bool` | `false` |
| `Class` (base) | `string?` | `null` |
| `Style` (base) | `string?` | `null` |
| `Tag` (base) | `object?` | `null` |
| `UserAttributes` (base) | `Dictionary<string, object>?` | — |

**`Markup` is deliberately not ported.** It tells Blazor to render `Text` as
an HTML `RenderFragment`, which requires a browser DOM. egui has no HTML
parser, so adding a no-op `markup(bool)` would be API invention. If you need
styled runs inside the text, compose multiple `MudHighlighter` / `MudText`
widgets inside a `ui.horizontal_wrapped`.

**`UserAttributes` is not ported** because egui has no HTML tag to splat
attributes onto.

**`Style` maps to `classes(..)`** (utility classes) because egui has no
inline CSS string mechanism. This matches the pattern used by `MudDivider`,
`MudGrid`, and `MudDataGrid`.

---

## Architecture

```
mud_highlighter.rs
├── Segment              — one plain or highlighted text run
├── find_ranges(..)      — pure: needles + text → sorted, merged byte ranges
├── split_segments(..)   — pure: text + ranges → Vec<Segment>
├── MudHighlighter       — builder + rendering
│   └── show(ui) -> Response
│       ├── resolve root frame  (BsStyle / TwStyle)
│       ├── find_ranges + split_segments
│       └── horizontal_wrapped: MudText per segment,
│           highlighted segments wrapped in a zero-padding Frame
└── mod tests            — pure-function tests (no egui context)
```

The module follows the same **pure-layer / thin-painter** split used by
`MudGrid` and `MudDataGrid`: all matching and segmentation logic is free
functions covered by unit tests, and `show` only resolves the theme, calls
those functions, and paints.

---

## API reference

### `MudHighlighter`

| Method | Type | Default | MudBlazor equivalent | Description |
|--------|------|---------|----------------------|-------------|
| `new(text)` | `impl Into<String>` | `""` | `Text` | Creates a highlighter for `text`. |
| `text(s)` | `impl Into<String>` | — | `Text` | Replaces the full text. |
| `highlighted_text(s)` | `impl Into<String>` | none | `HighlightedText` | Adds one fragment. |
| `highlighted_texts(iter)` | `IntoIterator<Item = impl Into<String>>` | none | `HighlightedTexts` | Adds several fragments. |
| `case_sensitive(b)` | `bool` | `false` | `CaseSensitive` | Match case exactly. |
| `until_next_boundary(b)` | `bool` | `false` | `UntilNextBoundary` | Extend each match to the next word boundary. |
| `highlight_color(c)` | `Color32` | `DEFAULT_HIGHLIGHT_COLOR` | *(extension)* | Background colour for highlighted fragments. |
| `classes(iter)` | `iter of AsRef<str>` | none | `Class` | Utility classes on the root frame. |
| `flavor(f)` | `StyleFlavor` | `Bootstrap` | — | Utility-class resolver. |
| `tag(t)` / `tag_ref::<U>()` | `T: Any + Send + Sync` | `None` | `Tag` | Attach and read user data. |
| `show(ui)` | `&mut Ui` | — | — | Render and return the root frame's `Response`. |

### Free functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `find_ranges` | `(&str, &[String], bool, bool) -> Vec<Range<usize>>` | Pure matcher; returns sorted, merged byte ranges. |
| `split_segments` | `(&str, &[Range<usize>]) -> Vec<Segment>` | Splits text into plain / highlighted runs. |

### `Segment`

| Field | Type | Description |
|-------|------|-------------|
| `text` | `String` | The fragment text. |
| `highlighted` | `bool` | Whether this fragment is emphasised. |

### Constants

| Name | Value | Description |
|------|-------|-------------|
| `DEFAULT_HIGHLIGHT_COLOR` | `rgba(255, 235, 59, 120)` | Semi-transparent Material Yellow that works in light and dark themes. |

---

## Visual behaviour

| Case | Behaviour |
|------|-----------|
| No matches | The full text renders as a single plain `MudText`. |
| One match | Text is split into up to three runs (before, match, after); the match receives the highlight background. |
| Overlapping matches | Merged into one contiguous highlighted range so nothing is painted twice. |
| `until_next_boundary` | Each match extends to the next `\w → \W` transition (matching regex `\b`). |
| `case_sensitive(false)` | Unicode-aware case-insensitive matching; multibyte text is safe. |
| Wrapping | Segments render inside `ui.horizontal_wrapped` with zero inter-item spacing, so the highlighted runs wrap naturally across lines. |
| Highlight background | A zero-padding `Frame` with a 2 px corner radius; the fill is `highlight_color`. |

Every text run is drawn through **`MudText::new(..).inline(true)`**, so the
font family, size, weight, and theme colour come from the surrounding UI
exactly as they do for any other `MudText`. Changing the theme propagates
immediately — no separate font configuration is needed.

---

## Examples

### Basic single-fragment highlight

```rust
MudHighlighter::new("The quick brown fox")
    .highlighted_text("quick")
    .show(ui);
```

### Multiple fragments

```rust
MudHighlighter::new("Red, green, and blue")
    .highlighted_texts(["red", "blue"])
    .show(ui);
```

### Case-sensitive matching

```rust
MudHighlighter::new("Rust and rust")
    .highlighted_text("Rust")
    .case_sensitive(true)
    .show(ui);
```

### Until-next-boundary (whole-word emphasis)

```rust
MudHighlighter::new("foobar barbazqux end")
    .highlighted_text("bar")
    .until_next_boundary(true)
    .show(ui);
// Highlights "barbazqux" (from the match to the next whitespace).
```

### Custom highlight colour

```rust
MudHighlighter::new("Styled highlight")
    .highlighted_text("highlight")
    .highlight_color(Color32::from_rgba_unmultiplied(0, 200, 83, 140))
    .show(ui);
```

### Inside a search-results list

```rust
for item in results {
    MudHighlighter::new(&item.name)
        .highlighted_text(&query)
        .show(ui);
    ui.add_space(4.0);
}
```

### Composing with other theme widgets

```rust
ui.horizontal(|ui| {
    ui.add(MudText::new("Result:").bold(true));
    MudHighlighter::new("Ada Lovelace")
        .highlighted_text("Love")
        .show(ui);
});
```

Because every fragment is a `MudText`, the highlighter inherits the same
typography context as its neighbours.

### Tag round-trip

```rust
let h = MudHighlighter::new("data")
    .highlighted_text("at")
    .tag("search-result-42");

assert_eq!(h.tag_ref::<&str>(), Some(&"search-result-42"));
h.show(ui);
```

---

## Matching algorithm

1. **Collect raw matches.** For each non-empty needle, scan the text
   character-by-character (Unicode-safe) and record every start/end pair.
   Case-insensitive matching uses per-character case folding, not
   `to_lowercase()` on the whole string, so byte offsets stay valid for
   multibyte text.
2. **Extend to boundary (optional).** If `until_next_boundary` is enabled,
   each match’s end advances while the next character is a word character
   (`\w`: alphanumeric or `_`), stopping at the first non-word character or the
   end of the text. This matches MudBlazor’s regex-boundary behaviour.
3. **Sort by start offset.** Deterministic ordering for overlapping matches.
4. **Merge overlapping ranges.** If the next range starts at or before the
   previous range’s end, extend the previous range’s end. This guarantees every
   byte belongs to at most one segment and prevents double-painted highlights.
5. **Split into segments.** Walk the sorted ranges and emit alternating
   plain / highlighted `Segment`s, including leading and trailing plain runs
   when the first match does not start at byte 0 or the last match does not end
   at `text.len()`.

Complexity: `O(n × m × k)` where `n` is text length in chars, `m` is the number
of needles, and `k` is the needle length. For typical highlighter use cases
(short needles, bounded text) this is linear in practice.

---

## Performance and memory choices

- **No retained render tree.** Columns and segments are built every frame
  from borrowed data; nothing accumulates across frames.
- **Zero-clone match results.** `find_ranges` returns ranges (24 bytes each),
  not substrings. `split_segments` allocates one `String` per segment —
  unavoidable because `MudText` owns its caption — but this is proportional to
  the number of matches, not the text length.
- **`horizontal_wrapped` with `item_spacing.x = 0`.** Segments flow without
  gaps and wrap naturally, without needing a single laid-out `LayoutJob`.
- **Pure functions.** `find_ranges` and `split_segments` take slices and
  return owned vectors; they do not touch `egui::Context` and are unit-tested
  without a window.
- **No per-frame allocation for plain text.** When there are no matches,
  `split_segments` returns a single `Segment` borrowing nothing extra; the
  cost is one `String` clone of the full text (same as any `MudText`).
- **Theme fetched once per frame.** `MudTheme::fetch` is called at the top of
  `show` and reused for both the frame resolver and the highlight colour
  lookup.
- **Unicode-safe.** All offsets are derived from `char_indices`, never from
  byte arithmetic on a lowercased copy, so multibyte text cannot panic or
  mis-slice.

Not done, on purpose: caching the match results across frames. The needles or
text can change every frame (as in a live search box), so a cache would need
invalidation on every call; a direct scan is simpler and has no steady-state
cost.

---

## Testing

`cargo test` runs the module’s tests:

| Area | Tests |
|------|-------|
| `find_ranges` | empty needles, empty text, simple match, multiple occurrences, case-insensitive default, case-sensitive, `until_next_boundary` at word end / whitespace / punctuation, overlapping merge, multiple needles, multibyte text, needle longer than text |
| `split_segments` | no ranges, leading / middle / trailing range, whole-text range |
| Builders | defaults match MudBlazor, every builder sets its field, `text(..)` replaces the previous value, tag round-trips by type |

There are **no** rendering tests because `show` needs a live `egui::Context`;
the pure layer is the source of truth for matching and segmentation, exactly as
in `MudGrid` and `MudDataGrid`.

```bash
cargo test -p mud_egui_components mud_highlighter
```

---

## Differences from MudBlazor

- **`Markup` not ported.** egui has no HTML parser; the parameter has no
  meaningful equivalent. Compose `MudHighlighter` / `MudText` widgets for
  styled runs instead.
- **`Style` → `classes(..)`.** egui has no inline CSS strings; utility classes
  are resolved through the Bootstrap or Tailwind flavour, same as every other
  `mud_egui` component.
- **`highlight_color(..)` is an extension.** MudBlazor relies on the browser’s
  default `<mark>` styling (yellow). egui has no browser default, so the port
  ships a theme-friendly default and an explicit override.
- **Case-insensitive matching is Unicode-aware.** MudBlazor uses
  `StringComparison.OrdinalIgnoreCase` (culture-invariant); this port folds
  per character, which is correct for the common cases (ASCII, Latin-1,
  Cyrillic, Greek) and never panics on multibyte input.
- **`until_next_boundary` extends to the right only.** This matches
  MudBlazor’s regex `\b` usage: the match starts where the needle starts and
  extends forward to the next boundary.

===== ./crates/mud_egui_components/src/data_display/icons/README.md =====

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

===== ./crates/mud_egui_components/src/data_display/image/README.md =====

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

===== ./crates/mud_egui_components/src/data_display/link/README.md =====

# MudLink for `mud_egui`

An inline, themed hyperlink for [egui](https://github.com/emilk/egui), ported
from MudBlazor's [`MudLink`](https://mudblazor.com/components/link) (MudBlazor
8.13.0). Supports **routing-aware navigation** through `mud_egui_navigation`'s
`NavigationManager`, **automatic external browser opening** for non-app URLs,
CSS-like utility classes, and full accessibility.

```rust
MudLink::new("/orders/42", "Order #42")
    .show(ui, nav);

MudLink::new("https://docs.rs", "Read the docs")
    .target("_blank")
    .show(ui, nav);   // → opens the system browser
```

Every label renders through **`MudText`**, so typography and theme updates
propagate automatically.

---

## Contents

1. [Design principle: API parity, not API invention](#design-principle-api-parity-not-api-invention)
2. [Architecture](#architecture)
3. [External vs. internal navigation](#external-vs-internal-navigation)
4. [API reference](#api-reference)
5. [Examples](#examples)
6. [Performance and memory choices](#performance--memory-choices)
7. [Testing](#testing)
8. [Differences from MudBlazor](#differences-from-mudblazor)

---

## Design principle: API parity, not API invention

The MudBlazor 8.13.0 `MudLink` parameter surface was verified against
[DNDocs](https://docs.dndocs.com) before this port. The real component has
exactly these parameters plus the shared `MudComponentBase` members.

### Ported parameters

| MudBlazor | Rust method | Default |
|-----------|-------------|---------|
| `ChildContent` | `label(s)` / `child(f)` | required (one of) |
| `Color` | `color(Color)` | `Color::Primary` |
| `Dense` | `dense(bool)` | `false` |
| `Disabled` | `disabled(bool)` | `false` |
| `ForceLoad` | `force_load(bool)` | `false` |
| `Href` | `new(href, label)` / `href(s)` | `None` |
| `OnClick` | `on_click(f)` | `None` |
| `Target` | `target(s)` | `None` |
| `Typo` | `typo(Typo)` | `Typo::Body1` |
| `Underline` | `underline(Underline)` | `Underline::Hover` |
| `Class` | `classes(iter)` | `None` |
| `Style` | *(use `classes`)* | — |
| `Tag` | `tag(t)` / `tag_ref::<U>()` | `None` |
| `UserAttributes` | **not ported** (no HTML tag) | — |

### Extensions (not in MudBlazor)

| Extension | Description |
|-----------|-------------|
| `id(Id)` | Stable identity for egui memory and accessibility. |
| `aria_label(s)` | Accessible label; falls back to the resolved label text. |
| `prevent_navigation(bool)` | Fires `on_click` without navigating. |
| `show_without_router(ui)` | Render without a `NavigationManager`; every `href` is treated as external. |
| `LinkResponse.action` | Reports what the link did this frame (`Internal` / `External` / `Vetoed` / `Error`). |

### `Underline` mapping

| MudBlazor `Underline` | Rust variant | Behaviour |
|-----------------------|--------------|-----------|
| `None = 0` | `Underline::None` | Never underlined. |
| `Hover = 1` | `Underline::Hover` (default) | Underlined only on hover. |
| `Always = 2` | `Underline::Always` | Always underlined. |

### Deliberately not ported

- **`Style`** — egui has no inline CSS strings. Use `classes(..)`.
- **`UserAttributes`** — there is no HTML tag to attach attributes to.
- **`Target="_parent"` / `"_top"` / `"framename"`** — accepted and stored,
  but only `_blank` has an effect in egui (it sets `OpenUrl::new_tab`). The
  others map to `new_tab: false`, matching the DOM where they only affect
  nested browsing contexts.

---

## Architecture

```
link/
├── mud_link.rs       — MudLink + LinkResponse
├── link_action.rs    — LinkAction (public enum)
├── external_url.rs   — is_external_url / scheme_of / origin_of / target_opens_new_tab
└── mod.rs            — re-exports

mud_egui_core/src/enums/
└── underline.rs      — Underline (public enum)
```

**Pure layer.** `is_external_url`, `scheme_of`, `origin_of`, and
`target_opens_new_tab` are free functions, unit-tested without an
`egui::Context`.

**Routing layer.** The link talks to `mud_egui_navigation`'s
`NavigationManager` for internal navigation, and to `egui::Context::open_url`
for external URLs.

**Component layer.** `show` / `show_without_router` are the only public
rendering entry points. They measure the label with an invisible probe, paint
the label + underline, and dispatch clicks.

---

## External vs. internal navigation

The rule is exactly what the caller asked for: **if the link starts with
`https://` (or any other scheme) and does not match the app URL, open the
system browser.**

```
                     ┌────────────── clicked? ──────────────┐
                     │                                       │
                     ▼                                       ▼
              href is None?  ─── yes ───► LinkAction::None
                     │
                     no
                     ▼
              prevent_navigation? ─ yes ─► LinkAction::None
                     │
                     no
                     ▼
              force_load? ──── yes ─────► ctx.open_url(to_absolute(href))
                     │                       → LinkAction::External
                     no
                     ▼
              no router? ───── yes ─────► ctx.open_url(href)
                     │                       → LinkAction::External
                     no
                     ▼
              is_external_url(href, base)?
                     │
                     ├── yes ──► ctx.open_url(href)
                     │             → LinkAction::External
                     │
                     └── no ───► nav.navigate_to(href, false)
                                  ├─ Ok(true)  → LinkAction::Internal
                                  ├─ Ok(false) → LinkAction::Vetoed
                                  └─ Err(e)    → LinkAction::Error
```

### `is_external_url` truth table

| `href` | `base_uri` | Result | Reason |
|---|---|---|---|
| `"/orders/42"` | `"app://my-app/"` | **internal** | no scheme |
| `"orders/42"` | `"app://my-app/"` | **internal** | no scheme |
| `"app://my-app/orders/42"` | `"app://my-app/"` | **internal** | same origin |
| `"https://example.com/foo"` | `"app://my-app/"` | **external** | different scheme |
| `"https://example.com/x"` | `"https://example.com/"` | **internal** | same origin |
| `"https://other.com/x"` | `"https://example.com/"` | **external** | different origin |
| `"http://example.com/x"` | `"https://example.com/"` | **external** | different scheme |
| `"mailto:a@b.com"` | `"app://my-app/"` | **external** | known external scheme |
| `"tel:+15551234"` | `"app://my-app/"` | **external** | known external scheme |
| `"ftp://files.example.com/x"` | `"app://my-app/"` | **external** | known external scheme |
| `"file:///etc/hosts"` | `"app://my-app/"` | **external** | known external scheme |
| `"data:image/png;base64,..."` | `"app://my-app/"` | **external** | known external scheme |
| `"unknownscheme:x"` | `"app://my-app/"` | **external** | unknown scheme → external |

### `Target` mapping

| MudBlazor `Target` | egui effect |
|---|---|
| `"_blank"` | `OpenUrl { new_tab: true }` |
| `"_self"` | `OpenUrl { new_tab: false }` |
| `"_parent"` | `OpenUrl { new_tab: false }` |
| `"_top"` | `OpenUrl { new_tab: false }` |
| `"framename"` | `OpenUrl { new_tab: false }` |
| `None` | `OpenUrl { new_tab: false }` |

`new_tab` only matters when the app is embedded in a web browser. On desktop
`eframe` apps the OS always opens the default browser.

### Setup

`MudLink` uses **`egui::Context::open_url`**, which `egui-winit` forwards to
the platform's default browser (via `webbrowser` on Linux/Windows, `open` on
macOS). No extra dependency on your side is required — just `eframe`.

If your `eframe` version is older than the one that wires `open_url` on your
platform, you can add a one-line fallback in `eframe::App::update`:

```rust
// Optional manual fallback for platforms where `open_url` isn't wired.
let cmds = ctx.output(|o| o.commands.clone());
for cmd in cmds {
    if let egui::OutputCommand::OpenUrl(u) = cmd {
        let _ = webbrowser::open(&u.url);
    }
}
```

---

## API reference

### `MudLink<'a>`

| Method | Type | Default | Mirrors | Description |
|--------|------|---------|---------|-------------|
| `new(href, label)` | `(impl Into<String>, impl Into<String>)` | — | `Href` + `ChildContent` | Full constructor. |
| `href_only(href)` | `impl Into<String>` | — | `Href` | No label (set later). |
| `label_only(label)` | `impl Into<String>` | — | `ChildContent` | No `href`. |
| `href(s)` | `impl Into<String>` | `None` | `Href` | Navigation target. |
| `label(s)` | `impl Into<String>` | `None` | `ChildContent` (text) | Text label. |
| `child(f)` | `FnMut(&mut Ui) + 'a` | `None` | `ChildContent` | Custom label. |
| `target(s)` | `impl Into<String>` | `None` | `Target` | `_blank` etc. |
| `color(c)` | `Color` | `Primary` | `Color` | Link colour. |
| `typo(t)` | `Typo` | `Body1` | `Typo` | Typography. |
| `underline(u)` | `Underline` | `Hover` | `Underline` | Underline behaviour. |
| `disabled(b)` | `bool` | `false` | `Disabled` | Disabled state. |
| `dense(b)` | `bool` | `false` | `Dense` | Compact padding. |
| `force_load(b)` | `bool` | `false` | `ForceLoad` | Force browser open for relative URLs. |
| `on_click(f)` | `FnMut()` | `None` | `OnClick` | Fires before navigation. |
| `classes(iter)` | `iter of AsRef<str>` | none | `Class` | Utility classes. |
| `flavor(StyleFlavor)` | `StyleFlavor` | default | — | Utility resolver. |
| `tag(t)` / `tag_ref::<U>()` | `T: Any + Send + Sync` | `None` | `Tag` | Arbitrary data. |
| `id(Id)` | `Id` | `None` | *(extension)* | Stable identity. |
| `aria_label(s)` | `impl Into<String>` | `None` | *(extension)* | Accessible label. |
| `prevent_navigation(b)` | `bool` | `false` | *(extension)* | Fire `on_click` only. |
| `show(ui, nav)` | `(&mut Ui, &mut NavigationManager)` | — | — | Draw with routing. |
| `show_without_router(ui)` | `&mut Ui` | — | — | Draw without routing. |

### `LinkResponse`

| Field | Type | Description |
|-------|------|-------------|
| `response` | `egui::Response` | The link's own egui response. |
| `action` | `LinkAction` | What the link did this frame. |

### `LinkAction`

| Variant | Description |
|---------|-------------|
| `None` | Not clicked, disabled, no `href`, or `prevent_navigation`. |
| `Internal { uri }` | Navigated through the `NavigationManager`. |
| `Vetoed { uri }` | Navigation was vetoed by a guard. |
| `Error { uri, message }` | Navigation failed (malformed URI, outside base, …). |
| `External { url, new_tab }` | Opened in the system browser. |

Helper methods: `navigated()`, `was_blocked()`, `is_external()`,
`target() -> Option<&str>`.

### `Underline`

| Variant | Description |
|---------|-------------|
| `None` | Never underlined. |
| `Hover` (default) | Underlined only on hover. |
| `Always` | Always underlined. |

---

## Examples

### Internal routing

```rust
MudLink::new("/orders/42", "Order #42").show(ui, nav);
```

### External URL (opens browser)

```rust
MudLink::new("https://docs.rs/egui", "Read the docs").show(ui, nav);
```

### External URL in a new tab

```rust
MudLink::new("https://example.com", "Open example")
    .target("_blank")
    .show(ui, nav);
```

### mailto / tel

```rust
MudLink::new("mailto:support@example.com", "Email support").show(ui, nav);
MudLink::new("tel:+15551234", "Call us").show(ui, nav);
```

### Force-load a relative URL in the browser

```rust
MudLink::new("/legacy/page", "Open in browser")
    .force_load(true)
    .show(ui, nav);
```

`ForceLoad` bypasses the router and opens `nav.to_absolute("/legacy/page")`
in the system browser, matching Blazor's full-page-reload behaviour.

### Underlined link

```rust
MudLink::new("/about", "About")
    .underline(Underline::Always)
    .show(ui, nav);
```

### Disabled link

```rust
MudLink::new("/restricted", "Restricted")
    .disabled(true)
    .show(ui, nav);
```

### Custom typography

```rust
MudLink::new("/chapter/1", "Chapter 1")
    .typo(Typo::H3)
    .color(Color::Secondary)
    .show(ui, nav);
```

### Custom label content

```rust
MudLink::new("/profile/42", "")
    .child(|ui| {
        ui.add(MudIcon::new(filled::PERSON));
        ui.add(MudText::new("Profile"));
    })
    .show(ui, nav);
```

### Fire an `on_click` callback (with href)

```rust
MudLink::new("/orders/42", "Order #42")
    .on_click(|| println!("opened order 42"))
    .show(ui, nav);
```

### Fire an `on_click` callback (without href)

```rust
MudLink::label_only("Open settings dialog")
    .on_click(|| self.dialog_open = true)
    .prevent_navigation(true)   // no href anyway, but explicit
    .show(ui, nav);
```

### Without a router

```rust
MudLink::new("https://example.com", "Example")
    .show_without_router(ui);
```

Every `href` is treated as external.

### Observe the outcome

```rust
let result = MudLink::new("/orders/42", "Order #42").show(ui, nav);
match result.action {
    LinkAction::Internal { uri } => log::info!("navigated to {uri}"),
    LinkAction::Vetoed { uri }   => log::warn!("navigation to {uri} vetoed"),
    LinkAction::Error { uri, message } => log::error!("failed to open {uri}: {message}"),
    LinkAction::External { url, .. } => log::info!("opened {url} in browser"),
    LinkAction::None => {}
}
```

### Utility classes

```rust
MudLink::new("/x", "Styled link")
    .classes(["font-bold", "text-secondary", "hover:no-underline"])
    .flavor(StyleFlavor::Tailwind)
    .show(ui, nav);
```

### Accessibility

```rust
MudLink::new("/help", "Help")
    .aria_label("Open the help center")
    .show(ui, nav);
```

---

## Performance and memory choices

| Choice | Effect |
|--------|--------|
| Pure `is_external_url` | No allocation; only slices are returned. |
| `scheme_of` and `origin_of` borrow from the input | Zero allocation. |
| Measure probe uses `UiBuilder::invisible()` | No layout side effects, no painting. |
| `child` closure runs twice per frame (measure + draw) | Matches `MudToggleGroup` custom-content convention; documented. |
| `LinkAction` is `Clone` + `PartialEq` + `Eq` | Cheap to return, easy to test. |
| Underline is one `line_segment` | No shape tree. |
| All text through `MudText` | Theme/font updates propagate. |
| `Target` is only inspected on click | No per-frame work. |
| `OpenUrl` is dispatched via `ctx.open_url` | Handled by `egui-winit` after the frame; no blocking. |

---

## Testing

`cargo test` runs 30+ pure-function tests:

| Area | Tests |
|------|-------|
| `scheme_of` | http, https, mailto, tel, relative paths, invalid schemes |
| `origin_of` | path stripping, port preservation, query/fragment stripping, non-slash schemes |
| `is_external_url` | full truth table (relative, same-origin, different-origin, different-scheme, mailto/tel/sms, ftp/file) |
| `target_opens_new_tab` | `_blank` (case-insensitive), other targets, `None` |
| Underline | default, `should_paint` truth table, `name` |
| Builders | defaults, `new` / `href_only` / `label_only`, every field |
| `LinkAction` | every variant, `navigated`, `was_blocked`, `target`, `is_external` |
| Tag | round-trips by type |

Rendering (`show`) needs a live `egui::Context` and a `NavigationManager`;
the pure layer is the source of truth.

```bash
cargo test -p mud_egui_components mud_link
cargo test -p mud_egui_core underline
```

---

## Differences from MudBlazor

- **`ForceLoad` behaviour** — MudBlazor makes the browser do a full page load
  at the current href; this port calls `ctx.open_url(nav.to_absolute(href))`
  so the absolute URL opens in the system browser. Visually the same, and it
  works on desktop where "full page load" has no meaning.
- **`Target` values other than `_blank`** are stored but have no effect (the
  DOM equivalent only matters inside a nested browsing context).
- **`Style` → `classes(..)`.** egui has no inline CSS strings.
- **`UserAttributes` not ported.** There is no HTML tag.
- **`Underline` is a new public enum in `mud_egui_core::enums`** (own file),
  matching the one-enum-per-file convention. Its C# values (`None = 0`,
  `Hover = 1`, `Always = 2`) are preserved as variants.
- **`LinkAction`** is a new public enum (own file). It reports the outcome of
  one frame so callers can log, observe, or test navigation without inspecting
  the router's state before and after.
- **`prevent_navigation`** and `show_without_router` are extensions.
- **External URL detection** is a new feature (not in MudBlazor, which relies
  on the browser's own link handling). It is implemented as a pure function
  and covered by a full truth table.

===== ./crates/mud_egui_components/src/data_display/list/README.md =====

# MudList for `mud_egui`

A scrollable, selectable vertical list for [egui](https://github.com/emilk/egui),
ported from MudBlazor's [`MudList<T>`](https://mudblazor.com/components/list),
`MudListItem<T>`, and `MudListSubheader<T>` (MudBlazor 8.13.0).

```rust
let mut state = ListState::<&str>::new();

MudList::new()
    .selection_mode(SelectionMode::Single)
    .show(ui, &mut state, |list| {
        list.subheader("Inbox");
        list.item(MudListItem::new("starred").with_text("Starred").icon(filled::STAR));
        list.text_item("sent", "Sent");
        list.text_item("drafts", "Drafts");
    });
```

Every visible string is rendered through **`MudText`**, and every icon through
**`MudIcon`**, so the list inherits the surrounding typography and theme
automatically.

---

## Contents

1. [Design principle: API parity, not API invention](#design-principle-api-parity-not-api-invention)
2. [Architecture](#architecture)
3. [API reference](#api-reference)
4. [Examples](#examples)
5. [Selection and expansion](#selection-and-expansion)
6. [Performance and memory choices](#performance-and-memory-choices)
7. [Testing](#testing)
8. [Differences from MudBlazor](#differences-from-mudblazor)

---

## Design principle: API parity, not API invention

The MudBlazor parameter surface was checked against the published API docs and
the component sources before this port.

### `MudList<T>`

| MudBlazor | Rust method | Default |
|---|---|---|
| `CheckBoxColor` | `checkbox_color(Color)` | `Color::Default` |
| `CheckedIcon` | `checked_icon(s)` | `CHECK_BOX` |
| `ChildContent` | closure to `show` | required |
| `Color` | `color(Color)` | `Primary` |
| `Comparer` | *(Rust `T: PartialEq`)* | — |
| `Dense` | `dense(bool)` | `false` |
| `Disabled` | `disabled(bool)` | `false` |
| `Gutters` | `gutters(bool)` | `true` |
| `Padding` | `padding(bool)` | `false` |
| `ReadOnly` | `readonly(bool)` | `false` |
| `SelectedValue` | `ListState::selected_value` | `None` |
| `SelectedValues` | `ListState::selected_values` | `[]` |
| `SelectionMode` | `selection_mode(SelectionMode)` | `Single` |
| `UncheckedIcon` | `unchecked_icon(s)` | `CHECK_BOX_OUTLINE_BLANK` |
| `Class` (base) | `classes(iter)` | none |
| `Style` (base) | *(use `classes`)* | — |
| `Tag` (base) | `tag(t)` / `tag_ref::<U>()` | `None` |
| `UserAttributes` (base) | **not ported** (no HTML tag) | — |

### `MudListItem<T>`

| MudBlazor | Rust method | Default |
|---|---|---|
| `AvatarContent` | `avatar(f)` | `None` |
| `ChildContent` | `child(f)` | `None` |
| `Dense` | `dense(bool)` | inherit |
| `Disabled` | `disabled(bool)` | `false` |
| `ExpandIconColor` | `expand_icon_color(Color)` | `Default` |
| `ExpandLessIcon` | `expand_less_icon(s)` | `EXPAND_LESS` |
| `ExpandMoreIcon` | `expand_more_icon(s)` | `EXPAND_MORE` |
| `Expanded` | `expanded(bool)` | `None` (uncontrolled) |
| `Gutters` | `gutters(bool)` | inherit |
| `Href` | `href(s)` | `None` |
| `Icon` | `icon(s)` | `None` |
| `IconColor` | `icon_color(Color)` | `Inherit` |
| `IconSize` | `icon_size(Size)` | `Medium` |
| `Inset` | `inset(bool)` | `false` |
| `NestedList` | `nested(f)` | `None` |
| `OnClick` | `on_click(f)` | `None` |
| `OnClickPreventDefault` | `on_click_prevent_default(bool)` | `false` |
| `Ripple` | `ripple(bool)` | `true` |
| `SecondaryText` | `secondary_text(s)` | `None` |
| `Target` | `target(s)` | `None` |
| `Text` | `text(s)` / `with_text(s)` | `None` |
| `Value` | `MudListItem::new(value)` | required |

### `MudListSubheader<T>`

| MudBlazor | Rust method | Default |
|---|---|---|
| `ChildContent` | `MudListSubheader::new(text)` | required |
| `Gutters` | `gutters(bool)` | inherit |
| `Inset` | `inset(bool)` | `false` |

### Deliberately not ported

- **`Comparer`** — MudBlazor allows a custom `IEqualityComparer<T>`. In Rust,
  implement `PartialEq` on your value type; the list uses `T: PartialEq`
  everywhere.
- **`Href` / `Target` / `ForceLoad`** — stored as fields for parity, but egui
  has no browser router. Clicking fires `on_click(&value)`; the caller
  navigates. `Target` and `ForceLoad` are unused.
- **`UserAttributes`** — there is no HTML tag to splat attributes onto.
- **`Style`** — egui has no inline CSS strings; use `classes(..)`.

---

## Architecture

```
mud_list.rs
├── MudList<'a, T>         — list configuration + render entry point
│   └── show(ui, state, |list| ..) -> (Response, R)
├── MudListItem<'a, T>     — per-item config (text, icon, avatar, nested, …)
├── MudListSubheader       — non-interactive header row
├── ListUi<'a, T>          — collector passed to the show closure
├── ListState<T>           — caller-owned selection + expansion
├── apply_selection(..)    — pure selection state machine
├── geometry helpers       — item_height, subheader_height, indent_px,
│                            icon_px, avatar_px
└── render helpers         — draw_item, draw_subheader, paint_checkbox,
                             start_ripple, paint_ripple

list_entry.rs              — private ListEntry enum (own file per project rule)
```

**Pure layer.** `apply_selection` and all geometry helpers are free functions
covered by unit tests, exactly as in `MudDataGrid` and `MudGrid`.

**State layer.** `ListState<T>` is caller-owned. It is the egui equivalent of
MudBlazor's `SelectedValue`, `SelectedValues`, and `Expanded` bindings.

**Component layer.** `show` resolves the theme and wrapper frame, calls the
pure layer, allocates one row per item, and paints. It never measures text to
place rows, which keeps layout deterministic.

---

## API reference

### `MudList<'a, T>`

| Method | Default | Mirrors | Description |
|---|---|---|---|
| `new()` | — | — | MudBlazor defaults. |
| `selection_mode(SelectionMode)` | `Single` | `SelectionMode` | `Single` / `Toggle` / `Multi`. |
| `color(Color)` | `Primary` | `Color` | Accent for selected items. |
| `checkbox_color(Color)` | `Default` | `CheckBoxColor` | Checkbox accent in `Multi`. |
| `checked_icon(s)` | `CHECK_BOX` | `CheckedIcon` | Checked glyph. |
| `unchecked_icon(s)` | `CHECK_BOX_OUTLINE_BLANK` | `UncheckedIcon` | Unchecked glyph. |
| `dense(bool)` | `false` | `Dense` | Compact rows. |
| `disabled(bool)` | `false` | `Disabled` | Disables every item. |
| `readonly(bool)` | `false` | `ReadOnly` | Prevents selection; keeps expansion. |
| `gutters(bool)` | `true` | `Gutters` | Side padding on items. |
| `padding(bool)` | `false` | `Padding` | Vertical padding around the list. |
| `classes(iter)` | none | `Class` | Utility classes on the wrapper. |
| `flavor(StyleFlavor)` | default | — | Bootstrap or Tailwind. |
| `tag(t)` / `tag_ref::<U>()` | `None` | `Tag` | Arbitrary user data. |
| `show(ui, state, \|list\| ..)` | — | — | Draws and returns `(Response, R)`. |

### `MudListItem<'a, T>`

| Method | Mirrors | Description |
|---|---|---|
| `new(value)` | `Value` | Required value. |
| `text(value, s)` | `Text` | Convenience constructor. |
| `with_text(s)` | `Text` | Primary caption. |
| `secondary_text(s)` | `SecondaryText` | Small caption under the title. |
| `icon(s)` | `Icon` | Leading Material SVG path. |
| `icon_color(Color)` | `IconColor` | Default `Inherit`. |
| `icon_size(Size)` | `IconSize` | Default `Medium`. |
| `avatar(f)` | `AvatarContent` | Overrides `icon`. |
| `child(f)` | `ChildContent` | Overrides `text` + `secondary_text`. |
| `nested(f)` | `NestedList` | Expandable child items. |
| `expanded(bool)` | `Expanded` | Initial expansion state. |
| `dense(bool)` | `Dense` | Overrides the list. |
| `disabled(bool)` | `Disabled` | Disables this item. |
| `inset(bool)` | `Inset` | Extra left indent. |
| `gutters(bool)` | `Gutters` | Overrides the list. |
| `href(s)` | `Href` | Stored for parity; navigate in `on_click`. |
| `target(s)` | `Target` | Stored; unused. |
| `force_load(bool)` | `ForceLoad` | Stored; unused. |
| `ripple(bool)` | `Ripple` | Default `true`. |
| `on_click(f)` | `OnClick` | `FnMut(&T)`. |
| `on_click_prevent_default(bool)` | `OnClickPreventDefault` | Skips selection/expansion. |
| `id(s)` | — | Stable key for expansion and ripple. |
| `expand_less_icon(s)` | `ExpandLessIcon` | Default `EXPAND_LESS`. |
| `expand_more_icon(s)` | `ExpandMoreIcon` | Default `EXPAND_MORE`. |
| `expand_icon_color(Color)` | `ExpandIconColor` | Default `Default`. |

### `ListState<T>`

| Field / method | Description |
|---|---|
| `selected_value: Option<T>` | Single / toggle selection. |
| `selected_values: Vec<T>` | Multi selection. |
| `expanded: Vec<(String, bool)>` | Expansion state. |
| `is_selected(&T, SelectionMode)` | Mode-aware membership test. |
| `is_expanded(&str)` | `Option<bool>`; `None` if uninitialised. |
| `ensure_expanded(&str, bool)` | Seeds the initial value once. |
| `toggle_expanded(&str)` | Flips the state. |
| `clear_expanded()` | Drops all expansion entries. |

### Geometry constants

| Name | Value |
|---|---|
| `ITEM_HEIGHT` | `48.0` |
| `DENSE_ITEM_HEIGHT` | `32.0` |
| `SUBHEADER_HEIGHT` | `36.0` |
| `DENSE_SUBHEADER_HEIGHT` | `28.0` |
| `GUTTER_PX` | `12.0` |
| `INSET_PX` | `16.0` |
| `INDENT_PER_DEPTH` | `16.0` |

---

## Examples

### Single selection (default)

```rust
let mut state = ListState::<&str>::new();

MudList::new().show(ui, &mut state, |list| {
    list.text_item("inbox", "Inbox");
    list.text_item("sent", "Sent");
    list.text_item("drafts", "Drafts");
});

if let Some(folder) = state.selected_value {
    // …
}
```

### Multi selection with checkboxes

```rust
MudList::new()
    .selection_mode(SelectionMode::Multi)
    .checkbox_color(Color::Success)
    .show(ui, &mut state, |list| {
        for tag in ["urgent", "work", "personal"] {
            list.text_item(tag, tag);
        }
    });

// state.selected_values holds every checked tag.
```

### Toggle selection (deselectable)

```rust
MudList::new()
    .selection_mode(SelectionMode::Toggle)
    .show(ui, &mut state, |list| {
        list.text_item("active", "Active only");
    });
```

Clicking the selected item clears `selected_value`.

### Icons and secondary text

```rust
MudList::new().show(ui, &mut state, |list| {
    list.item(
        MudListItem::new("starred")
            .with_text("Starred")
            .secondary_text("12 messages")
            .icon(filled::STAR)
            .icon_color(Color::Warning),
    );
    list.item(
        MudListItem::new("archive")
            .with_text("Archive")
            .icon(filled::ARCHIVE),
    );
});
```

### Avatars

```rust
list.item(
    MudListItem::new("ada")
        .with_text("Ada Lovelace")
        .avatar(|ui| {
            ui.add(MudAvatar::new().text("AL").size(Size::Small));
        }),
);
```

`avatar` overrides `icon`, matching MudBlazor.

### Custom row content

```rust
list.item(
    MudListItem::new("custom").child(|ui| {
        ui.add(MudText::new("Custom").typo(Typo::Body1));
        ui.add(MudText::new("content").typo(Typo::Caption));
    }),
);
```

`child` overrides `text` and `secondary_text`.

### Nested expandable items

```rust
list.item(
    MudListItem::new("projects")
        .with_text("Projects")
        .icon(filled::FOLDER)
        .nested(|nested| {
            nested.text_item("alpha", "Alpha");
            nested.text_item("beta", "Beta");
        }),
);
```

Expansion is remembered in `state.expanded`; seed it with `.expanded(true)`
to start open.

### Subheaders

```rust
list.subheader("Work");
list.text_item("inbox", "Inbox");
list.text_item("sent", "Sent");

list.subheader("Personal");
list.text_item("home", "Home");
```

### Dense list

```rust
MudList::new().dense(true).show(ui, &mut state, |list| { /* … */ });
```

Or per item with `.dense(true)`.

### Disabled and read-only

```rust
MudList::new()
    .readonly(true)   // expansion still works
    .show(ui, &mut state, |list| { /* … */ });

MudList::new()
    .disabled(true)   // nothing responds
    .show(ui, &mut state, |list| { /* … */ });
```

### Gutters and inset

```rust
MudList::new()
    .gutters(false)
    .show(ui, &mut state, |list| {
        list.item(MudListItem::new(1).with_text("Flush"));
        list.item(MudListItem::new(2).with_text("Inset").inset(true));
    });
```

### Click callback and tag

```rust
let list = MudList::new()
    .tag("sidebar")
    .show(ui, &mut state, |list| {
        list.item(
            MudListItem::new("logout")
                .with_text("Log out")
                .on_click(|value| println!("clicked {value:?}")),
        );
    });

assert_eq!(list.0.widget_info, list.0.widget_info); // response is available
assert_eq!(/* tag */ "sidebar", "sidebar");
```

---

## Selection and expansion

### Selection

| Mode | Click unselected | Click selected |
|---|---|---|
| `Single` | selects it | no change |
| `Toggle` | selects it | clears the selection |
| `Multi` | adds it to `selected_values` | removes it |

`Single` and `Toggle` write `state.selected_value`; `Multi` writes
`state.selected_values`. The two fields are kept mutually exclusive by
`apply_selection`. This mirrors MudBlazor's note that `SelectedValue` and
`SelectedValues` are read/written only in their respective modes.

`readonly(true)` disables selection entirely while keeping expansion.
`disabled(true)` (list) or `.disabled(true)` (item) disables everything,
including expansion and ripple.

### Expansion

Expansion state lives in `ListState::expanded`, a `Vec<(String, bool)>`
keyed by the item's `id` (or its positional path when `id` is not set).

- `.expanded(true)` **seeds** the initial state once. After that, clicks
  toggle it. This matches MudBlazor's remark that setting `Expanded` without
  two-way binding initialises the state.
- To control expansion manually, read `state.is_expanded(key)` before
  rendering and set `state.expanded` yourself; clicks will still toggle it
  unless you intercept the item's `on_click`.

### `OnClickPreventDefault`

When `on_click_prevent_default(true)` is set, clicking the row only fires
`on_click`. Neither selection nor expansion changes. This mirrors MudBlazor's
`OnClickPreventDefault`.

---

## Performance and memory choices

| Choice | Effect |
|--------|--------|
| `MudListItem` is built once per frame from borrowed closures | No retained render tree. |
| Rows are allocated with `allocate_exact_size` + absolute child `Ui`s | No per-row layout pass; row height is known before painting. |
| `apply_selection` is a pure state machine | Testable without an `egui::Context`. |
| Expansion state is `Vec<(String, bool)>` | No `HashMap` hashing; items are few. |
| Ripple is memory-backed and keyed by `ui.scope_id().with(path)` | No per-item state on the list struct. |
| `start_ripple` reads `ui.input(..)` **before** `memory_mut(..)` | Avoids the re-entrant `RwLock` deadlock fixed in `MudToggleGroup`. |
| Strings are moved into the render path, not cloned per frame | The item is single-use per frame. |
| All text goes through `MudText`; all icons through `MudIcon` | Theme/font updates propagate automatically; no duplicated style logic. |

Not done, on purpose: a row-height cache and a text-measurement cache. Both
would need invalidation rules that belong to a layout engine; the list stays
deterministic instead.

---

## Testing

`cargo test` runs the module's tests:

| Area | Tests |
|------|-------|
| `apply_selection` | single select/reclick, single replace, toggle deselect, toggle replace, multi membership and order |
| `is_selected` | mode-aware membership |
| Expansion | `ensure_expanded` seeds once, `toggle_expanded` flips/creates, `clear_expanded` empties |
| Geometry | item/subheader heights per density, indent scaling, icon px mapping, avatar px per density |
| Defaults | `MudList` matches MudBlazor, `MudListItem` defaults, `MudListSubheader` defaults |
| Builders | every `MudListItem` builder sets its field |
| Collector | order and subheader insertion |
| Tag | round-trips by type |

Rendering itself (`show`) needs a live `egui::Context` and is not unit-tested;
the pure layer is the source of truth for selection, expansion, and geometry.

```bash
cargo test -p mud_egui_components mud_list
```

---

## Differences from MudBlazor

- **No `Comparer` parameter.** Implement `PartialEq` on your value type; the
  list uses `T: PartialEq` for membership tests.
- **`SelectedValue` and `SelectedValues` are unified in `ListState<T>`.**
  The caller owns both fields and passes `&mut ListState` to `show`; the list
  writes the field that matches `selection_mode`.
- **`Href` / `Target` / `ForceLoad` are stored, not acted on.** egui has no
  browser router; clicking fires `on_click(&value)` and the caller navigates.
  `Target` and `ForceLoad` have no meaningful egui equivalent and are unused.
- **`Style` is `classes(..)`.** egui has no inline CSS strings; utility
  classes are resolved through the Bootstrap or Tailwind flavour.
- **`UserAttributes` not ported.** There is no HTML tag.
- **`Expanded` is seeded, not hard-controlled.** Setting `.expanded(true)`
  initialises the state on first render; subsequent clicks toggle it, matching
  MudBlazor's non-two-way behaviour. To fully control it, update
  `state.expanded` yourself each frame.
- **Selection modes are `SelectionMode::{Single, Toggle, Multi}`** — the
  existing `mud_egui_core::enums::SelectionMode`, which already matches
  MudBlazor's `SingleSelection` / `ToggleSelection` / `MultiSelection`.
- **No new public enums.** The only enum in the module is the private
  `ListEntry` collector, kept in its own file per project convention.

===== ./crates/mud_egui_components/src/data_display/simple_table/README.md =====

# MudSimpleTable for `mud_egui`

A lightweight, non-data-bound table for [egui](https://github.com/emilk/egui),
ported from MudBlazor's [`MudSimpleTable`](https://mudblazor.com/components/simpletable)
(MudBlazor 8.13.0). Draws striped rows, hover highlighting, borders, elevation,
dense rows, outlined/square variants, and a fixed header — without any data
model, sorting, filtering, or pagination.

```rust
MudSimpleTable::new()
    .striped(true)
    .hover(true)
    .bordered(true)
    .show(ui, |t| {
        t.header(|h| {
            h.cell("Name");
            h.cell_width("Age", 80.0);
            h.cell("City");
        });
        for person in &people {
            t.row(|r| {
                r.cell(person.name.clone());
                r.cell(person.age.to_string());
                r.cell(person.city.clone());
            });
        }
    });
```

Every text cell renders through **`MudText`**, so typography and theme updates
propagate automatically.

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

`MudSimpleTable` is a **styled wrapper around raw table markup**, not a data
grid. The MudBlazor component's `ChildContent` is documented as using HTML
table tags (`<thead>`, `<tbody>`, `<tr>`, `<th>`, `<td>`), and the official
examples use those tags directly. `MudTh` / `MudTd` / `MudTr` belong to
`MudTable<T>` and are **not** part of `MudSimpleTable`.

The egui port mirrors this with a closure-based builder that collects header,
body, and footer rows. No new public enums are introduced; the only enum in the
module (`RowKind`, `CellContent`) is `pub(crate)` and lives in its own file.

### Ported parameters

| MudBlazor | Rust method | Default |
|-----------|-------------|---------|
| `Bordered` | `bordered(bool)` | `false` |
| `ChildContent` | closure to `show` | required |
| `Dense` | `dense(bool)` | `false` |
| `Elevation` | `elevation(u8)` | `1` |
| `FixedHeader` | `fixed_header(bool)` | `false` |
| `Hover` | `hover(bool)` | `false` |
| `Outlined` | `outlined(bool)` | `false` |
| `Square` | `square(bool)` | `false` |
| `Striped` | `striped(bool)` | `false` |
| `Class` (base) | `classes(iter)` | none |
| `Style` (base) | *(use `classes`)* | — |
| `Tag` (base) | `tag(t)` / `tag_ref::<U>()` | `None` |
| `UserAttributes` (base) | **not ported** (no HTML tag) | — |

### Deliberately not ported

- **`Style`** — egui has no inline CSS strings; use `classes(..)`.
- **`UserAttributes`** — there is no HTML tag to splat attributes onto.
- **`MudTh` / `MudTd` / `MudTr` sub-components** — these are for `MudTable<T>`,
  not `MudSimpleTable`. The table builder's `header` / `row` / `footer`
  closures replace them.
- **CSS `height` on `FixedHeader`** — replaced by the explicit
  `height(f32)` builder, which maps directly to egui's
  `ScrollArea::max_height`.

---

## Architecture

```
mud_simple_table.rs
├── MudSimpleTable<'a>       — configuration + render entry point
│   └── show(ui, |t| ..) -> (Response, R)
├── SimpleTableUi<'a>        — collector: header / row / footer
├── RowUi<'a>                — per-row cell builder
├── TableCell<'a>            — (content, optional width)
├── Pure helpers             — row_height, resolve_column_widths,
│                             column_count, column_hints, elevation_shadow
└── Tests                    — 20+ pure-function tests

simple_table_internal.rs      — private CellContent / RowKind enums
```

**Pure layer.** `resolve_column_widths`, `column_hints`, `column_count`,
`row_height`, and `elevation_shadow` are free functions, covered by unit tests,
exactly as in `MudDataGrid` and `MudGrid`.

**Collector layer.** `SimpleTableUi` records header, body, and footer rows
before any painting. The table knows every cell's width hint before it
allocates a single rectangle.

**Component layer.** `show` resolves the theme and wrapper frame, calls the
pure layer, allocates one rectangle per row, and paints through child `Ui`s.

---

## API reference

### `MudSimpleTable<'a>`

| Method | Default | Mirrors | Description |
|--------|---------|---------|-------------|
| `new()` | — | — | MudBlazor defaults. |
| `bordered(bool)` | `false` | `Bordered` | Left/right cell borders. |
| `dense(bool)` | `false` | `Dense` | Compact row heights. |
| `elevation(u8)` | `1` | `Elevation` | Drop shadow (0–24). |
| `fixed_header(bool)` | `false` | `FixedHeader` | Sticky header (requires `height`). |
| `hover(bool)` | `false` | `Hover` | Row hover highlight. |
| `outlined(bool)` | `false` | `Outlined` | Outer border. |
| `square(bool)` | `false` | `Square` | Zero corner radius. |
| `striped(bool)` | `false` | `Striped` | Alternating row fills. |
| `height(f32)` | `None` | `Style="height:…"` | Scrollable body height. |
| `classes(iter)` | none | `Class` | Wrapper utility classes. |
| `flavor(StyleFlavor)` | default | — | Bootstrap / Tailwind. |
| `tag(t)` / `tag_ref::<U>()` | `None` | `Tag` | Arbitrary user data. |
| `show(ui, \|t\| ..)` | — | `ChildContent` | Draws and returns `(Response, R)`. |

### `SimpleTableUi<'a>`

| Method | Description |
|--------|-------------|
| `header(\|h\| ..)` | Sets the header row (replaces any previous). |
| `row(\|r\| ..)` | Adds a body row. |
| `footer(\|f\| ..)` | Sets the footer row (replaces any previous). |
| `row_count()` | Number of body rows collected. |
| `is_empty()` | Whether no body rows have been collected. |

### `RowUi<'a>`

| Method | Description |
|--------|-------------|
| `cell(text)` | Text cell, rendered through `MudText`. |
| `cell_width(text, px)` | Text cell with an explicit width. |
| `cell_ui(f)` | Custom-UI cell; `f: FnMut(&mut Ui)`. |
| `cell_ui_width(f, px)` | Custom-UI cell with an explicit width. |
| `len()` / `is_empty()` | Cell count. |

### Geometry constants

| Name | Value |
|------|-------|
| `ROW_HEIGHT` | `48.0` |
| `DENSE_ROW_HEIGHT` | `32.0` |
| `CELL_PADDING_X` | `12.0` |
| `CELL_PADDING_Y` | `8.0` |
| `MAX_ELEVATION` | `24` |
| `FALLBACK_WIDTH` | `1280.0` |

---

## Examples

### Basic table

```rust
MudSimpleTable::new().show(ui, |t| {
    t.header(|h| {
        h.cell("Name");
        h.cell("Age");
    });
    t.row(|r| {
        r.cell("Alice");
        r.cell("30");
    });
    t.row(|r| {
        r.cell("Bob");
        r.cell("35");
    });
});
```

### Striped + hover + bordered

```rust
MudSimpleTable::new()
    .striped(true)
    .hover(true)
    .bordered(true)
    .show(ui, |t| {
        t.header(|h| {
            h.cell("ID");
            h.cell("Name");
            h.cell("Email");
        });
        for (i, person) in people.iter().enumerate() {
            t.row(|r| {
                r.cell(i.to_string());
                r.cell(person.name.clone());
                r.cell(person.email.clone());
            });
        }
    });
```

### Explicit column widths

```rust
MudSimpleTable::new().show(ui, |t| {
    t.header(|h| {
        h.cell_width("ID", 60.0);
        h.cell("Name");
        h.cell_width("Actions", 120.0);
    });
    t.row(|r| {
        r.cell("1");
        r.cell("Alice");
        r.cell_ui_width(|ui| {
            ui.add(MudButton::new("Edit").size(Size::Small));
        }, 120.0);
    });
});
```

### Dense table

```rust
MudSimpleTable::new().dense(true).show(ui, |t| {
    t.header(|h| h.cell("Key"));
    t.row(|r| r.cell("Value"));
});
```

### Elevation and rounded corners

```rust
MudSimpleTable::new()
    .elevation(4)
    .show(ui, |t| { /* … */ });

MudSimpleTable::new()
    .square(true)   // no rounded corners
    .elevation(0)   // no shadow
    .show(ui, |t| { /* … */ });
```

### Outlined table

```rust
MudSimpleTable::new()
    .outlined(true)
    .show(ui, |t| { /* … */ });
```

### Fixed header with a scrollable body

```rust
MudSimpleTable::new()
    .fixed_header(true)
    .height(300.0)
    .show(ui, |t| {
        t.header(|h| {
            h.cell("Name");
            h.cell("Age");
        });
        for person in &people {
            t.row(|r| {
                r.cell(person.name.clone());
                r.cell(person.age.to_string());
            });
        }
    });
```

`fixed_header(true)` is a no-op unless `height` is also set, matching
MudBlazor's requirement that CSS `height` be set when `FixedHeader` is true.

### Custom-UI cells

```rust
MudSimpleTable::new().show(ui, |t| {
    t.header(|h| {
        h.cell("Product");
        h.cell("Actions");
    });
    for product in &products {
        t.row(|r| {
            r.cell_ui(|ui| {
                ui.horizontal(|ui| {
                    ui.add(MudIcon::new(filled::INVENTORY));
                    ui.add(MudText::new(product.name.clone()));
                });
            });
            r.cell_ui(|ui| {
                ui.horizontal(|ui| {
                    ui.add(MudButton::new("Edit").size(Size::Small));
                    ui.add(MudButton::new("Delete").size(Size::Small).color(Color::Error));
                });
            });
        });
    }
});
```

### Footer row

```rust
MudSimpleTable::new().show(ui, |t| {
    t.header(|h| {
        h.cell("Item");
        h.cell("Qty");
    });
    for line in &cart {
        t.row(|r| {
            r.cell(line.name.clone());
            r.cell(line.qty.to_string());
        });
    }
    t.footer(|f| {
        f.cell("Total");
        f.cell(total_qty.to_string());
    });
});
```

### Tag round-trip

```rust
let table = MudSimpleTable::new()
    .tag("schedule")
    .show(ui, |t| { /* … */ });
```

---

## Layout and rendering

### Column widths

The table collects all rows first, then resolves column widths from the
**first explicit width hint** per column:

1. Scan the header, then every body row, then the footer.
2. For each column, take the first `Some(px)` encountered.
3. Columns with no hint are “flexible”.
4. The remaining space is split equally among flexible columns.
5. If every column is fixed, the result is the sum of the explicit widths.

Explicit widths are set via `cell_width(text, px)` or
`cell_ui_width(closure, px)`.

### Row heights

| Density | Row height |
|---------|------------|
| Normal  | `48.0` |
| Dense   | `32.0` |

### Fixed header

When `fixed_header(true)` **and** `height(..)` are both set:

- The header is drawn once, outside the scroll area.
- The body rows scroll inside a `ScrollArea::vertical().max_height(…)`.
- The footer, if present, is drawn after the scroll area.

When only `height(..)` is set, the entire table (header + body + footer)
scrolls together.

### Borders and stripes

- `bordered(true)` draws a vertical line between each pair of cells.
- A horizontal line is always drawn under every row (header, body, footer).
- `striped(true)` fills odd body rows with `faint_bg_color`.
- `hover(true)` fills the hovered body row with `hovered.weak_bg_fill`.
- `outlined(true)` draws a 1 px rounded outline around the whole table.

### Elevation

`elevation(n)` maps to `egui::epaint::Shadow` on the wrapper `Frame`, using
the same scale as `MudAvatar` / `MudBadge` / `MudDataGrid`. Elevation `0`
produces no shadow. Values above `24` are clamped.

### Text styling

- Header and footer cells: `MudText` with `Typo::Subtitle2` and `bold(true)`.
- Body cells: `MudText` with `Typo::Body2`.
- Custom-UI cells: the closure is responsible for using `MudText` / `MudIcon`.

---

## Performance and memory choices

| Choice | Effect |
|--------|--------|
| Rows are collected once per frame and rendered once | No retained render tree. |
| `resolve_column_widths` is a single linear pass | No per-column allocation beyond the result vector. |
| Rows use `allocate_exact_size` + absolute child `Ui`s | No per-row layout pass; heights are known before painting. |
| Header / footer are stored as `Option<Vec<TableCell>>` | Only one of each per frame. |
| Cell text is moved into the render path, not cloned | Each `TableCell` is single-use per frame. |
| All text goes through `MudText` | Theme/font updates propagate automatically. |
| `fixed_header` uses `ScrollArea::max_height` | Only the body is virtualised, matching MudBlazor. |

Not done, on purpose: automatic content measurement for column widths. Doing
so would require every cell closure to run twice per frame (measure + draw),
which is a significant cost for large tables. The explicit-width mechanism
gives callers the same control without that cost.

---

## Testing

`cargo test` runs the module's tests:

| Area | Tests |
|------|-------|
| `resolve_column_widths` | all-flexible, all-fixed, mixed, over-constrained, non-finite available, empty input |
| `column_count` | widest row, missing sections |
| `column_hints` | first non-`None` wins, per-column |
| `row_height` | dense vs normal |
| `elevation_shadow` | zero, clamped, monotonic |
| Defaults | match MudBlazor |
| Builders | every field, negative `height` clamps to `0` |
| Tag | round-trips by type |
| Collector | preserves row order, header replacement |

Rendering itself (`show`) needs a live `egui::Context` and is not unit-tested;
the pure layer is the source of truth for layout.

```bash
cargo test -p mud_egui_components mud_simple_table
```

---

## Differences from MudBlazor

- **No HTML tags.** The `header` / `row` / `footer` closures replace
  `<thead>` / `<tbody>` / `<tfoot>`, and `cell` / `cell_ui` replace
  `<th>` / `<td>`. This is the idiomatic egui equivalent.
- **No `MudTh` / `MudTd` / `MudTr`.** Those belong to `MudTable<T>`, not
  `MudSimpleTable`; they are not ported here.
- **Explicit column widths.** MudBlazor relies on CSS auto-layout; egui has
  no equivalent without measuring every cell. `cell_width` / `cell_ui_width`
  give callers the same control.
- **`Style` → `classes(..)`.** egui has no inline CSS strings.
- **`UserAttributes` not ported.** There is no HTML tag.
- **`FixedHeader` requires `height(..)`.** MudBlazor requires CSS `height` to
  be set; this port requires the egui equivalent, a numeric height.
- **`Elevation` is `u8`, not `int`.** Values above `24` are clamped,
  matching the other `mud_egui` components.

===== ./crates/mud_egui_components/src/data_display/table/README.md =====

# MudTable for `mud_egui`

A sortable, filterable, paginated table with multiselection, responsive
stacking, child rows and a built-in pager for
[egui](https://github.com/emilk/egui), ported from MudBlazor's
[`MudTable<T>`](https://mudblazor.com/components/table) (MudBlazor 8.13.0).

```rust
let mut state = TableState::<Person>::new();

MudTable::new()
    .hover(true)
    .striped(true)
    .multi_selection(true)
    .show(ui, &people, &mut state, |t| {
        t.header(|h| {
            h.sort_th("Name", "name", |a, b| a.name.cmp(&b.name));
            h.sort_th("Age",  "age",  |a, b| a.age.cmp(&b.age));
        });
        t.row_template(|r, p| {
            r.td(Td::text(p.name.clone()).label("Name"));
            r.td(Td::text(p.age.to_string()).label("Age").align(ColumnAlign::Right));
        });
    });
```

Every text goes through **`MudText`**, every glyph through **`MudIcon`**, and
the pager reuses **`MudButton`** / **`MudToggleGroup`**.

---

## Contents

1. [Design principle: API parity, not API invention](#design-principle-api-parity-not-api-invention)
2. [Architecture](#architecture)
3. [API reference](#api-reference)
4. [Examples](#examples)
5. [Data pipeline](#data-pipeline)
6. [Performance and memory choices](#performance-and-memory-choices)
7. [Testing](#testing)
8. [Differences from MudBlazor](#differences-from-mudblazor)

---

## Design principle: API parity, not API invention

MudBlazor's `MudTable<T>` was checked against DNDocs and the source before
implementing. It is **row-template driven**: you supply `RowTemplate` (and
usually `RowTemplate` with `MudTd` cells), plus optional `HeaderContent` /
`FooterContent`, and the table handles sorting, filtering, pagination and
selection.

That is a different model from `MudDataGrid<T>` (column descriptors). This port
follows the `MudTable` model, and reuses the existing
`SortDirection` / `ColumnAlign` / `Breakpoint` / `SelectionMode` enums — no new
public enums are introduced. The only enums in the module (`CellContent`,
`RowKind`) are `pub(crate)` and live in `table_internal.rs`.

### Deliberately not ported

| MudBlazor | Why |
|-----------|-----|
| Inline editing (`Editable`, `RowEditingTemplate`, `EditTrigger`, `ApplyButtonPosition`, `CommitEditIcon`, `CancelEditIcon`, `RowEditPreview/Commit/Cancel`, `IsEditRowSwitchingBlocked`, `ReadOnly`) | Requires an edit-state machine and form validation. Out of scope for a display table. |
| Grouping (`GroupBy`, `GroupHeaderTemplate`, `GroupFooterTemplate`, `Group*Class`, `Group*Style`, `CollapseAllGroups`, `ExpandAllGroups`) | Hierarchical grouping is a separate widget; adding it here would balloon the surface. |
| `ServerData`, `TotalItems`, `Virtualize`, `OverscanCount`, `IsVirtualized` | Server paging and windowed rendering need async plumbing and a virtualised `Ui`. Use the built-in paging. |
| `Comparer` | Rust `T: PartialEq` is the idiomatic equivalent. |
| `ColGroup` | Replaced by [`MudTable::column_widths`] — the same purpose (per-column sizing) expressed in egui terms. |
| `Style`, `RowStyle`, `ContainerStyle`, `CellClass` | egui has no inline CSS strings. `row_class` / `row_class_func` / `header_classes` / `footer_classes` / `container_classes` are resolved through the Bootstrap or Tailwind flavour instead. |
| `UserAttributes` | There is no HTML tag to splat attributes onto. |
| `CustomHeader` / `CustomFooter` flags | Not needed: calling [`TableUi::header`] / [`TableUi::footer`] more than once produces multi-row headers/footers directly. |

---

## Architecture

```
mud_table.rs
├── MudTable<'a, T>            — configuration + render entry point
│   └── show(ui, items, state, |t| ..) -> (Response, R)
├── TableState<T>              — sorts, page, rows-per-page, selection
├── Td<'a>                     — body/footer cell (text | ui, label, align, width)
├── Th<'a, T>                  — header cell (text | ui, sort key + comparator)
├── RowUi<'a>                  — per-row cell collector
├── HeaderUi<'a, T>            — per-header-row cell collector
├── FooterUi<'a>               — per-footer-row cell collector
├── TableUi<'a, T>             — top-level collector (headers, footers,
│                                row_template, child_row)
├── Pure helpers               — row_height, effective_row_height,
│                                active_breakpoint, is_stacked, toggle_sort,
│                                sort_indices_by, page_count, clamp_page,
│                                page_range, resolve_column_widths,
│                                resolve_column_count, resolve_column_hints,
│                                elevation_shadow, toggle_selection
├── draw_cell_in_rect / draw_cell_content / paint_checkbox
└── draw_table_pager           — MudTablePager equivalent

table_internal.rs              — pub(crate) CellContent / RowKind
```

**Pure layer.** Everything decidable from numbers is a free function, unit
tested without an `egui::Context`.

**State layer.** `TableState<T>` is caller-owned — the egui equivalent of
`SelectedItem`, `SelectedItems`, `CurrentPage`, `RowsPerPage`, `SortLabel` and
`SortDirection` bindings.

**Component layer.** `show` filters → sorts → pages → runs the row template
once per visible row → resolves widths → paints.

---

## API reference

### `MudTable<'a, T>`

| Method | Default | Mirrors | Description |
|--------|---------|---------|-------------|
| `new()` | — | — | MudBlazor defaults. |
| `allow_unsorted(bool)` | `true` | `AllowUnsorted` | Allows the third `None` sort state. |
| `filter(Fn(&T) -> bool)` | `None` | `Filter` | Row predicate. |
| `pager(bool)` | `true` | `PagerContent` | Built-in pager. |
| `current_page(usize)` | `None` | `CurrentPage` | One-call override. |
| `rows_per_page(usize)` | `None` | `RowsPerPage` | One-call override. |
| `page_size_options(iter)` | `[10,25,50,100]` | `PageSizeOptions` | Pager sizes. |
| `rows_per_page_string(s)` | `"Rows per page:"` | `RowsPerPageString` | Pager label. |
| `all_items_text(s)` | `"All"` | `AllItemsText` | Label for `usize::MAX`. |
| `hide_rows_per_page(bool)` | `false` | `HideRowsPerPage` | Hides the size selector. |
| `multi_selection(bool)` | `false` | `MultiSelection` | Checkbox column. |
| `select_on_row_click(bool)` | `true` | `SelectOnRowClick` | Row click toggles. |
| `selection_changeable(bool)` | `true` | `SelectionChangeable` | Freezes selection. |
| `bordered(bool)` | `false` | `Bordered` | Cell dividers. |
| `dense(bool)` | `false` | `Dense` | Compact rows. |
| `elevation(u8)` | `1` | `Elevation` | Shadow (0–24). |
| `hover(bool)` | `false` | `Hover` | Row hover fill. |
| `outlined(bool)` | `false` | `Outlined` | Outer border. |
| `square(bool)` | `false` | `Square` | No rounding. |
| `striped(bool)` | `false` | `Striped` | Alternating fills. |
| `breakpoint(Breakpoint)` | `Xs` | `Breakpoint` | Stacking threshold. |
| `item_size(f32)` | `None` | `ItemSize` | Explicit row height. |
| `height(f32)` | `None` | `Height` | Scrollable table. |
| `fixed_header(bool)` | `false` | `FixedHeader` | Sticky header. |
| `fixed_footer(bool)` | `false` | `FixedFooter` | Sticky footer. |
| `horizontal_scrollbar(bool)` | `false` | `HorizontalScrollbar` | Horizontal scroll. |
| `column_widths(iter)` | `None` | `ColGroup` | Per-column widths. |
| `row_class(s)` | `None` | `RowClass` | Row utility classes. |
| `row_class_func(f)` | `None` | `RowClassFunc` | Per-row classes. |
| `on_row_click(f)` | `None` | `OnRowClick` | `FnMut(&T, usize)`. |
| `on_row_mouse_enter(f)` | `None` | `OnRowMouseEnter` | Hover-enter. |
| `on_row_mouse_leave(f)` | `None` | `OnRowMouseLeave` | Hover-leave. |
| `toolbar(f)` | `None` | `ToolBarContent` | Above the table. |
| `loading(bool)` | `false` | `Loading` | Overlay + spinner. |
| `loading_progress_color(Color)` | `Info` | `LoadingProgressColor` | Spinner tint. |
| `no_records_text(s)` | `"No records to display"` | `NoRecordsContent` | Empty text. |
| `loading_text(s)` | `"Loading…"` | `LoadingContent` | Loading text. |
| `aria_label(s)` | `None` | — | Accessible label. |
| `classes(iter)` | none | `TableClass` | Table classes. |
| `container_classes(s)` | `None` | `ContainerClass` | Container classes. |
| `header_classes(s)` | `None` | `HeaderClass` | Header row classes. |
| `footer_classes(s)` | `None` | `FooterClass` | Footer row classes. |
| `flavor(StyleFlavor)` | default | — | Utility resolver. |
| `tag(t)` / `tag_ref::<U>()` | `None` | `Tag` | Arbitrary data. |
| `show(ui, items, state, \|t\| ..)` | — | — | Returns `(Response, R)`. |

### `Td<'a>`

| Method | Mirrors | Description |
|--------|---------|-------------|
| `text(s)` | `MudTd` + text | Text cell via `MudText`. |
| `ui(f)` | `MudTd` + `ChildContent` | Custom cell. |
| `label(s)` | `MudTd.DataLabel` | Responsive-mode label. |
| `hidden(bool)` | `MudTd.Hidden` | Skips the cell. |
| `align(ColumnAlign)` | `MudTd` alignment | Left / Center / Right. |
| `width(f32)` | `MudTd` width | Explicit column hint. |

### `Th<'a, T>`

| Method | Mirrors | Description |
|--------|---------|-------------|
| `text(s)` | `MudTh` | Plain header cell. |
| `ui(f)` | `MudTh` + `ChildContent` | Custom header cell. |
| `sortable(bool)` | `MudTh.Sortable` | Enable/disable sorting. |
| `sort_by(key, cmp)` | `MudTableSortLabel.SortLabel` + `SortBy` | Key + comparator. |
| `hidden(bool)` | `MudTh.Hidden` | Skips the column. |
| `align(ColumnAlign)` | `MudTh` alignment | Alignment. |
| `width(f32)` | `MudTh` width | Explicit column hint. |
| `is_sortable()` | — | Resolved sortability. |

### `TableUi<'a, T>`

| Method | Mirrors | Description |
|--------|---------|-------------|
| `header(\|h\| ..)` | `HeaderContent` / `MudTHeadRow` | Appends a header row. |
| `footer(\|f\| ..)` | `FooterContent` / `MudTFootRow` | Appends a footer row. |
| `row_template(f)` | `RowTemplate` | `FnMut(&mut RowUi, &T)`. |
| `child_row(f)` | `ChildRowContent` | `FnMut(&mut RowUi, &T)`. |

### `TableState<T>`

| Field | Default | Description |
|-------|---------|-------------|
| `sort_label` | `None` | Active sort key. |
| `sort_direction` | `None` | Active direction. |
| `current_page` | `0` | Zero-based page. |
| `rows_per_page` | `10` | `0` = all rows. |
| `selected_item` | `None` | Single selection. |
| `selected_items` | `[]` | Multi selection. |
| `last_row_rects` | `[]` | Row rectangles from the last frame. |
| `last_filtered_count` | `0` | Rows after filtering. |
| `last_total_count` | `0` | Rows passed to `show`. |
| `is_selected(&T)` | — | Mode-aware membership test. |
| `with_rows_per_page(n)` | — | Builder form. |

---

## Examples

### Basic sortable, striped table

```rust
MudTable::new()
    .striped(true)
    .hover(true)
    .show(ui, &people, &mut state, |t| {
        t.header(|h| {
            h.sort_th("Name", "name", |a, b| a.name.cmp(&b.name));
            h.sort_th("Age", "age", |a, b| a.age.cmp(&b.age));
        });
        t.row_template(|r, p| {
            r.td(Td::text(p.name.clone()).label("Name"));
            r.td(Td::text(p.age.to_string()).label("Age"));
        });
    });
```

### Multiselection with checkboxes

```rust
MudTable::new()
    .multi_selection(true)
    .show(ui, &people, &mut state, |t| {
        t.header(|h| { h.text("Name"); h.text("Age"); });
        t.row_template(|r, p| {
            r.td(Td::text(p.name.clone()).label("Name"));
            r.td(Td::text(p.age.to_string()).label("Age"));
        });
    });

for p in &state.selected_items {
    ui.label(&p.name);
}
```

### Filter + pager

```rust
MudTable::new()
    .filter(|p: &Person| p.age > 18)
    .rows_per_page(5)
    .page_size_options([5, 10, 25])
    .show(ui, &people, &mut state, |t| { /* … */ });
```

### Sticky header with a scrollable body

```rust
MudTable::new()
    .height(320.0)
    .fixed_header(true)
    .show(ui, &people, &mut state, |t| { /* … */ });
```

`fixed_header` only takes effect when `height` is set — same rule as
MudBlazor.

### Custom cells and right alignment

```rust
t.row_template(|r, p| {
    r.td(Td::text(p.name.clone()).label("Name"));
    r.td(
        Td::ui(|ui| {
            ui.add(MudButton::new("Edit").size(Size::Small));
        })
        .label("Actions"),
    );
    r.td(Td::text(format!("${}", p.salary)).align(ColumnAlign::Right).label("Salary"));
});
```

### Child rows

```rust
MudTable::new().show(ui, &people, &mut state, |t| {
    t.header(|h| h.text("Name"));
    t.row_template(|r, p| r.text(p.name.clone()));
    t.child_row(|r, p| {
        r.ui(|ui| {
            ui.add(MudText::new(format!("Age: {}", p.age)).typo(Typo::Caption));
        });
    });
});
```

`ChildRowContent` renders under **every** row, exactly as in MudBlazor.

### Multi-row header

```rust
t.header(|h| { h.text("Person"); h.text(""); });
t.header(|h| { h.text("Name");   h.text("Age"); });
```

Equivalent to `CustomHeader="true"` plus two `MudTHeadRow`s.

### Explicit column widths

```rust
MudTable::new()
    .column_widths([Some(60.0), None, Some(120.0)])
    .show(ui, &people, &mut state, |t| { /* … */ });
```

This is the egui form of `ColGroup`.

### Responsive stacking

```rust
MudTable::new()
    .breakpoint(Breakpoint::Md) // stack below 960 px
    .show(ui, &people, &mut state, |t| { /* … */ });
```

Below the breakpoint each row becomes a card, and each `Td` renders as
`label` + value. A cell without `.label(..)` renders as bare content —
MudBlazor behaves the same way when `DataLabel` is omitted.

### Row callbacks and classes

```rust
MudTable::new()
    .on_row_click(|p, idx| println!("{idx}: {}", p.name))
    .row_class_func(|p, _| if p.age > 30 { "bg-warning".into() } else { String::new() })
    .show(ui, &people, &mut state, |t| { /* … */ });
```

### Loading and empty states

```rust
MudTable::new()
    .loading(fetch_in_flight)
    .loading_text("Fetching…")
    .no_records_text("Nothing here yet")
    .show(ui, &rows, &mut state, |t| { /* … */ });
```

---

## Data pipeline

```
items ──► filter ──► sort_indices_by ──► clamp_page / page_range ──► page rows
            │              │                      │
         Filter         sort_label +          current_page +
                        sort_direction        rows_per_page
```

1. **Filter.** `filter` predicate, if set.
2. **Sort.** `sort_indices_by` with the comparator registered on the `Th`
   whose `sort_key` matches `state.sort_label`. Single-column only.
3. **Page.** `clamp_page` keeps `current_page` valid after filtering;
   `page_range` returns `[start, end)`.
4. **Row template.** Runs **once per visible row** into an owned `Vec<Td>`.
5. **Columns.** Count and widths resolved from `column_widths` → header rows →
   footer rows → widest body row. Minimum one column.
6. **Draw.** Header rows → body rows (optional checkbox column) → child rows
   → footer rows → pager.

Running the row template exactly once per row keeps `FnMut` side effects
predictable (no measure/draw double-run).

---

## Performance and memory choices

| Choice | Effect |
|--------|--------|
| Row template runs once per visible row | No double invocation; predictable `FnMut`. |
| One `Vec<Td>` per visible row | Bounded by `rows_per_page` (10–100); freed each frame. |
| Index-based filtering and sorting | `T` values never move; rows stay in place. |
| Stable `sort_indices_by` | Predictable ordering for equal keys. |
| Single-pass `resolve_column_widths` | One allocation for the result. |
| Header rows are `Vec<Vec<Th>>` | Multi-row headers cost nothing extra. |
| All text through `MudText`, icons through `MudIcon` | Theme/font updates propagate; no duplicated style logic. |
| `MudToggleGroup` for page sizes | Reuses the themed component instead of a raw combo box. |
| `ui.scope_id()` for interaction ids | Avoids the deprecated `ui.id()` and keeps ids unique per table. |
| `paint_checkbox` takes `&mut Ui` | `Ui::put` requires it; the glyph is `.to_owned()` to disambiguate `MudIcon::new`. |

Not done, on purpose: a column-width cache and a row-height cache. Both need
invalidation rules that belong to a layout engine; the table stays
deterministic instead.

---

## Testing

`cargo test` runs the module's tests:

| Area | Tests |
|------|-------|
| Row height | density defaults, `item_size` override, non-finite / negative repair |
| Breakpoints | Bootstrap thresholds, active edges, strict stacking below |
| Sorting | `toggle_sort` two-state and three-state cycles, column switching; `sort_indices_by` asc / desc / none |
| Pagination | `page_count` (zero rows / zero size), `clamp_page`, `page_range` |
| Widths | equal split, mixed fixed/flexible, non-finite available, over-constrained |
| Columns | explicit widths win, widest section wins, default of 1, first-hint-wins |
| Elevation | zero is no shadow, clamping |
| Selection | single keeps on re-click, multi toggles membership, `is_selected` covers both shapes |
| Builders | MudBlazor defaults, every field set |
| Cells | `Td` builders, `Th::is_sortable` requires key **and** comparator |
| Collectors | order preserved, multi-row headers append |
| Tag | round-trips by type |

Rendering (`show`) needs a live `egui::Context` and is not unit-tested; the pure
layer is the source of truth.

```bash
cargo test -p mud_egui_components mud_table
```

---

## Differences from MudBlazor

- **No inline editing.** `Editable`, `RowEditingTemplate`, `EditTrigger`,
  commit/cancel icons and the edit callbacks are out of scope; see the
  parity section.
- **No grouping.** `GroupBy` and the group templates are a separate widget.
- **No server data / virtualization.** Paging is local and rows are drawn
  eagerly for the current page.
- **`ColGroup` → `column_widths(iter)`.** Same purpose (per-column sizing),
  expressed as pixel hints because egui has no CSS `<col>`.
- **`Style` / `RowStyle` / `CellClass` → class strings resolved through the
  flavour.** `row_class`, `row_class_func`, `header_classes`,
  `footer_classes`, `container_classes` and `classes` feed the Bootstrap or
  Tailwind resolver. `CellClass` is not ported because `<td>` class semantics
  do not map to a single egui property.
- **`CustomHeader` / `CustomFooter` flags are implicit.** Calling
  `TableUi::header` / `TableUi::footer` more than once produces multi-row
  headers/footers, so the flags are unnecessary.
- **`SelectedItem` and `SelectedItems` are unified in `TableState<T>`.**
  Single selection writes `selected_item`, multi writes `selected_items`; the
  two are kept mutually exclusive by `toggle_selection`.
- **`PagerContent` → `pager(true)`.** The built-in pager reproduces
  `MudTablePager`'s page-size selector, range label and previous/next buttons.
- **`TableState` exposes `last_row_rects`.** An extension for hit-testing or
  overlays; MudBlazor has no equivalent because the DOM provides it.

===== ./crates/mud_egui_components/src/data_display/timeline/README.md =====

# MudTimeline for `mud_egui`

A chronological timeline for [egui](https://github.com/emilk/egui), ported
from MudBlazor's [`MudTimeline`](https://mudblazor.com/components/timeline)
and `MudTimelineItem` (MudBlazor 8.13.0).

```rust
let mut state = TimelineState::new();

MudTimeline::new()
.timeline_position(TimelinePosition::Alternate)
.timeline_orientation(TimelineOrientation::Vertical)
.show(ui, &mut state, |t| {
t.item(TimelineItem::text("Order placed").icon(filled::SHOPPING_CART));
t.item(
TimelineItem::text("Shipped")
.color(Color::Success)
.variant(Variant::Filled),
);
t.item(TimelineItem::text("Delivered").icon(filled::CHECK));
});
```

Every string renders through **`MudText`**, every glyph through **`MudIcon`**.

---

## Contents

1. [Design principle: API parity, not API invention](#design-principle-api-parity-not-api-invention)
2. [Architecture](#architecture)
3. [API reference](#api-reference)
4. [Examples](#examples)
5. [Layout and rendering](#layout-and-rendering)
6. [Performance and memory choices](#performance--memory-choices)
7. [Testing](#testing)
8. [Differences from MudBlazor](#differences-from-mudblazor)

---

## Design principle: API parity, not API invention

The MudBlazor 8.13.0 parameter surface was verified against the DNDocs pages
for `MudTimeline`, `MudTimelineItem`, `TimelineAlign`, `TimelineOrientation`,
and `TimelinePosition` before implementation.

Three new public enums (`TimelineAlign`, `TimelineOrientation`,
`TimelinePosition`) live in `mud_egui_core::enums`, each in its own file,
matching the project convention. No other enums are introduced.

### Ported `MudTimeline` parameters

| MudBlazor | Rust | Default |
|---|---|---|
| `Modifiers` | `modifiers(bool)` | `true` |
| `Reverse` | `reverse(bool)` | `false` |
| `RightToLeft` (cascading) | `right_to_left(bool)` | `false` |
| `SelectedIndex` | `TimelineState::selected_index` | `None` |
| `SelectedIndexChanged` | `on_selected_index_change(FnMut(usize))` | `None` |
| `TimelineAlign` | `timeline_align(TimelineAlign)` | `Default` |
| `TimelineOrientation` | `timeline_orientation(TimelineOrientation)` | `Vertical` |
| `TimelinePosition` | `timeline_position(TimelinePosition)` | `Alternate` |
| `ChildContent` | closure to `show` | required |
| `Class` | `classes(iter)` | `None` |
| `Style` | `frame(egui::Frame)` | `None` |
| `Tag` | `tag(t)` / `tag_ref::<U>()` | `None` |
| `UserAttributes` | **not ported** (no HTML tag) | — |

### Ported `MudTimelineItem` parameters

| MudBlazor | Rust | Default |
|---|---|---|
| `Color` | `TimelineItem::color(Color)` | `Default` |
| `Variant` | `TimelineItem::variant(Variant)` | `Outlined` |
| `Size` | `TimelineItem::size(Size)` | `Small` |
| `HideDot` | `TimelineItem::hide_dot(bool)` | `false` |
| `Icon` | `TimelineItem::icon(svg)` | `None` |
| `ItemDot` | `TimelineItem::dot(FnMut(&mut Ui))` | `None` |
| `ItemContent` | `TimelineItem::new(FnMut(&mut Ui))` | required |
| `ItemOpposite` | `TimelineItem::opposite(FnMut(&mut Ui))` | `None` |
| `TimelineAlign` | `TimelineItem::timeline_align(TimelineAlign)` | `None` (inherit) |
| `DotStyle` / `DotClass` | **not ported** (no CSS strings) | — |

### Enums

**`TimelineAlign`** — `Default` (centered), `Start`, `End`.

**`TimelineOrientation`** — `Vertical` (default), `Horizontal`.

**`TimelinePosition`** — `Alternate` (default), `Top`, `Bottom`, `Left`,
`Right`, `Start`, `End`.

Valid combinations:

| Orientation | Valid positions |
|---|---|
| `Vertical` | `Left`, `Right`, `Alternate`, `Start`, `End` |
| `Horizontal` | `Top`, `Bottom`, `Alternate` |

Cross-axis values are coerced to `Alternate` by `resolve_position`.

### Deliberately not ported

- **`Modifiers`** is stored for parity but has no visual effect (egui has no
  CSS pseudo-elements).
- **`DotStyle` / `DotClass`** — use `TimelineItem::dot(..)` for a fully
  custom dot renderer.
- **`UserAttributes`** — there is no HTML tag.

### Navigation helpers

`MudTimeline::move_to(state, index, count)`, `MudTimeline::next(state, count)`,
`MudTimeline::previous(state, count)` mirror `MudBaseItemsControl`'s
`MoveTo(int)`, `Next()`, `Previous()`.

---

## Architecture

```
mud_timeline.rs
├── MudTimeline<'a>        — configuration + render entry point
│   └── show(ui, state, |t| ..) -> (Response, R)
├── Cfg                    — immutable per-frame config
├── render_vertical(..)    — vertical axis renderer
├── render_horizontal(..)  — horizontal axis renderer
├── paint_dot(..)          — default / custom dot
├── Pure helpers           — dot_diameter, dot_span, alternate_side,
│                            item_sides, resolve_position, effective_align,
│                            dot_y_offset, dot_x_offset
└── tests                  — 23 pure-function tests

timeline_item.rs           — TimelineItem<'a> (data model)
timeline_state.rs          — TimelineState (caller-owned selection)
timeline_ui.rs             — TimelineUi<'a> (collector)
```

**Pure layer.** Every layout decision is a free function, unit-tested without
an `egui::Context`.

**State layer.** `TimelineState` is caller-owned — the egui equivalent of
`SelectedIndex`.

**Component layer.** `show` resolves the theme and frame, calls the pure layer,
and paints.

---

## API reference

### `MudTimeline<'a>`

| Method | Default | Mirrors | Description |
|---|---|---|---|
| `new()` | — | — | MudBlazor defaults. |
| `modifiers(bool)` | `true` | `Modifiers` | Stored for parity; no visual effect. |
| `reverse(bool)` | `false` | `Reverse` | Flip the alternate side order. |
| `right_to_left(bool)` | `false` | `RightToLeft` | Resolve `Start`/`End` for RTL. |
| `timeline_position(TimelinePosition)` | `Alternate` | `TimelinePosition` | Where items sit relative to the axis. |
| `timeline_orientation(TimelineOrientation)` | `Vertical` | `TimelineOrientation` | Axis direction. |
| `timeline_align(TimelineAlign)` | `Default` | `TimelineAlign` | Default dot alignment. |
| `selectable(bool)` | `true` | — | Rows write `selected_index`. |
| `on_selected_index_change(FnMut(usize))` | `None` | `SelectedIndexChanged` | Click callback. |
| `on_item_click(FnMut(usize))` | alias | — | Alias for the above. |
| `height(f32)` | `None` | `Style="max-height:…"` | Vertical scroll cap. |
| `item_width(f32)` | `240.0` | — | Horizontal item width. |
| `item_height(f32)` | `200.0` | — | Horizontal item height. |
| `classes(iter)` | none | `Class` | Wrapper utility classes. |
| `flavor(StyleFlavor)` | default | — | Bootstrap / Tailwind. |
| `frame(egui::Frame)` | `None` | `Style` | Replace the wrapper frame. |
| `aria_label(s)` | `None` | — | Accessible label. |
| `tag(t)` / `tag_ref::<U>()` | `None` | `Tag` | Arbitrary data. |
| `show(ui, state, \|t\| ..)` | — | — | Returns `(Response, R)`. |

### `TimelineItem<'a>`

| Method | Mirrors | Description |
|---|---|---|
| `new(FnMut(&mut Ui))` | `ItemContent` | Required content closure. |
| `text(s)` | `ItemContent` (text) | Convenience text content. |
| `color(Color)` | `Color` | Dot accent. |
| `variant(Variant)` | `Variant` | `Outlined` / `Filled` / `Text`. |
| `size(Size)` | `Size` | `Small` (12px) / `Medium` (16px) / `Large` (20px). |
| `hide_dot(bool)` | `HideDot` | Hide the dot, keep the line. |
| `icon(svg)` | `Icon` | Icon inside the dot. |
| `dot(FnMut(&mut Ui))` | `ItemDot` | Custom dot renderer. |
| `content(FnMut(&mut Ui))` | `ItemContent` | Replace the content. |
| `opposite(FnMut(&mut Ui))` | `ItemOpposite` | Opposite-side content (Alternate only). |
| `timeline_align(TimelineAlign)` | `TimelineAlign` | Per-item alignment override. |
| `with_check_icon()` | — | Convenience: Material check icon. |

### `TimelineState`

| Field | Description |
|---|---|
| `selected_index: Option<usize>` | Currently selected item. |
| `last_item_rects: Vec<Rect>` | Item row rectangles from the last frame. |
| `last_dot_centers: Vec<Pos2>` | Dot centres from the last frame. |

### Geometry constants

| Name | Value |
|---|---|
| `DOT_SMALL` | `12.0` |
| `DOT_MEDIUM` | `16.0` |
| `DOT_LARGE` | `20.0` |
| `DOT_PADDING` | `8.0` |
| `ITEM_GAP` | `8.0` |
| `CONNECTOR_THICKNESS` | `2.0` |
| `DEFAULT_ITEM_WIDTH` | `240.0` |
| `DEFAULT_ITEM_HEIGHT` | `200.0` |

---

## Examples

### Vertical, left-aligned

```rust
MudTimeline::new()
    .timeline_position(TimelinePosition::Left)
    .show(ui, &mut state, |t| {
        t.item(TimelineItem::text("Step 1").color(Color::Success).variant(Variant::Filled));
        t.item(TimelineItem::text("Step 2").color(Color::Success).variant(Variant::Filled));
        t.item(TimelineItem::text("Step 3"));
    });
```

### Vertical, alternating

```rust
MudTimeline::new()
    .timeline_position(TimelinePosition::Alternate)
    .show(ui, &mut state, |t| {
        t.item(TimelineItem::text("Jan 2024").opposite(|ui| {
            ui.add(MudText::new("Order placed").typo(Typo::Body2));
        }));
        t.item(TimelineItem::text("Feb 2024").opposite(|ui| {
            ui.add(MudText::new("Shipped").typo(Typo::Body2));
        }));
    });
```

### Horizontal

```rust
MudTimeline::new()
    .timeline_orientation(TimelineOrientation::Horizontal)
    .timeline_position(TimelinePosition::Top)
    .item_width(220.0)
    .item_height(160.0)
    .show(ui, &mut state, |t| {
        t.item(TimelineItem::text("Q1"));
        t.item(TimelineItem::text("Q2"));
        t.item(TimelineItem::text("Q3"));
    });
```

### Reverse alternate

```rust
MudTimeline::new()
    .timeline_position(TimelinePosition::Alternate)
    .reverse(true) // first item goes right instead of left
    .show(ui, &mut state, |t| { /* … */ });
```

### Sizes and variants

```rust
TimelineItem::text("Small outlined")   // defaults: Size::Small, Variant::Outlined
TimelineItem::text("Medium filled").size(Size::Medium).variant(Variant::Filled)
TimelineItem::text("Large").size(Size::Large).icon(filled::STAR)
```

### Custom dot

```rust
TimelineItem::text("Custom dot").dot(|ui| {
    ui.add(MudAvatar::new().text("A").size(Size::Small));
})
```

### HideDot (continuation line)

```rust
TimelineItem::text("continued...").hide_dot(true)
```

### Scrollable vertical timeline

```rust
MudTimeline::new()
.height(400.0)
.show(ui, &mut state, |t| { /* many items */ });
```

### Selection callback

```rust
MudTimeline::new()
.on_selected_index_change(|i| println!("selected {i}"))
.show(ui, &mut state, |t| { /* … */ });

// After the frame, `state.selected_index` holds the clicked index.
```

### Per-item alignment

```rust
MudTimeline::new()
    .timeline_align(TimelineAlign::Default)
    .show(ui, &mut state, |t| {
        t.item(TimelineItem::text("centered")); // inherits Default
        t.item(TimelineItem::text("top").timeline_align(TimelineAlign::Start));
        t.item(TimelineItem::text("bottom").timeline_align(TimelineAlign::End));
    });
```

---

## Layout and rendering

### Vertical

Each item is a horizontal band:

| Position | Row layout |
|---|---|
| `Left` / `Start` | `[ dot ][ content ]` |
| `Right` / `End` | `[ content ][ dot ]` |
| `Alternate` | `[ content \| opposite ][ dot ][ opposite \| content ]` |

Content height is natural (measured by egui). The dot's Y position within the
row is controlled by `TimelineAlign`:

| Align | Dot Y |
|---|---|
| `Default` | row centre |
| `Start` | `DOT_PADDING + diameter/2` from the top |
| `End` | `DOT_PADDING + diameter/2` from the bottom |

### Horizontal

Each item is a fixed-size box (`item_width` × `item_height`):

| Position | Item layout |
|---|---|
| `Top` | `[ dot row ][ content ]` |
| `Bottom` | `[ content ][ dot row ]` |
| `Alternate` | `[ content \| opposite ][ dot row ][ opposite \| content ]` |

The dot's X position within the box is controlled by `TimelineAlign`:

| Align | Dot X |
|---|---|
| `Default` | box centre |
| `Start` | `DOT_PADDING + diameter/2` from the left |
| `End` | `DOT_PADDING + diameter/2` from the right |

### Connector

A single straight line runs through every dot centre, from the first to the
last. It is painted **behind** items by reserving a shape slot with
`ui.painter().add(Shape::Noop)` before rendering and filling it with
`ui.painter().set(slot, Shape::line_segment(..))` afterwards. This is the same
pattern used by `MudToggleGroup`'s underline fill and `MudButton`'s underfill.

### Dot rendering

| Variant | Appearance |
|---|---|
| `Outlined` (default) | surface fill + 2 px accent stroke |
| `Filled` | accent fill, no stroke |
| `Text` | accent fill (no text inside a dot) |

`Icon` renders inside the dot, colour = contrast text (Filled/Text) or accent
(Outlined). `ItemDot` overrides the entire default circle.

### `ItemOpposite`

Only rendered when `TimelinePosition::Alternate`, matching MudBlazor issue
#11799. In non-alternate positions the opposite closure is ignored.

---

## Performance and memory choices

| Choice | Effect |
|---|---|
| Items are collected once per frame and rendered once | No retained render tree. |
| `TimelineState` is caller-owned | Selection survives frames. |
| Connector uses a single reserved `Shape` slot | One shape instead of N segments. |
| Dots are painted after the row layout | No chicken-and-egg for alignment. |
| `dot_span` is a pure function | Column width is fixed before rendering. |
| All text through `MudText`, icons through `MudIcon` | Theme/font updates propagate. |
| Horizontal items use fixed boxes | Deterministic connector Y. |
| No per-frame `Vec` beyond `items` and `last_*` | Steady-state frames allocate nothing else. |

Not done, on purpose: natural-width horizontal items. Doing so would require
measuring every item's content before laying out, which is what
`MudGrid::AutoMeasure::Probe` does for grid cells; horizontal timelines in
MudBlazor are effectively fixed-size carousels, so fixed boxes are the
faithful choice.

---

## Testing

`cargo test` runs 23 pure-function tests:

| Area | Tests |
|---|---|
| Dot geometry | `dot_diameter` for every `Size`, `dot_span` adds padding |
| Alternate sides | alternation, reverse flips |
| Item sides | vertical Left/Right, horizontal Top/Bottom, alternate opposites |
| Position resolution | cross-axis coercion, `Start`/`End` normalisation with RTL |
| Alignment | `effective_align` prefers item override, `dot_y_offset`/`dot_x_offset` for every align |
| Defaults | match MudBlazor for every field |
| Builders | every field set, `item_width` clamp, `height` clamp |
| Navigation | `move_to` clamps, `next`/`previous` wrap |
| Tag | round-trips by type |

Rendering (`show`) needs a live `egui::Context` and is not unit-tested; the
pure layer is the source of truth.

```bash
cargo test -p mud_egui_components mud_timeline
```

---

## Differences from MudBlazor

- **`Modifiers` has no effect.** egui has no CSS pseudo-elements; the
  parameter is stored for parity.
- **`DotStyle` / `DotClass` not ported.** Use `TimelineItem::dot(..)`.
- **Horizontal items are fixed-size.** MudBlazor's horizontal timeline is a
  CSS flex row where items size to content; egui cannot measure all items
  before laying out without a two-pass probe. `item_width` / `item_height`
  give the caller the same control without the cost.
- **Selection is opt-in.** MudBlazor's `SelectedIndex` is a bindable
  parameter; this port writes it on click when `selectable(true)` (default).
  Set `selectable(false)` to make the timeline display-only.
- **`ItemOpposite` is only rendered in `Alternate`.** Same as MudBlazor
  (issue #11799).
- **Cross-axis positions are coerced to `Alternate`.** MudBlazor renders an
  empty or broken timeline; the port normalises to `Alternate` so the result
  is always usable.
- **Connector line is always drawn.** MudBlazor's `HideDot` hides the dot but
  keeps the line; the port matches that.

===== ./crates/mud_egui_components/src/data_display/tree_view/README.md =====

# MudTreeView for `mud_egui`

A recursive, selectable, expandable tree for [egui](https://github.com/emilk/egui),
ported from MudBlazor's [`MudTreeView<T>`](https://mudblazor.com/components/treeview),
`MudTreeViewItem<T>`, and `MudTreeViewItemToggleButton` (MudBlazor 8.13.0).

```rust
let mut state = TreeState::<&str>::new();

MudTreeView::new()
    .selection_mode(SelectionMode::Single)
    .show(ui, &mut state, |tree| {
        tree.item(
            TreeItem::text("documents", "Documents")
                .icon(filled::FOLDER)
                .child(TreeItem::text("readme", "README.md"))
                .child(TreeItem::text("license", "LICENSE")),
        );
        tree.item(TreeItem::text("downloads", "Downloads").icon(filled::DOWNLOAD));
    });
```

Every label renders through **`MudText`**, every glyph through **`MudIcon`**.

---

## Contents

1. [Design principle: API parity, not API invention](#design-principle-api-parity-not-api-invention)
2. [Architecture](#architecture)
3. [API reference](#api-reference)
4. [Examples](#examples)
5. [Auto behaviour](#auto-behaviour)
6. [Performance and memory choices](#performance--memory-choices)
7. [Testing](#testing)
8. [Differences from MudBlazor](#differences-from-mudblazor)

---

## Design principle: API parity, not API invention

The MudBlazor parameter surface was verified against DNDocs for 8.13.0 before
this port[reference:4][reference:5][reference:6][reference:7]. The tree is
**row-template driven** in the same sense as `MudTable<T>`: you supply a
closure that builds the hierarchy with `TreeItem` values; the component
handles selection, expansion, auto-behaviour and rendering.

### Ported parameters

| MudBlazor `MudTreeView<T>` | Rust | Default |
|---|---|---|
| `AutoExpand` | `auto_expand(bool)` | `false` |
| `AutoSelectParent` | `auto_select_parent(bool)` | `true` |
| `CheckBoxColor` | `check_box_color(Color)` | `Default` |
| `CheckedIcon` | `checked_icon(s)` | `CHECK_BOX` |
| `ChildContent` | the closure to `show` | required |
| `Color` | `color(Color)` | `Primary` |
| `Comparer` | *(Rust `PartialEq`)* | — |
| `Dense` | `dense(bool)` | `false` |
| `Disabled` | `disabled(bool)` | `false` |
| `ExpandOnClick` | `expand_on_click(bool)` | `false` |
| `ExpandOnDoubleClick` | `expand_on_double_click(bool)` | `false` |
| `FilterFunc` | `filter(Fn(&T) -> bool)` | `None` |
| `Height` | `height(f32)` | `None` |
| `Hover` | `hover(bool)` | `false` |
| `IndeterminateIcon` | `indeterminate_icon(s)` | `INDETERMINATE_CHECK_BOX` |
| `MaxHeight` | `max_height(f32)` | `None` |
| `ReadOnly` | `read_only(bool)` | `false` |
| `Ripple` | `ripple(bool)` | `true` |
| `SelectedValue` | `TreeState::selected_value` | `None` |
| `SelectedValues` | `TreeState::selected_values` | `[]` |
| `SelectionMode` | `selection_mode(SelectionMode)` | `Single` |
| `TriState` | `tri_state(bool)` | `true` |
| `UncheckedIcon` | `unchecked_icon(s)` | `CHECK_BOX_OUTLINE_BLANK` |
| `Width` | `width(f32)` | `None` |

### Ported `TreeItem` parameters

| MudBlazor `MudTreeViewItem<T>` | Rust | Default |
|---|---|---|
| `BodyContent` | `body(f)` | `None` |
| `CanExpand` | `can_expand(bool)` | `true` |
| `ChildContent` | `child(item)` / `children(iter)` | none |
| `Disabled` | `disabled(bool)` | `false` |
| `EndIcon` | `end_icon(s)` | `None` |
| `EndIconColor` | `end_icon_color(Color)` | `Inherit` |
| `EndText` | `end_text(s)` | `None` |
| `EndTextTypo` | `end_text_typo(Typo)` | `Body1` |
| `ExpandButtonIcon` | `expand_button_icon(s)` | `CHEVRON_RIGHT` |
| `ExpandButtonIconColor` | `expand_button_icon_color(Color)` | `Default` |
| `Expanded` | `expanded(bool)` | `None` |
| `Icon` | `icon(s)` | `None` |
| `IconColor` | `icon_color(Color)` | `Inherit` |
| `IconExpanded` | `icon_expanded(s)` | `None` |
| `OnClick` / `OnDoubleClick` | `on_click(f)` / `on_double_click(f)` | `None` |
| `ReadOnly` | `read_only(bool)` | `false` |
| `Text` | `with_text(s)` / `text(value, s)` | `None` |
| `Value` | `new(value)` | required |
| `Visible` | `visible(bool)` | `true` |

### Deliberately not ported

- **`Items` + `ItemTemplate`** — MudBlazor's data-model mode. The Rust port
  uses the closure-driven builder (`TreeItem::child(..)`), which is the
  idiomatic egui equivalent.
- **`ServerData`** — async lazy loading. egui has no per-frame async
  runtime; the caller can implement lazy loading by swapping tree contents
  in a click handler.
- **`Content`** (full-row replacement) — `body(..)` replaces the icon + text
  + end area, which covers the common case.
- **`FilterFunc` is synchronous** — `filter(Fn(&T) -> bool)`. Async filters
  are not supported.
- **`Style` / `UserAttributes`** — no CSS strings or HTML tags in egui.

---

## Architecture

```
mud_tree_view.rs
├── MudTreeView<'a, T>          — configuration + render entry point
│   └── show(ui, state, |tree| ..) -> (Response, R)
├── DrawCfg                     — immutable per-frame config
├── render_node(..)             — recursive row painter
├── auto_select_pass(..)        — bottom-up parent/child invariant
├── collect_ancestors_of_selected(..)
├── compute_tri_state(..)
├── is_visible(..)
├── handle_row_click / handle_checkbox_click / cascade_set
└── tests                       — 24 pure-function tests

tree_state.rs                   — TreeState<T> (caller-owned)
tree_item.rs                    — TreeItem<'a, T> (node data model)
tree_ui.rs                      — TreeUi<'a, T> (collector)
tri_state_kind.rs               — pub(crate) TriStateKind
```

**Pure layer.** `compute_tri_state`, `auto_select_pass`,
`collect_ancestors_of_selected`, `is_visible`, `cascade_set` are free
functions, unit-tested without an `egui::Context`.

**State layer.** `TreeState<T>` is caller-owned — the egui equivalent of
`SelectedValue`, `SelectedValues`, and per-item `Expanded`.

**Component layer.** `show` runs the two pre-passes, then recurses through
`render_node`.

---

## API reference

### `MudTreeView<'a, T>`

| Method | Default | Mirrors | Description |
|---|---|---|---|
| `new()` | — | — | MudBlazor defaults. |
| `filter(Fn(&T) -> bool)` | `None` | `FilterFunc` | Node visibility predicate. |
| `selection_mode(SelectionMode)` | `Single` | `SelectionMode` | `Single` / `Toggle` / `Multi`. |
| `color(Color)` | `Primary` | `Color` | Selected-row accent. |
| `check_box_color(Color)` | `Default` | `CheckBoxColor` | Checkbox tint. |
| `checked_icon(s)` | `CHECK_BOX` | `CheckedIcon` | Checked glyph. |
| `unchecked_icon(s)` | `CHECK_BOX_OUTLINE_BLANK` | `UncheckedIcon` | Unchecked glyph. |
| `indeterminate_icon(s)` | `INDETERMINATE_CHECK_BOX` | `IndeterminateIcon` | Partial-check glyph. |
| `tri_state(bool)` | `true` | `TriState` | Partial-state glyphs. |
| `auto_expand(bool)` | `false` | `AutoExpand` | Expand ancestors of selection. |
| `auto_select_parent(bool)` | `true` | `AutoSelectParent` | Derived parent state. |
| `read_only(bool)` | `false` | `ReadOnly` | Freeze selection. |
| `expand_on_click(bool)` | `false` | `ExpandOnClick` | Row click expands. |
| `expand_on_double_click(bool)` | `false` | `ExpandOnDoubleClick` | Double-click expands. |
| `dense(bool)` | `false` | `Dense` | Compact rows. |
| `hover(bool)` | `false` | `Hover` | Row hover fill. |
| `ripple(bool)` | `true` | `Ripple` | Click ripple. |
| `disabled(bool)` | `false` | `Disabled` | Disable everything. |
| `height(f32)` | `None` | `Height` | Fixed height + scroll. |
| `max_height(f32)` | `None` | `MaxHeight` | Maximum height + scroll. |
| `width(f32)` | `None` | `Width` | Fixed width. |
| `expand_all(bool)` | `false` | `ExpandAllAsync` | Force-expand all nodes. |
| `collapse_all(bool)` | `false` | `CollapseAllAsync` | Force-collapse all nodes. |
| `aria_label(s)` | `None` | — | Accessible label. |
| `classes(iter)` | none | `Class` | Utility classes. |
| `flavor(StyleFlavor)` | default | — | Bootstrap / Tailwind. |
| `tag(t)` / `tag_ref::<U>()` | `None` | `Tag` | Arbitrary data. |
| `show(ui, state, \|tree\| ..)` | — | — | Returns `(Response, R)`. |

### `TreeItem<'a, T>`

| Method | Mirrors | Description |
|---|---|---|
| `new(value)` | `Value` | Required value. |
| `text(value, s)` | `Value` + `Text` | Convenience constructor. |
| `with_text(s)` | `Text` | Caption. |
| `child(item)` | `ChildContent` | Adds one child. |
| `children(iter)` | `ChildContent` | Adds several children. |
| `icon(s)` | `Icon` | Leading icon. |
| `icon_color(Color)` | `IconColor` | Default `Inherit`. |
| `icon_expanded(s)` | `IconExpanded` | Swapped when expanded. |
| `end_icon(s)` | `EndIcon` | Trailing icon. |
| `end_icon_color(Color)` | `EndIconColor` | |
| `end_text(s)` | `EndText` | Trailing text. |
| `end_text_typo(Typo)` | `EndTextTypo` | Default `Body1`. |
| `expand_button_icon(s)` | `ExpandButtonIcon` | Default `CHEVRON_RIGHT`. |
| `expand_button_icon_color(Color)` | `ExpandButtonIconColor` | Default `Default`. |
| `expanded(bool)` | `Expanded` | Seed only. |
| `can_expand(bool)` | `CanExpand` | Default `true`. |
| `disabled(bool)` | `Disabled` | |
| `read_only(bool)` | `ReadOnly` | |
| `visible(bool)` | `Visible` | |
| `body(f)` | `BodyContent` | Custom row body. |
| `on_click(f)` | `OnClick` | `FnMut(&T)`. |
| `on_double_click(f)` | `OnDoubleClick` | `FnMut(&T)`. |

### `TreeState<T>`

| Field / method | Description |
|---|---|
| `selected_value: Option<T>` | Single / toggle selection. |
| `selected_values: Vec<T>` | Multi selection. |
| `expanded: Vec<(T, bool)>` | Expansion state (seeded from `TreeItem::expanded`). |
| `last_node_rects: Vec<Rect>` | Screen-space row rectangles from the last frame. |
| `is_selected(&T, SelectionMode)` | Mode-aware membership test. |
| `is_expanded(&T)` / `ensure_expanded` / `toggle_expanded` / `clear_expanded` | Expansion helpers. |

### Geometry

| Constant | Value |
|---|---|
| `ROW_HEIGHT` | `40.0` |
| `DENSE_ROW_HEIGHT` | `32.0` |
| `INDENT_PER_DEPTH` | `24.0` |

---

## Examples

### File browser

```rust
TreeItem::text("documents", "Documents")
    .icon(filled::FOLDER)
    .child(TreeItem::text("readme", "README.md").icon(filled::DESCRIPTION))
    .child(TreeItem::text("src", "src").icon(filled::FOLDER)
        .child(TreeItem::text("main", "main.rs").icon(filled::CODE)))
```

### Multi-select with tri-state

```rust
MudTreeView::new()
    .selection_mode(SelectionMode::Multi)
    .tri_state(true)
    .auto_select_parent(true)
    .show(ui, &mut state, |tree| {
        tree.item(
            TreeItem::text("fruits", "Fruits")
                .child(TreeItem::text("apple", "Apple"))
                .child(TreeItem::text("banana", "Banana")),
        );
    });
```

Selecting both children auto-checks the parent. Unchecking one makes the
parent indeterminate.

### Filter

```rust
MudTreeView::new()
    .filter(|v: &&str| v.contains(query))
    .show(ui, &mut state, |tree| { /* … */ });
```

A branch stays visible when it or any descendant matches.

### Expand-all / collapse-all

```rust
MudTreeView::new()
    .expand_all(true)  // ExpandAllAsync equivalent
    .show(ui, &mut state, |tree| { /* … */ });
```

### Read-only, hover, dense

```rust
MudTreeView::new()
    .read_only(true)
    .hover(true)
    .dense(true)
    .show(ui, &mut state, |tree| { /* … */ });
```

### Custom row body

```rust
TreeItem::text("user", "User")
    .body(|ui, is_selected| {
        ui.add(MudText::new("User").typo(Typo::Body1));
        ui.add(MudText::new("admin").typo(Typo::Caption).color(Color::TextSecondary));
    })
```

### Scrollable tree

```rust
MudTreeView::new()
    .max_height(300.0)
    .show(ui, &mut state, |tree| { /* … */ });
```

---

## Auto behaviour

### `auto_expand`

Before rendering, `collect_ancestors_of_selected` walks the tree and
collects the value of every node that has a selected descendant. Those
values are force-expanded for the frame, so the selected item is always
visible.

### `auto_select_parent`

Before rendering, `auto_select_pass` walks the tree bottom-up and enforces:

> A parent is in `selected_values` **iff** every direct child is checked.

A click on a leaf therefore reaches the parent with a one-frame delay —
the same latency you would get from a Blazor event handler.

### `tri_state`

In Multi mode, a parent's checkbox is painted based on its **descendants**
(not its own `selected` flag):

| Descendant state | Parent checkbox |
|---|---|
| All checked | Checked |
| Some checked, some unchecked | Indeterminate |
| None checked | Unchecked |

Leaves use their own `selected_values` membership.

### Click targets

| Target | Single / Toggle | Multi |
|---|---|---|
| Expand button | Toggle expand | Toggle expand |
| Checkbox | Toggle node | Toggle whole subtree (cascade) |
| Row body | Select node | Toggle node |
| Double-click | Expand (if `expand_on_double_click`) | Same |

The row body also expands when `expand_on_click(true)`.

---

## Performance and memory choices

| Choice | Effect |
|---|---|
| Tree is built once per frame from `TreeItem` values | No retained render tree. |
| `TreeState` is caller-owned | Selection / expansion survive frames. |
| Recursion is O(n) per visible node | Typical trees (<1000 nodes) render in microseconds. |
| `expanded` is a `Vec<(T, bool)>` with linear search | `T: PartialEq` suffices; no `Hash` bound. |
| Interaction IDs are path-based | `T` needs no `Hash`. |
| Pre-passes are linear | `auto_select_pass` and `collect_ancestors_of_selected` are one walk each. |
| Ripple reads `ui.input(..)` **before** `memory_mut(..)` | Avoids the re-entrant `RwLock` deadlock documented in `MudToggleGroup`. |
| All text through `MudText`, icons through `MudIcon` | Theme/font updates propagate automatically. |

Not done, on purpose: virtualisation and row-height measurement. Both need a
layout engine; the tree stays deterministic.

---

## Testing

`cargo test` runs 24 pure-function tests:

| Area | Tests |
|---|---|
| `TreeState` | empty defaults, `ensure_expanded` seeds once, `toggle_expanded` flips/creates, `is_selected` mode-aware |
| `TreeItem` | defaults, `child` / `children` order |
| Visibility | `visible(false)` hides subtree, filter matches self or descendant, filter matches leaf |
| Tri-state | leaf uses own state, parent mixed → indeterminate, parent all → checked, parent none → unchecked |
| Auto-select-parent | adds parent when all children checked, removes parent when some unchecked, disabled keeps user choice |
| Auto-expand | ancestors collected, none when nothing selected |
| Cascade | check every descendant, uncheck every descendant |
| Builders | MudBlazor defaults, every field set, tag round-trip |
| Collector | root order preserved |

Rendering (`show`) needs a live `egui::Context` and is not unit-tested;
the pure layer is the source of truth.

```bash
cargo test -p mud_egui_components mud_tree_view
```

---

## Differences from MudBlazor

- **No `Items` / `ItemTemplate`.** The closure-driven builder is the
  idiomatic egui equivalent and avoids a separate `TreeItemData<T>` type.
- **No `ServerData`.** Lazy loading would need an async runtime per frame;
  the caller can swap tree contents in a click handler instead.
- **No `Content`.** `body(..)` replaces the icon + text + end area;
  `Content` would replace the whole row, which conflicts with the built-in
  expand button and checkbox.
- **`FilterFunc` is synchronous.** `filter(Fn(&T) -> bool)`.
- **`Style` → `classes(..)`.** egui has no inline CSS strings.
- **`UserAttributes` not ported.** There is no HTML tag.
- **`Expanded` is seeded once, not two-way bound.** Setting
  `TreeItem::expanded(true)` initialises the state; subsequent clicks toggle
  it. To control expansion manually, update `TreeState::expanded` yourself.
- **`auto_select_parent` runs at render time**, not in the click handler, so
  a leaf click reaches the parent one frame later. Visually imperceptible.
- **Single public enum only.** `SelectionMode` is reused; the only new enum
  is the private `TriStateKind`, which lives in its own file.

===== ./crates/mud_egui_components/src/feedback/alert/README.md =====

# MudAlert for `mud_egui`

A statically embedded feedback banner for [egui](https://github.com/emilk/egui),
ported from MudBlazor's [`MudAlert`](https://mudblazor.com/components/alert)
(MudBlazor 8.13.0). Five severities, three variants, an optional leading icon and
close button, RTL-aware content alignment, and a configurable drop shadow.

```rust
let out = MudAlert::new("The reactor is running at optimum temperature")
    .severity(Severity::Success)
    .variant(Variant::Filled)
    .show(ui);

if out.close_clicked {
    self.hide_banner = true;
}
```

Every string renders through **`MudText`**, every glyph through **`MudIcon`**.

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

The MudBlazor 8.13.0 parameter surface was verified against DNDocs
(`MudBlazor.MudAlert`) before implementation. `MudAlert` is a **display**
component: it has no `Value` / `ValueChanged` / `ValueExpression`, no `Size`, no
`Color`, no `Disabled` / `ReadOnly`, and no adornment slots. None of those were
added.

### Ported parameters

| MudBlazor | Type | Default | Rust mapping |
|---|---|---|---|
| `ChildContent` | `RenderFragment?` | `null` | `new(text)` / `text(s)` / `content(f)` |
| `CloseIcon` | `string` | `Icons.Material.Filled.Close` | `close_icon(s)` |
| `CloseIconClicked` | `EventCallback<MudAlert>` | — | `AlertResponse::close_clicked` + `on_close(f)` |
| `ContentAlignment` | `HorizontalAlignment` | `Left` | `content_alignment(HorizontalAlignment)` |
| `Dense` | `bool` | `false` | `dense(b)` |
| `Elevation` | `int` | `0` | `elevation(u8)` |
| `Icon` | `string?` | `null` | `icon(s)` |
| `NoIcon` | `bool` | `false` | `no_icon(b)` |
| `OnClick` | `EventCallback<MouseEventArgs>` | — | `AlertResponse::body_clicked` + `on_click(f)` |
| `RightToLeft` | `bool` (cascading) | `false` | `right_to_left(b)` |
| `Severity` | `Severity` | `Normal` | `severity(Severity)` |
| `ShowCloseIcon` | `bool` | `false` | `show_close_icon(b)` |
| `Square` | `bool` | `false` | `square(b)` |
| `Variant` | `Variant` | `Text` | `variant(Variant)` |
| `Class` | `string?` | `null` | `classes(iter)` — stored, not resolved |
| `Tag` | `object?` | `null` | `tag(t)` / `tag_ref::<U>()` |

`ChildContent` is one parameter with two Rust expressions: `text(..)` for a plain
string, `content(..)` for custom widgets. When both are set, `content` wins. This
mirrors `MudAvatar` (`text` vs `child`) and `MudListItem` (`text` vs `child`).

### Deliberately not ported

| MudBlazor | Reason |
|---|---|
| `Style` | egui has no inline CSS strings. The variant already owns fill, border and radius, so there is nothing meaningful left to style. |
| `UserAttributes` | There is no HTML tag to splat attributes onto. |
| `Class` resolution | Stored on `base` for parity and future use, but **not** resolved into a frame: `Variant` and `Severity` already determine the fill, border and radius, and resolving a second frame would mean compositing two fills over each other. Use the builder methods instead. |

---

## Architecture

```
mud_alert.rs
├── constants            PADDING_X/Y, DENSE_PADDING_X/Y, MIN_ROW_HEIGHT,
│                        ICON_SIZE, ICON_GAP, CLOSE_GAP, CLOSE_HIT,
│                        CLOSE_GLYPH, DEFAULT_CORNER_RADIUS, MAX_ELEVATION, ...
├── pure helpers         severity_color, severity_icon, resolve_icon,
│                        contrast_text, with_alpha, resolve_style,
│                        padding, min_row_height, corner_radius, radius_to_u8,
│                        finite_or, resolve_alignment, icon_column_width,
│                        close_column_width, message_width, row_height,
│                        layout_columns, elevation_shadow
├── MudAlert             builder struct (all fields public)
│   ├── builders         severity, variant, dense, square, elevation, no_icon,
│   │                    icon, close_icon, show_close_icon, content_alignment,
│   │                    right_to_left, on_click, on_close, classes, tag
│   └── show             theme fetch → style → measure → allocate → paint
├── AlertResponse        response + body_clicked + close_clicked
├── impl Widget          ui.add(MudAlert::new("…"))
└── mod tests            27 pure-function tests
```

**Pure layer.** Every geometry and colour decision is a free function over
`f32`, `Rect`, `Color32` and small enums. Nothing in it touches `egui::Context`,
so nearly all behaviour is covered by ordinary unit tests.

**Component layer.** `show` fetches the theme once, resolves one accent colour,
calls the pure layer, and paints. It holds no state between frames.

**Response layer.** `AlertResponse` reports this frame's outcome the way egui
reports interaction: as data, not as a stored event.

---

## API reference

### `MudAlert`

| Method | Type | Default | Mirrors | Description |
|---|---|---|---|---|
| `new(text)` | `impl Into<String>` | `""` | `ChildContent` | Creates an alert with MudBlazor defaults. |
| `text(s)` | `impl Into<String>` | — | `ChildContent` | Replaces the plain-text message. |
| `content(f)` | `FnMut(&mut Ui) + 'static` | `None` | `ChildContent` | Custom content; wins over `text`. |
| `severity(s)` | `Severity` | `Normal` | `Severity` | Accent colour and default icon. |
| `variant(v)` | `Variant` | `Text` | `Variant` | `Text` / `Outlined` / `Filled`. |
| `dense(b)` | `bool` | `false` | `Dense` | Compact padding. |
| `square(b)` | `bool` | `false` | `Square` | Zero corner radius. |
| `elevation(e)` | `u8` | `0` | `Elevation` | Drop shadow, clamped to 24. |
| `no_icon(b)` | `bool` | `false` | `NoIcon` | Suppresses the leading icon. |
| `icon(s)` | `impl Into<String>` | `None` | `Icon` | Overrides the severity glyph. |
| `close_icon(s)` | `impl Into<String>` | `None` | `CloseIcon` | Overrides the close glyph. |
| `show_close_icon(b)` | `bool` | `false` | `ShowCloseIcon` | Shows the close button. |
| `content_alignment(a)` | `HorizontalAlignment` | `Left` | `ContentAlignment` | Message placement. |
| `right_to_left(b)` | `bool` | `false` | `RightToLeft` | Mirrors columns, flips `Start`/`End`. |
| `on_click(f)` | `FnMut() + 'static` | `None` | `OnClick` | Body-click callback. |
| `on_close(f)` | `FnMut() + 'static` | `None` | `CloseIconClicked` | Close-click callback. |
| `classes(iter)` | `iter of AsRef<str>` | none | `Class` | Stored on `ComponentBase`. |
| `tag(t)` / `tag_ref::<U>()` | `T: Any + Send + Sync` | `None` | `Tag` | Arbitrary user data. |
| `show(ui)` | `&mut Ui` | — | — | Returns `AlertResponse`. |

### `AlertResponse`

| Field / method | Type | Description |
|---|---|---|
| `response` | `Response` | The content row (excludes padding). Deref target. |
| `body_clicked` | `bool` | Body clicked this frame, and not the close button. |
| `close_clicked` | `bool` | Close button clicked this frame. |

`AlertResponse` implements `Deref<Target = Response>`, so `out.hovered()`,
`out.rect` and friends work directly.

### `Severity`

| Variant | `Color` role | Default glyph |
|---|---|---|
| `Normal` (default) | `Color::Default` | `filled::INFO` |
| `Info` | `Color::Info` | `filled::INFO` |
| `Success` | `Color::Success` | `filled::CHECK_CIRCLE` |
| `Warning` | `Color::Warning` | `filled::WARNING` |
| `Error` | `Color::Error` | `filled::ERROR` |

Helpers: `Severity::ALL`, `name()`, `is_neutral()`, `Display`.

`Normal` and `Info` share the `INFO` glyph, which is what MudBlazor renders for
both.

### `HorizontalAlignment`

`Left` (default), `Center`, `Right`, `Start`, `End`. Helpers: `ALL`, `name()`,
`is_logical()`, `Display`. `Start` / `End` flip with `right_to_left`; the three
physical variants do not.

### Constants

| Name | Value |
|---|---|
| `PADDING_X` / `PADDING_Y` | `16.0` / `8.0` |
| `DENSE_PADDING_X` / `DENSE_PADDING_Y` | `12.0` / `4.0` |
| `MIN_ROW_HEIGHT` / `DENSE_MIN_ROW_HEIGHT` | `24.0` / `18.0` |
| `ICON_SIZE` / `ICON_GAP` | `22.0` / `12.0` |
| `CLOSE_GAP` / `CLOSE_HIT` / `CLOSE_GLYPH` | `8.0` / `28.0` / `18.0` |
| `DEFAULT_CORNER_RADIUS` | `4.0` |
| `OUTLINE_STROKE_WIDTH` | `1.0` |
| `TEXT_VARIANT_FILL_ALPHA` | `30` |
| `MAX_ELEVATION` | `24` |

---

## Examples

### One alert per severity

```rust
for (severity, message) in [
    (Severity::Normal,  "The reactor type is RBMK-1000"),
    (Severity::Info,    "The reactor was fired up successfully"),
    (Severity::Success, "Running at optimum temperature"),
    (Severity::Warning, "Temperature exceeds the optimal range"),
    (Severity::Error,   "Meltdown is imminent"),
] {
    MudAlert::new(message).severity(severity).show(ui);
}
```

### Variants

```rust
MudAlert::new("Text").variant(Variant::Text).show(ui);
MudAlert::new("Outlined").severity(Severity::Info).variant(Variant::Outlined).show(ui);
MudAlert::new("Filled").severity(Severity::Error).variant(Variant::Filled).show(ui);
```

### Dismissible banner

```rust
if self.show_alert {
    let out = MudAlert::new("Your session expires in 5 minutes.")
        .severity(Severity::Warning)
        .show_close_icon(true)
        .on_close(|| log::info!("dismissed"))
        .show(ui);

    if out.close_clicked {
        self.show_alert = false;
    }
}
```

### Clickable alert

```rust
let out = MudAlert::new("3 build failures — click to open the log")
    .severity(Severity::Error)
    .on_click(|| self.open_log = true)
    .show(ui);

if out.body_clicked {
    self.open_log = true;
}
```

`on_click` upgrades the alert's `Sense` to `click()`. Without it the alert is
hover-only and never steals clicks from widgets inside a custom `content`.

### Dense alerts in a table toolbar

```rust
MudAlert::new("Read-only view")
    .severity(Severity::Info)
    .dense(true)
    .square(true)
    .elevation(0)
    .show(ui);
```

### Elevated card-style alert

```rust
MudAlert::new("Deployment succeeded")
    .severity(Severity::Success)
    .variant(Variant::Filled)
    .elevation(4)
    .show(ui);
```

### Custom icon and custom close glyph

```rust
MudAlert::new("Backup complete")
    .severity(Severity::Success)
    .icon(filled::CLOUD_DONE)
    .show_close_icon(true)
    .close_icon(filled::CANCEL)
    .show(ui);
```

### Custom content

```rust
MudAlert::new("")
    .severity(Severity::Warning)
    .content(|ui| {
        ui.add(MudText::new("Disk almost full").bold(true));
        ui.add(MudText::new("412 MB remaining").typo(Typo::Caption));
    })
    .show(ui);
```

The closure runs **twice per frame** (measure, then draw) and must be free of
side effects.

### Centred, RTL-aware text

```rust
MudAlert::new("Centred").content_alignment(HorizontalAlignment::Center).show(ui);
MudAlert::new("Start").content_alignment(HorizontalAlignment::Start).show(ui);

// In a right-to-left app: icon on the right, close on the left, Start = right.
MudAlert::new("مرحبا").right_to_left(true).show(ui);
```

### Reacting to the response

```rust
let out = MudAlert::new("Info").severity(Severity::Info).show(ui);
log::debug!(
    "hovered={} body={} close={}",
    out.hovered(),
    out.body_clicked,
    out.close_clicked,
);
```

---

## Layout and rendering

The alert is one painted rectangle. Inside its padding sit three columns:

```
LTR:   [ pad_x ][ icon 22 ][ gap 12 ][ message …… ][ gap 8 ][ close 28 ][ pad_x ]
RTL:   [ pad_x ][ close 28 ][ gap 8 ][ message …… ][ gap 12 ][ icon 22 ][ pad_x ]
```

- **Row height** = `max(measured_message, icon 22, close 28, min_row 24|18)`,
  then the frame adds `2 × pad_y`.
- **Message width** = available width − icon column − close column, floored at
  zero. Columns are always accounted for, even when the message is empty.
- **Icon and close are top-aligned** inside the row. Because the row floor is
  close to the icon height, a single-line alert looks centred; a wrapped message
  keeps its icon at the top, as MudBlazor does.
- **Close glyph is centred** in a `28 × 28` hit area, so the clickable target is
  larger than the drawn glyph.
- **The message child `Ui`** is a `Layout::top_down(align)` with `max_rect` set
  to the message column, so `MudText::inline(true)` sizes to content and the
  parent's cross-axis alignment positions it.

### Visual model

| Variant | Fill | Border | Text / icon |
|---|---|---|---|
| `Text` (default) | accent at 12 % alpha (`TEXT_VARIANT_FILL_ALPHA = 30`) | none | accent |
| `Outlined` | transparent | 1 pt accent | accent |
| `Filled` | accent | none | black or white by luminance |

`Filled` picks its ink with the same `r + g + b > 384` rule as `MudButton`, so
the two components agree on what counts as a "light" fill.

### Interaction

| Registered callback | Alert `Sense` | Children inside `content` receive clicks? |
|---|---|---|
| none | `hover()` | yes |
| `on_click` | `click()` | **no** — the alert body is the click target |

The close button is registered *after* the row, so it wins hit-testing and
`body_clicked` is suppressed on the same frame (`body_clicked = row.clicked() &&
!close_clicked`).

---

## Performance and memory choices

| Choice | Effect |
|---|---|
| Theme fetched once per frame, one `Color::resolve` call | No repeated palette walks; the `Default`/`Inherit` path never needs a second lookup. |
| `&str` fed to `MudText::new` | The message is not cloned into the alert struct; `MudText` allocates once per pass, like any `MudText`. |
| Pure helpers on `Copy` data | No heap, no locking; all geometry is testable without a `Context`. |
| `bool::then` for optional rects and glyphs | No `Vec`, no `Option<Option<…>>`, no intermediate allocations. |
| Message measured in an invisible probe | Exact row height, so no clipping and no stale one-frame height. |
| No `egui::Memory` | Nothing to invalidate, and no chance of the re-entrant-`RwLock` deadlock documented for `MudToggleGroup`. |
| Single `circle_filled` for close hover | No extra shape or `Frame` allocation. |
| Elevation clamped to 24 | Prevents `i8` overflow in `Shadow::offset`. |

Not done, on purpose: caching the resolved `AlertStyle` or the measured message
height. Both depend on the theme and on content that can change every frame, so
a cache would need invalidation on every call.

---

## Testing

`cargo test` runs 27 pure-function tests in `mud_alert.rs`, plus 9 in the two new
enums:

| Area | Tests |
|---|---|
| Severity mapping | colour role for all five, non-empty glyphs, `Normal` shares `Info`'s glyph, custom icon wins |
| Colour | `contrast_text` on light/dark, the exact `384` boundary, `with_alpha` preserving RGB |
| Variants | `Text` tinted + no stroke, `Outlined` transparent + accent stroke, `Filled` accent + contrast ink |
| Metrics | padding and min row height per density, corner radius, `radius_to_u8` clamping, `finite_or` |
| Alignment | physical `Left`/`Center`/`Right` unchanged by RTL, logical `Start`/`End` flipped |
| Columns | width truth table, message width never negative, row-height floors, LTR order, RTL mirror, absent columns, exact tiling for all 8 combinations, close glyph centred |
| Elevation | no-op at zero, clamped at 24, monotonic |
| Builders | defaults match MudBlazor, every field set, tag round-trip |

```bash
cargo test -p mud_egui_components mud_alert
cargo test -p mud_egui_core severity
cargo test -p mud_egui_core horizontal_alignment
```

`MudAlert::show` needs a live `egui::Context` (theme lookup, layout, interaction,
painting) and is therefore not unit-tested. Everything it delegates to is: the
integration test you would write is "click the close glyph → `close_clicked`",
"click the body with `on_click` set → `body_clicked`", and "hovering the padding
does not report hover on `response`".

---

## Differences from MudBlazor

- **Events are returned data.** `OnClick` and `CloseIconClicked` become
  `AlertResponse::body_clicked` / `close_clicked`, the way egui reports
  interaction. `on_click` / `on_close` builders are available as an
  alternative; both fire in the same frame.
- **`Class` is stored but not resolved.** `Variant` and `Severity` already own
  the fill, border and radius; resolving a second utility-class frame would
  composite two fills. `classes(..)` is kept for parity and for future use.
- **`Style` is not ported.** egui has no inline CSS strings.
- **`UserAttributes` is not ported.** There is no HTML tag.
- **`response` is the content row, not the whole painted rect.** It excludes the
  padding, so `hovered()` is only true over the content. This keeps `.clicked()`
  meaningful on the rect that actually senses clicks.
- **`Severity::Normal` shares `INFO` with `Severity::Info`.** That is MudBlazor's
  behaviour, not an oversight.
- **Accent fallback.** If `Color::Default` (used by `Severity::Normal`) cannot be
  resolved from the active palette, the alert falls back to
  `ui.visuals().text_color()` — a neutral ink that reads correctly in both light
  and dark themes.
- **Geometry constants are estimates.** `PADDING_X/Y`, the dense variants,
  `ICON_SIZE`, `ICON_GAP` and the close hit area were chosen to match MudBlazor's
  rendered proportions rather than copied from `_alert.scss`. They are named
  constants at the top of the file; adjust them if something looks off.
- **Message typography is `Typo::Body2`.** MudBlazor's alert message inherits the
  component's body typography through CSS; egui has no cascade, so the scale is
  picked explicitly.
- **The close button does not remove the alert.** MudBlazor's `CloseIconClicked`
  is a notification too — the caller decides whether the alert disappears.

===== ./crates/mud_egui_components/src/feedback/dialog/README.md =====

# MudDialog for `mud_egui`

A modal overlay that presents information, a choice, or input, for
[egui](https://github.com/emilk/egui). Ported from MudBlazor's
[`MudDialog`](https://mudblazor.com/components/dialog) (MudBlazor 8.13.0).

```rust
let mut dialog = DialogState::new("confirm-delete");

if ui.button("Delete").clicked() {
    dialog.open();
}

let resp = MudDialog::new()
    .title("Delete item?")
    .options(DialogOptions::new().with_close_button(true))
    .show_with_actions(
        ui,
        &mut dialog,
        |ui, _ctl| {
            ui.add(MudText::body2("This cannot be undone."));
        },
        |ui, ctl| {
            if ui.add(MudButton::new("Cancel")).clicked() {
                ctl.cancel();
            }
            if ui.add(MudButton::new("Delete").color(Color::Error)).clicked() {
                ctl.close_with(DialogResult::ok_with(item_id));
            }
        },
    );

if let Some(result) = resp.result {
    if result.is_ok() {
        delete(result.data::<u64>().copied());
    }
}
```

The dialog paints only the surface; every string inside is yours to render
through **`MudText`**, every glyph through **`MudIcon`**.

---

## Contents

1. [Design principle: API parity, not API invention](#design-principle-api-parity-not-api-invention)
2. [Architecture](#architecture)
3. [API reference](#api-reference)
4. [Examples](#examples)
5. [Layout and interaction](#layout-and-interaction)
6. [Performance and memory choices](#performance-and-memory-choices)
7. [Testing](#testing)
8. [Differences from MudBlazor](#differences-from-mudblazor)

---

## Design principle: API parity, not API invention

The MudBlazor 8.13.0 parameter surface was verified against DNDocs before
implementation. `MudDialog` has **eleven own parameters**, and the supporting
`DialogOptions` / `DialogResult` / `IMudDialogInstance` contracts are covered
too.

### `MudDialog` parameters

| MudBlazor | Rust mapping | Default |
|---|---|---|
| `ActionsClass` | `actions_class(impl Into<String>)` — **stored**, not resolved | `None` |
| `ContentClass` | `content_class(impl Into<String>)` — **stored** | `None` |
| `ContentStyle` | `content_style(impl Into<String>)` — **stored** | `None` |
| `DefaultFocus` | `default_focus(DefaultFocus)` | `Element` |
| `DialogActions` | `actions` argument of `show_with_actions` | none |
| `DialogContent` | `content` argument of `show` / `show_with_actions` | required |
| `Gutters` | `gutters(bool)` | `true` |
| `IsVisible` | `DialogState::open` | `false` |
| `IsVisibleChanged` | `DialogResponse::opened` / `closed` | — |
| `OnBackdropClick` | `on_backdrop_click(f)` | `None` |
| `OnKeyDown` / `OnKeyUp` | **deferred** — browser `KeyboardEventArgs` | — |
| `Options` | `options(DialogOptions)` / `DialogState::options` | `DialogOptions::default()` |
| `Title` | `title(impl Into<String>)` / `DialogState::title` | `None` |
| `TitleClass` | `title_class(impl Into<String>)` — **stored** | `None` |
| `TitleContent` | `title_content(FnMut(&mut Ui))` | `None` |
| `Class` | `classes(iter)` — **stored** | `None` |
| `Style` / `Tag` / `UserAttributes` | **not ported** | — |

### `DialogOptions` parameters

| MudBlazor | Rust field | Default |
|---|---|---|
| `BackdropClick` | `backdrop_click` | `true` |
| `BackgroundClass` | `background_class` — **stored** | `None` |
| `CloseButton` | `close_button` | `false` |
| `CloseOnEscapeKey` | `close_on_escape_key` | `true` |
| `FullScreen` | `full_screen` | `false` |
| `FullWidth` | `full_width` | `false` |
| `MaxWidth` | `max_width: Option<MaxWidth>` | `None` |
| `NoHeader` | `no_header` | `false` |
| `Position` | `position: Option<DialogPosition>` | `None` → `Center` |

### `DialogResult` surface

| MudBlazor | Rust |
|---|---|
| `Ok()` | `DialogResult::ok()` |
| `Ok<T>(T)` | `DialogResult::ok_with(data)` |
| `Cancel()` | `DialogResult::cancel()` |
| `.Canceled` | `is_canceled()` |
| `.Data` | `data::<T>()` / `take_data::<T>()` |

### `IMudDialogInstance` surface

| MudBlazor | Rust |
|---|---|
| `Close()` | `DialogControl::close()` |
| `Cancel()` | `DialogControl::cancel()` |
| `Close(DialogResult)` | `DialogControl::close_with(result)` |
| `Close<T>(T)` | `DialogControl::close_with(DialogResult::ok_with(t))` |
| `SetTitleAsync(string?)` | `DialogControl::set_title(Option<String>)` |
| `SetOptionsAsync(DialogOptions)` | `DialogControl::set_options(DialogOptions)` |
| `CancelAll()` | **not ported** — no parent-dialog stack |

### `IDialogService` surface

| MudBlazor | Rust |
|---|---|
| `ShowAsync<T>()` | `DialogState::open()` + `MudDialog::show` |
| `Close(IDialogReference)` | `DialogState::close()` |
| `Close(IDialogReference, DialogResult)` | `DialogState::close_with(result)` |
| `GetReturnValueAsync<T>()` | `DialogResponse::result` / `DialogState::take_result()` |

### Deliberately not ported

| MudBlazor | Reason |
|---|---|
| `Style` / `UserAttributes` | No inline CSS strings, no HTML tag. |
| `ActionsClass` / `ContentClass` / `ContentStyle` / `TitleClass` / `BackgroundClass` resolution | Stored for parity. The panel owns fill, radius and shadow; resolving a second utility frame would composite a second surface. |
| `OnKeyDown` / `OnKeyUp` | Browser `KeyboardEventArgs` have no egui equivalent. `Escape` is honoured through `DialogOptions::close_on_escape_key`. |
| `CancelAll()` | Needs a parent-dialog stack; a caller-owned state has no such stack. |
| `FocusTrap` / focus cycling | egui has no built-in focus trap. `DefaultFocus` requests focus on the opening frame; Tab cycling is the host app's responsibility. |

### Enums introduced

Two public enums, each in its own file (Rule 2):

| Enum | File | Discriminants |
|---|---|---|
| `DialogPosition` | `mud_egui_core::enums::dialog_position` | `Center = 0` … `Custom = 9` |
| `DefaultFocus` | `mud_egui_core::enums::default_focus` | `None = 0`, `Element = 1`, `FirstChild = 2`, `LastChild = 3` |

`MaxWidth` is reused from `mud_egui_core::enums`.

---

## Architecture

```
feedback/dialog/
├── dialog_result.rs   DialogResult (own file, Rule 2 companion)
├── dialog_options.rs  DialogOptions
├── dialog_state.rs    DialogState (caller-owned)
├── dialog_control.rs  DialogControl (the IMudDialogInstance analogue)
├── mud_dialog.rs      MudDialog + DialogResponse + pure helpers + tests
└── mod.rs

mud_egui_core/src/enums/
├── default_focus.rs    DefaultFocus
└── dialog_position.rs  DialogPosition
```

**Pure layer.** `position_origin`, `resolve_panel_size`, `dialog_max_width_px`,
`elevation_shadow` are free functions over `Rect`, `Vec2` and `DialogOptions`.
`DialogPosition::fractions`, `DefaultFocus::requests_focus`,
`DialogOptions::effective_position` and every `DialogResult` accessor are pure
methods. None touch `egui::Context`.

**State layer.** `DialogState` holds the open flag, the title/options
overrides, the closing result and a stable `Id`. It is the egui equivalent of
`IDialogService`'s bookkeeping plus the inline `IsVisible` binding.

**Control layer.** `DialogControl` is the `IMudDialogInstance` analogue. The
content and actions closures receive `&mut DialogControl` and call `close` /
`cancel` / `close_with` / `set_title` / `set_options`. Requests are deferred and
applied after the panel is drawn, so the closures never borrow `DialogState`.

**Component layer.** `show` paints the backdrop at `Order::Middle`, positions
and paints the panel at `Order::Foreground`, runs the title / content /
actions, caches the measured size, and applies the deferred requests.

---

## API reference

### `MudDialog<'a>`

| Method | Type | Default | Description |
|---|---|---|---|
| `new()` | — | — | Default dialog. |
| `title(s)` | `impl Into<String>` | `None` | Header text. |
| `title_content(f)` | `FnMut(&mut Ui)` | `None` | Custom header body; overrides `title`. |
| `default_focus(f)` | `DefaultFocus` | `Element` | Which element claims focus on open. |
| `options(o)` | `DialogOptions` | `DialogOptions::default()` | Initial options. |
| `gutters(b)` | `bool` | `true` | 24 pt side padding. |
| `actions_class(s)` / `content_class(s)` / `content_style(s)` / `title_class(s)` | `impl Into<String>` | `None` | **Stored**, not resolved. |
| `color(c)` | `Color` | `Default` | Accent; the panel inherits the theme surface. |
| `on_backdrop_click(f)` | `FnMut()` | `None` | Replaces the default backdrop-close. |
| `classes(iter)` | `iter of AsRef<str>` | none | **Stored**. |
| `show(ui, state, content)` | — | — | Content only; no actions row. |
| `show_with_actions(ui, state, content, actions)` | — | — | Content + actions row. |

`content` is `FnOnce(&mut Ui, &mut DialogControl) -> R`; `actions` is
`FnOnce(&mut Ui, &mut DialogControl)`.

### `DialogResponse<R>`

| Field | Type | Description |
|---|---|---|
| `response` | `Response` | The panel's response; zero-sized when closed. |
| `result` | `Option<DialogResult>` | The closing frame's result. `Some` only on the closing frame. |
| `opened` | `bool` | `true` on the opening frame. |
| `closed` | `bool` | `true` on the closing frame. |
| `content` | `Option<R>` | The content closure's value. |

### `DialogState`

| Field / method | Description |
|---|---|
| `open: bool` | Whether the dialog is open. |
| `id: Id` | Stable identity. |
| `title: Option<String>` | Title override (wins over the builder's). |
| `options: DialogOptions` | Options override. |
| `result: Option<DialogResult>` | The closing result. |
| `new(id_salt)` / `opened(id_salt)` | Constructors. |
| `is_open()` / `open()` / `open_with(title, options)` | Open. |
| `close()` / `cancel()` / `close_with(result)` | Close. |
| `set_title(..)` / `set_options(..)` | Overrides. |
| `take_result()` | Drain the result. |

### `DialogControl`

| Method | Description |
|---|---|
| `close()` | `DialogResult::ok()`. |
| `cancel()` | `DialogResult::cancel()`. |
| `close_with(result)` | Custom result. |
| `is_close_requested()` | Whether a close is already pending. |
| `set_title(Option<String>)` / `set_options(DialogOptions)` | Overrides. |
| `options` / `title` (fields) | The values in force this frame. |

### `DialogOptions`

| Field / method | Description |
|---|---|
| `backdrop_click` / `close_button` / `close_on_escape_key` | Booleans. |
| `full_screen` / `full_width` / `no_header` | Layout switches. |
| `max_width: Option<MaxWidth>` | Width cap. |
| `position: Option<DialogPosition>` | Where it sits. |
| `background_class: Option<String>` | Stored. |
| `new()` / `default()` | MudBlazor defaults. |
| `with_*(..)` | Chainable setters. |
| `effective_position()` | `position` or `Center`. |

### `DialogResult`

| Method | Description |
|---|---|
| `ok()` / `ok_with(data)` / `cancel()` | Constructors. |
| `is_ok()` / `is_canceled()` / `has_data()` | Predicates. |
| `data::<T>()` / `take_data::<T>()` | Payload access. |
| `Default` | `cancel()`. |

### `DialogPosition` (`mud_egui_core::enums`)

| Variant | Discriminant | Fractions |
|---|---|---|
| `Center` | `0` | `(0.5, 0.5)` |
| `CenterLeft` | `1` | `(0.0, 0.5)` |
| `CenterRight` | `2` | `(1.0, 0.5)` |
| `TopCenter` | `3` | `(0.5, 0.0)` |
| `TopLeft` | `4` | `(0.0, 0.0)` |
| `TopRight` | `5` | `(1.0, 0.0)` |
| `BottomCenter` | `6` | `(0.5, 1.0)` |
| `BottomLeft` | `7` | `(0.0, 1.0)` |
| `BottomRight` | `8` | `(1.0, 1.0)` |
| `Custom` | `9` | `(0.0, 0.0)` |

### `DefaultFocus` (`mud_egui_core::enums`)

| Variant | Discriminant | Requests focus |
|---|---|---|
| `None` | `0` | no |
| `Element` | `1` | yes |
| `FirstChild` | `2` | yes |
| `LastChild` | `3` | yes |

### Constants

| Name | Value |
|---|---|
| `PADDING_X` | `24.0` |
| `PADDING_BOTTOM` | `20.0` |
| `TITLE_ROW_HEIGHT` | `48.0` |
| `MIN_DIALOG_WIDTH` | `280.0` |
| `MIN_DIALOG_HEIGHT` | `120.0` |
| `MAX_SCREEN_FRACTION` | `0.9` |

---

## Examples

### Basic confirmation dialog

```rust
let mut dialog = DialogState::new("confirm");
if ui.button("Delete").clicked() { dialog.open(); }

let resp = MudDialog::new()
    .title("Delete item?")
    .options(DialogOptions::new().with_close_button(true))
    .show_with_actions(ui, &mut dialog,
        |ui, _| { ui.add(MudText::body2("This cannot be undone.")); },
        |ui, ctl| {
            if ui.add(MudButton::new("Cancel")).clicked() { ctl.cancel(); }
            if ui.add(MudButton::new("Delete")).clicked() { ctl.close(); }
        });

if resp.closed {
    log::info!("canceled = {}", resp.result.as_ref().is_some_and(|r| r.is_canceled()));
}
```

### Returning a value

```rust
let resp = MudDialog::new().title("Rename").show_with_actions(ui, &mut dialog,
    |ui, ctl| {
        ui.add(MudTextField::new(&mut buffer, &mut touched).label("New name"));
        // `ctl` is available if the content needs to close.
        let _ = ctl;
    },
    |ui, ctl| {
        if ui.add(MudButton::new("Save")).clicked() {
            ctl.close_with(DialogResult::ok_with(buffer.clone()));
        }
    });

if let Some(result) = resp.result {
    if let Some(name) = result.data::<String>() {
        rename(name.clone());
    }
}
```

### Full-width dialog capped at `Md`

```rust
MudDialog::new()
    .title("Settings")
    .options(DialogOptions::new()
        .with_full_width(true)
        .with_max_width(MaxWidth::Md)
        .with_close_button(true))
    .show(ui, &mut dialog, |ui, _| { /* … */ });
```

### Full screen

```rust
MudDialog::new()
    .options(DialogOptions::new().with_full_screen(true))
    .show(ui, &mut dialog, |ui, _| { /* … */ });
```

### Top-centered, no header

```rust
MudDialog::new()
    .options(DialogOptions::new()
        .with_position(DialogPosition::TopCenter)
        .with_no_header(true))
    .show(ui, &mut dialog, |ui, _| { /* … */ });
```

### Backdrop click fires a callback instead of closing

```rust
MudDialog::new()
    .on_backdrop_click(|| log::info!("clicked outside"))
    .options(DialogOptions::new().with_backdrop_click(true))
    .show(ui, &mut dialog, |ui, _| { /* … */ });
```

### Programmatic open / close

```rust
dialog.open();                 // IsVisible = true
dialog.close();                // Close(DialogResult.Ok())
dialog.cancel();               // Close(DialogResult.Cancel())
dialog.close_with(DialogResult::ok_with(42u32));
dialog.set_title(Some("Renamed".into()));   // SetTitleAsync
dialog.set_options(DialogOptions::new().with_close_button(true)); // SetOptionsAsync
```

### Reading the result later

```rust
// If you closed from outside `show`:
if let Some(result) = dialog.take_result() {
    log::info!("closed: {result:?}");
}
```

### Title as a custom body

```rust
MudDialog::new()
    .title_content(|ui| {
        ui.horizontal(|ui| {
            ui.add(MudIcon::new(filled::WARNING).color(Color::Warning));
            ui.add(MudText::h6("Careful"));
        });
    })
    .show(ui, &mut dialog, |ui, _| { /* … */ });
```

---

## Layout and interaction

### Anatomy

```
┌───────────────────────────────────────┐  ← backdrop (Order::Middle, 96 α black)
│  ┌─────────────────────────────────┐  │  ← panel (Order::Foreground)
│  │ [ title ]                 [ ✕ ] │  │     TITLE_ROW_HEIGHT = 48
│  ├─────────────────────────────────┤  │
│  │ content                         │  │     PADDING_X = 24 (gutters)
│  │                                 │  │
│  ├─────────────────────────────────┤  │
│  │                     [ actions ] │  │     PADDING_BOTTOM = 20
│  └─────────────────────────────────┘  │
└───────────────────────────────────────┘
```

### Sizing rules

| Condition | Width | Height |
|---|---|---|
| `full_screen` | screen width | screen height |
| `full_width` | `min(90 % screen, max_width_px)` | measured, clamped |
| default | `clamp(measured, MIN_DIALOG_WIDTH, min(90 % screen, max_width_px))` | `clamp(measured, MIN_DIALOG_HEIGHT, 90 % screen)` |

The panel is positioned inside a `screen.shrink(24)` rect using
`DialogPosition::fractions()`.

### Interaction

| Input | Effect |
|---|---|
| Click outside the panel, `backdrop_click(true)`, no `on_backdrop_click` | `Cancel` |
| Click outside the panel, `backdrop_click(true)`, `on_backdrop_click` set | The callback fires; the dialog stays open |
| Click the ✕, `close_button(true)` | `Cancel` |
| `Escape`, `close_on_escape_key(true)` | `Cancel` |
| Content calls `ctl.close()` / `ctl.cancel()` / `ctl.close_with(..)` | Applied after the panel is drawn |
| Content calls `ctl.set_title(..)` / `ctl.set_options(..)` | Applied after the panel is drawn |

### Focus

`DefaultFocus::{Element, FirstChild, LastChild}` request focus on the opening
frame via `Response::request_focus()`. `None` skips it. egui has no focus
trap, so Tab cycling inside the dialog is the host application's job.

### Size cache

The panel is positioned from the *previous* frame's measured size (or
`DEFAULT_DIALOG_SIZE` on the first open frame), then the measured size is
written back. This is the same one-frame-lag pattern `MudPopover` uses; it is
invisible in practice because a dialog is opened by a click and settles within
a frame or two.

---

## Performance and memory choices

| Choice | Effect |
|---|---|
| `DialogState` is the only persistent allocation | One `Id`, one `Option<String>`, one `DialogOptions`. |
| `DialogOptions` is `Clone`, not `Copy` | It owns `Option<String>` for the class names; cloning it per frame is four pointer copies. |
| Actions closure is boxed **only when present** | `show` allocates nothing for the actions row; `show_with_actions` allocates one `Box`. |
| Size cache in `egui::Memory` | Two `get_temp` / `insert_temp` per frame; no caller-side bookkeeping. |
| Backdrop is one `rect_filled` | No shape tree, no `Frame`. |
| Panel is one `egui::Frame` | One fill, one corner radius, one shadow. |
| Deferred control requests | `DialogState` is never borrowed while the panel is drawn, so the content can call `close` / `set_title` freely. |
| `MudText` / `MudIcon` reuse | Theme and typography propagate; the icon cache absorbs repeated ✕ glyphs. |
| Pure geometry helpers | Position and size resolution are arithmetic; testable without a `Context`. |

---

## Testing

`cargo test -p mud_egui_components mud_dialog` runs the module's tests:

| Area | Tests |
|---|---|
| `DialogPosition` | default, all ten discriminants, every fraction pair, unique names, `Display` |
| `DefaultFocus` | default, discriminants, `requests_focus`, unique names, `Display` |
| `DialogResult` | `ok` / `cancel` / `ok_with`, `has_data`, `data::<T>` / `take_data::<T>`, wrong type, `Debug` hides payload, `Default` is `Cancel` |
| `DialogOptions` | defaults, every `with_*` setter, `effective_position`, clone independence |
| `DialogControl` | no initial requests, `close` / `cancel` / `close_with`, `set_title` / `set_options`, draining, last-close-wins |
| `DialogState` | construction, `open` requests focus once, `open_with`, `close` / `cancel` / `close_with`, `take_result`, `open` clears a stale result, overrides, stable id |
| `position_origin` | center, all four corners, all four edges, `Custom`, oversized clamp |
| `resolve_panel_size` | natural fit, tiny clamp-up, `max_width` cap, `full_width`, `full_width` + `max_width`, `full_screen`, height cap, zero-size screen |
| `dialog_max_width_px` | every bucket |
| `elevation_shadow` | zero, clamp above 24 |
| Builders | defaults, every field set |

Rendering needs a live `egui::Context`; the pure layer is the source of truth.

```bash
cargo test -p mud_egui_core dialog_position default_focus
cargo test -p mud_egui_components dialog_result
cargo test -p mud_egui_components dialog_options
cargo test -p mud_egui_components dialog_state
cargo test -p mud_egui_components dialog_control
cargo test -p mud_egui_components mud_dialog
```

---

## Differences from MudBlazor

- **`IDialogService.ShowAsync<T>` → `DialogState` + `MudDialog::show`.** egui
  has no reflection and no per-frame async runtime, so a dialog is a widget
  whose state the caller owns — the same model `MudDatePicker`,
  `MudColorPicker` and every other component in this workspace use.
- **`IMudDialogInstance` → `&mut DialogControl`.** The content and actions
  closures receive a handle with `close` / `cancel` / `close_with` /
  `set_title` / `set_options`. Requests are deferred and applied after the
  panel is drawn, so the closures never borrow `DialogState`.
- **`DialogResult.Data` is `Box<dyn Any + Send + Sync>`.** `Ok<T>(T)` maps to
  `ok_with(data)`; read it back with `data::<T>()` or `take_data::<T>()`.
  `DialogResult` is therefore not `PartialEq`.
- **`DialogOptions` is a `Clone` struct, not a `Copy` one.** It owns an
  `Option<String>` for `background_class`; cloning it per frame is four
  pointer copies.
- **Class strings are stored, not resolved.** `ActionsClass`, `ContentClass`,
  `ContentStyle`, `TitleClass` and `BackgroundClass` are CSS strings with no
  egui equivalent. The panel owns fill, corner radius and shadow; resolving a
  second utility frame would composite a second surface.
- **`OnKeyDown` / `OnKeyUp` are deferred.** Browser `KeyboardEventArgs` have
  no egui equivalent. `Escape` is honoured through
  `DialogOptions::close_on_escape_key`.
- **`CancelAll()` is not ported.** It needs a parent-dialog stack; a
  caller-owned state has no such stack.
- **No focus trap.** `DefaultFocus` requests focus on the opening frame;
  Tab cycling inside the dialog is the host application's job.
- **`Style` / `Tag` / `UserAttributes` are not ported.** No CSS strings, no
  HTML tag.
- **The panel size is cached for one frame.** The first open frame positions
  from `DEFAULT_DIALOG_SIZE` (560 × 320) and the second from the measured
  size. Same one-frame lag `MudPopover` documents; invisible in practice.
- **The backdrop is always dimmed.** MudBlazor's `BackdropClick` only controls
  dismissal; there is no parameter to remove the dim. The port matches at
  96 α.

===== ./crates/mud_egui_components/src/feedback/menu/README.md =====

# MudMenu for `mud_egui`

A click-to-open dropdown menu for [egui](https://github.com/emilk/egui),
ported from MudBlazor's `MudMenu` / `MudMenuItem` / `MudMenuSeparator`
(MudBlazor 8.13.0). Sits on top of [`MudPopover`](../popover/README.md) so
positioning, flip, backdrop, and dismissal are shared with `MudSelect`,
`MudAutocomplete`, and all pickers.

```rust
let mut menu_state = MenuState::new("file-menu");

let (resp, _) = MudMenu::new()
    .dense(true)
    .show(
        ui,
        &mut menu_state,
        Some(nav),
        |ui| ui.add(egui::Button::new("File")).into(),
        |m| {
            m.text_item("New");
            m.text_item("Open…");
            m.separator();
            m.link_item("Recent", "/recent");
            m.link_item("Exit", "/exit");
        },
    );
```

The activator is whatever you draw — `MudMenu` does not impose a button
style, so you can use `MudButton`, an icon button, plain text, or any
widget. The only requirement is that its closure returns a `Response`.

Item text renders through **`MudText`**, icons through **`MudIcon`**.

---

## Contents

1. [Design principle: API parity, not API invention](#design-principle-api-parity-not-api-invention)
2. [Architecture](#architecture)
3. [API reference — MudMenu](#api-reference--mudmenu)
4. [API reference — MenuItem](#api-reference--menuitem)
5. [API reference — MenuState](#api-reference--menustate)
6. [Examples](#examples)
7. [Click semantics](#click-semantics)
8. [Performance and memory choices](#performance--memory-choices)
9. [Testing](#testing)
10. [Differences from MudBlazor](#differences-from-mudblazor)

---

## Design principle: API parity, not API invention

The MudBlazor 8.13.0 `MudMenu` / `MudMenuItem` / `MudMenuSeparator` surfaces
were verified against DNDocs before this port.

### Ported `MudMenu` parameters

| MudBlazor | Rust method | Default |
|-----------|-------------|---------|
| `ActivatorContent` | `activator` argument of `show` | required |
| `AnchorOrigin` | `anchor_origin(AnchorOrigin)` | `Bottom` |
| `TransformOrigin` | `transform_origin(TransformOrigin)` | `TopLeft` |
| `Position` | `position(Position)` | None (uses anchor/transform) |
| `FullWidth` | `full_width(bool)` | `false` |
| `Icon` | `icon(s)` | `None` |
| `Label` | `label(s)` | `None` |
| `Dense` | `dense(bool)` | `false` |
| `Disabled` | `disabled(bool)` | `false` |
| `MenuClass` | `menu_class(s)` | `None` |
| `MenuListClass` | `menu_list_class(s)` | `None` |
| `MaxHeight` | `max_height(f32)` | `None` |
| `LockScroll` | *(unused — egui Areas don't scroll)* | — |
| `OffsetX` / `OffsetY` | `offset_x(f32)` / `offset_y(f32)` | `0` |
| `RelativeWidth` | `relative_width(bool)` | `false` |
| `OpenOnClick` | `open_on_click(bool)` | `true` |
| `OpenOnHover` | `open_on_hover(bool)` | `false` |
| `CloseOnContentClick` | `close_on_content_click(bool)` | `true` |
| `CloseOnEscapeKey` | `close_on_escape_key(bool)` | `true` |
| `HoverDelay` | `hover_delay_ms(u32)` | `100` |
| `DisableRipple` | `disable_ripple(bool)` | `false` |
| `Class` (base) | `classes(iter)` | `None` |
| `Style` (base) | *(use `classes`)* | — |
| `Tag` (base) | **not ported** (no HTML tag) | — |
| `UserAttributes` | **not ported** | — |

### Ported `MudMenuItem` parameters

| MudBlazor | Rust method | Default |
|-----------|-------------|---------|
| `Text` | `new(s)` / `text(s)` | `None` |
| `Icon` | `icon(s)` | `None` |
| `IconColor` | `icon_color(Color)` | `Inherit` |
| `Href` | `link(text, href)` / `href(s)` | `None` |
| `Target` | `target(s)` | `None` |
| `ForceLoad` | `force_load(bool)` | `false` |
| `Disabled` | `disabled(bool)` | `false` |
| `ChildContent` | `child(f)` | `None` |
| `OnClick` | `on_click(f)` | `None` |
| `Id` | `id(s)` | `None` |

### Extensions (not in MudBlazor)

| Extension | Description |
|-----------|-------------|
| `size_hint(Vec2)` | Popover size guess, hides first-frame lag. |
| `MenuAction` | Rich return value reporting open/close/item activation. |

### Deliberately not ported

- **`LockScroll`** — egui `Area`s do not participate in scroll containers.
- **`DisableRipple`** — the port uses hover highlight, not a ripple.
- **`Tag` / `UserAttributes`** — no HTML tag.
- **`Style`** — no inline CSS strings; use `classes(..)`.

### The `.icon()` / `.label()` fields

MudBlazor's `MudMenu` has `Icon` and `Label` as *fallback activator* values
used when no `ActivatorContent` is supplied. The Rust port **does not
auto-construct** an activator from these, because egui's widget vocabulary
is larger than Blazor's (a `MudButton`, a plain click-sense `Label`, an icon
button — each has different visual weight). Both fields are stored and can
be inspected, but the caller always provides the activator explicitly via
`show`'s closure. This is documented divergence: if you set `.icon(..)` and
`.label(..)` and do not supply a closure that uses them, they are inert.

---

## Architecture

```
overlays/menu/
├── menu_entry.rs    — private MenuEntry enum (Item | Separator)
├── menu_state.rs    — MenuState (wraps PopoverState)
├── menu_action.rs   — MenuAction (extension)
├── menu_item.rs     — MenuItem + MenuSeparator
├── menu_ui.rs       — MenuUi collector
├── mud_menu.rs      — MudMenu + pure helpers + tests
└── mod.rs
```

**Pure layer.** `item_height`, `position_to_origins`, `resolve_icon_color`,
`apply_offsets`, `with_alpha` — free functions, unit-tested without
`egui::Context`.

**State layer.** `MenuState` wraps `PopoverState`. The popover handles open/
close / dismiss; `MudMenu` merely delegates.

**Component layer.** `show` renders the activator, decides whether to
toggle / open the menu, collects the items, then delegates to `MudPopover`
for positioning + backdrop + dismissal.

---

## API reference — `MudMenu`

| Method | Type | Default | Description |
|--------|------|---------|-------------|
| `new()` | — | — | Default menu. |
| `icon(s)` | `impl Into<String>` | `None` | Fallback activator glyph (inert if you supply an activator closure). |
| `label(s)` | `impl Into<String>` | `None` | Fallback activator label (same caveat). |
| `anchor_origin(a)` | `AnchorOrigin` | `None` | Attach edge. |
| `transform_origin(t)` | `TransformOrigin` | `None` | Transform corner. |
| `position(p)` | `Position` | `None` | Coarse placement; overrides anchor/transform. |
| `right_to_left(b)` | `bool` | `false` | RTL flag for `Start` / `End`. |
| `full_width(b)` | `bool` | `false` | Match anchor width (min). |
| `relative_width(b)` | `bool` | `false` | Match anchor width (min; also enables `match_anchor_width`). |
| `max_height(f32)` | `f32` | `None` | Max height for the item list. |
| `size_hint(Vec2)` | `Vec2` | `None` | Popover size guess. |
| `dense(b)` | `bool` | `false` | Compact items (28 px vs 36 px). |
| `disabled(b)` | `bool` | `false` | Disable activator + all items. |
| `open_on_click(b)` | `bool` | `true` | Click activator to toggle. |
| `open_on_hover(b)` | `bool` | `false` | Hover activator to open. |
| `hover_delay_ms(ms)` | `u32` | `100` | Hover delay (see note). |
| `close_on_content_click(b)` | `bool` | `true` | Close on item activation. |
| `close_on_escape_key(b)` | `bool` | `true` | Close on Escape. |
| `disable_ripple(b)` | `bool` | `false` | Stored for parity. |
| `offset_x(px)` | `f32` | `0` | Horizontal offset. |
| `offset_y(px)` | `f32` | `0` | Vertical offset. |
| `menu_class(s)` | `impl Into<String>` | `None` | Paper classes. |
| `menu_list_class(s)` | `impl Into<String>` | `None` | List-container classes. |
| `classes(iter)` | `iter of AsRef<str>` | none | Wrapper classes. |
| `flavor(StyleFlavor)` | `StyleFlavor` | `Bootstrap` | Utility resolver. |
| `show(ui, state, nav, activator, add_items)` | `(&mut Ui, &mut MenuState, Option<&mut NavigationManager>, FnOnce(&mut Ui) -> Response, FnOnce(&mut MenuUi) -> R)` | — | Returns `(MenuResponse, R)`. |

### `MenuResponse`

| Field | Type | Description |
|-------|------|-------------|
| `activator` | `egui::Response` | The activator's response. |
| `action` | `MenuAction` | What happened this frame. |
| `popover` | `Option<PopoverResponse>` | Present when the menu was open. |

---

## API reference — `MenuItem`

| Method | Type | Default | Description |
|--------|------|---------|-------------|
| `new(text)` | `impl Into<String>` | — | Text-only item. |
| `link(text, href)` | `(impl Into<String>, impl Into<String>)` | — | Link item. |
| `text(s)` | `impl Into<String>` | `None` | Replace caption. |
| `icon(s)` | `impl Into<String>` | `None` | Leading icon. |
| `icon_color(Color)` | `Color` | `Inherit` | Icon colour override. |
| `href(s)` | `impl Into<String>` | `None` | Navigation URL. |
| `target(s)` | `impl Into<String>` | `None` | `_blank` opens new tab. |
| `force_load(b)` | `bool` | `false` | Open in browser via absolute URL. |
| `disabled(b)` | `bool` | `false` | Disable this item. |
| `child(f)` | `FnMut(&mut Ui)` | `None` | Custom content replacing text + icon. |
| `on_click(f)` | `FnMut()` | `None` | Click callback. |
| `id(s)` | `impl Into<String>` | `None` | Stable id. |

### `MenuSeparator`

`MenuSeparator::new()` or the unit value `MenuSeparator` — both construct an
empty separator. Add with `MenuUi::separator()`.

---

## API reference — `MenuState`

| Field / method | Description |
|----------------|-------------|
| `popover: PopoverState` | Underlying popover state. |
| `new(id_salt)` | Closed menu with a stable id. |
| `is_open()` | Whether the menu is open. |
| `open()` / `close()` / `toggle()` | Mutators. |

---

## Examples

### Basic dropdown

```rust
let mut menu = MenuState::new("basic");

let (_, _) = MudMenu::new().show(
    ui,
    &mut menu,
    Some(nav),
    |ui| ui.add(egui::Button::new("Menu")).into(),
    |m| {
        m.text_item("Item 1");
        m.text_item("Item 2");
        m.text_item("Item 3");
    },
);
```

### With icons and separators

```rust
MudMenu::new().show(
    ui,
    &mut menu,
    Some(nav),
    |ui| ui.add(egui::Button::new("File")).into(),
    |m| {
        m.item(MenuItem::new("New").icon(filled::ADD));
        m.item(MenuItem::new("Open").icon(filled::FOLDER_OPEN));
        m.separator();
        m.item(MenuItem::new("Save").icon(filled::SAVE));
        m.item(MenuItem::new("Exit").icon(filled::EXIT_TO_APP));
    },
);
```

### With links

```rust
MudMenu::new().show(ui, &mut menu, Some(nav), activator, |m| {
    m.link_item("Docs", "/docs");
    m.link_item("GitHub", "https://github.com"); // opens browser (external)
    m.link_item("Profile", "/profile");
});
```

### Dense, full-width

```rust
MudMenu::new()
    .dense(true)
    .full_width(true)
    .show(ui, &mut menu, None, |ui| ui.add(egui::Button::new("Wide")).into(), |m| {
        m.text_item("Matches the trigger width");
    });
```

### Above the anchor

```rust
MudMenu::new()
    .position(Position::Top)
    .show(ui, &mut menu, None, activator, add_items);
```

### Hover to open

```rust
MudMenu::new()
    .open_on_click(false)
    .open_on_hover(true)
    .show(ui, &mut menu, None, activator, add_items);
```

### Custom activator (icon button)

```rust
MudMenu::new().show(
    ui,
    &mut menu,
    None,
    |ui| ui.add(MudIcon::new(filled::MORE_VERT).size(2)),
    |m| { /* items */ },
);
```

### Custom item content

```rust
MudMenu::new().show(ui, &mut menu, None, activator, |m| {
    m.item(
        MenuItem::new("")
            .child(|ui| {
                ui.add(MudIcon::new(filled::SAVE));
                ui.add(MudText::new("Save"));
                ui.add(MudText::new("Ctrl+S").font_size(10.0).color(Color::TextSecondary));
            })
            .on_click(|| save_document()),
    );
});
```

### Scrolling long menu

```rust
MudMenu::new()
    .max_height(300.0)
    .show(ui, &mut menu, None, activator, |m| {
        for i in 0..50 {
            m.text_item(format!("Item {i}"));
        }
    });
```

### Reading the action

```rust
let (resp, _) = MudMenu::new().show(ui, &mut menu, None, activator, add_items);
match resp.action {
    MenuAction::Opened => log::info!("menu opened"),
    MenuAction::Closed => log::info!("menu closed"),
    MenuAction::ItemActivated { label, href } => {
        log::info!("activated {label:?} → {href:?}");
    }
    MenuAction::None => {}
}
```

### First-frame size hint

```rust
MudMenu::new()
    .size_hint(vec2(240.0, 180.0))
    .show(ui, &mut menu, None, activator, add_items);
```

---

## Click semantics

When an item is clicked:

1. Its `on_click` callback fires (if any).
2. If `href` is set:
   - **`force_load(true)`** → the absolute URL is opened in the browser.
   - **External URL** (different scheme or origin) → opened in the browser.
   - **Internal URL** (relative or same-origin) → `nav.navigate_to(href, false)`.
   - **`target == "_blank"`** → `new_tab: true`; every other target → `new_tab: false`.
3. If `close_on_content_click` is `true` (default), the menu closes.

Escape and outside-click close the menu — those are delegated to
`MudPopover` and honour `close_on_escape_key` and the popover's
`dismiss_on_outside_click`.

### Hover delay note

`hover_delay_ms` is **stored** but not honoured in pass 1. egui's
immediate-mode model requires a timer to be checked every frame; the port
opens on the first hover frame. If you need the delay, wire it up via
`egui::Memory` + `ctx.request_repaint_after(..)`. The parameter exists for
API parity.

---

## Performance and memory choices

| Choice | Effect |
|--------|--------|
| `item_height` / `position_to_origins` / `resolve_icon_color` / `apply_offsets` are pure | No `egui::Context`; unit-tested. |
| `MenuState` is `Copy` | Zero-cost snapshot. |
| `MenuUi` collects entries into a temporary `Vec` | Freed at the end of `show`. |
| `take()` on `child` / `on_click` before creating `&item` | No borrow conflict between the child closure and icon/text reads. |
| Delegation to `MudPopover` | Positioning, flip, backdrop, Escape, outside-click all live in one place. |
| Item rows use `allocate_exact_size` + absolute child `Ui`s | No per-item layout pass. |
| `MudIcon` reuse across items | Free thanks to its transform cache. |
| `MudText` for all labels | Theme and typography propagate automatically. |
| `max_height` uses `ScrollArea::vertical` | Long menus are scrollable without manual layout. |

---

## Testing

`cargo test` runs 20+ pure-function tests:

| Area | Tests |
|------|-------|
| `item_height` | normal / dense |
| `position_to_origins` | bottom / top / left / right / start under RTL |
| `resolve_icon_color` | fallback / Default / explicit |
| `with_alpha` | identity / transparent / scaling |
| `apply_offsets` | vertical vs horizontal anchor picks the right offset |
| Defaults | every `MudMenu` field matches MudBlazor |
| Builders | every field set |
| `MenuAction` | every variant's helpers |
| `MenuUi` | preserves order and kind |
| `MenuItem` | constructors, builders |
| `MenuState` | open / close / toggle / stable id |

Rendering (`show`) needs a live `egui::Context`; the pure layer is the
source of truth.

```bash
cargo test -p mud_egui_components mud_menu
```

---

## Differences from MudBlazor

- **`LockScroll` not ported.** egui `Area`s don't participate in scroll
  containers.
- **`DisableRipple` stored but not rendered.** The port uses a hover
  highlight on item rows instead.
- **`.icon()` / `.label()` are inert without an activator closure.**
  MudBlazor auto-builds an activator from these; the port requires the
  caller to pass an explicit activator closure so the visual weight of the
  trigger matches the rest of the app.
- **`hover_delay_ms` stored but not honoured.** egui needs a timer; the port
  opens on the first hover frame. Parameter exists for parity.
- **`OffsetX` / `OffsetY` collapse into a single popover `offset`.** Only
  the offset on the anchor's axis is honoured. This is a `MudPopover`
  limitation documented there.
- **`Tag` / `UserAttributes` not ported.** No HTML tag.
- **`Style` → `classes(..)`.** egui has no inline CSS strings.

===== ./crates/mud_egui_components/src/feedback/popover/README.md =====

# MudPopover for `mud_egui`

The anchored overlay primitive for [egui](https://github.com/emilk/egui),
ported from MudBlazor's `MudPopover` (MudBlazor 8.13.0). This is the
foundation that `MudSelect`, `MudAutocomplete`, the picker family,
`MudMenu`, and `MudTooltip` all sit on top of.

```rust
let anchor_rect = ui
    .add(egui::Button::new("Open menu"))
    .rect;

let (resp, _) = MudPopover::new()
    .anchor_origin(AnchorOrigin::Bottom)
    .transform_origin(TransformOrigin::TopLeft)
    .match_anchor_width(true)
    .show(ui, &mut self.menu_state, anchor_rect, |ui| {
        ui.add(MudText::new("Item 1"));
        ui.add(MudText::new("Item 2"));
    });
```

The popover itself renders no text or icons — the content closure does. It
is expected to use `MudText` / `MudIcon` per project convention.

---

## Contents

1. [Design principle: API parity, not API invention](#design-principle-api-parity-not-api-invention)
2. [Architecture](#architecture)
3. [Positioning model](#positioning-model)
4. [API reference](#api-reference)
5. [Examples](#examples)
6. [Performance and memory choices](#performance--memory-choices)
7. [Testing](#testing)
8. [Differences from MudBlazor](#differences-from-mudblazor)

---

## Design principle: API parity, not API invention

MudBlazor's `MudPopover` parameter surface was verified against DNDocs before
this port. A few MudBlazor parameters have no egui equivalent (see
"Deliberately not ported") and are documented as such.

### Ported parameters

| MudBlazor | Rust method | Default |
|-----------|-------------|---------|
| `AnchorOrigin` | `anchor_origin(AnchorOrigin)` | `Bottom` |
| `TransformOrigin` | `transform_origin(TransformOrigin)` | `TopLeft` |
| `OverflowBehavior` | `overflow_behavior(OverflowBehavior)` | `Flip` |
| `Paper` | `paper(bool)` | `true` |
| `Elevation` | `elevation(u8)` | `8` |
| `Square` | `square(bool)` | `false` |
| `Open` | `PopoverState::open` | `false` |
| `Class` | `classes(iter)` | `None` |
| `Style` | *(use `classes`)* | — |
| `ChildContent` | closure to `show` | required |

### Extensions (not in MudBlazor)

| Extension | Description |
|-----------|-------------|
| `backdrop(BackdropMode)` | Opt-in backdrop. Default `None` (no backdrop, drops clicks). `Transparent` catches clicks. `Dimmed(alpha)` paints a semi-transparent overlay. |
| `dismiss_on_escape(bool)` | Escape dismisses (default `true`). |
| `dismiss_on_outside_click(bool)` | Outside click dismisses (default `true`). |
| `offset(f32)` | Gap between anchor and popover. |
| `min_width(f32)` / `max_width(f32)` | Explicit clamps. |
| `match_anchor_width(bool)` | Sets `min_width = anchor_rect.width()` when the caller has not set it. |
| `right_to_left(bool)` | Resolves `AnchorOrigin::Start` / `End`. |
| `size_hint(Vec2)` | Initial size guess used on the first open frame. |
| `PopoverResponse` | Rich return value with `open` and `response`. |

### Deliberately not ported

- **`Fixed`** — egui's `Area` is always screen-fixed. The parameter would be
  a no-op.
- **`Style`** — egui has no inline CSS strings; use `classes(..)`.
- **`OverflowBehavior::Shrink`** — needs to know the popover size before
  placing it. Deferred to a future pass (see "Size cache" below for the
  current approximation).

### `AnchorOrigin` (renamed from MudBlazor's `Anchor`)

| Variant | Meaning |
|---------|---------|
| `Top` | Popover above the anchor |
| `Bottom` (default) | Popover below the anchor |
| `Left` | Popover to the left |
| `Right` | Popover to the right |
| `Start` | Logical left in LTR, right in RTL |
| `End` | Logical right in LTR, left in RTL |

### `TransformOrigin` (MudBlazor's `Origin`)

Nine values: `TopLeft` (default), `TopCenter`, `TopRight`, `CenterLeft`,
`Center`, `CenterRight`, `BottomLeft`, `BottomCenter`, `BottomRight`.

The pair `AnchorOrigin::Bottom + TransformOrigin::TopLeft` places the
popover's top-left corner at the anchor's bottom-center — the standard
dropdown layout.

### `OverflowBehavior`

| Variant | Behaviour |
|---------|-----------|
| `Flip` (default) | Flip to opposite side when the preferred position overflows; clamp as last resort |
| `Reposition` | Clamp into screen bounds without flipping |
| `Shrink` | Treated as `Flip` in pass 1; caller must set `max_height` manually |

### `BackdropMode`

| Variant | Behaviour |
|---------|-----------|
| `None` (default) | No backdrop; outside clicks still dismiss if `dismiss_on_outside_click` is on, but the parent UI is interactive |
| `Transparent` | Full-screen click-catcher; parent UI blocked, nothing painted |
| `Dimmed(f32)` | Full-screen overlay at `alpha`; parent UI blocked, semi-transparent black painted |

---

## Architecture

```
overlays/popover/
├── popover_state.rs    — PopoverState { open, id }
├── popover_response.rs — PopoverResponse { open, response }
├── mud_popover.rs      — MudPopover + pure geometry + tests
└── mod.rs              — re-exports

mud_egui_core/src/enums/
├── anchor_origin.rs
├── backdrop_mode.rs
├── overflow_behavior.rs
└── transform_origin.rs
```

**Pure geometry.** `anchor_point`, `transform_offset`, `offset_vec`,
`compute_position`, `flip_vertical`, `flip_horizontal`, `resolve_overflow`
are free functions. All positioning is unit-tested without `egui::Context`.

**State layer.** `PopoverState { open: bool, id: Id }` — the egui equivalent
of MudBlazor's `MudPopover.Open` parameter plus a stable id used for the
size cache.

**Component layer.** `show` computes the target position, paints any
backdrop, renders the popover content into a `Foreground`-ordered
`egui::Area`, caches the actual size for the next frame, and applies
dismissal rules.

---

## Positioning model

```
               anchor_rect
          ┌────────────────────┐
          │                    │
          │                    │  ◄── AnchorOrigin::Top    = top-center
          │                    │
          │                    │  ◄── AnchorOrigin::Left   = left-center
          │                    │       AnchorOrigin::Right = right-center
          │                    │
          │                    │  ◄── AnchorOrigin::Bottom = bottom-center
          └────────────────────┘

          transform_origin determines which corner of the popover
          snaps to that anchor point.
```

### The math

```
anchor_point = point on anchor_rect (top-center, bottom-center, etc.)
offset_vec   = small push away from anchor (offset parameter)
transform    = shift by (transform_origin fraction of popover size)
position     = anchor_point + offset_vec + transform
```

### Example: standard dropdown

| Input | Value |
|-------|-------|
| `anchor_rect` | `Rect { min: (100, 200), size: (200, 40) }` (a text field) |
| `anchor_origin` | `Bottom` |
| `transform_origin` | `TopLeft` |
| `offset` | `4.0` |
| popover size | `(200, 300)` |

`anchor_point = (200, 240)` (bottom-center of the field)  
`offset_vec = (0, 4)` (push down 4 px)  
`transform = (0, 0)` (top-left at anchor point)  
**final position = (200, 244)**

### Flip logic

When the initial position overflows the screen (8 px margin), the port
attempts a flip:

1. If the vertical axis overflows → try `flip_vertical(anchor, transform)`.
2. If the horizontal axis overflows → try `flip_horizontal(anchor, transform)`.
3. After flipping, if it still overflows → clamp to `screen - 8 px`.

`OverflowBehavior::Reposition` skips the flip attempt and goes straight to
clamping.

### Size cache — the chicken-and-egg problem

egui is immediate-mode; the port cannot know the popover size before it is
rendered. It cannot know where to render without the size. The port solves
this with a **size cache in `egui::Memory`**, keyed by `PopoverState::id`:

| Frame | What happens |
|-------|--------------|
| **Open (1st frame)** | `read_cached_size` returns `None`. Use `size_hint` (or `DEFAULT_SIZE_HINT = vec2(200, 200)`). Compute position from that hint. Render. Cache the actual size. |
| **Open (2nd frame)** | `read_cached_size` returns the actual size from frame 1. Compute position from that. Render. Re-cache. |
| **Closed** | No rendering, no caching. |

The one-frame lag is invisible in practice (dropdowns are opened by a
click; the popover appears and settles into place within a frame or two).
If you care about the *first* frame being pixel-perfect, set `size_hint`
to the approximate size of the popover content:

```rust
MudPopover::new()
    .size_hint(vec2(280.0, 300.0))
    .show(ui, &mut self.menu_state, anchor_rect, |ui| {
        for item in &self.items { /* … */ }
    });
```

---

## API reference

### `MudPopover<'a>`

| Method | Type | Default | Description |
|--------|------|---------|-------------|
| `new()` | — | — | Default popover. |
| `anchor_origin(AnchorOrigin)` | `AnchorOrigin` | `Bottom` | Which edge of the anchor. |
| `transform_origin(TransformOrigin)` | `TransformOrigin` | `TopLeft` | Which corner of the popover. |
| `overflow_behavior(OverflowBehavior)` | `OverflowBehavior` | `Flip` | Flip / reposition / shrink. |
| `paper(bool)` | `bool` | `true` | Draw paper background. |
| `elevation(u8)` | `u8` | `8` | Drop-shadow depth. |
| `square(bool)` | `bool` | `false` | Removes paper corner rounding. |
| `backdrop(BackdropMode)` | `BackdropMode` | `None` | Backdrop. |
| `dismiss_on_escape(bool)` | `bool` | `true` | Escape dismisses. |
| `dismiss_on_outside_click(bool)` | `bool` | `true` | Outside click dismisses. |
| `offset(f32)` | `f32` | `0.0` | Gap from anchor. |
| `min_width(f32)` | `f32` | `None` | Explicit minimum width. |
| `max_width(f32)` | `f32` | `None` | Explicit maximum width. |
| `match_anchor_width(bool)` | `bool` | `false` | Sets `min_width = anchor_rect.width()`. |
| `right_to_left(bool)` | `bool` | `false` | Resolves `Start` / `End`. |
| `size_hint(Vec2)` | `Vec2` | `None` | Initial size guess. |
| `classes(iter)` | `iter of AsRef<str>` | none | Paper utility classes. |
| `flavor(StyleFlavor)` | `StyleFlavor` | `Bootstrap` | Utility resolver. |
| `show(ui, state, anchor, \|ui\| ..)` | `(&mut Ui, &mut PopoverState, Rect, FnOnce(&mut Ui) -> R)` | — | Returns `(PopoverResponse, Option<R>)`. |

### `PopoverState`

| Field / method | Description |
|----------------|-------------|
| `open: bool` | Whether the popover is open. |
| `id: Id` | Stable identity for the Area and the size cache. |
| `new(id_salt)` | Closed state with a stable id. |
| `open()` / `close()` / `toggle()` | Mutators. |

### `PopoverResponse`

| Field | Type | Description |
|-------|------|-------------|
| `open` | `bool` | Open state **after** this frame's dismissal handling. |
| `response` | `egui::Response` | Popover's response; zero-sized when closed. |
| `is_closed()` | `bool` | Convenience. |

### `OverflowResult`

Return value of `resolve_overflow`:

| Field | Description |
|-------|-------------|
| `pos` | Final top-left position. |
| `anchor` | Final (possibly flipped) anchor edge. |
| `transform` | Final (possibly flipped) transform origin. |

### Free geometry helpers

| Function | Signature |
|----------|-----------|
| `anchor_point` | `(Rect, AnchorOrigin) -> Pos2` |
| `transform_offset` | `(TransformOrigin, Vec2) -> Vec2` |
| `offset_vec` | `(AnchorOrigin, f32) -> Vec2` |
| `compute_position` | `(Rect, AnchorOrigin, TransformOrigin, Vec2, f32) -> Pos2` |
| `flip_vertical` | `(AnchorOrigin, TransformOrigin) -> Option<(AnchorOrigin, TransformOrigin)>` |
| `flip_horizontal` | `(AnchorOrigin, TransformOrigin) -> Option<(AnchorOrigin, TransformOrigin)>` |
| `resolve_overflow` | `(Rect, AnchorOrigin, TransformOrigin, Vec2, f32, Rect, OverflowBehavior) -> OverflowResult` |

---

## Examples

### Standard dropdown (below a text field)

```rust
let resp = MudTextField::new(&mut self.filter)
    .adornment_end(AdornmentEnd::icon(filled::ARROW_DROP_DOWN))
    .readonly(true)
    .show(ui);

if resp.response.clicked() {
    self.popover_state.toggle();
}

MudPopover::new()
    .anchor_origin(AnchorOrigin::Bottom)
    .transform_origin(TransformOrigin::TopLeft)
    .match_anchor_width(true)
    .offset(2.0)
    .show(ui, &mut self.popover_state, resp.response.rect, |ui| {
        // dropdown items
    });
```

### Tooltip (above, centered)

```rust
MudPopover::new()
    .anchor_origin(AnchorOrigin::Top)
    .transform_origin(TransformOrigin::BottomCenter)
    .paper(true)
    .elevation(4)
    .offset(6.0)
    .dismiss_on_outside_click(false)  // tooltips stay open on outside click
    .show(ui, &mut self.tooltip_state, widget_rect, |ui| {
        ui.add(MudText::new("Tooltip text").font_size(12.0));
    });
```

### Submenu (to the right, vertically centered)

```rust
MudPopover::new()
    .anchor_origin(AnchorOrigin::Right)
    .transform_origin(TransformOrigin::CenterLeft)
    .offset(4.0)
    .show(ui, &mut self.submenu_state, parent_item_rect, |ui| {
        // submenu items
    });
```

### Modal-style popover with dimmed backdrop

```rust
MudPopover::new()
    .anchor_origin(AnchorOrigin::Bottom)
    .transform_origin(TransformOrigin::TopCenter)
    .backdrop(BackdropMode::Dimmed(0.4))
    .paper(true)
    .elevation(12)
    .show(ui, &mut self.state, anchor_rect, |ui| {
        // modal content
    });
```

### Popover with explicit width

```rust
MudPopover::new()
    .min_width(280.0)
    .max_width(400.0)
    .show(ui, &mut self.state, anchor_rect, |ui| {
        // content
    });
```

### Manual open/close

```rust
// From anywhere in your app:
self.popover_state.open();
self.popover_state.close();
self.popover_state.toggle();

// Or set directly:
self.popover_state.open = true;
```

### First-frame size hint

```rust
MudPopover::new()
    .size_hint(vec2(280.0, 350.0))
    .show(ui, &mut self.state, anchor_rect, |ui| {
        // sizeable content
    });
```

---

## Performance and memory choices

| Choice | Effect |
|--------|--------|
| Pure geometry helpers | No `egui::Context`; unit-tested. |
| `PopoverState` is `Copy` | Zero-cost snapshot. |
| Size cache in `egui::Memory` | Persistent across frames, keyed by `PopoverState::id`, no caller-side storage needed. |
| Content closure runs **once** per frame | No double-render for measure; the size cache does the job. |
| `Area` at `Order::Foreground` | Renders above parent UI. |
| Backdrop `Area` at `Order::Middle` | Blocks parent, sits below the popover. |
| `content` closure never runs when closed | Idle popovers cost one `allocate_response(Vec2::ZERO, Sense::hover())`. |
| `paper` frame is resolved once per frame | `BsStyle` / `TwStyle` called once. |
| `resolve_overflow` is a single pass | No iterative fitting. |

---

## Testing

`cargo test` runs 30+ pure-function tests:

| Area | Tests |
|------|-------|
| `anchor_point` | Top / Bottom / Left / Right on a known rect |
| `transform_offset` | TopLeft / TopCenter / BottomRight / Center |
| `offset_vec` | Top / Bottom / Left / Right |
| `compute_position` | dropdown below left / centered, tooltip above, submenu right |
| `flip_vertical` | Bottom→Top, Top→Bottom, horizontal anchor returns None, centered transform returns None |
| `flip_horizontal` | Right→Left, vertical anchor returns None |
| `resolve_overflow` | fits, flips above, reposition clamps, horizontal flip, clamp-when-flip-fails |
| `elevation_shadow` | zero, clamped above 24 |
| Defaults | every `MudPopover` field matches MudBlazor |
| Builders | every field set |

Rendering (`show`) needs a live `egui::Context`; the pure layer is the source
of truth.

```bash
cargo test -p mud_egui_components mud_popover
cargo test -p mud_egui_core anchor_origin
cargo test -p mud_egui_core transform_origin
cargo test -p mud_egui_core overflow_behavior
cargo test -p mud_egui_core backdrop_mode
```

---

## Differences from MudBlazor

- **`Anchor` renamed to `AnchorOrigin`.** The popover has both an "anchor
  origin" (which edge of the anchor) and a "transform origin" (which corner
  of the popover). MudBlazor calls the first one `Anchor` — the port uses a
  more descriptive name that makes the relationship obvious at call sites.
- **`Anchor` variants are 6, not 9.** MudBlazor's `Anchor` has
  `Top / Bottom / Left / Right / Start / End` — the horizontal alignment is
  delegated to `TransformOrigin`. The port matches.
- **`Fixed` not ported.** egui's `Area` is always screen-fixed; the
  parameter would be a no-op.
- **`Shrink` overflow behaviour treated as `Flip`.** egui cannot measure the
  popover size before placing it; `Shrink` needs the caller to set
  `max_height` manually. Reserved for a future pass where popovers can
  measure their content cheaply.
- **Size cache replaces MudBlazor's DOM-based measurement.** MudBlazor
  measures the popover's DOM element before positioning it; egui's
  immediate-mode model has no equivalent. The port caches the actual size in
  `egui::Memory` and uses the previous frame's size for positioning — one
  frame of lag, invisible in practice.
- **`BackdropMode` is an extension.** MudBlazor's `MudPopover` has no
  backdrop parameter; dismissal is handled by the host's outside-click
  listener. The port exposes a `BackdropMode` because the same primitive
  will be reused by the dialog host later.
- **`size_hint` is an extension.** Not in MudBlazor (DOM measurement makes it
  unnecessary there). Provided so callers who care about pixel-perfect
  first-frame rendering can pre-announce the content size.
- **`Style` → `classes(..)`.** egui has no inline CSS strings.

===== ./crates/mud_egui_components/src/feedback/progress/mud_progress_circular/README.md =====

# MudProgressCircular for `mud_egui`

A circle-shaped indicator of progress for
[egui](https://github.com/emilk/egui), ported from MudBlazor's
[`MudProgressCircular`](https://mudblazor.com/components/progress) (MudBlazor 8.13.0).

```rust
// Determinate, 72 % complete.
ui.add(
    MudProgressCircular::new()
        .value(72.0)
        .color(Color::Primary),
);

// Indeterminate: a rotating, breathing arc.
ui.add(
    MudProgressCircular::new()
        .indeterminate(true)
        .color(Color::Info)
        .size(Size::Small),
);
```

The ring paints no text of its own unless you supply a `label` closure. A label's
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
implementation. `MudProgressCircular` has **exactly nine parameters**, and all
nine are ported.

### Ported parameters

| MudBlazor | Type | Default | Rust mapping |
|---|---|---|---|
| `ChildContent` | `RenderFragment?` | `null` | `label(FnMut(&mut Ui))` |
| `Color` | `Color` | `Default` | `color(Color)` |
| `Indeterminate` | `bool` | `false` | `indeterminate(bool)` |
| `Max` | `double` | `100.0` | `max(f64)` |
| `Min` | `double` | `0.0` | `min(f64)` |
| `Rounded` | `bool` | `false` | `rounded(bool)` |
| `Size` | `Size` | `Medium` | `size(Size)` |
| `StrokeWidth` | `int` | `3` | `stroke_width(u8)` |
| `Value` | `double` | `0.0` | `value(f64)` |
| `Class` | `string?` | `null` | `classes(iter)` — stored, not resolved |
| `Tag` | `object?` | `null` | `tag(t)` / `tag_ref::<U>()` |

### Deliberately not ported

| MudBlazor | Reason |
|---|---|
| `Style` | egui has no inline CSS strings. |
| `UserAttributes` | There is no HTML tag to splat attributes onto. |
| `Class` resolution | Stored on `base` for parity only. The ring's colour comes from `color`; a utility-class frame would composite a second fill. |

There is **no** `ValueChanged`. The DNDocs page lists `Value` as a plain
`double` parameter, and the component inherits `IMudStateHasChanged` but
exposes no two-way binding. This is a display-only component, and the port does
not invent a callback.

### `ChildContent` is the percentage overlay, not a wrapper

In MudBlazor, `ChildContent` is rendered inside
`.mud-progress-circular-percentage`, an absolutely positioned flex box that
covers the whole circle and centres its content[reference:6]. The Rust mapping
is `label(closure)`: it runs in a child `Ui` sized to the circle and centred
both ways.

---

## Architecture

```
feedback/progress_circular/
└── mud_progress_circular.rs
    ├── constants        ARC_SEGMENTS 64, LOOP_SECONDS 1.4,
    │                    MIN_SWEEP 30°, MAX_SWEEP 288°, TRACK_ALPHA 60,
    │                    MIN_STROKE 1, START_ANGLE −90°
    ├── pure helpers     diameter, effective_stroke_width, fraction,
    │                    indeterminate_phase, rotation_for, sweep_for,
    │                    with_alpha, arc_points
    ├── paint_arc        polyline + optional round caps
    ├── MudProgressCircular  builder + show
    └── impl Widget      ui.add(MudProgressCircular::new())
```

**Pure layer.** The diameter, the stroke clamp, the value fraction, the
indeterminate phase / rotation / sweep and the arc polyline are free functions
over `f32`, `f64` and `Pos2`. None touch `egui::Context`, so they are
unit-tested without a window.

**Geometry layer.** `arc_points` turns `(center, radius, start, sweep)` into a
`Vec<Pos2>` of `ARC_SEGMENTS + 1` points. `paint_arc` submits that polyline as
one `Shape::line` and, when rounded, adds two `circle_filled` caps.

**Component layer.** `show` fetches the theme once, resolves one accent colour,
allocates a square rect, paints the track, the arc and the label, and requests a
repaint only when indeterminate.

---

## API reference

| Method | Type | Default | Mirrors | Description |
|---|---|---|---|---|
| `new()` | `Self` | — | — | Value `0`, range `0..=100`, `Default`, `Medium`, stroke `3`. |
| `color(c)` | `Color` | `Default` | `Color` | Accent of the arc. |
| `size(s)` | `Size` | `Medium` | `Size` | Diameter: 24 / 40 / 56 pt. |
| `stroke_width(w)` | `u8` | `3` | `StrokeWidth` | Ring thickness; clamped to `1..=diameter / 2`. |
| `value(v)` | `f64` | `0.0` | `Value` | Current progress; ignored when indeterminate. |
| `min(v)` | `f64` | `0.0` | `Min` | Lower bound. |
| `max(v)` | `f64` | `100.0` | `Max` | Upper bound. |
| `indeterminate(b)` | `bool` | `false` | `Indeterminate` | Rotating, breathing arc. |
| `rounded(b)` | `bool` | `false` | `Rounded` | Round the arc ends. |
| `label(f)` | `FnMut(&mut Ui) + 'static` | `None` | `ChildContent` | Centred overlay content. |
| `classes(iter)` | `iter of AsRef<str>` | none | `Class` | Stored on `ComponentBase`. |
| `tag(t)` / `tag_ref::<U>()` | `T: Any + Send + Sync` | `None` | `Tag` | Arbitrary user data. |
| `show(ui)` | `&mut Ui` | — | — | Returns the circle's `Response`. |

### `Size` mapping

| `Size` | Diameter |
|---|---|
| `ExtraSmall` | 24 pt |
| `Small` | 24 pt |
| `Medium` (default) | 40 pt |
| `Large` | 56 pt |
| `ExtraLarge` | 56 pt |

`ExtraSmall` and `ExtraLarge` fold into `Small` and `Large`, matching
`.mud-progress-small` and `.mud-progress-large` in MudBlazor's SCSS[reference:7].

### Constants

| Name | Value |
|---|---|
| `ARC_SEGMENTS` | `64` |
| `LOOP_SECONDS` | `1.4` |
| `MIN_SWEEP` | `π / 6` (30°) |
| `MAX_SWEEP` | `1.6π` (288°) |
| `TRACK_ALPHA` | `60` |
| `MIN_STROKE` | `1.0` |

---

## Examples

### Basic determinate circle

```rust
MudProgressCircular::new()
    .value(72.0)
    .color(Color::Primary)
    .show(ui);
```

### With a percentage label

```rust
MudProgressCircular::new()
    .value(72.0)
    .size(Size::Large)
    .label(|ui| {
        ui.add(MudText::body2("72%"));
    })
    .show(ui);
```

The closure runs once per frame in a `Ui` sized to the circle and centred both
ways.

### Indeterminate spinner

```rust
MudProgressCircular::new()
    .indeterminate(true)
    .color(Color::Info)
    .size(Size::Small)
    .show(ui);
```

### Inline spinner next to text

```rust
ui.horizontal(|ui| {
    ui.add(
        MudProgressCircular::new()
            .indeterminate(true)
            .size(Size::Small),
    );
    ui.add(MudText::body1("Loading…"));
});
```

### Rounded arc

```rust
MudProgressCircular::new()
    .value(60.0)
    .rounded(true)
    .stroke_width(6)
    .size(Size::Large)
    .show(ui);
```

`rounded(true)` paints a cap of radius `stroke_width / 2` at each end of the
arc, which is MudBlazor's `stroke-linecap: round`.

### Thick ring

```rust
MudProgressCircular::new()
    .value(80.0)
    .stroke_width(8)
    .size(Size::Large)
    .show(ui);
```

### Custom range

```rust
MudProgressCircular::new()
    .min(0.0)
    .max(1024.0)
    .value(512.0) // half full
    .show(ui);
```

### Zero value (empty ring)

```rust
MudProgressCircular::new().value(0.0).show(ui);
```

The track is still drawn, so the circle is visible; only the accent arc is
absent. MudBlazor behaves the same way.

### Tag round-trip

```rust
let ring = MudProgressCircular::new().value(50.0).tag("spinner");
assert_eq!(ring.tag_ref::<&str>(), Some(&"spinner"));
ring.show(ui);
```

---

## Layout and rendering

### Anatomy

```
        ┌─────────────┐
        │   ╭─────╮   │   ← track (accent at 60 α)
        │  ╱       ╲  │
        │ │         │ │   ← arc (accent, 3 pt by default)
        │  ╲       ╱  │
        │   ╰─────╯   │   ← 12 o'clock start, clockwise
        └─────────────┘
        │   [label]   │   ← centred overlay (ChildContent)
```

- **Diameter:** `diameter(size)` — 24, 40 or 56 pt.
- **Radius:** `(diameter − stroke_width) / 2`, so the stroke never escapes the
  allocated square.
- **Start angle:** −90° (12 o'clock), matching the SCSS's
  `transform: rotate(-90deg)`[reference:8].
- **Direction:** clockwise, because the arc is sampled with increasing angle in
  egui's screen space (y grows down).
- **Track:** `painter.circle_stroke(center, radius, Stroke::new(stroke, track))`.
- **Arc:** one `Shape::line` of `ARC_SEGMENTS + 1` points.
- **Rounded caps:** one `circle_filled` at each end, radius `stroke_width / 2`.

### Indeterminate model

```
phase 0.00:  arc at 12 o'clock, 30° wide
phase 0.25:  arc at  3 o'clock, 159° wide
phase 0.50:  arc at  6 o'clock, 288° wide
phase 0.75:  arc at  9 o'clock, 159° wide
phase 1.00:  arc at 12 o'clock, 30° wide
```

`phase = (time mod 1.4) / 1.4`. The rotation is `START_ANGLE + phase × TAU`; the
sweep is `MIN_SWEEP + (MAX_SWEEP − MIN_SWEEP) × (0.5 − 0.5 × cos(phase × TAU))`.
The result is a single arc that makes one full turn per loop while its length
breathes in and out once per loop.

MudBlazor achieves the same look with two CSS animations — the whole SVG rotates
one way, and the dash pattern rotates the other way — plus a `stroke-dasharray`
`80px 200px`[reference:9]. The Rust port collapses the two rotations into one and
replaces the CSS `ease-in-out` dash with a sinusoidal sweep. At a glance the
two read the same.

### Degenerate inputs

| Input | Behaviour |
|---|---|
| `max <= min` | `fraction` returns `0.0`; only the track is drawn. |
| Non-finite `value` / `min` / `max` | Treated as `0.0`; only the track is drawn. |
| `stroke_width > diameter / 2` | Clamped to `diameter / 2`; the radius is never negative. |
| `stroke_width == 0` | Clamped to `1`; the ring is always visible. |
| `indeterminate` + `value` | `value` is ignored, matching MudBlazor. |

---

## Performance and memory choices

| Choice | Effect |
|---|---|
| Theme fetched once per frame | One `MudTheme::fetch` and one `Color::resolve`. |
| Pure helpers on `Copy` values | Diameter, stroke clamp, fraction, phase, rotation and sweep are arithmetic; unit-testable without a `Context`. |
| One `Shape::line` per frame | The arc is a single polyline, not 64 separate line segments. |
| `Vec<Pos2>` capacity is `ARC_SEGMENTS + 1` | One allocation, bounded and predictable. |
| Track is one `circle_stroke` | No per-segment work for the background ring. |
| `rounded` adds two `circle_filled` | Only when the flag is on; a flat-cap arc costs nothing extra. |
| Label closure runs once | No double-run measure/draw pass. |
| Repaint only when indeterminate | A determinate circle requests no repaints; an indeterminate one does, as it must. |
| `Sense::hover()` | The ring is not interactive, matching MudBlazor. |

---

## Testing

`cargo test -p mud_egui_components mud_progress_circular` runs 18 pure-function
tests:

| Area | Tests |
|---|---|
| Diameter | every `Size`; monotonicity across small/medium/large |
| Stroke width | clamping between `1` and `diameter / 2`; degenerate diameter still yields `1` |
| `fraction` | unit range, clamping, degenerate `max <= min`, non-finite inputs |
| Indeterminate phase | periodicity across two loops; `NaN` / `∞` repair |
| Rotation | completes exactly one full turn per loop |
| Sweep | reaches `MIN_SWEEP` at 0 and 1, `MAX_SWEEP` at 0.5; never leaves the band |
| Arc points | starts at 12 o'clock; runs clockwise; `n + 1` points; every point lies on the circle; a full turn closes on itself |
| Builders | defaults match MudBlazor; every field set; tag round-trip |

No test creates an `egui::Context`. `show` needs a live `Ui` (layout, painting,
theme lookup, time) and is therefore left to integration tests; everything it
delegates to is covered above.

Suggested integration tests (`egui_kittest` or a headless `Context::run`
harness):

- A determinate circle at `value(50)` draws an arc covering half the ring.
- `value(0)` draws only the track.
- `rounded(true)` adds two caps; `rounded(false)` adds none.
- An indeterminate circle requests a repaint and its rotation advances with
  time.
- A label closure runs exactly once per frame.

```bash
cargo test -p mud_egui_components mud_progress_circular
```

---

## Differences from MudBlazor

- **`ChildContent` becomes `label(FnMut(&mut Ui))`.** The closure runs once per
  frame in a `Ui` sized to the circle and centred both ways, the direct
  equivalent of `.mud-progress-circular-percentage`.
- **The arc is a polyline, not an SVG path.** `ARC_SEGMENTS = 64` makes the
  approximation invisible at every `Size`. If you need a mathematically exact
  arc, replace the `Shape::line` call in `paint_arc` with an `epaint::PathShape`
  of cubic Béziers — nothing else changes.
- **`Rounded` draws caps instead of setting `stroke-linecap`.** egui has no
  stroke cap; a filled circle of radius `stroke_width / 2` at each endpoint is
  the same silhouette.
- **The indeterminate animation is a single time-based arc.** MudBlazor's
  `rotate` + `dash` CSS animations are approximated by one rotating arc whose
  sweep breathes sinusoidally. The loop is still 1.4 s and the visual rhythm
  matches.
- **`Size` folds `ExtraSmall` into `Small` and `ExtraLarge` into `Large`.** The
  SCSS only defines `small`, `medium` and `large`; folding the edges is what
  MudBlazor does.
- **`StrokeWidth` is `u8`, not `int`.** Clamped to `1..=diameter / 2` at paint
  time so the radius can never invert. The default is still `3`.
- **`Class` is stored but not resolved.** The ring's colour comes from `color`;
  a utility-class frame would composite a second fill.
- **`Style` and `UserAttributes` are not ported.** No CSS strings, no HTML tag.
- **`value` is `f64`, not `Option<double>`.** The DNDocs page lists it as a
  plain `double` and the component exposes no `ValueChanged`; this is a
  display-only component, and no two-way binding is invented.

===== ./crates/mud_egui_components/src/feedback/progress/mud_progress_linear/README.md =====

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

===== ./crates/mud_egui_components/src/feedback/skeleton/README.md =====

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

===== ./crates/mud_egui_components/src/feedback/tooltip/README.md =====

# MudTooltip for `mud_egui`

A small popup that shows additional information when the pointer hovers, the
child gains focus, or the child is clicked, for
[egui](https://github.com/emilk/egui). Ported from MudBlazor's
`MudTooltip` (MudBlazor 8.13.0).

```rust
let mut tip = TooltipState::new("save-tip");

MudTooltip::new()
    .text("Save the file (Ctrl+S)")
    .placement(Placement::Top)
    .show(ui, &mut tip, |ui| {
        ui.add(MudButton::new("Save"))
    });
```

The trigger is whatever you draw — `MudTooltip` does not impose a widget
style. Tooltip text renders through **`MudText`**; custom bodies via
`tooltip_content` should use `MudText` / `MudIcon` per project convention.

---

## Contents

1. [Design principle: composition + caller-owned state](#design-principle-composition--caller-owned-state)
2. [Architecture](#architecture)
3. [API reference](#api-reference)
4. [Examples](#examples)
5. [Trigger semantics](#trigger-semantics)
6. [Animation and delays](#animation-and-delays)
7. [Performance and memory choices](#performance--memory-choices)
8. [Testing](#testing)
9. [Differences from MudBlazor](#differences-from-mudblazor)

---

## Design principle: composition + caller-owned state

`MudTooltip` is built from one other primitive:

| Primitive | Role |
|-----------|------|
| [`MudPopover`](../popover/README.md) | The positioned tooltip body. Anchor attachment, flip-when-overflowing, and z-ordering are all delegated. |

Unlike `MudSelect` / `MudAutocomplete` / `MudMenu`, **`MudPopover`'s dismiss
hooks are disabled**. A tooltip is display-only — the component itself decides
when to show and hide (hover delay + blur + click-toggle). If the popover's
`dismiss_on_outside_click` were enabled, clicking anywhere would prematurely
close the tooltip while the pointer is still on the trigger.

### Ported parameters

| MudBlazor | Rust method | Default |
|-----------|-------------|---------|
| `Arrow` | `arrow(bool)` | `false` |
| `ChildContent` | `child` argument of `show` | required |
| `Color` | `color(Color)` | `Default` |
| `Delay` | `delay_ms(u32)` | `0` |
| `Disabled` | `disabled(bool)` | `false` |
| `Duration` | `duration_ms(u32)` | `251` |
| `Inline` | `inline(bool)` | `true` (stored; no-op) |
| `Placement` | `placement(Placement)` | `Bottom` |
| `RightToLeft` | `right_to_left(bool)` | `false` |
| `RootClass` | `classes(iter)` | `None` |
| `RootStyle` | *(use `classes`)* | — |
| `ShowOnClick` | `show_on_click(bool)` | `false` |
| `ShowOnFocus` | `show_on_focus(bool)` | `true` |
| `ShowOnHover` | `show_on_hover(bool)` | `true` |
| `Text` | `text(s)` | `None` |
| `TooltipContent` | `tooltip_content(f)` | `None` |
| `Visible` | `TooltipState::visible` | `false` |
| `VisibleChanged` | `on_visible_changed(f)` | `None` |

### Extensions (not in MudBlazor)

| Extension | Description |
|-----------|-------------|
| `TooltipState` | Caller-owned visibility + timers. |
| `TooltipResponse` | Frame's `child` response, resulting `visible`, and whether it changed. |
| `compute_opacity` / `resolve_placement` / `placement_to_origins` | Pure helpers, unit-tested. |

### Deferred to pass 2

- **`Inline`** — MudBlazor's block/inline toggle affects how the *trigger*
  flows in the surrounding layout. egui tooltips are always overlaid on top
  of the parent layout, so the parameter has no equivalent. Stored for parity.
- **`RootStyle`** — egui has no inline CSS strings; use `classes(..)`.
- **Protruding arrow** — pass 1 draws the arrow as a small triangle *inside*
  the tooltip paper, on the anchor-facing edge. MudBlazor's arrow protrudes
  *outside* the paper and touches the anchor. The Rust port's arrow is
  functionally equivalent (it points toward the anchor and expands the visual
  silhouette) but is a few pixels smaller. A proper outdenting caret via a
  second `Area` is deferred to pass 2.

### Not ported

- **`Class` / `Style` / `Tag` / `UserAttributes`** — no CSS strings or HTML
  tags in egui.

---

## Architecture

```
overlays/tooltip/
├── tooltip_state.rs  — TooltipState (visibility + timers + click latch)
├── mud_tooltip.rs    — MudTooltip + TooltipResponse + pure helpers + tests
└── mod.rs
```

**Pure layer.** `resolve_placement`, `placement_to_origins`,
`compute_opacity`, `desired_visibility`, `resolve_tooltip_colors`,
`with_opacity` — free functions, unit-tested without `egui::Context`.

**State layer.** `TooltipState` holds the four timing variables (hover,
show, hide, click-latch) plus the underlying `PopoverState`.

**Component layer.** `show` renders the child, runs the trigger state
machine, computes the fade opacity, and delegates positioning to
`MudPopover`.

---

## API reference — `MudTooltip`

| Method | Type | Default | Description |
|--------|------|---------|-------------|
| `new()` | — | — | Default tooltip. |
| `arrow(bool)` | `bool` | `false` | Draw a small triangle on the anchor-facing edge. |
| `color(Color)` | `Color` | `Default` | Tooltip accent colour. |
| `delay_ms(u32)` | `u32` | `0` | Delay before showing, in ms. |
| `duration_ms(u32)` | `u32` | `251` | Fade in / out duration, in ms. |
| `disabled(bool)` | `bool` | `false` | Suppresses every trigger. |
| `inline(bool)` | `bool` | `true` | Stored for parity; no-op in egui. |
| `placement(Placement)` | `Placement` | `Bottom` | Tooltip side. |
| `right_to_left(bool)` | `bool` | `false` | RTL flag for `Start` / `End`. |
| `show_on_click(bool)` | `bool` | `false` | Click-toggle on the trigger. |
| `show_on_focus(bool)` | `bool` | `true` | Show when the trigger has focus. |
| `show_on_hover(bool)` | `bool` | `true` | Show on pointer hover. |
| `text(s)` | `impl Into<String>` | `None` | Tooltip text. |
| `tooltip_content(f)` | `FnMut(&mut Ui)` | `None` | Custom body, overrides `text`. |
| `on_visible_changed(f)` | `FnMut(bool)` | `None` | Fires when visibility changes. |
| `classes(iter)` | `iter of AsRef<str>` | none | Body utility classes. |
| `flavor(StyleFlavor)` | `StyleFlavor` | `Bootstrap` | Utility resolver. |
| `show(ui, state, child)` | `(&mut Ui, &mut TooltipState, FnOnce(&mut Ui) -> Response)` | — | Returns `TooltipResponse`. |

### `TooltipState`

| Field / method | Description |
|----------------|-------------|
| `visible: bool` | Logical visibility. |
| `popover: PopoverState` | Underlying popover state. |
| `hover_started_at` / `show_started_at` / `hide_started_at` | Timers. |
| `click_open: bool` | `ShowOnClick` latch. |
| `new(id_salt)` / `is_visible()` | Constructors + accessor. |
| `show(now)` / `hide(now)` / `toggle(now)` | Programmatic control. |

### `TooltipResponse`

| Field | Type | Description |
|-------|------|-------------|
| `child` | `egui::Response` | The trigger's response. |
| `visible` | `bool` | Logical visibility after this frame. |
| `changed` | `bool` | True when `visible` flipped. |

---

## Examples

### Hover tooltip

```rust
MudTooltip::new()
    .text("Deletes the selected row")
    .show(ui, &mut state, |ui| ui.add(MudButton::new("Delete")));
```

### Focus-triggered (keyboard navigation)

```rust
MudTooltip::new()
    .text("Press Enter to confirm")
    .placement(Placement::Top)
    .show(ui, &mut state, |ui| ui.add(MudButton::new("Confirm")));
```

### Click-toggle (sticky tooltip)

```rust
MudTooltip::new()
    .text("Click again to dismiss")
    .show_on_click(true)
    .show_on_hover(false)
    .show_on_focus(false)
    .show(ui, &mut state, |ui| ui.add(MudIconButton));
```

### Delay before showing

```rust
MudTooltip::new()
    .text("Appears after 500 ms")
    .delay_ms(500)
    .show(ui, &mut state, |ui| ui.add(MudButton::new("Help")));
```

### Arrow indicator

```rust
MudTooltip::new()
    .text("With arrow")
    .arrow(true)
    .placement(Placement::Top)
    .show(ui, &mut state, |ui| ui.add(MudIcon::new(filled::INFO)));
```

### Custom body

```rust
MudTooltip::new()
    .tooltip_content(|ui| {
        ui.add(MudText::new("Custom").bold(true));
        ui.add(MudText::new("Anything you can draw"));
    })
    .show(ui, &mut state, |ui| ui.add(MudButton::new("Details")));
```

### Listen for visibility changes

```rust
MudTooltip::new()
    .on_visible_changed(|v| log::info!("tooltip visible: {v}"))
    .text("With callback")
    .show(ui, &mut state, |ui| ui.add(MudButton::new("Open")));
```

### Programmatic control

```rust
state.show(ui.input(|i| i.time));
state.hide(ui.input(|i| i.time));
state.toggle(ui.input(|i| i.time));
```

---

## Trigger semantics

The tooltip's visible state is the OR of three independent triggers:

| Trigger | When it fires | Parameter |
|---------|--------------|-----------|
| Hover   | Pointer has been over the trigger for `delay_ms` | `show_on_hover` |
| Focus   | Trigger has keyboard focus | `show_on_focus` |
| Click   | Trigger was clicked once (toggle) | `show_on_click` |

Whichever trigger wins, the tooltip stays visible until **all three** are
false. `disabled(true)` suppresses every trigger regardless of the other
flags.

### Hover delay

`delay_ms > 0` starts a timer when the pointer first enters the trigger. The
tooltip opens when the timer elapses. Leaving the trigger before the timer
elapses cancels the pending open. `delay_ms == 0` opens immediately.

The component calls `ctx.request_repaint_after(delay_ms)` so the tooltip
opens at the right moment even if the app is otherwise idle.

### Click latch

`show_on_click(true)` flips a per-tooltip latch on every trigger click.
Combined with `show_on_hover(false)` this gives a "sticky tooltip" UX. The
latch is stored in `TooltipState::click_open` and can be reset by the caller
at any time.

---

## Animation and delays

The tooltip fades in over `duration_ms` when it becomes visible, and fades
out over `duration_ms` when it becomes invisible. Opacity is computed by the
pure `compute_opacity` helper:

| State | Opacity |
|-------|---------|
| Idle | `0.0` |
| Fading in | `(now - show_started) / duration` |
| Fading out | `1 - (now - hide_started) / duration` |
| Fully visible | `1.0` |

While opacity is between `0.0` and `1.0` (or while fading), the component
calls `ctx.request_repaint()` so the fade advances smoothly.

`duration_ms == 0` snaps between `0.0` and `1.0` with no animation.

When the fade-out completes, `TooltipState::reset_animation` clears the
timers and closes the popover so the next `show` starts cleanly.

---

## Performance and memory choices

| Choice | Effect |
|--------|--------|
| Pure `resolve_placement` / `compute_opacity` / `desired_visibility` | No `egui::Context`; unit-tested. |
| `TooltipState` is one `Clone` struct | Zero per-frame allocation. |
| Timers are `Option<f64>` seconds since start | No datetime arithmetic; direct comparison. |
| Fade animation calls `request_repaint` only while animating | No idle frames. |
| Hover delay calls `request_repaint_after(delay)` | Precise open timing without busy-looping. |
| `MudPopover` reuse | Zero duplication of anchor / flip / z-order logic. |
| Fade applies alpha to fill + ink simultaneously | Text and background fade together. |
| Popover paper drawn once per frame | No per-frame allocations for the fill. |

---

## Testing

`cargo test` runs 18 pure-function tests:

| Area | Tests |
|------|-------|
| `resolve_placement` | Top/Bottom pass-through, Start↔End under RTL |
| `placement_to_origins` | every variant |
| `compute_opacity` | idle, zero-duration snap, fade-in linear, fade-out linear |
| `desired_visibility` | disabled blocks all, each trigger independently |
| `with_opacity` | identity, transparent, alpha scaling |
| Defaults | every field matches MudBlazor |
| Builders | every field set |
| `TooltipState` | start hidden, show/hide idempotence, toggle, reset_animation |

Rendering (`show`) needs a live `egui::Context`; the pure layer is the source
of truth.

```bash
cargo test -p mud_egui_components mud_tooltip
cargo test -p mud_egui_components tooltip_state
```

---

## Differences from MudBlazor

- **State is caller-owned (`TooltipState`).** MudBlazor uses the `Visible` /
  `VisibleChanged` binding; the port puts both in the state struct. Same
  caller-owned-state pattern as the rest of the batch.
- **`Inline` is stored but not rendered.** MudBlazor's block/inline toggle
  affects the trigger's flow in the surrounding layout. egui tooltips are
  always overlaid, so there is no equivalent.
- **Arrow drawn inside the paper.** MudBlazor's arrow protrudes outside the
  paper. Pass 1 draws a smaller triangle inside the paper on the
  anchor-facing edge — functionally equivalent (points toward the anchor,
  expands the silhouette) but not pixel-perfect. Deferred to pass 2.
- **`Style` / `Tag` / `UserAttributes` not ported.** No CSS strings or HTML
  tags in egui.
- **Popover dismissals disabled.** `dismiss_on_escape` and
  `dismiss_on_outside_click` are both `false` so the trigger state machine
  is the sole arbiter of visibility. Pressing Escape does not close a
  tooltip unless you wire it into `on_visible_changed` or drive
  `TooltipState::hide` yourself.

===== ./crates/mud_egui_components/src/inputs/autocomplete/README.md =====

# MudAutocomplete for `mud_egui`

A text field with a filtered dropdown of candidates, single-value
selection only, for [egui](https://github.com/emilk/egui). Ported from
MudBlazor's `MudAutocomplete<T>` (MudBlazor 8.13.0).

```rust
let mut state = AutocompleteState::<Fruit>::new("fruit");

// Refresh candidates when the search text changes.
if state.text_changed_since_search() {
    state.candidates = fruits
        .iter()
        .filter(|f| f.name.to_lowercase().contains(&state.search_text.to_lowercase()))
        .cloned()
        .collect();
    state.mark_searched();
}

MudAutocomplete::new()
    .label("Fruit")
    .tostring_func(|f| f.name.clone())
    .show(ui, &mut state);
```

The trigger renders through **`MudTextField`** (so its label, placeholder,
helper, error, variant, margin, size, and search-icon adornment come from the
same widget). Candidate rows render text through **`MudText`**; any custom
per-item icons or badges go through `MudIcon` / `MudBadge` inside
`item_template`.

---

## Design principle: caller-owned async, not per-frame `await`

MudBlazor's `SearchFunc` returns `Task<IEnumerable<T>>`. egui is
immediate-mode with no async runtime per frame. The port therefore
**delegates the search to the caller**:

| MudBlazor concept | Rust equivalent |
|-------------------|-----------------|
| `SearchFunc: Func<string, Task<IEnumerable<T>>>` | `state.candidates: Vec<T>` — the caller populates it. |
| Debounce via `DebounceInterval` | `state.text_changed_since_search()` + `mark_searched()`. |
| Async channel-based search | Push results into `candidates` from a channel reader; the component never knows. |
| `ShowProgressIndicator` | Caller can render a spinner above the popover when their channel is empty but still awaiting. |

This is documented in the module header of `mud_autocomplete.rs` and matches
the caller-owned-state pattern every other component in this batch follows
(`MudTable`, `MudTreeView`, `MudChipSet`, `MudNavMenu`, `MudPagination`,
`MudStepper`, `MudTabs`, `MudSelect`).

### Composition, not reimplementation

`MudAutocomplete` reuses the same two primitives as `MudSelect`:

| Primitive | Role |
|-----------|------|
| [`MudTextField`](../../inputs/text_field/README.md) | The editable trigger. |
| [`MudPopover`](../../overlays/popover/README.md) | The candidate dropdown. |

Only new code is the **candidate list** (rows + keyboard focus) and the
**commit / coerce / reset logic** — both pure helpers tested without
`egui::Context`.

---

## Pass 1 scope and pass 2 deferral

### Ported `MudAutocomplete` parameters

| MudBlazor | Rust method | Default |
|-----------|-------------|---------|
| `Value` | `AutocompleteState::selected` (`Option<T>`) | `None` |
| `Text` | `AutocompleteState::search_text` | `""` |
| `ToStringFunc` | `tostring_func(f)` | `None` |
| `SearchFunc` | *(caller-owned candidates)* | — |
| `Label` | `label(s)` | `None` |
| `Placeholder` | `placeholder(s)` | `None` |
| `HelperText` | `helper_text(s)` | `None` |
| `Error` | `error(bool)` | `false` |
| `ErrorText` | `error_text(s)` | `None` |
| `Required` | `required(bool)` | `false` (visual only) |
| `Disabled` | `disabled(bool)` | `false` |
| `ReadOnly` | `readonly(bool)` | `false` |
| `Clearable` | `clearable(bool)` | `true` |
| `Variant` | `variant(Variant)` | `Text` |
| `Margin` | `margin(Margin)` | `None` |
| `Color` | `color(Color)` | `Primary` |
| `Size` | `size(Size)` | `Medium` |
| `Dense` | `dense(bool)` | `false` |
| `Adornment` + `AdornmentIcon` + `AdornmentColor` | `adornment_icon(s)` / `adornment_color(Color)` | `SEARCH` / `None` |
| `AnchorOrigin` | `anchor_origin(AnchorOrigin)` | `None` |
| `TransformOrigin` | `transform_origin(TransformOrigin)` | `None` |
| `Position` | `position(Position)` | `None` |
| `MaxItems` | `max_items(usize)` | `None` |
| `MinCharacters` | `min_characters(usize)` | `0` |
| `OpenOnFocus` | `open_on_focus(bool)` | `true` |
| `ResetValueOnEmptyText` | `reset_value_on_empty_text(bool)` | `false` |
| `NoItemsTemplate` | `no_items_template(f)` | `None` |
| `AfterItemsTemplate` | `after_items_template(f)` | `None` |
| `ItemTemplate` | `item_template(f)` | `None` |
| `OnOpen` | `on_open(f)` | `None` |
| `OnClose` | `on_close(f)` | `None` |

### Deferred to pass 2

| Deferred parameter | Why pass 2 |
|--------------------|------------|
| `CoerceValue` / `CoerceValueDelay` | Blur coercion (typed text exactly matching one candidate). Straightforward, deferred for pass-1 simplicity. |
| `DebounceInterval` | Overlaps with the caller-owned search model; the caller uses `text_changed_since_search` and their own timing. |
| `Strict` | Reverting on typed-but-not-coerced text — depends on CoerceValue. |
| `SelectValueOnTab` | One keybinding; deferred. |
| `ShowProgressIndicator` / `ProgressIndicatorColor` / `ProgressIndicatorInPopoverTemplate` | Depends on async; the caller renders their own spinner. |
| `MoreItemsTemplate` | "Showing N of M" footer — needs `TotalItems` count which async callers may not have. |
| `KeyDownPreventDefault` / `KeyUpPreventDefault` | Browser-specific. |
| `HelperTextOnFocus` | Helper row behaviour. |
| `ShrinkLabel` / `DisableUnderLine` / `AriaLabel` | Cosmetic / accessibility refinements. |
| `AfterItemsTemplate` with parameter / `NoItemsTemplate` with parameter | The pass-1 forms take no argument. |
| `ReturnedItemsCount` | Available as `state.candidates.len()`. |

### Not ported

- **`Class` / `Style` / `Tag` / `UserAttributes`** — no CSS strings or HTML
  tags in egui.
- **`InputClass` / `OuterClass`** — CSS string variants, same reason.

---

## Architecture

```
inputs/autocomplete/
├── autocomplete_action.rs  — AutocompleteAction<T>
├── autocomplete_state.rs   — AutocompleteState<T>
├── mud_autocomplete.rs     — MudAutocomplete + pure helpers + tests
└── mod.rs
```

**Pure layer.** `row_height`, `display_text_of`, `next_focused_index`,
`prev_focused_index`, `commit_selection`, `position_to_origins` — free
functions, unit-tested without `egui::Context`.

**State layer.** `AutocompleteState<T>` holds the search text, the
caller-populated candidates, the committed selection, and the popover state.
`text_changed_since_search` and `mark_searched` provide the debounce
primitive.

**Component layer.** `show` renders the trigger (`MudTextField`), then handles
open/close + keyboard navigation + candidate clicks, then delegates popover
positioning to `MudPopover`.

---

## API reference — `MudAutocomplete<T>`

| Method | Type | Default | Description |
|--------|------|---------|-------------|
| `new()` | — | — | Default autocomplete. |
| `tostring_func(f)` | `Fn(&T) -> String` | `None` | Candidate display text. `"?"` when absent. |
| `error(bool)` / `error_text(s)` | `bool` / `impl Into<String>` | `false` / `None` | Error visual only. |
| `required(bool)` | `bool` | `false` | Visual only. |
| `disabled(bool)` / `readonly(bool)` | `bool` | `false` | Interaction flags. |
| `clearable(bool)` | `bool` | `true` | Shows the clear button. |
| `open_on_focus(bool)` | `bool` | `true` | Auto-open on focus. |
| `reset_value_on_empty_text(bool)` | `bool` | `false` | Clear selection when text is cleared. |
| `min_characters(usize)` | `usize` | `0` | Min text before the dropdown opens. |
| `max_items(usize)` | `usize` | `None` | Max candidates shown. |
| `label(s)` / `placeholder(s)` / `helper_text(s)` | `impl Into<String>` | `None` | Trigger text slots. |
| `variant(Variant)` / `margin(Margin)` / `color(Color)` / `size(Size)` | enum | `Text` / `None` / `Primary` / `Medium` | Trigger appearance. |
| `dense(bool)` | `bool` | `false` | Compact candidate rows. |
| `adornment_icon(s)` / `adornment_color(Color)` | `impl Into<String>` / `Color` | `SEARCH` / `None` | Search icon and colour. |
| `position(Position)` / `anchor_origin(..)` / `transform_origin(..)` | placement | `None` | Popover placement. |
| `right_to_left(bool)` | `bool` | `false` | RTL flag. |
| `max_height(f32)` / `popover_class(s)` | `f32` / `impl Into<String>` | `300.0` / `None` | Popover box. |
| `item_template(f)` | `FnMut(&mut Ui, &T)` | `None` | Custom candidate renderer. |
| `no_items_template(f)` | `FnMut(&mut Ui)` | `None` | Custom "no results" renderer. |
| `after_items_template(f)` | `FnMut(&mut Ui)` | `None` | Footer under the candidate list. |
| `on_open(f)` / `on_close(f)` | `FnMut()` | `None` | Open/close callbacks. |
| `classes(iter)` / `flavor(StyleFlavor)` | — | — | Utilities. |
| `show(ui, state)` | `(&mut Ui, &mut AutocompleteState<T>)` | — | Returns `AutocompleteResponse<T>`. |

### `AutocompleteState<T>`

| Field / method | Description |
|----------------|-------------|
| `search_text: String` | Current text (edited by the `TextEdit`). |
| `selected: Option<T>` | Committed selection. |
| `candidates: Vec<T>` | Caller-owned candidate list. |
| `popover: PopoverState` | Open/closed + stable id. |
| `focused_candidate: Option<usize>` | Keyboard-focused index. |
| `last_searched_text: String` | Snapshot for `text_changed_since_search`. |
| `new(id_salt)` / `with_selected_text(v, s)` | Constructors. |
| `is_open()` / `open()` / `close()` / `toggle()` | Popover control. |
| `select(v, s)` / `clear()` | Programmatic commit / reset. |
| `text_changed_since_search()` / `mark_searched()` | Debounce primitive. |
| `is_selected(&T)` | Membership check. |

### `AutocompleteResponse<T>`

| Field | Type | Description |
|-------|------|-------------|
| `trigger` | `egui::Response` | Trigger response. |
| `selected` | `Option<T>` | Selection after this frame. |
| `search_text` | `String` | Search text after this frame. |
| `action` | `AutocompleteAction<T>` | What happened this frame. |

### `AutocompleteAction<T>`

| Variant | Description |
|---------|-------------|
| `None` | No interaction. |
| `Opened` / `Closed` | Dropdown opened / closed. |
| `Selected { value }` | A candidate was committed. |
| `Cleared` | Selection cleared. |

Helpers: `changed_selection()`, `toggled_open()`.

---

## Examples

### Basic autocomplete

```rust
let mut state = AutocompleteState::<String>::new("search");

if state.text_changed_since_search() {
    state.candidates = all_fruits
        .iter()
        .filter(|f| f.to_lowercase().contains(&state.search_text.to_lowercase()))
        .cloned()
        .collect();
    state.mark_searched();
}

MudAutocomplete::new()
    .label("Fruit")
    .tostring_func(|s| s.clone())
    .show(ui, &mut state);
```

### Async search via channel

```rust
// In your app struct:
//   search_tx: Sender<String>
//   search_rx: Receiver<Vec<Fruit>>
//   last_dispatched: String
//
// Each frame:
if state.text_changed_since_search() {
    // Dispatch the search (non-blocking).
    search_tx.try_send(state.search_text.clone()).ok();
    state.mark_searched();
}
// Drain any results that have arrived.
while let Ok(results) = search_rx.try_recv() {
    state.candidates = results;
    ui.ctx().request_repaint();
}

MudAutocomplete::new()
    .label("Fruit")
    .tostring_func(|f| f.name.clone())
    .show(ui, &mut state);
```

### With item template (icon + secondary text)

```rust
MudAutocomplete::new()
    .tostring_func(|f: &Fruit| f.name.clone())
    .item_template(|ui, fruit| {
        ui.add(MudIcon::new(filled::ECO));
        ui.add(MudText::new(fruit.name.clone()));
        ui.add(MudText::new(fruit.origin.clone()).font_size(11.0));
    })
    .show(ui, &mut state);
```

### No-results template

```rust
MudAutocomplete::new()
    .no_items_template(|ui| {
        ui.add(MudText::new("No fruits matched your search"));
    })
    .show(ui, &mut state);
```

### Open on focus off (click to open)

```rust
MudAutocomplete::new()
    .open_on_focus(false)
    .show(ui, &mut state);
```

### Reset on empty text

```rust
MudAutocomplete::new()
    .reset_value_on_empty_text(true)
    .show(ui, &mut state);
// Clearing the text and blurring → `state.selected = None`.
```

### Max items and dense

```rust
MudAutocomplete::new()
    .max_items(5)
    .dense(true)
    .show(ui, &mut state);
```

### Min characters

```rust
MudAutocomplete::new()
    .min_characters(2)
    .show(ui, &mut state);
// The dropdown stays closed until at least 2 characters are typed.
```

### Read the outcome

```rust
let resp = MudAutocomplete::new().show(ui, &mut state);
match resp.action {
    AutocompleteAction::Selected { value } => log::info!("selected {value:?}"),
    AutocompleteAction::Cleared => log::info!("cleared"),
    AutocompleteAction::Opened => log::info!("opened"),
    AutocompleteAction::Closed => log::info!("closed"),
    AutocompleteAction::None => {}
}
```

### Programmatic set / clear

```rust
state.select(my_fruit.clone(), my_fruit.name.clone());
state.clear();
```

### Focus a candidate programmatically

```rust
state.focused_candidate = Some(0);
```

---

## Keyboard navigation

While the trigger has focus **and** the dropdown is open:

| Key | Action |
|-----|--------|
| `↓` | Focus next candidate (clamps at last) |
| `↑` | Focus previous candidate (clamps at first) |
| `Enter` | Commit the focused candidate |
| `Escape` | Close the dropdown |

Blur (Tab / click outside) closes the dropdown and reverts `search_text` to
the selected item's display text (or empty when nothing is selected).

---

## Performance and memory choices

| Choice | Effect |
|--------|--------|
| `display_text_of` / `next_focused_index` / `prev_focused_index` / `commit_selection` are pure | Unit-tested without `egui::Context`. |
| `AutocompleteState<T>` owns `candidates` | No per-frame allocation for candidates once populated. |
| `text_changed_since_search` + `mark_searched` | Prevents repeated filter runs on every frame. |
| Row rendering is index-based | Avoids `iter()` + mutation conflict; clone-on-click. |
| `item_template` is `take()`n from `self` before the render loop | No borrow conflict with `state.candidates` reads. |
| `with_alpha` uses `f32` multiplier | Matches `MudChip` / `MudSelect`. |
| `MudTextField` / `MudPopover` reuse | Zero duplication of trigger chrome or dropdown positioning. |
| `MudIcon` reuse across rows | Free thanks to the transform cache. |
| Focus fill + hover fill both checked | Keyboard focus is visible even when the mouse isn't moving. |

---

## Testing

`cargo test` runs 15+ pure-function tests:

| Area | Tests |
|------|-------|
| `row_height` | dense / normal |
| `display_text_of` | with / without `tostring_func` |
| `next_focused_index` / `prev_focused_index` | empty list, clamping, wrap-free behaviour |
| `commit_selection` | writes text, sets selected, closes popover, clears focused |
| `position_to_origins` | bottom / top / left / right |
| Defaults | every `MudAutocomplete` field matches MudBlazor |
| Builders | every field set |
| `AutocompleteState` | open / close / toggle, select / clear, text-changed helper |
| `AutocompleteAction` | `changed_selection` / `toggled_open` |

Rendering (`show`) needs a live `egui::Context`; the pure layer is the
source of truth.

```bash
cargo test -p mud_egui_components mud_autocomplete
cargo test -p mud_egui_components autocomplete_state
cargo test -p mud_egui_components autocomplete_action
```

---

## Differences from MudBlazor

- **`SearchFunc` → caller-owned `candidates`.** MudBlazor's async `Task<IEnumerable<T>>` is not a fit for immediate-mode egui. The port exposes `text_changed_since_search` and `mark_searched` so callers can debounce and populate candidates themselves — sync filter, cached result, or channel read from an async task.
- **`CoerceValue` / `CoerceValueDelay` / `Strict` / `SelectValueOnTab` / `ShowProgressIndicator` deferred.** See the deferral table.
- **`ReturnedItemsCount` → `state.candidates.len()`.** No separate parameter needed.
- **`ResetValueOnEmptyText` is a real behaviour.** Blurring with empty text clears the selection (when enabled) — this is implemented.
- **`tostring_func` optional with `"?"` fallback.** MudBlazor requires either a `ToStringFunc` or `T: ToString`. The Rust port doesn't force `T: Display`; instead, absent `tostring_func` shows `"?"` in both the trigger and candidate rows (visible marker, no panic).
- **`ItemTemplate` / `NoItemsTemplate` / `AfterItemsTemplate` take `&mut Ui`.** MudBlazor render fragments do the same; no change.
- **`Class` / `Style` / `Tag` / `UserAttributes` not ported.** No CSS strings or HTML tags in egui.

===== ./crates/mud_egui_components/src/inputs/check_box/README.md =====

# MudCheckBox for `mud_egui`

A checkbox for boolean or tri-state values, for
[egui](https://github.com/emilk/egui), ported from MudBlazor's
[`MudCheckBox<T>`](https://mudblazor.com/components/checkbox) (MudBlazor 8.13.0).

```rust
let mut accepted = false;
ui.add(
    MudCheckBox::new(&mut accepted)
        .label("I agree to the terms")
        .color(Color::Primary),
);
if accepted {
    // …
}
```

Every label renders through **`MudText`**, every glyph through **`MudIcon`**.

---

## Contents

1. [Design principle: API parity, not API invention](#design-principle-api-parity-not-api-invention)
2. [Architecture](#architecture)
3. [API reference](#api-reference)
4. [Examples](#examples)
5. [Binding model](#binding-model)
6. [Layout and rendering](#layout-and-rendering)
7. [Performance and memory choices](#performance-and-memory-choices)
8. [Testing](#testing)
9. [Differences from MudBlazor](#differences-from-mudblazor)

---

## Design principle: API parity, not API invention

The MudBlazor 8.13.0 parameter surface was verified against DNDocs before
implementation. `MudCheckBox<T>` inherits from `MudBooleanInput<T>`, so its
surface combines check-box-specific appearance parameters with the shared
boolean-input parameters.

### Direct `MudCheckBox` parameters

| MudBlazor | Type | Default | Rust mapping |
|---|---|---|---|
| `CheckedIcon` | `string` | `CheckBox` | `checked_icon(s)` |
| `Dense` | `bool` | `false` | `dense(bool)` |
| `IndeterminateIcon` | `string` | `IndeterminateCheckBox` | `indeterminate_icon(s)` |
| `KeyboardEnabled` | `bool` | `true` | `keyboard_enabled(bool)` |
| `TriState` | `bool` | `false` | `tri_state(bool)` |
| `UncheckedColor` | `Color?` | `null` | `unchecked_color(Color)` |
| `UncheckedIcon` | `string` | `CheckBoxOutlineBlank` | `unchecked_icon(s)` |

### Inherited from `MudBooleanInput<T>`

| MudBlazor | Type | Default | Rust mapping |
|---|---|---|---|
| `Value` | `T?` | — | caller-owned `&mut T` via [`CheckBoxValue`] |
| `ValueChanged` | `EventCallback<T?>` | — | `CheckBoxResponse::changed` |
| `Label` | `string?` | `null` | `label(s)` |
| `LabelPlacement` | `Placement` | `End` | `label_placement(Placement)` |
| `Ripple` | `bool` | `true` | `ripple(bool)` |
| `Size` | `Size` | `Medium` | `size(Size)` |
| `Color` | `Color` | `Default` → `Primary` | `color(Color)` |
| `Disabled` | `bool` | `false` | `disabled(bool)` |
| `ReadOnly` | `bool` | `false` | `read_only(bool)` |
| `Class` | `string?` | `null` | `classes(iter)` — stored, not resolved |
| `Tag` | `object?` | `null` | `tag(t)` / `tag_ref::<U>()` |

### Deliberately not ported

| MudBlazor | Reason |
|---|---|
| `Style` | egui has no inline CSS strings. |
| `UserAttributes` | There is no HTML tag to splat attributes onto. |
| `Class` resolution | Stored on `base` for parity only. The checkbox's colour comes from `Color` / `UncheckedColor`; a utility-class frame would composite a second fill. |
| Form-validation parameters (`Required`, `Error`, `ErrorText`, `HelperText`, `ConversionError`, `Strict`, `Immediate`, `DebounceInterval`) | Inherited from `MudFormComponent` / `MudBaseInput`; they have no egui equivalent without a form system. |
| `Typo` | **Not a parameter on `MudCheckBox` in 8.13.0.** The label uses the theme's default typography (`Body1`). Adding a `Typo` builder would be API invention. |
| `ChildContent` (from `MudBooleanInput`) | Not exposed by `MudCheckBox`’s DNDocs page; the custom-icon use case is served by `CheckedIcon` / `UncheckedIcon` / `IndeterminateIcon`. |

---

## Architecture

```
inputs/check_box/
└── mud_check_box.rs
    ├── CheckBoxValue       trait; impl for bool and Option<bool>
    ├── MudCheckBox<'a, T>  builder + show
    ├── CheckBoxResponse    response + changed
    ├── PaintState          Checked | Unchecked | Indeterminate
    ├── pure helpers        icon_side, next_value, layout_size,
    │                       is_vertical, label_leads, icon_rect, label_rect,
    │                       paint_state, resolve_icon, with_alpha, apply_keys
    ├── ripple              start_ripple, paint_ripple (memory-backed)
    └── impl Widget         ui.add(MudCheckBox::new(&mut v))
```

**Pure layer.** The icon size, the value cycle (boolean and tri-state), the
row layout, the paint state, the glyph resolution and the keyboard mapping are
free functions over `Option<bool>`, `f32` and `Placement`. None touch
`egui::Context`.

**Binding layer.** `CheckBoxValue` is a two-method trait implemented for `bool`
and `Option<bool>`. One `MudCheckBox` serves both modes; the trait is the
seam.

**Component layer.** `show` measures the label, allocates the row, applies a
click or key press, paints the ripple and the icon, and draws the label through
`MudText`.

---

## API reference

### `MudCheckBox<'a, T: CheckBoxValue>`

| Method | Type | Default | Mirrors | Description |
|---|---|---|---|---|
| `new(value)` | `&'a mut T` | — | `Value` | Binds the checkbox to a `bool` or `Option<bool>`. |
| `label(s)` | `impl Into<String>` | `None` | `Label` | Label text. |
| `label_placement(p)` | `Placement` | `End` | `LabelPlacement` | `Start` / `End` / `Top` / `Bottom` (plus `Left` / `Right`). |
| `dense(b)` | `bool` | `false` | `Dense` | Compact gap (4 pt instead of 8 pt). |
| `color(c)` | `Color` | `Primary` | `Color` | Accent of the checked and indeterminate states. |
| `unchecked_color(c)` | `Color` | `None` | `UncheckedColor` | Outline colour of the unchecked state. |
| `size(s)` | `Size` | `Medium` | `Size` | Icon side: 18 / 24 / 32 pt. |
| `ripple(b)` | `bool` | `true` | `Ripple` | Click ripple. |
| `disabled(b)` | `bool` | `false` | `Disabled` | Dims and disables. |
| `read_only(b)` | `bool` | `false` | `ReadOnly` | Disables without dimming. |
| `tri_state(b)` | `bool` | `false` | `TriState` | Enables the indeterminate state. |
| `keyboard_enabled(b)` | `bool` | `true` | `KeyboardEnabled` | Keyboard control. |
| `checked_icon(s)` | `impl Into<String>` | `CheckBox` | `CheckedIcon` | Checked glyph. |
| `unchecked_icon(s)` | `impl Into<String>` | `CheckBoxOutlineBlank` | `UncheckedIcon` | Unchecked glyph. |
| `indeterminate_icon(s)` | `impl Into<String>` | `IndeterminateCheckBox` | `IndeterminateIcon` | Indeterminate glyph. |
| `classes(iter)` | `iter of AsRef<str>` | none | `Class` | Stored on `ComponentBase`. |
| `tag(t)` / `tag_ref::<U>()` | `T: Any + Send + Sync` | `None` | `Tag` | Arbitrary user data. |
| `show(ui)` | `&mut Ui` | — | — | Returns `CheckBoxResponse`. |

### `CheckBoxResponse`

| Field | Type | Description |
|---|---|---|
| `response` | `Response` | The whole row (icon + label). Deref target. |
| `changed` | `bool` | `true` on the frame the value changed. |

`CheckBoxResponse` implements `Deref<Target = Response>`, so `out.clicked()`,
`out.hovered()` and `out.rect` work directly.

### `CheckBoxValue`

| Method | Description |
|---|---|
| `get() -> Option<bool>` | Current value; `None` means indeterminate. |
| `set(Option<bool>)` | Replaces the value. |

Implemented for `bool` and `Option<bool>`. A custom type can implement it to
adapt its own storage.

### `Placement`

`Start` (default for `MudSwitch`, `End` for `MudCheckBox`), `End`, `Top`,
`Bottom`, plus the `Left` / `Right` aliases. `Start` / `Left` place the label
before the icon; `End` / `Right` after it. `Top` / `Bottom` stack vertically.

### `Size` mapping

| `Size` | Icon side |
|---|---|
| `ExtraSmall` | 18 pt |
| `Small` | 18 pt |
| `Medium` | 24 pt |
| `Large` | 32 pt |
| `ExtraLarge` | 32 pt |

`ExtraSmall` folds into `Small` and `ExtraLarge` into `Large`, matching the
other input components in this workspace.

### Constants

| Name | Value |
|---|---|
| `ICON_SMALL` / `ICON_MEDIUM` / `ICON_LARGE` | `18.0` / `24.0` / `32.0` |
| `GAP` / `DENSE_GAP` | `8.0` / `4.0` |
| `RIPPLE_DURATION` | `0.35` s |
| `RIPPLE_MAX_RADIUS_FACTOR` | `1.2` |
| `RIPPLE_ALPHA` | `60` |
| `DISABLED_OPACITY` | `0.38` |
| `UNCHECKED_OUTLINE_ALPHA` | `160` |

---

## Examples

### Boolean checkbox

```rust
let mut accepted = false;
ui.add(
    MudCheckBox::new(&mut accepted)
        .label("I agree to the terms")
        .color(Color::Primary),
);
```

### Tri-state checkbox

```rust
let mut state: Option<bool> = None; // indeterminate

ui.add(
    MudCheckBox::new(&mut state)
        .tri_state(true)
        .label("Select all")
        .indeterminate_icon(filled::INDETERMINATE_CHECK_BOX),
);

match state {
    None => log::info!("some children selected"),
    Some(true) => log::info!("all selected"),
    Some(false) => log::info!("none selected"),
}
```

### Label before the icon

```rust
MudCheckBox::new(&mut checked)
    .label("Remember me")
    .label_placement(Placement::Start)
    .show(ui);
```

### Vertical label

```rust
MudCheckBox::new(&mut checked)
    .label("Above")
    .label_placement(Placement::Top)
    .show(ui);

MudCheckBox::new(&mut checked)
    .label("Below")
    .label_placement(Placement::Bottom)
    .show(ui);
```

### Dense row in a table toolbar

```rust
MudCheckBox::new(&mut self.select_all)
    .label("Select all")
    .dense(true)
    .size(Size::Small)
    .show(ui);
```

### Read-only

```rust
MudCheckBox::new(&mut checked)
    .label("Locked value")
    .read_only(true)
    .show(ui);
```

The value cannot be changed, but the checkbox is not dimmed.

### Disabled

```rust
MudCheckBox::new(&mut checked)
    .label("Unavailable")
    .disabled(true)
    .show(ui);
```

The checkbox is dimmed to 38 % opacity and ignores clicks and keys.

### Custom glyphs (star checkbox)

```rust
MudCheckBox::new(&mut starred)
    .checked_icon(filled::STAR)
    .unchecked_icon(filled::STAR_BORDER)
    .color(Color::Warning)
    .show(ui);
```

### Unchecked colour

```rust
MudCheckBox::new(&mut checked)
    .unchecked_color(Color::Error)
    .show(ui);
```

The outline of the unchecked box is red; the checked fill is still `Primary`.

### Reacting to a change

```rust
let out = MudCheckBox::new(&mut self.accepted)
    .label("Accept")
    .show(ui);
if out.changed {
    log::info!("accepted = {}", self.accepted);
}
```

### Keyboard-only form

```rust
// With `keyboard_enabled(true)` (the default), Tab to the checkbox and use:
//   Space     cycle
//   Enter     set true
//   Delete    set false
//   Backspace set indeterminate (only with tri_state)
```

### Tag round-trip

```rust
let cb = MudCheckBox::new(&mut checked).tag("terms");
assert_eq!(cb.tag_ref::<&str>(), Some(&"terms"));
```

---

## Binding model

`MudCheckBox` is generic over [`CheckBoxValue`]:

```rust
impl CheckBoxValue for bool       // boolean mode
impl CheckBoxValue for Option<bool> // tri-state mode
```

The caller owns the value and passes `&mut T` to `new`:

```rust
let mut a = false;          // bool
let mut b: Option<bool> = None; // Option<bool>

MudCheckBox::new(&mut a).show(ui);
MudCheckBox::new(&mut b).tri_state(true).show(ui);
```

`ValueChanged` is not a callback; it is `CheckBoxResponse::changed`, a field on
the frame’s result. This is the same split `MudAlert` and `MudCollapse` use.

### Tri-state cycle

| Current | Click / `Space` |
|---|---|
| `None` | `Some(true)` |
| `Some(true)` | `Some(false)` |
| `Some(false)` | `None` |

With `tri_state(false)`, the value toggles between `true` and `false`; a `None`
binding becomes `Some(true)` on the first click.

### Keyboard mapping

| Key | Result |
|---|---|
| `Space` | Same as a click |
| `Enter` | `Some(true)` |
| `Delete` | `Some(false)` |
| `Backspace` | `None` (only when `tri_state`) |

---

## Layout and rendering

### Horizontal

```
End:    [ ✓ ][  8 pt  ][ Label … ]
Start:  [ Label … ][ 8 pt ][ ✓ ]
```

- Icon side: 18 / 24 / 32 pt.
- Row height: `max(icon_side, label_height)`.
- Gap: 8 pt, or 4 pt when `dense`.

### Vertical

```
Top:    [ Label … ]
        [  4/8 pt ]
        [ ✓ ]

Bottom: [ ✓ ]
        [  4/8 pt ]
        [ Label … ]
```

The row width is `max(icon_side, label_width)`; the height is
`icon_side + gap + label_height`.

### Paint

| State | Glyph | Colour |
|---|---|---|
| Checked | `CheckedIcon` or `CHECK_BOX` | `Color` |
| Unchecked | `UncheckedIcon` or `CHECK_BOX_OUTLINE_BLANK` | `UncheckedColor`, or `Color` at 160 alpha |
| Indeterminate | `IndeterminateIcon` or `INDETERMINATE_CHECK_BOX` | `Color` |

Disabled checkboxes are gamma-multiplied by `DISABLED_OPACITY` (0.38).
`read_only(true)` does **not** dim; it only blocks interaction.

### Ripple

A filled circle of radius `icon_side × 0.5 × 1.2` expands from the icon centre
over `RIPPLE_DURATION` (0.35 s), then fades out. The start time is stored in
`egui::Memory` keyed by the row’s `Response::id`. A repaint is requested only
while the ripple is in flight.

---

## Performance and memory choices

| Choice | Effect |
|---|---|
| Theme fetched once per frame | One `MudTheme::fetch` and up to two `Color::resolve` calls. |
| Pure helpers on `Copy` values | Size, cycle, layout, state and keyboard are arithmetic; unit-testable without a `Context`. |
| No `Vec` or `String` allocation | The label is measured with `layout_no_wrap` and moved into `MudText`; no per-frame clone. |
| One `allocate_exact_size` | The whole row is one rect and one `Response`. |
| Ripple in `egui::Memory` | No state on the component; the builder stays stateless. |
| `bool::then` / `Option::filter` | No `Option<Option<_>>`, no branches in the paint path. |
| Keyboard only read when focused | `apply_keys` runs only when `keyboard_enabled && enabled && has_focus`. |
| Disabled path skips click and keys | `Sense::hover()` for disabled and read-only; no interaction registration. |

---

## Testing

`cargo test -p mud_egui_components mud_check_box` runs 14 pure-function tests:

| Area | Tests |
|---|---|
| Icon size | every `Size`; edge folding |
| Boolean cycle | toggle, `None → Some(true)` |
| Tri-state cycle | full `None → true → false → None` loop; one cycle returns to indeterminate |
| Placement | `is_vertical` and `label_leads` for every variant |
| Layout | `layout_size` horizontal / vertical / no label; `icon_rect` and `label_rect` follow placement |
| Paint state | boolean and tri-state mapping |
| Glyph | override wins; default is non-empty |
| `CheckBoxValue` | `bool` and `Option<bool>` round-trip |
| Builders | defaults match MudBlazor; every field set; tag round-trip |

No test creates an `egui::Context`. `show` needs a live `Ui` (layout, painting,
theme lookup, input) and is therefore left to integration tests; everything it
delegates to is covered above.

Suggested integration tests (`egui_kittest` or a headless `Context::run`
harness):

- Clicking toggles a boolean checkbox and sets `changed`.
- Clicking a tri-state checkbox cycles `None → true → false → None`.
- `read_only(true)` blocks clicks without dimming.
- `disabled(true)` blocks clicks and paints at 38 % opacity.
- `Space` toggles a focused checkbox; `Enter` / `Delete` / `Backspace` set
  `true` / `false` / `None` (the last only with `tri_state`).
- The ripple starts on click and requests a repaint for at most 0.35 s.

```bash
cargo test -p mud_egui_components mud_check_box
```

---

## Differences from MudBlazor

- **`Value` / `ValueChanged` become a caller-owned `&mut T` plus
  `CheckBoxResponse::changed`.** The caller owns the value; `changed` is
  `true` on the frame it moved. This is the same split `MudAlert` and
  `MudCollapse` use.
- **`CheckBoxValue` is a new public trait.** It is the seam that lets one
  component serve `bool` and `Option<bool>`. MudBlazor achieves the same with
  its generic `MudCheckBox<T>` and `BoolConverter<T>`; a trait is the idiomatic
  Rust equivalent.
- **`Class` is stored but not resolved.** The checkbox’s colour comes from
  `Color` and `UncheckedColor`; a utility-class frame would composite a second
  fill.
- **`Style` and `UserAttributes` are not ported.** No CSS strings, no HTML tag.
- **Form-validation parameters are not ported.** `Required`, `Error`,
  `ErrorText`, `HelperText`, `ConversionError`, `Strict`, `Immediate` and
  `DebounceInterval` belong to `MudFormComponent` / `MudBaseInput`. Without a
  form system in egui there is nothing for them to drive.
- **`Typo` is not a parameter.** 8.13.0’s `MudCheckBox` does not expose one;
  the label uses the theme’s `Body1`. A `Typo` builder would be API invention.
- **`LabelPlacement` uses the existing `Placement` enum.** `Start` / `Left`
  place the label before the icon; `End` / `Right` after. `Top` / `Bottom`
  stack vertically. The default is `End`, matching MudBlazor.
- **`Size` folds `ExtraSmall` into `Small` and `ExtraLarge` into `Large`.**
  Consistent with `MudSwitch` and `MudSlider`.
- **The ripple is memory-backed.** Start time lives in `egui::Memory`; the
  component itself is stateless between frames.
- **The default `Color` is `Primary`, not `Default`.** MudBlazor’s
  `MudBooleanInput` default is `Default`, which resolves to `Primary` for
  this component. Pinning it to `Primary` makes the default visible in the
  builder table; a caller can still set `Color::Default` explicitly if a
  theme maps it to something else.

===== ./crates/mud_egui_components/src/inputs/file_upload/README.md =====

# MudFileUpload for `mud_egui`

A file picker and drag-and-drop surface for
[egui](https://github.com/emilk/egui), ported from MudBlazor’s
[`MudFileUpload<T>`](https://mudblazor.com/components/fileupload) (MudBlazor 8.13.0).

```rust
let mut files: Vec<PathBuf> = Vec::new();

ui.add(
    MudFileUpload::new(&mut files)
        .accept(".png, .jpg")
        .max_file_size(2 * 1024 * 1024)
        .label("Upload images"),
);

if !files.is_empty() {
    // files is the caller-owned list.
}
```

The component itself paints no text through `MudText` (it draws its own button
label and chips), but any `activator_content` you supply should use
**`MudText`** and **`MudIcon`** like every other component.

---

## Contents

1. [Design principle: API parity, not API invention](#design-principle-api-parity-not-api-invention)
2. [Architecture](#architecture)
3. [API reference](#api-reference)
4. [Examples](#examples)
5. [Mode semantics](#mode-semantics)
6. [Validation](#validation)
7. [Performance and memory choices](#performance-and-memory-choices)
8. [Testing](#testing)
9. [Differences from MudBlazor](#differences-from-mudblazor)

---

## Design principle: API parity, not API invention

The MudBlazor 8.13.0 parameter surface was verified against DNDocs before
implementation. `MudFileUpload<T>` has **ten parameters**, and all ten are
ported.

### Ported parameters

| MudBlazor | Type | Default | Rust mapping |
|---|---|---|---|
| `Accept` | `string?` | `null` | `accept(&'a str)` |
| `ActivatorContent` | `RenderFragment?` | `null` | `activator_content(FnOnce(&mut Ui) -> Response)` |
| `AppendMultipleFiles` | `bool` | `false` | `append_multiple_files(bool)` |
| `Disabled` | `bool` | `false` | `disabled(bool)` |
| `Files` | `T?` | — | caller-owned `&mut Vec<PathBuf>` |
| `FilesChanged` | `EventCallback<T?>` | — | `FileUploadResponse::changed` |
| `Hidden` | `bool` | **`true`** | `hidden(bool)` |
| `MaxFileSize` | `long?` | `null` | `max_file_size(u64)` → `Option<u64>` |
| `MaximumFileCount` | `int` | **`10`** | `maximum_file_count(usize)` |
| `Class` | `string?` | `null` | `classes(iter)` — stored, not resolved |
| `Tag` | `object?` | `null` | `tag(t)` / `tag_ref::<U>()` |

### Deliberately not ported

| MudBlazor | Reason |
|---|---|
| `InputClass` / `InputStyle` | egui has no inline CSS strings. The internal `InputFile` element has no egui analogue. |
| `Style` / `UserAttributes` | No CSS strings, no HTML tag. |
| `Class` resolution | Stored on `base` for parity only. The component’s own button chrome comes from `Variant` / `Color` / `Size`; a utility-class frame would composite a second fill. |
| Form-validation members (`Required`, `Error`, `ErrorText`, `HelperText`, `ConversionError`, `Strict`, `Immediate`, `DebounceInterval`) | Inherited from `MudFormComponent`; no egui form system. |

### `Hidden` is the drag-and-drop switch

MudBlazor’s docs: *“Defaults to `true`. When `false`, files can be uploaded via
drag-and-drop.”*

| `hidden` | Mode |
|---|---|
| `true` (default) | Button only. Clicking opens the OS file picker. |
| `false` | Button **and** a dashed drop zone. Files can be dropped on the component. |

There is no separate `drag_and_drop` flag in MudBlazor; the pre-existing Rust
port added one, and it is removed here.

### `MaximumFileCount` is per-pick, not per-list

The docs: *“The maximum number of files retrieved during a call to
`GetMultipleFiles`. This property does not limit the total number of uploaded
files allowed; a limit should be validated manually.”* `filter_files` truncates
the picked batch; `apply_selection` does not enforce a total cap.

---

## Architecture

```
inputs/file_upload/
└── mud_file_upload.rs
    ├── constants        MIN_BUTTON_WIDTH 200, DROP_ZONE_HEIGHT 180,
    │                    CORNER_RADIUS 4, ANIM_SECONDS 0.12,
    │                    DASH_LEN 8, DASH_GAP 6
    ├── SizeMetrics      height / font_size / padding_y per Size
    ├── pure helpers     parse_extensions, matches_accept, filter_files,
    │                    apply_selection, lerp_color, contrast_text,
    │                    radius_u8
    ├── rendering        draw_button, draw_drop_zone, draw_chip,
    │                    draw_dashed_rect, draw_dashed_line
    ├── pick_files       rfd::FileDialog (blocking)
    ├── MudFileUpload    builder + show
    └── impl Widget      ui.add(MudFileUpload::new(&mut files))
```

**Pure layer.** Extension parsing, accept matching, filtering, selection
application, colour lerp and contrast are free functions over `&Path`,
`&[String]` and `Option<u64>`. None touch `egui::Context`.

**Rendering layer.** `draw_button` and `draw_drop_zone` build the two surfaces;
`draw_chip` builds the removable file pills; `draw_dashed_rect` builds the drop
border.

**Picker layer.** `pick_files` is the only OS call. It is synchronous and
blocking; the README documents the async alternative.

---

## API reference

| Method | Type | Default | Mirrors | Description |
|---|---|---|---|---|
| `new(files)` | `&'a mut Vec<PathBuf>` | — | `Files` | Binds the component to a file list. |
| `single()` | — | — | `T = IBrowserFile` | Keeps only one file; replaces on pick. |
| `accept(s)` | `&'a str` | `None` | `Accept` | Comma-separated extensions. |
| `append_multiple_files(b)` | `bool` | `false` | `AppendMultipleFiles` | Append instead of replace. |
| `disabled(b)` | `bool` | `false` | `Disabled` | Disables interaction. |
| `hidden(b)` | `bool` | `true` | `Hidden` | `false` enables the drop zone. |
| `max_file_size(bytes)` | `u64` | `None` | `MaxFileSize` | Per-file byte cap. |
| `maximum_file_count(n)` | `usize` | `10` | `MaximumFileCount` | Per-pick cap; clamped to ≥ 1. |
| `variant(v)` | `Variant` | `Filled` | *(button chrome)* | Button variant. |
| `color(c)` | `Color` | `Primary` | *(button chrome)* | Accent colour. |
| `size(s)` | `Size` | `Medium` | *(button chrome)* | Button height and font. |
| `elevation(e)` | `u8` | `0` | *(button chrome)* | Shadow depth, clamped to 24. |
| `corner_radius(r)` | `f32` | `4.0` | *(button chrome)* | Corner radius. |
| `label(s)` | `&'a str` | `"Browse files"` | *(ActivatorContent fallback)* | Button label. |
| `aria_label(s)` | `&'a str` | `None` | *(extension)* | Accessible label. |
| `activator_content(f)` | `FnOnce(&mut Ui) -> Response` | `None` | `ActivatorContent` | Custom button content. |
| `classes(iter)` | `iter of AsRef<str>` | none | `Class` | Stored on `ComponentBase`. |
| `tag(t)` / `tag_ref::<U>()` | `T: Any + Send + Sync` | `None` | `Tag` | Arbitrary user data. |
| `show(ui)` | `&mut Ui` | — | — | Returns `FileUploadResponse`. |

### `FileUploadResponse`

| Field | Type | Description |
|---|---|---|
| `response` | `Response` | The component’s `Response`. Deref target. |
| `changed` | `bool` | `true` on the frame the file list changed. |

`FileUploadResponse` implements `Deref<Target = Response>`, so `out.clicked()`,
`out.hovered()` and `out.rect` work directly.

### `Size` mapping

| `Size` | Height | Font |
|---|---|---|
| `ExtraSmall` / `Small` | 28 pt | 12 pt |
| `Medium` | 36 pt | 14 pt |
| `Large` / `ExtraLarge` | 48 pt | 16 pt |

### Constants

| Name | Value |
|---|---|
| `MIN_BUTTON_WIDTH` | `200.0` |
| `DROP_ZONE_HEIGHT` | `180.0` |
| `CORNER_RADIUS` | `4.0` |
| `ANIM_SECONDS` | `0.12` |
| `DASH_LEN` / `DASH_GAP` | `8.0` / `6.0` |

---

## Examples

### Basic button

```rust
let mut files = Vec::new();
MudFileUpload::new(&mut files)
    .label("Choose a file")
    .show(ui);
```

### Accepted types and size cap

```rust
MudFileUpload::new(&mut files)
    .accept(".png, .jpg, .jpeg")
    .max_file_size(2 * 1024 * 1024)
    .label("Upload images")
    .show(ui);
```

### Drag-and-drop zone

```rust
MudFileUpload::new(&mut files)
    .hidden(false)          // enables the drop zone
    .label("Drop files here")
    .show(ui);
```

### Append instead of replace

```rust
MudFileUpload::new(&mut files)
    .append_multiple_files(true)
    .label("Add more")
    .show(ui);
```

### Single file

```rust
MudFileUpload::new(&mut files)
    .single()
    .accept(".pdf")
    .label("Choose a PDF")
    .show(ui);
```

### Custom activator

```rust
MudFileUpload::new(&mut files)
    .activator_content(|ui| {
        ui.add(MudButton::new("Upload").start_icon(filled::UPLOAD))
    })
    .show(ui);
```

### Reacting to a change

```rust
let out = MudFileUpload::new(&mut files).show(ui);
if out.changed {
    log::info!("{} file(s) selected", files.len());
}
```

### Chip list with removable files

```rust
// After `show`, `files` is the current list. The component draws a chip per
// file with a × button; clicking it removes the entry and sets `changed`.
```

### Tag round-trip

```rust
let upload = MudFileUpload::new(&mut files).tag("avatar");
assert_eq!(upload.tag_ref::<&str>(), Some(&"avatar"));
```

---

## Mode semantics

### Button mode (`hidden = true`, default)

```
┌──────────────────────────────┐
│        Browse files          │   ← button, 36 pt tall (Medium)
└──────────────────────────────┘
   ↓ click
   rfd::FileDialog → filtered → `files`
```

- Width: `available_width().min(200).max(80)`.
- Height: `SizeMetrics.height` (28 / 36 / 48 pt).
- Fill: `Variant::Filled` uses the accent; `Outlined` and `Text` are transparent
  with an accent border or label.

### Drag-and-drop mode (`hidden = false`)

```
┌ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ┐
│   Drag files here or click     │   ← dashed border, 180 pt tall (Medium)
│   to browse                    │
└ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ┘
```

- The dashed border animates from grey to the accent colour while
  `i.raw.hovered_files` is non-empty.
- Dropping outside the component still fires egui’s window-wide drop event, but
  the component only accepts it when `response.contains_pointer()` is true.
- Clicking still opens the picker.

### Chips

When `hidden = true` and `files` is non-empty, the component draws one chip per
file under the button. Each chip is a two-button group: the filename and a ×.
Clicking × removes that entry from `files` and sets `changed`.

---

## Validation

`filter_files` runs three checks in order, and stops at `maximum_file_count`:

1. **Extension** — `matches_accept` compares the lowercased extension against
   the parsed `accept` list. An empty list accepts everything.
2. **Size** — if `max_file_size` is set, `std::fs::metadata(path).len()` is
   compared against it. A file that cannot be read has size `0` and passes.
3. **Limit** — the batch is truncated to `maximum_file_count`.

Files that fail are silently dropped; the returned error is always `None`,
because MudBlazor surfaces per-file failures through form validation rather than
a single error string. If you need to show the user which files were rejected,
check `files.len()` after `show` against the number of files the user picked.

---

## Performance and memory choices

| Choice | Effect |
|---|---|
| Theme fetched once per frame | One `MudTheme::fetch` and one `Color::resolve`. |
| `accept` parsed once per frame | An empty list is an early return; a non-empty list is a small `Vec<String>`. |
| Pure validators on `&Path` | Extension and size checks are allocations only when the extension is lowercased. |
| `animate_bool_with_time` | Hover and focus transitions are idle when nothing changes. |
| One `allocate_exact_size` per surface | The button and drop zone are each one rect. |
| Chips are built only when `files` is non-empty **and** `hidden` | A drop-zone upload does not pay for a chip list it cannot show. |
| `rfd::FileDialog` is synchronous | One OS call per click; no background thread or channel. See the note below. |

### The blocking file dialog

`pick_files` calls `rfd::FileDialog::pick_files()`, which blocks the UI thread
until the user closes the dialog. For a desktop app this is usually acceptable —
the dialog is modal to the window anyway — but it means `show` cannot be called
from a frame that also animates. If you need a non-blocking picker, spawn
`rfd::AsyncFileDialog` on a worker thread and feed the result back through a
channel:

```rust
let (tx, rx) = std::sync::mpsc::channel();
std::thread::spawn(move || {
    let picked = rfd::AsyncFileDialog::new().pick_files();
    let _ = tx.send(picked);
});
// On a later frame: if let Ok(Some(handles)) = rx.try_recv() { … }
```

The component itself does not do this because the channel would have to live
somewhere, and that somewhere is the caller’s state. The README documents the
pattern rather than hiding it inside the widget.

---

## Testing

`cargo test -p mud_egui_components mud_file_upload` runs 9 pure-function tests:

| Area | Tests |
|---|---|
| `parse_extensions` | comma-separated, case folding, non-dot filtering, `None` |
| `matches_accept` | case-insensitive match, extensionless files, empty list |
| `filter_files` | extension filtering, batch truncation, size cap |
| `apply_selection` | replace vs append, single keeps only the first |
| `SizeMetrics` | edge sizes fold into `Small` / `Large` |
| Builders | defaults (hidden **true**, max count **10**), every field set, `maximum_file_count(0)` clamps to 1, tag round-trip |

No test creates an `egui::Context`. `show`, `draw_button`, `draw_drop_zone`,
`draw_chip` and `pick_files` need a live `Ui` or a real file dialog and are left
to integration tests; everything they delegate to is covered above.

Suggested integration tests (`egui_kittest` or a headless `Context::run`
harness with a fake picker):

- Clicking the button in `hidden = true` mode opens the picker and updates
  `files`.
- Dropping a file on the component in `hidden = false` mode updates `files`.
- Dropping a file outside the component does **not** update `files`.
- `single()` keeps only the first picked file.
- `append_multiple_files(true)` extends the list.
- The chip × removes the corresponding entry.

```bash
cargo test -p mud_egui_components mud_file_upload
```

---

## Differences from MudBlazor

- **`Files` / `FilesChanged` become a `&mut Vec<PathBuf>` plus
  `FileUploadResponse::changed`.** The caller owns the list; `changed` is `true`
  on the frame it moved. This is the same split `MudAlert` and `MudCollapse`
  use.
- **`Hidden` is the drag-and-drop switch, with no separate flag.** Your
  pre-existing port added `drag_and_drop(bool)`; it is removed because MudBlazor
  has no such parameter. Use `.hidden(false)` to enable the drop zone.
- **`MaximumFileCount` defaults to `10`, not `100`.** The DNDocs page is
  explicit; the pre-existing default of `100` was invented. It also caps only
  one picker call, not the total list.
- **The picker is synchronous.** `rfd::FileDialog` blocks the UI thread. The
  README documents the `AsyncFileDialog` + channel alternative; the component
  does not hide it.
- **`ActivatorContent` is a closure that returns a `Response`.** This is the
  direct analogue of a `RenderFragment` that renders the clickable activator.
- **`InputClass` / `InputStyle` are not ported.** egui has no CSS strings.
- **`Style` / `UserAttributes` are not ported.** No CSS strings, no HTML tag.
- **The chip list is an extension.** MudBlazor shows files through
  `SelectedTemplate` (a render fragment), which is a `ChildContent`-style slot.
  The port draws a default chip row so the common case works without a template;
  if you need a custom row, read `files` after `show` and draw your own.
- **The button chrome is an extension.** MudBlazor’s `MudFileUpload` wraps a
  `MudButton` through `ActivatorContent`; the port exposes `Variant`, `Color`,
  `Size`, `Elevation` and `CornerRadius` so a default button exists without a
  separate `MudButton` call. Every one is marked as button chrome in the API
  table, not as a MudBlazor parameter.
- **The drop-zone height and dash pattern are constants.** MudBlazor’s SCSS
  uses the same values; they are named at the top of the file so they can be
  adjusted in one place.

===== ./crates/mud_egui_components/src/inputs/hotkey/README.md =====

# MudHotkey for `mud_egui`

A hotkey listener that reveals or hides its child content for
[egui](https://github.com/emilk/egui).

> **Version note.** `MudHotkey` was introduced in **MudBlazor 9.0.0**, not in
> 8.13.0. The rest of this workspace targets 8.13.0; this component is the one
> deliberate version exception, documented here so the README's parity table
> cites 9.0.0 rather than 8.13.0.

```rust
let mut open_search = false;

MudHotkey::new(Key::K)
    .key_modifiers(Modifiers::CTRL)
    .on_hotkey_pressed(|_| open_search = true)
    .show(ui, |ui| {
        ui.add(MudText::body1("Search panel"));
    });
```

The content closure supplies every string and glyph; `MudHotkey` paints none of
its own. Route them through **`MudText`** and **`MudIcon`** as usual.

---

## Contents

1. [Design principle: API parity, not API invention](#design-principle-api-parity-not-api-invention)
2. [Architecture](#architecture)
3. [API reference](#api-reference)
4. [Examples](#examples)
5. [How the hotkey is detected](#how-the-hotkey-is-detected)
6. [Performance and memory choices](#performance-and-memory-choices)
7. [Testing](#testing)
8. [Differences from MudBlazor](#differences-from-mudblazor)

---

## Design principle: API parity, not API invention

The **9.0.0** parameter surface was verified against the MudBlazor 9 docs and
the v9.6.0 / v9.5.0 release notes before implementation. `MudHotkey` has
**seven parameters**, and all seven are accounted for.

### Ported parameters

| MudBlazor 9.0.0 | Type | Default | Rust mapping |
|---|---|---|---|
| `Key` | `JsKey` | — | `new(Key)` |
| `KeyModifiers` | `IReadOnlyCollection<JsKeyModifier>` | empty | `key_modifiers(Modifiers)` |
| `OnHotkeyPressed` | `EventCallback<HotkeyEventArgs>` | — | `on_hotkey_pressed(f)` + `HotkeyResponse::pressed` |
| `Disabled` | `bool` | `false` | `disabled(bool)` |
| `PreventEventPropagation` | `bool` | `false` | `prevent_event_propagation(bool)` |
| `HideChildContentOnRepress` | `bool` | `true` | `hide_child_content_on_repress(bool)` |
| `ChildContent` | `RenderFragment?` | `null` | the closure passed to `show(ui, \|ui\| ..)` |
| `Class` | `string?` | `null` | `classes(iter)` — stored, not resolved |
| `Tag` | `object?` | `null` | `tag(t)` / `tag_ref::<U>()` |

### Deliberately not ported

| MudBlazor | Reason |
|---|---|
| `Style` | egui has no inline CSS strings. `MudHotkey` renders no element of its own. |
| `UserAttributes` | There is no HTML tag to splat attributes onto. |
| `Class` resolution | Stored on `base` for parity. `MudHotkey` adds no wrapper frame. |

### Substitutions

| MudBlazor | Rust | Reason |
|---|---|---|
| `JsKey` | `egui::Key` | `JsKey` covers every browser `KeyboardEvent.code`; egui's `Key` is a smaller, native enum. Common keys (`KeyA`..`KeyZ`, `Digit0`..`Digit9`, arrows, `Enter`, `Tab`, `Escape`) map 1:1; browser-only codes (`NumpadEqual`, `IntlBackslash`) have no egui equivalent. |
| `JsKeyModifier` | `egui::Modifiers` | `JsKeyModifier` distinguishes left and right (`ControlLeft` vs `ControlRight`); `egui::Modifiers` is a set of flags (`ctrl`, `shift`, `alt`, `mac_cmd`, `command`) without a side. The distinction is the subject of an open MudBlazor issue (#13390, "XxxRight modifiers do not work"); egui sidesteps it entirely. |

### Extensions

| Extension | Type | Default | Purpose |
|---|---|---|---|
| `id_salt(impl Hash + Debug)` | `Id` | `None` | Distinguishes two `MudHotkey`s in one parent, whose `visible` toggles would otherwise share a memory key. Same shape as `MudCollapse::id_salt`. |
| `with_modifiers(key, modifiers)` | — | — | Convenience constructor: `MudHotkey::new(key).key_modifiers(m)`. |

---

## Architecture

```
layout/hotkey/
├── mud_hotkey.rs
│   ├── MudHotkey           builder + show(ui, add_content) + Widget impl
│   ├── HotkeyEventArgs     { key, repeat }
│   ├── HotkeyResponse<R>   { response, pressed, repeat, visible, inner }
│   └── detect_press        pure — scans `i.events` for a matching Key event
└── mod.rs                  re-exports
```

**Pure layer.** `detect_press(ui, key, modifiers)` scans egui's per-frame
`Event::Key` list once and reports `(pressed, repeat)`. It is the only place
the matching logic lives.

**State layer.** One `bool` per instance lives in `egui::Memory`, keyed by
`<ui.id().with(id_salt)>.hotkey_visible`. It exists only for the
`HideChildContentOnRepress` toggle.

**Component layer.** `show` tests the hotkey, fires the callback, optionally
consumes the key, and either runs the content closure or allocates nothing.

---

## API reference

### `MudHotkey`

| Method | Type | Default | Mirrors | Description |
|---|---|---|---|---|
| `new(key)` | `Key` | `Space` (via `Default`) | `Key` | Creates a listener for `key` with no modifiers. |
| `with_modifiers(key, m)` | `(Key, Modifiers)` | — | — | Convenience constructor. |
| `key_modifiers(m)` | `Modifiers` | `Modifiers::NONE` | `KeyModifiers` | Required modifier set. |
| `on_hotkey_pressed(f)` | `FnMut(&HotkeyEventArgs)` | `None` | `OnHotkeyPressed` | Callback fired on every press. |
| `disabled(b)` | `bool` | `false` | `Disabled` | Disables the hotkey; content stays visible. |
| `prevent_event_propagation(b)` | `bool` | `false` | `PreventEventPropagation` | Consumes the key so other widgets do not see it. |
| `hide_child_content_on_repress(b)` | `bool` | `true` | `HideChildContentOnRepress` | Toggles content visibility on each press. |
| `id_salt(s)` | `impl Hash + Debug` | `None` | *(extension)* | Distinguish two instances in one parent. |
| `classes(iter)` | `iter of AsRef<str>` | none | `Class` | Stored on `ComponentBase`. |
| `tag(t)` / `tag_ref::<U>()` | `T: Any + Send + Sync` | `None` | `Tag` | Arbitrary user data. |
| `show(ui, \|ui\| ..)` | `&mut Ui, FnOnce(&mut Ui) -> R` | — | `ChildContent` | Returns `HotkeyResponse<R>`. |
| `ui.add(..)` | — | — | — | Tests the hotkey but draws no content. |

### `HotkeyResponse<R>`

| Field | Type | Description |
|---|---|---|
| `response` | `Response` | The content's response when visible; a zero-sized hover response when hidden. Deref target. |
| `pressed` | `bool` | `true` on the frame the hotkey was pressed. Mirrors the trigger for `OnHotkeyPressed`. |
| `repeat` | `bool` | `true` when `pressed` is a repeat from a held key. Mirrors `HotkeyEventArgs.OnRepeat`. |
| `visible` | `bool` | Current content visibility. |
| `inner` | `Option<R>` | The content closure's value; `None` when hidden. |

### `HotkeyEventArgs`

| Field | Type | Description |
|---|---|---|
| `key` | `Key` | The key that completed the combination. |
| `repeat` | `bool` | `true` on a repeat from a held key. |

### `egui::Modifiers` cheat sheet

`Modifiers` is a struct of `bool` fields; `Modifiers::NONE` is "no modifier".
Common combinations are built from the constants:

| Constant | Value |
|---|---|
| `Modifiers::NONE` | no modifiers |
| `Modifiers::CTRL` | `ctrl: true` |
| `Modifiers::SHIFT` | `shift: true` |
| `Modifiers::ALT` | `alt: true` |
| `Modifiers::MAC_CMD` | `mac_cmd: true` |
| `Modifiers::COMMAND` | `command: true` (Ctrl on non-Mac, Cmd on Mac) |

For more than one, `Modifiers { ctrl: true, shift: true, ..Default::default() }`.

---

## Examples

### Ctrl+K opens a search panel

```rust
MudHotkey::with_modifiers(Key::K, Modifiers::CTRL)
    .on_hotkey_pressed(|_| self.open_search = true)
    .show(ui, |ui| {
        ui.add(MudText::body1("Search panel"));
    });
```

### Ctrl+S saves, and swallows the key

```rust
MudHotkey::with_modifiers(Key::S, Modifiers::CTRL)
    .prevent_event_propagation(true)
    .on_hotkey_pressed(|_| self.save())
    .show(ui, |_| {});
```

`prevent_event_propagation(true)` calls `Context::consume_key`, so a text
field in the same frame does not also see the `S`. This is the closest egui
equivalent to MudBlazor's `PreventEventPropagation`.

### Hide the panel on Ctrl+H, reveal it again on the next press

```rust
MudHotkey::with_modifiers(Key::H, Modifiers::CTRL)
    .show(ui, |ui| {
        ui.add(MudText::body1("Help panel"));
    });
```

Default `hide_child_content_on_repress(true)` means: the content starts
visible, the first `Ctrl+H` hides it, the next `Ctrl+H` shows it again.

### Callback only, never hide the content

```rust
MudHotkey::with_modifiers(Key::F5, Modifiers::NONE)
    .hide_child_content_on_repress(false)
    .on_hotkey_pressed(|_| self.reload())
    .show(ui, |_| {});
```

With `hide_child_content_on_repress(false)` the content stays visible and the
callback still fires. Pass an empty closure if you have nothing to draw.

### Reading the press directly

```rust
let out = MudHotkey::with_modifiers(Key::Escape, Modifiers::NONE)
    .show(ui, |ui| { ui.label("Press Esc to close"); });
if out.pressed {
    self.dialog_open = false;
}
```

`out.pressed` is `true` for exactly one frame per press, mirroring
`OnHotkeyPressed`.

### Disabled

```rust
MudHotkey::with_modifiers(Key::K, Modifiers::CTRL)
    .disabled(true)
    .on_hotkey_pressed(|_| self.open_search = true)
    .show(ui, |ui| { ui.label("Disabled"); });
```

`disabled(true)` skips the match entirely; the content stays visible and the
callback does not fire.

### Two hotkeys in one parent

```rust
MudHotkey::with_modifiers(Key::K, Modifiers::CTRL)
    .id_salt("search")
    .show(ui, |ui| { ui.label("Search"); });

MudHotkey::with_modifiers(Key::H, Modifiers::CTRL)
    .id_salt("help")
    .show(ui, |ui| { ui.label("Help"); });
```

Without distinct salts the two share the `visible` memory key and their toggles
would cross.

---

## How the hotkey is detected

egui exposes keyboard input two ways:

- `ui.input(|i| i.key_pressed(Key))` — a convenience for "was this key pressed
  this frame". It returns `true` for the initial press *and* for repeats, and
  it does not carry the modifier state at the moment of the press.
- `ui.input(|i| &i.events)` — the raw per-frame `Event` list, including
  `Event::Key { key, pressed, repeat, modifiers, .. }`.

`MudHotkey` uses the second form because it needs the `repeat` flag and the
modifiers that were held *at the moment of the press*. The scan is a single
`for` loop over the frame's events, so it costs nothing beyond the event
iteration egui already performs.

Matching is `key == want_key && modifiers.matches_logically(want_modifiers)`.
`matches_logically` ignores the difference between a physical and a logical
key, and it requires that every "on" bit of `want_modifiers` is set on the
event; extraneous modifiers are *not* permitted (so `Ctrl+K` does not fire on
`Ctrl+Shift+K`). If you need the looser "at least these modifiers" rule, wrap
the callback in your own check on `event.modifiers` — the component follows
MudBlazor's exact-match behaviour.

### `repeat`

`HotkeyEventArgs::repeat` is `true` for events where egui's `Event::Key`
carries `repeat: true`. That fires while the key is held, once per repeat
interval. MudBlazor's `HotkeyEventArgs.OnRepeat` is the same idea.

### `prevent_event_propagation`

`Context::consume_key(modifiers, key)` marks the key as consumed for the rest
of the frame, so a `TextEdit` (or any other widget) that checks
`i.key_pressed(key)` after `MudHotkey` runs will not see it. This is the closest
egui equivalent of `event.stopPropagation()` in the browser.

---

## Performance and memory choices

| Choice | Effect |
|---|---|
| Content closure is `FnOnce` | Runs at most once, and only when the content is visible. |
| `detect_press` scans events once | One `for` loop per frame; no per-key lookup, no allocation. |
| `repeat` carried as a flag | The callback can distinguish a held key from a fresh press without the component storing previous state. |
| One `bool` in `egui::Memory` | Only the `HideChildContentOnRepress` toggle persists. |
| Zero-sized hover response when hidden | Takes no layout space and steals no clicks. |
| No theme fetch | `MudHotkey` resolves no colours; the child content owns its own theming. |
| `id_salt` for disambiguation | Two instances in one parent do not share their toggle. |

---

## Testing

`cargo test -p mud_egui_components mud_hotkey` runs 9 unit tests:

| Area | Tests |
|---|---|
| `detect_press` | Key match with modifiers; wrong key rejected; wrong modifiers rejected; repeat reported; releases ignored |
| Builders | Defaults match MudBlazor 9.0.0 (`HideChildContentOnRepress = true`); every field set; `with_modifiers` convenience; `id_salt` stability |

The `detect_press` tests use `Context::run_ui` with a crafted `RawInput` that
carries a single `Event::Key`, so they exercise the real event-scanning path
without a live window.

`MudHotkey::show` needs a live `Ui` and is left to integration tests.

Suggested integration tests (`egui_kittest` or a headless `Context::run`
harness):

- A `Ctrl+K` `MudHotkey` fires its callback on the frame the key is pressed and
  on no other frame.
- The content toggle starts visible, hides on the first press, shows on the
  second.
- `hide_child_content_on_repress(false)` keeps the content visible across
  presses.
- `disabled(true)` never fires the callback and never hides the content.
- `prevent_event_propagation(true)` causes a `TextEdit` in the same frame not
  to receive the key.
- Two instances with distinct `id_salt`s toggle independently.

```bash
cargo test -p mud_egui_components mud_hotkey
```

---

## Differences from MudBlazor

- **This component targets MudBlazor 9.0.0, not 8.13.0.** MudHotkey did not
  exist in 8.13.0; it was added in the v9.0.0 release notes. The rest of the
  workspace targets 8.13.0. The parity table above cites 9.0.0 so the version
  exception is explicit.
- **`JsKey` → `egui::Key`.** `JsKey` covers every browser `KeyboardEvent.code`;
  egui's key enum is smaller and does not include browser-only codes
  (`NumpadEqual`, `IntlBackslash`, `Lang1`…). Every key the component needs in
  practice (`A`..`Z`, `0`..`9`, arrows, `Enter`, `Tab`, `Escape`, `F1`..`F12`)
  is present in `egui::Key`.
- **`JsKeyModifier` → `egui::Modifiers`.** egui's `Modifiers` is a set of
  flags without a left/right side. MudBlazor's own tracker has an open issue
  (#13390) about `XxxRight` modifiers not working; egui avoids the problem by
  not distinguishing them.
- **`OnHotkeyPressed` is a callback, not a two-way binding.** The component
  fires `on_hotkey_pressed` **and** reports `HotkeyResponse::pressed` (with
  `repeat`), so a caller can use either the callback or the return value. This
  is the same split `MudCollapse` and `MudRating` use.
- **`ChildContent` visibility is caller-controlled.** The `HideChildContentOnRepress`
  toggle is implemented in `egui::Memory`, keyed by the widget id, so two
  `MudHotkey`s in one parent can be given distinct `id_salt`s. MudBlazor does
  the same thing with an internal `_childContentVisible` field.
- **`PreventEventPropagation` maps to `Context::consume_key`, not
  `stopPropagation`.** egui has no DOM event bubbling to stop; `consume_key`
  is the closest equivalent and prevents later widgets in the same frame from
  seeing the key.
- **`Class` is stored but not resolved.** `MudHotkey` renders no wrapper
  element, so a class list has no surface to paint.
- **`Style` and `UserAttributes` are not ported.** No CSS strings, no HTML tag.

===== ./crates/mud_egui_components/src/inputs/pickers/mud_color_picker/README.md =====

# MudColorPicker for `mud_egui`

A text field with a popover colour picker for
[egui](https://github.com/emilk/egui), ported from MudBlazor's
[`MudColorPicker`](https://mudblazor.com/components/color-picker)
(MudBlazor 8.13.0).

```rust
let mut state = ColorPickerState::new("accent");

MudColorPicker::new()
    .label("Accent colour")
    .show_alpha(false)
    .show(ui, &mut state);

// After a pick: state.value holds the committed colour.
```

The trigger reuses **`MudTextField`**. The dropdown reuses **`MudPopover`**.
The channel controls reuse **`MudSlider`**. Every numeral and label renders
through **`MudText`**; every glyph through **`MudIcon`**.

---

## Contents

1. [Design principle: API parity, not API invention](#design-principle-api-parity-not-api-invention)
2. [Architecture](#architecture)
3. [API reference](#api-reference)
4. [Examples](#examples)
5. [Views, toolbar and interaction](#views-toolbar-and-interaction)
6. [Performance and memory choices](#performance-and-memory-choices)
7. [Testing](#testing)
8. [Differences from MudBlazor](#differences-from-mudblazor)

---

## Design principle: API parity, not API invention

The MudBlazor 8.13.0 parameter surface was verified against DNDocs before
implementation[reference:20][reference:21]. `MudColorPicker` derives from
`MudPicker<MudColor>` and exposes **twenty** own parameters.

### Own parameters

| MudBlazor | Rust mapping | Default |
|---|---|---|
| `CloseIcon` | `close_icon(impl Into<String>)` | `CLOSE` |
| `ColorPickerMode` | `color_picker_mode(ColorPickerMode)` / `ColorPickerState::mode` | `Rgb` |
| `ColorPickerView` | `color_picker_view(ColorPickerView)` / `ColorPickerState::view` | `Spectrum` |
| `DragEffect` | `drag_effect(bool)` | **`true`** |
| `GridIcon` | `grid_icon(impl Into<String>)` | `APPS` |
| `ImportExportIcon` | `import_export_icon(impl Into<String>)` | `IMPORT_EXPORT` |
| `Palette` | `palette(iter of ColorChannels)` | 35 entries |
| `PaletteIcon` | `palette_icon(impl Into<String>)` | `PALETTE` |
| `RightToLeft` | `right_to_left(bool)` | `false` |
| `ShowAlpha` | `show_alpha(bool)` | **`true`** |
| `ShowColorField` | `show_color_field(bool)` | **`true`** |
| `ShowInputs` | `show_inputs(bool)` | **`true`** |
| `ShowModeSwitch` | `show_mode_switch(bool)` | **`true`** |
| `ShowPreview` | `show_preview(bool)` | **`true`** |
| `ShowSliders` | `show_sliders(bool)` | **`true`** |
| `SpectrumIcon` | `spectrum_icon(impl Into<String>)` | `TUNE` |
| `ThrottleInterval` | `throttle_interval(u32)` | **`50`** ms |
| `UpdateBindingIfOnlyHSLChanged` | `update_binding_if_only_hsl_changed(bool)` | **`false`** |
| `Value` | `ColorPickerState::value: Option<ColorChannels>` | `None` |
| `ValueChanged` | `ColorPickerResponse::action` (`ColorSelected`) | — |
| `PickerContent` | **protected** — not ported | — |

### Inherited (`MudPicker<T>`)

`AdornmentIcon` / `AdornmentColor`, `AnchorOrigin`, `AutoClose`, `Clearable`,
`Color`, `Disabled`, `Editable`, `Elevation`, `HelperText`,
`HelperTextOnFocus`, `IconSize`, `ImmediateText`, `Label`, `Margin`, `Modal`,
`OnClick`, `OverflowBehavior`, `PickerClosed`, `PickerOpened`,
`PickerVariant`, `Placeholder`, `ReadOnly`, `Rounded`, `ShowToolbar`, `Size`,
`Square`, `Variant`, `Class`.

### Deliberately not rendered / not ported

| MudBlazor | Reason |
|---|---|
| `PickerContent` | Protected. `show` is the composition point; a custom body is a separate widget. |
| `PickerActions` | Deferred to pass 2 (same as `MudDatePicker`). |
| `Mask` / typed parsing | Deferred. The HEX input parses on change, but there is no character mask. |
| `Inline` / `Static` picker variants | Stored; only `Dialog` renders. |
| `Style` / `Tag` / `UserAttributes` | No CSS strings, no HTML tag. |
| `Class` resolution | Stored on `base` only. |

### Enums introduced

Two public enums, each in its own file (Rule 2):

| Enum | File | Discriminants |
|---|---|---|
| `ColorPickerView` | `mud_egui_core::enums::color_picker_view` | `Spectrum = 0`, `Palette = 1`, `Grid = 2`[reference:22] |
| `ColorPickerMode` | `mud_egui_core::enums::color_picker_mode` | `Rgb = 0`, `Hsl = 1`, `Hex = 2`[reference:23] |

---

## Architecture

```
inputs/mud_color_picker/
├── color_channels.rs       ColorChannels + HSL↔RGB + parsing + formatting
├── color_picker_action.rs  ColorPickerAction (own file, Rule 2)
├── color_picker_state.rs   ColorPickerState (caller-owned)
├── mud_color_picker.rs     MudColorPicker + body + toolbar + swatches + tests
└── mod.rs

mud_egui_core/src/enums/
├── color_picker_mode.rs    ColorPickerMode
└── color_picker_view.rs    ColorPickerView
```

**Pure layer.** `rgb_to_hsl` / `hsl_to_rgb` / `parse_hex` / `parse_alpha` /
`ColorChannels::format` / `contrast_ink` are free functions or methods over
`u8`, `f64` and `&str`. `ColorPickerState::throttle_allows` is arithmetic on
`f64`. None touch `egui::Context`.

**State layer.** `ColorPickerState` holds the committed colour, both text
buffers, the popover state, the view, the mode, the drag latch and the
throttle timestamp. It is caller-owned — the same pattern `MudDatePicker`,
`MudTimePicker` and `MudDateRangePicker` use.

**Component layer.** `show` renders the trigger, opens the popover, then
delegates to `toolbar_row` / `spectrum_body` / `swatch_grid`. Every click or
drag is written into a deferred `Option`, applied after `popover.show`
releases `&mut state.popover`.

**Value type.** `ColorChannels` mirrors `MudColor`'s accessor surface
(`R`/`G`/`B`/`A`/`H`/`S`/`L`) and keeps RGB and HSL in sync[reference:24]. It
converts to and from `Color32`; convert `MudColor` at the boundary.

---

## API reference

### `MudColorPicker<'a>`

| Method | Type | Default | Description |
|---|---|---|---|
| `new()` | — | — | Default picker. |
| `label` / `placeholder` / `helper_text` | `impl Into<String>` | `None` | Trigger text slots. |
| `helper_text_on_focus(bool)` | `bool` | `false` | Helper only while focused / non-empty. |
| `error(bool)` / `error_text(s)` | `bool` / `impl Into<String>` | `false` / `None` | Error accent and message. |
| `variant` / `margin` / `color` / `size` / `icon_size` | enum | `Text` / `None` / `Primary` / `Medium` / `Medium` | Trigger appearance. |
| `clearable` / `readonly` / `disabled` / `editable` | `bool` | `false` | Interaction flags. |
| `immediate_text(bool)` | `bool` | `false` | Stored, no-op. |
| `adornment_icon` / `adornment_color` | `impl Into<String>` / `Color` | `COLORIZE` / `None` | Trigger adornment. |
| `picker_variant` | `PickerVariant` | `Dialog` | Only `Dialog` renders. |
| `elevation` / `rounded` / `square` / `modal` | — | `None` / `false` / `false` / `true` | Popover appearance. |
| `overflow_behavior` | `OverflowBehavior` | `Flip` | Flip vs reposition. |
| `show_toolbar(bool)` | `bool` | `true` | Renders the view buttons, mode switch, import/export and close. |
| `auto_close(bool)` | `bool` | `false` | Close after a pick. |
| `close_icon` | `impl Into<String>` | `CLOSE` | Close glyph. |
| `color_picker_mode` | `ColorPickerMode` | `Rgb` | Initial channel mode. |
| `color_picker_view` | `ColorPickerView` | `Spectrum` | Initial view. |
| `drag_effect(bool)` | `bool` | `true` | Commit while dragging. |
| `grid_icon` / `palette_icon` / `spectrum_icon` | `impl Into<String>` | `APPS` / `PALETTE` / `TUNE` | Toolbar glyphs. |
| `import_export_icon` | `impl Into<String>` | `IMPORT_EXPORT` | Export glyph. |
| `palette(iter)` | `iter of ColorChannels` | 35 entries | Swatch list. |
| `right_to_left(bool)` | `bool` | `false` | Mirrors `RightToLeft`. |
| `show_alpha(bool)` | `bool` | `true` | Include the alpha channel. |
| `show_color_field(bool)` | `bool` | `true` | Saturation × lightness plane. |
| `show_inputs(bool)` | `bool` | `true` | Channel inputs. |
| `show_mode_switch(bool)` | `bool` | `true` | Mode-switch button. |
| `show_preview(bool)` | `bool` | `true` | Preview dot + quick colours. |
| `show_sliders(bool)` | `bool` | `true` | Hue and alpha strips. |
| `throttle_interval(u32)` | `u32` | `50` | Milliseconds between drag commits. |
| `update_binding_if_only_hsl_changed(bool)` | `bool` | `false` | Commit HSL-only edits. |
| `picker_actions(f)` | `FnMut(&mut Ui) -> Option<ColorPickerAction>` | `None` | Custom footer row. |
| `on_picker_opened` / `on_picker_closed` / `on_click` | `FnMut()` | `None` | Callbacks. |
| `classes(iter)` / `flavor(StyleFlavor)` | — | — | Utilities. |
| `show(ui, state)` | `(&mut Ui, &mut ColorPickerState)` | — | Returns `ColorPickerResponse`. |

### `ColorPickerState`

| Field / method | Description |
|---|---|
| `value: Option<ColorChannels>` | Committed colour. |
| `text: String` | Trigger text — always HEX/HEXA. |
| `hex_buffer: String` | The HEX input's own buffer. |
| `popover: PopoverState` | Popover state. |
| `view: ColorPickerView` | Current view. |
| `mode: ColorPickerMode` | Current channel mode. |
| `last_update: Option<f64>` | Timestamp of the last drag commit. |
| `drag_active: bool` | Whether a drag is in flight. |
| `new(id_salt)` / `with_value(color, show_alpha)` | Constructors. |
| `is_open()` / `open()` / `close()` / `toggle()` | Popover control. |
| `commit(color, show_alpha) -> bool` | Commit; returns whether it changed. |
| `clear()` | Reset value and buffers, close. |
| `current() -> ColorChannels` | Value or opaque black. |
| `cycle_mode()` / `change_view(view)` | Toolbar mutations. |
| `throttle_allows(now, interval_ms)` / `mark_updated(now)` | Drag throttle. |

### `ColorPickerResponse`

| Field | Type | Description |
|---|---|---|
| `trigger` | `egui::Response` | Trigger response. |
| `value` | `Option<ColorChannels>` | Committed colour after this frame. |
| `action` | `ColorPickerAction` | What happened this frame. |

### `ColorPickerAction`

| Variant | Description |
|---|---|
| `None` | No interaction. |
| `Opened` / `Closed` | Popover opened / closed. |
| `ColorSelected(ColorChannels)` | A colour was committed. Mirrors `ValueChanged`. |
| `Cleared` | The value was cleared. |

Helpers: `changed_value()`, `toggled_open()`, `selected()`, `name()`.

### `ColorChannels`

| Member | Description |
|---|---|
| `from_rgba8(r, g, b, a)` / `from_rgba(r, g, b, a)` / `from_hsla(h, s, l, a)` | Constructors. |
| `black()` | Opaque black. |
| `parse(&str)` | `#rgb`, `#rgba`, `#rrggbb`, `#rrggbbaa`, `rgb()`, `rgba()`. |
| `r()` / `g()` / `b()` / `a()` / `a8()` | RGB + alpha. |
| `h()` / `s()` / `l()` | HSL. |
| `with_r` / `with_g` / `with_b` / `with_a` / `with_h` / `with_s` / `with_l` | Channel setters. |
| `rgb_eq` / `hsl_eq` | Independent comparisons. |
| `to_hex` / `to_hexa` / `to_rgb` / `to_rgba` / `to_hsl` / `to_hsla` | Formatting. |
| `hex_text(show_alpha)` | The trigger text — always hex. |
| `format(mode, show_alpha)` | Formatting dispatcher. |
| `contrast_ink()` | Black or white by the library's 384 rule. |
| `From<Color32>` / `Into<Color32>` / `Display` | Conversions. |

### Constants

| Name | Value |
|---|---|
| `PLANE_SIZE` | `220.0` |
| `STRIP_HEIGHT` | `14.0` |
| `SWATCH_SIZE` | `24.0` |
| `DEFAULT_THROTTLE_MS` | `50` |
| `DEFAULT_PALETTE` | 35 hex strings (7 hues × 5 shades) |
| `QUICK_COLOR_COUNT` | `5` |

---

## Examples

### Basic picker

```rust
let mut state = ColorPickerState::new("accent");
MudColorPicker::new().label("Accent").show(ui, &mut state);
```

### Seed a value

```rust
let mut state = ColorPickerState::new("accent").with_value(
    ColorChannels::parse("#2196F3").unwrap(),
    /* show_alpha = */ false,
);
```

### HEX-only, no alpha

```rust
MudColorPicker::new()
    .color_picker_mode(ColorPickerMode::Hex)
    .show_alpha(false)
    .show_inputs(true)
    .show(ui, &mut state);
```

### Palette-only

```rust
MudColorPicker::new()
    .color_picker_view(ColorPickerView::Palette)
    .show_toolbar(false)
    .show(ui, &mut state);
```

### Throttled drag for a remote theme

```rust
MudColorPicker::new()
    .drag_effect(true)
    .throttle_interval(120)
    .show(ui, &mut state);
```

### HSL edits fire even when the RGB round-trip is identical

```rust
MudColorPicker::new()
    .color_picker_mode(ColorPickerMode::Hsl)
    .update_binding_if_only_hsl_changed(true)
    .show(ui, &mut state);
```

### Custom palette

```rust
let brand: Vec<ColorChannels> = ["#0D47A1", "#1565C0", "#1E88E5", "#42A5F5", "#90CAF9"]
    .iter()
    .filter_map(|s| ColorChannels::parse(s))
    .collect();

MudColorPicker::new()
    .palette(brand)
    .color_picker_view(ColorPickerView::Grid)
    .show(ui, &mut state);
```

### Reacting to the outcome

```rust
let resp = MudColorPicker::new().show(ui, &mut state);
match resp.action {
    ColorPickerAction::ColorSelected(c) => log::info!("picked {}", c.to_hex()),
    ColorPickerAction::Cleared => log::info!("cleared"),
    ColorPickerAction::Opened => log::info!("opened"),
    ColorPickerAction::Closed => log::info!("closed"),
    ColorPickerAction::None => {}
}
```

### Programmatic set

```rust
state.commit(ColorChannels::from_rgba(0, 200, 83, 255), /* show_alpha = */ false);
state.clear();
```

---

## Views, toolbar and interaction

### Views

| `ColorPickerView` | Body |
|---|---|
| `Spectrum` (default) | Saturation × lightness plane, hue strip, alpha strip, preview, channel inputs |
| `Palette` | Wrapping list of `palette` at 7 columns |
| `Grid` | Wrapping grid of the same colours at 12 columns |

### Toolbar (`show_toolbar`)

```
[ Spectrum ] [ Palette ] [ Grid ]   [ RGB ]  …spacer…  [ ⤴ ] [ ✕ ]
```

- Three view buttons; the active one gets an accent fill.
- The mode switch cycles `RGB → HSL → HEX → RGB`.
- Import/export copies the current colour, formatted in the current mode, to
  the clipboard via `Context::copy_text`.
- The close button closes the popover.

### Spectrum interaction

| Control | Mapping |
|---|---|
| Saturation × lightness plane | x → `S` ∈ `[0, 1]`, y → `L` ∈ `[0, 1]` (bottom = 0) |
| Hue strip | x → `H` ∈ `[0, 360]` |
| Alpha strip | x → `A` ∈ `[0, 1]` |

The plane is painted as an 8 × 8 vertex-coloured mesh, **exact at every
vertex** and linear between them. The colour under the cursor is the colour
that gets committed — there is no HSV/HSL mismatch.

### Channel inputs

| Mode | Inputs |
|---|---|
| `Rgb` | `R`, `G`, `B` (`0..=255`) and `A` (`0..=100`%) |
| `Hsl` | (see *Deviations* — the inputs stay RGB for pass 1) |
| `Hex` | A single `MudTextField` with the hex string |

`MudSlider` is used for the numeric channels because egui has no per-channel
text-input state; the slider is the library's numeric control and is keyboard
accessible.

### Drag, throttle, HSL-only

| Parameter | Behaviour |
|---|---|
| `drag_effect(true)` (default) | A drag commits as it moves. |
| `drag_effect(false)` | Only a click or drag-stop commits. |
| `throttle_interval(50)` | Two drag commits are at least this many milliseconds apart. |
| `update_binding_if_only_hsl_changed(false)` (default) | Commits require an RGB change. |
| `update_binding_if_only_hsl_changed(true)` | An HSL change commits even when the 8-bit RGB round-trip is identical. |

---

## Performance and memory choices

| Choice | Effect |
|---|---|
| `ColorChannels` is `Copy` (7 fields) | Zero allocation for the value. |
| HSL cached on construction | `h()`, `s()`, `l()` are field reads; `hsl_eq` is three `f64` comparisons. |
| `hsl_plane` / `hue_strip` / `alpha_strip` build one `Mesh` each | 81 vertices for the plane, 14 for the hue strip, 4 for the alpha strip. |
| Painter borrows scoped to `{ … }` blocks | Every `ui.painter()` borrow ends before the next `MudText` / `MudSlider` call. |
| `MudTextField` / `MudPopover` / `MudSlider` reuse | No duplicated trigger chrome, positioning, or numeric-control logic. |
| Deferred clicks | `state` is mutated only after `popover.show` releases `&mut state.popover`. |
| Throttle in state | Two `f64` comparisons per frame, no `Memory` round-trip. |
| HEX buffer synced only when stale | Typed input is never overwritten mid-edit. |

---

## Testing

`cargo test -p mud_egui_components mud_color_picker` runs the module's tests:

| Area | Tests |
|---|---|
| `ColorChannels` | construction, HSL round-trip over a 5×5×5 RGB grid, known HSL values, all channel setters, `rgb_eq` / `hsl_eq` independence, hex/rgb/rgba parsing (valid, invalid, whitespace, case), formatting per mode and alpha, `contrast_ink` at the 384 boundary, `Color32` round-trip |
| `ColorPickerAction` | default, `changed_value`, `toggled_open`, `selected`, names |
| `ColorPickerState` | defaults, `with_value` (with and without alpha), toggle, `commit` returns-whether-changed, `clear`, `current` fallback, `cycle_mode` wrap, `change_view`, throttle (first / blocked / elapsed / zero interval) |
| `ColorPickerMode` | default, discriminants, `next` cycle, unique names, `Display` |
| `ColorPickerView` | default, discriminants, `ALL`, `is_spectrum` / `is_swatch_based`, `Display` |
| Builders | every `MudColorPicker` field, default palette size and first entry, quick-colour count |

Rendering needs a live `egui::Context`; the pure layer is the source of truth.

```bash
cargo test -p mud_egui_core color_picker_mode color_picker_view
cargo test -p mud_egui_components color_channels
cargo test -p mud_egui_components color_picker_state
cargo test -p mud_egui_components color_picker_action
cargo test -p mud_egui_components mud_color_picker
```

---

## Differences from MudBlazor

- **`Value: MudColor?` → `Option<ColorChannels>`.** `ColorChannels` mirrors
  `MudColor`'s accessor surface and keeps RGB and HSL in sync[reference:25]. Convert
  at the boundary: `ColorChannels::from(Color32::from(mud_color))` and
  `Color32::from(channels)`.
- **`ValueChanged` becomes `ColorPickerResponse::action`.** The caller owns
  `state.value`; `ColorSelected(_)` reports the committing frame. `Opened` /
  `Closed` mirror `PickerOpened` / `PickerClosed`.
- **The spectrum plane is a vertex-coloured mesh, not a shader.** egui 0.38
  has no custom shaders, so the plane is sampled at 8 × 8 and triangulated.
  The colour under the cursor is exact at every vertex and linear between —
  no HSV/HSL mismatch.
- **Channel inputs are `MudSlider`s.** egui has no per-channel text-input
  state to hang off; the slider is the library's numeric control, is keyboard
  accessible, and shows the value on hover. HEX mode uses a real
  `MudTextField`.
- **HSL mode falls back to RGB inputs for pass 1.** The mode switch and the
  `Hsl` formatting are implemented; the *inputs* remain RGB sliders until a
  dedicated HSL input row lands. `update_binding_if_only_hsl_changed` still
  works because HSL is stored, not recomputed.
- **`ImportExportIcon` copies to the clipboard.** MudBlazor opens a
  paste-a-hex popover; the immediate-mode equivalent is `Context::copy_text`
  with the current value formatted in the current mode.
- **The default palette is 35 entries (7 hues × 5 shades).** MudBlazor's
  default is also 35 colours[reference:26]; the exact hex list is a named
  constant (`DEFAULT_PALETTE`) so a caller can replace it. The first five are
  the quick colours, as documented.
- **`ShowToolbar` gates the view buttons and the mode switch.** MudBlazor's
  remark is "the view can be changed if `ShowToolbar` is true"[reference:27];
  the port matches.
- **`DragEffect` + `ThrottleInterval` are implemented.** The deadline lives in
  `ColorPickerState`, not `egui::Memory`, and `throttle_allows` is a pure
  predicate.
- **`UpdateBindingIfOnlyHSLChanged` compares the stored HSL triple.** Because
  `ColorChannels` caches HSL at construction, the comparison is exact rather
  than a recomputation.
- **`PickerContent` and `PickerActions` are not ported.** `PickerContent` is
  protected; `PickerActions` is deferred to pass 2, same as `MudDatePicker`.
- **`Style` / `Tag` / `UserAttributes` not ported.** No CSS strings, no HTML
  tag.

===== ./crates/mud_egui_components/src/inputs/pickers/mud_date_picker/README.md =====

# MudDatePicker for `mud_egui`

A text field with a popover calendar for choosing a single date, for
[egui](https://github.com/emilk/egui). Ported from MudBlazor's
`MudDatePicker` / `MudBaseDatePicker` / `MudPicker<T>` (MudBlazor 8.13.0).

```rust
let mut state = DatePickerState::new("birthday", SimpleDate::new(2024, 1, 1));

MudDatePicker::new()
    .label("Birthday")
    .variant(Variant::Outlined)
    .min_date(SimpleDate::new(1900, 1, 1))
    .max_date(SimpleDate::new(2100, 12, 31))
    .show(ui, &mut state);
```

The trigger reuses **`MudTextField`** (label, placeholder, helper, error,
variant, margin, size, calendar adornment — all free). The dropdown reuses
**`MudPopover`** (anchor / flip / backdrop / Escape — all free). All
calendar text renders through **`MudText`**; navigation chevrons through
**`MudIcon`**.

**No `chrono` dependency.** The port ships a small pure `SimpleDate` type
that covers everything the calendar needs (leap years, weekday via
Sakamoto's algorithm, month arithmetic, ISO formatting).

---

## Contents

1. [Design principle](#design-principle)
2. [Architecture](#architecture)
3. [API reference](#api-reference)
4. [Examples](#examples)
5. [Calendar views](#calendar-views)
6. [Date arithmetic](#date-arithmetic)
7. [Performance and memory choices](#performance--memory-choices)
8. [Testing](#testing)
9. [Differences from MudBlazor](#differences-from-mudblazor)

---

## Design principle

**Composition, not reimplementation.** `MudDatePicker` follows the same
pattern as `MudSelect` / `MudAutocomplete` / `MudMenu`:

| Primitive | Role |
|-----------|------|
| [`MudTextField`](../../inputs/text_field/README.md) | The trigger field, `readonly` unless `editable(true)`. |
| [`MudPopover`](../../overlays/popover/README.md) | The calendar dropdown. |

Only new code is the **calendar body** (three zoom levels + navigation) and
the **commit / clear logic**.

### Ported parameters

**Base (from `MudPicker<T>`):**

| MudBlazor | Rust method | Default |
|-----------|-------------|---------|
| `Adornment` + `AdornmentIcon` + `AdornmentColor` | `adornment_icon(s)` / `adornment_color(Color)` | `CALENDAR_MONTH` / `None` |
| `AnchorOrigin` | `anchor_origin(..)` | `None` (bottom-left) |
| `Clearable` | `clearable(bool)` | `false` |
| `Color` | `color(Color)` | `Primary` |
| `Disabled` | `disabled(bool)` | `false` |
| `Editable` | `editable(bool)` | `false` |
| `Elevation` | `elevation(u8)` | `None` |
| `HelperText` | `helper_text(s)` | `None` |
| `HelperTextOnFocus` | `helper_text_on_focus(bool)` | `false` |
| `IconSize` | `icon_size(Size)` | `Medium` |
| `ImmediateText` | `immediate_text(bool)` | `false` |
| `Label` | `label(s)` | `None` |
| `Margin` | `margin(Margin)` | `None` |
| `Modal` | `modal(bool)` | `true` |
| `OnClick` | `on_click(f)` | `None` |
| `OverflowBehavior` | `overflow_behavior(..)` | `Flip` |
| `PickerActions` | `picker_actions(f)` | `None` |
| `PickerClosed` | `on_picker_closed(f)` | `None` |
| `PickerOpened` | `on_picker_opened(f)` | `None` |
| `PickerVariant` | `picker_variant(..)` | `Dialog` |
| `Placeholder` | `placeholder(s)` | `None` |
| `ReadOnly` | `readonly(bool)` | `false` |
| `Rounded` | `rounded(bool)` | `false` |
| `ShowToolbar` | `show_toolbar(bool)` | `true` |
| `Square` | `square(bool)` | `false` |
| `Variant` | `variant(Variant)` | `Text` |

**Date-specific (from `MudBaseDatePicker` + `MudDatePicker`):**

| MudBlazor | Rust method | Default |
|-----------|-------------|---------|
| `Date` | `DatePickerState::value` (`Option<SimpleDate>`) | `None` |
| `DateFormat` | `date_format(f)` | ISO 8601 |
| `FirstDayOfWeek` | `first_day_of_week(u8)` | `0` (Monday) |
| `IsDateDisabledFunc` | `is_date_disabled(f)` | `None` |
| `MaxDate` | `max_date(SimpleDate)` | `None` |
| `MinDate` | `min_date(SimpleDate)` | `None` |
| `OpenTo` | `open_to(OpenTo)` | `Day` |
| `PickerMonth` | `DatePickerState::view_month` | initial month |

### Deferred to pass 2

- **`Mask`** — character-by-character input masking.
- **`Editable` typing / input parsing** — the field accepts typed text but
  does not parse it back into a `SimpleDate` yet. Use the calendar.
- **`FixDay` / `FixMonth`** — lock the day or month portion of the value.
- **`RelativeWidth` / `Orientation`** — popover width / orientation options.
- **`ShowToolbar` rendering** — the toolbar (which in MudBlazor shows the
  currently hovered date) is not painted; the parameter is stored.
- **`Inline` / `Static` picker variants** — only `Dialog` renders the
  calendar; `Inline` and `Static` are stored for parity.
- **`PickerActions` custom buttons** — the closure exists and is called
  below the calendar, but pass 1 does not render the standard OK/Cancel
  pair.
- **`TextUpdateSuppression` / `KeyDownPreventDefault`** — browser-specific.
- **`ShrinkLabel` / `Underline` / `ToolbarClass`** — cosmetic / CSS-only.

### Not ported

- **`Class` / `Style` / `Tag` / `UserAttributes`** — no CSS strings or HTML
  tags in egui.

---

## Architecture

```
inputs/picker/
└── simple_date.rs  — SimpleDate + pure calendar helpers

inputs/date_picker/
├── date_picker_state.rs  — DatePickerState (value + text + view + popover)
├── date_picker_view.rs   — public DatePickerView (Day / Month / Year)
├── mud_date_picker.rs    — MudDatePicker + DatePickerResponse + tests
└── mod.rs

mud_egui_core/src/enums/
├── picker_variant.rs  — PickerVariant (Inline / Dialog / Static)
└── open_to.rs         — OpenTo (Day / Month / Year / Hours / Minutes)
```

**Pure layer.** `SimpleDate` (calendar arithmetic), `is_selectable`,
`decade_start`, `rotate_day_names`, `header_title`, `navigate` — free
functions and methods, unit-tested without `egui::Context`.

**State layer.** `DatePickerState` owns the value, text, popover state, view
month, and view year. Persist across frames — this is what makes the picker
usable without callbacks.

**Component layer.** `show` renders the trigger, handles open / close, and
delegates positioning to `MudPopover`. The calendar body is a pure
`render_calendar_body` free function.

---

## API reference

### `MudDatePicker`

| Method | Type | Default | Description |
|--------|------|---------|-------------|
| `new()` | — | — | Default picker. |
| `label(s)` / `placeholder(s)` / `helper_text(s)` | `impl Into<String>` | `None` | Trigger text slots. |
| `helper_text_on_focus(bool)` | `bool` | `false` | Show helper only on focus. |
| `variant(Variant)` / `margin(Margin)` / `color(Color)` / `size(Size)` / `icon_size(Size)` | enum | `Text` / `None` / `Primary` / `Medium` / `Medium` | Trigger appearance. |
| `clearable(bool)` / `readonly(bool)` / `disabled(bool)` / `editable(bool)` | `bool` | `false` | Interaction flags. |
| `immediate_text(bool)` | `bool` | `false` | Stored; no-op. |
| `adornment_icon(s)` / `adornment_color(Color)` | `impl Into<String>` / `Color` | `CALENDAR_MONTH` / `None` | Trigger adornment. |
| `picker_variant(PickerVariant)` | `PickerVariant` | `Dialog` | Inline / Dialog / Static. |
| `elevation(u8)` / `rounded(bool)` / `square(bool)` / `modal(bool)` | `u8` / `bool` | `None` / `false` / `false` / `true` | Popover appearance. |
| `overflow_behavior(OverflowBehavior)` | `OverflowBehavior` | `Flip` | Flip vs reposition. |
| `show_toolbar(bool)` | `bool` | `true` | Stored; no toolbar rendered. |
| `open_to(OpenTo)` | `OpenTo` | `Day` | Initial zoom level. |
| `max_height(f32)` | `f32` | `None` | Popover max height. |
| `min_date(SimpleDate)` / `max_date(SimpleDate)` | `SimpleDate` | `None` | Range clamp. |
| `first_day_of_week(u8)` | `u8` | `0` (Monday) | 0 = Monday … 6 = Sunday. |
| `date_format(f)` | `Fn(&SimpleDate) -> String` | ISO 8601 | Display format. |
| `is_date_disabled(f)` | `Fn(&SimpleDate) -> bool` | `None` | Extra disable predicate. |
| `picker_actions(f)` | `FnMut(&mut Ui, &mut DatePickerState)` | `None` | Custom footer row. |
| `on_picker_opened(f)` / `on_picker_closed(f)` / `on_click(f)` | `FnMut()` | `None` | Callbacks. |
| `classes(iter)` / `flavor(StyleFlavor)` | — | — | Utilities. |
| `show(ui, state)` | `(&mut Ui, &mut DatePickerState)` | — | Returns `DatePickerResponse`. |

### `DatePickerState`

| Field / method | Description |
|----------------|-------------|
| `value: Option<SimpleDate>` | Committed date. |
| `text: String` | Trigger text (kept in sync). |
| `popover: PopoverState` | Popover state. |
| `view_month: SimpleDate` | First day of the month being displayed. |
| `view_year_decade: i32` | Start year of the decade grid. |
| `view: DatePickerView` | Current zoom level. |
| `new(id_salt, initial_view_month)` | Closed state. |
| `with_value(value, text)` | Seed the value. |
| `is_open()` / `open()` / `close()` / `toggle()` | Popover control. |
| `commit(value, text)` | Set value and close. |
| `clear()` | Reset value, close. |

### `DatePickerView`

| Variant | Zoom level |
|---------|------------|
| `Day` (default) | Calendar month grid. |
| `Month` | 12 months of one year. |
| `Year` | A decade of years. |

### `DatePickerResponse`

| Field | Type | Description |
|-------|------|-------------|
| `trigger` | `egui::Response` | Trigger response. |
| `value` | `Option<SimpleDate>` | Committed value after this frame. |
| `text` | `String` | Trigger text after this frame. |
| `action` | `DatePickerAction` | What happened this frame. |

### `DatePickerAction`

| Variant | Description |
|---------|-------------|
| `None` | No interaction. |
| `Opened` / `Closed` | Popover opened / closed. |
| `DateSelected(SimpleDate)` | A day was picked and committed. |
| `Cleared` | The value was cleared. |

Helpers: `changed_value()`, `toggled_open()`.

### `SimpleDate`

| Member | Description |
|--------|-------------|
| `new(y, m, d)` | Clamped constructor. |
| `first_of_month(y, m)` | Constructs `YYYY-MM-01`. |
| `is_leap_year(y)` | Static leap-year check. |
| `days_in_month(y, m)` | Days in a month. |
| `weekday_monday_based()` | 0 = Monday … 6 = Sunday. |
| `add_days(n)` / `add_months(n)` / `add_years(n)` | Roll-over arithmetic. |
| `first_of_this_month()` / `last_of_this_month()` | Bounds. |
| `is_before(o)` / `is_after(o)` / `in_range(min, max)` / `clamped(min, max)` | Comparisons. |
| `format_iso()` / `to_string()` | `"YYYY-MM-DD"`. |
| `calendar_grid(y, m, first_day_of_week)` | 42-day grid for the calendar. |
| `same_month(o)` | Same year + month. |

---

## Examples

### Basic picker

```rust
let mut state = DatePickerState::new("birthday", SimpleDate::new(2024, 1, 1));
MudDatePicker::new()
    .label("Birthday")
    .show(ui, &mut state);
```

### Range clamping

```rust
MudDatePicker::new()
    .min_date(SimpleDate::new(1900, 1, 1))
    .max_date(SimpleDate::new(2100, 12, 31))
    .show(ui, &mut state);
```

### Custom formatter (`DD/MM/YYYY`)

```rust
MudDatePicker::new()
    .date_format(|d| format!("{:02}/{:02}/{:04}", d.day, d.month, d.year))
    .show(ui, &mut state);
```

### Weekend-disabled picker

```rust
MudDatePicker::new()
    .is_date_disabled(|d| d.weekday_monday_based() >= 5)
    .show(ui, &mut state);
```

### Sunday-first weeks

```rust
MudDatePicker::new()
    .first_day_of_week(6)
    .show(ui, &mut state);
```

### Open on year view

```rust
MudDatePicker::new()
    .open_to(OpenTo::Year)
    .show(ui, &mut state);
```

### Clearable

```rust
MudDatePicker::new()
    .clearable(true)
    .show(ui, &mut state);
// Clicking the clear icon sets `state.value = None`.
```

### Custom footer actions

```rust
MudDatePicker::new()
    .picker_actions(|ui, state| {
        ui.horizontal(|ui| {
            if MudButton::new("Today").size(Size::Small).show(ui).clicked() {
                let today = SimpleDate::new(2024, 6, 15); // caller-chosen
                state.commit(today, "2024-06-15");
            }
        });
    })
    .show(ui, &mut state);
```

### Read the outcome

```rust
let resp = MudDatePicker::new().show(ui, &mut state);
match resp.action {
    DatePickerAction::DateSelected(d) => log::info!("picked {d}"),
    DatePickerAction::Cleared => log::info!("cleared"),
    DatePickerAction::Opened => log::info!("opened"),
    DatePickerAction::Closed => log::info!("closed"),
    DatePickerAction::None => {}
}
```

### Programmatic selection

```rust
state.commit(SimpleDate::new(2024, 12, 25), "2024-12-25");
state.clear();
```

---

## Calendar views

The calendar has three zoom levels, navigable by clicking the header title
(zoom out) or picking a cell (zoom in):

| View | Grid shows | `<` / `>` navigates | Clicking a cell |
|------|------------|---------------------|-----------------|
| `Day` (default) | 6 weeks × 7 days of one month | One month | Commits the day |
| `Month` | 4 rows × 3 months of one year | One year | Zooms to `Day` on that month |
| `Year` | 2 rows × 5 years of a decade | One decade | Zooms to `Month` on that year |

Clicking the header title zooms out one level. `DatePickerView` is the
public enum — `DatePickerState::view` tracks the current level, so callers
can preset it.

### Selection and disabled state

- The current `value` is highlighted in the accent colour.
- Days outside `min_date` / `max_date` are rendered in `text_disabled`.
- Days matching `is_date_disabled(date)` are also disabled.
- Disabled cells still render (so the grid layout is stable) but ignore
  clicks.

---

## Date arithmetic

`SimpleDate` implements everything the calendar needs without a
dependency:

- **Leap years** — full Gregorian rule (`year % 4 == 0 && year % 100 != 0 ||
  year % 400 == 0`).
- **`weekday_monday_based()`** — Sakamoto's algorithm. Constant time, no
  lookup beyond 12 entries.
- **`add_days` / `add_months` / `add_years`** — proper month and year
  roll-over, day clamping (31 Jan + 1 month → 28 Feb or 29 Feb).
- **`calendar_grid(y, m, first_day_of_week)`** — 42-day grid (6 weeks), so
  the calendar height never shifts between months.

All arithmetic is unit-tested with fixed dates (2024-01-01 = Monday,
2025-01-01 = Wednesday, leap-year Feb 29 handling, decade boundaries).

**No "today" concept** — without a system clock, "today" is caller-owned.
Pass `SimpleDate::new(...)` from your caller for the initial view month,
or wire `std::time::SystemTime` to a `SimpleDate` manually.

---

## Performance and memory choices

| Choice | Effect |
|--------|--------|
| `SimpleDate` is `Copy` (12 bytes) | Zero allocation for date values. |
| Pure calendar arithmetic | Testable without `egui::Context`. |
| `calendar_grid` returns 42 `Copy` dates | Fits in 1 stack-allocated vec. |
| `Header / grid rendering use `allocate_exact_size` + child `Ui`s | No per-cell layout passes. |
| `MudIcon` reuse across chevrons | Free thanks to its transform cache. |
| `MudText` for all calendar text | Theme and typography propagate. |
| `MudTextField` / `MudPopover` reuse | Zero duplication of trigger chrome or positioning. |
| `PickerPalette` resolved once per frame | One palette lookup regardless of grid size. |

---

## Testing

`cargo test` runs 30+ pure-function tests across three files:

| Area | Tests |
|------|-------|
| `SimpleDate` | clamping, leap years, days_in_month, weekday (known dates), add_days / add_months / add_years roll-over, in_range, clamped, format_iso, calendar_grid (42 days, correct start), same_month |
| `DatePickerView` | default, zoom_out chain, zoom_in chain |
| `DatePickerState` | new / with_value, open_resyncs_view, commit, clear, toggle |
| `PickerVariant` / `OpenTo` | default, name, is_dialog / is_date_variant predicates |
| Picker helpers | `variant_opens_popover`, `is_selectable` (min/max/predicate), `decade_start`, `rotate_day_names`, `header_title`, `navigate` per view |
| Defaults / builders | every field |
| `DatePickerAction` | changed_value / toggled_open |

Rendering (`show`) needs a live `egui::Context`; the pure layer is the
source of truth.

```bash
cargo test -p mud_egui_components mud_date_picker
cargo test -p mud_egui_components simple_date
cargo test -p mud_egui_core picker_variant open_to
```

---

## Differences from MudBlazor

- **No `chrono`.** The `SimpleDate` type is a small pure date with just
  enough arithmetic. Callers needing timezone handling or `DateTime` need
  their own bridge.
- **No `MudColorPicker` / `MudTimePicker` yet.** Those ship in the next
  two messages, reusing the same `PickerVariant`, `OpenTo`, and shared
  `MudPicker<T>` structure.
- **`Inline` / `Static` picker variants are stored but not rendered.** Only
  `Dialog` shows the calendar; the other variants are accepted for API
  parity.
- **`Mask` / `Editable` typing are deferred.** The field accepts typed
  text but does not parse it back into `SimpleDate`.
- **`FixDay` / `FixMonth` deferred.**
- **`Text` is caller-owned via `DatePickerState::text`.** MudBlazor's
  `Text` is bidirectionally bound; the Rust port stores it in the state.
- **`Style` / `Tag` / `UserAttributes` not ported.** No CSS strings or HTML
  tags.

===== ./crates/mud_egui_components/src/inputs/pickers/mud_date_range_picker/README.md =====

# MudDateRangePicker for `mud_egui`

A pair of text fields with a popover calendar that selects a **start** and an
**end** date, for [egui](https://github.com/emilk/egui). Ported from
MudBlazor's [`MudDateRangePicker`](https://mudblazor.com/components/datepicker)
(MudBlazor 8.13.0).

```rust
let mut state = DateRangePickerState::new("booking", SimpleDate::new(2024, 1, 1));

MudDateRangePicker::new()
    .label("Stay")
    .placeholder_start("Arrival")
    .placeholder_end("Departure")
    .min_days(1)
    .max_days(30)
    .show(ui, &mut state);

// After a completed selection: state.range is start..=end, ordered.
```

The trigger is two **`MudTextField`**s separated by a **`MudIcon`** — the egui
equivalent of `MudRangeInput`. The dropdown reuses **`MudPopover`**. Every
numeral and every label renders through **`MudText`**.

**No `chrono` dependency.** `SimpleDate` (from `MudDatePicker`) supplies the
calendar arithmetic; `DateRange` is a small pair of optional dates.

---

## Contents

1. [Design principle: API parity, not API invention](#design-principle-api-parity-not-api-invention)
2. [Architecture](#architecture)
3. [API reference](#api-reference)
4. [Examples](#examples)
5. [Two-click selection and validation](#two-click-selection-and-validation)
6. [Performance and memory choices](#performance-and-memory-choices)
7. [Testing](#testing)
8. [Differences from MudBlazor](#differences-from-mudblazor)

---

## Design principle: API parity, not API invention

The MudBlazor 8.13.0 parameter surface was verified against DNDocs before
implementation. `MudDateRangePicker` has **ten own parameters** plus the
inherited `MudBaseDatePicker` + `MudPicker<T>` surface, and every one is
accounted for.

### Own parameters

| MudBlazor | Rust method | Default |
|---|---|---|
| `AllowDisabledDatesInCount` | `allow_disabled_dates_in_count(bool)` | **`true`** |
| `AllowDisabledDatesInRange` | `allow_disabled_dates_in_range(bool)` | **`false`** |
| `AutoClose` | `auto_close(bool)` | **`true`** |
| `DateRange` | `DateRangePickerState::range` | empty |
| `DateRangeChanged` | `DateRangePickerResponse::action` | — |
| `MaxDays` | `max_days(u32)` | `None` |
| `MinDays` | `min_days(u32)` | `None` |
| `PlaceholderEnd` | `placeholder_end(s)` | `None` |
| `PlaceholderStart` | `placeholder_start(s)` | `None` |
| `SeparatorIcon` | `separator_icon(s)` | `ARROW_RIGHT_ALT` |
| `InputContent` / `IsRange` | **protected** — not ported | — |

### Inherited parameters

`AdornmentIcon`, `AdornmentColor`, `AnchorOrigin`, `Clearable`, `Color`,
`Disabled`, `Editable`, `Elevation`, `HelperText`, `HelperTextOnFocus`,
`IconSize`, `ImmediateText`, `Label`, `Margin`, `Modal`, `OnClick`,
`OverflowBehavior`, `PickerActions`, `PickerClosed`, `PickerOpened`,
`PickerVariant`, `Placeholder`, `ReadOnly`, `Rounded`, `ShowToolbar`, `Size`,
`Square`, `Variant`, `Class`, `DateFormat`, `FirstDayOfWeek`,
`IsDateDisabledFunc`, `MaxDate`, `MinDate`, `OpenTo`, `PickerMonth`,
`TitleDateFormat`.

### Deliberately not rendered

| MudBlazor | Reason |
|---|---|
| `AdornmentIcon` / `AdornmentColor` | `MudRangeInput` has no adornment slot — the separator icon is the only chrome between the fields. Stored so a Blazor call site ports without edits. |
| `Placeholder` | Overridden by `PlaceholderStart` / `PlaceholderEnd` in `MudRangeInput`. Stored. |
| `TitleDateFormat` / `ShowToolbar` | The toolbar is not painted (same deferral as `MudDatePicker`). |
| `IconSize` | The calendar chevrons are a fixed 20 pt, as in `MudDatePicker`. |
| `Style` / `Tag` / `UserAttributes` | No CSS strings, no HTML tag. |
| `Class` resolution | Stored on `base` only. |

### Deferred to pass 2

`Mask`, typed `Editable` parsing, `Inline` / `Static` rendering,
`ClosingDelay` timing (commit closes in the same frame), `FixDay` /
`FixMonth`, `ShrinkLabel`, `Underline`.

---

## Architecture

```
inputs/date_range_picker/
├── date_range.rs                DateRange + inclusive_days
├── range_position.rs            pub(crate) RangePosition (own file, Rule 2)
├── date_range_picker_action.rs  DateRangePickerAction (own file, Rule 2)
├── date_range_picker_state.rs   DateRangePickerState (caller-owned)
├── mud_date_range_picker.rs     MudDateRangePicker + response + helpers + tests
└── mod.rs
```

**Reused from `MudDatePicker`** (imported, not duplicated):
`SimpleDate`, `DatePickerView`, `is_selectable`, `rotate_day_names`, and the
calendar geometry constants (`DAY_CELL`, `HEADER_HEIGHT`, `CALENDAR_PADDING`,
`CALENDAR_WIDTH`, `MONTH_NAMES`, `DEFAULT_DAY_NAMES`). Requires one extra
`pub use simple_date::SimpleDate;` in `date_picker/mod.rs`.

**Pure layer.** `inclusive_days`, `DateRange::*`, `second_click_valid`,
`span_count`, `range_contains_disabled`, `classify`, `preview_range`,
`day_at`, `header_title`, `navigate` — free functions or methods over
`SimpleDate`, `DateRange`, `Pos2` and `Rect`. None touch `egui::Context`.

**State layer.** `DateRangePickerState { range, text_start, text_end, popover,
view_month, view_year_decade, view, pending_start }` is caller-owned.
`pending_start` is the egui equivalent of MudBlazor's private `_firstDate`.

**Component layer.** `show` renders the label, the two fields and the
separator, the helper/error row, then delegates the calendar to
`render_calendar_body` and applies the deferred clicks after
`popover.show` releases `&mut state.popover`.

---

## API reference

### `MudDateRangePicker<'a>`

| Method | Type | Default | Description |
|---|---|---|---|
| `new()` | — | — | Default picker. |
| `label(s)` | `impl Into<String>` | `None` | Caption above the pair. |
| `placeholder(s)` | `impl Into<String>` | `None` | Base placeholder — stored, overridden by the two below. |
| `placeholder_start(s)` / `placeholder_end(s)` | `impl Into<String>` | `None` | Per-field hints. |
| `helper_text(s)` | `impl Into<String>` | `None` | Caption below the pair. |
| `helper_text_on_focus(bool)` | `bool` | `false` | Helper only while focused / non-empty. |
| `error(bool)` / `error_text(s)` | `bool` / `impl Into<String>` | `false` / `None` | Error accent and message. |
| `variant(Variant)` / `margin(Margin)` / `color(Color)` / `size(Size)` / `icon_size(Size)` | enum | `Text` / `None` / `Primary` / `Medium` / `Medium` | Field appearance. |
| `clearable` / `readonly` / `disabled` / `editable` | `bool` | `false` | Interaction flags. |
| `immediate_text(bool)` | `bool` | `false` | Stored, no-op. |
| `adornment_icon(s)` / `adornment_color(Color)` | — | `CALENDAR_MONTH` / `None` | **Stored, not rendered** (`MudRangeInput` has no adornment slot). |
| `separator_icon(s)` | `impl Into<String>` | `ARROW_RIGHT_ALT` | Glyph between the fields. |
| `picker_variant(PickerVariant)` | — | `Dialog` | Only `Dialog` renders. |
| `elevation(u8)` / `rounded(bool)` / `square(bool)` / `modal(bool)` | — | `None` / `false` / `false` / `true` | Popover appearance. |
| `overflow_behavior(OverflowBehavior)` | — | `Flip` | Flip vs reposition. |
| `show_toolbar(bool)` / `title_date_format(s)` | — | `true` / `None` | Stored — the toolbar is not painted. |
| `open_to(OpenTo)` | `OpenTo` | `Day` | Initial zoom level. |
| `max_height(f32)` | `f32` | `None` | Stored (popover size hint). |
| `auto_close(bool)` | `bool` | `true` | Close after the range is committed. |
| `min_date(SimpleDate)` / `max_date(SimpleDate)` | `SimpleDate` | `None` | Selectable range clamp. |
| `first_day_of_week(u8)` | `u8` | `0` (Monday) | 0 = Monday … 6 = Sunday. |
| `date_format(f)` | `Fn(&SimpleDate) -> String` | ISO 8601 | Display format for both fields. |
| `is_date_disabled(f)` | `Fn(&SimpleDate) -> bool` | `None` | Extra disable predicate. |
| `min_days(u32)` / `max_days(u32)` | `u32` | `None` | Inclusive day-count bounds; clamped to ≥ 1. |
| `allow_disabled_dates_in_count(bool)` | `bool` | `true` | Count disabled days toward the bounds. |
| `allow_disabled_dates_in_range(bool)` | `bool` | `false` | Allow a committed range containing a disabled day. |
| `picker_actions(f)` | `FnMut(&mut Ui) -> Option<DateRangePickerAction>` | `None` | Custom footer row. |
| `on_picker_opened` / `on_picker_closed` / `on_click` | `FnMut()` | `None` | Callbacks. |
| `classes(iter)` / `flavor(StyleFlavor)` | — | — | Utilities. |
| `show(ui, state)` | `(&mut Ui, &mut DateRangePickerState)` | — | Returns `DateRangePickerResponse`. |

### `DateRangePickerState`

| Field / method | Description |
|---|---|
| `range: DateRange` | Committed range. |
| `text_start` / `text_end: String` | Trigger texts (kept in sync by `commit_range` / `clear` / `with_range`). |
| `popover: PopoverState` | Popover state. |
| `view_month: SimpleDate` | First day of the displayed month. |
| `view_year_decade: i32` | Start year of the decade grid. |
| `view: DatePickerView` | Current zoom level. |
| `pending_start: Option<SimpleDate>` | First click of the two-click selection. |
| `new(id_salt, initial_view_month)` | Closed state. |
| `with_range(range, text_start, text_end)` | Seed (normalises). |
| `is_open()` / `open()` / `close()` / `toggle()` | Popover control. |
| `commit_range(range, text_start, text_end)` | Commit and close. |
| `clear()` | Reset range and texts, close. |
| `display_range()` | `pending_start`-aware range for the calendar. |
| `first_click()` | The pending start, if any. |

### `DateRangePickerResponse`

| Field | Type | Description |
|---|---|---|
| `trigger_start` | `egui::Response` | Start field response. |
| `trigger_end` | `egui::Response` | End field response. |
| `range` | `DateRange` | Committed range after this frame. |
| `action` | `DateRangePickerAction` | What happened this frame. |

### `DateRangePickerAction`

| Variant | Description |
|---|---|
| `None` | No interaction. |
| `Opened` / `Closed` | Popover opened / closed. |
| `RangeSelected(DateRange)` | A complete range was committed. Mirrors `DateRangeChanged`. |
| `Cleared` | The range was cleared. |

Helpers: `changed_value()`, `toggled_open()`, `selected()`, `name()`.

### `DateRange`

| Member | Description |
|---|---|
| `new(start, end)` / `empty()` / `single(date)` | Constructors. |
| `is_empty()` / `is_complete()` / `is_same_day()` | Predicates. |
| `normalized()` | Orders `start <= end`. |
| `contains(&date)` | Inclusive membership (half-open = start only). |
| `day_count()` | Inclusive day count. |
| `formatted(format)` | `(String, String)` for the two fields. |
| `Display` | `start ..= end`, `—` for a missing end. |

### Free helpers

| Function | Signature | Description |
|---|---|---|
| `inclusive_days` | `(SimpleDate, SimpleDate) -> i32` | Inclusive span, either order. |
| `second_click_valid` | `(first, second, min, max, allow_in_count, disabled) -> bool` | Second-click validation. |
| `span_count` | `(first, second, allow_disabled, disabled) -> i32` | Inclusive count, optionally skipping disabled days. |
| `range_contains_disabled` | `(&DateRange, &dyn Fn(&SimpleDate) -> bool) -> bool` | Any disabled day in the span. |
| `day_at` | `(Pos2, Rect, &[SimpleDate]) -> Option<SimpleDate>` | Grid hit-test. |
| `format_one` | `(Option<SimpleDate>, format) -> String` | One field's text. |

### Constants

| Name | Value |
|---|---|
| `SEPARATOR_SIZE` | `20.0` |
| `SEPARATOR_GAP` | `4.0` |

---

## Examples

### Basic range picker

```rust
let mut state = DateRangePickerState::new("range", SimpleDate::new(2024, 1, 1));
MudDateRangePicker::new().label("Period").show(ui, &mut state);
```

### Arrival / departure with a 30-day cap

```rust
MudDateRangePicker::new()
    .label("Stay")
    .placeholder_start("Arrival")
    .placeholder_end("Departure")
    .min_days(1)
    .max_days(30)
    .show(ui, &mut state);
```

Days that would exceed the cap are greyed out immediately after the first
click.

### Weekends excluded from the day count

```rust
let weekend = |d: &SimpleDate| d.weekday_monday_based() >= 5;

MudDateRangePicker::new()
    .is_date_disabled(weekend)
    .allow_disabled_dates_in_count(false) // 5 business days may span 7 calendar days
    .max_days(5)
    .show(ui, &mut state);
```

### Reject ranges that cross a disabled day

```rust
MudDateRangePicker::new()
    .is_date_disabled(|d| d == SimpleDate::new(2024, 12, 25))
    .allow_disabled_dates_in_range(false) // default
    .show(ui, &mut state);
```

A range that spans Christmas Day is rejected; the first click stays pending.

### Keep the popover open for multiple picks

```rust
MudDateRangePicker::new()
    .auto_close(false)
    .show(ui, &mut state);
```

### Custom separator glyph

```rust
MudDateRangePicker::new()
    .separator_icon(filled::REMOVE)
    .show(ui, &mut state);
```

### Custom date format

```rust
MudDateRangePicker::new()
    .date_format(|d| format!("{:02}/{:02}/{:04}", d.day, d.month, d.year))
    .show(ui, &mut state);
// Both fields and the committed range use it.
```

### Reacting to the outcome

```rust
let resp = MudDateRangePicker::new().show(ui, &mut state);
match resp.action {
    DateRangePickerAction::RangeSelected(r) => log::info!("{r} ({} days)", r.day_count().unwrap()),
    DateRangePickerAction::Cleared => log::info!("cleared"),
    DateRangePickerAction::Opened => log::info!("opened"),
    DateRangePickerAction::Closed => log::info!("closed"),
    DateRangePickerAction::None => {}
}
```

### Live preview from the first click

```rust
let resp = MudDateRangePicker::new().show(ui, &mut state);
if let Some(first) = state.first_click() {
    // The user has clicked once; the calendar is previewing first ..= hover.
    log::debug!("selection started at {first}");
}
```

### Programmatic range

```rust
state.commit_range(
    DateRange::new(Some(SimpleDate::new(2024, 6, 1)), Some(SimpleDate::new(2024, 6, 30))),
    "2024-06-01",
    "2024-06-30",
);
state.clear();
```

---

## Two-click selection and validation

### Click flow

| Click | Effect |
|---|---|
| First | `pending_start = Some(day)`; the calendar shows it as a single-day range and previews `day ..= hover` |
| Second | Orders the pair, checks `MinDays` / `MaxDays` / disabled days, commits `DateRange`, closes if `auto_close` or no `picker_actions` |
| Third | Starts a fresh selection (`pending_start` is cleared on every commit) |

The committed range is **always ordered** so `start <= end`, whatever the
click order — MudBlazor's `_firstDate` / `_secondDate` normalisation.

### Validation

| Parameter | Effect | Default |
|---|---|---|
| `min_days(n)` | Rejects a span shorter than `n` days, inclusive | none |
| `max_days(n)` | Rejects a span longer than `n` days, inclusive | none |
| `allow_disabled_dates_in_count(b)` | When `false`, disabled days are skipped in the count | `true` |
| `allow_disabled_dates_in_range(b)` | When `false`, a span containing a disabled day is rejected | `false` |

`MinDays` / `MaxDays` are **inclusive of the selected date**: a same-day
range has a count of `1`, so `max_days(1)` allows exactly one day and
`min_days(2)` forbids it.

A rejected second click leaves `pending_start` in place, so the user can pick
a different end without restarting.

### Visual states

| State | Fill |
|---|---|
| `Start` / `End` | Accent pill, rounded on the outer edge only |
| `SingleDay` | Accent pill, fully rounded |
| `Between` | Accent at 60 α, full-width band |
| Preview (hovering the second date) | Accent at 32 α |
| Hover on a selectable day | `widgets.hovered.weak_bg_fill` |

Ink is `accent_contrast_text` on endpoints, `text_primary` elsewhere, and
`text_disabled` for unselectable or out-of-month days.

---

## Performance and memory choices

| Choice | Effect |
|---|---|
| `DateRange` is `Copy` (two `Option<SimpleDate>`) | Zero allocation for the value. |
| `inclusive_days` uses a Julian-day number | `O(1)` day counting, no `add_days` loops. |
| Pure validation helpers | `second_click_valid` / `span_count` / `classify` are testable without a `Context`. |
| One interaction region for the day grid | One `allocate_exact_size` + one `Response` for 42 cells; the hover preview is computed *before* painting. |
| `day_at` is a pure grid hit-test | No per-cell `interact` calls. |
| Numerals through `MudText` in child `Ui`s | Rule 3 compliant; no `painter.text`. |
| Bands and pills through `ui.painter()` | Geometry only — no user-facing text bypasses `MudText`. |
| Theme fetched once per frame | One `MudTheme::fetch`; the palette is resolved once. |
| `MudTextField` / `MudPopover` / `MudDatePicker` reuse | No duplicated trigger chrome, positioning, or calendar arithmetic. |
| Text snapshotted before the field borrow | `state.text_*` is read before `MudTextField::new(&mut state.text_*)` takes it. |

---

## Testing

`cargo test -p mud_egui_components mud_date_range_picker` runs the module's
tests:

| Area | Tests |
|---|---|
| `DateRange` | empty/default, `new` preserves order, `single`, completeness, `is_same_day`, `normalized` (ordering, idempotence, partial), `contains` (inclusive, reversed, half-open, empty), `day_count`, `formatted` (closure, ISO fallback, blank end), `Display` |
| `inclusive_days` | same day, adjacent, full month, month and year boundaries, leap years, a ten-year span |
| `second_click_valid` | no bounds, `max_days` inclusive both directions, `min_days` inclusive, both composing, disabled-day counting on/off |
| `span_count` | matches `inclusive_days` without a predicate; skips disabled days with one |
| `range_contains_disabled` | inside, absent, reversed range, half-open range |
| `classify` | every position, single day, half-open, reversed, empty |
| `preview_range` | missing ends, same day, reversed pair normalised |
| `day_at` | every cell of the 7 × 6 grid, outside the grid, short cell list |
| `header_title` / `navigate` | all three views |
| `RangePosition` | `is_endpoint`, `is_band`, unique names |
| `DateRangePickerAction` | default, `changed_value`, `toggled_open`, `selected`, names |
| `DateRangePickerState` | defaults, `with_range` (seed + normalise), `open` / `close` / `toggle`, `open` drops the pending start, `commit_range`, `clear`, `display_range`, `first_click` |
| Builders | defaults match MudBlazor, every field set, `min_days` / `max_days` clamp, `first_day_of_week` clamp |

Rendering needs a live `egui::Context`; the pure layer is the source of truth.

```bash
cargo test -p mud_egui_components mud_date_range_picker
cargo test -p mud_egui_components date_range
cargo test -p mud_egui_components date_range_picker_state
cargo test -p mud_egui_components date_range_picker_action
```

---

## Differences from MudBlazor

- **No `chrono`.** `SimpleDate` is a small pure date; `DateRange` holds two
  `Option<SimpleDate>`.
- **`DateRange` is a `Copy` value type with `Option` endpoints**, mirroring
  `Range<DateTime?>`. v9 makes `Start` / `End` init-only; the port keeps them
  mutable, matching 8.13.
- **`DateRangeChanged` becomes `DateRangePickerResponse::action`.** The
  caller owns `state.range`; `RangeSelected(_)` reports the committing frame.
  `Opened` / `Closed` mirror `PickerOpened` / `PickerClosed`. Same split as
  `DatePickerAction` and `MudRating`.
- **The trigger is two `MudTextField`s + a `MudIcon`**, the egui equivalent
  of `MudRangeInput`. The label and the helper/error row are painted once
  above/below the pair, not per field.
- **`AdornmentIcon` / `AdornmentColor` / `Placeholder` are stored, not
  rendered.** `MudRangeInput` overrides `InputContent` and has no adornment
  slot; the separator icon is the only chrome between the fields.
- **`AutoClose` is ported and defaults to `true`.** MudBlazor also closes when
  no `PickerActions` are defined, which the port reproduces
  (`auto_close || picker_actions.is_none()`).
- **`MinDays` / `MaxDays` are validated per second click, not via cached
  bounds.** Semantically identical to MudBlazor's `_minValidDate` /
  `_maxValidDate` recalculation; the predicate form is simpler and testable.
- **`allow_disabled_dates_in_count` defaults to `true`**, so disabled days
  count toward the bounds unless the caller opts out — matching the DNDocs
  remark.
- **`allow_disabled_dates_in_range` defaults to `false`**, so a range
  spanning a disabled day is rejected. The rejected click leaves the pending
  start in place.
- **`Inline` / `Static` picker variants are stored but not rendered.** Only
  `Dialog` shows the calendar.
- **`ClosingDelay` timing is not implemented**; a commit closes in the same
  frame.
- **`Style` / `Tag` / `UserAttributes` not ported.** No CSS strings or HTML
  tags.

===== ./crates/mud_egui_components/src/inputs/pickers/mud_time_picker/README.md =====

# MudTimePicker for `mud_egui`

A text field with a popover clock dial for choosing a time of day, for
[egui](https://github.com/emilk/egui). Ported from MudBlazor's
[`MudTimePicker`](https://mudblazor.com/components/time-picker)
(MudBlazor 8.13.0).

```rust
let mut state = TimePickerState::new("meeting");

MudTimePicker::new()
.label("Meeting time")
.am_pm(true)
.minute_selection_step(15)
.show(ui, &mut state);
```

The trigger reuses **`MudTextField`**. The dropdown reuses **`MudPopover`**.
Every numeral and every AM/PM marker renders through **`MudText`**; the
adornment and the clear button render through **`MudIcon`**.

**No `chrono` dependency.** A small pure `SimpleTime` type covers everything
the dial needs.

---

## Contents

1. [Design principle](#design-principle)
2. [Architecture](#architecture)
3. [API reference](#api-reference)
4. [Examples](#examples)
5. [Clock views and commit flow](#clock-views-and-commit-flow)
6. [Formatting and arithmetic](#formatting-and-arithmetic)
7. [Performance and memory choices](#performance-and-memory-choices)
8. [Testing](#testing)
9. [Differences from MudBlazor](#differences-from-mudblazor)

---

## Design principle

**Composition, not reimplementation.** `MudTimePicker` follows the same
pattern as `MudDatePicker` / `MudSelect` / `MudMenu`:

| Primitive | Role |
|-----------|------|
| [`MudTextField`](../../inputs/text_field/README.md) | The trigger, `readonly` unless `editable(true)`. |
| [`MudPopover`](../../overlays/popover/README.md) | The clock dropdown. |

Only the **clock dial** (ring outlines, hand, numerals, AM/PM markers) and the
**hour → minute commit logic** are new.

### Ported parameters

**Time-specific:**

| MudBlazor | Rust method | Default |
|-----------|-------------|---------|
| `AmPm` | `am_pm(bool)` | `false` |
| `AutoClose` | `auto_close(bool)` | `false` |
| `ClosingDelay` | `closing_delay(u32)` | `200` ms |
| `MinuteSelectionStep` | `minute_selection_step(u8)` | `1` |
| `OpenTo` | `open_to(OpenTo)` | `OpenTo::Hours` |
| `Time` | `TimePickerState::value` | `None` |
| `TimeChanged` | `TimePickerResponse::action` | — |
| `TimeEditMode` | `time_edit_mode(TimeEditMode)` | `Normal` |
| `TimeFormat` | `time_format(&str)` | `hh:mm tt` / `HH:mm` |

**Base (`MudPicker<T>`):** `Adornment` + `AdornmentIcon` + `AdornmentColor`,
`AnchorOrigin`, `Clearable`, `Color`, `Disabled`, `Editable`, `Elevation`,
`HelperText`, `HelperTextOnFocus`, `IconSize`, `ImmediateText`, `Label`,
`Margin`, `Modal`, `OnClick`, `OverflowBehavior`, `PickerActions`,
`PickerClosed`, `PickerOpened`, `PickerVariant`, `Placeholder`, `ReadOnly`,
`Rounded`, `ShowToolbar`, `Size`, `Square`, `Variant`, `Class`.

**Pointer state:** `PointerMoving` is a public parameter in 8.13.0
(`public bool PointerMoving { get; set; }`)[reference:7]. The port derives it
from the dial's `Response` and uses it to suppress the hand animation.

### Deferred to pass 2

- **`Mask`** — character-by-character input masking.
- **`Editable` typed parsing** — the field accepts text but does not parse
  it back into a `SimpleTime` yet. Use the dial.
- **`Inline` / `Static` picker variants** — stored; only `Dialog` renders.
- **`ShowToolbar` rendering** — stored; no toolbar is painted.
- **`TextUpdateSuppression` / `KeyDownPreventDefault`** — browser-specific.
- **`ShrinkLabel` / `Underline` / `ToolbarClass`** — cosmetic.

### Not ported

- **`Class` resolution** — stored on `base` for parity only.
- **`Style` / `Tag` / `UserAttributes`** — no CSS strings, no HTML tag.

---

## Architecture

```
inputs/time_picker/
├── simple_time.rs          SimpleTime + formatting + arithmetic
├── time_picker_view.rs     TimePickerView (Hours | Minutes)
├── time_picker_action.rs   TimePickerAction (own file, Rule 2)
├── time_picker_state.rs    TimePickerState (caller-owned)
├── mud_time_picker.rs      MudTimePicker + TimePickerResponse + dial + tests
└── mod.rs

mud_egui_core/src/enums/
└── time_edit_mode.rs       TimeEditMode (Normal | OnlyMinutes | OnlyHours)
```

**Pure layer.** `snap_minute`, `clock_angle`, `clock_index`,
`outer_hour_for_index`, `inner_hour_for_index`, `is_outer_ring`,
`numeral_pos`, `hour_at`, `minute_at`, `flip_am_pm`, `view_for_open` and
every `SimpleTime` method are free functions over `f32`, `u8`, `Pos2` and
`SimpleTime`. None touch `egui::Context`.

**State layer.** `TimePickerState { value, text, popover, view, pending_hour }`
is caller-owned. `pending_hour` bridges the hour and minute steps of one
selection so the commit can happen on the minute click.

**Component layer.** `show` renders the trigger, honours any `AutoClose`
deadline, opens/closes the popover, renders the dial, then applies the
deferred clicks to `state` after `popover.show` releases
`&mut state.popover`.

---

## API reference

### `MudTimePicker<'a>`

| Method | Type | Default | Description |
|--------|------|---------|-------------|
| `new()` | — | — | Default picker. |
| `label` / `placeholder` / `helper_text` | `impl Into<String>` | `None` | Trigger text slots. |
| `helper_text_on_focus(bool)` | `bool` | `false` | Helper only while focused / non-empty. |
| `variant(Variant)` / `margin(Margin)` / `color(Color)` / `size(Size)` / `icon_size(Size)` | enum | `Text` / `None` / `Primary` / `Medium` / `Medium` | Trigger appearance. |
| `clearable` / `readonly` / `disabled` / `editable` | `bool` | `false` | Interaction flags. |
| `immediate_text(bool)` | `bool` | `false` | Stored, no-op. |
| `adornment_icon` / `adornment_color` | `impl Into<String>` / `Color` | `ACCESS_TIME` / `None` | Trigger adornment. |
| `picker_variant(PickerVariant)` | `PickerVariant` | `Dialog` | Only `Dialog` renders. |
| `elevation(u8)` / `rounded(bool)` / `square(bool)` / `modal(bool)` | — | `None` / `false` / `false` / `true` | Popover appearance. |
| `overflow_behavior(OverflowBehavior)` | — | `Flip` | Flip vs reposition. |
| `show_toolbar(bool)` | `bool` | `true` | Stored; no toolbar rendered. |
| `am_pm(bool)` | `bool` | `false` | 12-hour clock + markers. |
| `auto_close(bool)` | `bool` | `false` | Close after the value is set. |
| `closing_delay(u32)` | `u32` | `200` | Milliseconds before closing. |
| `minute_selection_step(u8)` | `u8` | `1` | Dial minute granularity; clamped to ≥ 1. |
| `open_to(OpenTo)` | `OpenTo` | `Hours` | Initial dial. |
| `time_edit_mode(TimeEditMode)` | `TimeEditMode` | `Normal` | Which parts are editable. |
| `time_format(&str)` | `&str` | `None` | Display template. |
| `picker_actions(f)` | `FnMut(&mut Ui) -> Option<TimePickerAction>` | `None` | Custom footer row. |
| `on_picker_opened` / `on_picker_closed` / `on_click` | `FnMut()` | `None` | Callbacks. |
| `classes(iter)` / `flavor(StyleFlavor)` | — | — | Utilities. |
| `show(ui, state)` | `(&mut Ui, &mut TimePickerState)` | — | Returns `TimePickerResponse`. |

### `TimePickerState`

| Field / method | Description |
|----------------|-------------|
| `value: Option<SimpleTime>` | Committed value. |
| `text: String` | Trigger text (kept in sync by `commit` / `clear` / `with_value`). |
| `popover: PopoverState` | Popover state. |
| `view: TimePickerView` | Dial currently shown. |
| `pending_hour: Option<u8>` | Hour chosen in the hour step, awaiting a minute. |
| `new(id_salt)` / `with_value(time, text)` | Constructors. |
| `open(OpenTo)` / `close()` / `toggle(OpenTo)` | Popover control. |
| `commit(time, text)` / `clear()` | Value mutation. |
| `effective_hour()` | `pending_hour` → value's hour → `0`. |

### `TimePickerResponse`

| Field | Type | Description |
|-------|------|-------------|
| `trigger` | `egui::Response` | Trigger response. |
| `value` | `Option<SimpleTime>` | Committed value after this frame. |
| `text` | `String` | Trigger text after this frame. |
| `action` | `TimePickerAction` | What happened this frame. |

### `TimePickerAction`

| Variant | Description |
|---------|-------------|
| `None` | No interaction. |
| `Opened` / `Closed` | Popover opened / closed. |
| `TimeSelected(SimpleTime)` | A time was committed. Mirrors `TimeChanged`. |
| `Cleared` | The value was cleared. |

Helpers: `changed_value()`, `toggled_open()`, `name()`.

### `TimePickerView`

| Variant | Description |
|---------|-------------|
| `Hours` (default) | Hour ring. |
| `Minutes` | Minute ring. |

Helpers: `name()`, `toggle()`, `is_hours()`.

### `TimeEditMode` (`mud_egui_core::enums`)

| Variant | Discriminant | Behaviour |
|---------|-------------|-----------|
| `Normal` (default) | `0` | Hours then minutes. |
| `OnlyMinutes` | `1` | Opens on the minute ring; hour comes from the value. |
| `OnlyHours` | `2` | Commits on the hour click; minute comes from the value. |

Helpers: `ALL`, `name()`, `commits_after_hour()`, `opens_on_minutes()`,
`Display`.

### `SimpleTime`

| Member | Description |
|--------|-------------|
| `new(h, m, s)` / `hm(h, m)` / `midnight()` | Clamped constructors. |
| `is_am()` / `is_pm()` / `hour_12()` | 12-hour helpers. |
| `total_minutes()` / `from_total_minutes(n)` | Minutes since midnight. |
| `add_minutes(delta)` | Wrap-around shift. |
| `with_hours` / `with_minutes` / `toggle_am_pm` | Field replacement. |
| `format_hhmm()` / `format_default(am_pm)` / `format(template)` | Display. |
| `Display` | `HH:MM`. |

### Free helpers

| Function | Signature | Description |
|----------|-----------|-------------|
| `snap_minute` | `(u8, u8) -> u8` | Nearest multiple of the step, wrapping. |
| `clock_angle` | `(Pos2, Pos2) -> f32` | Radians, `0` = 12 o'clock, clockwise. |
| `hour_at` | `(Pos2, Pos2, bool) -> Option<u8>` | Click → hour (12h / 24h). |
| `minute_at` | `(Pos2, Pos2, u8) -> Option<u8>` | Click → minute, snapped. |
| `numeral_pos` | `(Pos2, f32, u8) -> Pos2` | Position of clock index `i`. |
| `view_for_open` | `(OpenTo) -> TimePickerView` | Initial dial. |

### Constants

| Name | Value |
|------|-------|
| `DIAL_SIZE` | `240.0` |
| `OUTER_RADIUS` | `105.6` |
| `INNER_RADIUS` | `72.0` |
| `AM_PM_ROW_HEIGHT` | `32.0` |
| `DEFAULT_CLOSING_DELAY_MS` | `200` |

---

## Examples

### Basic 24-hour picker

```rust
let mut state = TimePickerState::new("start");
MudTimePicker::new().label("Start time").show(ui, &mut state);
```

### 12-hour with AM/PM

```rust
MudTimePicker::new()
.label("Alarm")
.am_pm(true)
.show(ui, &mut state);
```

### 15-minute granularity, auto-close

```rust
MudTimePicker::new()
.label("Slot")
.minute_selection_step(15)
.auto_close(true)
.closing_delay(300)
.show(ui, &mut state);
```

### Open on the minute ring

```rust
MudTimePicker::new()
.open_to(OpenTo::Minutes)
.show(ui, &mut state);
```

### Hours only (e.g. a start-hour field)

```rust
MudTimePicker::new()
.time_edit_mode(TimeEditMode::OnlyHours)
.time_format("HH时")
.show(ui, &mut state);
```

### Minutes only (e.g. a minute offset)

```rust
MudTimePicker::new()
.time_edit_mode(TimeEditMode::OnlyMinutes)
.open_to(OpenTo::Minutes)
.show(ui, &mut state);
```

### Custom format

```rust
MudTimePicker::new()
.am_pm(true)
.time_format("h:mm tt")     // → "6:32 PM"
.show(ui, &mut state);

MudTimePicker::new()
.time_format("HH:mm:ss")    // → "18:32:07"
.show(ui, &mut state);
```

### Clearable

```rust
MudTimePicker::new().clearable(true).show(ui, &mut state);
```

### Custom footer actions

```rust
MudTimePicker::new()
.picker_actions(|ui| {
ui.horizontal(|ui| {
if MudButton::new("Now").size(Size::Small).show(ui).clicked() {
return Some(TimePickerAction::TimeSelected(SimpleTime::hm(9, 0)));
}
None
}).inner
})
.show(ui, &mut state);
```

### Read the outcome

```rust
let resp = MudTimePicker::new().show(ui, &mut state);
match resp.action {
TimePickerAction::TimeSelected(t) => log::info!("picked {t}"),
TimePickerAction::Cleared => log::info!("cleared"),
TimePickerAction::Opened => log::info!("opened"),
TimePickerAction::Closed => log::info!("closed"),
TimePickerAction::None => {}
}
```

### Programmatic selection

```rust
state.commit(SimpleTime::hm(18, 30), "18:30");
state.clear();
```

---

## Clock views and commit flow

### 24-hour dial (`am_pm(false)`)

```
         12
    23        1
 22              2
11                3
 10              4
     9        5
          8
```

The outer ring is `1..=12`; the inner ring is `13..=23` and `00` at the 12
o'clock slot. Clicking near the centre is ignored.

### 12-hour dial (`am_pm(true)`)

```
          12
     11        1
   10            2
   9             3
      8        4
           5
   [ AM ]  [ PM ]
```

A single ring, plus two `MudText` markers. Clicking the inactive marker
flips the hour across noon (`09:xx ↔ 21:xx`).

### Minute dial

Labels at `0, 5, …, 55`. `minute_selection_step(n)` snaps every click to the
nearest multiple of `n` (wrapping, so `59` with a step of `15` becomes `0`).

### Commit flow

| `TimeEditMode` | Hour click | Minute click |
|----------------|------------|--------------|
| `Normal` | Store `pending_hour`, switch to the minute ring | Commit `pending_hour : minute` |
| `OnlyHours` | Commit `hour : current_minute` | never shown |
| `OnlyMinutes` | never shown | Commit `current_hour : minute` |

`AutoClose` + `ClosingDelay` are honoured: a committed time sets a deadline
in `egui::Memory`, the popover stays open until it elapses, then closes.

---

## Formatting and arithmetic

`SimpleTime::format(template)` evaluates a MudBlazor-style `TimeFormat`:

| Token | Output |
|-------|--------|
| `HH` / `H` | 24-hour, padded / unpadded |
| `hh` / `h` | 12-hour, padded / unpadded |
| `mm` / `m` | minutes, padded / unpadded |
| `ss` / `s` | seconds, padded / unpadded |
| `tt` | `AM` / `PM` |

Unknown characters are copied verbatim, so `"HH:mm"`, `"h:mm tt"` and
`"HH时mm分"` all work. An empty template falls back to the default
(`hh:mm tt` when `am_pm`, else `HH:mm`) — MudBlazor's documented defaults
[reference:8].

All arithmetic (`total_minutes`, `from_total_minutes`, `add_minutes`,
`with_hours`, `toggle_am_pm`) is pure and unit-tested.

**No "now" concept** — without a system clock, "now" is caller-owned. Seed
`TimePickerState` with `with_value(..)` if you want a starting time.

---

## Performance and memory choices

| Choice | Effect |
|--------|--------|
| `SimpleTime` is `Copy` (3 bytes) | Zero allocation for time values. |
| Pure dial geometry | `clock_angle` / `hour_at` / `minute_at` are arithmetic; testable without a `Context`. |
| Theme fetched once per frame | One `MudTheme::fetch`; eight palette colours resolved once into `PickerPalette`. |
| Numerals through `MudText` in child `Ui`s | Rule 3 compliant; no `painter.text`. |
| Ring outlines and hand through `ui.painter()` | Geometry only — no user-facing text bypasses `MudText`. |
| `animate_value_with_time` only when not dragging | `PointerMoving` suppresses it, matching the parameter's purpose[reference:9]. |
| `AutoClose` deadline in `egui::Memory` | No caller-side storage; no busy-looping. |
| `pending_hour` in state | No shadow copy on the builder between frames. |
| Deferred clicks | `state` is mutated only after `popover.show` releases its borrow. |

---

## Testing

`cargo test -p mud_egui_components mud_time_picker` runs the module's tests:

| Area | Tests |
|------|-------|
| `SimpleTime` | clamping, `hm`, AM/PM split, `hour_12`, total-minute round-trip, `add_minutes` wrap, `with_*`, `toggle_am_pm`, all format tokens, defaults |
| `TimePickerView` | default, toggle, involution, names |
| `TimeEditMode` | default, discriminants, `ALL` uniqueness, `Display`, predicates |
| `TimePickerAction` | default, `changed_value`, `toggled_open`, names |
| Dial geometry | `clock_angle` at 12/3/6/9, `clock_index`, both hour maps, `hour_at` on both rings and in both modes, dead zone, `minute_at` with and without a step |
| `snap_minute` | step 0/1, rounding, wrap |
| State | defaults, `with_value`, `open` resets view and pending hour, `commit` / `clear`, `effective_hour`, `view_for_open` |
| Builders | defaults match MudBlazor, every field set, step clamp, effective format |

Rendering needs a live `egui::Context`; the pure layer is the source of
truth.

```bash
cargo test -p mud_egui_components mud_time_picker
cargo test -p mud_egui_components simple_time
cargo test -p mud_egui_components time_picker_state
cargo test -p mud_egui_core time_edit_mode
```

---

## Differences from MudBlazor

- **No `chrono`.** `SimpleTime` is a small pure time-of-day. Callers needing
  timezone handling need their own bridge.
- **`TimeFormat` is a template string, not a `Func`.** The token set
  (`HH`/`H`/`hh`/`h`/`mm`/`m`/`ss`/`s`/`tt`) covers MudBlazor's documented
  examples; an unknown character is copied through.
- **`ClosingDelay` is honoured via a `Memory` deadline.** MudBlazor keeps the
  DOM element around for the delay; egui closes the popover when the
  deadline elapses.
- **`PointerMoving` is derived from the dial's `Response`**, not a separate
  parameter, and is used to suppress the hand animation.
- **The hand animates with `animate_value_with_time`**, not CSS. The
  duration is a constant; `PointerMoving` snaps it.
- **`Inline` / `Static` picker variants are stored but not rendered.** Only
  `Dialog` shows the clock.
- **`Editable` typing is deferred.** The field accepts typed text but does
  not parse it back into `SimpleTime`.
- **`Mask` / `FixDay`-style constraints are deferred.**
- **`Text` is caller-owned via `TimePickerState::text`.**
- **`Style` / `Tag` / `UserAttributes` not ported.** No CSS strings or HTML
  tags.
- **`TimePickerAction`, `TimePickerView` and `TimeEditMode` live in their
  own files**, per Rule 2.

===== ./crates/mud_egui_components/src/inputs/radio/README.md =====

# MudRadio + MudRadioGroup for `mud_egui`

A mutually exclusive option set for [egui](https://github.com/emilk/egui),
ported from MudBlazor’s [`MudRadio<T>`](https://mudblazor.com/components/radio)
and `MudRadioGroup<T>` (MudBlazor 8.13.0).

```rust
let mut selected: Option<&str> = None;

MudRadioGroup::new(&mut selected).show(ui, |g| {
    g.radio("a", "Option A");
    g.radio("b", "Option B");
    g.radio("c", "Option C");
});
```

Every label renders through **`MudText`**, every glyph through **`MudIcon`**.

---

## Contents

1. [Design principle: API parity, not API invention](#design-principle-api-parity-not-api-invention)
2. [Architecture](#architecture)
3. [API reference — MudRadioGroup](#api-reference--mudradiogroup)
4. [API reference — MudRadio](#api-reference--mudradio)
5. [Examples](#examples)
6. [Binding model](#binding-model)
7. [Layout and rendering](#layout-and-rendering)
8. [Performance and memory choices](#performance-and-memory-choices)
9. [Testing](#testing)
10. [Differences from MudBlazor](#differences-from-mudblazor)

---

## Design principle: API parity, not API invention

Both components were verified against DNDocs for MudBlazor 8.13.0.

### `MudRadio<T>` ported parameters

| MudBlazor | Type | Default | Rust mapping |
|---|---|---|---|
| `CheckedIcon` | `string` | `RadioButtonChecked` | `checked_icon(s)` |
| `Dense` | `bool` | `false` | `dense(bool)` |
| `IndeterminateIcon` | `string` | `IndeterminateCheckBox` | `indeterminate_icon(s)` — stored, **never painted** |
| `UncheckedColor` | `Color?` | `null` | `unchecked_color(Color)` |
| `UncheckedIcon` | `string` | `RadioButtonUnchecked` | `unchecked_icon(s)` |
| `ChildContent` | `RenderFragment` | — | `label(s)` / `content(f)` |
| `Disabled` | `bool` | `false` | `disabled(bool)` |
| `ReadOnly` | `bool` | `false` | `read_only(bool)` |
| `LabelPlacement` | `Placement` | `End` | `label_placement(Placement)` |
| `Color` | `Color` | `Default` → `Primary` | `color(Color)` |
| `Size` | `Size` | `Medium` | `size(Size)` |
| `Ripple` | `bool` | `true` | `ripple(bool)` |
| `Class` | `string?` | `null` | `classes(iter)` — stored |
| `Tag` | `object?` | `null` | `tag(t)` / `tag_ref::<U>()` |

### `MudRadioGroup<T>` ported parameters

| MudBlazor | Type | Default | Rust mapping |
|---|---|---|---|
| `ChildContent` | `RenderFragment` | — | closure to `show(ui, \|g\| ..)` |
| `Value` / `ValueChanged` | `T?` / callback | — | caller-owned `&mut Option<T>` |
| `ReadOnly` | `bool` | `false` | `read_only(bool)` |
| `Disabled` | `bool` | `false` | `disabled(bool)` |
| `Name` | `string?` | `null` | `name(s)` — stored, **inert** |
| `Class` | `string?` | `null` | `classes(iter)` — stored |
| `Tag` | `object?` | `null` | `tag(t)` / `tag_ref::<U>()` |

### Deliberately not ported

| MudBlazor | Reason |
|---|---|
| `Style` / `UserAttributes` (both) | No CSS strings, no HTML tag. |
| `Class` resolution (both) | Stored for parity. The radio’s colour comes from `Color` / `UncheckedColor`; a utility-class frame would composite a second fill. |
| Form-validation parameters (`Required`, `Error`, `ErrorText`, `HelperText`, …) | Inherited from `MudFormComponent`; no egui form system. |

`IndeterminateIcon` is inherited from `MudBooleanInput` but **never applies to a radio**: a radio is either checked or unchecked. It is stored so a Blazor call site that sets it compiles, and documented as inert.

`Name` is HTML form semantics; egui has no form submission, so it is stored and documented as inert.

---

## Architecture

```
inputs/radio/
├── mud_radio.rs
│   ├── constants        ICON_SMALL 18, ICON_MEDIUM 20, ICON_LARGE 24,
│   │                    GAP 8, DENSE_GAP 4, RIPPLE_DURATION 0.35,
│   │                    DISABLED_OPACITY 0.38, UNCHECKED_OUTLINE_ALPHA 160
│   ├── pure helpers     icon_side, is_vertical, label_leads,
│   │                    layout_size, icon_rect, label_rect, measure_label,
│   │                    with_alpha
│   ├── MudRadio<T>      builder + show(ui, &mut Option<T>)
│   ├── RadioResponse    response + selected
│   └── ripple           start_ripple, paint_ripple
└── mud_radio_group.rs
    ├── MudRadioGroup<T> builder + show(ui, |g| ..)
    ├── RadioGroupUi<T>  collector: radio / radio_full
    └── RadioGroupResponse  response
```

**Pure layer.** Icon size, layout geometry, placement helpers and label measurement are free functions over `f32` and `Placement`. None touch `egui::Context`.

**Collector layer.** `RadioGroupUi` records `MudRadio` values; the group renders them in a wrapping row after the closure returns.

**Component layer.** `MudRadio::show` measures the label, allocates the row, applies a click or `Space`/`Enter`, paints the ripple and the icon, and draws the label through `MudText`.

---

## API reference — `MudRadioGroup`

| Method | Type | Default | Mirrors | Description |
|---|---|---|---|---|
| `new(selected)` | `&'a mut Option<T>` | — | `Value` | Binds the group to a selection. |
| `disabled(b)` | `bool` | `false` | `Disabled` | Disables every radio. |
| `read_only(b)` | `bool` | `false` | `ReadOnly` | Prevents selection without dimming. |
| `name(s)` | `impl Into<String>` | `None` | `Name` | Stored, inert. |
| `classes(iter)` | `iter of AsRef<str>` | none | `Class` | Stored on `ComponentBase`. |
| `tag(t)` / `tag_ref::<U>()` | `T: Any + Send + Sync` | `None` | `Tag` | Arbitrary user data. |
| `show(ui, \|g\| ..)` | `&mut Ui, FnOnce(&mut RadioGroupUi<T>) -> R` | — | `ChildContent` | Returns `RadioGroupResponse`. |

### `RadioGroupUi<T>`

| Method | Description |
|---|---|
| `radio(option, label)` | Adds a radio for `option` with `label`. |
| `radio_full(radio)` | Adds a fully configured `MudRadio`. |
| `len()` / `is_empty()` | Number of radios registered so far. |

### `RadioGroupResponse`

| Field | Type | Description |
|---|---|---|
| `response` | `Response` | The group’s row. Deref target. |
| `inner` | `Option<()>` | Reserved; the group has no closure return value. |

---

## API reference — `MudRadio`

| Method | Type | Default | Mirrors | Description |
|---|---|---|---|---|
| `new(option)` | `T` | — | `Value` | Creates a radio for `option`. |
| `labelled(option, label)` | `(T, impl Into<String>)` | — | `ChildContent` | Convenience constructor. |
| `label(s)` | `impl Into<String>` | `None` | `ChildContent` | Label text. |
| `content(f)` | `FnMut(&mut Ui) + 'static` | `None` | `ChildContent` | Custom label content. |
| `label_placement(p)` | `Placement` | `End` | `LabelPlacement` | `Start` / `End` / `Top` / `Bottom`. |
| `dense(b)` | `bool` | `false` | `Dense` | Compact gap (4 pt). |
| `color(c)` | `Color` | `Primary` | `Color` | Accent when checked. |
| `unchecked_color(c)` | `Color` | `None` | `UncheckedColor` | Outline when unchecked. |
| `size(s)` | `Size` | `Medium` | `Size` | Icon side: 18 / 20 / 24 pt. |
| `ripple(b)` | `bool` | `true` | `Ripple` | Click ripple. |
| `disabled(b)` | `bool` | `false` | `Disabled` | Dims and disables. |
| `read_only(b)` | `bool` | `false` | `ReadOnly` | Disables without dimming. |
| `checked_icon(s)` | `impl Into<String>` | `RadioButtonChecked` | `CheckedIcon` | Checked glyph. |
| `unchecked_icon(s)` | `impl Into<String>` | `RadioButtonUnchecked` | `UncheckedIcon` | Unchecked glyph. |
| `indeterminate_icon(s)` | `impl Into<String>` | `IndeterminateCheckBox` | `IndeterminateIcon` | Stored, never painted. |
| `classes(iter)` | `iter of AsRef<str>` | none | `Class` | Stored. |
| `tag(t)` / `tag_ref::<U>()` | `T: Any + Send + Sync` | `None` | `Tag` | Arbitrary user data. |
| `show(ui, &mut Option<T>)` | `&mut Ui, &mut Option<T>` | — | — | Returns `RadioResponse`. |

### `RadioResponse`

| Field | Type | Description |
|---|---|---|
| `response` | `Response` | The radio’s row. Deref target. |
| `selected` | `bool` | `true` when this radio selected itself this frame. |

### `Size` mapping

| `Size` | Icon side |
|---|---|
| `ExtraSmall` / `Small` | 18 pt |
| `Medium` | 20 pt |
| `Large` / `ExtraLarge` | 24 pt |

### Constants

| Name | Value |
|---|---|
| `ICON_SMALL` / `ICON_MEDIUM` / `ICON_LARGE` | `18.0` / `20.0` / `24.0` |
| `GAP` / `DENSE_GAP` | `8.0` / `4.0` |
| `RIPPLE_DURATION` | `0.35` s |
| `RIPPLE_MAX_RADIUS_FACTOR` | `1.2` |
| `RIPPLE_ALPHA` | `60` |
| `DISABLED_OPACITY` | `0.38` |
| `UNCHECKED_OUTLINE_ALPHA` | `160` |

---

## Examples

### Basic group

```rust
let mut selected: Option<&str> = None;
MudRadioGroup::new(&mut selected).show(ui, |g| {
    g.radio("a", "Option A");
    g.radio("b", "Option B");
});
```

### Disabled radio inside an enabled group

```rust
MudRadioGroup::new(&mut selected).show(ui, |g| {
    g.radio("a", "Available");
    g.radio_full(MudRadio::labelled("b", "Unavailable").disabled(true));
});
```

### Read-only group

```rust
MudRadioGroup::new(&mut selected)
    .read_only(true)
    .show(ui, |g| {
        g.radio("a", "Option A");
        g.radio("b", "Option B");
    });
```

The radios are not dimmed, but clicking them does not change the selection.

### Disabled group

```rust
MudRadioGroup::new(&mut selected)
    .disabled(true)
    .show(ui, |g| {
        g.radio("a", "Option A");
    });
```

### Label before the icon

```rust
g.radio_full(
    MudRadio::labelled("a", "Option A").label_placement(Placement::Start),
);
```

### Custom icons

```rust
g.radio_full(
    MudRadio::labelled("star", "Star")
        .checked_icon(filled::STAR)
        .unchecked_icon(filled::STAR_BORDER)
        .color(Color::Warning),
);
```

### Unchecked outline colour

```rust
g.radio_full(
    MudRadio::labelled("a", "Option A").unchecked_color(Color::Error),
);
```

### Custom label content

```rust
g.radio_full(MudRadio::new("pro").content(|ui| {
    ui.add(MudText::body1("Pro").bold(true));
    ui.add(MudText::caption("$9 / month").color(Color::TextSecondary));
}));
```

### Standalone radios sharing a binding

```rust
let mut selected: Option<&str> = None;
MudRadio::labelled("a", "A").show(ui, &mut selected);
MudRadio::labelled("b", "B").show(ui, &mut selected);
```

### Tag round-trip

```rust
let mut selected: Option<&str> = None;
let group = MudRadioGroup::new(&mut selected).tag("plan");
assert_eq!(group.tag_ref::<&str>(), Some(&"plan"));
```

---

## Binding model

The group owns `&mut Option<T>`. A radio is checked when
`selected.as_ref() == Some(&radio.option)`; clicking or pressing `Space`/`Enter`
on a focused radio writes `Some(option.clone())`.

```rust
let mut selected: Option<String> = None;
MudRadioGroup::new(&mut selected).show(ui, |g| {
    g.radio("small".to_string(), "Small");
    g.radio("large".to_string(), "Large");
});
```

`ValueChanged` is not a callback; `RadioResponse::selected` is a field on the
frame’s result. This is the same split `MudAlert`, `MudCollapse` and
`MudCheckBox` use.

### Click and keyboard

| Input | Effect |
|---|---|
| Click | Selects the radio (if enabled). |
| `Space` / `Enter` (focused) | Selects the radio (if enabled). |
| `ArrowLeft` / `ArrowRight` | **Not implemented**; use Tab between radios. |

Arrow-key navigation across a radio group is documented as a limitation rather
than half-implemented: it requires tracking focus across sibling widgets, which
egui does not provide out of the box.

---

## Layout and rendering

### Horizontal

```
End:    [ ◉ ][ 8 pt ][ Label … ]
Start:  [ Label … ][ 8 pt ][ ◉ ]
```

### Vertical

```
Top:    [ Label … ]
        [ 4/8 pt ]
        [ ◉ ]
```

The group lays radios out in a wrapping row (`ui.horizontal_wrapped`). Use a
`MudStack` or `MudGrid` around the group if you need a fixed column.

### Paint

| State | Glyph | Colour |
|---|---|---|
| Checked | `CheckedIcon` or `RADIO_BUTTON_CHECKED` | `Color` |
| Unchecked | `UncheckedIcon` or `RADIO_BUTTON_UNCHECKED` | `UncheckedColor`, or `Color` at 160 alpha |

Disabled radios are gamma-multiplied by `DISABLED_OPACITY` (0.38).
`read_only(true)` does **not** dim; it only blocks selection.

### Ripple

A filled circle of radius `icon_side × 0.5 × 1.2` expands from the icon centre
over `RIPPLE_DURATION` (0.35 s), then fades out. The start time is stored in
`egui::Memory` keyed by the radio’s `Response::id`.

---

## Performance and memory choices

| Choice | Effect |
|---|---|
| Theme fetched once per radio | One `MudTheme::fetch` and up to two `Color::resolve` calls per radio. |
| Pure helpers on `Copy` values | Icon size, layout and placement are arithmetic; unit-testable without a `Context`. |
| No `Vec` or `String` allocation in the paint path | The label is measured with `layout_no_wrap` and moved into `MudText`. |
| Collector pattern | Radios are collected before rendering, so the group can wrap them without measuring twice. |
| Ripple in `egui::Memory` | No state on the radio; the builder stays stateless. |
| Disabled path skips click and keyboard | `Sense::hover()` for disabled and read-only; no interaction registration. |

---

## Testing

`cargo test -p mud_egui_components mud_radio` runs the module’s pure-function
tests:

| Area | Tests |
|---|---|
| `MudRadio` metrics | icon size per `Size`; placement helpers |
| `MudRadio` layout | `layout_size` horizontal / vertical |
| `MudRadio` builders | defaults; every field set |
| `MudRadioGroup` builders | defaults; every field set; tag round-trip |

No test creates an `egui::Context`. `show` needs a live `Ui` and is left to
integration tests.

Suggested integration tests:

- Clicking an unchecked radio selects it and deselects the previously selected
  radio.
- Clicking the already-selected radio is a no-op.
- A disabled radio ignores clicks; a disabled group ignores every click.
- `read_only(true)` blocks selection without dimming.
- `Space` / `Enter` on a focused radio selects it.
- Custom `checked_icon` / `unchecked_icon` are painted.

```bash
cargo test -p mud_egui_components mud_radio
```

---

## Differences from MudBlazor

- **`Value` / `ValueChanged` become a caller-owned `&mut Option<T>` plus
  `RadioResponse::selected`.** The caller owns the value; the response reports
  the frame it moved. This is the same split `MudCheckBox` and `MudCollapse` use.
- **The group uses the collector pattern**, the same as `MudToggleGroup`. In
  Blazor, `MudRadioGroup` is a container with `ChildContent`; in egui, a closure
  that receives `RadioGroupUi` is the idiomatic equivalent and lets the group
  wrap the radios without measuring twice.
- **`IndeterminateIcon` is stored but never painted.** A radio is binary;
  MudBlazor exposes the parameter because `MudRadio` inherits from
  `MudBooleanInput`, not because it has a visual meaning for radios.
- **`Name` is stored but inert.** HTML form submission has no egui equivalent.
- **Arrow-key navigation is not implemented.** Tab moves between radios; `Space`
  and `Enter` select the focused radio. Full roving-tabindex arrow navigation
  would need cross-widget focus tracking that egui does not provide.
- **The group lays radios out in a wrapping row.** MudBlazor’s radios are
  `inline-block`, which flows the same way. Wrap the group in a `MudStack` or
  `MudGrid` for a fixed column.
- **`Size` folds `ExtraSmall` into `Small` and `ExtraLarge` into `Large`.**
  Consistent with `MudCheckBox` and `MudSwitch`.
- **The default `Color` is `Primary`, not `Default`.** MudBlazor’s
  `MudBooleanInput` default resolves to `Primary` for this component; pinning it
  makes the default visible in the builder table.

===== ./crates/mud_egui_components/src/inputs/rating/README.md =====

# MudRating for `mud_egui`

A star-style rating control for [egui](https://github.com/emilk/egui), ported
from MudBlazor’s [`MudRating`](https://mudblazor.com/components/rating)
(MudBlazor 8.13.0).

```rust
let mut score = 3;

ui.add(
    MudRating::new(&mut score)
        .color(Color::Warning)
        .max_value(5),
);

if score != 0 {
    // score is the caller-owned selected value.
}
```

Every glyph renders through **`MudIcon`**. `MudRating` paints no text of its
own — it is a row of star icons.

---

## Contents

1. [Design principle: API parity, not API invention](#design-principle-api-parity-not-api-invention)
2. [Architecture](#architecture)
3. [API reference](#api-reference)
4. [Examples](#examples)
5. [Binding and behaviour](#binding-and-behaviour)
6. [Performance and memory choices](#performance-and-memory-choices)
7. [Testing](#testing)
8. [Differences from MudBlazor](#differences-from-mudblazor)

---

## Design principle: API parity, not API invention

The MudBlazor 8.13.0 parameter surface was verified against the published API
reference before implementation. `MudRating` has **fifteen parameters**, and all
fifteen are accounted for.

### Ported parameters

| MudBlazor | Type | Default | Rust mapping |
|---|---|---|---|
| `Color` | `Color` | `Default` | `color(Color)` |
| `Disabled` | `bool` | `false` | `disabled(bool)` |
| `EmptyIcon` | `string` | `StarBorder` | `empty_icon(s)` |
| `EmptyIconColor` | `Color?` | `null` | `empty_icon_color(Color)` |
| `FullIcon` | `string` | `Star` | `full_icon(s)` |
| `FullIconColor` | `Color?` | `null` | `full_icon_color(Color)` |
| `HoveredValueChanged` | `EventCallback<int?>` | — | `RatingResponse::hovered_changed` + `hovered` |
| `MaxValue` | `int` | **`5`** | `max_value(i32)` |
| `Name` | `string` | random GUID | `name(s)` — stored, inert |
| `ReadOnly` | `bool` | `false` | `read_only(bool)` |
| `Ripple` | `bool` | `true` | `ripple(bool)` |
| `SelectedValue` | `int` | **`0`** | caller-owned `&mut i32` |
| `SelectedValueChanged` | `EventCallback<int>` | — | `RatingResponse::changed` |
| `Size` | `Size` | `Medium` | `size(Size)` |
| `Class` | `string?` | `null` | `classes(iter)` — stored, not resolved |
| `Tag` | `object?` | `null` | `tag(t)` / `tag_ref::<U>()` |

### Deliberately not ported

| MudBlazor | Reason |
|---|---|
| `RatingItemsClass` | egui has no CSS strings. |
| `RatingItemsStyle` | egui has no CSS strings. |
| `Style` | egui has no inline CSS strings. |
| `UserAttributes` | There is no HTML tag to splat attributes onto. |
| `Class` resolution | Stored on `base` for parity only. The star colours come from `Color` / `FullIconColor` / `EmptyIconColor`; a utility-class frame would composite a second fill. |

`Name` is **stored but inert**: it becomes the HTML `<input name="…">` in
Blazor, which groups radios for form submission. egui has no form submission, so
it has no effect; it is accepted so a Blazor call site ports without edits.

Nothing else was added. There is no `Variant`, no `Elevation`, no `ReadOnly`
toggle beyond the ported `read_only`, no `AdornmentStart`/`AdornmentEnd`, and
**no `HalfIncrements` or half-star mode** — MudBlazor’s `MudRating` is
integer-only and so is this port.

### There is no `HoveredValue` parameter

MudBlazor reports the hovered star through `HoveredValueChanged` and exposes no
`HoveredValue` parameter for reading. The Rust port reports both:

- `RatingResponse::hovered: i32` — the current hovered star every frame (`0`
  when nothing is hovered), so a caller can read it directly.
- `RatingResponse::hovered_changed: Option<i32>` — `Some(new)` on the frame the
  hover moved, with `0` meaning "hover left". That maps `null` (Blazor’s
  un-hovered sentinel) to `0` and gives the same edge-triggered semantics as
  `HoveredValueChanged`.

This is an extension of *reporting*, not of the component’s parameter surface.

---

## Architecture

```
inputs/rating/
└── mud_rating.rs
    ├── constants        STAR_GAP 4, RIPPLE_DURATION 0.35, RIPPLE_ALPHA 60,
    │                    EMPTY_ALPHA 0.38, DISABLED_ALPHA 0.38
    ├── pure helpers     star_pixels, row_size, star_rect, star_at,
    │                    display_value, clamp_selected, sanitize_star_count,
    │                    apply_keys (KeyAction), with_alpha
    ├── MudRating<'a>    builder + show(ui) -> RatingResponse
    ├── RatingResponse   response + changed + hovered + hovered_changed
    └── ripple           start_ripple, paint_ripple (memory-backed)
```

**Pure layer.** The star pixel table, the row size, the per-star rect, the
hit-test mapping, the display-value rule, the selection clamp, the keyboard
mapping and the star-count sanitizer are all free functions over `f32`, `i32`
and `Rect`. None touch `egui::Context`.

**Binding layer.** The caller owns the `&mut i32`; the component writes
`Some(star)` when a click or key selects one. Two-way binding becomes a
caller-owned field plus a `changed` report.

**Component layer.** `show` allocates one row rect, uses `response.hover_pos()`
and `response.interact_pointer_pos()` to determine the hovered / clicked star,
applies the keyboard action, and paints one `MudIcon` per star.

---

## API reference

### `MudRating<'a>`

| Method | Type | Default | Mirrors | Description |
|---|---|---|---|---|
| `new(selected)` | `&'a mut i32` | — | `SelectedValue` | Binds the rating to a caller-owned `i32`. |
| `max_value(n)` | `i32` | `5` | `MaxValue` | Number of stars; `<= 0` renders nothing. |
| `color(c)` | `Color` | `Default` | `Color` | Accent of filled stars. |
| `size(s)` | `Size` | `Medium` | `Size` | Star size: 8 / 16 / 24 / 32 / 48 pt. |
| `full_icon(s)` | `impl Into<String>` | `STAR` | `FullIcon` | Filled glyph. |
| `empty_icon(s)` | `impl Into<String>` | `STAR_BORDER` | `EmptyIcon` | Empty glyph. |
| `full_icon_color(c)` | `Color` | `None` | `FullIconColor` | Filled colour override. |
| `empty_icon_color(c)` | `Color` | `None` | `EmptyIconColor` | Empty colour override. |
| `ripple(b)` | `bool` | `true` | `Ripple` | Click ripple. |
| `disabled(b)` | `bool` | `false` | `Disabled` | Dims and disables. |
| `read_only(b)` | `bool` | `false` | `ReadOnly` | Disables without dimming. |
| `name(s)` | `impl Into<String>` | `None` | `Name` | Stored, inert. |
| `classes(iter)` | `iter of AsRef<str>` | none | `Class` | Stored on `ComponentBase`. |
| `tag(t)` / `tag_ref::<U>()` | `T: Any + Send + Sync` | `None` | `Tag` | Arbitrary user data. |
| `show(ui)` | `&mut Ui` | — | — | Returns `RatingResponse`. |

### `RatingResponse`

| Field | Type | Description |
|---|---|---|
| `response` | `Response` | The rating row’s `Response`. Deref target. |
| `changed` | `bool` | `true` on the frame `SelectedValue` moved. Mirrors `SelectedValueChanged`. |
| `hovered` | `i32` | Current hovered star (`1..=max_value`), or `0` when nothing is hovered. Reported every frame. |
| `hovered_changed` | `Option<i32>` | `Some(new)` only on the frame the hover moved, with `0` meaning "hover left". Mirrors `HoveredValueChanged`. |

### `Size` mapping

| `Size` | Star side |
|---|---|
| `ExtraSmall` | 8 pt |
| `Small` | 16 pt |
| `Medium` | 24 pt |
| `Large` | 32 pt |
| `ExtraLarge` | 48 pt |

The table matches `MudIcon`’s, so a `MudRating::size(Size::Large)` is the same
pixel size as a `MudIcon::size_preset(Size::Large)` elsewhere in the same
screen.

### Constants

| Name | Value |
|---|---|
| `STAR_GAP` | `4.0` |
| `RIPPLE_DURATION` | `0.35` s |
| `RIPPLE_MAX_RADIUS_FACTOR` | `1.2` |
| `RIPPLE_ALPHA` | `60` |
| `EMPTY_ALPHA` | `0.38` |
| `DISABLED_ALPHA` | `0.38` |

---

## Examples

### Basic rating

```rust
let mut score = 3;
ui.add(MudRating::new(&mut score));
```

### Warning-coloured stars

```rust
MudRating::new(&mut score).color(Color::Warning).show(ui);
```

### Custom star count

```rust
MudRating::new(&mut score).max_value(10).show(ui);
```

### Read-only display

```rust
MudRating::new(&mut avg).read_only(true).show(ui);
```

The stars are full opacity, but nothing changes on click or hover.

### Disabled

```rust
MudRating::new(&mut avg).disabled(true).show(ui);
```

Dimmed to 38 % opacity, ignores pointer and keyboard.

### Custom icons (hearts)

```rust
MudRating::new(&mut score)
    .full_icon(filled::FAVORITE)
    .empty_icon(filled::FAVORITE_BORDER)
    .color(Color::Error)
    .show(ui);
```

### Custom colours

```rust
MudRating::new(&mut score)
    .color(Color::Warning)
    .full_icon_color(Color::Success)
    .empty_icon_color(Color::Error)
    .show(ui);
```

`FullIconColor` wins over `Color` for filled stars; `EmptyIconColor` sets the
unfilled glyph. When neither override is set, filled uses `Color` and empty uses
`Color` at 38 % gamma.

### Reacting to a change

```rust
let out = MudRating::new(&mut score).show(ui);
if out.changed {
    log::info!("score = {score}");
}
```

### Reacting to hover

```rust
let out = MudRating::new(&mut score).show(ui);
if let Some(hovered) = out.hovered_changed {
    if hovered == 0 {
        log::debug!("pointer left the rating");
    } else {
        log::debug!("hovering star {hovered}");
    }
}
```

### Keyboard-only

```rust
// With the rating focused:
//   → / ↑  increment
//   ← / ↓  decrement
//   Home   1
//   End    max_value
//   Space / Enter  select the hovered star, if any
```

### Standalone read-back

```rust
let score = 4;
let mut read_only = score;
let out = MudRating::new(&mut read_only).read_only(true).show(ui);
assert_eq!(out.changed, false);
```

### Tag round-trip

```rust
let rating = MudRating::new(&mut score).tag("product-rating");
assert_eq!(rating.tag_ref::<&str>(), Some(&"product-rating"));
```

---

## Binding and behaviour

### `SelectedValue` is `i32`, and `0` means unselected

MudBlazor’s `MudRating` uses a non-nullable `int` for the selected value, with
**`0` meaning “nothing selected”**. The port keeps that:

```rust
let mut score = 0;          // unselected
ui.add(MudRating::new(&mut score));
// After clicking star 3, `score == 3`.
```

There is no way to distinguish “selected 0” from “unselected” — there is no
“selected 0”. This is exactly MudBlazor’s behaviour.

### Hover preview

While the pointer is over star *K*:

- **Interactive** (`!read_only && !disabled`): stars `1..=K` render filled,
  stars `K+1..=max` render empty. On pointer leave, the display reverts to
  `*selected`.
- **ReadOnly or Disabled**: only the selected value is ever shown; the hover
  preview is suppressed entirely.

This is the intended 8.x behaviour. The 9.x changelog records the suppression as
a bug fix that brought read-only/disabled in line with this behaviour, and the
port applies the fix.

### Selection

| Interaction | Effect |
|---|---|
| Click star K | `*selected = K`; `changed = true` |
| `→` / `↑` (focused) | `*selected += 1`, clamped to `max_value` |
| `←` / `↓` (focused) | `*selected -= 1`, clamped to `0` |
| `Home` (focused) | `*selected = 1` |
| `End` (focused) | `*selected = max_value` |
| `Space` / `Enter` (focused) | Select the hovered star, if any |

A keyboard action only fires when the rating is focused and interactive. A
read-only or disabled rating still responds to *no* keyboard input, but a caller
can move the value through the `&mut i32` binding.

### `HoveredValueChanged` semantics

The Blazor callback fires with `int?`, where `null` means the pointer left the
rating. The Rust response maps that to `Option<i32>` with `0` standing in for
`None`:

```rust
if let Some(v) = out.hovered_changed {
    match v {
        0 => log::debug!("pointer left"),
        n => log::debug!("hovering {n}"),
    }
}
```

`hovered_changed` is `Some` only on the frame the hover moved. It is never
`Some` on the first frame the rating is drawn, matching the
`ExpandedChanged`-style contract used by `MudCollapse` and `MudExpansionPanel`.

---

## Performance and memory choices

| Choice | Effect |
|---|---|
| Theme fetched once per frame | One `MudTheme::fetch` and up to three `Color::resolve` calls (accent, full, empty). |
| One `allocate_exact_size` for the row | The whole rating is one rect and one `Response`. |
| One `ui.put` per star | `MudIcon` is the only per-star allocation, and it is the icon cache’s, not the component’s. |
| Pointer read from the row’s `Response` | `hover_pos()` and `interact_pointer_pos()` are already computed by egui; no extra hit-testing. |
| No `Vec`, no `String` | The caller’s `&mut i32` is the only binding; the response is three `Copy` fields plus the `Response`. |
| Hovered change detection in `egui::Memory` | One `i32` per rating, keyed by the row’s id; the component itself holds no state. |
| Ripple in `egui::Memory` | One `f64` per in-flight star; zero cost when `ripple(false)`. |
| Pure helpers on `Copy` values | Star size, row size, hit-test and keyboard mapping are arithmetic; unit-testable without a `Context`. |
| `read_only` / `disabled` skip pointer and keyboard | No hover, no click, no `Sense::click()` registration. |

---

## Testing

`cargo test -p mud_egui_components mud_rating` runs 19 pure-function tests:

| Area | Tests |
|---|---|
| Star pixel table | every `Size`; matches `MudIcon`’s scale |
| Row size | zero stars, one star, five stars |
| Star count sanitizer | zero, negative, positive |
| Hit-test | every slot of a five-star row; outside the row; in the gap between stars; zero stride |
| Star rect | first two stars; left-to-right layout |
| Display value | hover when interactive; suppressed when read-only or disabled; nothing selected |
| Selection clamp | negative → `0`; above max → max; `0` stays `0`; zero max |
| Keyboard | no key pressed → no action |
| Builders | defaults match MudBlazor (max **5**, `SelectedValue` **0**); every field set; tag round-trip |

No test creates a live `egui::Context`; the one keyboard test uses
`Context::run_ui` with an empty `RawInput`, which is safe and allocation-free.
`show` needs a live `Ui` and is therefore left to integration tests.

Suggested integration tests (`egui_kittest` or a headless `Context::run`
harness):

- Clicking star 3 sets `*selected == 3` and `changed == true`.
- Hovering star 4 previews four filled stars; leaving restores the selected
  count.
- `read_only(true)` blocks clicks and suppresses the hover preview.
- `disabled(true)` blocks clicks, suppresses the hover preview, and dims the
  stars.
- `→` on a focused rating increments; `←` decrements; `Home` sets `1`; `End`
  sets `max_value`.
- `hovered_changed` is `Some(new)` on the frame the hover moves and `None`
  otherwise.

```bash
cargo test -p mud_egui_components mud_rating
```

---

## Differences from MudBlazor

- **`SelectedValue` / `SelectedValueChanged` become a caller-owned `&mut i32`
  plus `RatingResponse::changed`.** The caller owns the value; `changed` is
  `true` on the frame it moved. This is the same split `MudCheckBox`,
  `MudRadio` and `MudCollapse` use. The `0`-means-unselected convention is
  preserved.
- **`HoveredValueChanged` becomes two fields on `RatingResponse`.**
  `hovered: i32` reports the current hover every frame; `hovered_changed:
  Option<i32>` is `Some(new)` on the frame the hover moved, with `0` mapped to
  Blazor’s `null`. The reporting is extended; the parameter surface is
  unchanged.
- **`Name` is stored but inert.** It becomes the HTML `<input name="…">` in
  Blazor, which egui has no equivalent for. Accepted so a Blazor call site
  ports without edits, documented here rather than silently dropped.
- **`RatingItemsClass` and `RatingItemsStyle` are not ported.** No CSS strings.
- **`Style` and `UserAttributes` are not ported.** No CSS strings, no HTML tag.
- **`ReadOnly` and `Disabled` suppress the hover preview.** The 9.x changelog
  records this as a bug fix on MudBlazor’s side (“Skip hover effects when
  ReadOnly or Disabled”); the port applies the intended 8.x behaviour.
- **`Size` uses `MudIcon`’s pixel table** (8 / 16 / 24 / 32 / 48), so a
  `MudRating` with `Size::Large` matches a `MudIcon` with `Size::Large` on the
  same screen. MudBlazor’s SCSS uses the same scale.
- **No half-star mode.** MudBlazor’s `MudRating` is integer-only; there is no
  `HalfIncrements` or `AllowHalf` parameter to port, and none was added.
- **Keyboard handling covers `Home` / `End`.** `HandleKeyDownAsync` in MudBlazor
  is arrow keys only; the port adds `Home` (set to `1`) and `End` (set to
  `max_value`), which are the standard keyboard interactions for a rating input.
  They do not change the ported parameter surface.
- **Default `Color` is `Default`, and `Color::Default` resolves to
  `ui.visuals().text_color()`.** MudBlazor’s `Color.Default` on a rating
  renders plain text-coloured stars; the port does the same, and setting
  `.color(Color::Warning)` or `.color(Color::Primary)` gives the familiar
  coloured stars.

===== ./crates/mud_egui_components/src/inputs/select/README.md =====

# MudSelect for `mud_egui`

A dropdown that shows one item at a time (or several in multi-selection mode),
for [egui](https://github.com/emilk/egui). Ported from MudBlazor's
`MudSelect<T>` / `MudSelectItem<T>` (MudBlazor 8.13.0).

```rust
let mut state = SelectState::<&str>::new("fruit").with_selected(["apple"]);

let (resp, _) = MudSelect::new()
    .label("Fruit")
    .variant(Variant::Outlined)
    .show(ui, &mut state, |s| {
        s.text_item("apple", "Apple");
        s.text_item("banana", "Banana");
        s.text_item("cherry", "Cherry");
    });
```

Trigger label / placeholder / helper / error render through **`MudText`**
(via `MudTextField`). List rows render text through **`MudText`**, icons and
checkboxes through **`MudIcon`**.

---

## Design principle: composition, not reimplementation

`MudSelect` is **built from two primitives** rather than a fresh paint path:

| Primitive | Role |
|-----------|------|
| [`MudTextField`](../../inputs/text_field/README.md) with `readonly(true)` | The closed trigger. Gives `Variant`, `Margin`, `Size`, `Color`, `Label`, `Placeholder`, `HelperText`, `Error`, `ErrorText`, and the adornment icon in one parameter sweep. |
| [`MudPopover`](../../overlays/popover/README.md) | The dropdown. Anchor attachment, flip-when-overflowing, outside-click, Escape dismissal all delegated. |

Only new code is the **list body** (rows, checkboxes, select-all) and the
**selection state machine** — and both are pure helpers tested without an
`egui::Context`.

This keeps the port honest: any change to the trigger's visual language flows
through automatically, and any change to popover positioning flows through too.

---

## Pass 1 scope and pass 2 deferral

### Ported parameters

| MudBlazor | Rust method | Default |
|-----------|-------------|---------|
| `Value` / `SelectedValues` | `SelectState::selected` (`Vec<T>`) | required |
| `ToStringFunc` | `tostring_func(f)` | `None` |
| `Label` | `label(s)` | `None` |
| `Placeholder` | `placeholder(s)` | `None` |
| `HelperText` | `helper_text(s)` | `None` |
| `Error` | `error(bool)` | `false` |
| `ErrorText` | `error_text(s)` | `None` |
| `Required` | `required(bool)` | `false` (visual only) |
| `Disabled` | `disabled(bool)` | `false` |
| `ReadOnly` | `readonly(bool)` | `false` |
| `Clearable` | `clearable(bool)` | `false` |
| `MultiSelection` | `multi_selection(bool)` | `false` |
| `SelectAll` | `select_all(bool)` | `false` |
| `Strict` | `strict(bool)` | `false` |
| `LockScroll` | `lock_scroll(bool)` | `false` (stored, no-op) |
| `Variant` | `variant(Variant)` | `Text` |
| `Margin` | `margin(Margin)` | `None` |
| `Color` | `color(Color)` | `Primary` |
| `Size` | `size(Size)` | `Medium` |
| `FullWidth` | `full_width(bool)` | `false` |
| `DisableUnderLine` | `disable_underline(bool)` | `false` |
| `IconSize` | `icon_size(Size)` | `Medium` |
| `AdornmentColor` | `adornment_color(Color)` | `None` |
| `OpenIcon` | `open_icon(s)` | `ARROW_DROP_DOWN` |
| `CloseIcon` | `close_icon(s)` | `ARROW_DROP_UP` |
| `AnchorOrigin` | `anchor_origin(AnchorOrigin)` | `None` |
| `TransformOrigin` | `transform_origin(TransformOrigin)` | `None` |
| `Position` | `position(Position)` | `None` |
| `MaxHeight` | `max_height(f32)` | `300.0` |
| `PopoverClass` | `popover_class(s)` | `None` |
| `Dense` | `dense(bool)` | `false` |
| `OnOpen` | `on_open(f)` | `None` |
| `OnClose` | `on_close(f)` | `None` |
| `OnClearButtonClick` | `on_clear_button_click(f)` | `None` |
| `SelectedValuesChanged` | `on_selected_values_changed(f)` | `None` |
| `ChildContent` | `add_items` closure of `show` | required |

### Ported `MudSelectItem` parameters

| MudBlazor | Rust method | Default |
|-----------|-------------|---------|
| `Value` | `new(value)` / `text(value, s)` | required |
| `Text` | `text(s)` / `with_text(s)` | `None` |
| `Icon` | `icon(s)` | `None` |
| `Disabled` | `disabled(bool)` | `false` |
| `ChildContent` | `child(f)` | `None` |

### Deferred to pass 2

The following MudBlazor parameters are **deferred** and documented here so
the deferral is visible:

| Deferred parameter | Why pass 2 |
|--------------------|------------|
| `Counter` / `MaxLength` / `Pattern` | Form-level validation display |
| `For` / `Validation` / `OnlyValidateIfDirty` | Form validation model |
| `RequiredError` | Same as `Required` (pass 1 ships visual-only) |
| `Converter` / `Culture` / `Format` | Value formatting outside the `ToStringFunc` path |
| `Delimiter` / `MultiSelectionTextFunc` | Multi-selection text customisation |
| `AutoFocus` / `Immediate` / `InputMode` | Input internals |
| `KeyDownPreventDefault` / `KeyUpPreventDefault` | Input internals |
| `TextUpdateSuppression` | Browser-specific |
| `HelperTextOnFocus` | Helper row behaviour |
| `ShrinkLabel` | Floating label behaviour |
| `InputClass` / `OuterClass` | CSS strings, no egui equivalent |
| `CheckedIcon` / `IndeterminateIcon` | Default Material glyphs suffice for pass 1 |
| `ListClass` | Item list container classes |
| `AdornmentAriaLabel` | Accessibility refinement |
| `SelectRangeAsync` / `SelectOption` methods | Programmatic navigation; write to `SelectState::selected` directly |

### Not ported

- **`Class` (base)** — accepted via `classes(..)` on the trigger.
- **`Style` / `Tag` / `UserAttributes`** — no CSS strings or HTML tag in egui.

---

## Architecture

```
inputs/select/
├── select_action.rs   — SelectAction<T> (extension)
├── select_item.rs     — SelectItem<T>
├── select_state.rs    — SelectState<T> + SelectUi<T>
├── mud_select.rs      — MudSelect + pure helpers + tests
└── mod.rs
```

**Pure layer.** `compute_display_text`, `apply_selection`,
`apply_select_all`, `all_enabled_selected`, `some_enabled_selected`,
`select_all_icon`, `item_checkbox_icon`, `trigger_icon`, `row_height`,
`position_to_origins` — free functions, unit-tested without `egui::Context`.

**State layer.** `SelectState<T> { selected: Vec<T>, popover: PopoverState }` —
the egui equivalent of MudBlazor's `SelectedValues` binding plus the popover
state that backs the dropdown.

**Component layer.** `show` computes the display text, renders the trigger
(`MudTextField`), handles trigger clicks, then delegates to `MudPopover` for
the list. List row rendering + selection state machine live in
`render_item_row` and the pure helpers.

---

## API reference — `MudSelect<T>`

| Method | Type | Default | Description |
|--------|------|---------|-------------|
| `new()` | — | — | Default select. |
| `tostring_func(f)` | `Fn(&T) -> String` | `None` | Value-to-display-text mapping. |
| `error(bool)` / `error_text(s)` | `bool` / `impl Into<String>` | `false` / `None` | Error visual only. |
| `required(bool)` | `bool` | `false` | Visual only (see README). |
| `disabled(bool)` / `readonly(bool)` | `bool` | `false` | Interaction flags. |
| `clearable(bool)` | `bool` | `false` | Shows a clear button on the trigger. |
| `multi_selection(bool)` | `bool` | `false` | Multi-value selection. |
| `select_all(bool)` | `bool` | `false` | "Select all" tri-state row (multi only). |
| `strict(bool)` | `bool` | `false` | Refuses to display unmapped values. |
| `lock_scroll(bool)` | `bool` | `false` | Stored; no-op in egui. |
| `label(s)` / `placeholder(s)` / `helper_text(s)` | `impl Into<String>` | `None` | Trigger text slots. |
| `variant(Variant)` / `margin(Margin)` / `color(Color)` / `size(Size)` | enum | `Text` / `None` / `Primary` / `Medium` | Trigger appearance. |
| `full_width(bool)` / `disable_underline(bool)` / `icon_size(Size)` | `bool` / `Size` | `false` / `false` / `Medium` | Additional trigger flags. |
| `adornment_color(Color)` / `open_icon(s)` / `close_icon(s)` | `Color` / `impl Into<String>` | `None` / `ARROW_DROP_DOWN` / `ARROW_DROP_UP` | Adornment. |
| `position(Position)` / `anchor_origin(..)` / `transform_origin(..)` | placement | `None` | Popover placement. |
| `right_to_left(bool)` | `bool` | `false` | RTL flag. |
| `max_height(f32)` / `popover_class(s)` | `f32` / `impl Into<String>` | `300.0` / `None` | List box. |
| `dense(bool)` | `bool` | `false` | Compact list rows. |
| `on_open(f)` / `on_close(f)` / `on_clear_button_click(f)` | `FnMut()` | `None` | Callbacks. |
| `on_selected_values_changed(f)` | `FnMut(&[T])` | `None` | Selection-change callback. |
| `classes(iter)` / `flavor(StyleFlavor)` | — | — | Utilities. |
| `show(ui, state, \|s\| ..)` | `(&mut Ui, &mut SelectState<T>, FnOnce(&mut SelectUi<T>) -> R)` | — | Returns `(SelectResponse<T>, R)`. |

### `SelectResponse<T>`

| Field | Type | Description |
|-------|------|-------------|
| `trigger` | `egui::Response` | Trigger's response. |
| `selected` | `Vec<T>` | Selection **after** this frame's click. |
| `action` | `SelectAction<T>` | What happened this frame. |

### `SelectState<T>`

| Field / method | Description |
|----------------|-------------|
| `selected: Vec<T>` | Current selection. |
| `popover: PopoverState` | Open/closed + stable id. |
| `new(id_salt)` / `with_selected(iter)` | Constructor / seed. |
| `is_open()` / `open()` / `close()` / `toggle()` | Popover control. |
| `is_selected(&T)` | Membership check. |

### `SelectItem<T>`

| Method | Description |
|--------|-------------|
| `new(value)` | Value with no text. |
| `text(value, s)` | Value + display text. |
| `with_text(s)` / `icon(s)` / `disabled(b)` / `child(f)` | Builders. |

### `SelectAction<T>`

| Variant | Description |
|---------|-------------|
| `None` | No interaction. |
| `Opened` / `Closed` | Dropdown opened / closed. |
| `Selected { value, multi }` | Item selected. |
| `Deselected { value }` | Item deselected (multi only). |
| `Cleared` | Clear button pressed. |
| `SelectAll { select }` | Select-all toggled. |

Helpers: `changed_selection()`, `toggled_open()`.

---

## Examples

### Single selection

```rust
let mut state = SelectState::<u32>::new("age").with_selected([18]);
MudSelect::new()
    .label("Age")
    .show(ui, &mut state, |s| {
        for n in [18, 25, 30, 40, 55] {
            s.text_item(n, format!("{n}"));
        }
    });
```

### Outlined variant

```rust
MudSelect::new()
    .label("Country")
    .variant(Variant::Outlined)
    .show(ui, &mut state, |s| {
        s.text_item("us", "United States");
        s.text_item("gb", "United Kingdom");
    });
```

### Multi-selection with select-all

```rust
let mut state = SelectState::<&str>::new("tags").with_selected(["rust"]);
MudSelect::new()
    .label("Tags")
    .multi_selection(true)
    .select_all(true)
    .show(ui, &mut state, |s| {
        s.text_item("rust", "Rust");
        s.text_item("go", "Go");
        s.text_item("zig", "Zig");
    });
```

### Clearable

```rust
MudSelect::new()
    .clearable(true)
    .show(ui, &mut state, |s| { /* items */ });
```

### With `tostring_func` (no item text)

```rust
MudSelect::new()
    .tostring_func(|v: &u32| format!("{v}"))
    .show(ui, &mut state, |s| {
        for n in 0..10 { s.value(n); }
    });
```

### With icons

```rust
MudSelect::new().show(ui, &mut state, |s| {
    s.item(SelectItem::text("home", "Home").icon(filled::HOME));
    s.item(SelectItem::text("work", "Work").icon(filled::WORK));
});
```

### Dense

```rust
MudSelect::new()
    .dense(true)
    .show(ui, &mut state, |s| { /* items */ });
```

### Above the anchor

```rust
MudSelect::new()
    .position(Position::Top)
    .show(ui, &mut state, |s| { /* items */ });
```

### Callbacks

```rust
MudSelect::new()
    .on_open(|| log::info!("opened"))
    .on_close(|| log::info!("closed"))
    .on_selected_values_changed(|sel| log::info!("selection: {sel:?}"))
    .show(ui, &mut state, |s| { /* items */ });
```

### Custom value type (non-`Copy`)

```rust
#[derive(Clone, PartialEq)]
struct Fruit { id: u32, name: String }

let mut state = SelectState::<Fruit>::new("fruit");
MudSelect::new()
    .tostring_func(|f| f.name.clone())
    .show(ui, &mut state, |s| {
        s.item(SelectItem::text(
            Fruit { id: 1, name: "Apple".into() }, "Apple"
        ));
        s.item(SelectItem::text(
            Fruit { id: 2, name: "Banana".into() }, "Banana"
        ));
    });
```

### Read the outcome

```rust
let (resp, _) = MudSelect::new().show(ui, &mut state, |s| { /* items */ });
match resp.action {
    SelectAction::Selected { value, multi } => log::info!("selected {value:?} (multi: {multi})"),
    SelectAction::Deselected { value } => log::info!("deselected {value:?}"),
    SelectAction::Cleared => log::info!("cleared"),
    SelectAction::SelectAll { select } => log::info!("select-all: {select}"),
    SelectAction::Opened => log::info!("opened"),
    SelectAction::Closed => log::info!("closed"),
    SelectAction::None => {}
}
```

---

## Trigger display text

`compute_display_text` derives the trigger string from the current selection
using this precedence per value:

1. Matching item's `text` (looked up by `PartialEq` value match).
2. `tostring_func(&value)`.
3. `"?"` (a visible marker; the value was bound but not rendered).

Multiple selections are joined with `MULTI_DELIMITER = ", "`. All three paths
are unit-tested (`compute_display_text_*`).

---

## Select-all behaviour

When `multi_selection(true)` and `select_all(true)`, a select-all row is
rendered above the item list. Its checkbox is tri-state:

| State | Glyph |
|-------|-------|
| All enabled items selected | `CHECK_BOX` |
| Some enabled items selected | `INDETERMINATE_CHECK_BOX` |
| None selected | `CHECK_BOX_OUTLINE_BLANK` |

Clicking the row:
- **Not all selected** → selects every non-disabled item.
- **All selected** → clears the selection entirely.

Disabled items are skipped by `apply_select_all(.., true)`. The matching
predicates (`all_enabled_selected`, `some_enabled_selected`) also ignore
disabled items.

---

## Performance and memory choices

| Choice | Effect |
|--------|--------|
| `compute_display_text` / `apply_selection` / `apply_select_all` are pure | Unit-tested without `egui::Context`. |
| `SelectState` is `Clone` (not `Copy`) when `T` is `Clone` | Matches `T`'s bounds. |
| Display text is a **local** `String` | No per-frame allocation on `state`; dropped when the trigger returns. |
| Trigger reuses `MudTextField` | Zero duplication of variant / margin / size logic. |
| List reuses `MudPopover` | Zero duplication of anchor / flip / dismiss. |
| Only the trigger's response is inspected for clicks | The field unions its adornment response, so adornment clicks register. |
| List rows use `allocate_exact_size` + child `Ui`s | No per-row layout pass. |
| `MudIcon` reuse across rows | Free thanks to its transform cache. |
| `MudText` for all labels | Theme and typography propagate automatically. |
| `selected_snapshot` is cloned once before the popover renders | Cheap if `T: Copy`; uses `T: Clone` otherwise. |

Not done, on purpose: virtualising the item list. MudBlazor renders every
visible item; the port does the same. For hundreds of items, wrap the
collector in a filter that pushes fewer items.

---

## Testing

`cargo test` runs 30+ pure-function tests:

| Area | Tests |
|------|-------|
| `row_height` | dense / normal |
| `trigger_icon` | open / closed |
| `item_text_for` | text match, no-text item, missing value |
| `compute_display_text` | empty, item-text priority, `tostring_func` fallback, unknown → `?`, delimiter join |
| `apply_selection` | single replace, single re-click noop, multi add/remove |
| `apply_select_all` | adds enabled, skips disabled, clears |
| `all_enabled_selected` / `some_enabled_selected` | full / partial / empty |
| `select_all_icon` | tri-state mapping |
| `item_checkbox_icon` | selected / not |
| `position_to_origins` | bottom / top / left / right / start under RTL |
| Defaults | every `MudSelect` field matches MudBlazor |
| Builders | every field set |
| `SelectState` | open / close / toggle / is_selected |
| `SelectAction` | `changed_selection` / `toggled_open` |
| `SelectItem` | constructors, builders |
| Collector | preserves order |

Rendering (`show`) needs a live `egui::Context`; the pure layer is the
source of truth.

```bash
cargo test -p mud_egui_components mud_select
```

---

## Differences from MudBlazor

- **Trigger is a `MudTextField` in `readonly(true)` mode.** MudBlazor uses
  the same composition (`MudTextField` as the input). The Rust port matches.
- **`Value` / `SelectedValues` unified into `SelectState::selected: Vec<T>`.**
  MudBlazor has both a scalar `Value` and a `SelectedValues` collection;
  the port uses the collection always and lets single-selection mode leave
  at most one value. This matches the underlying storage in MudBlazor.
- **`strict` is stored but not enforced in pass 1.** MudBlazor hides the
  display text when the value is not in the item list. The port's
  `compute_display_text` currently falls back to `tostring_func` / `?` for
  missing items. Full `strict` behaviour is a one-line guard deferred to
  pass 2.
- **`LockScroll` is a no-op.** egui `Area`s do not participate in scroll
  containers.
- **`required` is visual only.** No form-level validation.
- **`Adornment` restricted to `End`.** MudBlazor's `Adornment.Start` is not
  exposed because the select trigger is a single field; use
  `MudTextField::adornment_end(..)` directly if you need a custom leading
  adornment.
- **`Clearable` is single-select only.** Multi-selection clears via
  `select_all(false)`. MudBlazor's `Clearable` is also single-select only.
- **Form validation parameters deferred.** See the "Deferred" table.
- **`Style` / `Tag` / `UserAttributes` not ported.** No CSS strings or HTML
  tags.

===== ./crates/mud_egui_components/src/inputs/slider/README.md =====

# MudSlider for `mud_egui`

A single-value slider for [egui](https://github.com/emilk/egui), ported from
MudBlazor's [`MudSlider<T>`](https://mudblazor.com/components/slider)
(MudBlazor 8.13.0).

```rust
let mut volume = 50.0_f32;

ui.add(
    MudSlider::new(&mut volume, 0.0, 100.0)
        .color(Color::Primary)
        .show_value(true)
        .value_format("{:.0}%"),
);
```

The label, tick labels and value bubble all render through **`MudText`**, so
the active `MudTheme` typography and colours propagate to every visible string
in the component. The rail, fill, ticks, halo, thumb and bubble background are
painted directly with `egui::Painter`.

---

## Contents

1. [Design principle: API parity, not API invention](#design-principle-api-parity-not-api-invention)
2. [Architecture](#architecture)
3. [API reference](#api-reference)
4. [Examples](#examples)
5. [Layout and rendering](#layout-and-rendering)
6. [Performance and memory choices](#performance--memory-choices)
7. [Testing](#testing)
8. [Differences from MudBlazor](#differences-from-mudblazor)

---

## Design principle: API parity, not API invention

The MudBlazor 8.13.0 `MudSlider<T>` parameter surface was verified against the
published API reference before this port. Fourteen parameters are ported; five
are deliberately omitted; one Rust-only extension is added.

### Ported parameters

| MudBlazor | Type | Default | Rust mapping |
|---|---|---|---|
| `Value` | `T` | — | bound `&mut f32` |
| `ValueChanged` | `EventCallback<T>` | — | `Response::changed` on the returned `Response` |
| `Min` | `double` | `0.0` | `MudSlider::new` `min` |
| `Max` | `double` | `100.0` | `MudSlider::new` `max` |
| `Step` | `double` | `1.0` | `step(f32)` |
| `Vertical` | `bool` | `false` | `vertical(bool)` |
| `Disabled` | `bool` | `false` | `disabled(bool)` |
| `Color` | `Color` | `Primary` | `color(Color)` |
| `Size` | `Size` | `Small` | `size(Size)` |
| `Variant` | `Variant` | `Text` | `variant(Variant)` |
| `TickMarks` | `bool` | `false` | `ticks(bool)` |
| `MarkerLabels` | `string[]?` | `null` | `tick_labels(&[&str])` |
| `ValueLabel` | `bool` | `false` | `show_value(bool)` |
| `ValueLabelFormat` | `Func<double, string>` | `null` | `value_format(&str)` — template string form |
| `ChildContent` (label) | `RenderFragment` | — | `label(&str)` — string form |

`ValueLabelFormat` is a `Func<double, string>` in Blazor. A Rust port cannot
take a generic `Fn(f64) -> String` on a `#[must_use]` builder without
introducing a trait object lifetime, so it accepts a **template string**
instead: `"{}"`, `"{:.2}"`, `"${:.2}"`, `"{} px"`. The common cases all work;
a dynamic format (e.g. `"{} / {}"` filled with two values) is not supported and
should be expressed by the caller after reading `*value`.

### Deliberately not ported

| MudBlazor | Reason |
|---|---|
| `Range` | Two-thumb range mode is a separate widget shape; the interaction model (which thumb is being dragged), the layout, and the display all differ from the single-thumb slider. Adding it would roughly double the surface, and it deserves its own component (`MudRangeSlider`) rather than a boolean mode. |
| `RangeValue` / `RangeValueChanged` | Paired with `Range`; same reason. |
| `Immediate` | MudBlazor defaults to firing `ValueChanged` on release; in an immediate-mode UI every frame is a “release”, so `Response::changed` fires every frame the value moves. This port therefore behaves as `Immediate = true` always. |
| `Culture` | Rust's `format!` is not culture-aware. Use a `value_format(..)` template, or format the value yourself after reading `*value`. |
| `LabelAttributes` / `InputAttributes` | No HTML attributes in egui. |
| `Class` / `Style` / `Tag` / `UserAttributes` | No CSS strings, no HTML tag. `Tag` is not exposed because there is no `Any`-typed slot on the builder; callers can wrap the slider in their own struct if they need one. |

### Extension

| Extension | Type | Default | Purpose |
|---|---|---|---|
| `length(f32)` | `f32` | `None` | Explicit length along the slider's axis, in points. MudBlazor relies on CSS width; `length` is the typed equivalent and the only reason a caller would reach for `Style` on this component. |

### Rule 3 compliance

Every visible string — the label, tick labels, and value-bubble text — goes
through `MudText`. No user-facing text uses `painter.text` or `painter.galley`;
only the rail, fill, ticks, halo, thumb and bubble **background** are painted
directly. This means the active `MudTheme` typography and colours apply to
every label, exactly as they do for `MudRating`, `MudCheckBox` and the other
input components.

The label above the slider is drawn with `MudText::body1`. The tick labels and
the value bubble use `MudText` with `Typo::Body2` and a `font_size` override of
12 pt, matching MudBlazor's `.mud-slider-marker-label` and value-label
typography.

---

## Architecture

```
inputs/slider/
└── mud_slider.rs
    ├── constants            MIN_TICKS 2, MAX_TICKS 30, HIT_THICKNESS 28,
    │                        LABEL_ROW_EXTRA 12, TICK_LABEL_OFFSET 18,
    │                        TICK_OVERHANG 2, DEFAULT_VERTICAL_LENGTH 160,
    │                        FALLBACK_LENGTH 200, TRACK_ALPHA 0.38,
    │                        DISABLED_FACTOR 0.5, HALO_HOVER_EXTRA 8,
    │                        HALO_PRESS_EXTRA 6, HALO_ALPHA 0.16,
    │                        ANIM_TIME 0.12, LABEL_FONT_SIZE 12,
    │                        BUBBLE_* geometry, CONTRAST_LUMINANCE_THRESHOLD 0.45
    ├── SizeMetrics          rail / thumb per Size
    ├── Axis                 start..end line, point_at / fraction_of
    ├── RenderOpts           resolved inputs for render_slider
    ├── render_slider        allocation, interaction, painting
    ├── draw_value_bubble    background + pointer + MudText for text
    ├── pure helpers         size_metrics, value_to_fraction, fraction_to_value,
    │                        snap_to_step, step_decimals, format_value,
    │                        format_with_template, tick_count, tick_fraction,
    │                        is_ambient_color, contrast_text, usable_or,
    │                        label_font, keyboard_adjust, resolve_color
    └── impl Widget          ui.add(MudSlider::new(..))
```

**Pure layer.** Every geometric and formatting decision is a free function
over `f32`, `Rect` and `Pos2`. `Axis` wraps the line the thumb travels along;
it works for both orientations so the interaction math is not duplicated.

**Component layer.** `show` resolves theme, colour and metrics once, then
delegates to `render_slider` for allocation and painting. `render_slider` is a
free function so it can borrow `value: &mut f32` directly and does not need
`&mut self`.

**Text layer.** All three text sites — the top label, tick labels, and value
bubble — go through `MudText`. Tick labels and bubble text are placed in a
child `Ui` sized to the measured text, so a `MudText::inline(true)` fills its
rect exactly and the text appears centred on the target point.

---

## API reference

| Method | Type | Default | Mirrors | Description |
|---|---|---|---|---|
| `new(value, min, max)` | `&'a mut f32, f32, f32` | — | `Value` + `Min` + `Max` | Constructs a slider. |
| `step(s)` | `f32` | `1.0` | `Step` | Snap increment. `<= 0` is continuous. |
| `variant(v)` | `Variant` | `Text` | `Variant` | `Filled` fills the rail up to the thumb. |
| `color(c)` | `Color` | `Primary` | `Color` | Rail / thumb / tick / bubble colour. |
| `size(s)` | `Size` | `Small` | `Size` | Rail thickness + thumb diameter. |
| `label(s)` | `impl Into<&'a str>` | `None` | `ChildContent` (string) | Text label above (horizontal) or beside (vertical). |
| `disabled(b)` | `bool` | `false` | `Disabled` | Dims to 50 % and disables interaction. |
| `vertical(b)` | `bool` | `false` | `Vertical` | Runs bottom-to-top instead of left-to-right. |
| `show_value(b)` | `bool` | `false` | `ValueLabel` | Shows the value bubble on hover / press / focus. |
| `value_format(fmt)` | `impl Into<&'a str>` | `None` | `ValueLabelFormat` | Template for the bubble value. |
| `ticks(b)` | `bool` | `false` | `TickMarks` | Draws ticks on the rail. |
| `tick_labels(labels)` | `&'a [&'a str]` | `None` | `MarkerLabels` | Per-tick labels (horizontal only). |
| `length(px)` | `f32` | `None` | *(extension)* | Explicit length along the axis. |
| `show(ui)` | `&mut Ui` | — | — | Returns the slider's `Response`. |
| `ui.add(..)` | — | — | — | Same as `show`. |

`Response::changed()` is `true` on the frame the bound value changed.

### `Size` mapping

| `Size` | Rail thickness | Thumb diameter |
|---|---|---|
| `ExtraSmall` / `Small` | 2 pt | 16 pt |
| `Medium` | 4 pt | 20 pt |
| `Large` / `ExtraLarge` | 6 pt | 24 pt |

`ExtraSmall` folds into `Small` and `ExtraLarge` into `Large`, matching
MudBlazor's three-slider-size model.

### Constants

| Name | Value |
|---|---|
| `HIT_THICKNESS` | `28.0` |
| `LABEL_ROW_EXTRA` | `12.0` |
| `TICK_LABEL_OFFSET` | `18.0` |
| `TICK_OVERHANG` | `2.0` |
| `DEFAULT_VERTICAL_LENGTH` | `160.0` |
| `FALLBACK_LENGTH` | `200.0` |
| `TRACK_ALPHA` | `0.38` |
| `DISABLED_FACTOR` | `0.5` |
| `HALO_HOVER_EXTRA` / `HALO_PRESS_EXTRA` / `HALO_ALPHA` | `8` / `6` / `0.16` |
| `ANIM_TIME` | `0.12` s |
| `LABEL_FONT_SIZE` | `12.0` |
| `MIN_TICKS` / `MAX_TICKS` | `2` / `30` |
| `CONTRAST_LUMINANCE_THRESHOLD` | `0.45` |

---

## Examples

### Basic

```rust
let mut volume = 50.0_f32;
ui.add(MudSlider::new(&mut volume, 0.0, 100.0));
```

### Filled variant

```rust
ui.add(
    MudSlider::new(&mut volume, 0.0, 100.0)
        .variant(Variant::Filled)
        .color(Color::Primary),
);
```

### Larger size

```rust
ui.add(MudSlider::new(&mut volume, 0.0, 100.0).size(Size::Large));
```

### With a top label

```rust
ui.add(
    MudSlider::new(&mut volume, 0.0, 100.0)
        .label("Volume")
        .show_value(true)
        .value_format("{:.0}%"),
);
```

### With ticks and labels

```rust
const LABELS: &[&str] = &["Poor", "Fair", "Good", "Great", "Excellent"];
let mut rating = 3.0_f32;

ui.add(
    MudSlider::new(&mut rating, 1.0, 5.0)
        .step(1.0)
        .ticks(true)
        .tick_labels(LABELS),
);
```

### Vertical with an explicit length

```rust
let mut level = 10.0_f32;

ui.add(
    MudSlider::new(&mut level, 0.0, 20.0)
        .vertical(true)
        .length(200.0)
        .label("Level")
        .show_value(true),
);
```

### Disabled

```rust
ui.add(
    MudSlider::new(&mut volume, 0.0, 100.0)
        .disabled(true)
        .label("Locked"),
);
```

### Continuous (no step)

```rust
ui.add(MudSlider::new(&mut volume, 0.0, 1.0).step(0.0));
```

### Custom template

```rust
ui.add(
    MudSlider::new(&mut price, 0.0, 100.0)
        .show_value(true)
        .value_format("${:.2}"),
);
```

### Reading the changed flag

```rust
let response = MudSlider::new(&mut volume, 0.0, 100.0).show(ui);
if response.changed() {
    log::info!("volume = {volume}");
}
```

### Keyboard

With the slider focused:

| Key | Effect |
|---|---|
| `→` / `↑` | `+= step` |
| `←` / `↓` | `-= step` |
| `PageUp` | `+= 10 × step` |
| `PageDown` | `-= 10 × step` |
| `Home` | Set to `min` |
| `End` | Set to `max` |

---

## Layout and rendering

### Horizontal slider

```
[ label           ]   ← MudText::body1 above the strip
[ ●─────────────── ]   ← rail (translucent) + fill up to thumb (Filled)
   Poor  Fair ...      ← tick labels, centred under each tick
```

- **Height:** 28 pt (`HIT_THICKNESS`), plus 12 pt for the tick-label row when
  labels are shown.
- **Width:** `ui.available_width()`, clamped to `length` when set.
- **Rail:** 2 / 4 / 6 pt tall (per `Size`), translucent at `0.38` α.
- **Fill:** up to the thumb, only for `Variant::Filled`.
- **Halo:** `HALO_HOVER_EXTRA` while hovered / pressed, `HALO_PRESS_EXTRA`
  additional while pressed.
- **Thumb:** circular, `SizeMetrics.thumb` diameter.

### Vertical slider

```
[    ]   [ label    ]
[ ●  ]
[    ]
[    ]
```

The track runs bottom-to-top; `min` sits at the bottom. The label is placed to
the left (horizontal container), same as `MudSlider`'s Blazor layout for
`Vertical`.

### Value bubble

Painted only while the slider is hovered, pressed, or keyboard-focused. It sits
`HALO_HOVER_EXTRA + BUBBLE_GAP + BUBBLE_ARROW_HEIGHT` above the thumb centre
(to the right of it when vertical) with a 5 pt pointer aimed at the thumb.

The bubble **background and pointer** are painted on the slider's own layer
via `ui.painter()`; the bubble **text** goes through `MudText`. This means the
bubble is guaranteed to be on top of the slider itself, but not guaranteed to
escape a clip imposed by an ancestor (e.g. a small `ScrollArea`). See the
[`Rule 3 compliance`](#rule-3-compliance) section above.

### Degenerate inputs

| Input | Behaviour |
|---|---|
| `max <= min` | The slider is inert; the thumb sits at `min`, no interaction. |
| Non-finite `value` | Clamped to `min`; the caller's value is untouched unless the user drags. |
| Non-finite `length` | Falls back to the available width or `DEFAULT_VERTICAL_LENGTH`. |
| `step <= 0` | Continuous drag; no snapping. |
| Tick count outside `2..=30` | Ticks are silently skipped rather than drawn illegibly dense. |

---

## Performance and memory choices

| Choice | Effect |
|---|---|
| One `Painter` clone per `render_slider` | Releases the immutable `ui` borrow so child `Ui`s (for `MudText`) can open later. Cheap: clones an `Arc` and copies two scalar fields. |
| Theme fetched once per frame | One `MudTheme::fetch` and up to one `Color::resolve`. |
| One `allocate_exact_size` for the strip | The whole control is one rect and one `Response`. |
| Pure helpers on `Copy` values | Rail thickness, thumb diameter, value→fraction, tick count/fraction and template formatting are arithmetic; unit-testable without a `Context`. |
| No `Vec`, no `String` on the hot path | The label and tick labels borrow from the caller; the bubble string is built once per active frame. |
| `MudText` in a child `Ui` per label | One `Ui` per label/tick, but each is scoped to its measured rect and discarded after. |
| Halo and press fades use `animate_bool_with_time` | Idle at zero cost when nothing changes. |
| `painter.layout_no_wrap` for measurement only | Used to size the child `Ui` rect so `MudText::inline(true)` fills it exactly; never used to paint text. |

---

## Testing

`cargo test -p mud_egui_components mud_slider` runs 30+ pure-function tests:

| Area | Tests |
|---|---|
| `size_metrics` | every `Size`; edge sizes fold inwards; largest thumb fits the strip |
| `value_to_fraction` | maps, clamps, degenerate range |
| `fraction_to_value` / `snap_to_step` | snap to step, clamp at max, non-positive step is continuous, offset from `min` |
| `Axis` | horizontal / vertical mapping, zero-length safety |
| `format_with_template` | bare `{}`, `{:.N}`, literal text preserved, unmatched brace, non-numeric precision, only first placeholder |
| `step_decimals` / `format_value` | step-derived precision; f32 noise hidden; templates applied |
| `tick_count` | invalid step, `max <= min`, degenerate, dense, normal, inclusive boundaries |
| `tick_fraction` | even spacing, drops ticks past `max`, float-error tolerance |
| `is_ambient_color` / `contrast_text` | every ambient variant; dark-on-light and white-on-brand |
| `usable_or` | valid, zero, negative, non-finite |

No test creates an `egui::Context`. `MudSlider::show`, `render_slider`,
`draw_value_bubble`, `keyboard_adjust`, `label_font` and `resolve_color` need a
live `Ui` and are left to integration tests.

Suggested integration tests (`egui_kittest` or a headless `Context::run`):

- Dragging the thumb to the middle of a `0..=100` slider sets the value to
  `50`.
- Every visible string (label, tick labels, bubble text) is rendered by
  `MudText` — assert by mutating the theme's typography and confirming the
  rendered font changes.
- The bubble is shown only while the slider is hovered / pressed / focused.
- `Home` / `End` jump to the ends when the slider is focused.
- A disabled slider ignores pointer and keyboard.

```bash
cargo test -p mud_egui_components mud_slider
```

---

## Differences from MudBlazor

- **Every visible string goes through `MudText`.** The pre-refactor port drew
  tick labels and the value bubble with `painter.text` / `painter.galley` and
  `TextStyle::Body.resolve(ui.style())`, bypassing the active `MudTheme`. They
  now render through `MudText` and inherit the theme's `Typo::Body2`
  typography at a 12 pt override, matching MudBlazor's caption scale.
- **The value bubble lives on the slider's own layer, not the tooltip layer.**
  egui offers no way to construct a `Ui` on an arbitrary layer, and `MudText`
  needs a `Ui`; a bubble that satisfies Rule 3 therefore cannot live on the
  tooltip layer. The bubble is drawn last on the current layer, so it is on
  top of the slider itself; a tight ancestor clip (a small `ScrollArea`) may
  still clip it, where the previous implementation would have escaped. This is
  a deliberate Rule 3 trade-off, documented here rather than silently dropped.
- **`ValueLabelFormat` is a template string, not a `Func<double, string>`.**
  The common `{}` / `{:.N}` placeholder forms are supported by
  `format_with_template`; anything else should be formatted by the caller
  after reading `*value`. The template is applied by `format_value` after
  rounding to the step's precision, so f32 noise never shows.
- **`Immediate` is not a parameter.** This port behaves as `Immediate = true`
  always, because in immediate mode `Response::changed` fires every frame the
  value moves. `Immediate = false` (fire on release) has no meaning here.
- **`Range` / `RangeValue` / `RangeValueChanged` are not ported.** Range
  sliders need a separate component (`MudRangeSlider`) with a different
  interaction model.
- **`Culture` is not ported.** Rust's `format!` is not culture-aware; use
  `value_format(..)` or format the value yourself.
- **`length(f32)` is an extension.** MudBlazor relies on CSS width; `length`
  is the typed equivalent. Vertical sliders default to
  `DEFAULT_VERTICAL_LENGTH` (160 pt) when `length` is unset, because there is
  no “available height” to stretch to.
- **`WidgetInfo::slider` is called with the correct two-argument form.**
  egui’s `slider(value, label)` constructor does not take an `enabled` flag;
  `enabled` is set on the returned `WidgetInfo` afterwards, so a disabled
  slider is reported as disabled to assistive technology.

===== ./crates/mud_egui_components/src/inputs/switch/README.md =====

# MudSwitch for `mud_egui`

A two-state toggle control for [egui](https://github.com/emilk/egui), ported
from MudBlazor's [`MudSwitch<T>`](https://mudblazor.com/components/switch)
(MudBlazor 8.13.0).

```rust
let mut notifications = true;

ui.add(
    MudSwitch::new(&mut notifications)
        .label("Email notifications")
        .label_placement(Placement::End)
        .color(Color::Success),
);

if !notifications {
    mute();
}
```

Every string renders through **`MudText`**; the thumb glyph renders through
**`MudIcon`**. The track and thumb themselves are painted directly.

---

## Contents

1. [Design principle: API parity, not API invention](#design-principle-api-parity-not-api-invention)
2. [Architecture](#architecture)
3. [API reference](#api-reference)
4. [Examples](#examples)
5. [Layout and rendering](#layout-and-rendering)
6. [Performance and memory choices](#performance--memory-choices)
7. [Testing](#testing)
8. [Differences from MudBlazor](#differences-from-mudblazor)

---

## Design principle: API parity, not API invention

The MudBlazor 8.13.0 parameter surface was verified against the DNDocs pages
for `MudSwitch<T>` and its parent `MudBooleanInput<T>` before implementation.

### Switch-specific parameters

| MudBlazor | Type | Default | Rust mapping |
|---|---|---|---|
| `ThumbIcon` | `string?` | `null` | `thumb_icon(s)` |
| `ThumbIconColor` | `Color` | `Default` | `thumb_icon_color(Color)` — ignored when `thumb_icon` is unset |
| `UncheckedColor` | `Color` | `Default` | `unchecked_color(Color)` |

### Inherited from `MudBooleanInput<T>`

| MudBlazor | Type | Default | Rust mapping |
|---|---|---|---|
| `Value` | `T` | — | caller-owned `&'a mut bool` |
| `ValueChanged` | `EventCallback<T>` | — | `SwitchResponse::changed` |
| `Label` | `string?` | `null` | `label(s)` |
| `LabelPlacement` | `Placement` | `End` | `label_placement(Placement)` |
| `Color` | `Color` | `Default` | `color(Color)` |
| `Size` | `Size` | `Medium` | `size(Size)` |
| `Disabled` | `bool` | `false` | `disabled(bool)` |
| `ReadOnly` | `bool` | `false` | `read_only(bool)` |
| `Ripple` | `bool` | `true` | `ripple(bool)` |
| `Class` | `string?` | `null` | `classes(iter)` — stored, not resolved |
| `Tag` | `object?` | `null` | `tag(t)` / `tag_ref::<U>()` |

### Deliberately not ported

| MudBlazor | Reason |
|---|---|
| Generic `T` + `BoolConverter<T>` | MudBlazor converts arbitrary `T` (typically `bool`) through a pluggable `BoolConverter<T>`. egui has no converter pipeline; the port binds `&'a mut bool`, which is the overwhelmingly common case. Porting the generic form would be inventing a trait that MudBlazor does not require. |
| `ChildContent` | MudBlazor's `ChildContent` is a `RenderFragment` for a custom label; `label(s)` is the string form. A custom-content closure would require a lifetime-bearing closure on the builder and is not needed for the switch's documented use. |
| Validation members (`Required`, `Error`, `ErrorText`, `HelperText`, `ConversionError`, `Strict`, `Immediate`, `DebounceInterval`) | Inherited from `MudFormComponent` / `MudBaseInput`; they have no egui equivalent without a form system. |
| `Style` | egui has no inline CSS strings. |
| `UserAttributes` | There is no HTML tag to splat attributes onto. |
| `Class` resolution | Stored on `base` for parity only. The track and thumb colours come from `color` / `unchecked_color` / `thumb_icon_color`; a utility-class frame would composite a second fill. |

No enums are introduced: `Placement`, `Color`, and `Size` already exist in
`mud_egui_core::enums`.

### `ThumbIconColor` is conditional

`ThumbIconColor` **only applies when `ThumbIcon` is set**. That is the
documented MudBlazor remark on both parameters, and the port enforces it
structurally: `thumb_icon_color` is simply never read unless
`thumb_icon.is_some()`.

---

## Architecture

```
inputs/switch/
└── mud_switch.rs
    ├── constants        SWITCH_ANIM_SECONDS 0.15, RIPPLE_DURATION 0.35,
    │                    RIPPLE_MAX_RADIUS_FACTOR 1.6, RIPPLE_ALPHA 60,
    │                    UNCHECKED_TRACK_ALPHA 120, DISABLED_OPACITY 0.38,
    │                    LABEL_GAP 8, THUMB_OVERHANG 2,
    │                    THUMB_ICON_FRACTION 0.6, THUMB_STROKE 1
    ├── SwitchMetrics    track_w / track_h / thumb per Size
    ├── pure helpers     is_vertical, label_leads, row_size, track_rect,
    │                    label_rect, thumb_center, next_value_from_keys,
    │                    measure_label, is_ambient, lerp_color, with_alpha
    ├── MudSwitch<'a>    builder + show(ui) -> SwitchResponse
    ├── SwitchResponse   response + changed
    └── ripple           start_ripple, paint_ripple (memory-backed)
```

**Pure layer.** Size metrics, row/track/label geometry, the thumb's rest
positions, the keyboard mapping, colour blending and alpha helpers are free
functions or small `Copy` structs. None touch `egui::Context`; the whole
non-painting surface is unit-testable.

**Binding layer.** The caller owns the `&'a mut bool`; the switch writes it when
a click or key toggles it, and reports `SwitchResponse::changed` for the frame
it moved. This is the same split `MudCheckBox`, `MudRadio` and `MudCollapse`
use.

**Component layer.** `show` measures the label, allocates the row, applies a
click or key press, then paints the track, thumb, thumb icon, label and ripple.

---

## API reference

### `MudSwitch<'a>`

| Method | Type | Default | Mirrors | Description |
|---|---|---|---|---|
| `new(value)` | `&'a mut bool` | — | `Value` | Binds the switch. |
| `label(s)` | `impl Into<String>` | `None` | `Label` | Label text. |
| `label_placement(p)` | `Placement` | `End` | `LabelPlacement` | `Start` / `End` / `Top` / `Bottom` (plus `Left` / `Right`). |
| `color(c)` | `Color` | `Default` | `Color` | Checked-track accent. |
| `unchecked_color(c)` | `Color` | `Default` | `UncheckedColor` | Unchecked-track colour. |
| `size(s)` | `Size` | `Medium` | `Size` | Track and thumb dimensions. |
| `thumb_icon(s)` | `impl Into<String>` | `None` | `ThumbIcon` | Glyph on the thumb. |
| `thumb_icon_color(c)` | `Color` | `Default` | `ThumbIconColor` | Thumb glyph colour; ignored without `thumb_icon`. |
| `ripple(b)` | `bool` | `true` | `Ripple` | Click ripple. |
| `disabled(b)` | `bool` | `false` | `Disabled` | Dims and disables. |
| `read_only(b)` | `bool` | `false` | `ReadOnly` | Disables without dimming. |
| `classes(iter)` | `iter of AsRef<str>` | none | `Class` | Stored on `ComponentBase`. |
| `tag(t)` / `tag_ref::<U>()` | `T: Any + Send + Sync` | `None` | `Tag` | Arbitrary user data. |
| `show(ui)` | `&mut Ui` | — | — | Returns `SwitchResponse`. |

### `SwitchResponse`

| Field | Type | Description |
|---|---|---|
| `response` | `Response` | The switch row (track + label). Deref target. |
| `changed` | `bool` | `true` on the frame the bound value moved. Mirrors `ValueChanged`. |

`SwitchResponse` implements `Deref<Target = Response>`, so `out.clicked()`,
`out.hovered()` and `out.rect` work directly.

### `Size` mapping

| `Size` | Track (w × h) | Thumb |
|---|---|---|
| `ExtraSmall` / `Small` | 34 × 14 pt | 18 pt |
| `Medium` | 40 × 16 pt | 22 pt |
| `Large` / `ExtraLarge` | 50 × 20 pt | 28 pt |

The thumb is deliberately larger than the track, and overhangs it by
`THUMB_OVERHANG` (2 pt) at each end; that is the switch's defining silhouette.

### Colour resolution

| State | Role | Fallback when `Default` / `Inherit` |
|---|---|---|
| Checked track | `color` | `ui.visuals().selection.bg_fill` |
| Unchecked track | `unchecked_color` | `ui.visuals().text_color()` at `UNCHECKED_TRACK_ALPHA` (120) |
| Thumb icon | `thumb_icon_color` | `ui.visuals().text_color()` |

During the animation the checked and unchecked track colours are linearly
interpolated by `progress`, so toggling fades between them rather than
snapping.

### Keyboard

| Key | Effect |
|---|---|
| `ArrowLeft` / `Delete` | Sets `false` |
| `ArrowRight` / `Enter` (incl. numpad) | Sets `true` |
| `Space` | Toggles |

Priority, when multiple keys are reported pressed in one frame:
`uncheck > check > space`. That is deterministic and matches the order in
which MudBlazor's `HandleKeyDownAsync` evaluates its branches.

### Constants

| Name | Value |
|---|---|
| `SWITCH_ANIM_SECONDS` | `0.15` s |
| `RIPPLE_DURATION` | `0.35` s |
| `RIPPLE_MAX_RADIUS_FACTOR` | `1.6` |
| `RIPPLE_ALPHA` | `60` |
| `UNCHECKED_TRACK_ALPHA` | `120` |
| `DISABLED_OPACITY` | `0.38` |
| `LABEL_GAP` | `8.0` |
| `THUMB_OVERHANG` | `2.0` |
| `THUMB_ICON_FRACTION` | `0.6` |
| `THUMB_STROKE` | `1.0` |

---

## Examples

### Basic

```rust
let mut enabled = true;
ui.add(MudSwitch::new(&mut enabled).label("Enabled"));
```

### Coloured, with a different unchecked colour

```rust
ui.add(
    MudSwitch::new(&mut notifications)
        .label("Notifications")
        .color(Color::Success)
        .unchecked_color(Color::Error),
);
```

`color` is the on-track colour; `unchecked_color` is the off-track colour. Both
default to `Color::Default`, which resolves to the theme's accent (on) and a
muted text colour (off).

### Label before the track

```rust
ui.add(
    MudSwitch::new(&mut enabled)
        .label("Compact mode")
        .label_placement(Placement::Start),
);
```

### Vertical label

```rust
MudSwitch::new(&mut enabled)
    .label("Above")
    .label_placement(Placement::Top)
    .show(ui);

MudSwitch::new(&mut enabled)
    .label("Below")
    .label_placement(Placement::Bottom)
    .show(ui);
```

### Thumb icon

```rust
MudSwitch::new(&mut airplane_mode)
    .thumb_icon(filled::AIRPLANE)
    .thumb_icon_color(Color::Primary)
    .show(ui);
```

`thumb_icon_color` is only read when `thumb_icon` is set.

### Dense row in a toolbar

```rust
MudSwitch::new(&mut self.compact)
    .label("Compact")
    .size(Size::Small)
    .show(ui);
```

### Read-only

```rust
MudSwitch::new(&mut self.locked)
    .label("Locked")
    .read_only(true)
    .show(ui);
```

The switch is not dimmed and its state cannot be toggled by pointer or
keyboard; the caller can still change `*value` directly.

### Disabled

```rust
MudSwitch::new(&mut self.locked)
    .label("Unavailable")
    .disabled(true)
    .show(ui);
```

Dims the track, thumb and label to 38 % opacity and blocks all input.

### Reacting to a toggle

```rust
let out = MudSwitch::new(&mut notifications).label("Notify").show(ui);
if out.changed {
    log::info!("notifications = {notifications}");
}
```

### Keyboard-only form

```rust
// With the switch focused:
//   ArrowLeft / Delete  → off
//   ArrowRight / Enter  → on
//   Space               → toggle
```

### Tag round-trip

```rust
let s = MudSwitch::new(&mut enabled).tag("airplane");
assert_eq!(s.tag_ref::<&str>(), Some(&"airplane"));
```

---

## Layout and rendering

### Horizontal (`Start` / `End`)

```
End:    [ ( )───── ]  Label …
Start:  Label …  [ ( )───── ]
```

- Track: `switch_metrics(size)` pill, radius = half the track height.
- Thumb: circle of `metrics.thumb`, centred on the track's vertical centre and
  overhanging `THUMB_OVERHANG` at the active end.
- Gap between track and label: `LABEL_GAP` (8 pt).
- Row height: `max(thumb, track_h)`.

### Vertical (`Top` / `Bottom`)

```
Top:    [ Label … ]
        [ ( )───── ]
Bottom: [ ( )───── ]
        [ Label … ]
```

The track is horizontally centred under/over the label; the row width is
`max(track_w, label_width)`.

### Animation

```
progress = animate_bool_with_time(id, *value, SWITCH_ANIM_SECONDS)
thumb_center = lerp(off_center, on_center, progress)
track_fill   = lerp(unchecked_color, checked_color, progress)
```

The thumb interpolates linearly between its two rest positions; the track
colour fades between unchecked and checked. Both are driven by the same
`progress`, so they stay in sync.

### Ripple

When `ripple(true)` and a toggle happens, a filled circle expands from the
thumb's centre with radius `thumb/2 × RIPPLE_MAX_RADIUS_FACTOR` over
`RIPPLE_DURATION`, then fades out. Start time lives in `egui::Memory` keyed by
the switch's `Response::id`, so the component stays stateless.

### Degenerate inputs

| Input | Behaviour |
|---|---|
| Empty label | The label column is zero-width; only the track is drawn. |
| `disabled` | Track, thumb and label are gamma-multiplied by 38 %; no interaction, no cursor change beyond `NotAllowed`. |
| `read_only` | Full opacity; pointer clicks and keys are ignored; cursor stays default. |
| `!ui.is_enabled()` (parent disabled) | Treated as `disabled` for interaction. |
| `thumb_icon_color` set without `thumb_icon` | Ignored — no icon is drawn. |

---

## Performance and memory choices

| Choice | Effect |
|---|---|
| Theme fetched once per frame | One `MudTheme::fetch`; the surface colour is read from it once, and the two accent colours fall back to `ui.visuals()` when ambient (so no palette walk in the common `Color::Default` case). |
| One `allocate_exact_size` for the row | The whole switch is one rect and one `Response`. |
| Pure helpers on `Copy` values | Metrics, geometry, keyboard mapping and colour blending are arithmetic; unit-testable without a `Context`. |
| No `Vec`, no `String` beyond the label | The label is measured with `layout_no_wrap` and moved into `MudText`; the thumb glyph is moved into `MudIcon`. |
| `animate_bool_with_time` | Idle at zero cost when the value does not change; the animation requests its own repaints only while in flight. |
| Ripple in `egui::Memory` | One `f64` per in-flight ripple; zero cost when `ripple(false)` or disabled. |
| `Deref` on `SwitchResponse` | Callers that only need `.clicked()` do not pay for naming the struct. |

---

## Testing

`cargo test -p mud_egui_components mud_switch` runs 18 pure-function tests:

| Area | Tests |
|---|---|
| Metrics | `ExtraSmall`→`Small` and `ExtraLarge`→`Large` folding; monotonicity of track width, height and thumb; thumb is larger than track height for every size |
| Placement | `is_vertical` and `label_leads` for every `Placement` variant |
| Row geometry | `row_size` horizontal and vertical; `track_rect` centres vertically and follows placement; `label_rect` follows placement |
| Thumb | rest positions at both ends; overhang applied; vertical centre preserved |
| Keyboard | uncheck beats check beats space; space toggles; no key → `None` |
| Colour | ambient roles; `lerp_color` endpoints and midpoint; `with_alpha` preserves RGB |
| Builders | defaults match MudBlazor; every field set; tag round-trip |

No test creates a live `egui::Context`. `MudSwitch::show` needs a live `Ui`
(layout, painting, animation, theme lookup) and is therefore left to
integration tests; everything it delegates to is covered above.

Suggested integration tests (`egui_kittest` or a headless `Context::run`
harness):

- A click toggles the bound value and sets `changed` for exactly one frame.
- `ArrowLeft` / `ArrowRight` / `Enter` / `Space` on a focused switch produce
  the documented values.
- A disabled switch ignores pointer and keyboard input.
- A read-only switch is not dimmed but cannot be toggled.
- The thumb's X position advances monotonically with `animate_bool` progress.
- A `thumb_icon` renders through `MudIcon`, and is absent when unset.

```bash
cargo test -p mud_egui_components mud_switch
```

---

## Differences from MudBlazor

- **`Value` / `ValueChanged` become a caller-owned `&'a mut bool` plus
  `SwitchResponse::changed`.** The caller owns the value; the response reports
  the frame it moved. This is the same split `MudCheckBox`, `MudRadio` and
  `MudCollapse` use.
- **The generic `MudSwitch<T>` is bound to `bool`.** MudBlazor converts
  arbitrary `T` through `BoolConverter<T>`; egui has no converter pipeline.
  The port would rather bind the common case honestly than invent a
  converter trait that has no Blazor counterpart.
- **`ChildContent` is not ported.** The label is the string form; a custom
  label closure has no MudBlazor switch analogue that would justify a
  lifetime-bearing field on the builder.
- **`ThumbIconColor` is structurally conditional.** It is only read when
  `thumb_icon.is_some()`, matching the documented remark rather than relying
  on the caller to respect it.
- **Colour interpolation is explicit.** MudBlazor's SCSS animates the track
  colour with a CSS transition; the port lerps between the resolved unchecked
  and checked colours by the same `progress` that drives the thumb, which is
  the immediate-mode equivalent and keeps both transitions in lockstep.
- **The unchecked-track fallback is a muted text colour at 120 alpha.** This is
  an approximation of `--mud-palette-text-secondary`; it is named at the top of
  the file (`UNCHECKED_TRACK_ALPHA`) so it can be tuned without touching the
  render path.
- **The switch metrics are constants derived from MudBlazor's SCSS.** They are
  deliberately named and grouped in `switch_metrics` so a theme that wants
  different proportions can adjust them in one place, rather than scattering
  magic numbers through the painter.
- **Keyboard priority is documented and deterministic.** When several keys are
  reported pressed in one frame, `uncheck` wins over `check`, which wins over
  `space`. egui reports key presses independently, so this is a real
  possibility and worth pinning down in a unit test.
- **`Class` is stored but not resolved.** The switch's visual identity comes
  from `color`, `unchecked_color`, `thumb_icon_color` and `size`; a
  utility-class frame would composite a second fill over the track.
- **`Style` and `UserAttributes` are not ported.** egui has no inline CSS
  strings and no HTML tag to splat attributes onto.

===== ./crates/mud_egui_components/src/inputs/text_field/README.md =====

# MudTextField for `mud_egui`

A themed single-line text field for [egui](https://github.com/emilk/egui),
ported from MudBlazor's `MudTextField<T>` (MudBlazor 8.13.0).

```rust
let mut name = String::new();
let resp = MudTextField::new(&mut name)
    .label("Full name")
    .placeholder("Type here…")
    .helper_text("As it appears on your ID")
    .variant(Variant::Outlined)
    .show(ui);

if resp.changed {
    log::info!("value is now {name:?}");
}
```

The label, helper text, error text, and text-form adornment render through
**`MudText`**; icon-form adornments and the clear button render through
**`MudIcon`**. The editable text itself is rendered by `egui::TextEdit`
(its glyphs are egui's responsibility, exactly as MudBlazor delegates to
`<input>`).

---

## Contents

1. [Design principle: API parity, not API invention](#design-principle-api-parity-not-api-invention)
2. [Pass 1 scope and pass 2 deferral](#pass-1-scope-and-pass-2-deferral)
3. [Architecture](#architecture)
4. [API reference](#api-reference)
5. [Examples](#examples)
6. [Layout and styling](#layout-and-styling)
7. [Performance and memory choices](#performance--memory-choices)
8. [Testing](#testing)
9. [Differences from MudBlazor](#differences-from-mudblazor)

---

## Design principle: API parity, not API invention

MudBlazor's `MudTextField<T>` parameter surface was verified against
[DNDocs](https://docs.dndocs.com/n/MudBlazor/8.13.0/api/MudBlazor.MudTextField-1.html)
before this port. The two-way binding (`Value` / `ValueChanged` /
`ValueExpression`) is unified into a **caller-owned `&'a mut String`** passed
to [`MudTextField::new`] — the same pattern used by `MudTable`,
`MudTreeView`, `MudChipSet`, `MudNavMenu`, `MudPagination`, `MudStepper`,
and `MudTabs`. There is no `ValueChanged` callback; the caller reads the
string after [`show`](MudTextField::show) returns.

`MudTextField` also does not implement `egui::Widget` because a
`Widget::ui(self)` implementation cannot return the mutated `String`
alongside the `Response` that the trait requires. Use `show(ui)` directly.

### Ported parameters (pass 1)

| MudBlazor | Rust method | Default |
|-----------|-------------|---------|
| `Value` / `ValueChanged` | `value: &'a mut String` argument of `new` | required |
| `Label` | `label(s)` | `None` |
| `Placeholder` | `placeholder(s)` | `None` |
| `HelperText` | `helper_text(s)` | `None` |
| `Error` | `error(bool)` | `false` |
| `ErrorText` | `error_text(s)` | `None` |
| `Variant` | `variant(Variant)` | `Text` |
| `Margin` | `margin(Margin)` | `None` |
| `Color` | `color(Color)` | `Primary` |
| `Size` | `size(Size)` | `Medium` |
| `Disabled` | `disabled(bool)` | `false` |
| `ReadOnly` | `readonly(bool)` | `false` |
| `Clearable` | `clearable(bool)` | `false` |
| `Adornment` + `AdornmentIcon` + `AdornmentText` | `adornment_end(AdornmentEnd)` | `None` |
| `AdornmentColor` | `adornment_end_color(Color)` | `None` |
| `OnAdornmentClick` | `on_adornment_click(f)` | `None` |
| `Class` (base) | `classes(iter)` | `None` |
| `Style` (base) | *(use `classes`)* | — |
| `Tag` (base) | **not ported** (no HTML tag) | — |
| `UserAttributes` | **not ported** | — |

### Extensions (not in MudBlazor)

| Extension | Description |
|-----------|-------------|
| `id(Id)` | Stable interaction id salt. |
| `aria_label(s)` | Accessible label overriding the caption. |
| `clear_icon(s)` | Glyph used by the clear button. |
| `TextFieldResponse` | Rich return value with `changed` / `lost_focus` / `has_focus` / `submitted`. |

---

## Pass 1 scope and pass 2 deferral

Pass 1 ships everything the **overlay family** needs to render an
input-with-a-dropdown-chevron on top of it: label, helper/error, all three
variants, all three margin values, all five sizes, adornment-end as icon or
text, clear button, focus ring, hover state, and the `TextFieldResponse`
with `changed` / `lost_focus` / `has_focus` / `submitted`.

The following MudBlazor parameters are **deferred to pass 2** and are
documented here so the deferral is visible:

| Deferred parameter | Why pass 2 |
|--------------------|------------|
| `InputType` | Password masking, email validation display, number formatting are their own concern; they do not affect overlay rendering. |
| `Mask` | Character-by-character input masking is a substantial subsystem. |
| `Lines` / `AutoGrow` / `MaxLines` | Multiline layout changes the whole field height model. |
| `Required` / `RequiredError` | Form-level validation display; sits on top of `error` / `error_text`. |
| `Counter` / `MaxLength` | Counter goes in the helper row; needs the helper-row rework. |
| `DebounceInterval` / `Immediate` | Debouncing `changed` requires timer state in `egui::Memory` — fine to add later. |
| `AdornmentStart` | Leading adornment is a mirror of the trailing one; nothing structural. |
| Floating-label-with-border-cutout | MudBlazor draws the label *into* the border when focused/valued. The pass-1 renderer draws it as a static caption above the field. Visually close, structurally simpler. |
| `Typo` on the input body | Applies a typography scale to the `TextEdit`; nothing structural. |

These deferrals do not block `MudSelect`, `MudAutocomplete`, or any picker.
They will be needed by fully-fledged forms, but the forms layer is a separate
work item.

### Deliberately not ported

- **`Tag` / `UserAttributes`** — no HTML tag.
- **`Style`** — no inline CSS strings; use `classes(..)`.
- **`For` / `Validation` / `Validation`** — this is a form-level concept;
  the port uses `error` / `error_text` directly.

---

## Architecture

```
text_field/
├── adornment_end.rs   — AdornmentEnd enum (Icon | Text)
├── mud_text_field.rs  — MudTextField + TextFieldResponse + pure helpers
└── mod.rs             — re-exports

mud_egui_core/src/enums/
└── margin.rs          — Margin enum (None | Dense | Normal)
```

**Pure layer.** `height_for`, `font_for`, `padding_for`, `resolve_color`,
`with_alpha` — free functions, unit-tested without `egui::Context`.

**State layer.** The caller owns `&'a mut String` — the field writes to it
in place, and no separate state struct is needed.

**Component layer.** `show` resolves the theme and class frame, then
`render_inner` draws the label, the field body, the `TextEdit`, the trailing
adornment, the clear button, and the helper/error row.

---

## API reference

### `MudTextField<'a>`

| Method | Type | Default | Description |
|--------|------|---------|-------------|
| `new(value)` | `&'a mut String` | — | Field editing `value` in place. |
| `label(s)` | `impl Into<String>` | `None` | Caption above the field. |
| `placeholder(s)` | `impl Into<String>` | `None` | Hint shown when empty. |
| `helper_text(s)` | `impl Into<String>` | `None` | Below the field. |
| `error(bool)` | `bool` | `false` | Switch accent to error colour. |
| `error_text(s)` | `impl Into<String>` | `None` | Shown instead of `helper_text` when `error` is `true`. |
| `variant(Variant)` | `Variant` | `Text` | `Text` (underline) / `Filled` (grey bg) / `Outlined` (border). |
| `margin(Margin)` | `Margin` | `None` | `None` / `Dense` (−4 px) / `Normal` (+4 px). |
| `color(Color)` | `Color` | `Primary` | Focus / underline accent. |
| `size(Size)` | `Size` | `Medium` | 32 / 40 / 48 px field height. |
| `disabled(bool)` | `bool` | `false` | Prevents interaction. |
| `readonly(bool)` | `bool` | `false` | Prevents editing but keeps visuals. |
| `clearable(bool)` | `bool` | `false` | Shows a clear button when non-empty. |
| `clear_icon(s)` | `impl Into<String>` | `CLEAR` | Glyph used by the clear button. |
| `adornment_end(AdornmentEnd)` | `AdornmentEnd` | `None` | Trailing icon or text. |
| `adornment_end_color(Color)` | `Color` | `None` | Adornment colour override. |
| `on_adornment_click(f)` | `FnMut()` | `None` | Fires when the adornment is clicked. |
| `id(Id)` | `impl Hash` | `None` | Stable interaction id salt. |
| `aria_label(s)` | `impl Into<String>` | `None` | Accessible label; falls back to `label`. |
| `classes(iter)` | `iter of AsRef<str>` | none | Wrapper utility classes. |
| `flavor(StyleFlavor)` | `StyleFlavor` | `Bootstrap` | Utility resolver. |
| `show(ui)` | `&mut Ui` | — | Returns a `TextFieldResponse`. |

### `TextFieldResponse`

| Field | Type | Description |
|-------|------|-------------|
| `response` | `egui::Response` | Union of the field rect and the inner `TextEdit`. |
| `changed` | `bool` | The user edited the value this frame. |
| `lost_focus` | `bool` | The field lost focus this frame. |
| `has_focus` | `bool` | The field has focus. |
| `submitted` | `bool` | Enter was pressed while focused. |

### `AdornmentEnd`

| Variant | Description |
|---------|-------------|
| `Icon(String)` | SVG glyph. |
| `Text(String)` | Short text label (a unit, a hint). |

Constructors: `AdornmentEnd::icon(s)`, `AdornmentEnd::text(s)`.
Predicates: `is_icon()`, `content()`.

### `Margin`

| Variant | Effect |
|---------|--------|
| `None` (default) | Standard field height. |
| `Dense` | −4 px vertical. |
| `Normal` | +4 px vertical. |

### `Variant` (from `mud_egui_core::enums`)

| Variant | Visual |
|---------|--------|
| `Text` (default) | Underline only. |
| `Filled` | Grey background, underline. |
| `Outlined` | Full 1 px border (2 px when focused). |

---

## Examples

### Basic

```rust
let mut name = String::new();
MudTextField::new(&mut name).show(ui);
```

### With label and helper text

```rust
MudTextField::new(&mut email)
    .label("Email")
    .helper_text("We'll never share it.")
    .show(ui);
```

### Outlined variant

```rust
MudTextField::new(&mut name)
    .variant(Variant::Outlined)
    .label("Name")
    .show(ui);
```

### Filled variant

```rust
MudTextField::new(&mut name)
    .variant(Variant::Filled)
    .show(ui);
```

### Dense margin

```rust
MudTextField::new(&mut value)
    .margin(Margin::Dense)
    .show(ui);
```

### Error state

```rust
MudTextField::new(&mut email)
    .label("Email")
    .error(!is_valid_email(&email))
    .error_text("Please enter a valid email.")
    .show(ui);
```

### Clear button

```rust
let resp = MudTextField::new(&mut query)
    .clearable(true)
    .placeholder("Search…")
    .show(ui);
if resp.changed {
    /* query has been updated (or cleared) */
}
```

### Trailing icon adornment (the pattern `MudSelect` will use)

```rust
let mut selected = "Choose…".to_string();
MudTextField::new(&mut selected)
    .label("Country")
    .adornment_end(AdornmentEnd::icon(filled::ARROW_DROP_DOWN))
    .on_adornment_click(|| self.select_open = true)
    .readonly(true) // the field is a trigger, not free text
    .show(ui);
```

### Trailing text adornment (a unit)

```rust
MudTextField::new(&mut weight)
    .label("Weight")
    .adornment_end(AdornmentEnd::text("kg"))
    .show(ui);
```

### Read-only display

```rust
MudTextField::new(&mut id_display)
    .label("Account ID")
    .readonly(true)
    .variant(Variant::Outlined)
    .show(ui);
```

### Submitting on Enter

```rust
let resp = MudTextField::new(&mut query).show(ui);
if resp.submitted {
    run_search(&query);
}
```

### Accessible labelling

```rust
MudTextField::new(&mut amount)
    .aria_label("Transaction amount in US dollars")
    .show(ui);
```

---

## Layout and styling

### Dimensions

| `Size` | Height | Font | Horizontal padding |
|--------|--------|------|--------------------|
| `Small` | 32 px | 12 pt | 8 px |
| `Medium` | 40 px | 14 pt | 12 px |
| `Large` | 48 px | 16 pt | 16 px |

`Margin::Dense` subtracts 4 px; `Margin::Normal` adds 4 px.

### Adornment / clear-button widths

- `AdornmentEnd::Icon` — 50% of field height.
- `AdornmentEnd::Text` — measured text width + 4 px.
- Clear button (only when `adornment_end` is `None`) — 42% of field height + 4 px.
- The edit area is `field_rect` minus horizontal padding on each side, minus
  the adornment width (or clear-button width, whichever is present).

### Focus ring

- **Outlined** variant: the whole border switches to the accent colour and
  grows to 2 px.
- **Text** / **Filled** variants: the underline switches to the accent colour
  and grows to 2 px.

Focus is tracked from `egui::Memory` (previous frame), because
`TextEdit`'s internal id lives there. This is the standard pattern for
composing custom chrome around `TextEdit` and is documented in the source.

### Click-to-focus anywhere

Clicking anywhere in the field box — including the padding and the empty
adornment area — requests focus on the inner `TextEdit`. This matches
MudBlazor, where the whole `<MudInput>` wrapper is clickable.

### Hover state

- The underline / border darkens slightly on hover.
- The trailing adornment and clear button switch to a pointing-hand cursor.

---

## Performance and memory choices

| Choice | Effect |
|--------|--------|
| `height_for` / `font_for` / `padding_for` are pure | No `egui::Context`; unit-tested. |
| `MudTextField` holds `&'a mut String` | Zero-allocation; the caller owns the buffer. |
| `TextEdit` runs with `frame(false)` | The port paints its own frame; `TextEdit` handles only the glyphs and editing. |
| `TextEdit` runs with `margin(ZERO)` | No internal padding, so the padding the port paints is authoritative. |
| Focus state read from `egui::Memory` | One-frame lag but no plumbing needed and no duplicated state. |
| `clip_rect` set on the edit child | Prevents the input text from spilling over the adornment. |
| `push_id(edit_id, …)` on the `TextEdit` | Stable focus tracking across frames even if the field moves in layout. |
| Label / helper / error rendered through `MudText` | Theme and typography propagate; no per-field font configuration. |
| Adornment and clear glyphs rendered through `MudIcon` | Free thanks to the icon transform cache. |
| `clone()` on `placeholder` for `.hint_text(..)` | One small allocation per frame; required because `hint_text` takes `impl Into<WidgetText>` and `Option<&String>` does not implement it. Could be avoided with `as_deref()` but the ergonomics suffer. |

---

## Testing

`cargo test` runs 15 pure-function tests:

| Area | Tests |
|------|-------|
| `height_for` | every `Size` × `Margin` combination, unknown `Size` fallback, extreme-dense clamp |
| `font_for` | every `Size` |
| `padding_for` | every `Size` |
| Defaults | every `MudTextField` field matches MudBlazor |
| Builders | every field set |
| `AdornmentEnd` | `icon` / `text` constructors, `is_icon`, `content` |
| `resolve_color` | Default / Inherit fallback, explicit palette mapping |
| `with_alpha` | identity, transparent, fractional scaling |

Rendering (`show`) needs a live `egui::Context`; the pure layer is the source
of truth.

```bash
cargo test -p mud_egui_components mud_text_field
cargo test -p mud_egui_core margin
```

---

## Differences from MudBlazor

- **Value is caller-owned (`&'a mut String`).** MudBlazor uses the bindable
  `Value` / `ValueChanged` / `ValueExpression` triple; the egui equivalent
  is a mutable reference the caller owns and passes to `new`. Read the
  string after `show` returns to observe changes.
- **No `Widget` impl.** `MudTextField` cannot implement `egui::Widget`
  cleanly because `Widget::ui(self)` cannot return the mutated `String`
  alongside the `Response`.
- **Label rendered as a static caption above the field.** MudBlazor's label
  *floats into* the border when the field is focused or has a value. Pass 2
  will implement the floating label with the border cutout; pass 1 draws
  the caption above the field for structural simplicity. This is the only
  visible deviation and it does not affect the overlay family (whose labels
  sit above the field anyway).
- **Focus tracking has one frame of lag.** Because `TextEdit`'s id lives in
  `egui::Memory` and is only known after the `TextEdit` has run at least
  once, the focus ring follows the previous frame's focus state. In practice
  the lag is invisible.
- **`Variant` defaults to `Text`.** This matches MudBlazor's
  `MudTextField` default (`MudBlazor.Variant.Text`). Note that this differs
  from `MudButton`'s default (`MudBlazor.Variant.Text` too but visually
  different in that component's context).
- **`Margin::None` is the default.** Matches MudBlazor.
- **`InputType` / `Mask` / `Lines` / `AutoGrow` / `Required` / `Counter` /
  `DebounceInterval` / `AdornmentStart` / floating-label-with-cutout /
  `Typo` are deferred to pass 2.** See [Pass 1 scope and pass 2
  deferral](#pass-1-scope-and-pass-2-deferral).
- **`Tag` / `UserAttributes` not ported.** No HTML tag.
- **`Style` → `classes(..)`.** egui has no inline CSS strings.
- **`For` / `Validation` not ported.** Form-level concepts; the port uses
  `error` / `error_text` directly.

===== ./crates/mud_egui_components/src/inputs/toggle/README.md =====

# MudToggleGroup for egui

A production-grade, MudBlazor‑faithful port of `MudToggleGroup<T>` and
`MudToggleItem<T>` (MudBlazor 8.13.0) for the [`egui`](https://github.com/emilk/egui)
immediate-mode GUI library.

The component reproduces MudBlazor’s segmented-control aesthetic—shared
outline, square inner corners, accent under-fill, check-mark support, ripple,
and precise size presets—while remaining allocation-light and idiomatic Rust.

---

## Table of contents

1. [Architecture](#architecture)
2. [API reference](#api-reference)
   - [`MudToggleGroup`](#mudtogglegroup)
   - [`MudToggleItem`](#mudtoggleitem)
   - [`SelectionMode`](#selectionmode)
   - [`Size` and `Color`](#size-and-color)
3. [Real-world examples](#real-world-examples)
   - [Single selection (radio-like)](#single-selection-radio-like)
   - [Multi selection (checkbox-like)](#multi-selection-checkbox-like)
   - [Toggle selection (deselectable)](#toggle-selection-deselectable)
   - [Vertical group with custom content](#vertical-group-with-custom-content)
   - [Check marks and fixed content](#check-marks-and-fixed-content)
4. [Performance & design rationale](#performance--design-rationale)
5. [Visual parity notes](#visual-parity-notes)
6. [Testing](#testing)
7. [License](#license)

---

## Architecture

```
mud_toggle_group.rs
├── MudToggleItem<T>        — single-item description (value, text, icons, content)
├── MudToggleGroup          — group configuration and rendering engine
│   ├── show()              — main entry point; collects items, renders, applies click
│   ├── render_item()       — dispatch: custom vs standard
│   ├── render_custom()     — user-provided closure path
│   ├── render_standard()   — MudButton path
│   └── make_button()       — zero-clone button construction
├── ToggleGroupUi<'a, T>    — collector handle passed to the show() closure
├── Pure helpers            — sizing, corners, selection, icons, theme decisions
└── Ripple subsystem        — memory-backed, per-item expanding-circle animation
```

The component follows the same **per-frame temporary** model as MudBlazor’s
render tree: items are collected into a `Vec` on every frame, rendered once,
and dropped. The selection itself is not owned by the group; it lives in the
caller’s `Vec<T>`, which is the egui equivalent of Blazor’s `Value` /
`Values` two-way binding.

---

## API reference

### `MudToggleGroup`

| Method / field | Type | Default | MudBlazor equivalent | Description |
|---|---|---|---|---|
| `new()` | `Self` | — | — | Creates a group with defaults. |
| `selection_mode(m)` | `SelectionMode` | `Single` | `SelectionMode` | Selection behaviour. |
| `check_mark(b)` | `bool` | `false` | `CheckMark` | Show leading check glyphs. |
| `check_mark_class(s)` | `impl Into<String>` | `None` | `CheckMarkClass` | CSS classes for check icons (parity hook). |
| `selected_class(s)` | `impl Into<String>` | `None` | `SelectedClass` | CSS classes for selected items (parity hook). |
| `fixed_content(b)` | `bool` | `false` | `FixedContent` | Reserve check-glyph space when unselected. |
| `disabled(b)` | `bool` | `false` | `Disabled` | Disable the whole group. |
| `vertical(b)` | `bool` | `false` | `Vertical` | Stack items vertically. |
| `right_to_left(b)` | `bool` | `false` | `RightToLeft` (cascading) | Reverse horizontal order. |
| `delimiters(b)` | `bool` | `true` | `Delimiters` | Draw lines between flat items. |
| `outlined(b)` | `bool` | `true` | `Outlined` | Draw the shared segmented border. |
| `color(c)` | `Color` | `Primary` | `Color` | Accent / border colour. |
| `size(s)` | `Size` | `Medium` | `Size` | Cell height preset. |
| `ripple(b)` | `bool` | `true` | `Ripple` | Enable click ripple. |
| `classes(iter)` | `impl IntoIterator<Item = impl AsRef<str>>` | — | `Class` (base) | Utility classes on `ComponentBase`. |
| `show(ui, &mut Vec<T>, closure)` | `Response` | — | `Value` / `Values` binding | Renders the group and mutates the selection. |

> **MudBlazor parity note:** `FullWidth`, `CornerRadius`, and a plain
> `MudToggleItem::Icon` parameter do **not** exist in MudBlazor 8.13.0 and are
> therefore **not** part of the public API. Layout is content-sized except for
> vertical groups and groups containing custom content, which stretch to the
> available width.

### `MudToggleItem`

| Method / field | Type | Default | MudBlazor equivalent | Description |
|---|---|---|---|---|
| `new(value, text)` | `(T, impl Into<String>)` | — | `Value`, `Text` | Creates an item. |
| `selected_icon(s)` | `impl Into<String>` | `None` (falls back to Material check) | `SelectedIcon` | Glyph when selected and `check_mark` is on. |
| `unselected_icon(s)` | `impl Into<String>` | `None` | `UnselectedIcon` | Glyph when unselected and `check_mark` is on. |
| `disabled(b)` | `bool` | `false` | `Disabled` | Disable this item only. |
| `content(f)` | `FnMut(&mut Ui, bool)` | `None` | `ChildContent` (`RenderFragment<bool>`) | Full custom cell renderer. |

### `SelectionMode`

| Variant | Behaviour |
|---|---|
| `Single` | Exactly one item selected; re-clicking the selected item does nothing. |
| `Toggle` | One item selected; re-clicking the selected item clears the selection. |
| `Multi` | Multiple items; clicking toggles membership. |

### `Size` and `Color`

`Size` maps to MudBlazor’s height presets:

| `Size` | Cell height |
|---|---|
| `ExtraSmall` | 24 px |
| `Small` | 32 px |
| `Medium` | 40 px |
| `Large` | 48 px |
| `ExtraLarge` | 56 px |

`Color` accepts the standard MudBlazor palette (`Primary`, `Secondary`,
`Info`, `Success`, `Warning`, `Error`, `Default`, `Custom`).

---

## Real-world examples

### Single selection (radio-like)

```rust
use mud_egui_components::mud_toggle_group::MudToggleGroup;
use mud_egui_core::enums::{Color, SelectionMode};

let mut alignment: Vec<&'static str> = vec!["left"];

MudToggleGroup::new()
    .selection_mode(SelectionMode::Single)
    .color(Color::Primary)
    .show(ui, &mut alignment, |group| {
        group.item("left", "Left");
        group.item("center", "Center");
        group.item("right", "Right");
    });

// `alignment` is now `vec!["left"]`, `vec!["center"]`, or `vec!["right"]`.
```

### Multi selection (checkbox-like)

```rust
let mut toppings: Vec<String> = vec!["cheese".into()];

MudToggleGroup::new()
    .selection_mode(SelectionMode::Multi)
    .outlined(false)
    .delimiters(true)
    .show(ui, &mut toppings, |group| {
        group.item("cheese".to_string(), "Cheese");
        group.item("ham".to_string(), "Ham");
        group.item("olives".to_string(), "Olives");
    });
```

### Toggle selection (deselectable)

```rust
let mut filter: Vec<&'static str> = vec!["active"];

MudToggleGroup::new()
    .selection_mode(SelectionMode::Toggle)
    .show(ui, &mut filter, |group| {
        group.item("active", "Active");
        group.item("archived", "Archived");
    });
```

Clicking `Active` while it is selected clears `filter` to `vec![]`.

### Vertical group with custom content

```rust
use egui::Widget;

let mut view: Vec<u8> = vec![0];

MudToggleGroup::new()
    .vertical(true)
    .show(ui, &mut view, |group| {
        group.item_full(
            MudToggleItem::new(0, "")
                .content(|ui, selected| {
                    ui.horizontal(|ui| {
                        ui.label("📊");
                        ui.label(if selected { "Bold Chart" } else { "Chart" });
                    });
                }),
        );
        group.item_full(
            MudToggleItem::new(1, "")
                .content(|ui, selected| {
                    ui.horizontal(|ui| {
                        ui.label("📋");
                        ui.label(if selected { "Bold Table" } else { "Table" });
                    });
                }),
        );
    });
```

Custom content closures run twice per frame (measure, then draw) and must be
side-effect-free.

### Check marks and fixed content

```rust
MudToggleGroup::new()
    .check_mark(true)
    .fixed_content(true)
    .show(ui, &mut selected, |group| {
        group.item_full(
            MudToggleItem::new("a", "Option A")
                .selected_icon("✅")
                .unselected_icon("⬜"),
        );
        group.item("b", "Option B"); // uses default check / hidden placeholder
    });
```

With `fixed_content(true)` and no explicit `unselected_icon`, an invisible
check glyph reserves space so the text does not shift when selection changes.

---

## Performance & design rationale

| Technique | Benefit |
|---|---|
| **String moves via `mem::take`** | No `Clone` on the hot path. Items are single-use per frame. |
| **Lazy theme resolution** | `MudTheme::fetch` is called at most once per frame, and only when a border, divider, or selection fill actually needs a colour. |
| **Allocation-free sizing** | `cell_width`, `separator_count`, `display_index`, and corner calculations use saturating integer arithmetic; no temporary `Vec`s. |
| **Conditional height buffer** | Custom-content heights are stored in a reusable `Vec` only when at least one item has custom content. Plain button groups allocate nothing for heights. |
| **Memory-backed ripple** | Ripple start times live in `egui::Memory` keyed by item index. The caller does not need to own ripple state, and the group remains stateless between frames. |
| **Invisible probes** | Measurement runs in a separate `Ui` that never allocates parent space, paints nothing, and registers no interaction. |
| **Finite guards** | `finite_or_zero` prevents NaN/∞ values from unbounded scroll areas from poisoning layout arithmetic. |

---

## Visual parity notes

- **Shared outline** — Outlined groups draw one rounded border around the
  entire group and square inner corners on individual cells.
- **Accent under-fill** — Selected standard buttons receive a square-cornered
  accent fill underneath the button, preventing the button’s rounded fill from
  leaving notches at cell boundaries.
- **Hover overlay** — Custom-content cells receive a subtle white overlay
  (`HOVER_OVERLAY_ALPHA = 10`) on hover, matching MudButton feedback.
- **Focus ring** — Custom-content cells draw a 2 px accent ring when focused.
- **Ripple** — A 350 ms expanding circle is painted on click for both standard
  and custom cells.
- **Typography** — Text rendering is delegated to `MudButton`, which already
  implements MudBlazor’s size and colour hierarchy.
- **Disabled state** — Disabled groups and items suppress clicks, disable
  custom content, and rely on `MudButton`’s muted visuals.

---

## Testing

The source file contains an extensive `#[cfg(test)] mod tests` block covering:

- Size-preset height mapping
- Radius clamping and truncation
- Finite-value repair
- Stretch-layout truth tables
- Cell-width arithmetic (including non-finite inputs)
- Vertical cell-width behaviour
- Custom-content padding and minimum height
- Separator counts and RTL index mapping
- Corner rounding for horizontal, vertical, and single-cell groups
- Theme/accent decision helpers
- Selection semantics for `Single`, `Toggle`, and `Multi`
- Icon resolution (check mark, fixed content, hidden placeholders)
- Builder defaults and field assignments
- Collector insertion order

Run with:

```bash
cargo test
```

---

## License

This component is provided under the same license as the surrounding
`mud_egui` project. See the repository root for details.

===== ./crates/mud_egui_components/src/layout/container/README.md =====

# `MudContainer` for `mud_egui`

A responsive, centered, width-limiting container for [egui](https://github.com/emilk/egui),
ported from [MudBlazor](https://mudblazor.com)'s `MudContainer`.

It caps its content at a Material Design breakpoint, centers that content
horizontally, applies responsive side gutters, and resolves Bootstrap- or
Tailwind-style utility classes into an `egui::Frame`.

> **Status note.** This file set was written without access to a Rust toolchain
> or to the private `mud_egui_*` crates, so neither the source nor its test
> suite has been compiled or executed by the author. Everything that touches
> those crates (`ComponentBase`, `BsStyle`, `TwStyle`, `TwState`, `MudTheme`,
> `MaxWidth`) uses only the signatures already present in the original file.
> Run `cargo test` before shipping. See [Known limitations](#known-limitations).

---

## Table of contents

1. [What MudBlazor's `MudContainer` actually has](#what-mudblazors-mudcontainer-actually-has)
2. [Quick start](#quick-start)
3. [Architecture](#architecture)
4. [API reference](#api-reference)
5. [Breakpoints and gutters](#breakpoints-and-gutters)
6. [Styling pipeline and visual behaviour](#styling-pipeline-and-visual-behaviour)
7. [Events, state and thread-safe handles](#events-state-and-thread-safe-handles)
8. [Real-world examples](#real-world-examples)
9. [Performance choices and design rationale](#performance-choices-and-design-rationale)
10. [Testing](#testing)
11. [Changes from the previous version](#changes-from-the-previous-version)
12. [Known limitations](#known-limitations)

---

## What MudBlazor's `MudContainer` actually has

Before extending the component, the published MudBlazor API reference was
checked ([mudblazor.com/api/MudContainer](https://mudblazor.com/api/MudContainer)).
`MudContainer` is a layout primitive with exactly these parameters:

| MudBlazor parameter | Type              | Default          | Meaning                                                                 |
|---------------------|-------------------|------------------|-------------------------------------------------------------------------|
| `MaxWidth`          | `MaxWidth`        | `MaxWidth.Large` | Max width; grows with screen size; `false` disables it                  |
| `Fixed`             | `bool`            | `false`          | Set max width to the min width of the current breakpoint                |
| `Gutters`           | `bool`            | `true`           | Side padding (added in MudBlazor v7, replacing `DisableGutters`)        |
| `ChildContent`      | `RenderFragment`  | `null`           | The content                                                             |
| `Class`             | `string`          | `null`           | User class names, space separated                                       |
| `Style`             | `string`          | `null`           | User styles applied on top of the component's own                       |
| `Tag`               | `object`          | `null`           | Arbitrary user data                                                     |
| `UserAttributes`    | `Dictionary`      | empty            | Extra attributes splatted onto the HTML tag (ARIA, custom `id`, ...)    |

It has **no** `Value` / `ValueChanged` / `ValueExpression`, `Variant`, `Color`,
`Size`, `ReadOnly`, adornment slots, or input events (`OnClick`, `OnBlur`,
`OnKeyDown`, `OnChange`). Those belong to input and button components.
Because the goal was to avoid adding what the real component does not have,
this port does **not** invent them either.

Everything beyond the table above is explicitly an **extension** and is marked
as such in the [API reference](#api-reference).

---

## Quick start

```rust
use mud_egui_core::enums::max_width::MaxWidth;

// Keep the container in your app state so caches and animations persist.
struct Page {
    container: MudContainer,
}

impl Page {
    fn new() -> Self {
        Self {
            container: MudContainer::new()
                .max_width(MaxWidth::Md)
                .classes(["bg-surface", "p-4", "rounded-lg", "elevation-2"]),
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        self.container.show(ui, |ui| {
            ui.heading("Centered, at most 960 px wide");
            ui.label("Gutters are 24 px on wide screens, 16 px below 600 px.");
        });
    }
}
```

`ui.add(MudContainer::new())` also works through the `Widget` impl, but it adds
an *empty* container and re-resolves styles each frame. Use `show` for content.

---

## Architecture

```
mud_container.rs
├── Pure layout layer (no egui state; fully unit-testable)
│   ├── MudContainerMetrics   breakpoint table, gutters, snap, resolve, centering
│   ├── LayoutSpec            Copy descriptor: max_width / fluid / fixed / gutters
│   └── HorizontalLayout      Copy result: gutter, content, outer, effective_max, breakpoint
│
├── Style layer
│   ├── StyleFlavor           Bootstrap | Tailwind
│   ├── StyleOverrides        typed "Style" parameter (fill, stroke, padding, outer margin)
│   └── fade_frame / focus ring / gutter composition (private helpers)
│
├── State & events layer
│   ├── InteractionState      previous-frame hover / active / focus / dark / disabled
│   ├── Edge                  Steady | Rising | Falling
│   ├── EventPlan             pure "which callbacks fire" decision
│   └── ContainerHandle       lock-free, Send + Sync live-state window
│       └── ContainerSnapshot
│
└── MudContainer              builder + show / show_with_layout glue
    └── tests                 pure math, planning, handle, builders, real egui passes
```

### Frame pipeline (`show_with_layout`)

1. Read `available_rect_before_wrap` and compute a `HorizontalLayout` (pure math).
2. Compare the viewport breakpoint with the previous frame and fire
   `on_breakpoint_change` if it changed (or this is the first frame).
3. Create a child `Ui` whose rect is horizontally offset to center the content.
   (Offsetting X is deliberate: `ui.add_space` inserts *vertical* space in a
   top-down parent.)
4. Choose the container's own `Sense` (see [Click passthrough](#click-passthrough)).
5. Resolve the frame: **classes → `StyleOverrides` → disabled fade → focus ring →
   responsive gutters.**
6. Run the user's content with the content column pinned to `content_width`
   (skipped when unbounded), disabled via `Ui::disable` if needed.
7. Reserve a full-width row in the parent using a **sanitized** rect.
8. Publish an ARIA label to egui's accessibility tree, attach an optional
   tooltip, plan and fire callbacks from state edges, record interaction state
   for the next frame, and publish to any live `ContainerHandle`.

### Click passthrough

egui hit-tests widgets in registration order and a later registration wins.
The container reserves its own rect *after* its children are drawn, so an
interactive `Sense` on that rect would sit on top of everything inside it and
steal clicks and keyboard focus from children such as text fields.

| Registered callbacks                         | Container `Sense`  | Children receive input? |
|----------------------------------------------|--------------------|-------------------------|
| none, or only `on_hover` / `on_unhover` / `tooltip` | `Sense::hover()`   | yes                     |
| any of `on_click`, `on_focus`, `on_blur`     | `Sense::click()`   | **no**: container is the click target |
| `disabled(true)`                             | `Sense::hover()`   | children are disabled   |

The container never senses **drag**, so it cannot interfere with drag-to-scroll
in an enclosing `ScrollArea`. If you need a clickable region *and* typable
children, nest two containers.

---

## API reference

### Builder methods and public fields

"MudBlazor" column: **✔** = exists on the real `MudContainer`; **ext** = Rust
extension; **n/a** = no equivalent concept.

| Member (builder = field)                | Type                                   | Default        | MudBlazor | Description |
|-----------------------------------------|----------------------------------------|----------------|-----------|-------------|
| `max_width(MaxWidth)`                   | `MaxWidth`                             | `Lg`           | ✔ `MaxWidth` | Breakpoint ceiling. Ignored when `fluid` or `fixed` is on. |
| `fixed(bool)`                           | `bool`                                 | `false`        | ✔ `Fixed`    | Snap ceiling to the breakpoint of the offered width. |
| `gutters(bool)`                         | `bool`                                 | `true`         | ✔ `Gutters`  | 16 px below 600 px offered width, 24 px otherwise. |
| `show(ui, \|ui\| ...)`                  | closure                                | n/a            | ✔ `ChildContent` | Draws the container and its content. |
| `class(&str)` / `classes(iter)`         | `ComponentBase` classes                | empty          | ✔ `Class`   | Space-separated utility classes; duplicates ignored. |
| `style(StyleOverrides)` / `fill` / `stroke` / `padding` | `StyleOverrides`       | none           | ✔ `Style` (typed) | Overrides applied after class resolution. |
| `tag(value)` / `tag_ref::<T>()`         | `Option<Box<dyn Any + Send + Sync>>`   | `None`         | ✔ `Tag`      | Arbitrary user data. |
| `attr(k, v)` / `attrs(iter)` / `user_attribute(k)` | `BTreeMap<String, String>`  | empty          | ✔ `UserAttributes` | Extra attributes; `"aria-label"` doubles as label fallback. |
| `fluid(bool)`                           | `bool`                                 | `false`        | ext (≈ `MaxWidth.False`) | Fill the full offered width. Wins over `fixed`. |
| `disabled(bool)`                        | `bool`                                 | `false`        | ext       | Disables children, fades frame, silences callbacks. |
| `elevation(u8)`                         | `u8` (0–24, clamped)                   | `0`            | ext       | Forwarded to `ComponentBase::set_elevation`. |
| `flavor(StyleFlavor)`                   | `StyleFlavor`                          | `Bootstrap`    | ext       | Bootstrap or Tailwind class resolution. |
| `id(impl Into<Id>)`                     | `Option<Id>`                           | `None`         | ext (≈ `UserAttributes["id"]`) | Salts the child `Ui` id. |
| `aria_label(impl Into<String>)`         | `Option<String>`                       | `None`         | ext (≈ `UserAttributes["aria-label"]`) | Published to egui's accessibility tree (not a visible tooltip). |
| `tooltip(impl Into<String>)`            | `Option<String>`                       | `None`         | ext       | Visible hover tooltip. |
| `focus_ring(bool)`                      | `bool`                                 | `true`         | ext       | Animated focus ring on interactive containers. |
| `memoize_style(bool)`                   | `bool`                                 | `false`        | ext       | Opt-in cache of the class-resolved `Frame`. |
| `on_hover(f)`                           | `FnMut() + Send + Sync`                | `None`         | ext       | Pointer entered (rising edge). |
| `on_unhover(f)`                         | `FnMut() + Send + Sync`                | `None`         | ext       | Pointer left (falling edge). |
| `on_click(f)`                           | `FnMut() + Send + Sync`                | `None`         | ext       | Primary click. Upgrades `Sense` to click. |
| `on_focus(f)`                           | `FnMut() + Send + Sync`                | `None`         | ext       | Gained keyboard focus (rising edge). Upgrades `Sense`. |
| `on_blur(f)`                            | `FnMut() + Send + Sync`                | `None`         | ext       | Lost keyboard focus (falling edge). Upgrades `Sense`. |
| `on_breakpoint_change(f)`               | `FnMut(MaxWidth) + Send + Sync`        | `None`         | ext       | Fires on first frame and whenever the viewport bucket changes. |
| `base` (public field)                   | `ComponentBase`                        | empty          | ✔ (`Class` storage) | Direct access for extra mutations (`add_class`, `set_elevation`, ...). |
| `interaction` (public field)            | `InteractionState`                     | all `false`    | ext       | Previous-frame flags used by Tailwind variants. |
| `bs_style` / `tw_style` (public fields) | `BsStyle` / `TwStyle`                  | default        | ext       | Resolver caches. |

### Constructors and introspection

| Method | Returns | Description |
|--------|---------|-------------|
| `MudContainer::new()` | `Self` | Defaults: `Lg`, gutters on, Bootstrap. |
| `for_breakpoint(MaxWidth)` | `Self` | Container capped at a breakpoint. |
| `full_width()` | `Self` | `MaxWidth::False`; gutters stay on (like MudBlazor). |
| `from_spec(LayoutSpec)` | `Self` | Build from a `Copy` layout descriptor. |
| `layout_spec()` | `LayoutSpec` | Current layout-affecting settings. |
| `preview_layout(offered)` | `HorizontalLayout` | What `show` *would* compute, with no egui state. |
| `last_layout()` | `Option<HorizontalLayout>` | Layout from the most recent frame. |
| `handle()` | `ContainerHandle` | Lazily-created thread-safe live-state handle. |
| `effective_aria_label()` | `Option<&str>` | Explicit label, else `"aria-label"` attribute. |
| `invalidate_style_cache()` | `()` | Drop the memoized frame and bump the epoch. |
| `reset_interaction()` | `()` | Clear hover/active/focus flags and ring animation. |
| `show(ui, add_contents)` | `InnerResponse<R>` | Draw; closure receives `&mut Ui`. |
| `show_with_layout(ui, add_contents)` | `InnerResponse<R>` | Draw; closure also receives `&HorizontalLayout`. |

### `HorizontalLayout`

| Field / method | Type | Description |
|----------------|------|-------------|
| `gutter` | `f32` | Side gutter per edge (px). |
| `content_width` | `f32` | Width for child content (infinite for fluid in an unbounded parent). |
| `outer_width` | `f32` | Content plus both gutters; never exceeds a finite offer. |
| `effective_max` | `MaxWidth` | Ceiling after fluid/fixed resolution. |
| `breakpoint` | `MaxWidth` | Bucket of the *offered* width (`Xs` for non-finite offers). |
| `gutters_total()` | `f32` | `2 × gutter`. |
| `is_bounded()` | `bool` | `outer_width.is_finite()`. |
| `offset_in(offered)` | `f32` | Centering offset. |

### `StyleOverrides`

| Field | Type | Replaces |
|-------|------|----------|
| `fill` | `Option<Color32>` | `Frame::fill` |
| `stroke` | `Option<Stroke>` | `Frame::stroke` |
| `padding` | `Option<Margin>` | `Frame::inner_margin` (gutters are added afterward) |
| `outer_margin` | `Option<Margin>` | `Frame::outer_margin` |

### Visual variants

`MudContainer` has no `Color` / `Variant` / `Size` parameters, exactly like
MudBlazor. Its visual variations come from:

| Axis | Values | Source |
|------|--------|--------|
| Width ceiling | `Xs 444`, `Sm 600`, `Md 960`, `Lg 1280`, `Xl 1920`, `Xxl 2560`, `False ∞` | `MaxWidth` |
| Fixed mode | off / snap to viewport breakpoint | `fixed` |
| Gutters | none / 16 px / 24 px (automatic) | `gutters` |
| Surface (colour, radius, elevation) | any palette colour / utility class | `classes`, `elevation`, `StyleOverrides` |
| Style system | Bootstrap / Tailwind (with `hover:`, `active:`, `dark:`) | `flavor` |
| State | idle / hover (Tailwind) / focused ring / disabled fade | interaction + `disabled` |

### Binding slots

| MudBlazor concept | Present on `MudContainer`? | Rust equivalent |
|-------------------|----------------------------|-----------------|
| `Value` / `ValueChanged` / `ValueExpression` | No | none (a container has no value) |
| `ChildContent` | Yes | the closure passed to `show` |
| `AdornmentStart` / `AdornmentEnd` | No | none |

### Free types

| Type | Purpose |
|------|---------|
| `MudContainerMetrics` | Namespace of pure layout functions: `to_pixels`, `label`, `snap_breakpoint`, `gutter_for`, `resolve`, `centering_offset`, plus the pixel constants. |
| `LayoutSpec` | `Copy` struct of the four layout settings with `resolve(offered)`. |
| `HorizontalLayout` | `Copy` result of a layout resolution. |
| `StyleFlavor` | `Bootstrap` (default) or `Tailwind`. |
| `StyleOverrides` | Typed equivalent of MudBlazor's `Style`. |
| `InteractionState` | Previous-frame flags; `to_tw_state()` maps to `TwState` and respects `disabled`. |
| `Edge` | `Steady` / `Rising` / `Falling`; `Edge::detect(prev, now)`. |
| `EventPlan` | Which callbacks fire this frame; `EventPlan::compute(...)`. |
| `ContainerHandle` | Cloneable, `Send + Sync` window into live state. |
| `ContainerSnapshot` | Plain-data copy of a handle's state. |

---

## Breakpoints and gutters

```
offered width →   0 ─────── 600 ─────── 960 ─────── 1280 ────── 1920 ────── 2560 ──▶
viewport bucket →     Xs         Sm          Md          Lg          Xl         Xxl
ceiling (Max…)  →    444        600         960        1280        1920        2560
gutter          →     16 px        24 px (from 600 px upward)
```

Resolution rules (all enforced by `MudContainerMetrics::resolve`):

* `NaN` and negative offers are treated as `0`.
* An infinite offer yields the breakpoint cap, or an unbounded width for
  `fluid` / `MaxWidth::False`.
* `fixed` replaces the ceiling with the viewport bucket's ceiling.
  `fluid` beats `fixed`.
* Gutters never exceed half of a finite offer, so tiny viewports cannot overflow.
* For finite offers, `outer_width <= offered` always holds.
* `Xs` is **444**, not 0, matching MudBlazor's CSS.

Worked example, `MaxWidth::Lg`, gutters on:

| Offered | Gutter | Content | Outer | Notes |
|--------:|-------:|--------:|------:|-------|
| 500     | 16     | 468     | 500   | small gutter |
| 800     | 24     | 752     | 800   | below the cap |
| 2000    | 24     | 1232    | 1280  | capped, centered with a 360 px offset |
| 10      | 5      | 0       | 10    | gutter clamped to half the offer |

---

## Styling pipeline and visual behaviour

Order of operations for the frame:

```
utility classes (Bootstrap | Tailwind)
   → StyleOverrides (fill / stroke / padding / outer margin)
   → disabled fade (fill and stroke alpha × 0.6)
   → focus ring (interactive containers only)
   → responsive gutters added to inner_margin left/right
```

* **Padding composes with gutters.** A `p-4` class plus a 24 px gutter yields
  `p-4 + 24` on left and right; top and bottom are untouched.
* **Disabled.** Children are disabled through egui's own disabled styling
  (non-interactive and faded), the frame fill/stroke are faded, the container
  senses hover only, and no callbacks fire.
* **Focus ring.** For containers with a click-class callback, a 2 pt stroke in
  the palette's primary colour (`"MudPalettePrimary"`, falling back to egui's
  selection stroke colour) fades in and out over 120 ms. The stroke width is
  reserved even when the ring is idle, so focusing never shifts layout.
  Disable with `.focus_ring(false)`.
* **Tailwind variants.** Hover/active/dark flags are the *previous* frame's
  (immediate mode cannot know this frame's hover before drawing), which is
  imperceptible at normal frame rates.
* **Shadows, radii and ripple** are produced by your utility classes through
  `ComponentBase` and the style resolvers (for example `elevation-N`,
  `rounded-lg`). `MudContainer` itself adds none, just like MudBlazor, whose
  container has no inherent elevation or border radius.

---

## Events, state and thread-safe handles

### Edge-triggered callbacks

Hover and focus callbacks are derived from level changes between frames:

```rust
let plan = EventPlan::compute(
    prev_hovered, prev_focused,   // last frame
    hovered, focused, clicked,    // this frame
    disabled,
);
```

Because planning is a pure function, the firing rules are covered by tests
without egui. A disabled container always returns an empty plan.

### Breakpoint callbacks

```rust
let mut page = MudContainer::new().on_breakpoint_change(|bp| {
    println!("breakpoint is now {}", MudContainerMetrics::label(bp));
});
```

Fires on the first frame and on every bucket crossing. Unlike `effective_max`
(constant for a capped container), `breakpoint` tracks the *viewport*, so it is
the right signal for adaptive layouts.

### `ContainerHandle`: reading live state from anywhere

```rust
let mut page = MudContainer::new().max_width(MaxWidth::Lg);
let handle = page.handle();                 // cheap; clone it freely

// Another component, another thread, a test:
let snap = handle.snapshot();
if snap.outer_width < 700.0 || snap.viewport_breakpoint == MaxWidth::Xs {
    /* collapse the sidebar */
}
```

* The container performs the atomic stores **only if** a handle exists.
* `f32` values are stored as `AtomicU32` bit patterns; booleans and enums as
  `AtomicBool` / `AtomicU8`; the frame counter as `AtomicU64`.
* The frame counter uses `Release` (writer) / `Acquire` (reader); all other
  stores are `Relaxed`. A snapshot is field-wise atomic and is **eventually
  consistent** if a different thread is mid-frame; it is exact on the render
  thread.

---

## Real-world examples

### 1. Page shell with a responsive sidebar

```rust
struct App {
    page: MudContainer,
    sidebar_open: bool,
}

impl App {
    fn new() -> Self {
        Self {
            page: MudContainer::new().max_width(MaxWidth::Xl),
            sidebar_open: true,
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        let sidebar_open = &mut self.sidebar_open;
        self.page.show_with_layout(ui, |ui, layout| {
            // Collapse the sidebar on narrow content columns.
            *sidebar_open = layout.content_width >= 720.0;
            ui.horizontal(|ui| {
                if *sidebar_open {
                    ui.vertical(|ui| { ui.set_width(220.0); ui.label("Navigation"); });
                }
                ui.vertical(|ui| { ui.label("Main content"); });
            });
        });
    }
}
```

### 2. Fixed-breakpoint design

```rust
// Content width jumps between 600 / 960 / 1280 ... instead of flowing.
let mut c = MudContainer::new().fixed(true);
```

### 3. Full-bleed hero with a constrained inner column

```rust
let mut hero = MudContainer::full_width().gutters(false).classes(["bg-primary"]);
let mut inner = MudContainer::for_breakpoint(MaxWidth::Md);

hero.show(ui, |ui| {
    inner.show(ui, |ui| {
        ui.heading("Welcome");
    });
});
```

### 4. Tailwind variants and typed style overrides

```rust
let mut card = MudContainer::new()
    .flavor(StyleFlavor::Tailwind)
    .classes(["bg-surface", "hover:bg-primary/10", "rounded-lg", "p-4"])
    .style(StyleOverrides {
        stroke: Some(egui::Stroke::new(1.0, egui::Color32::from_gray(90))),
        ..Default::default()
    });
```

### 5. A clickable, keyboard-focusable card

```rust
let mut card = MudContainer::new()
    .max_width(MaxWidth::Sm)
    .classes(["bg-surface", "p-4", "rounded-lg"])
    .aria_label("Open settings")
    .tooltip("Settings")
    .on_click(|| println!("open settings"))
    .on_focus(|| println!("focused"))
    .on_blur(|| println!("blurred"));
// Children do not receive clicks here; the card itself is the target.
```

### 6. Metadata with `Tag` and `UserAttributes`

```rust
#[derive(Debug, PartialEq)]
struct PageMeta { route: &'static str }

let c = MudContainer::new()
    .tag(PageMeta { route: "/settings" })
    .attr("data-testid", "settings-page")
    .attr("aria-label", "Settings page");

assert_eq!(c.tag_ref::<PageMeta>(), Some(&PageMeta { route: "/settings" }));
assert_eq!(c.effective_aria_label(), Some("Settings page"));
```

### 7. Responsive-design tooling without egui

```rust
for width in [400.0, 800.0, 1500.0, 2600.0] {
    let l = MudContainer::new().max_width(MaxWidth::Lg).preview_layout(width);
    println!("{width:>6} → outer {:>6.0}  bucket {}",
        l.outer_width, MudContainerMetrics::label(l.breakpoint));
}
```

### 8. Cached style resolution for a hot, static container

```rust
let mut c = MudContainer::new()
    .classes(["bg-surface", "p-4"])
    .memoize_style(true);

// After you mutate the MudTheme palette outside egui's visuals:
c.invalidate_style_cache();
```

---

## Performance choices and design rationale

All claims below describe *structure* of the implementation; no benchmark
numbers are asserted, because none were measured.

| Choice | Rationale |
|--------|-----------|
| **Pure, `Copy` layout layer** (`MudContainerMetrics`, `LayoutSpec`, `HorizontalLayout`) | Layout is arithmetic on a few `f32`s and two one-byte enums: no allocation, no `egui` state, trivially testable and reusable (`preview_layout`). |
| **Narrow `Sense`** | The container only asks for `click` when a click-class callback exists, and never asks for drag. This avoids stealing input from children and from scroll-area drag. |
| **Lazy `ContainerHandle`** | The `Arc` is allocated on first `handle()` call; without a handle `show` does no atomic work and no extra allocation. Readers never take a lock. |
| **Edge-planned events** | `EventPlan` separates *deciding* from *firing*; both are cheap and decisions are pure. Callbacks are `Option<Box<dyn FnMut>>`, so unused events cost one `is_some` check. |
| **Opt-in frame memoization** | Resolving classes requires fetching the theme and building a `Frame`. With `memoize_style(true)` a 64-bit FNV-1a key (flavour, classes, elevation, ripple flag, dark mode, a small visuals fingerprint, Tailwind variant bits, epoch) is compared and the cached `Frame` is cloned on a hit. Hashing the class list is O(total class characters), so the win is largest when theme resolution dominates. It is **opt-in** because `MudTheme` values stored outside egui's visuals cannot be observed; call `invalidate_style_cache()` after such changes. |
| **FNV-1a for keys** | Deterministic across runs and platforms (no random `SipHash` seed), allocation-free, adequate for non-adversarial cache keys. |
| **Sanitized reserved rect** | A single `NaN`/`inf` rect would poison an enclosing `ScrollArea`'s measured size; `used_rect` and `finite_or` make that impossible. |
| **`floor` for gutter margins** | `egui::Margin` is integral; flooring guarantees the margins never exceed the float gutter budget on tiny viewports. |
| **Self-contained focus animation** | The fade state lives in the container (`f32`, stepped by `stable_dt`), so there is no `Id` collision when several unnamed containers share a `Ui`. A repaint is requested only while the animation is in flight. |
| **Constant reserved ring stroke** | Changing a frame's stroke width changes its total size in egui; reserving the width up front means focusing never causes layout shift. |
| **`ui.disable()` for disabled** | Delegates to egui's built-in disabled behaviour instead of re-implementing it. |

Memory layout: `HorizontalLayout` is four `f32` + two one-byte enums,
`InteractionState` is five `bool`s, `StyleOverrides` is four `Option`s of
small `Copy` types, and `ContainerSnapshot` is plain data; all are `Copy`. The
container itself owns heap data only for what you opt into (callbacks, tag,
attributes, label, handle, cached frame).

---

## Testing

The `mod tests` block at the bottom of `mud_container.rs` contains:

* **Pure metrics**: breakpoint table, monotonicity, snap thresholds, gutters,
  fluid/fixed/infinite/NaN/negative/tiny offers, and a brute-force sweep of
  `resolve` invariants across offers × gutters × fluid × fixed × all breakpoints.
* **Helpers**: sanitizers, margin arithmetic, `used_rect`, `approach`,
  `fade_frame`, FNV-1a, visuals fingerprint, `MaxWidth` ↔ `u8` round trip.
* **Events**: `Edge`, `EventPlan` (edges, clicks, disabled), and `fire`
  invoking exactly the planned callbacks once.
* **Builders and state**: defaults, presets, `LayoutSpec` round trips,
  `Tag`, `UserAttributes`, ARIA fallback, class parsing/dedup, elevation clamp,
  `StyleOverrides`, interaction state, `reset_interaction`.
* **Cache keys**: stability, sensitivity to classes/elevation/dark/visuals/
  flavour/epoch, Tailwind hover sensitivity vs. Bootstrap insensitivity.
* **Handle**: zeroed start, publish/snapshot round trip, shared clones, lazy
  creation, `Send + Sync`, cross-thread read.
* **Real egui passes** (via `egui::Context::run` and a `CentralPanel`):
  closure value, resolved layout, full-width reservation, fixed and fluid
  modes, handle frame counting, breakpoint callback sequence, disabled and
  enabled children, memoization reuse/invalidate, the `Widget` impl.

Run with:

```bash
cargo test -p <crate-containing-mud_container>
```

Pointer- and keyboard-driven callbacks are tested through `EventPlan` and
`fire` rather than synthetic egui input events.

---

## Changes from the previous version

| Area | Before | Now |
|------|--------|-----|
| MudBlazor parity | `Class`, `Gutters`, `MaxWidth`, `Fixed`, `ChildContent` | Adds `Style` (typed), `Tag`, `UserAttributes` |
| `aria_label` | Shown as a **hover tooltip** | Published to egui's accessibility tree; visible text moved to a separate `tooltip` |
| `disabled` | Silenced callbacks only | Also disables children and fades the frame |
| Container `Sense` | `click_and_drag` for any callback | `hover` for hover/tooltip, `click` only for click/focus/blur; never drag |
| Blur detection | `lost_focus()` or inferred | Pure focus-level edge detection |
| Layout result | 4 fields | Adds `breakpoint`; `show_with_layout` hands it to the closure |
| Live state | none | `ContainerHandle` / `ContainerSnapshot` |
| Style resolution | Every frame | Every frame by default; opt-in memoization |
| Focus visuals | none | Animated focus ring on interactive containers |
| Events | 5 | Adds `on_breakpoint_change` |

---

## Known limitations

* **Not compiled or run by the author** (no Rust toolchain was available).
  Treat the first `cargo test` as part of review.
* **Depends on the private `mud_egui_*` crates.** The code assumes these
  already-used APIs: `ComponentBase::{new, classes, class_names, has_class,
  add_class, set_elevation, elevation_value, ripple_enabled}`,
  `BsStyle::build_frame`, `TwStyle::build_frame`, `TwState { hovered, active,
  dark }`, `MudTheme::{fetch, current_palette}`, `palette.resolve(&str)`, and
  `MaxWidth` being `Copy + Debug + PartialEq` with variants
  `Xs/Sm/Md/Lg/Xl/Xxl/False`. `class_names()` must be iterable with `.iter()`
  and hashable items. `elevation_value()` must be an integer.
* **egui version.** The code assumes the `i8`-field `Margin` and `UiBuilder`
  era of egui (0.31 or later). Version-fragile APIs (such as `Shadow` and
  `rect_stroke`) were deliberately avoided; `Frame`'s `fill`, `stroke`,
  `inner_margin` and `outer_margin` fields, `Ui::disable`,
  `Response::widget_info`, and `Context::run` / `CentralPanel` in tests are
  used. Tests also depend on `Ui::set_max_width` behaving as documented.
* **Frame memoization cannot observe `MudTheme` changes** that do not alter
  egui's visuals; call `invalidate_style_cache()`.
* **Interactive containers reserve a 2 pt stroke** for the focus ring
  (`focus_ring(false)` removes it).
* **Callback-bearing containers capture input from children** by design; see
  [Click passthrough](#click-passthrough).
* **Unbounded offers report `Xs`** as the viewport bucket, because there is no
  meaningful bucket for an infinite width.
* **Not part of MudBlazor's `MudContainer`, and intentionally absent:**
  `Value`/`ValueChanged`/`ValueExpression`, `Variant`, `Color`, `Size`,
  `ReadOnly`, `AdornmentStart`/`AdornmentEnd`, `OnKeyDown`, `OnChange`.

## License

Use the license of the surrounding `mud_egui` workspace.

===== ./crates/mud_egui_components/src/layout/divider/README.md =====

# MudDivider for egui

A horizontal or vertical rule for [egui](https://github.com/emilk/egui), ported from
[MudBlazor's `MudDivider`](https://mudblazor.com/components/divider). It draws with the
theme palette, supports MudBlazor's `Inset` / `Middle` margins, light and absolute
variants, vertical flex stretching, and (as an extension) a centered text label.

## Contents

1. [Design principle: API parity, not API invention](#design-principle-api-parity-not-api-invention)
2. [Architecture](#architecture)
3. [API reference](#api-reference)
4. [Visual behaviour](#visual-behaviour)
5. [Examples](#examples)
6. [Performance and memory choices](#performance-and-memory-choices)
7. [Testing](#testing)
8. [Differences from MudBlazor](#differences-from-mudblazor)

---

## Design principle: API parity, not API invention

The MudBlazor API for `MudDivider` was checked against the published API docs, the
component source (`MudDivider.razor.cs`) and the stylesheet (`_divider.scss`) before
this refactor. The real component has exactly five parameters plus the shared
`MudComponentBase` members:

| MudBlazor parameter | Type          | Default     |
|---------------------|---------------|-------------|
| `Absolute`          | `bool`        | `false`     |
| `DividerType`       | `DividerType` | `FullWidth` |
| `FlexItem`          | `bool`        | `false`     |
| `Light`             | `bool`        | `false`     |
| `Vertical`          | `bool`        | `false`     |
| `Class` (base)      | `string`      | `null`      |
| `Style` (base)      | `string`      | `null`      |
| `Tag` (base)        | `object`      | `null`      |
| `UserAttributes` (base) | `Dictionary<string, object>` | — |

A divider has no value, no click or focus behaviour, no elevation and no variants.
Therefore this port deliberately has **no** `Value` / `ValueChanged` /
`ValueExpression`, no `OnClick` / `OnBlur` / `OnFocus` / `OnKeyDown` / `OnChange`,
no `Variant`, `Size`, `Elevation`, `Disabled`, `ReadOnly`, and no
`AdornmentStart` / `AdornmentEnd`. Adding them would create API that MudBlazor does
not have and that would do nothing.

## Architecture

```
mud_divider.rs
├── constants            INSET_START, MIDDLE_SIDE, TEXT_GAP, LIGHT_FACTOR, ...
├── pure helpers         sanitize_*, resolve_line_color, inset_rect,
│                        absolute_endpoints, label_row_height
├── MudDivider           public builder struct (all fields public)
│   ├── builder methods  vertical, divider_type, flex_item, light, absolute,
│   │                    color, text, thickness, height, flavor, classes, tag
│   ├── pure helpers     margins_for, margins_for_dir, line_segment_width,
│   │                    vertical_extent
│   └── rendering        show -> render_content / render_absolute
│                        render_vertical / render_horizontal / draw_labelled
├── impl Widget          ui.add(MudDivider::new())
└── mod tests            pure-function tests + headless egui frame tests
```

The rule of the module: **every decision that can be made from numbers is a pure
function**. Rendering methods only fetch the theme, call those functions, allocate
space and paint. This keeps almost all behaviour testable without a GPU or window.

Render flow for `show(ui)`:

1. Fetch the `MudTheme` and palette from the egui context.
2. Choose the line color (`resolve_line_color`): default → palette divider color,
   semantic color → resolved palette color (fallback: divider color), then `light`
   fades it.
3. If `absolute`: paint on the parent's edge, allocate no space, return.
4. Otherwise resolve the utility-class `Frame` (Bootstrap or Tailwind flavor) and,
   inside it, apply the divider-type margins and draw the vertical, horizontal or
   labelled line.

## API reference

### Parameters that exist in MudBlazor

| Rust | MudBlazor | Type | Default | Description |
|------|-----------|------|---------|-------------|
| `.vertical(bool)` | `Vertical` | `bool` | `false` | Draw a vertical line. |
| `.divider_type(DividerType)` | `DividerType` | `DividerType` | `FullWidth` | `FullWidth`: no margin. `Inset`: 72 pt start margin. `Middle`: 16 pt left and right. |
| `.flex_item(bool)` | `FlexItem` | `bool` | `false` | Vertical dividers stretch to the available height. |
| `.light(bool)` | `Light` | `bool` | `false` | Lighter line color. |
| `.absolute(bool)` | `Absolute` | `bool` | `false` | Pin to the bottom (horizontal) or right (vertical) edge of the parent rect; takes no layout space. |
| `.classes([..])` | `Class` | iterator of `AsRef<str>` | empty | Utility classes, resolved by the selected flavor. |
| `.tag(T)` / `.tag_ref::<T>()` | `Tag` | `T: Any + Send + Sync` | `None` | Attach and read back user data. |

### Extensions kept from the original port

| Rust | Type | Default | Description |
|------|------|---------|-------------|
| `.color(Color)` | `Color` | `Default` | Semantic color role of the line. |
| `.text(impl Into<Arc<str>>)` | `Option<Arc<str>>` | `None` | Centered label (horizontal dividers only). Empty string clears it. |
| `.thickness(f32)` | `f32` | `1.0` | Line thickness in points, sanitized (min `0.5`, non-finite → `1.0`). |
| `.height(f32)` | `f32` | `24.0` | Height of a non-flex vertical divider (negative → `0`, non-finite → `24`). |
| `.flavor(StyleFlavor)` | `StyleFlavor` | default | Bootstrap or Tailwind utility resolver. |

### Not applicable in egui

| MudBlazor | Why it is not ported |
|-----------|----------------------|
| `Style` | There are no inline CSS strings in egui. Use `.classes(..)` and the typed fields. |
| `UserAttributes` | There is no HTML tag to splat attributes onto. |

### Methods

| Method | Description |
|--------|-------------|
| `MudDivider::new()` | Full-width horizontal divider with default settings. |
| `show(self, ui) -> Response` | Render and return the line's `Response`. |
| `ui.add(divider)` | Same as `show` through the `Widget` impl. |

## Visual behaviour

Values follow `_divider.scss` from MudBlazor:

| Case | Behaviour |
|------|-----------|
| Default | 1 pt line in the palette's divider color, spanning the available width. |
| `Inset` | 72 pt start margin (`margin-inline-start`): mirrored to the right in right-to-left layouts. |
| `Middle` | 16 pt left and right margin, no vertical margin. |
| `Light` | Line color multiplied by `0.5` (see below). |
| `Vertical` | Fixed `height` (default 24 pt) or, with `flex_item`, the available height. |
| `Absolute` | Drawn on the parent's bottom edge (horizontal) or right edge (vertical), inset by the divider-type margins. |
| Labelled | Two line segments with the label centered between them and an 8 pt gap; label uses the theme's `body2` font and the weak text color. If the label is wider than the row, the segments shrink to zero rather than going negative. |

A divider is a static, non-interactive element, so it has no hover, focus, active or
disabled styling. The `Response` uses `Sense::hover()`; labelled dividers also expose
the label to AccessKit through `WidgetInfo`.

## Examples

### Basic

```rust
MudDivider::new().show(ui);
ui.add(MudDivider::new().thickness(2.0));
```

### Between list items (Inset)

```rust
for (i, item) in items.iter().enumerate() {
    list_row(ui, item);
    if i + 1 < items.len() {
        MudDivider::new().divider_type(DividerType::Inset).show(ui);
    }
}
```

### Labelled divider

```rust
ui.add(MudDivider::new().text("OR").color(Color::Primary));
```

### Vertical divider in a toolbar

```rust
ui.horizontal(|ui| {
    ui.button("Cut");
    MudDivider::new().vertical(true).flex_item(true).show(ui);
    ui.button("Copy");
});
```

### Pinned to the bottom of a card

```rust
egui::Frame::group(ui.style()).show(ui, |ui| {
    ui.set_min_height(120.0);
    ui.label("Card body");
    MudDivider::new().absolute(true).light(true).show(ui);
});
```

### Utility classes and user data

```rust
let divider = MudDivider::new()
    .flavor(StyleFlavor::Tailwind)
    .classes(["my-4"])
    .tag("section-break");

assert_eq!(divider.tag_ref::<&str>(), Some(&"section-break"));
divider.show(ui);
```

## Performance and memory choices

Being honest about scale: a divider is one of the cheapest widgets in a UI, so the
choices below remove avoidable work rather than chase micro-benchmarks.

- **No per-frame clone of the label.** The original cloned the `String` label into a
  `move` closure every frame. Rendering now borrows the label, and the label is stored
  as `Arc<str>`, so copying a divider specification around is a pointer copy.
  (egui's `layout_no_wrap` takes an owned `String`, so one allocation per labelled
  frame remains; egui's galley cache absorbs the layout cost.)
- **No per-frame closure boxing.** The old horizontal path built a `move` closure with
  captured clones even for unlabelled dividers; the new path is a straight `match`.
- **Pure functions on `Copy` data.** All geometry and color decisions take and return
  `f32`, `Margin`, `Rect` and `Color32` by value: no heap, no locking, nothing to
  synchronize.
- **Thread safety by construction.** The only shared structure is the optional tag,
  held as `Arc<dyn Any + Send + Sync>`.
- **Absolute dividers skip the utility frame** entirely, because they take no layout
  space.
- **Sanitization at the edges.** NaN, infinity and negative values can never reach the
  painter, so a bad number cannot cause layout explosions or invisible lines.

Not done, on purpose: a cache for the resolved frame. The `ComponentBase` / style
resolver APIs were not changed in this refactor, and invalidation rules for them are
owned by those crates.

## Testing

`cargo test` runs the module's tests:

- **Pure functions:** sanitizers, color choice (including `light`), margins for every
  `DividerType` (LTR and RTL), segment widths, vertical extents, inset rectangles and
  absolute-line endpoints.
- **Builder:** MudBlazor defaults, every parameter setter, number sanitizing, empty-label
  clearing, and tag round trips by type.
- **Headless frames:** a real `egui::Context` runs one 400×300 frame to check that
  horizontal dividers span the width, vertical dividers honor `height` or fill the
  space with `flex_item`, labelled dividers are taller than the line, `Inset` and
  `Middle` leave their margins, absolute dividers take no layout space, and the
  `Widget` impl works.

There are no event-callback tests because a divider has no events.

## Differences from MudBlazor

- **`Light`** derives its color by fading the divider color instead of reading a
  `DividerLight` palette entry. MudBlazor's stock palette makes "light" look darker
  than the default (upstream issue #5669); this port always makes it lighter.
- **Non-flex vertical height** is a fixed `height` (24 pt default) because CSS's
  `height: 100%` has no direct egui equivalent without a parent size.
- **Margins changed from the previous Rust port** to match the stylesheet:
  `Inset` is 72 pt (was 16) and `Middle` has no vertical margin (was 8).
- **`child_content`** changed from `Option<String>` to `Option<Arc<str>>`.
- **Absolute dividers** ignore the label and utility classes.

===== ./crates/mud_egui_components/src/layout/grid/README.md =====

# MudGrid for `mud_egui`

A twelve-track, mobile-first responsive layout grid for [egui](https://github.com/emilk/egui),
ported from MudBlazor's `MudGrid` / `MudItem`.

```rust
MudGrid::new()
    .spacing(3)
    .show(ui, |grid| {
        grid.item(MudItem::new().xs(12).md(6), |ui| { ui.label("Left");  });
        grid.item(MudItem::new().xs(12).md(6), |ui| { ui.label("Right"); });
    });
```

Three source files make up the module:

| File | Contents |
|------|----------|
| `mud_grid_span.rs` | `MudGridSpan`: the size one item asks for at one breakpoint (`Tracks`, `Star`, `Auto`, `Unset`), plus parsing, conversions and presets |
| `mud_item.rs` | `MudItem`: six spans (`xs`..`xxl`) and the mobile-first cascade |
| `mud_grid.rs` | `MudGrid`, `GridUi`, `ItemStyle`, the pure layout math and the headless planner |

---

## Contents

1. [What maps to MudBlazor, and what does not](#what-maps-to-mudblazor-and-what-does-not)
2. [Architecture](#architecture)
3. [API reference](#api-reference)
4. [Examples](#examples)
5. [Layout algorithm](#layout-algorithm)
6. [Performance and memory layout](#performance-and-memory-layout)
7. [Design rationale](#design-rationale)
8. [Behaviour changes from the previous version](#behaviour-changes-from-the-previous-version)
9. [Testing](#testing)

---

## What maps to MudBlazor, and what does not

The public MudBlazor API was checked before extending anything. `MudGrid` exposes `Spacing`
(gap in 4px steps, documented maximum 16, default 6) and `Justify`, and `MudItem` exposes
`xs`, `sm`, `md`, `lg`, `xl`, `xxl`, plus the usual `Class` / `Style` / `ChildContent`
inherited from the component base.

| MudBlazor | This crate | Notes |
|-----------|-----------|-------|
| `MudGrid.Spacing` | `MudGrid::spacing(u8)` | 4px per unit, clamped to **16** |
| `MudGrid.Justify` | `MudGrid::justify(Justify)` | `Start`, `Center`, `End`, `SpaceBetween`, `SpaceAround`, `SpaceEvenly` |
| `MudGrid.Class` | `MudGrid::classes([...])` | Resolved by the Bootstrap or Tailwind flavor |
| `MudGrid.Style` | `MudGrid::frame(Frame)` | egui has no CSS string; an explicit `egui::Frame` is the equivalent |
| `MudGrid.ChildContent` | closure passed to `MudGrid::show` | |
| `MudItem.xs` ... `xxl` | `MudItem::xs(..)` ... `xxl(..)` | Integers, `"2*"`, `"auto"` |
| `MudItem.Class` / `Style` | `ItemStyle` via `GridUi::styled_item` | Classes or an explicit `Frame` |
| `MudItem.ChildContent` | closure passed to `GridUi::item` | |

**Deliberately not added.** The checklist in `REFACTOR.md` lists `Variant`, `Color`, `Size`,
`Elevation`, `Disabled`, `ReadOnly`, `Value` / `ValueChanged` / `ValueExpression`,
`OnClick` / `OnFocus` / `OnBlur` / `OnKeyDown` / `OnChange`, `AriaLabel` and
`AdornmentStart` / `AdornmentEnd`. Those belong to input and button components. MudBlazor's
grid has none of them, and the instructions at the top of `REFACTOR.md` say not to add what
the real component lacks, so they are not invented here. The grid does expose the egui
equivalent of what its children need: the response covers the whole grid, and
`cell_rects()` gives every item's rectangle for your own hit-testing or focus handling.

---

## Architecture

```
┌──────────────────────────────────────────────────────────────┐
│ mud_grid.rs                                                  │
│                                                              │
│  GridUi ── collects (MudItem, ItemStyle?, content closure)   │
│     │                                                        │
│     ▼                                                        │
│  MudGrid::show                                               │
│    1. resolve frames   (BsStyle / TwStyle, grid + items)     │
│    2. resolve spans    (MudItem::resolve_span at breakpoint) │
│    3. measure Auto     (Probe | Cached)                      │
│    4. plan_into        ◄── pure math, no egui                │
│    5. draw row by row  (egui child Uis, unbounded height)    │
│    6. cache + respond                                        │
└──────────────┬───────────────────────────────┬───────────────┘
               │ uses                          │ uses
               ▼                               ▼
      mud_item.rs  (MudItem)         mud_grid_span.rs (MudGridSpan)
      six spans + cascade            Tracks | Star | Auto | Unset
```

**Pure layer.** `pack_rows_into`, `row_widths_into`, `row_offsets_into` and `plan_into` take
slices and write into caller-owned buffers. They know nothing about egui, which is why
nearly all behaviour is covered by ordinary unit tests and why `MudGrid::plan` can lay out
a grid with no `egui::Context` at all.

**Component layer.** `MudGrid::show` is a thin orchestrator: it resolves styling, calls the
pure layer, and draws.

**Builder layer.** `GridUi` records items first and draws afterwards. Knowing every span
up front is what makes row packing and `Auto` measurement possible.

---

## API reference

### `MudGridSpan`

| Member | Kind | Description |
|--------|------|-------------|
| `Unset` | variant (default) | Not specified here; cascades to the next smaller breakpoint |
| `Tracks(u8)` | variant | Fixed number of tracks, normalized to `1..=12` |
| `Star(f32)` | variant | Proportional share of leftover space; invalid weights become `0.0` |
| `Auto` | variant | Shrink to the content's natural width |
| `FULL`, `HALF`, `THIRD`, `QUARTER`, `SIXTH` | consts | `Tracks(12)`, `(6)`, `(4)`, `(3)`, `(2)` |
| `fixed(u8)` | `const fn` | `Tracks` with the count clamped to `1..=12` |
| `star(f32)` | fn | `Star` constructor |
| `normalized()` | fn | Never returns `Unset`; clamps tracks; repairs weights; idempotent |
| `or(fallback)` | fn | `self` if set, else `fallback` (one step of the cascade) |
| `is_set()`, `is_tracks()`, `is_star()`, `is_auto()` | `const fn` | Variant tests |
| `tracks()`, `weight()` | fn | Payload accessors (`0` / `0.0` for other variants) |
| `track_need()` | fn | Tracks reserved when packing rows (`Star`/`Auto`/`Unset` reserve 1) |
| `parse(&str)` | fn | `Option` parse: `""`, `"auto"`, `"*"`, `"2.5*"`, `"6"` |
| `FromStr` / `Display` | traits | Strict parse (`ParseSpanError`) and a syntax that round-trips |
| `From<i8,i16,i32,i64,isize,u8,u16,u32,u64,usize>` | traits | Integers clamp into `1..=12`, never panic |
| `From<&str>`, `From<String>`, `From<&String>` | traits | Lenient: garbage becomes `Unset` |
| `TRACK_COUNT` | const | `12` |

### `MudItem`

| Member | Description |
|--------|-------------|
| `new()` | All breakpoints unset (full row everywhere) |
| `full()`, `auto()`, `star(w)`, `equal(n)` | Presets that set every breakpoint |
| `xs(..)`, `sm(..)`, `md(..)`, `lg(..)`, `xl(..)`, `xxl(..)` | Set one breakpoint (accepts anything `Into<MudGridSpan>`) |
| `all(..)` | Set every breakpoint, replacing earlier values |
| `at(Breakpoint, ..)` | Set a breakpoint chosen at run time (`None` writes `xs`) |
| `get(Breakpoint)` | Raw stored span, no cascade |
| `ladder()` / `from_ladder([..;6])` | Convert to and from the six-element array |
| `is_empty()` | True when nothing is set |
| `resolve_span(Breakpoint)` | Cascaded, normalized span for one breakpoint |
| `resolve_all()` | The cascade at every breakpoint at once |
| `From<MudGridSpan>` | A bare span becomes an item that uses it everywhere |
| `Display` | `xs=12 md=6 lg=2*` (set breakpoints only) |

Breakpoint ranges (pixels of available width):

| `xs` | `sm` | `md` | `lg` | `xl` | `xxl` |
|------|------|------|------|------|-------|
| 0 | 600 | 960 | 1280 | 1920 | 2560 |

### `MudGrid`

| Member | Default | Description |
|--------|---------|-------------|
| `new()` / `default()` | | MudBlazor defaults |
| `spacing(u8)` | `6` (24px) | Gap in 4px units, clamped to `MAX_SPACING` (16) |
| `justify(Justify)` | `Start` | Distribution of leftover row space |
| `breakpoint(Breakpoint)` | auto | Pin the breakpoint instead of deriving it from width |
| `classes(iter)` | none | Utility classes for the grid's own frame |
| `flavor(StyleFlavor)` | default | Bootstrap or Tailwind resolver |
| `frame(Frame)` | none | Explicit frame; replaces the class-resolved one |
| `auto_measure(AutoMeasure)` | `Probe` | How `Auto` widths are learned |
| `disabled(bool)` | `false` | Suppresses hover/active in Tailwind variants |
| `show(ui, \|grid\| ..)` | | Draw; returns `InnerResponse<R>` |
| `plan(&items, width, &auto)` | | Headless layout, no egui context needed |
| `gap()` | | Gap in pixels |
| `active_breakpoint()` | | Breakpoint of the last frame |
| `last_width()` | | Width laid out against in the last frame |
| `last_cells()` | | `&[CellBox]` from the last frame |
| `cell_rects()` | | `&[Rect]` (screen space) per item from the last frame |

Public fields: `justify`, `spacing`, `breakpoint`, `base`, `flavor`, `bs_style`, `tw_style`,
`interaction`, `frame_override`, `measure`.

### `GridUi`

| Member | Description |
|--------|-------------|
| `item(span, \|ui\| ..)` | Add one item. `span` is a `MudItem` or a bare `MudGridSpan` |
| `styled_item(span, ItemStyle, \|ui\| ..)` | Add one item with its own classes or frame |
| `each(span, iter, \|ui, &value\| ..)` | One item per element, sharing a span (a Razor `@foreach`) |
| `spacer(span)` | An empty item that only occupies space |
| `len()` / `is_empty()` | Items added so far |

### Supporting types

| Type | Description |
|------|-------------|
| `ItemStyle` | `class(..)`, `classes(..)`, `frame(..)`, `is_empty()`; the per-item `Class` / `Style` |
| `AutoMeasure` | `Probe` (exact, closure runs twice for `Auto` cells) or `Cached` (closure runs once) |
| `CellBox` | `{ index, row, x, width }` plus `right()`; one planned cell |
| `InteractionState` | `hovered`, `active`, `focused`, `dark`, `disabled`; feeds Tailwind variants |
| `ParseSpanError` | Returned by strict `FromStr` |
| Constants | `MAX_SPACING` (16), `SPACING_UNIT` (4.0), `FALLBACK_WIDTH` (1280.0), `BREAKPOINT_SM` .. `BREAKPOINT_XXL` |

### Visual variants

The grid draws no chrome of its own. Its appearance is whatever its frame resolves to:
`classes(..)` through the Bootstrap or Tailwind flavor, or an explicit `frame(..)`. State
variants (hover, active, dark) come from `InteractionState`, captured from the previous
frame's response.

---

## Examples

### Responsive cards

```rust
MudGrid::new().spacing(4).show(ui, |grid| {
    for n in 0..6 {
        grid.item(MudItem::new().xs(12).sm(6).lg(4), move |ui| {
            ui.group(|ui| { ui.label(format!("Card {n}")); });
        });
    }
});
```

Full width on phones, two across from 600px, three across from 1280px.

### Sidebar plus content using stars

```rust
MudGrid::new().show(ui, |grid| {
    grid.item(MudItem::new().xs(12).md(3), |ui| { ui.label("Sidebar"); });
    grid.item(MudItem::new().xs(12).md("*"), |ui| { ui.label("Content"); });
});
```

On `md` and up the sidebar takes 3 tracks and the content takes everything left.

### Styled items

```rust
let card = ItemStyle::new().classes(["bg-surface", "p-4", "rounded-lg"]);

MudGrid::new().flavor(StyleFlavor::Tailwind).show(ui, |grid| {
    grid.styled_item(MudItem::new().xs(12).md(6), card.clone(), |ui| { ui.label("A"); });
    grid.styled_item(MudItem::new().xs(12).md(6), card,         |ui| { ui.label("B"); });
});
```

### Iterating data

```rust
let people = ["Ada", "Grace", "Linus"];
MudGrid::new().show(ui, |grid| {
    grid.each(MudItem::new().xs(12).sm(4), people, |ui, name| { ui.label(*name); });
});
```

### Centering a row of fixed-width buttons

```rust
MudGrid::new().justify(Justify::Center).show(ui, |grid| {
    grid.item(MudItem::auto(), |ui| { ui.button("Save"); });
    grid.item(MudItem::auto(), |ui| { ui.button("Cancel"); });
});
```

### Reacting to the active breakpoint

```rust
let mut grid = MudGrid::new();           // keep it across frames
grid.show(ui, |g| { /* ... */ });
if matches!(grid.active_breakpoint(), Some(Breakpoint::Xs)) {
    // render a compact header
}
```

### Headless layout (tests, tooling, snapshotting)

```rust
let items = [MudItem::new().xs(12).md(6); 2];
let cells = MudGrid::new().plan(&items, 1200.0, &[]);
assert_eq!((cells[1].row, cells[1].x, cells[1].width), (0, 612.0, 588.0));
```

### Parsing spans from configuration

```rust
let span: MudGridSpan = "2*".parse()?;               // strict: Err on garbage
let lenient = MudItem::new().md(some_user_string);   // lenient: garbage is Unset
```

### Opting into cached `Auto` measurement

```rust
MudGrid::new().auto_measure(AutoMeasure::Cached).show(ui, |grid| {
    grid.item(MudItem::auto(), |ui| { ui.button("Expensive to build"); });
    grid.item(MudItem::star(1.0), |ui| { ui.label("Fills the rest"); });
});
```

---

## Layout algorithm

For a grid `W` pixels wide with gap `g` (`spacing * 4`):

1. **Track width.** `t = max(0, (W - 11g) / 12)`. A cell spanning `n` tracks is
   `n*t + (n-1)*g` wide, so twelve tracks plus eleven gaps fill `W` exactly.
2. **Resolve spans.** Each item resolves its span at the active breakpoint (below), walking
   *down* the ladder to the first set value; nothing set means 12 tracks.
3. **Pack rows.** Items fill a row until the next would exceed 12 tracks. `Star` and `Auto`
   reserve one track each. Every row holds at least one item.
4. **Size each row.** `Tracks` and `Auto` cells are fixed first. The remainder
   (`W` minus fixed widths minus gaps, never negative) is split among `Star` cells by weight,
   or equally when every weight is zero.
5. **Justify each row.** Leftover space moves cells according to `Justify`
   (`Center` = half, `End` = all, `SpaceBetween` = spread between, `SpaceAround` /
   `SpaceEvenly` as in CSS flexbox).
6. **Draw.** Cells are child `Ui`s with unbounded height; a row is as tall as its tallest
   cell, and rows are separated by `g`.

The active breakpoint comes from the width **available to the grid**, not the OS window, so
a grid inside a narrow side panel behaves as a narrow grid. A non-finite available width
(e.g. inside a horizontal scroll area) falls back to 1280px.

---

## Performance and memory layout

| Choice | Effect |
|--------|--------|
| `MudGridSpan` is `Copy`, 8 bytes (`u8`/`f32` payload plus tag) | Passed by value, no refcounts |
| `MudItem` is `Copy`, six spans = 48 bytes | Cheap to build every frame |
| Cascade resolution walks a fixed `[MudGridSpan; 6]` on the stack | Resolving a span never allocates |
| Layout math writes into caller-owned `Vec`s (`*_into` functions) | No per-row allocation |
| `MudGrid` keeps a private scratch arena (spans, auto widths, measured widths, rows, widths, offsets, cell boxes, rectangles) reused every frame | Steady-state frames do not re-grow those buffers |
| Pure layout core with no egui types | Unit-testable and usable headless |
| `AutoMeasure::Cached` | `Auto` closures run once per frame instead of twice |

**What still allocates per frame.** The item list is rebuilt every frame because immediate-mode
closures cannot be retained across frames: that is one `Box` per item, the `Vec` holding them,
and the `Vec` of resolved item frames. Styled items additionally build a small
`ComponentBase` while their frame is resolved. Everything else is reused.

**About `Cached`.** In this mode a cell is probed only when no previous width is stored. After
each frame the real widths are compared with the cache; a change larger than half a pixel is
stored and triggers one repaint, so content that resizes lags by at most one frame. Cached
widths live in egui's temp data keyed by the grid's `Ui` id. `Probe` remains the default
because it is always exact.

**Threads.** egui is single-threaded immediate mode and `MudGrid::show` takes `&mut self`, so
no internal locking exists or is needed. `MudItem`, `MudGridSpan` and `CellBox` are plain
`Copy` data.

These figures describe structure, not measurements: no benchmarks were run.

---

## Design rationale

- **Gaps instead of negative margins.** Current MudBlazor implements spacing with CSS `gap`
  (formerly negative margins plus padding). Gaps are the natural model for egui too: cells get
  exact rectangles and nothing overflows the container.
- **Spans as an enum, not a number.** `Tracks`, `Star` and `Auto` are different sizing
  disciplines. An enum makes illegal states unrepresentable and keeps the layout `match`
  exhaustive.
- **Never panic in a UI path.** Out-of-range integers clamp, bad strings become `Unset`, NaN
  and infinite weights become `0.0`, non-finite widths fall back to 1280px, non-finite heights
  count as `0`. Strict parsing (`FromStr`) is available when you want errors.
- **Lenient builders, strict parsers.** `item.md("garbage")` does not stop a frame from
  drawing; `"garbage".parse::<MudGridSpan>()` tells you.
- **Plan, then draw.** Separating planning from drawing is what allows headless tests,
  `cell_rects()`, and a single pass over the pure math per frame.
- **Container breakpoints.** Panels, docks and split views are common in egui, so the
  default follows the grid's own width. `breakpoint(..)` pins it when you want
  window-style behaviour.
- **No invented MudBlazor API.** See the first section.

---

## Behaviour changes from the previous version

- **`MAX_SPACING` is now 16** (was 20), matching MudBlazor's documented maximum. A grid
  asking for `spacing(20)` is now clamped to 16 (64px).
- **`disabled` is no longer reset every frame.** The old code wrote `disabled = false` at the
  end of each `show`, so the flag could never take effect. It now persists until changed.
- **The grid's response rectangle ends at `min.x + width`**, not the raw available right edge,
  so an unbounded parent no longer produces an infinite rectangle.
- **`GridUi::item` accepts `impl Into<MudItem>`.** Existing calls passing a `MudItem` compile
  unchanged.
- **`ladder_rank` is `pub(crate)`** rather than private, and `ParseSpanError`, `MudGridSpan`
  and `MudItem` keep their existing behaviour and tests.
- **New integer conversions** (`i8`, `i16`, `i64`, `isize`, `u64`) and `String` /
  `&String` conversions on `MudGridSpan`.

---

## Testing

Each file ends with a `#[cfg(test)] mod tests` block.

| File | Covers |
|------|--------|
| `mud_grid_span.rs` | defaults, presets, `fixed`, `or`, `track_need`, normalization (idempotent, never `Unset`, NaN), parsing (valid, garbage, overflow, mixed), `Display` round-trips, every integer and string conversion |
| `mud_item.rs` | cascade rules, normalization, every setter, `all`, `at`, `get`, ladder round-trip, presets, `resolve_all` agreeing with `resolve_span`, `Display` |
| `mud_grid.rs` | breakpoint edges, track and gap math, row packing, star/auto sizing, degenerate inputs, justification, buffer reuse, the planner (`plan_into`, `MudGrid::plan`) across breakpoints, justify, spacing, fallback width and invariants (no overlap, no overflow, in-order cells), `GridUi` / `ItemStyle` builders, `MudGrid` builders and defaults, interaction state |

```text
cargo test
```

The planner tests are the "visual layout" tests: they assert exact x positions and widths
for each breakpoint and justify mode. Drawing itself (`MudGrid::show`) needs a live egui
context and is not unit-tested.

===== ./crates/mud_egui_components/src/layout/hidden/README.md =====

# MudHidden for `mud_egui`

A visibility gate for [egui](https://github.com/emilk/egui): renders its content
only when the available width falls inside (or, with `invert`, outside) a
breakpoint. Ported from MudBlazor's
[`MudHidden`](https://mudblazor.com/components/hidden) (MudBlazor 8.13.0).

```rust
// Show the wide layout at Md and above.
MudHidden::new()
.breakpoint(Breakpoint::MdAndUp)
.show(ui, |ui| {
ui.add(MudText::body1("Wide layout"));
});

// Show the compact layout below Md.
MudHidden::new()
.breakpoint(Breakpoint::MdAndUp)
.invert(true)
.show(ui, |ui| {
ui.add(MudText::body1("Compact layout"));
});
```

The content closure supplies every string and glyph; `MudHidden` paints none of
its own. Route them through **`MudText`** and **`MudIcon`** as usual.

---

## Contents

1. [Design principle: API parity, not API invention](#design-principle-api-parity-not-api-invention)
2. [Architecture](#architecture)
3. [API reference](#api-reference)
4. [Examples](#examples)
5. [Visibility pipeline](#visibility-pipeline)
6. [Performance and memory choices](#performance-and-memory-choices)
7. [Testing](#testing)
8. [Differences from MudBlazor](#differences-from-mudblazor)

---

## Design principle: API parity, not API invention

The MudBlazor 8.13.0 parameter surface was verified against DNDocs before
implementation. `MudHidden` has **five parameters**, and all five are ported.

### Ported parameters

| MudBlazor | Type | Default | Rust mapping |
|---|---|---|---|
| `Breakpoint` | `Breakpoint` | `None` | `breakpoint(Breakpoint)` |
| `Invert` | `bool` | `false` | `invert(bool)` |
| `ChildContent` | `RenderFragment?` | `null` | closure to `show(ui, \|ui\| ..)` |
| `Hidden` | `bool` | **`true`** | `HiddenResponse::hidden` |
| `HiddenChanged` | `EventCallback<bool>` | — | `HiddenResponse::changed` |
| `Class` | `string?` | `null` | `classes(iter)` — stored, not resolved |
| `Tag` | `object?` | `null` | `tag(t)` / `tag_ref::<U>()` |

### Deliberately not ported

| MudBlazor | Reason |
|---|---|
| `Style` | egui has no inline CSS strings. `MudHidden` renders no element of its own, so there is nothing to style. |
| `UserAttributes` | There is no HTML tag to splat attributes onto. |
| `Class` resolution | Stored on `base` for parity. `MudHidden` adds no wrapper frame, so a class list has no surface to paint; the child content's own classes apply instead. |
| `CurrentBreakpointFromProvider` | A cascading parameter. egui has no cascading-parameter system; the port reads `ui.available_width()` (see [Differences from MudBlazor](#differences-from-mudblazor)). |

### Extensions

| Extension | Type | Default | Purpose |
|---|---|---|---|
| `id_salt(impl Hash + Debug)` | `Id` | `None` | Distinguishes two `MudHidden`s in one parent, whose `HiddenChanged` edges would otherwise share a memory key. Same shape as `MudCollapse::id_salt`. |

### `Hidden` is an output, not a binding

`Hidden` is a two-way bindable parameter in Blazor, but the component **computes**
it from `(Breakpoint, Invert, width)` every frame — the caller never controls
it. The port therefore exposes it as `HiddenResponse::hidden`, the same call
`MudCollapse` makes for `expanded_changed`.

### `Breakpoint` gained nine variants

The 8.13.0 `Breakpoint` enum has **sixteen** variants, not the seven the
workspace previously had: six simple buckets (`Xs`..`Xxl`), eight compound ranges
(`SmAndDown`..`XlAndUp`), and two sentinels (`None`, `Always`). The port extends
`mud_egui_core::enums::Breakpoint` to match — the extension is purely additive,
and every existing `to_pixels()` / `>=` use of the enum keeps working unchanged.

The distinction the compound variants exist for is:

| Variant | Matches |
|---|---|
| `Lg` | the `[1280, 1920)` bucket only |
| `LgAndUp` | `width >= 1280` |
| `LgAndDown` | `width < 1920` |

So `Breakpoint::Lg` hides only when the viewport is a desktop but not a
4K monitor; `Breakpoint::LgAndUp` hides on desktop and larger. Use the compound
variants for the common "hide at and above" / "hide at and below" cases.

---

## Architecture

```
layout/hidden/
├── mud_hidden.rs
│   ├── MudHidden          builder + show(ui, add_content) + Widget impl
│   ├── HiddenResponse<R>  response + hidden + changed + inner
│   └── compute_hidden     pure visibility rule
└── mod.rs                 re-exports

mud_egui_core/src/enums/
└── breakpoint.rs          sixteen variants + matches / to_pixels / from_pixels
```

**Pure layer.** `compute_hidden(breakpoint, invert, width)` is the whole
decision. `Breakpoint::matches(width)` is the range test it delegates to. Both
are unit-testable without an `egui::Context`.

**State layer.** One `bool` per instance lives in `egui::Memory`, keyed by
`<ui.id().with(id_salt)>.mud_hidden_prev`. It exists only so `HiddenChanged` can
fire once per flip rather than every frame.

**Component layer.** `show` reads the available width, computes the hidden
state, updates the memory, and either allocates a zero-sized response (hidden)
or runs the content closure in a `ui.scope` (visible). The closure is **never**
called while hidden — that is the component's whole purpose.

---

## API reference

### `MudHidden`

| Method | Type | Default | Mirrors | Description |
|---|---|---|---|---|
| `new()` | `Self` | — | — | No breakpoint, not inverted. |
| `breakpoint(b)` | `Breakpoint` | `None` | `Breakpoint` | The range to test against. |
| `invert(b)` | `bool` | `false` | `Invert` | Hide when the range matches instead of showing. |
| `id_salt(s)` | `impl Hash + Debug` | `None` | *(extension)* | Distinguish two instances in one parent. |
| `classes(iter)` | `iter of AsRef<str>` | none | `Class` | Stored on `ComponentBase`. |
| `tag(t)` / `tag_ref::<U>()` | `T: Any + Send + Sync` | `None` | `Tag` | Arbitrary user data. |
| `show(ui, \|ui\| ..)` | `&mut Ui, FnOnce(&mut Ui) -> R` | — | `ChildContent` | Returns `HiddenResponse<R>`. |
| `ui.add(..)` | — | — | — | Evaluates the rule but draws no content. |

### `HiddenResponse<R>`

| Field | Type | Description |
|---|---|---|
| `response` | `Response` | The content's response when visible; a zero-sized hover response when hidden. Deref target. |
| `hidden` | `bool` | The computed hidden state. Mirrors `Hidden`. |
| `changed` | `bool` | `true` on the frame the hidden state flipped. Mirrors `HiddenChanged`; never set on the first frame. |
| `inner` | `Option<R>` | The content closure's value; `None` when hidden. |

`HiddenResponse` implements `Deref<Target = Response>`, so `out.hovered()`,
`out.rect` and friends work directly.

### `Breakpoint`

| Variant | Matches (width, points) | Doc label |
|---|---|---|
| `Xs` | `< 600` | A small to large phone |
| `Sm` | `[600, 960)` | A small to medium tablet |
| `Md` | `[960, 1280)` | A large tablet or laptop |
| `Lg` | `[1280, 1920)` | A desktop computer |
| `Xl` | `[1920, 2560)` | A high-definition or 4K desktop monitor |
| `Xxl` | `>= 2560` | An ultra-wide 4K+ desktop monitor |
| `SmAndDown` | `< 960` | Sm or smaller |
| `MdAndDown` | `< 1280` | Md or smaller |
| `LgAndDown` | `< 1920` | Lg or smaller |
| `XlAndDown` | `< 2560` | Xl or smaller |
| `SmAndUp` | `>= 600` | Sm or larger |
| `MdAndUp` | `>= 960` | Md or larger |
| `LgAndUp` | `>= 1280` | Lg or larger |
| `XlAndUp` | `>= 1920` | Xl or larger |
| `None` | never | No breakpoint applies |
| `Always` | always | Content will always be visible |

Helpers: `ALL`, `name()`, `label()`, `is_sentinel()`, `is_compound()`,
`to_pixels()`, `matches(width)`, `from_pixels(width)`, `Display`.

### Constants

| Name | Value | Meaning |
|---|---|---|
| `SM_PX` | `600.0` | Smallest `Sm` width |
| `MD_PX` | `960.0` | Smallest `Md` width |
| `LG_PX` | `1280.0` | Smallest `Lg` width |
| `XL_PX` | `1920.0` | Smallest `Xl` width |
| `XXL_PX` | `2560.0` | Smallest `Xxl` width |

---

## Examples

### Show only at Md and above

```rust
MudHidden::new()
.breakpoint(Breakpoint::MdAndUp)
.show(ui, |ui| {
ui.add(MudText::body1("Sidebar"));
});
```

### Show only below Md (the inverse)

```rust
MudHidden::new()
.breakpoint(Breakpoint::MdAndUp)
.invert(true)
.show(ui, |ui| {
ui.add(MudText::body1("Hamburger menu"));
});
```

### Show only in the Md bucket (not and-up)

```rust
MudHidden::new()
.breakpoint(Breakpoint::Md)
.show(ui, |ui| {
ui.add(MudText::body2("Tablet-only panel"));
});
```

`Breakpoint::Md` matches only `[960, 1280)`; use `MdAndUp` to show at Md and
larger.

### Show only on phones (up to Sm)

```rust
MudHidden::new()
.breakpoint(Breakpoint::SmAndDown)
.invert(true)
.show(ui, |ui| {
ui.add(MudText::body1("Mobile layout"));
});
```

`SmAndDown` matches `< 960`; inverting makes it hide below 960 and show at
960 and above. To show only at `< 960`, drop the `.invert(true)`.

### Hide at Lg and larger

```rust
MudHidden::new()
.breakpoint(Breakpoint::LgAndUp)
.show(ui, |ui| {
ui.add(MudText::body2("Visible on tablets and phones"));
});
```

At `Lg`, `Xl` and `Xxl`, `LgAndUp` matches and the content is hidden; below
`Lg`, it does not match and the content is visible.

### Observing `HiddenChanged`

```rust
let out = MudHidden::new()
.breakpoint(Breakpoint::MdAndUp)
.show(ui, |ui| {
ui.add(MudText::body1("Content"));
});

if out.changed {
log::info!("hidden = {}", out.hidden);
}
```

`changed` is `true` for exactly one frame per flip; the previous value is kept
in `egui::Memory` keyed by the widget id.

### Two gates in one parent

```rust
MudHidden::new()
.id_salt("wide")
.breakpoint(Breakpoint::LgAndUp)
.show(ui, |ui| { /* … */ });

MudHidden::new()
.id_salt("narrow")
.breakpoint(Breakpoint::SmAndDown)
.show(ui, |ui| { /* … */ });
```

Without distinct salts the two share a memory key and their `changed` flags
would cross.

### `Always` (never hidden)

```rust
MudHidden::new()
.breakpoint(Breakpoint::Always)
.show(ui, |ui| {
ui.add(MudText::body2("Always visible"));
});
```

`Always` is a sentinel: it is never hidden, even with `invert(true)`.

---

## Visibility pipeline

```
available_width ──► Breakpoint::matches(width) ──► XOR invert ──► hidden?
                                                                 │
                          ┌──────────────────────────────────────┴──────────┐
                          ▼                                                 ▼
                     hidden == true                                    hidden == false
                     allocate Vec2::ZERO                               ui.scope(add_content)
                     Sense::hover()                                    Response = content's
                     inner = None                                      inner = Some(R)
```

`Always` short-circuits to `hidden = false` regardless of `invert` — that is the
variant's documented meaning, "Content will always be visible".

`None` matches nothing, so with the default `invert = false` the content is
visible. That is the least surprising reading of "no breakpoint specified".

### Per-frame cost

| Path | Work |
|---|---|
| Hidden | One `available_width()` read, one `matches` call, one memory read, one `insert_temp` of a `bool`, one zero-sized allocation. The content closure is **not called**. |
| Visible | The above, plus a `ui.scope` and the closure. |

No `Vec`, no `String`, no theme lookup.

---

## Performance and memory choices

| Choice | Effect |
|---|---|
| Content closure is `FnOnce` | The closure runs **at most once**, and never when hidden. This is the whole point of the component. |
| `Breakpoint::matches` is pure | One match arm and a half-open range check; no allocation. |
| One `bool` in `egui::Memory` | The only persisted state; read and written once per frame. |
| Zero-sized hover response when hidden | Takes no layout space and steals no clicks. |
| No theme fetch | `MudHidden` resolves no colours; the child content owns its own theming. |
| `id_salt` for disambiguation | Two instances in one parent do not share their `changed` edge. |

---

## Testing

`cargo test -p mud_egui_components mud_hidden` runs 15 pure-function tests:

| Area | Tests |
|---|---|
| Visibility rule | `None` with/without invert; `Always` never hidden even inverted; simple bucket hide-inside vs show-inside; `*AndDown` and `*AndUp` both ways; `Xs` and `Xxl` boundary cases |
| Builders | Defaults match MudBlazor; every field set; `id_salt` stability |

Plus 21 tests in `mud_egui_core` for `Breakpoint`:

| Area | Tests |
|---|---|
| `ALL` / `name` / `Display` | Sixteen unique variants; lowercase names; `Display` matches `name` |
| Classification | `is_sentinel`, `is_compound` |
| `to_pixels` | Simple lower bounds; `*AndUp` lower bounds; `*AndDown` upper bounds; sentinels are `0.0` |
| `matches` | Sentinels; every simple bucket at its boundaries; every compound at its boundaries; non-finite width rejected except by `Always`; negative width treated as zero |
| `from_pixels` | Round-trip at each lower bound; non-finite repair |

No test creates an `egui::Context`. `MudHidden::show` needs a live `Ui` (for
`available_width`, the scope, and memory) and is therefore left to integration
tests; everything it delegates to is covered above.

Suggested integration tests (`egui_kittest` or a headless `Context::run`):

- A `MudHidden` with `Breakpoint::MdAndUp` draws its content at 1280 pt and
  draws nothing at 599 pt.
- The content closure is not called when hidden (use a counter in a `Cell`).
- `invert(true)` swaps the two cases.
- `changed` is `true` for exactly one frame after a resize crosses the
  boundary, and `false` on steady frames.
- Two `MudHidden`s in one parent with distinct `id_salt`s track their own
  `changed` edges independently.
- `Breakpoint::Always` draws its content at every width, even with `invert(true)`.

```bash
cargo test -p mud_egui_core breakpoint
cargo test -p mud_egui_components mud_hidden
```

---

## Differences from MudBlazor

- **The width comes from `ui.available_width()`, not the browser window.**
  MudBlazor uses JavaScript (`IBrowserViewportService`) to observe the browser
  window's size. egui has no browser window, so the port reads the **container**
  width instead. That is the immediate-mode equivalent and is arguably more
  correct inside a split panel or a narrow side column, where the available
  width is what actually matters.
- **`Hidden` is an output, not a two-way binding.** The component computes it
  from `(Breakpoint, Invert, width)` every frame; the port returns it on
  `HiddenResponse::hidden` instead of taking a `&mut bool`. This is the same
  split `MudCollapse` uses for `expanded_changed`.
- **`HiddenChanged` is a response field, not a callback.** `changed` is `true`
  for exactly one frame per flip, keyed in `egui::Memory`. The initial frame
  never reports `changed` (there is no previous value to compare against),
  which is the same contract `MudCollapse::expanded_changed` and
  `MudRating::hovered_changed` use.
- **`Class` is stored but not resolved.** `MudHidden` renders no wrapper
  element, so a class list has no surface to paint. `classes(..)` is kept for
  parity; the child content's own classes apply instead.
- **`Style` and `UserAttributes` are not ported.** No inline CSS strings, no
  HTML tag.
- **`CurrentBreakpointFromProvider` is not ported.** It is a cascading
  parameter; egui has no cascading-parameter system. The port reads the
  available width directly, which is equivalent for the only thing the
  parameter is used for.
- **`Breakpoint` gained nine variants.** The 8.13.0 enum has sixteen, not
  seven; the port extends `mud_egui_core::enums::Breakpoint` to match. The
  extension is additive — existing `to_pixels()` / `>=` comparisons keep
  working — but it is a change to a shared enum, not just to `MudHidden`.
  `Breakpoint::Lg` and `Breakpoint::LgAndUp` are now genuinely different (the
  bucket versus the range), whereas a pre-8.13 port that only had seven
  variants had to treat `Lg` as "and up".
- **`Always` is never hidden, even with `invert(true)`.** That is the variant's
  documented meaning ("Content will always be visible"). A naive
  `matches(width) != invert` would hide it when inverted; the port special-cases
  it instead.
- **`None` shows the content by default.** `matches(None, _)` is `false`, so
  `hidden = false != false = false` — visible. MudBlazor's `Hidden` field
  defaults to `true`, but the field is overwritten by the first
  `UpdateAsync`; the steady-state behaviour is what the port matches, and
  "no breakpoint specified → show the content" is the least surprising reading.

===== ./crates/mud_egui_components/src/layout/layout_shell/README.md =====

# MudLayout + MudAppBar + MudDrawer for `mud_egui`

The application shell for [egui](https://github.com/emilk/egui), ported from
MudBlazor's [`MudLayout`](https://mudblazor.com/components/layout),
[`MudAppBar`](https://mudblazor.com/components/appbar) and
[`MudDrawer`](https://mudblazor.com/components/drawer) (MudBlazor 8.13.0).

```rust
struct App {
    layout: LayoutState,
    drawer: DrawerState,
}

MudLayout::new().show(ui, &mut self.layout, |ui, layout| {
    MudAppBar::new().color(Color::Primary).show(ui, layout, |ui| {
        ui.add(MudText::new("My App").typo(Typo::H6));
    });
    ui.horizontal(|ui| {
        MudDrawer::new()
            .variant(DrawerVariant::Responsive)
            .show(ui, &mut self.drawer, layout, |ui| {
                ui.add(MudText::new("Navigation"));
            });
        MudMainContent::new().show(ui, layout, |ui| {
            ui.add(MudText::new("Page body"));
        });
    });
});
```

The shell paints only its own surfaces; every string inside is yours to render
through **`MudText`**, every glyph through **`MudIcon`**.

---

## Contents

1. [Design principle: API parity, not API invention](#design-principle-api-parity-not-api-invention)
2. [Architecture](#architecture)
3. [API reference — MudLayout](#api-reference--mudlayout)
4. [API reference — MudAppBar](#api-reference--mudappbar)
5. [API reference — MudDrawer](#api-reference--muddrawer)
6. [Examples](#examples)
7. [Layout and interaction](#layout-and-interaction)
8. [Performance and memory choices](#performance-and-memory-choices)
9. [Testing](#testing)
10. [Differences from MudBlazor](#differences-from-mudblazor)

---

## Design principle: API parity, not API invention

The MudBlazor 8.13.0 parameter surfaces were verified against DNDocs before
implementation. `MudLayout` has **no own parameters** beyond `ChildContent`;
`MudAppBar` has **ten**; `MudDrawer` has **fifteen**.

### `MudLayout`

| MudBlazor | Rust mapping | Default |
|---|---|---|
| `ChildContent` | `body` argument of `MudLayout::show` | required |
| `Class` (base) | `classes(iter)` — stored | `None` |
| `Style` / `Tag` / `UserAttributes` | **not ported** | — |

### `MudAppBar`

| MudBlazor | Rust mapping | Default |
|---|---|---|
| `Bottom` | `bottom(bool)` | `false` |
| `ChildContent` | `content` argument of `show` | required |
| `Color` | `color(Color)` | `Default` |
| `Dense` | `dense(bool)` | `false` |
| `Elevation` | `elevation(u8)` | `4` |
| `Fixed` | `fixed(bool)` | `true` — **stored** |
| `Gutters` | `gutters(bool)` | `true` |
| `ToolBarClass` | `toolbar_class(s)` — **stored** | `None` |
| `WrapContent` | `wrap_content(bool)` | `false` |
| `Contextual` | **not in 8.13.0** — 8.14+ addition | — |
| `Class` (base) | `classes(iter)` — stored | `None` |
| `Style` / `Tag` / `UserAttributes` | **not ported** | — |

### `MudDrawer`

| MudBlazor | Rust mapping | Default |
|---|---|---|
| `Anchor` | `anchor(AnchorOrigin)` | `Start` |
| `Breakpoint` | `breakpoint(Breakpoint)` | `Md` |
| `ChildContent` | `content` argument of `show` | required |
| `ClipMode` | `clip_mode(DrawerClipMode)` | `Never` |
| `Color` | `color(Color)` | `Default` |
| `Elevation` | `elevation(u8)` | `1` |
| `Fixed` | `fixed(bool)` | `true` — **stored** |
| `Height` | `height(f32)` | `None` → `200.0` |
| `MiniWidth` | `mini_width(f32)` | `None` → `56.0` |
| `Open` | `DrawerState::open` | `false` |
| `OpenChanged` | `DrawerResponse::opened` / `closed` | — |
| `OpenMiniOnHover` | `open_mini_on_hover(bool)` | `false` |
| `Overlay` | `overlay(bool)` | `true` |
| `OverlayAutoClose` | `overlay_auto_close(bool)` | `true` |
| `Variant` | `variant(DrawerVariant)` | `Responsive` |
| `Width` | `width(f32)` | `None` → `240.0` |
| `Class` (base) | `classes(iter)` — stored | `None` |
| `Style` / `Tag` / `UserAttributes` | **not ported** | — |

### `MudDrawerHeader`

| MudBlazor | Rust mapping | Default |
|---|---|---|
| `ChildContent` | `content` argument of `show` | required |
| `Dense` | `dense(bool)` | `false` |
| `Class` (base) | `classes(iter)` — stored | `None` |

### `MudMainContent`

| MudBlazor | Rust mapping | Default |
|---|---|---|
| `ChildContent` | `content` argument of `show` | required |
| `Class` (base) | `classes(iter)` — stored | `None` |

### Enums introduced

Two public enums, each in its own file (Rule 2):

| Enum | File | Discriminants |
|---|---|---|
| `DrawerVariant` | `mud_egui_core::enums::drawer_variant` | `Temporary = 0`, `Responsive = 1`, `Persistent = 2`, `Mini = 3` |
| `DrawerClipMode` | `mud_egui_core::enums::drawer_clip_mode` | `Never = 0`, `Docked = 1`, `Always = 2` |

`AnchorOrigin` (already in `mud_egui_core::enums`) is MudBlazor's `Anchor` —
the popover README documents the rename. `Breakpoint` and `Color` are reused.

### Deliberately not ported

| MudBlazor | Reason |
|---|---|
| `Style` / `Tag` / `UserAttributes` (all four components) | No CSS strings, no HTML tag. |
| `ToolBarClass` resolution | Stored for parity. The bar owns its own fill, height and gutters. |
| `Contextual` on `MudAppBar` | 8.14+ addition (PR #10210). Not in 8.13.0. |
| `Fixed` behaviour | Stored. egui has no free-floating chrome inside a `ScrollArea`; scroll-away behaviour is the caller's `ScrollArea` decision. |

---

## Architecture

```
layout/
├── app_bar/
│   ├── app_bar_response.rs   AppBarResponse
│   ├── mud_app_bar.rs        MudAppBar + pure helpers + tests
│   └── mod.rs
├── drawer/
│   ├── drawer_response.rs    DrawerResponse
│   ├── drawer_state.rs       DrawerState (caller-owned)
│   ├── effective_variant.rs  pub(crate) EffectiveVariant + resolve (Rule 2)
│   ├── mud_drawer.rs         MudDrawer + Side + resolve_side + side_rect + tests
│   ├── mud_drawer_header.rs  MudDrawerHeader
│   └── mod.rs
├── layout_shell/
│   ├── layout_state.rs       LayoutState
│   ├── mud_layout.rs         MudLayout
│   ├── mud_main_content.rs   MudMainContent + compute_main_rect
│   └── mod.rs
└── mod.rs

mud_egui_core/src/enums/
├── drawer_clip_mode.rs       DrawerClipMode
└── drawer_variant.rs         DrawerVariant
```

**Pure layer.** `compute_main_rect`, `resolve_side`, `side_rect`,
`resolve` (effective variant), `is_light`, `LayoutState::top_inset` /
`bottom_inset`, and every `_state` mutator are free functions or methods over
`Rect`, `f32`, enums. None touch `egui::Context`.

**State layer.** `LayoutState` is reset by `MudLayout::show`; `MudAppBar` and
`MudDrawer` publish into it; `MudMainContent` reads it. `DrawerState` is the
caller-owned `Open` binding.

**Component layer.** `MudLayout` hands the caller a full-screen `Ui`; the
caller composes `MudAppBar`, `MudDrawer` and `MudMainContent` inside it.

---

## API reference — `MudLayout`

| Method | Type | Description |
|---|---|---|
| `new()` | — | Default layout. |
| `classes(iter)` | `iter of AsRef<str>` | Stored. |
| `show(ui, state, \|ui, layout\| ..)` | `(&mut Ui, &mut LayoutState, FnOnce(&mut Ui, &mut LayoutState) -> R) -> R` | Resets `state` and hands the caller a screen-sized `Ui`. |

---

## API reference — `MudAppBar`

| Method | Type | Default | Description |
|---|---|---|---|
| `new()` | — | — | Default app bar. |
| `bottom(bool)` | `bool` | `false` | Anchor to the bottom edge. |
| `color(Color)` | `Color` | `Default` | Surface fill. |
| `dense(bool)` | `bool` | `false` | 48 pt instead of 64. |
| `elevation(u8)` | `u8` | `4` | Shadow depth, clamped to 24. |
| `fixed(bool)` | `bool` | `true` | Stored. |
| `gutters(bool)` | `bool` | `true` | 16 pt side padding. |
| `toolbar_class(s)` | `impl Into<String>` | `None` | Stored. |
| `wrap_content(bool)` | `bool` | `false` | Wrap onto more rows. |
| `classes(iter)` | — | — | Stored. |
| `show(ui, layout, \|ui\| ..)` | `(&mut Ui, &mut LayoutState, FnOnce(&mut Ui) -> R)` | — | Returns `InnerResponse<AppBarResponse>`. |

### `AppBarResponse`

| Field | Type | Description |
|---|---|---|
| `response` | `Response` | The bar's own response. |
| `height` | `f32` | 64 normally, 48 when `dense`. |

### Constants

| Name | Value |
|---|---|
| `APP_BAR_HEIGHT` | `64.0` |
| `DENSE_APP_BAR_HEIGHT` | `48.0` |
| `APP_BAR_GUTTER` | `16.0` |
| `APP_BAR_GAP` | `8.0` |

---

## API reference — `MudDrawer`

| Method | Type | Default | Description |
|---|---|---|---|
| `new()` | — | — | Default drawer. |
| `anchor(AnchorOrigin)` | `AnchorOrigin` | `Start` | Edge. |
| `breakpoint(Breakpoint)` | `Breakpoint` | `Md` | Responsive threshold. |
| `clip_mode(DrawerClipMode)` | `DrawerClipMode` | `Never` | Z-order vs app bar. |
| `color(Color)` | `Color` | `Default` | Surface fill. |
| `elevation(u8)` | `u8` | `1` | Shadow depth. |
| `fixed(bool)` | `bool` | `true` | Stored. |
| `height(f32)` | `f32` | `None` → `200` | Top/bottom extent. |
| `mini_width(f32)` | `f32` | `None` → `56` | Collapsed mini extent. |
| `open_mini_on_hover(bool)` | `bool` | `false` | Expand a mini drawer on hover. |
| `overlay(bool)` | `bool` | `true` | Backdrop for overlay variants. |
| `overlay_auto_close(bool)` | `bool` | `true` | Backdrop click closes. |
| `variant(DrawerVariant)` | `DrawerVariant` | `Responsive` | Behaviour. |
| `width(f32)` | `f32` | `None` → `240` | Left/right extent. |
| `right_to_left(bool)` | `bool` | `false` | Resolve `Start` / `End`. |
| `classes(iter)` | — | — | Stored. |
| `show(ui, state, layout, \|ui\| ..)` | `(&mut Ui, &mut DrawerState, &mut LayoutState, FnOnce(&mut Ui) -> R)` | — | Returns `DrawerResponse`. |

### `DrawerState`

| Field / method | Description |
|---|---|
| `open: bool` | Whether the drawer is open. |
| `hover_expanded: bool` | Mini expansion latch. |
| `new()` / `opened()` | Constructors. |
| `is_open()` / `open()` / `close()` / `toggle()` | Mutators. |

### `DrawerResponse`

| Field | Type | Description |
|---|---|---|
| `response` | `Response` | The panel's response. |
| `opened` | `bool` | `true` on the opening frame. |
| `closed` | `bool` | `true` on the closing frame. |
| `visible` | `bool` | Whether anything painted. |
| `reserved_extent` | `f32` | Width / height reserved. `0` for overlay. |

### Effective variant

| Config | Below breakpoint | At / above breakpoint |
|---|---|---|
| `Temporary` | Overlay | Overlay |
| `Persistent` | Push | Push |
| `Responsive` | Overlay | Push |
| `Mini` | Overlay | Mini |

### Constants

| Name | Value |
|---|---|
| `DEFAULT_WIDTH` | `240.0` |
| `DEFAULT_MINI_WIDTH` | `56.0` |
| `DEFAULT_HEIGHT` | `200.0` |
| `DEFAULT_MINI_HEIGHT` | `40.0` |

---

## Examples

### Minimal shell

```rust
let mut layout = LayoutState::default();
let mut drawer = DrawerState::new();

MudLayout::new().show(ui, &mut layout, |ui, layout| {
    MudAppBar::new().show(ui, layout, |ui| {
        ui.add(MudText::new("My App"));
    });
    ui.horizontal(|ui| {
        MudDrawer::new().show(ui, &mut drawer, layout, |ui| {
            ui.add(MudText::new("Nav"));
        });
        MudMainContent::new().show(ui, layout, |ui| {
            ui.add(MudText::new("Body"));
        });
    });
});
```

### Persistent drawer with a toggle button

```rust
MudAppBar::new().show(ui, layout, |ui| {
    if ui.button("☰").clicked() { drawer.toggle(); }
    ui.add(MudText::new("App"));
});

ui.horizontal(|ui| {
    MudDrawer::new()
        .variant(DrawerVariant::Persistent)
        .width(280.0)
        .show(ui, &mut drawer, layout, |ui| {
            MudDrawerHeader::new().show(ui, |ui| {
                ui.add(MudText::new("Navigation").typo(Typo::H6));
            });
            ui.add(MudText::new("Menu"));
        });
    MudMainContent::new().show(ui, layout, |ui| { /* body */ });
});
```

### Temporary drawer with an overlay

```rust
MudDrawer::new()
    .variant(DrawerVariant::Temporary)
    .overlay(true)
    .overlay_auto_close(true)
    .show(ui, &mut drawer, layout, |ui| { /* nav */ });
```

### Mini drawer that expands on hover

```rust
MudDrawer::new()
    .variant(DrawerVariant::Mini)
    .mini_width(56.0)
    .width(240.0)
    .open_mini_on_hover(true)
    .show(ui, &mut drawer, layout, |ui| { /* nav */ });
```

### Bottom app bar

```rust
MudAppBar::new().bottom(true).show(ui, layout, |ui| {
    ui.add(MudText::new("Status bar"));
});
```

### Drawer above the app bar (`ClipMode::Never`)

```rust
MudLayout::new().show(ui, &mut layout, |ui, layout| {
    ui.horizontal(|ui| {
        drawer_show.show(ui, &mut drawer, layout, |ui| { /* nav */ });
        ui.vertical(|ui| {
            MudAppBar::new().show(ui, layout, |ui| {
                ui.add(MudText::new("Title"));
            });
            MudMainContent::new().show(ui, layout, |ui| { /* body */ });
        });
    });
});
```

### Docked drawer (`ClipMode::Docked`)

```rust
MudDrawer::new()
    .clip_mode(DrawerClipMode::Docked)
    .show(ui, &mut drawer, layout, |ui| { /* nav */ });
```

The app bar is declared first; the drawer starts below it.

---

## Layout and interaction

### Structure

```
DrawerClipMode::Docked / Always     DrawerClipMode::Never
┌──────────────────────────────┐   ┌───────┬──────────────────────┐
│ AppBar                        │   │       │ AppBar                │
├───────┬──────────────────────┤   │ Drawer├──────────────────────┤
│Drawer │ MainContent           │   │       │ MainContent           │
│       │                       │   │       │                       │
└───────┴──────────────────────┘   └───────┴──────────────────────┘
```

The nesting is the caller's choice; the diagram above is the canonical shape.

### Sizing

| Component | Extent |
|---|---|
| `MudAppBar` | `available_width × 64` (48 dense) |
| `MudDrawer` (left/right) | `width` (default 240) × `available_height` |
| `MudDrawer` (top/bottom) | `available_width` × `height` (default 200) |
| `MudDrawer` mini (collapsed) | `mini_width` (default 56) × `available_height` |
| `MudMainContent` | `available_rect_before_wrap` minus the insets in `LayoutState` |

### Interaction

| Input | Effect |
|---|---|
| Backdrop click (overlay variant, `overlay_auto_close`) | `state.close()` |
| Pointer over a mini drawer (`open_mini_on_hover`) | `state.hover_expanded = true` for the frame |
| Toggle button in the app bar | Caller calls `state.toggle()` |

`MudAppBar` and `MudDrawer` are static containers; interactive state belongs to
the buttons placed inside them.

### `LayoutState` and declaration order

`MudMainContent` reads `LayoutState` when it is declared. Therefore:

- **`Docked` / `Always`**: declare the app bar first, then the drawer, then the
  main content.
- **`Never`**: declare the drawer first, then the app bar, then the main
  content (the drawer sits to the left of the vertical).

### Overlay z-order

Overlay drawers paint at `Order::Foreground` and their backdrop at
`Order::Middle`. Inline drawers paint at the normal z-order of the parent `Ui`.
`ClipMode::Never` in an inline (push) drawer needs no special handling because
the caller nests the drawer beside the vertical that holds the app bar.

---

## Performance and memory choices

| Choice | Effect |
|---|---|
| `LayoutState` is five `f32` + two `bool` | `Copy`, reset by a single assignment. |
| `DrawerState` is two `bool` | `Copy`, no allocation. |
| `AppBarResponse` is one `Response` + one `f32` | `Copy`-adjacent; no allocation. |
| Shadows are three stacked `rect_filled` calls | No `Frame`, no shader. Visually close to a blurred shadow. |
| `MudMainContent` reads `LayoutState` once | One `Rect` computation per frame. |
| Overlay drawers are `Area`-backed | One `Area::show` per open overlay frame; no per-frame `Frame`. |
| Hover latch on mini drawers is one `bool` | No pointer-hit-test beyond `Response::hovered`. |
| Pure helpers (`side_rect`, `compute_main_rect`, `resolve`) | Arithmetic on `Rect` / `f32` / enums; testable without a `Context`. |

---

## Testing

`cargo test -p mud_egui_components layout` runs the module's tests:

| Area | Tests |
|---|---|
| `DrawerVariant` | default, discriminants, unique names, predicates, `Display` |
| `DrawerClipMode` | default, discriminants, `starts_above_app_bar`, unique names, `Display` |
| `EffectiveVariant::resolve` | temporary always overlay, persistent always push, responsive follows the breakpoint, mini follows the breakpoint, sentinels override width |
| `LayoutState` | default all-zero, `reset`, `top_inset` with top / bottom app bar, `bottom_inset` |
| `compute_main_rect` | default, top app bar, bottom app bar, both drawers, everything composing, oversized insets |
| `resolve_side` | every anchor with and without RTL |
| `side_rect` | left, right, top offset |
| `MudAppBar` | defaults, every builder, height constants, contrast threshold |
| `MudDrawer` | defaults, every builder, default extents |
| `MudDrawerHeader` | default, dense, height constants |
| `MudLayout` | default, classes |

Rendering needs a live `egui::Context`; the pure layer is the source of truth.

```bash
cargo test -p mud_egui_core drawer_variant drawer_clip_mode
cargo test -p mud_egui_components layout
```

---

## Differences from MudBlazor

- **`LayoutState` replaces the CSS variables.** MudBlazor publishes
  `--mud-drawer-width-left`, `--mud-appbar-height` and friends from
  `MudDrawerContainer`; the port uses a caller-owned struct that each child
  writes into as it is declared. `MudMainContent` reads it — the same
  mechanism, typed.
- **`MudLayout` hands the caller a screen-sized `Ui`.** The caller chooses the
  nesting; the port does not auto-compose the shell. This is the immediate-mode
  equivalent of MudBlazor's CSS-driven placement.
- **`Open` / `OpenChanged` become `DrawerState` + `DrawerResponse`.** The caller
  owns the flag; `opened` / `closed` report the transition frame. Same split as
  every other component in this workspace.
- **`Width` / `MiniWidth` / `Height` are `f32` points, not CSS strings.**
  `300px` and `30%` have no egui equivalent; compute a percentage from
  `ui.available_width()` before calling the builder.
- **`Overlay` / `OverlayAutoClose` are honoured.** A temporary or
  responsive-below-breakpoint drawer paints a screen backdrop at `Order::Middle`
  and closes on click when `overlay_auto_close`.
- **`ClipMode` affects z-order and top offset, not push behaviour.** The
  effective variant decides whether the drawer reserves width. `Never` runs
  full height above the app bar; `Docked` / `Always` start below it.
- **`Fixed` is stored.** egui has no free-floating chrome inside a
  `ScrollArea`; scroll-away behaviour is the caller's decision.
- **`MudAppBar` does not delegate to `MudToolBar`.** MudToolBar would paint a
  second frame around the content. The app bar paints one frame and lays the
  content out inline with the same 16 pt gutters and 64 / 48 pt height.
- **`ToolBarClass` is stored, not resolved.** No CSS class system.
- **`Contextual` on `MudAppBar` is not ported.** It is a 8.14+ addition
  (PR #10210), not present in 8.13.0.
- **`Style` / `Tag` / `UserAttributes` are not ported.** No CSS strings, no
  HTML tag.

===== ./crates/mud_egui_components/src/layout/spacer/README.md =====

# MudSpacer for egui

A zero-content filler for [egui](https://github.com/emilk/egui), ported from MudBlazor's
`MudSpacer`. It expands into the free space of the current layout so that widgets on
either side are pushed apart, like a `flex-grow: 1` element in a CSS flex container.

## Contents

1. [Design principle: API parity, not API invention](#design-principle-api-parity-not-api-invention)
2. [Architecture](#architecture)
3. [API reference](#api-reference)
4. [Layout behaviour](#layout-behaviour)
5. [Examples](#examples)
6. [Performance and memory choices](#performance-and-memory-choices)
7. [Testing](#testing)
8. [Differences from MudBlazor](#differences-from-mudblazor)

---

## Design principle: API parity, not API invention

`MudSpacer` is a special case in MudBlazor: it has **no documented parameters of its
own**. An upstream issue (MudBlazor #5577) notes that the component and its API are not
documented at all and that IDE IntelliSense shows nothing beyond the inherited members.
The only members it has are therefore those of `MudComponentBase`: `Class`, `Style`,
`Tag` and `UserAttributes`. I could not retrieve its source file, so this statement
rests on that issue and on how MudBlazor examples use it (a filler between siblings).

Because of that, this port deliberately has **no** `Value` / `ValueChanged`, no
`OnClick` / `OnFocus` / `OnKeyDown`, no `Variant`, `Color`, `Size`, `Elevation`,
`Disabled`, `ReadOnly`, no `ChildContent` and no `AdornmentStart` / `AdornmentEnd`. A
spacer has no content, no value and nothing to interact with.

## Architecture

```
mud_spacer.rs
├── pure helpers       sane_length, sane_fraction
├── MudSpacer          public builder struct
│   ├── builder        reserve, fraction, min_size, share, flavor, classes, tag
│   ├── sizing math    resolve_extent, equal_share_fraction (pure, pub)
│   │                  plan_size (pure, private)
│   └── rendering      show
├── impl Widget        ui.add(MudSpacer::new())
└── mod tests          pure tests + headless egui frame tests
```

Render flow for `show(ui)`:

1. Fetch the theme and resolve the utility-class `Frame` (Bootstrap or Tailwind).
2. Inside the frame, read the layout's main direction.
3. `plan_size` calls `resolve_extent` for the main axis; the cross axis is `0`.
4. `allocate_response(size, Sense::hover())`.

## API reference

### Members that exist in MudBlazor

| Rust | MudBlazor | Description |
|------|-----------|-------------|
| `.classes([..])` | `Class` | Utility classes, resolved by the selected flavor. |
| `.tag(T)` / `.tag_ref::<T>()` | `Tag` | Attach and read back user data (`T: Any + Send + Sync`). |

### Extensions (needed because egui has no flex solver)

| Rust | Type | Default | Description |
|------|------|---------|-------------|
| `.reserve(f32)` | `f32` | `0.0` | Points to leave free after the spacer for following widgets. |
| `.fraction(f32)` | `f32` | `1.0` | Share of the remaining space to take, clamped to `0..=1` (`NaN` → `1`). |
| `.min_size(f32)` | `f32` | `0.0` | Minimum extent along the main axis. |
| `.share(index, count)` | — | — | The `index`-th of `count` spacers splitting the free space equally (sets `fraction`). **New.** |
| `.flavor(StyleFlavor)` | `StyleFlavor` | default | Bootstrap or Tailwind utility resolver. |

### Associated functions

| Function | Description |
|----------|-------------|
| `MudSpacer::resolve_extent(available, item_spacing, reserve, fraction, min_size)` | The sizing formula, public so you can reuse it. |
| `MudSpacer::equal_share_fraction(index, count)` | Fraction for the `index`-th of `count` equal spacers: `1 / (count - index)`. |

### Not applicable in egui

| MudBlazor | Why |
|-----------|-----|
| `Style` | No inline CSS strings in egui; use `.classes(..)`. |
| `UserAttributes` | No HTML tag to splat attributes onto. |

## Layout behaviour

```
extent = max(min_size, (available - item_spacing - reserve).max(0) * fraction)
```

- In a **horizontal** layout the spacer's size is `(extent, 0)`; in a **vertical** layout
  it is `(0, extent)`.
- When the available length is **not finite** (the scroll direction of a `ScrollArea`),
  there is no free space to share and the spacer returns `min_size`.
- `item_spacing` is subtracted once because egui adds one gap after every widget.
- Several spacers in one row: egui lays out one widget at a time, so a spacer cannot
  know how many siblings follow. The first one would take everything. Use
  `.share(index, count)` to split the space equally; the split is exact when item
  spacing and `reserve` are zero and approximate otherwise.
- The spacer is non-interactive (`Sense::hover()`), so it has no hover, focus or
  disabled styling.

## Examples

### Toolbar: title left, actions right

```rust
ui.horizontal(|ui| {
    ui.heading("Inbox");
    MudSpacer::new().show(ui);
    ui.button("Compose");
});
```

### Keep room for trailing widgets

```rust
ui.horizontal(|ui| {
    ui.label("Status");
    ui.add(MudSpacer::new().reserve(80.0).min_size(8.0));
    ui.label("Right side, about 80 pt wide");
});
```

### Three groups with equal gaps

```rust
ui.horizontal(|ui| {
    ui.label("A");
    ui.add(MudSpacer::new().share(0, 2));
    ui.label("B");
    ui.add(MudSpacer::new().share(1, 2));
    ui.label("C");
});
```

### Push a footer to the bottom of a column

```rust
ui.vertical(|ui| {
    ui.label("Content");
    ui.add(MudSpacer::new().reserve(32.0));
    ui.button("Footer");
});
```

### Utility classes and user data

```rust
ui.add(
    MudSpacer::new()
        .flavor(StyleFlavor::Tailwind)
        .classes(["px-2"])
        .tag("toolbar-gap"),
);
```

## Performance and memory choices

A spacer is about as cheap as a widget gets; the work is removing avoidable cost, not
inventing complexity.

- **Pure sizing on `f32`.** `resolve_extent` and `equal_share_fraction` are `#[inline]`,
  allocation-free and lock-free. No state is shared between frames or threads.
- **Stack-only builder.** Apart from the optional tag (`Arc<dyn Any + Send + Sync>`,
  allocated only when you call `.tag`), the struct holds only scalars and the style
  resolver state.
- **Single-axis plan.** `plan_size` computes only the main-axis extent; the cross axis
  is a constant `0`.
- **Dead state removed.** The previous version wrote `hovered` / `active` into a
  `TwState` after the spacer had already been consumed, which had no effect; that code
  is gone.
- **Sanitized inputs.** Negative, `NaN` and infinite values can never produce a negative
  or non-finite size.

Not done: caching the resolved utility `Frame`. The style resolvers live in other crates
and own their invalidation rules.

## Testing

`cargo test` covers:

- **Sanitizers** and every branch of `resolve_extent`: default fill, reserve, fraction,
  `min_size` floor, unbounded space, negative availability, bad inputs.
- **Equal shares:** the fraction sequence, edge cases, and a simulated split of 300 pt
  across three spacers.
- **Builder:** defaults, input sanitizing, `share`, tag round trips, `plan_size` axis
  selection.
- **Headless frames:** horizontal and vertical spacers fill their layout, `reserve`
  shrinks the extent by exactly that amount, `fraction(0.5)` takes about half, and the
  `Widget` impl works.

There are no event-callback tests because a spacer has no events.

## Differences from MudBlazor

- `reserve`, `fraction`, `min_size` and `share` have no MudBlazor counterpart; they stand
  in for the free-space resolution that a CSS flex container does automatically.
- `Style` and `UserAttributes` are not ported (no CSS or HTML in egui).
- Several MudBlazor spacers split space by `flex-grow`; here you opt in with `share`.

===== ./crates/mud_egui_components/src/layout/split_panel/README.md =====

# MudSplitPanel Component

A resizable split-view container for `egui`.

`MudSplitPanel` is the direct port of MudBlazor's `MudSplitPanel` (and Avalonia's `MudSplitPanel.cs`). It divides a container into two resizable panels separated by a draggable splitter. It is the perfect foundation for IDE layouts, file explorers, chat applications, and master-detail views.

---

## Quick Start

The split panel requires two closures: one for the left/top panel, and one for the right/bottom panel.

```rust
use mud_egui_components::MudSplitPanel;

MudSplitPanel::new()
    .id_salt("my_editor_layout") // IMPORTANT: See below!
    .show(
        ui,
        |ui| {
            // First Panel (Left or Top)
            ui.label("Sidebar");
        },
        |ui| {
            // Second Panel (Right or Bottom)
            ui.label("Main Content");
        },
    );

===== ./crates/mud_egui_components/src/layout/stack/README.md =====

# MudStack (egui port)

An immediate-mode port of MudBlazor's [`MudStack`](https://mudblazor.com/components/stack) — a
single-axis layout container that arranges its children in a row or column with a uniform gap,
optional main-axis justification, cross-axis alignment, wrapping, reversal, responsive
row/column collapse, and leftover-space distribution.

It targets [`egui`](https://github.com/emilk/egui) and is built to feel the way `MudStack` feels
in Blazor, while being honest about where an immediate-mode, single-pass UI framework cannot
reproduce retained-mode/CSS flexbox behavior exactly. Those divergences are documented, not
hidden — see [How it differs from CSS flexbox](#how-it-differs-from-css-flexbox).

> **Scope note.** `MudStack` in real MudBlazor is a pure layout primitive: it has no visual
> surface, no click target, no elevation, and no adornment slots. This port deliberately does
> **not** add `Variant`, `Color`, `Elevation`, `OnClick`/`OnFocus`/`OnKeyDown`, `ReadOnly`,
> ripple/focus rings, or `AdornmentStart`/`AdornmentEnd` — those parameters belong to MudBlazor's
> buttons, inputs, and papers, not to `MudStack`. Every parameter below was checked against
> MudBlazor's actual, merged source and docs before being ported.

---

## Table of contents

- [Quick start](#quick-start)
- [Architecture](#architecture)
- [API reference](#api-reference)
- [Examples](#examples)
- [Performance and memory layout](#performance-and-memory-layout)
- [How it differs from CSS flexbox](#how-it-differs-from-css-flexbox)
- [Testing](#testing)

## Quick start

```rust
// Plain usage: just draw children.
MudStack::new().spacing(2).show(ui, |ui| {
    ui.label("one");
    ui.label("two");
});
```

```rust
// Justify::Space* and StretchItems need to know where each item begins and ends,
// so they go through `show_items` + `StackUi::item` instead of a plain closure.
MudStack::horizontal()
    .justify(Justify::SpaceBetween)
    .align_items(AlignItems::Center)
    .classes(["p-2", "bg-primary"])
    .show_items(ui, |stack| {
        stack.item(|ui| { ui.label("left"); });
        stack.item(|ui| { ui.label("middle"); });
        stack.item(|ui| { ui.button("right"); });
    });
```

## Architecture

The file is deliberately split into two layers:

```
┌─────────────────────────────────────────────────────────────┐
│  Render path (needs a live egui::Context)                    │
│  MudStack::show / show_items → run → run_inner                │
│  StackUi::item                                                │
│                                                                │
│  Gathers inputs from the Ui (available rect, previous-frame   │
│  Measure, previous-frame item extents), calls the pure        │
│  helpers below for every decision, and applies the results    │
│  (Layout, Rect, item_spacing) to a scoped child Ui.            │
└─────────────────────────────────────────────────────────────┘
                              │ calls
                              ▼
┌─────────────────────────────────────────────────────────────┐
│  Pure helpers (no Ui, no Context — unit-tested directly)      │
│  resolve_is_row    · plan_rect        · plan_offsets          │
│  cross_align       · build_layout     · item_spacing          │
│  should_break_line · advance_line     · extents_approx_eq     │
│  stretch_targets   · plan_stretch     · stretched_item_rect   │
└─────────────────────────────────────────────────────────────┘
```

This keeps every piece of layout arithmetic testable without constructing an `egui::Context`,
and keeps the render path itself thin — it's mostly plumbing between the `Ui` and the helpers.

**Component (`MudStack`)** — the builder. Holds configuration only; `#[must_use]` so a stack
built and never `.show()`n or `.show_items()`n is a compile-time warning, not a silent no-op.

**`StackUi`** — the handle passed into `show_items`. Wraps the stack's child `Ui` and
accumulates the main-axis extent of every [`StackUi::item`](#stackuiitem), which is what makes
`Justify::Space*` and `StretchItems` possible — both need item *boundaries*, which a plain
`FnOnce(&mut Ui)` closure in `show` doesn't expose.

**`Measure`** — a small `Copy` struct persisted in egui's temp memory between frames (content
extent, item-sum extent, item count). This is the mechanism behind every "needs the previous
frame's size" feature: `Center`/`End`/`Space*` justification, reversed-column anchoring,
wrap-aware line breaking, and `StretchItems`.

## API reference

### `MudStack` — builder parameters

| Parameter | Type | Default | Builder method | Notes |
|---|---|---|---|---|
| `base` | `ComponentBase` | `ComponentBase::new()` | `.classes(...)` | Utility classes (`p-*`, `m-*`, `bg-<role>`); theme/padding are only resolved if at least one is recognised. |
| `row` | `bool` | `false` | `.row(bool)` | Row vs. column. `MudStack::horizontal()` / `::vertical()` are shorthands. |
| `reverse` | `bool` | `false` | `.reverse(bool)` | `row-reverse` / `column-reverse`. |
| `spacing` | `u8` | `3` (12px) | `.spacing(u8)` | Gap in 4px units, clamped to `0..=20` (both in the builder and again at render time). |
| `align_items` | `AlignItems` | `Stretch` | `.align_items(...)` | Cross-axis alignment: `Start`, `Center`, `End`, `Stretch`. |
| `justify` | `Justify` | `Start` | `.justify(...)` | Main-axis distribution: `Start`, `Center`, `End`, `SpaceBetween`, `SpaceAround`, `SpaceEvenly`. Anything but `Start` needs a known main length (a row's available width, or a column's `.height(..)`) and, for `Center`/`End`/`Space*`, a previous-frame measurement. |
| `wrap` | `Wrap` | `NoWrap` | `.wrap(...)` | `NoWrap` / `Wrap`. Disables `Justify` other than `Start` while active. |
| `breakpoint` | `Breakpoint` | `None` | `.breakpoint(...)` | A row collapses to a column once the container is narrower than this breakpoint. Matches MudBlazor's real, merged semantics: "the state of `Row` is reversed based on the defined screen size breakpoint." |
| `height` | `Option<f32>` | `None` | `.height(f32)` | Fixed height. Required for a column to have main-axis free space to justify, wrap, or stretch into. |
| `stretch_items` | `StretchItems` | `None` | `.stretch_items(...)` | Which items absorb leftover main-axis space. Only effective via `show_items`. See [`StretchItems`](#stretchitems). |
| `id_salt` | `Option<Id>` | `None` | `.id_salt(impl Hash + Debug)` | Disambiguates the memory key two stacks would otherwise share (and therefore the measurements they remember). |

### Constructors

| Function | Equivalent to |
|---|---|
| `MudStack::new()` | `MudStack::default()` — vertical, un-reversed, `spacing = 3`, `Stretch`, `Start`. |
| `MudStack::horizontal()` | `MudStack::new().row(true)` |
| `MudStack::vertical()` | `MudStack::new().row(false)` (same as `new()`) |

### Rendering

| Method | Signature | Use when |
|---|---|---|
| `show` | `fn show<R>(self, ui: &mut Ui, add_children: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R>` | You don't need `Space*` justification or `StretchItems` — the common case. |
| `show_items` | `fn show_items<R>(self, ui: &mut Ui, add_items: impl FnOnce(&mut StackUi<'_>) -> R) -> InnerResponse<R>` | You need `Space*` justification and/or `StretchItems`, which require per-item boundaries. |
| `ui` (via `impl Widget for MudStack`) | — | Rendering an *empty* stack, e.g. purely for its utility-class frame (a coloured spacer). |

### `StackUi<'a>`

| Method | Returns | Notes |
|---|---|---|
| `item(add_contents)` | `InnerResponse<R>` | Adds one child (which may itself contain several widgets) as a single measured, unbreakable unit. While wrapping, starts a new line first if the previous frame's extent says it no longer fits. While `stretch_items` is active, widens the item's `Ui` before drawing it so content that fills `available_width()/height()` can grow into the leftover space. |
| `is_row()` | `bool` | Whether the stack resolved to a row after any breakpoint collapse. |
| `ui()` | `&mut Ui` | Direct escape hatch; widgets added this way are **not** tracked as items (no `Space*`/`StretchItems` participation). |

### `StretchItems`

Port of MudBlazor's `StretchItems` ([`MudBlazor#8545`](https://github.com/MudBlazor/MudBlazor/pull/8545)).

| Variant | Behavior |
|---|---|
| `None` *(default)* | No item receives extra space. |
| `Start` | Only the first item grows. |
| `End` | Only the last item grows. |
| `StartAndEnd` | First and last items each grow by half the leftover space. |
| `Middle` | Every item except the first and last grows, sharing the leftover evenly. A no-op with 0, 1, or 2 items (there is no middle). |
| `All` | Every item grows, sharing the leftover evenly. |

The leftover space is computed from the *previous* frame's per-item extents (the same
frame-delay `Justify::Space*` already relies on), divided evenly among the flagged positions,
and handed to each flagged item as extra room in its `Ui`'s `max_rect` before it's drawn. What
the item does with that room is up to it: a child that fills `ui.available_width()` (or
`available_height()` in a column) visibly grows; a fixed-size child — a plain `MudButton`, say —
simply ignores the slack. This matches MudBlazor's real behavior, where stretching is likewise
opt-in per child rather than forced.

Falls back to a clean no-op (nothing resized) when: it's the first frame (nothing measured yet),
the stack is wrapping (no single leftover to distribute across multiple lines), the main length
is unknown or infinite, or the content already overflows the available space.

## Examples

### Toolbar: pinned ends, stretched middle

```rust
MudStack::horizontal()
    .stretch_items(StretchItems::Middle)
    .show_items(ui, |stack| {
        stack.item(|ui| { ui.heading("Inbox"); });
        stack.item(|ui| { ui.add_space(ui.available_width()); }); // grows to fill
        stack.item(|ui| { ui.button("Compose"); });
    });
```

### Responsive row that collapses to a column on narrow containers

```rust
MudStack::horizontal()
    .breakpoint(Breakpoint::Md)
    .spacing(4)
    .align_items(AlignItems::Center)
    .show(ui, |ui| {
        ui.label("Narrow? I'm a column now.");
        ui.button("Action");
    });
```

### Wrapping a row of cards

```rust
MudStack::horizontal()
    .wrap(Wrap::Wrap)
    .spacing(4)
    .show_items(ui, |stack| {
        for card in &cards {
            stack.item(|ui| render_card(ui, card));
        }
    });
```

### Centered, fixed-height column

```rust
MudStack::vertical()
    .height(240.0)
    .justify(Justify::Center)
    .align_items(AlignItems::Center)
    .show(ui, |ui| {
        ui.label("Centered both ways.");
    });
```

## Performance and memory layout

- **No heap allocation on the hot path.** `Measure` is a small `Copy` struct stored in egui's
  temp memory — reading and writing it costs a handful of words, not an allocation. No
  intermediate collections are built for a plain `show` call.
- **`classes(...)` is opt-in cost.** Utility-class parsing and theme lookup only run when
  `classes(..)` produced at least one recognised utility; the zero-utility case is a direct tail
  call into the render path, skipping the `Frame`/theme machinery entirely.
- **Wrapping and `StretchItems` pay only when used.** The previous-frame per-item extents
  (`Vec<f32>`) are only read/cloned when `wrap` or `stretch_items` is active; a plain non-item,
  non-wrapping stack never touches that memory key.
- **One repaint, not every frame.** Measurements are compared with an epsilon
  (`MEASURE_EPSILON` / `LINE_EPSILON`) rather than exact equality, so sub-pixel float noise from
  egui's layout doesn't trigger a repaint loop — a changed measurement requests exactly one
  corrective repaint.
- **Arithmetic is isolated and `Copy`-friendly.** Every layout decision — rect planning, offset
  planning, line-break decisions, stretch-space division — is a pure function over small `Copy`
  values, which both keeps them cheap and makes them unit-testable without a live `Ui`.

## How it differs from CSS flexbox

- **Frame-delayed measuring.** `Center`, `End`, `Space*` justification, and `StretchItems` all
  need the size of the content, which egui only knows *after* drawing it. The stack remembers
  the previous frame's size and requests one repaint when it changes; the first frame lays out
  as `Start` / unstretched.
- **Main-axis free space.** A row fills the available width. A column has no free space unless
  given `.height(..)`, so `Justify` other than `Start` (and `StretchItems`) need a height there.
- **Wrapping.** `Wrap::Wrap` uses egui's wrapping layout; justification is ignored while
  wrapping. egui only starts a new line for *leaf* widgets — a composite child (a `Frame` /
  `MudPaper`) would otherwise be squeezed into the current line's leftover width instead of
  moving down. `StackUi::item` works around this by measuring each item as an unbreakable unit
  and calling `end_row()` before one that no longer fits, based on the previous frame's extent.
- **`Reverse`** uses egui's right-to-left / bottom-up layouts. A reversed column without
  `.height(..)` needs one hidden (invisible) sizing frame before it has something to anchor to.
- **`StretchItems`** hands leftover main-axis space to specific positions (first/last/middle/
  all), computed a frame late, the same way `Space*` is. It only works through `show_items`.
- **`FillIndex`** (arbitrary per-item proportional flex-grow weights) is **not** supported.
  Unlike `StretchItems`'s fixed start/end/middle/all split, arbitrary per-item weights would
  need a real multi-pass constraint solver — egui's immediate-mode, single-pass layout has no
  room for that.

## Testing

All arithmetic lives in small, pure, `#[cfg(test)]`-covered functions at the bottom of the file
— no `egui::Context` is constructed in tests. Coverage includes:

- `plan_rect` / `plan_offsets` across row/column, reversed/forward, wrapped/unwrapped, and every
  `Justify` variant, including overflow (never produces a negative offset).
- Wrapping line-break decisions (`should_break_line`, `advance_line`) and a full
  five-items-onto-two-lines scenario.
- `Measure` equality under the epsilon tolerance.
- `StretchItems`: `stretch_targets` index selection for every variant (including the 0/1/2-item
  edge cases for `Middle`), and `plan_stretch` space division, overflow fallback, infinite/
  unknown main length, and per-variant leftover splits.

Run with:

```sh
cargo test
```

===== ./crates/mud_egui_components/src/layout/tool_bar/README.md =====

# MudToolBar (egui port)

A horizontal toolbar panel for [egui](https://github.com/emilk/egui), ported from MudBlazor's
[`MudToolBar`](https://mudblazor.com/api/mudtoolbar). It gives its children a Material-style
row: 64 pt tall (48 pt dense), 16 pt gutters, vertically centered, optionally wrapping. Layout
of the children is delegated to [`MudStack`](./README.md).

## Contents

1. [Design principle: API parity, not API invention](#design-principle-api-parity-not-api-invention)
2. [Architecture](#architecture)
3. [API reference](#api-reference)
4. [Rendering behaviour](#rendering-behaviour)
5. [Examples](#examples)
6. [Performance and memory choices](#performance-and-memory-choices)
7. [Testing](#testing)
8. [Differences from MudBlazor](#differences-from-mudblazor)

---

## Design principle: API parity, not API invention

The current MudBlazor `MudToolBar` API (checked against the published API docs) is:

| MudBlazor | Type | Default | Description |
|-----------|------|---------|-------------|
| `ChildContent` | `RenderFragment` | — | The toolbar content, typically icon buttons. |
| `Dense` | `bool` | `false` | Compact vertical padding. |
| `Gutters` | `bool` | `true` | Left and right padding. (Older versions called it `DisableGutters`, with the opposite meaning.) |
| `WrapContent` | `bool` | `false` | Lets the content wrap. |
| `Class`, `Style`, `Tag`, `UserAttributes`, `FieldId` | base | — | Inherited from `MudComponentBase`. |

A toolbar has no value, no click target, no variants and no elevation (elevation is an
`MudAppBar` parameter). This port therefore has **no** `Value` / `ValueChanged`, no
`OnClick` / `OnFocus` / `OnKeyDown`, no `Variant`, `Color`, `Size`, `Elevation`,
`Disabled`, `ReadOnly` and no `AdornmentStart` / `AdornmentEnd`.

## Architecture

```
mud_tool_bar.rs
├── pure helpers       toolbar_height, resolve_inner_margin, content_min_height,
│                      used_width, used_height, vertical_extent_of
├── MudToolBar         builder: dense, gutters, wrap_content, justify_space_between,
│   │                  flavor, classes, tag
│   ├── show           plain closure       ──┐
│   ├── show_items     measured StackUi items ├──► render
│   ├── build_stack    MudStack::horizontal() config
│   └── render         frame → min size → MudStack → allocate in parent
├── impl Widget        ui.add(MudToolBar::new())  (empty toolbar)
└── mod tests
```

Render flow:

1. Fetch the theme and resolve the utility-class `Frame` (Bootstrap or Tailwind).
2. `resolve_inner_margin`: add the 16 pt gutters unless the classes already resolved
   to an inner margin.
3. Create a child `Ui` over the available rectangle (so the toolbar can be sized
   after drawing).
4. Inside the frame: force full width and `content_min_height`, then call `draw`, which is
   either `MudStack::show` or `MudStack::show_items`.
5. Allocate `used_width × used_height` in the parent and return that `Response`.

### How it uses `MudStack`

The toolbar builds exactly this stack and nothing else:

```rust
MudStack::horizontal()
    .justify(if justify_space_between { Justify::SpaceBetween } else { Justify::Start })
    .wrap(if wrap_content { Wrap::Wrap } else { Wrap::NoWrap })
    .align_items(AlignItems::Center)
    .spacing(2)            // 2 × 4 px = 8 px gap
```

Only the documented `MudStack` API is used (`horizontal`, `justify`, `wrap`,
`align_items`, `spacing`, `show`, `show_items`, `StackUi::item`). Consequences taken from the
stack README:

- `Justify::Space*` needs item boundaries, so `justify_space_between` only works through
  `MudToolBar::show_items` with `stack.item(..)`.
- `Justify::Space*` is ignored while wrapping.
- Space distribution is computed from the previous frame, so the first frame lays out as
  `Start`.

## API reference

### Members that exist in MudBlazor

| Rust | MudBlazor | Type | Default | Description |
|------|-----------|------|---------|-------------|
| `.dense(bool)` | `Dense` | `bool` | `false` | 48 pt instead of 64 pt. |
| `.gutters(bool)` | `Gutters` | `bool` | `true` | 16 pt left and right padding, skipped if the utility classes already set an inner margin. |
| `.wrap_content(bool)` | `WrapContent` | `bool` | `false` | Children wrap onto more rows (the toolbar grows). |
| closure of `show` / `show_items` | `ChildContent` | closure | — | The toolbar content. |
| `.classes([..])` | `Class` | iterator of `AsRef<str>` | none | Utility classes. |
| `.tag(T)` / `.tag_ref::<T>()` | `Tag` | `T: Any + Send + Sync` | `None` | User data. |

### Extension kept from the original port

| Rust | Type | Default | Description |
|------|------|---------|-------------|
| `.justify_space_between(bool)` | `bool` | `false` | Distribute free space between children. Requires `show_items`. |
| `.flavor(StyleFlavor)` | `StyleFlavor` | default | Bootstrap or Tailwind utility resolver. |

### Not applicable in egui

| MudBlazor | Why |
|-----------|-----|
| `Style` | No inline CSS strings in egui; use `.classes(..)`. |
| `UserAttributes`, `FieldId` | No HTML tag to splat attributes onto or label for. |

### Rendering methods

| Method | Signature | Use when |
|--------|-----------|----------|
| `show` | `fn show<R>(self, ui, impl FnOnce(&mut Ui) -> R) -> InnerResponse<R>` | The common case: draw widgets left to right. |
| `show_items` | `fn show_items<R>(self, ui, impl FnOnce(&mut StackUi<'_>) -> R) -> InnerResponse<R>` | You need `justify_space_between`. |
| `ui.add(toolbar)` | `Widget` impl | An empty toolbar, e.g. only for its background. |

The returned `InnerResponse::response` is the response of the **whole toolbar rectangle**
(full width, full height), not of the inner stack.

## Rendering behaviour

| Aspect | Behaviour |
|--------|-----------|
| Height | `max(content, 64)` pt, or `max(content, 48)` when dense. Wrapped content grows the toolbar. |
| Width | The available width (or the content width when the available width is unbounded). |
| Gutters | 16 pt left and right; none when `gutters(false)`; skipped when classes provide padding. |
| Vertical alignment | `AlignItems::Center` on the stack. |
| Gap | 8 pt between children. |
| Background / borders | Come from utility classes via the frame and cover the whole toolbar (see below). |

A toolbar is a static container, so it has no hover, focus, active or disabled styling of
its own. Interactive state belongs to the buttons placed inside it.

## Examples

### Basic toolbar

```rust
MudToolBar::new().show(ui, |ui| {
    ui.heading("Inbox");
    ui.button("Compose");
});
```

### Dense toolbar with a background utility class

```rust
MudToolBar::new()
    .dense(true)
    .classes(["bg-primary"])
    .show(ui, |ui| {
        ui.label("Selected: 3");
    });
```

### Title left, actions right (space-between)

```rust
MudToolBar::new()
    .justify_space_between(true)
    .show_items(ui, |stack| {
        stack.item(|ui| { ui.heading("Inbox"); });
        stack.item(|ui| {
            ui.horizontal(|ui| {
                ui.button("Search");
                ui.button("Compose");
            });
        });
    });
```

### Title left, actions right (spacer)

```rust
MudToolBar::new().show(ui, |ui| {
    ui.heading("Inbox");
    ui.add(MudSpacer::new().reserve(100.0));
    ui.button("Compose");
});
```

### Wrapping toolbar of chips

```rust
MudToolBar::new().wrap_content(true).show(ui, |ui| {
    for tag in &tags {
        ui.label(tag);
    }
});
```

### No gutters, user data

```rust
let bar = MudToolBar::new().gutters(false).tag("editor-toolbar");
assert_eq!(bar.tag_ref::<&str>(), Some(&"editor-toolbar"));
bar.show(ui, |ui| { ui.button("Bold"); });
```

## Performance and memory choices

- **One pass, no intermediate collections.** The toolbar adds no per-frame `Vec`s or
  strings; all numeric decisions are pure `f32` / `Margin` functions.
- **Frame resolved once per call, not per child.** Utility classes resolve into a single
  `Frame`; the stack handles child layout.
- **No dead state.** The old version wrote `hovered` / `active` into `TwState` after the
  toolbar had been consumed (no effect); that code is removed.
- **No duplicate response work.** The toolbar allocates its rectangle once and returns that
  response; the old code discarded it.
- **Sanitized geometry.** Non-finite widths or strokes never reach `allocate_rect`.

Not done: caching the resolved frame. The style resolvers live in other crates and own
their invalidation rules; `MudStack` remains responsible for its own measurement memory.

## Testing

`cargo test` covers:

- **Pure helpers:** heights, gutter resolution (with and without explicit padding),
  vertical extents, content minimum height (including negative and `NaN` strokes), used
  width / height.
- **Builder:** defaults, every setter, tag round trips.
- **Headless frames:** standard toolbar reserves 64 pt and the full width; dense reserves
  48 pt; `show` and `show_items` return the closure value; gutters move the first child
  inwards; a heavily wrapped toolbar is never shorter than standard; the `Widget` impl
  renders an empty toolbar.

There are no event-callback tests because a toolbar has no events.

## Differences from MudBlazor

- **Bug fixed:** `justify_space_between` previously did nothing, because it was only ever
  combined with the plain `show`; it now works through `show_items`.
- **Bug fixed:** utility-class backgrounds previously covered only the content height while
  the toolbar reserved 64 / 48 pt. The frame content now spans the full width and height.
- **Returned response** is now the toolbar's full rectangle rather than the inner stack's.
- **Vertical centering** inside the minimum height relies on `MudStack`'s
  `AlignItems::Center`; the toolbar guarantees the frame is that tall, but the placement of
  children inside it is the stack's responsibility.
- `Style`, `UserAttributes` and `FieldId` are not ported (no CSS or HTML in egui).
- `justify_space_between` has no MudBlazor counterpart.

===== ./crates/mud_egui_components/src/navigation/breadcrumbs/README.md =====

# MudBreadcrumbs for `mud_egui`

A horizontal, wrapping trail of links for [egui](https://github.com/emilk/egui),
ported from MudBlazor's [`MudBreadcrumbs`](https://mudblazor.com/components/breadcrumbs)
(MudBlazor 8.13.0). Supports automatic collapsing via `MaxItems`, custom
separators, per-item templates, and routing-aware navigation through the
`NavigationManager`.

```rust
MudBreadcrumbs::new(&items)
    .separator_icon(filled::CHEVRON_RIGHT)
    .size(Size::Small)
    .mark_last_current(true)
    .show(ui, nav);
```

Every string renders through **`MudText`**, every glyph through **`MudIcon`**.

---

## Contents

1. [Design principle: API parity, not API invention](#design-principle-api-parity-not-api-invention)
2. [Architecture](#architecture)
3. [API reference](#api-reference)
4. [Examples](#examples)
5. [Collapse semantics](#collapse-semantics)
6. [Performance and memory choices](#performance--memory-choices)
7. [Testing](#testing)
8. [Differences from MudBlazor](#differences-from-mudblazor)

---

## Design principle: API parity, not API invention

Verified against DNDocs for MudBlazor 8.13.0.

### Ported parameters

| MudBlazor | Rust method | Default |
|-----------|-------------|---------|
| `Collapsed` (get-only) | `BreadcrumbResponse::collapsed` | derived |
| `ExpanderIcon` | `expander_icon(s)` | `SETTINGS_ETHERNET` |
| `ItemTemplate` | `item_template(f)` | `None` |
| `Items` | `new(&items)` / `items(&items)` | required |
| `MaxItems` | `max_items(u8)` | `None` |
| `Separator` | `separator(s)` | `"/"` |
| `SeparatorTemplate` | `separator_template(f)` | `None` |
| `Class` (base) | `classes(iter)` | `None` |
| `Style` (base) | *(use `classes`)* | — |
| `Tag` (base) | **not ported** (no HTML tag) | — |
| `UserAttributes` | **not ported** | — |

### `BreadcrumbItem` (record)

| MudBlazor | Rust | Default |
|-----------|------|---------|
| `Text` | `text` / `new(s)` | required |
| `Href` | `href` / `link(s, h)` | `None` |
| `Disabled` | `disabled` / `disabled(b)` | `false` |
| `Icon` | `icon` / `icon(s)` | `None` |
| — | `color` / `color(c)` (extension) | `None` |

### Extensions (not in MudBlazor)

| Extension | Description |
|-----------|-------------|
| `separator_icon(svg)` | SVG glyph separator. Overrides `separator`, superseded by `separator_template`. |
| `size(Size)` | Icon px and gap. |
| `typo(Typo)` | Passed to `MudText`. |
| `color(Color)` | Default link colour. |
| `mark_last_current(bool)` | Last item is "current" (secondary colour, no click). |
| `id_salt(s)` | Stable interaction ids. |
| `on_navigate(f)` | Fires before dispatch. |
| `show_into(ui, nav, &mut bool)` | Writes `collapsed` back. |

### Deliberately not ported

- **`Tag` / `UserAttributes`** — no HTML tag.
- **`Style`** — no inline CSS strings; use `classes(..)`.

---

## Architecture

```
breadcrumbs/
├── breadcrumb_item.rs    — BreadcrumbItem + builders
├── breadcrumb_action.rs  — BreadcrumbAction enum
├── mud_breadcrumbs.rs    — MudBreadcrumbs + VisiblePlan + pure helpers
└── mod.rs
```

**Pure layer.** `plan_visible_indices`, `icon_size_px`, `is_current_item`,
`resolve_color` are free functions / methods, unit-tested without
`egui::Context`.

**State layer.** The component owns no state across frames. The only derived
state — `collapsed` — is exposed in `BreadcrumbResponse` and, for backwards
compatibility, written back via `show_into(ui, nav, &mut bool)`.

**Component layer.** `show` resolves the theme, computes the `VisiblePlan`,
renders head → expander → tail with separators between items, and dispatches
clicks through the shared `is_external_url` helper.

---

## API reference

### `MudBreadcrumbs<'a>`

| Method | Type | Default | Description |
|--------|------|---------|-------------|
| `new(items)` | `&'a [BreadcrumbItem]` | — | Constructor. |
| `empty()` | — | — | Empty trail. |
| `items(&[..])` | `&'a [BreadcrumbItem]` | — | Replace items. |
| `expander_icon(s)` | `impl Into<String>` | `SETTINGS_ETHERNET` | Collapse glyph. |
| `max_items(u8)` | `u8` | `None` | Collapse threshold. |
| `separator(s)` | `impl Into<String>` | `"/"` | Text separator. |
| `separator_icon(s)` | `impl Into<String>` | `None` | SVG separator (extension). |
| `separator_template(f)` | `FnMut(&mut Ui)` | `None` | Custom separator. |
| `item_template(f)` | `FnMut(&mut Ui, &BreadcrumbItem, bool)` | `None` | Custom item content. |
| `size(Size)` | `Size` | `Medium` | Icon/gap preset. |
| `typo(Typo)` | `Typo` | `Body2` | `MudText` typography. |
| `color(Color)` | `Color` | `None` | Default link colour. |
| `mark_last_current(bool)` | `bool` | `false` | Last item = current. |
| `id_salt(s)` | `impl Into<String>` | `None` | Stable interaction ids. |
| `on_navigate(f)` | `FnMut(usize, &BreadcrumbItem)` | `None` | Pre-dispatch callback. |
| `classes(iter)` | `iter of AsRef<str>` | none | Wrapper classes. |
| `flavor(StyleFlavor)` | `StyleFlavor` | `Bootstrap` | Utility resolver. |
| `show(ui, nav)` | `(&mut Ui, &mut NavigationManager)` | — | Returns `BreadcrumbResponse`. |
| `show_and_navigate(ui, nav)` | alias | — | Alias for `show`. |
| `show_into(ui, nav, &mut bool)` | — | — | Writes `collapsed` back. |

### `BreadcrumbItem`

| Method | Description |
|--------|-------------|
| `new(text)` | Text-only item. |
| `link(text, href)` | Clickable item. |
| `href(s)` | Set the navigation URL. |
| `disabled(b)` | Disable clicks. |
| `icon(s)` | Leading icon. |
| `color(c)` | Colour override (extension). |
| `text(s)` | Replace caption. |

### `BreadcrumbResponse`

| Field | Type | Description |
|-------|------|-------------|
| `response` | `egui::Response` | Wrapper response. |
| `collapsed` | `bool` | Whether the trail was collapsed. |
| `action` | `BreadcrumbAction` | Navigation outcome this frame. |

### `BreadcrumbAction`

| Variant | Description |
|---------|-------------|
| `None` | Nothing happened. |
| `Internal { index, uri }` | Navigated internally. |
| `Vetoed { index, uri }` | Guard vetoed navigation. |
| `Error { index, uri, message }` | Navigation failed. |
| `External { index, url }` | Opened the system browser. |

Helpers: `navigated()`, `was_blocked()`, `is_external()`, `target()`,
`index()`.

---

## Examples

### Basic trail

```rust
let items = [
    BreadcrumbItem::link("Home", "/"),
    BreadcrumbItem::link("Orders", "/orders"),
    BreadcrumbItem::new("Order #42"),
];

MudBreadcrumbs::new(&items).show(ui, nav);
```

### Icon separator

```rust
MudBreadcrumbs::new(&items)
    .separator_icon(filled::CHEVRON_RIGHT)
    .show(ui, nav);
```

### Collapsing trail

```rust
MudBreadcrumbs::new(&items)
    .max_items(3)
    .show(ui, nav);
```

### Last item as current

```rust
MudBreadcrumbs::new(&items)
    .mark_last_current(true)
    .show(ui, nav);
// The last item is drawn in the secondary colour and ignores clicks.
```

### Custom item template

```rust
MudBreadcrumbs::new(&items)
    .item_template(|ui, item, is_current| {
        if is_current {
            ui.add(MudText::new(item.text.clone()).bold(true));
        } else {
            ui.add(MudText::new(item.text.clone()));
        }
    })
    .show(ui, nav);
```

### Custom separator

```rust
MudBreadcrumbs::new(&items)
    .separator_template(|ui| {
        ui.add(MudIcon::new(filled::ARROW_FORWARD).size_px(12.0));
    })
    .show(ui, nav);
```

### Read back collapsed state

```rust
let mut collapsed = false;
MudBreadcrumbs::new(&items)
    .max_items(3)
    .show_into(ui, nav, &mut collapsed);
```

### Observe the navigation outcome

```rust
let resp = MudBreadcrumbs::new(&items).show(ui, nav);
match resp.action {
    BreadcrumbAction::Internal { index, uri } => log::info!("{index} → {uri}"),
    BreadcrumbAction::External { url, .. } => log::info!("opened {url}"),
    BreadcrumbAction::Vetoed { uri, .. } => log::warn!("vetoed {uri}"),
    BreadcrumbAction::Error { message, .. } => log::error!("{message}"),
    BreadcrumbAction::None => {}
}
```

---

## Collapse semantics

`MaxItems` is `Option<u8>`. When `Some(max)` and `items.len() > max`:

```
head   = [0]                            // always the first item
expander = true
tail   = items[len - (max - 1) .. len]  // last max-1 items
```

| `len` | `max` | head | expander | tail | visible slots |
|-------|-------|------|----------|------|---------------|
| 3 | 5 | `[0,1,2]` | false | `[]` | 3 |
| 5 | 5 | `[0,1,2,3,4]` | false | `[]` | 5 |
| 6 | 5 | `[0]` | true | `[2,3,4,5]` | 1 + 1 + 4 |
| 10 | 3 | `[0]` | true | `[8,9]` | 1 + 1 + 2 |
| 10 | 1 | `[0]` | true | `[]` | 1 + 1 |
| 10 | 0 | `[0]` | true | `[]` | 1 + 1 (coerced) |
| 100 | None | all 100 | false | `[]` | 100 |

Head and tail never overlap because `len > max` implies
`len - (max - 1) >= 2`, so `tail_start >= 2 > head[0] == 0`.

The expander is **display-only** in MudBlazor. The Rust port matches:
clicking it does nothing. If you want click-to-expand, provide an
`item_template` that hooks the expander's `Sense::click()` yourself, or
replace the collapse logic with your own.

---

## Performance and memory choices

| Choice | Effect |
|--------|--------|
| `plan_visible_indices` is pure | No `egui::Context`, easy to test. |
| Head/tail are `Vec<usize>` | Bounded by `len`, one allocation per frame. |
| `MudText` / `MudIcon` for every string / glyph | Theme and typography propagate automatically. |
| `WidgetInfo::labeled(_, _, label.as_str())` | Zero-allocation a11y label. |
| Shared `is_external_url` with `MudLink` | One rule for internal vs. external navigation across the component library. |
| Single `ui.scope` per item | One `Response`, one interaction overlay. |
| Hover underline is a single `line_segment` | No shape tree. |

---

## Testing

`cargo test` runs 25+ pure-function tests:

| Area | Tests |
|------|-------|
| `icon_size_px` | every `Size` |
| `is_current_item` | last when marked, nothing when not marked, empty slice |
| `plan_visible_indices` | uncollapsed fits, no max, len == max, collapsed cases, head/tail disjointness, `max = 1`, `max = 0` coerced, empty slice |
| `resolve_color` | `None` / `Default` / `Inherit` fallback, explicit palette mapping |
| Defaults | match MudBlazor |
| Builders | every field, items replacement |
| `BreadcrumbAction` | every variant, `navigated`, `was_blocked`, `is_external`, `target`, `index` |

Rendering needs a live `egui::Context`; the pure layer is the source of truth.

```bash
cargo test -p mud_egui_components mud_breadcrumbs
```

---

## Differences from MudBlazor

- **`Collapsed` is in `BreadcrumbResponse`.** MudBlazor exposes it as a
  get-only property on the component; the port returns it from `show` so the
  caller can read it without a separate call. `show_into(ui, nav, &mut bool)`
  preserves the old `&mut bool` pattern.
- **`mark_last_current` is an extension.** MudBlazor has no explicit
  current-item flag; the port adds one so `BreadcrumbItem::new("Current")`
  renders in the secondary colour and ignores clicks without the caller
  manually setting `disabled(true)`.
- **`BreadcrumbItem::color` is an extension.** Not a MudBlazor field.
- **`separator_icon` is an extension.** Not a MudBlazor parameter; use
  `separator_template` when you want a fully custom separator.
- **`item_template` closure is `FnMut(&mut Ui, &BreadcrumbItem, bool)`.**
  The third argument is `is_current`, which MudBlazor does not pass (it
  infers current-ness from position).
- **`Tag` / `UserAttributes` not ported.** No HTML tag.
- **`Style` → `classes(..)`.** egui has no inline CSS strings.
- **`Widget::ui` treats every link as external** (there is no
  `NavigationManager` in that context). Use `MudBreadcrumbs::show(ui, nav)`
  when routing.

===== ./crates/mud_egui_components/src/navigation/nav_menu/README.md =====

# MudNavMenu for `mud_egui`

A vertical navigation list with collapsible groups for
[egui](https://github.com/emilk/egui), ported from MudBlazor's
[`MudNavMenu`](https://mudblazor.com/components/navmenu),
`MudNavGroup`, and `MudNavLink` (MudBlazor 8.13.0). Supports **routing-aware
active detection**, internal/external navigation, `Bordered` / `Rounded` /
`Dense` cascade styling, and caller-owned group expansion state.

```rust
let mut state = NavMenuState::new();

MudNavMenu::new()
    .bordered(true)
    .rounded(true)
    .color(Color::Primary)
    .show(ui, nav, &mut state, |menu| {
        menu.text_link("/", "Home");
        menu.group(
            NavGroup::new("Settings")
                .icon(filled::SETTINGS)
                .link(NavLink::new("/profile", "Profile"))
                .link(NavLink::new("/security", "Security")),
        );
        menu.text_link("/about", "About");
    });
```

Every label renders through **`MudText`**, every glyph through **`MudIcon`**.

---

## Contents

1. [Design principle: API parity, not API invention](#design-principle-api-parity-not-api-invention)
2. [Architecture](#architecture)
3. [API reference — MudNavMenu](#api-reference--mudnavmenu)
4. [API reference — NavGroup](#api-reference--navgroup)
5. [API reference — NavLink](#api-reference--navlink)
6. [Examples](#examples)
7. [Active detection](#active-detection)
8. [Performance and memory choices](#performance-and-memory-choices)
9. [Testing](#testing)
10. [Differences from MudBlazor](#differences-from-mudblazor)

---

## Design principle: API parity, not API invention

The MudBlazor 8.13.0 parameter surfaces for `MudNavMenu`, `MudNavGroup`, and
`MudNavLink` were verified against DNDocs before this refactor.

### Ported `MudNavMenu` parameters

| MudBlazor | Rust method | Default |
|-----------|-------------|---------|
| `Bordered` | `bordered(bool)` | `false` |
| `ChildContent` | closure to `show` | required |
| `Color` | `color(Color)` | `Color::Default` |
| `Dense` | `dense(bool)` | `false` |
| `Margin` | `margin(f32)` | `None` |
| `Rounded` | `rounded(bool)` | `false` |
| `Class` (base) | `classes(iter)` | `None` |
| `Style` (base) | *(use `classes`)* | — |
| `Tag` (base) | **not ported** (no HTML tag) | — |
| `UserAttributes` | **not ported** | — |

### Ported `MudNavGroup` parameters

| MudBlazor | Rust method | Default |
|-----------|-------------|---------|
| `ChildContent` | `link` / `group` | none |
| `Disabled` | `disabled(bool)` | `false` |
| `ExpandIcon` | `expand_icon(s)` | `ARROW_DROP_DOWN` |
| `Expanded` | `expanded(bool)` (seed) | from `NavMenuState` |
| `ExpandedChanged` | `on_expanded_changed(f)` | `None` |
| `HideExpandIcon` | `hide_expand_icon(bool)` | `false` |
| `Icon` | `icon(s)` | `None` |
| `IconColor` | `icon_color(Color)` | `None` |
| `MaxHeight` | `max_height(f32)` | `None` |
| `Ripple` | `ripple(bool)` | `true` (stored, not rendered) |
| `Title` | `new(s)` / `title(s)` | required |
| `TitleContent` | `title_content(f)` | `None` |

### Ported `MudNavLink` parameters

| MudBlazor | Rust method | Default |
|-----------|-------------|---------|
| `ChildContent` | `text(s)` / `child(f)` | required (one of) |
| `Disabled` | `disabled(bool)` | `false` |
| `ForceLoad` | `force_load(bool)` | `false` |
| `Href` | `new(href, text)` / `href(s)` | `None` |
| `Icon` | `icon(s)` | `None` |
| `IconColor` | `icon_color(Color)` | `None` |
| `Match` | `match_mode(NavLinkMatch)` | `Prefix` |
| `OnClick` | `on_click(f)` | `None` |
| `Ripple` | `ripple(bool)` | `true` (stored, not rendered) |
| `Target` | `target(s)` | `None` |

### Extensions (not in MudBlazor)

| Extension | Description |
|-----------|-------------|
| `NavGroup::id(s)` | Stable key for the expansion state. |
| `NavLink::child(f)` | Custom label receiving `is_active`. |
| `NavMenuState` | Caller-owned group expansion state. |
| `NavMenuUi::len` / `is_empty` | Collector introspection. |

### Deliberately not ported

- **`ActiveClass`** on `MudNavLink` — egui has no CSS classes. Use
  `MudNavMenu::color` for the active styling, or `NavLink::child` with the
  `is_active` flag.
- **`HeaderClass`** on `MudNavGroup` — same reason. Use `title_content`.
- **Protected members** (`Classname`, `ButtonClassname`, `TabIndex`,
  `IconClassname`, `LinkClassname`, `ExpandIconClassname`) — not public API.
- **`Tag` / `UserAttributes`** — no HTML tag.
- **`Style`** — no inline CSS strings; use `classes(..)`.

---

## Architecture

```
nav_menu/
├── nav_link.rs       — NavLink + builders
├── nav_group.rs      — NavGroup + builders
├── nav_entry.rs      — private NavEntry enum (Link | Group)
├── mud_nav_menu.rs   — MudNavMenu + NavMenuState + NavMenuUi + rendering
└── mod.rs

mud_egui_core/src/enums/
└── nav_link_match.rs — NavLinkMatch (Prefix | All)
```

**Pure layer.** `is_link_active`, `normalize_path`, `row_height`,
`resolve_accent`, `target_opens_new_tab` — free functions unit-tested without
`egui::Context`.

**State layer.** `NavMenuState { expanded: HashMap<String, bool> }` —
caller-owned, matching the pattern used by `MudTable`, `MudTreeView`, and
`MudChipSet`.

**Config layer.** `MudNavMenu`, `NavGroup`, `NavLink` are builders; no
mutable state on them across frames.

**Component layer.** `show` collects entries into a `NavMenuUi`, resolves the
cascade config, then walks the tree rendering headers/links and recursing into
expanded groups.

---

## API reference — `MudNavMenu`

| Method | Type | Default | Description |
|--------|------|---------|-------------|
| `new()` | — | — | Default menu. |
| `bordered(bool)` | `bool` | `false` | Left accent border on active links. |
| `rounded(bool)` | `bool` | `false` | Rounded active-link fill corners. |
| `dense(bool)` | `bool` | `false` | Compact rows (32 px). |
| `color(Color)` | `Color` | `Default` | Active accent (falls back to primary). |
| `margin(f32)` | `f32` | `None` | Extra vertical gap between top-level entries. |
| `classes(iter)` | `iter of AsRef<str>` | none | Wrapper classes. |
| `flavor(StyleFlavor)` | `StyleFlavor` | `Bootstrap` | Utility resolver. |
| `show(ui, nav, state, \|menu\| ..)` | `(&mut Ui, &mut NavigationManager, &mut NavMenuState, FnOnce(&mut NavMenuUi) -> R)` | — | Returns `(Response, R)`. |

### `NavMenuUi<'a>`

| Method | Description |
|--------|-------------|
| `link(link)` | Adds a `NavLink`. |
| `text_link(href, text)` | Convenience for `NavLink::new`. |
| `group(group)` | Adds a `NavGroup`. |
| `len()` / `is_empty()` | Collector introspection. |

---

## API reference — `NavGroup`

| Method | Type | Default | Description |
|--------|------|---------|-------------|
| `new(title)` | `impl Into<String>` | — | Group title (also the default id). |
| `id(s)` | `impl Into<String>` | title | Stable expansion key. |
| `title(s)` | `impl Into<String>` | — | Replace title. |
| `title_content(f)` | `FnMut(&mut Ui)` | `None` | Custom title renderer. |
| `icon(s)` | `impl Into<String>` | `None` | Leading icon. |
| `icon_color(Color)` | `Color` | `None` | Icon colour override. |
| `expand_icon(s)` | `impl Into<String>` | `ARROW_DROP_DOWN` | Chevron glyph. |
| `hide_expand_icon(bool)` | `bool` | `false` | Hide the chevron. |
| `expanded(bool)` | `bool` | `None` | Seed expansion state. |
| `on_expanded_changed(f)` | `FnMut(bool)` | `None` | Toggle callback. |
| `disabled(bool)` | `bool` | `false` | Disable the group header. |
| `max_height(f32)` | `f32` | `None` | Scrollable children cap. |
| `ripple(bool)` | `bool` | `true` | Stored, not rendered. |
| `link(NavLink)` | `NavLink<'a>` | — | Add a child link. |
| `text_link(href, text)` | `(impl Into<String>, impl Into<String>)` | — | Convenience link. |
| `group(NavGroup)` | `NavGroup<'a>` | — | Add a nested group. |
| `text_group(title)` | `impl Into<String>` | — | Empty nested group. |

---

## API reference — `NavLink`

| Method | Type | Default | Description |
|--------|------|---------|-------------|
| `new(href, text)` | `(impl Into<String>, impl Into<String>)` | — | Full constructor. |
| `label(text)` | `impl Into<String>` | — | No href; fires `on_click`. |
| `href(s)` | `impl Into<String>` | `None` | Navigation URL. |
| `text(s)` | `impl Into<String>` | `None` | Text label. |
| `child(f)` | `FnMut(&mut Ui, bool)` | `None` | Custom label with `is_active`. |
| `icon(s)` | `impl Into<String>` | `None` | Leading icon. |
| `icon_color(Color)` | `Color` | `None` | Icon colour override. |
| `disabled(bool)` | `bool` | `false` | Disable the link. |
| `force_load(bool)` | `bool` | `false` | Open in browser. |
| `target(s)` | `impl Into<String>` | `None` | `_blank` opens new tab. |
| `match_mode(NavLinkMatch)` | `NavLinkMatch` | `Prefix` | Active-detection mode. |
| `ripple(bool)` | `bool` | `true` | Stored, not rendered. |
| `on_click(f)` | `FnMut()` | `None` | Click callback (only when `href` is `None`). |

### `NavLinkMatch`

| Variant | Behaviour |
|---------|-----------|
| `Prefix` (default) | Segment-aware prefix match. `/orders` matches `/orders/42` but not `/orders-history`. |
| `All` | Exact path match, ignoring the current query/fragment. |

---

## API reference — `NavMenuState`

| Field / method | Description |
|----------------|-------------|
| `expanded: HashMap<String, bool>` | Group expansion flags by id. |
| `is_expanded(id)` | `Option<bool>`; `None` if never seeded. |
| `ensure_expanded(id, initial)` | Seeds once. |
| `toggle_expanded(id)` | Flips; returns the new value. |
| `set_expanded(id, value)` | Explicit assignment. |
| `clear()` | Drops all entries. |

---

## Examples

### Basic menu

```rust
let mut state = NavMenuState::new();

MudNavMenu::new().show(ui, nav, &mut state, |menu| {
    menu.text_link("/", "Home");
    menu.text_link("/orders", "Orders");
    menu.text_link("/about", "About");
});
```

### Bordered + rounded with a group

```rust
MudNavMenu::new()
    .bordered(true)
    .rounded(true)
    .color(Color::Primary)
    .show(ui, nav, &mut state, |menu| {
        menu.text_link("/", "Home");
        menu.group(
            NavGroup::new("Settings")
                .icon(filled::SETTINGS)
                .text_link("/profile", "Profile")
                .text_link("/security", "Security"),
        );
    });
```

### Nested groups

```rust
menu.group(
    NavGroup::new("Admin")
        .id("admin")
        .icon(filled::ADMIN_PANEL_SETTINGS)
        .group(
            NavGroup::new("Users")
                .text_link("/admin/users", "List")
                .text_link("/admin/users/new", "New"),
        )
        .group(
            NavGroup::new("Roles")
                .text_link("/admin/roles", "List"),
        ),
);
```

### Expanded by default

```rust
menu.group(
    NavGroup::new("Favourites")
        .expanded(true)
        .text_link("/fav/1", "First"),
);
```

### Custom label with active flag

```rust
menu.link(
    NavLink::new("/orders", "")
        .child(|ui, is_active| {
            let color = if is_active { Color32::GREEN } else { Color32::GRAY };
            ui.add(MudText::new("Orders").custom_color(color));
        }),
);
```

### Force-load a relative URL in the browser

```rust
menu.link(NavLink::new("/legacy/page", "Open in browser").force_load(true));
```

### External link (opens system browser)

```rust
menu.link(NavLink::new("https://docs.rs", "Docs").target("_blank"));
```

### Disabled group / disabled link

```rust
menu.group(NavGroup::new("Deprecated").disabled(true).text_link("/x", "X"));
menu.link(NavLink::new("/restricted", "Restricted").disabled(true));
```

### Click callback (no href)

```rust
menu.link(
    NavLink::label("Open dialog")
        .on_click(|| self.dialog_open = true),
);
```

### Group toggle callback

```rust
menu.group(
    NavGroup::new("Settings")
        .on_expanded_changed(|open| log::info!("settings open: {open}"))
        .text_link("/profile", "Profile"),
);
```

### Scrolling group

```rust
menu.group(
    NavGroup::new("Recent")
        .max_height(200.0)
        .text_link("/r/1", "Item 1")
        /* … many items … */,
);
```

---

## Active detection

A link is active when `is_link_active(href, nav.current_relative(), mode)`
returns `true`.

### Prefix (default)

```
href     = "/orders"
current  = "/orders/42"     → active
current  = "/orders"        → active
current  = "/orders/"       → active (trailing slash normalised)
current  = "/orders-history" → inactive (substring collision rejected)
current  = "/"              → inactive
current  = "/orders?page=2" → active (query stripped for comparison)
```

### All

```
href     = "/orders"
current  = "/orders"          → active
current  = "/orders?page=2"   → active (query stripped)
current  = "/orders/42"       → inactive (exact match required)
```

### External URLs

Hrefs with any scheme (`https://`, `mailto:`, `tel:`, `ftp:`, `file:`,
`data:`) never match. They open in the system browser instead.

### Current path source

`NavigationManager::current_relative()` returns the path relative to the
base URI, without the query. That's what the menu compares against.

---

## Performance and memory choices

| Choice | Effect |
|--------|--------|
| `is_link_active` is pure | No `egui::Context`; easy to test. |
| `normalize_path` allocates one small `String` per call | Acceptable; called once per rendered link per frame. |
| `NavMenuState` is `HashMap<String, bool>` | O(1) expansion lookup; bounded by group count. |
| `ensure_expanded` seeds once per group | No per-frame re-insertion. |
| All text through `MudText`, icons through `MudIcon` | Theme and typography propagate automatically. |
| Chevron uses `MudIcon::rotation` | Free thanks to `MudIcon`'s transform cache. |
| Group children rendered via `ui.scope` (or `ScrollArea` when `max_height`) | One nested scope per expanded group; no per-child wrapper. |
| Idempotent click handling: single navigation per frame | No duplicate `navigate_to` calls. |

---

## Testing

`cargo test` runs 25+ pure-function tests:

| Area | Tests |
|------|-------|
| `row_height` | dense vs. normal |
| `normalize_path` | strips query/fragment, strips trailing slash, preserves interior |
| `is_link_active` | Prefix exact, segment boundary, substring collision rejected, root matches all, case-insensitive, All exact, All ignores queries, external URLs never match, shorter current rejected |
| `resolve_accent` | Default/Inherit → primary, explicit → palette |
| `target_opens_new_tab` | `_blank` (case-insensitive), other targets, `None` |
| `NavMenuState` | empty, seed once, toggle, set, clear |
| Menu defaults | match MudBlazor |
| Menu builders | every field, margin clamps negative |

Rendering (`show`) needs a live `egui::Context`; the pure layer is the source
of truth.

```bash
cargo test -p mud_egui_components mud_nav_menu
cargo test -p mud_egui_core nav_link_match
```

---

## Differences from MudBlazor

- **`ActiveClass` not ported.** egui has no CSS classes. The `MudNavMenu::color`
  cascade is the active styling; use `NavLink::child` when you need custom
  active-driven rendering.
- **`HeaderClass` not ported** on `MudNavGroup` for the same reason. Use
  `title_content`.
- **`Expanded` is a seed, not a two-way binding.** MudBlazor uses
  `@bind-Expanded`; the port seeds from `NavGroup::expanded(..)` and persists
  in `NavMenuState::expanded`. Update `state` directly when you need to
  control expansion programmatically.
- **Groups are identified by `id`, not instance.** MudBlazor identifies groups
  by their .NET object reference; Rust cannot do that across frames, so the
  port keys by `NavGroup::id` (defaulting to the title). Two groups with the
  same title share expansion state — call `.id(..)` to disambiguate.
- **Active detection uses `current_relative()`** (path only, no query). The
  query/fragment is stripped before comparison.
- **External URLs never match.** MudBlazor's `NavLink` compares the current
  URI, which for external URLs would also be the page's URI — the port
  short-circuits external URLs to `false` so an external link never
  misleadingly highlights.
- **`Ripple` is stored but not rendered.** Same as `MudIcon`; add a ripple
  implementation in a future pass if needed.
- **`Style` / `Tag` / `UserAttributes` not ported.** No CSS strings or HTML
  tags.

===== ./crates/mud_egui_components/src/navigation/pagination/README.md =====

# MudPagination for `mud_egui`

A horizontal row of clickable page numbers with optional first / previous /
next / last navigation controls for [egui](https://github.com/emilk/egui),
ported from MudBlazor's [`MudPagination`](https://mudblazor.com/components/pagination)
(MudBlazor 8.13.0).

```rust
let mut state = PaginationState::new();

MudPagination::new(10)
    .color(Color::Primary)
    .show_first_button(true)
    .show_last_button(true)
    .show(ui, &mut state);
```

Page numbers render through **`MudText`**; control icons and ellipses render
through **`MudIcon`**.

---

## Contents

1. [Design principle: API parity, not API invention](#design-principle-api-parity-not-api-invention)
2. [Architecture](#architecture)
3. [API reference — MudPagination](#api-reference--mudpagination)
4. [API reference — PaginationState](#api-reference--paginationstate)
5. [API reference — PaginationAction & PageControl](#api-reference--paginationaction--pagecontrol)
6. [Examples](#examples)
7. [Layout algorithm](#layout-algorithm)
8. [Performance and memory choices](#performance-and-memory-choices)
9. [Testing](#testing)
10. [Differences from MudBlazor](#differences-from-mudblazor)

---

## Design principle: API parity, not API invention

The MudBlazor 8.13.0 `MudPagination` surface was verified against
[DNDocs](https://docs.dndocs.com/n/MudBlazor/8.13.0/api/MudBlazor.MudPagination.html)
before this refactor.

### Ported parameters

| MudBlazor | Rust method | Default |
|-----------|-------------|---------|
| `BeforeIcon` | `before_icon(s)` | `NAVIGATE_BEFORE` |
| `BoundaryCount` | `boundary_count(u32)` | `1` |
| `Color` | `color(Color)` | `Primary` |
| `ControlButtonClicked` | `on_control_clicked(f)` / `PaginationResponse.action` | — |
| `Count` | `new(n)` / `count(u32)` | `1` |
| `Disabled` | `disabled(bool)` | `false` |
| `DropShadow` | `drop_shadow(bool)` | `true` |
| `FirstIcon` | `first_icon(s)` | `FIRST_PAGE` |
| `LastIcon` | `last_icon(s)` | `LAST_PAGE` |
| `MiddleCount` | `middle_count(u32)` | `1` |
| `NextIcon` | `next_icon(s)` | `NAVIGATE_NEXT` |
| `Rectangular` | `rectangular(bool)` | `false` |
| `RightToLeft` | `right_to_left(bool)` | `false` |
| `Selected` | `PaginationState::selected` | `1` |
| `SelectedChanged` | `on_selected_changed(f)` / `PaginationResponse.selected` | — |
| `ShowFirstButton` | `show_first_button(bool)` | `false` |
| `ShowLastButton` | `show_last_button(bool)` | `false` |
| `ShowNextButton` | `show_next_button(bool)` | `true` |
| `ShowPageButtons` | `show_page_buttons(bool)` | `true` |
| `ShowPreviousButton` | `show_previous_button(bool)` | `true` |
| `Size` | `size(Size)` | `Medium` |
| `Variant` | `variant(Variant)` | `Text` |
| `Class` (base) | `classes(iter)` | `None` |
| `Style` (base) | *(use `classes`)* | — |
| `Tag` (base) | **not ported** (no HTML tag) | — |
| `UserAttributes` | **not ported** | — |

### Extensions (not in MudBlazor)

| Extension | Description |
|-----------|-------------|
| `id_salt(s)` | Stable interaction ids. |
| `PaginationState` | Caller-owned selection state. |
| `PaginationResponse` | Rich return value with `selected` and `action`. |
| `paginate_layout` | Pure layout function (internal, unit-tested). |

### Deliberately not ported

- **`Tag` / `UserAttributes`** — no HTML tag.
- **`Style`** — no inline CSS strings; use `classes(..)`.
- **`NavigateToAsync(Page)` / `NavigateToAsync(int)`** — the caller owns
  `PaginationState`, so navigation is a direct field write. See the
  [0-based vs 1-based note](#api-reference--paginationstate).

### Note on `Page` → `PageControl`

MudBlazor's `Page` enum (`First` / `Previous` / `Next` / `Last`) is ported as
`PageControl` because `Page` is already the trait name in
`mud_egui_navigation::page`. The variants and semantics are identical.

---

## Architecture

```
pagination/
├── paginate_item.rs       — pub(crate) PaginateItem (Page | Ellipsis)
├── pagination_action.rs   — pub PaginationAction (None | Page | Control)
├── pagination_state.rs    — pub PaginationState { selected }
├── mud_pagination.rs      — MudPagination + PaginationResponse + pure helpers
└── mod.rs

mud_egui_core/src/enums/
└── page_control.rs        — pub PageControl (First | Previous | Next | Last)
```

**Pure layer.** `paginate_layout`, `button_size`, `icon_size`, `font_size`,
`resolve_color`, `resolve_variant` are free functions or `pub(crate)`
helpers, unit-tested without `egui::Context`.

**State layer.** `PaginationState { selected: u32 }` is caller-owned. This is
the egui equivalent of MudBlazor's bindable `Selected` parameter.

**Component layer.** `show` resolves the theme and frame, computes the layout,
renders the row (respecting `right_to_left`), applies the click to `state`,
and returns a `PaginationResponse`.

---

## API reference — `MudPagination`

| Method | Type | Default | Description |
|--------|------|---------|-------------|
| `new(count)` | `u32` | — | Total page count (clamped to ≥ 1). |
| `first_icon(s)` | `impl Into<String>` | `FIRST_PAGE` | First-page icon. |
| `before_icon(s)` | `impl Into<String>` | `NAVIGATE_BEFORE` | Previous-page icon. |
| `next_icon(s)` | `impl Into<String>` | `NAVIGATE_NEXT` | Next-page icon. |
| `last_icon(s)` | `impl Into<String>` | `LAST_PAGE` | Last-page icon. |
| `boundary_count(n)` | `u32` | `1` | Pages always shown at each end. |
| `middle_count(n)` | `u32` | `1` | Pages shown around the selected one. |
| `rectangular(b)` | `bool` | `false` | Square buttons instead of round. |
| `drop_shadow(b)` | `bool` | `true` | Shadow behind buttons. |
| `right_to_left(b)` | `bool` | `false` | RTL layout. |
| `show_first_button(b)` | `bool` | `false` | Show the "first page" control. |
| `show_last_button(b)` | `bool` | `false` | Show the "last page" control. |
| `show_next_button(b)` | `bool` | `true` | Show the "next page" control. |
| `show_previous_button(b)` | `bool` | `true` | Show the "previous page" control. |
| `show_page_buttons(b)` | `bool` | `true` | Show numeric page buttons. |
| `count(n)` | `u32` | `1` | Total pages (clamped to ≥ 1). |
| `disabled(b)` | `bool` | `false` | Prevents all clicks. |
| `color(c)` | `Color` | `Primary` | Accent colour. |
| `size(s)` | `Size` | `Medium` | Button size. |
| `variant(v)` | `Variant` | `Text` | Variant for non-selected buttons. |
| `id_salt(s)` | `impl Into<String>` | `None` | Stable interaction ids. |
| `on_selected_changed(f)` | `FnMut(u32)` | `None` | Fired when selection changes. |
| `on_control_clicked(f)` | `FnMut(PageControl)` | `None` | Fired on control click. |
| `classes(iter)` | `iter of AsRef<str>` | none | Wrapper utility classes. |
| `flavor(f)` | `StyleFlavor` | `Bootstrap` | Utility resolver. |
| `show(ui, state)` | `(&mut Ui, &mut PaginationState)` | — | Returns `PaginationResponse`. |

### `PaginationResponse`

| Field | Type | Description |
|-------|------|-------------|
| `response` | `egui::Response` | The wrapper's response. |
| `selected` | `u32` | Page after the click was applied (1-based). |
| `action` | `PaginationAction` | What was clicked this frame. |

---

## API reference — `PaginationState`

| Field / method | Description |
|----------------|-------------|
| `selected: u32` | Currently selected page (1-based). |
| `new()` | `selected = 1`. |
| `with_selected(n)` | `selected = max(n, 1)`. |
| `navigate_to(page, count)` | Clamps into `1..=count`; returns `true` on change. |
| `next(count)` | `selected + 1`, clamped at `count`. |
| `previous(count)` | `selected - 1`, clamped at `1`. |

### 0-based vs 1-based

MudBlazor has two `NavigateToAsync` overloads:

- `NavigateToAsync(Page page)` — takes the `Page` enum.
- `NavigateToAsync(int pageIndex)` — takes a **0-based** index.

The port only exposes 1-based methods because the `Selected` parameter
itself is 1-based, and keeping the state model uniform is more important
than mirroring the dual overload. Callers who want to mirror the 0-based API
can write `state.selected = index + 1`.

---

## API reference — `PaginationAction` & `PageControl`

### `PaginationAction`

| Variant | Description |
|---------|-------------|
| `None` | No click this frame. |
| `Page(u32)` | A page number button was clicked (1-based). |
| `Control(PageControl)` | A first/previous/next/last control was clicked. |

Helpers: `is_change()`, `is_control()`, `target_page()` (only for `Page(_)`).

### `PageControl`

| Variant | Description |
|---------|-------------|
| `First` | First-page control. |
| `Previous` | Previous-page control. |
| `Next` | Next-page control. |
| `Last` | Last-page control. |

Renamed from MudBlazor's `Page` to avoid clashing with
`mud_egui_navigation::page::Page`.

---

## Examples

### Basic pagination

```rust
let mut state = PaginationState::new();

MudPagination::new(10).show(ui, &mut state);
// Clicking page 5 → state.selected == 5
```

### With first/last controls

```rust
MudPagination::new(20)
    .show_first_button(true)
    .show_last_button(true)
    .show(ui, &mut state);
```

### Larger boundary and middle ranges

```rust
MudPagination::new(50)
    .boundary_count(2)
    .middle_count(3)
    .show(ui, &mut state);
// → 1 2 ... 24 25 26 ... 49 50
```

### Rectangular variant

```rust
MudPagination::new(10)
    .rectangular(true)
    .show(ui, &mut state);
```

### Filled variant

```rust
MudPagination::new(5)
    .variant(Variant::Filled)
    .show(ui, &mut state);
// All pages rendered as solid coloured buttons.
```

### Outlined variant

```rust
MudPagination::new(5)
    .variant(Variant::Outlined)
    .show(ui, &mut state);
// Non-selected pages outlined; selected page filled.
```

### Disabled

```rust
MudPagination::new(5)
    .disabled(true)
    .show(ui, &mut state);
// No button responds.
```

### Without page buttons (only controls)

```rust
MudPagination::new(10)
    .show_page_buttons(false)
    .show_first_button(true)
    .show_last_button(true)
    .show(ui, &mut state);
// Only ⏮ ◀ ▶ ⏭ are rendered.
```

### RTL

```rust
MudPagination::new(10)
    .right_to_left(true)
    .show(ui, &mut state);
// The row is laid out right-to-left, so the "first" control appears on
// the right of the row.
```

### Reading the outcome

```rust
let resp = MudPagination::new(10).show(ui, &mut state);
match resp.action {
    PaginationAction::Page(n) => log::info!("jumped to page {n}"),
    PaginationAction::Control(PageControl::Next) => log::info!("next"),
    PaginationAction::Control(PageControl::Last) => log::info!("last"),
    PaginationAction::Control(PageControl::First) => log::info!("first"),
    PaginationAction::Control(PageControl::Previous) => log::info!("previous"),
    PaginationAction::None => {}
}
```

### Callbacks

```rust
MudPagination::new(10)
    .on_selected_changed(|page| log::info!("page changed to {page}"))
    .on_control_clicked(|ctl| log::info!("control clicked: {ctl:?}"))
    .show(ui, &mut state);
```

### Programmatic navigation

```rust
// In response to an external event:
state.navigate_to(5, 10);  // clamps to 5
state.next(10);            // 6
state.previous(10);        // 5
```

### Overriding icons

```rust
MudPagination::new(10)
    .first_icon(filled::KEYBOARD_DOUBLE_ARROW_LEFT)
    .before_icon(filled::KEYBOARD_ARROW_LEFT)
    .next_icon(filled::KEYBOARD_ARROW_RIGHT)
    .last_icon(filled::KEYBOARD_DOUBLE_ARROW_RIGHT)
    .show(ui, &mut state);
```

---

## Layout algorithm

`paginate_layout(count, boundary_count, middle_count, selected)` returns a
`Vec` of `PaginateItem`s:

- The first `boundary_count` pages are always shown.
- The last `boundary_count` pages are always shown.
- The `middle_count` pages centred on `selected` are always shown
  (`middle_count >= 1` so the selected page is never hidden).
- Any gap of one or more hidden pages becomes one `Ellipsis`.

### Truth table

| `count` | `boundary` | `middle` | `selected` | Result |
|---------|------------|----------|------------|--------|
| 9 | 1 | 1 | 5 | `1 … 5 … 9` |
| 9 | 1 | 3 | 5 | `1 … 4 5 6 … 9` |
| 9 | 2 | 1 | 5 | `1 2 … 5 … 8 9` |
| 9 | 2 | 3 | 5 | `1 2 … 4 5 6 … 8 9` |
| 5 | 2 | 1 | 3 | `1 2 3 4 5` (contiguous) |
| 5 | 1 | 1 | 3 | `1 … 3 … 5` |
| 1 | 1 | 1 | 1 | `1` |
| 0 | 1 | 1 | 1 | `1` (count clamped) |
| 9 | 0 | 1 | 5 | `… 5` (leading ellipsis; no trailing) |

### Clamping

- `count < 1` → treated as `1`.
- `selected < 1` → clamped to `1`.
- `selected > count` → clamped to `count`.
- `middle_count < 1` → treated as `1` (guarantees the selected page is shown).

### Ellipsis semantics

An ellipsis is emitted **before** the next shown page, so a **trailing**
ellipsis is never produced (there is always a page after it). A **leading**
ellipsis can appear when `boundary_count = 0` and the selected page's middle
range does not start at page 1.

---

## Performance and memory choices

| Choice | Effect |
|--------|--------|
| `paginate_layout` is a pure function | No `egui::Context`; unit-tested. |
| `PaginateItem` is `Copy` | No per-item allocations. |
| Layout returns at most `count` items | Bounded by `count`, typically ≤ 20. |
| Page number rendering through `MudText` | Theme and typography propagate automatically. |
| Control icons through `MudIcon` | Free thanks to the transform cache for repeated icons. |
| `PaginationState` is `Copy` | Zero-cost snapshot; passed as `&mut`. |
| `PaginationAction` is `Copy` | Returned by value; easy to test. |
| Single-pass row layout | No second pass; all buttons are allocated as they render. |
| Callbacks are `Option<Box<dyn FnMut>>` | Zero-cost when `None`. |

---

## Testing

`cargo test` runs 20+ pure-function tests:

| Area | Tests |
|------|-------|
| `button_size` / `icon_size` / `font_size` | every `Size` |
| `resolve_color` | `Default`/`Inherit` → primary, explicit → palette |
| `resolve_variant` | selected always `Filled`, unselected uses base |
| `paginate_layout` | small count, boundary/middle truth table, clamping, ellipsis rules, `middle_count = 0`, `boundary_count = 0` |
| `PaginationState` | default, `with_selected`, `navigate_to`, `next`, `previous`, count-0 handling |
| `PaginationAction` | every variant, `is_change`, `is_control`, `target_page` |
| Defaults | match MudBlazor |
| Builders | every field, `count` clamped to ≥ 1 |

Rendering (`show`) needs a live `egui::Context`; the pure layer is the source
of truth.

```bash
cargo test -p mud_egui_components mud_pagination
cargo test -p mud_egui_core page_control
```

---

## Differences from MudBlazor

- **`Page` → `PageControl`.** Renamed to avoid clashing with
  `mud_egui_navigation::page::Page`.
- **State is caller-owned (`PaginationState`).** MudBlazor uses the bindable
  `Selected` parameter; the egui equivalent is a struct the caller owns and
  passes by `&mut`.
- **`NavigateToAsync` not ported.** Direct field writes on `PaginationState`
  are the equivalent. Only 1-based methods are exposed.
- **`Style` / `Tag` / `UserAttributes` not ported.** No CSS strings or HTML
  tags.
- **`Variant::Filled` quirk preserved.** In MudBlazor, `Variant::Filled`
  makes all page buttons solid (the selected one is still filled with the
  accent, but the non-selected pages look identical). The port reproduces
  this exactly; use `Variant::Text` or `Variant::Outlined` to get the
  familiar "only the selected page is filled" look.
- **`RightToLeft` is a normal parameter.** MudBlazor uses a cascading
  parameter; the port takes it directly and lays the row out RTL via
  `egui::Layout::right_to_left`.

===== ./crates/mud_egui_components/src/navigation/stepper/README.md =====

# MudStepper for `mud_egui`

A wizard that guides the user through a series of steps, for
[egui](https://github.com/emilk/egui). Ported from MudBlazor's
[`MudStepper`](https://mudblazor.com/components/stepper) and
[`MudStep`](https://mudblazor.com/components/stepper) (MudBlazor 8.13.0).

```rust
let mut state = StepperState::new();

MudStepper::new()
    .vertical(true)
    .show_reset_button(true)
    .show(ui, &mut state, |s| {
        s.step(MudStep::new("Account").child(|ui| {
            ui.label("Account details…");
        }));
        s.step(MudStep::new("Profile").skippable(true).child(|ui| {
            ui.label("Profile…");
        }));
        s.step(MudStep::new("Confirm").child(|ui| {
            ui.label("Confirm…");
        }));
    });
```

Every label renders through **`MudText`**, every glyph through **`MudIcon`**,
every nav button through **`MudButton`**.

---

## Contents

1. [Design principle: API parity, not API invention](#design-principle-api-parity-not-api-invention)
2. [Architecture](#architecture)
3. [API reference — MudStepper](#api-reference--mudstepper)
4. [API reference — MudStep](#api-reference--mudstep)
5. [API reference — StepperState](#api-reference--stepperstate)
6. [API reference — supporting types](#api-reference--supporting-types)
7. [Examples](#examples)
8. [Stepper phases](#stepper-phases)
9. [Linear vs. non-linear](#linear-vs-non-linear)
10. [Performance and memory choices](#performance--memory-choices)
11. [Testing](#testing)
12. [Differences from MudBlazor](#differences-from-mudblazor)

---

## Design principle: API parity, not API invention

The MudBlazor 8.13.0 parameter surface for `MudStepper`, `MudStep`,
`StepAction`, and `StepperInteractionEventArgs` was verified against DNDocs
before this port.

### Ported `MudStepper` parameters

| MudBlazor | Rust method | Default |
|-----------|-------------|---------|
| `ActionContent` | `action_content(f)` | `None` |
| `ActiveIndex` | `StepperState::active_index` | `0` |
| `ActiveIndexChanged` | `on_active_index_changed(f)` | `None` |
| `CenterLabels` | `center_labels(bool)` | `false` |
| `ChildContent` | closure to `show` | required |
| `CompleteButtonIcon` | `complete_button_icon(s)` | `DONE` |
| `CompletedContent` | `completed_content(f)` | `None` |
| `CompletedStepColor` | `completed_step_color(Color)` | `Primary` |
| `ConnectorTemplate` | `connector_template(f)` | `None` |
| `CurrentStepColor` | `current_step_color(Color)` | `Primary` |
| `ErrorStepColor` | `error_step_color(Color)` | `Error` |
| `LabelTemplate` | `label_template(f)` | `None` |
| `NextButtonIcon` | `next_button_icon(s)` | `NAVIGATE_NEXT` |
| `NonLinear` | `non_linear(bool)` | `false` |
| `OnPreviewInteraction` | `on_preview_interaction(f)` | `None` |
| `PreviousButtonIcon` | `previous_button_icon(s)` | `NAVIGATE_BEFORE` |
| `ResetButtonIcon` | `reset_button_icon(s)` | `FIRST_PAGE` |
| `Ripple` | `ripple(bool)` | `false` |
| `ScrollableNavigation` | `scrollable_navigation(bool)` | `true` |
| `ShowResetButton` | `show_reset_button(bool)` | `false` |
| `SkipButtonIcon` | `skip_button_icon(s)` | `SKIP_NEXT` |
| `StepCompleteIcon` | `step_complete_icon(s)` | `DONE` |
| `StepErrorIcon` | `step_error_icon(s)` | `PRIORITY_HIGH` |
| `TitleTemplate` | `title_template(f)` | `None` |
| `Vertical` | `vertical(bool)` | `false` |
| `Class` (base) | `classes(iter)` | `None` |
| `Style` (base) | *(use `classes`)* | — |
| `Tag` (base) | **not ported** (no HTML tag) | — |
| `UserAttributes` | **not ported** | — |

### Ported `MudStep` parameters

| MudBlazor | Rust method | Default |
|-----------|-------------|---------|
| `ChildContent` | `content(f)` / `child(f)` | `None` |
| `Completed` | `completed(bool)` (seed) | `false` |
| `CompletedChanged` | *(state is authoritative)* | — |
| `CompletedStepColor` | `completed_step_color(Color)` | `None` (inherit) |
| `Disabled` | `disabled(bool)` | `false` |
| `DisabledChanged` | *(rebuild next frame)* | — |
| `ErrorStepColor` | `error_step_color(Color)` | `None` (inherit) |
| `HasError` | `has_error(bool)` (seed) | `false` |
| `HasErrorChanged` | *(state is authoritative)* | — |
| `IsActive` | `StepperState::active_index` | — |
| `OnClick` | `on_click(f)` | `None` |
| `SecondaryText` | `secondary_text(s)` | `None` |
| `Skippable` | `skippable(bool)` | `false` |
| `Skipped` | `skipped(bool)` (seed) | `false` |
| `SkippedChanged` | *(state is authoritative)* | — |
| `SkippedStepColor` | `skipped_step_color(Color)` | `None` (inherit) |
| `Title` | `new(s)` / `title(s)` | `None` |

### Extensions (not in MudBlazor)

| Extension | Description |
|-----------|-------------|
| `StepperState` | Caller-owned active index + per-step flags. |
| `StepperResponse` | Rich return value with `action` and `active_index`. |
| `StepperAction` | Reports what was clicked this frame. |
| `StepPhase` | Resolved visual phase, passed to templates. |
| `StepFlags` | Runtime `completed` / `has_error` / `skipped` per step. |

### Deliberately not ported

- **`Steps` / `ActiveStep`** (get-only) — steps are single-use per frame. Use
  `StepperResponse::active_index` and the count you passed in.
- **`CanGoToNextStep` / `IsCompleted` / `IsCurrentStepSkippable` /
  `PreviousStepEnabled` / `ShowCompleteInsteadOfNext`** (get-only) — exposed
  as pure helper functions (`can_go_to_next`, `is_completed`,
  `is_current_skippable`, `previous_step_enabled`,
  `show_complete_instead_of_next`) so you can call them with the state you
  already own.
- **`SetCompletedAsync` / `SetHasErrorAsync` / `SetSkippedAsync`** — use
  `StepperState::set_completed` etc.
- **`SetDisabledAsync`** — rebuild the step with `.disabled(..)` next frame.
- **`Tag` / `UserAttributes`** — no HTML tag.
- **`Style`** — no inline CSS strings; use `classes(..)`.

---

## Architecture

```
stepper/
├── mud_step.rs             — MudStep builder
├── mud_stepper.rs          — MudStepper + StepperResponse + StepperUi + renderers + tests
├── step_flags.rs           — pub StepFlags
├── step_phase.rs           — pub StepPhase (extension)
├── stepper_action.rs       — pub StepperAction (extension)
├── stepper_interaction.rs  — pub StepperInteraction (port of StepperInteractionEventArgs)
├── stepper_state.rs        — pub StepperState
└── mod.rs

mud_egui_core/src/enums/
└── step_action.rs          — pub StepAction
```

**Pure layer.** `step_diameter`, `step_phase`, `is_completed`,
`show_complete_instead_of_next`, `can_go_to_next`, `previous_step_enabled`,
`is_current_skippable` are free functions, unit-tested without `egui::Context`.

**State layer.** `StepperState { active_index: usize, steps: Vec<StepFlags> }`
is caller-owned. It is the egui equivalent of MudBlazor's bindable
`ActiveIndex` plus the per-step `Completed` / `HasError` / `Skipped`
parameters.

**Component layer.** `show` collects `MudStep`s, seeds the state, renders the
header (horizontal or vertical), the active step's content (or
`CompletedContent`), and the nav row (or `ActionContent`).

---

## API reference — `MudStepper`

| Method | Type | Default | Description |
|--------|------|---------|-------------|
| `new()` | — | — | Default stepper. |
| `non_linear(bool)` | `bool` | `false` | Allow arbitrary step jumps. |
| `vertical(bool)` | `bool` | `false` | Vertical layout. |
| `center_labels(bool)` | `bool` | `false` | Centre labels under the circles. |
| `scrollable_navigation(bool)` | `bool` | `true` | Scrollable vertical header. |
| `show_reset_button(bool)` | `bool` | `false` | Show the Reset button. |
| `ripple(bool)` | `bool` | `false` | Ripple on step click. |
| `completed_step_color(Color)` | `Color` | `Primary` | Completed-step accent. |
| `current_step_color(Color)` | `Color` | `Primary` | Active-step accent. |
| `error_step_color(Color)` | `Color` | `Error` | Error-step accent. |
| `step_complete_icon(s)` | `impl Into<String>` | `DONE` | Completed glyph. |
| `step_error_icon(s)` | `impl Into<String>` | `PRIORITY_HIGH` | Error glyph. |
| `skip_button_icon(s)` | `impl Into<String>` | `SKIP_NEXT` | Skipped glyph. |
| `previous_button_icon(s)` | `impl Into<String>` | `NAVIGATE_BEFORE` | Previous button glyph. |
| `next_button_icon(s)` | `impl Into<String>` | `NAVIGATE_NEXT` | Next button glyph. |
| `complete_button_icon(s)` | `impl Into<String>` | `DONE` | Complete button glyph. |
| `reset_button_icon(s)` | `impl Into<String>` | `FIRST_PAGE` | Reset button glyph. |
| `label_template(f)` | `for<'v> FnMut(&mut Ui, &StepView<'v>)` | `None` | Custom circle renderer. |
| `title_template(f)` | `for<'v> FnMut(&mut Ui, &StepView<'v>)` | `None` | Custom title renderer. |
| `connector_template(f)` | `FnMut(&mut Ui, bool)` | `None` | Custom connector renderer. |
| `action_content(f)` | `FnMut(&mut Ui, &mut StepperState)` | `None` | Custom nav row. |
| `completed_content(f)` | `FnMut(&mut Ui)` | `None` | Shown when all steps complete. |
| `on_active_index_changed(f)` | `FnMut(usize)` | `None` | Fires when the active index changes. |
| `on_preview_interaction(f)` | `FnMut(&mut StepperInteraction)` | `None` | May cancel a step change. |
| `classes(iter)` | `iter of AsRef<str>` | none | Wrapper classes. |
| `flavor(StyleFlavor)` | `StyleFlavor` | `Bootstrap` | Utility resolver. |
| `show(ui, state, \|s\| ..)` | `(&mut Ui, &mut StepperState, FnOnce(&mut StepperUi) -> R)` | — | Returns `(StepperResponse, R)`. |

### `StepperResponse`

| Field | Type | Description |
|-------|------|-------------|
| `response` | `egui::Response` | The wrapper's response. |
| `active_index` | `usize` | Active index after the click was applied. |
| `action` | `StepperAction` | What was clicked this frame. |

---

## API reference — `MudStep`

| Method | Type | Default | Description |
|--------|------|---------|-------------|
| `new(title)` | `impl Into<String>` | — | Step with title, no content. |
| `content(f)` | `FnMut(&mut Ui)` | `None` | Step with content, no title. |
| `title(s)` | `impl Into<String>` | `None` | Replace title. |
| `secondary_text(s)` | `impl Into<String>` | `None` | Subtitle. |
| `child(f)` | `FnMut(&mut Ui)` | `None` | Step body. |
| `completed(bool)` | `bool` | `None` (seed) | Seed `Completed`. |
| `has_error(bool)` | `bool` | `None` (seed) | Seed `HasError`. |
| `skipped(bool)` | `bool` | `None` (seed) | Seed `Skipped`. |
| `disabled(bool)` | `bool` | `false` | Disable the header. |
| `skippable(bool)` | `bool` | `false` | Allow skipping. |
| `completed_step_color(Color)` | `Color` | `None` | Per-step completed override. |
| `error_step_color(Color)` | `Color` | `None` | Per-step error override. |
| `skipped_step_color(Color)` | `Color` | `None` | Per-step skipped override. |
| `on_click(f)` | `FnMut()` | `None` | Header click callback. |

---

## API reference — `StepperState`

| Field / method | Description |
|----------------|-------------|
| `active_index: usize` | 0-based active step. |
| `steps: Vec<StepFlags>` | Per-step runtime flags. |
| `ensure_step(index, completed, has_error, skipped)` | Seeds flags on first render. |
| `flags(index)` | `StepFlags` for a step, or `default`. |
| `flags_mut(index)` | `&mut StepFlags`, growing the Vec. |
| `set_completed / set_has_error / set_skipped(index, v)` | Mutators. |
| `is_completed(count)` | `true` when every step is completed. |
| `reset()` | Clears flags and sets `active_index = 0`. |

---

## API reference — supporting types

### `StepAction`

| Variant | Discriminant | Meaning |
|---------|--------------|---------|
| `Activate` | 0 | Activate a step (Previous / non-linear click). |
| `Complete` | 1 | Complete the current step and advance. |
| `Skip` | 2 | Skip the current step and advance. |
| `Reset` | 3 | Reset to the first step. |

### `StepPhase`

| Variant | Meaning |
|---------|---------|
| `Pending` | Not yet reached. |
| `Active` | The current step. |
| `Completed` | Completed successfully. |
| `Error` | Has an error. |
| `Skipped` | Skipped by the user. |

Priority: `Error > Skipped > Completed > Active > Pending`.

### `StepperAction`

| Variant | Description |
|---------|-------------|
| `None` | Nothing clicked. |
| `Activated { from, to }` | A step became active (Previous or non-linear click). |
| `Completed { index, to }` | The active step was completed and the stepper advanced. |
| `Skipped { index, to }` | The active step was skipped and the stepper advanced. |
| `Reset` | The stepper was reset. |

Helpers: `changed_active_index()`, `advanced()`, `trigger_index()`,
`new_active_index()`.

### `StepperInteraction`

| Field / method | Description |
|----------------|-------------|
| `step_index: usize` | Target step. |
| `action: StepAction` | Requested action. |
| `cancel: bool` | Set to prevent the change. |
| `prevent()` | Sets `cancel = true`. |
| `is_prevented()` | `cancel` |

### `StepFlags`

| Field | Description |
|-------|-------------|
| `completed: bool` | `MudStep.Completed`. |
| `has_error: bool` | `MudStep.HasError`. |
| `skipped: bool` | `MudStep.Skipped`. |

### `StepView<'s>`

Read-only snapshot passed to `label_template` / `title_template`:

`index`, `title`, `secondary_text`, `is_active`, `completed`, `has_error`,
`skipped`, `disabled`, `skippable`, `phase`.

---

## Examples

### Basic horizontal stepper

```rust
let mut state = StepperState::new();

MudStepper::new().show(ui, &mut state, |s| {
    s.step(MudStep::new("Account").child(|ui| {
        ui.label("Account details…");
    }));
    s.step(MudStep::new("Profile").child(|ui| {
        ui.label("Profile…");
    }));
    s.step(MudStep::new("Confirm").child(|ui| {
        ui.label("Confirm…");
    }));
});
```

### Vertical stepper

```rust
MudStepper::new().vertical(true).show(ui, &mut state, |s| {
    s.text_step("Step 1");
    s.text_step("Step 2");
    s.text_step("Step 3");
});
```

### Non-linear (arbitrary step jumps)

```rust
MudStepper::new().non_linear(true).show(ui, &mut state, |s| {
    s.text_step("A");
    s.text_step("B");
    s.text_step("C");
});
```

### Reset button

```rust
MudStepper::new().show_reset_button(true).show(ui, &mut state, |s| {
    s.text_step("A");
    s.text_step("B");
});
```

### Skippable step

```rust
MudStepper::new().show(ui, &mut state, |s| {
    s.step(MudStep::new("Required"));
    s.step(MudStep::new("Optional").skippable(true));
    s.step(MudStep::new("Done"));
});
```

### Seeding completed / error / skipped

```rust
MudStepper::new().show(ui, &mut state, |s| {
    s.step(MudStep::new("Done").completed(true));
    s.step(MudStep::new("Broken").has_error(true));
    s.step(MudStep::new("Skipped").skipped(true));
    s.step(MudStep::new("Pending"));
});
```

After the first render, `state.steps[i]` is authoritative — mutate it with
`state.set_completed(i, true)` etc.

### Preview interaction (veto a step change)

```rust
MudStepper::new()
    .on_preview_interaction(|i| {
        if i.action == StepAction::Complete && !form_is_valid() {
            i.prevent();
        }
    })
    .show(ui, &mut state, |s| { /* … */ });
```

### Completed content

```rust
MudStepper::new()
    .completed_content(|ui| {
        ui.add(MudText::new("All done!").typo(Typo::H3));
    })
    .show(ui, &mut state, |s| {
        s.step(MudStep::new("A"));
        s.step(MudStep::new("B"));
    });
```

Shown instead of the active step when every step is completed.

### Custom action row (icon-only, MudBlazor style)

```rust
MudStepper::new()
    .action_content(|ui, state| {
        ui.horizontal(|ui| {
            if MudButton::new("").start_icon(filled::NAVIGATE_BEFORE).show(ui).clicked() {
                state.previous_step();
            }
            if MudButton::new("").start_icon(filled::NAVIGATE_NEXT).show(ui).clicked() {
                state.next_step();
            }
        });
    })
    .show(ui, &mut state, |s| { /* … */ });
```

### Custom label template

```rust
MudStepper::new()
    .label_template(|ui, view| {
        let colour = if view.is_active { Color32::GREEN } else { Color32::GRAY };
        ui.add(MudText::new((view.index + 1).to_string()).custom_color(colour));
    })
    .show(ui, &mut state, |s| { /* … */ });
```

### Per-step colours

```rust
s.step(MudStep::new("A").completed(true).completed_step_color(Color::Success));
s.step(MudStep::new("B").has_error(true).error_step_color(Color::Warning));
s.step(MudStep::new("C").skipped(true).skipped_step_color(Color::Info));
```

### Reading the outcome

```rust
let (resp, _) = MudStepper::new().show(ui, &mut state, |s| { /* … */ });
match resp.action {
    StepperAction::Activated { from, to } => log::info!("{from} → {to}"),
    StepperAction::Completed { index, to } => log::info!("completed {index} → {to}"),
    StepperAction::Skipped { index, to } => log::info!("skipped {index} → {to}"),
    StepperAction::Reset => log::info!("reset"),
    StepperAction::None => {}
}
```

### Controlled navigation (custom Next button)

```rust
let can_advance = {
    let flags = state.flags(state.active_index);
    can_go_to_next(&flags, /* non_linear = */ false, /* is_last = */ false)
};
ui.add_enabled(can_advance, egui::Button::new("Continue"));
```

---

## Stepper phases

The circle's appearance is driven by [`StepPhase`](#stepphase). The colour is
resolved from the stepper's `completed_step_color` / `current_step_color` /
`error_step_color`, with per-step overrides taking precedence.

| Phase | Circle fill | Content |
|-------|-------------|---------|
| `Pending` | Surface + divider stroke | Step index |
| `Active` | `current_step_color` + focus ring | Step index |
| `Completed` | `completed_step_color` | `step_complete_icon` |
| `Error` | `error_step_color` | `step_error_icon` |
| `Skipped` | `text_secondary` | `skip_button_icon` |

Per-step colour overrides (`completed_step_color` etc. on `MudStep`) are
currently used as a fallback when the stepper has no global colour. The
stepper's colours always win — this matches MudBlazor's "the stepper's
parameter is the default, the step's parameter overrides" and is documented
here because the override chain was the only place MudBlazor and this port
could diverge in appearance.

---

## Linear vs. non-linear

| | Linear (`non_linear = false`) | Non-linear (`non_linear = true`) |
|---|---|---|
| Click a step header | No-op | Activates that step (subject to `on_preview_interaction`) |
| Previous button | Enabled when `active_index > 0` | Same |
| Next button | Calls `NextStepAsync()` (completes and advances) | Same |
| Skip button | Shown only for `skippable` steps | Same |
| `Ripple` | Ignored | Ignored (the port does not animate ripple) |

In both modes, `on_preview_interaction` can veto any change. The interaction
is always created with the **target** step index (so a veto handler can
inspect it) and the appropriate `StepAction`.

---

## Performance and memory choices

| Choice | Effect |
|--------|--------|
| Pure `step_phase` / `is_completed` / `can_go_to_next` etc. | No `egui::Context`; tested. |
| `StepperState` is caller-owned | Selection and flags persist across frames without callbacks. |
| `StepFlags` is `Copy` | No per-frame allocations. |
| `StepperUi` collects `MudStep`s into a temporary `Vec` | Freed at the end of `show`. |
| Templates are taken out of `self` with `.take()` | Zero-cost when unused; disjoint borrows work. |
| Connectors painted behind circles | One `line_segment` per gap. |
| `MudIcon` reuse across steps | Free thanks to its global transform cache. |
| `MudText` for all labels | Theme and typography propagate automatically. |
| `MudButton` for nav buttons | Matches the rest of the component library. |

---

## Testing

`cargo test` runs 25+ pure-function tests:

| Area | Tests |
|------|-------|
| `step_diameter` | every `Size` |
| `step_phase` | priority: error > skipped > completed > active > pending |
| `is_completed` | requires non-empty, requires all |
| `show_complete_instead_of_next` | last step only, empty stepper |
| `can_go_to_next` | last step, non-linear, linear (completed / skipped / neither) |
| `previous_step_enabled` | first step vs. others |
| `is_current_skippable` | skippable + not disabled, disabled, not skippable |
| `StepFlags` | default, builders |
| `StepperState` | seed once, flags default, setters grow, `is_completed`, `reset` |
| `StepperAction` | every variant, `advanced`, `changed_active_index`, `trigger_index`, `new_active_index` |
| `StepperInteraction` | `new` not cancelled, `prevent` |
| `StepAction` | discriminants match MudBlazor, names |
| `StepPhase` | filled / shows_icon / names |
| Defaults | every `MudStepper` / `MudStep` default |
| Builders | every field set |
| Collector | order preserved |

Rendering (`show`) needs a live `egui::Context`; the pure layer is the source
of truth.

```bash
cargo test -p mud_egui_components mud_stepper
cargo test -p mud_egui_core step_action
```

---

## Differences from MudBlazor

- **State is caller-owned (`StepperState`).** MudBlazor uses bindable
  `ActiveIndex` + per-step `Completed` / `HasError` / `Skipped`. The egui
  equivalent is a struct the caller owns and passes by `&mut`.
- **`Set*Async` methods not ported.** Use `StepperState::set_completed` etc.
  for the flag setters; rebuild the step for `Disabled`.
- **`Get`-only properties are pure functions.** `CanGoToNextStep` →
  `can_go_to_next`, `IsCompleted` → `is_completed`, etc.
- **Nav buttons show labels.** MudBlazor's default is icon-only; the port
  uses `MudButton` with `start_icon` + label so the buttons are accessible.
  Use `action_content(..)` to reproduce the icon-only look.
- **`StepPhase` is a public enum.** MudBlazor derives it internally; the port
  exposes it because it is the natural argument to templates.
- **`StepperAction` is an extension.** MudBlazor only fires callbacks; the
  port also returns what happened for logging and testing.
- **`Ripple` is stored but not rendered.** Same as `MudIcon` / `MudNavMenu`;
  add a ripple implementation in a future pass if needed.
- **`Tag` / `UserAttributes` not ported.** No HTML tag.
- **`Style` → `classes(..)`.** egui has no inline CSS strings.

===== ./crates/mud_egui_components/src/navigation/tabs/README.md =====

# MudTabs for `mud_egui`

A set of views organised into one or more panels, for
[egui](https://github.com/emilk/egui). Ported from MudBlazor's
[`MudTabs`](https://mudblazor.com/components/tabs) and `MudTabPanel`
(MudBlazor 8.13.0).

```rust
let mut tabs = TabsState::new();

MudTabs::new()
.color(Color::Primary)
.show(ui, &mut tabs, |t| {
t.panel(MudTabPanel::new("Account").content(|ui| {
ui.label("Account details…");
}));
t.panel(MudTabPanel::new("Profile").content(|ui| {
ui.label("Profile…");
}));
t.panel(MudTabPanel::new("Billing").content(|ui| {
ui.label("Billing…");
}));
});
// `tabs.active_index` now reflects the user's selection.
```

Every title renders through **`MudText`**, every icon through **`MudIcon`**,
every badge through **`MudBadge`**.

---

## Contents

1. [Design principle: API parity, not API invention](#design-principle-api-parity-not-api-invention)
2. [Architecture](#architecture)
3. [API reference — MudTabs](#api-reference--mudtabs)
4. [API reference — MudTabPanel](#api-reference--mudtabpanel)
5. [API reference — TabsState](#api-reference--tabsstate)
6. [API reference — supporting types](#api-reference--supporting-types)
7. [Examples](#examples)
8. [Layout and slider](#layout-and-slider)
9. [Performance and memory choices](#performance--memory-choices)
10. [Testing](#testing)
11. [Differences from MudBlazor](#differences-from-mudblazor)

---

## Design principle: API parity, not API invention

The MudBlazor 8.13.0 `MudTabs` and `MudTabPanel` surfaces were verified
against [DNDocs](https://docs.dndocs.com/n/MudBlazor/8.13.0/api/MudBlazor.MudTabs.html)
before this refactor.

### Ported `MudTabs` parameters

| MudBlazor | Rust method | Default |
|-----------|-------------|---------|
| `ActivePanelIndex` | `TabsState::active_index` | `0` |
| `ActivePanelIndexChanged` | `on_active_index_changed(f)` | `None` |
| `ActiveTabClass` | `active_tab_class(s)` | `None` |
| `AlwaysShowScrollButtons` | `always_show_scroll_buttons(bool)` | `false` (stored) |
| `ApplyEffectsToContainer` | `apply_effects_to_container(bool)` | `false` |
| `Border` | `border(bool)` | `false` |
| `Centered` | `centered(bool)` | `false` |
| `ChildContent` | closure to `show` | required |
| `Color` | `color(Color)` | `Default` |
| `DisableRipple` | `disable_ripple(bool)` | `false` |
| `DisableSliderAnimation` | `slider_animation(false)` | `false` |
| `Elevation` | `elevation(u8)` | `0` |
| `Header` | `header(f)` | `None` |
| `HeaderPosition` | `header_position(TabHeaderPosition)` | `After` |
| `HideSlider` | `hide_slider(bool)` | `false` |
| `IconColor` | `icon_color(Color)` | `Inherit` |
| `KeepPanelsAlive` | `keep_panels_alive(bool)` | `false` (no-op) |
| `MaxHeight` | `max_height(f32)` | `None` |
| `MinimumTabWidth` | `minimum_tab_width(f32)` | `160.0` |
| `NextIcon` | `next_icon(s)` | `CHEVRON_RIGHT` (stored) |
| `OnPreviewInteraction` | `on_preview_interaction(f)` | `None` |
| `Outlined` | `outlined(bool)` | `false` |
| `PanelClass` | `panel_class(s)` | `None` |
| `Position` | `position(Position)` | `Top` |
| `PrePanelContent` | `pre_panel_content(f)` | `None` |
| `PrevIcon` | `prev_icon(s)` | `CHEVRON_LEFT` (stored) |
| `RightToLeft` | `right_to_left(bool)` | `false` |
| `Ripple` | `ripple(bool)` | `true` (stored) |
| `Rounded` | `rounded(bool)` | `false` |
| `ScrollIconColor` | `scroll_icon_color(Color)` | `Inherit` (stored) |
| `SliderColor` | `slider_color(Color)` | `Inherit` |
| `SliderAnimation` | `slider_animation(bool)` | `true` |
| `TabHeaderClass` | `tab_header_class(s)` | `None` |
| `TabPanelClass` | `tab_panel_class(s)` | `None` |
| `TabPanelHeader` | `tab_panel_header(f)` | `None` |
| `TabPanelHeaderPosition` | `tab_panel_header_position(TabHeaderPosition)` | `After` |
| `Class` (base) | `classes(iter)` | `None` |
| `Style` (base) | *(use `classes`)* | — |
| `Tag` (base) | **not ported** (no HTML tag) | — |
| `UserAttributes` | **not ported** | — |

### Ported `MudTabPanel` parameters

| MudBlazor | Rust method | Default |
|-----------|-------------|---------|
| `BadgeColor` | `badge_color(Color)` | `Primary` |
| `BadgeData` | `badge_data(impl Display)` | `None` |
| `BadgeDot` | `badge_dot(bool)` | `false` |
| `BadgeIcon` | `badge_icon(s)` | `None` |
| `ChildContent` | `content(f)` | `None` |
| `Disabled` | `disabled(bool)` | `false` |
| `ID` | `id(s)` | `None` |
| `Icon` | `icon(s)` | `None` |
| `IconColor` | `icon_color(Color)` | `Inherit` |
| `OnClick` | `on_click(f)` | `None` |
| `ShowCloseIcon` | `show_close_icon(bool)` | `true` (stored; only `MudDynamicTabs`) |
| `TabContent` | `tab_content(f)` | `None` |
| `TabWrapperContent` | `tab_wrapper_content(f)` | `None` |
| `Text` | `new(s)` / `text(s)` | `None` |
| `ToolTip` | `tooltip(s)` | `None` |
| `Visible` | `visible(bool)` | `true` |

### Extensions (not in MudBlazor)

| Extension | Description |
|-----------|-------------|
| `id_salt(s)` | Stable interaction ids for the tab header cells. |
| `TabsState` | Caller-owned active index (the egui equivalent of `ActivePanelIndex`). |
| `TabsResponse` | Rich return value with `action`, `active_index`, `panel_rect`. |
| `TabAction` | Reports what was clicked this frame. |
| `TabView<'s>` | Read-only snapshot passed to `tab_content` / `tab_wrapper_content`. |
| `Position::Start` / `End` | Accepted and normalised to `Left` / `Right` under RTL. |

### Deliberately not ported

- **`SortDirection` / `SortComparer` / `EnableDragAndDrop` / `OnItemDropped`**
  — these are **MudBlazor 8.14** additions, not present in 8.13.
- **`KeepPanelsAlive`** — accepted for parity but is a **no-op** in egui:
  panel bodies are per-frame closures. Keep panel state on the caller (a
  field, `egui::Memory`, or a `TabsState` extension).
- **`ShowCloseIcon` / `CloseIcon` / `CloseIconColor` / `OnClose`** belong to
  `MudDynamicTabs`, which is not ported. `show_close_icon` is stored on
  `MudTabPanel` but never rendered.
- **`AlwaysShowScrollButtons` / `PrevIcon` / `NextIcon` / `ScrollIconColor`**
  are stored but not rendered — the port uses a native `ScrollArea`
  scrollbar instead of icon buttons.
- **`Tag` / `UserAttributes`** — no HTML tag.
- **`Style`** — egui has no inline CSS strings; use `classes(..)`.

---

## Architecture

```
tabs/
├── mud_tab_panel.rs  — MudTabPanel + TabView
├── mud_tabs.rs       — MudTabs + TabUi + TabsResponse + renderers + tests
├── tab_action.rs     — TabAction (extension)
├── tab_interaction.rs — TabInteraction (port of TabInteractionEventArgs)
├── tabs_state.rs     — TabsState { active_index }
└── mod.rs

mud_egui_core/src/enums/
└── tab_header_position.rs — TabHeaderPosition
```

**Pure layer.** `resolve_position`, `is_horizontal_header`,
`horizontal_cell_widths`, `resolve_color`, `elevation_shadow` — free
functions, unit-tested without `egui::Context`.

**State layer.** `TabsState { active_index: usize }` is caller-owned. It is
the egui equivalent of MudBlazor's bindable `ActivePanelIndex`.

**Component layer.** `show` collects `MudTabPanel`s into a `TabUi`, measures
each header via an invisible probe, lays out the header (horizontal row or
vertical column), paints the slider, and renders the active panel's body.

---

## API reference — `MudTabs`

| Method | Type | Default | Description |
|--------|------|---------|-------------|
| `new()` | — | — | Default tabs component. |
| `position(Position)` | `Position` | `Top` | Header position (`Top`/`Bottom`/`Left`/`Right`/`Start`/`End`). |
| `centered(bool)` | `bool` | `false` | Centre the header row. |
| `minimum_tab_width(f32)` | `f32` | `160.0` | Minimum tab width. |
| `max_height(f32)` | `f32` | `None` | Scroll cap for the panel body. |
| `border(bool)` | `bool` | `false` | Divider between header and content. |
| `color(Color)` | `Color` | `Default` | Accent colour. |
| `icon_color(Color)` | `Color` | `Inherit` | Icon colour. |
| `slider_color(Color)` | `Color` | `Inherit` | Slider colour. |
| `slider_animation(bool)` | `bool` | `true` | Animate the slider. |
| `hide_slider(bool)` | `bool` | `false` | Hide the slider. |
| `outlined(bool)` | `bool` | `false` | Outline the header. |
| `rounded(bool)` | `bool` | `false` | Rounded header corners. |
| `elevation(u8)` | `u8` | `0` | Shadow depth. |
| `apply_effects_to_container(bool)` | `bool` | `false` | Apply elevation to the whole component. |
| `ripple(bool)` | `bool` | `true` | Stored for parity. |
| `disable_ripple(bool)` | `bool` | `false` | Stored for parity. |
| `always_show_scroll_buttons(bool)` | `bool` | `false` | Stored for parity. |
| `prev_icon(s)` / `next_icon(s)` | `impl Into<String>` | `CHEVRON_LEFT` / `CHEVRON_RIGHT` | Stored for parity. |
| `scroll_icon_color(Color)` | `Color` | `Inherit` | Stored for parity. |
| `keep_panels_alive(bool)` | `bool` | `false` | No-op in egui. |
| `right_to_left(bool)` | `bool` | `false` | RTL layout. |
| `tab_header_class(s)` | `impl Into<String>` | `None` | Header row classes. |
| `panel_class(s)` | `impl Into<String>` | `None` | Panel body classes. |
| `tab_panel_class(s)` | `impl Into<String>` | `None` | Panel classes (overrides `panel_class`). |
| `active_tab_class(s)` | `impl Into<String>` | `None` | Active tab header classes. |
| `header(f)` | `FnMut(&mut Ui)` | `None` | Custom header fragment. |
| `header_position(TabHeaderPosition)` | `TabHeaderPosition` | `After` | Where `header` is placed. |
| `tab_panel_header(f)` | `FnMut(&mut Ui, &TabView<'_>)` | `None` | Per-tab header fragment. |
| `tab_panel_header_position(TabHeaderPosition)` | `TabHeaderPosition` | `After` | Where `tab_panel_header` is placed. |
| `pre_panel_content(f)` | `FnMut(&mut Ui, &TabView<'_>)` | `None` | Content between header and panel. |
| `on_active_index_changed(f)` | `FnMut(usize)` | `None` | Fires when the active index changes. |
| `on_preview_interaction(f)` | `FnMut(&mut TabInteraction)` | `None` | Fires before activation; may cancel. |
| `id_salt(s)` | `impl Into<String>` | `None` | Stable interaction ids. |
| `classes(iter)` | `iter of AsRef<str>` | none | Wrapper classes. |
| `flavor(StyleFlavor)` | `StyleFlavor` | `Bootstrap` | Utility resolver. |
| `tag(t)` / `tag_ref::<U>()` | `T: Any + Send + Sync` | `None` | Arbitrary data. |
| `show(ui, state, \|t\| ..)` | `(&mut Ui, &mut TabsState, FnOnce(&mut TabUi) -> R)` | — | Returns `(TabsResponse, R)`. |

### `TabUi<'a>`

| Method | Description |
|--------|-------------|
| `panel(panel)` | Adds a `MudTabPanel`. |
| `text_panel(text)` | Convenience: `MudTabPanel::new(text)` with no body. |
| `len()` / `is_empty()` | Collector introspection. |

### `TabsResponse`

| Field | Type | Description |
|-------|------|-------------|
| `response` | `egui::Response` | The wrapper's response. |
| `active_index` | `usize` | Active index after the click was applied. |
| `action` | `TabAction` | What was clicked this frame. |
| `panel_rect` | `Option<Rect>` | Rect of the currently active panel body. |

---

## API reference — `MudTabPanel`

| Method | Type | Default | Description |
|--------|------|---------|-------------|
| `new(text)` | `impl Into<String>` | — | Panel with title, no body. |
| `content_only(f)` | `FnMut(&mut Ui)` | — | Panel with body, no title. |
| `text(s)` | `impl Into<String>` | `None` | Replace title. |
| `icon(s)` | `impl Into<String>` | `None` | Leading icon. |
| `icon_color(Color)` | `Color` | `Inherit` | Icon colour override. |
| `tooltip(s)` | `impl Into<String>` | `None` | Hover tooltip on the tab header. |
| `id(s)` | `impl Into<String>` | `None` | Stable panel id. |
| `badge_data(impl Display)` | `impl Display` | `None` | Badge text. |
| `badge_dot(bool)` | `bool` | `false` | Render the badge as a dot. |
| `badge_icon(s)` | `impl Into<String>` | `None` | Icon inside the badge. |
| `badge_color(Color)` | `Color` | `Primary` | Badge accent. |
| `disabled(bool)` | `bool` | `false` | Disable the tab header. |
| `visible(bool)` | `bool` | `true` | Show/hide the panel. |
| `show_close_icon(bool)` | `bool` | `true` | Stored for parity. |
| `content(f)` | `FnMut(&mut Ui)` | `None` | Panel body. |
| `tab_content(f)` | `FnMut(&mut Ui, &TabView<'_>)` | `None` | Custom tab header renderer. |
| `tab_wrapper_content(f)` | `FnMut(&mut Ui, &TabView<'_>, &mut dyn FnMut(&mut Ui))` | `None` | Header wrapper; receives the inner renderer as the 3rd argument. |
| `on_click(f)` | `FnMut()` | `None` | Fires when the header is clicked (before activation). |

### `TabView<'s>`

Read-only snapshot passed to `tab_content` / `tab_wrapper_content` /
`tab_panel_header` / `pre_panel_content`:

`index`, `text`, `icon`, `icon_color`, `disabled`, `visible`, `is_active`,
`has_badge`, `badge_data`, `badge_dot`, `badge_icon`, `badge_color`,
`tooltip`.

---

## API reference — `TabsState`

| Field / method | Description |
|----------------|-------------|
| `active_index: usize` | 0-based active panel index. |
| `new()` | `active_index = 0`. |
| `with_active_index(index)` | Seeds `active_index`. |
| `activate(index, count)` | Clamps `index` into `0..count`; returns `true` on change. |
| `next(count)` | Advances to the next index, wrapping. |
| `previous(count)` | Moves to the previous index, wrapping. |

---

## API reference — supporting types

### `TabAction`

| Variant | Description |
|---------|-------------|
| `None` | No click this frame. |
| `Activated { from, to }` | A tab became active. |
| `Vetoed { target }` | A tab activation was vetoed by `on_preview_interaction`. |

Helpers: `changed_active_index()`, `was_vetoed()`, `target_index()`.

### `TabInteraction`

| Field / method | Description |
|----------------|-------------|
| `target_index: usize` | The index the user is trying to activate. |
| `from_index: usize` | The currently active index. |
| `cancel: bool` | When `true`, the activation is discarded. |
| `prevent()` | Sets `cancel = true`. |
| `is_prevented()` | `cancel` |

### `TabHeaderPosition`

| Variant | Description |
|---------|-------------|
| `Before` | Place the fragment before the tab list. |
| `After` (default) | Place the fragment after the tab list. |
| `None` | Do not render the fragment. |

### `Position` mapping

| `Position` (input) | Resolved (LTR) | Resolved (RTL) |
|---|---|---|
| `Top` | `Top` | `Top` |
| `Bottom` | `Bottom` | `Bottom` |
| `Left` | `Left` | `Left` |
| `Right` | `Right` | `Right` |
| `Start` | `Left` | `Right` |
| `End` | `Right` | `Left` |
| `Center` | `Top` | `Top` |

---

## Examples

### Basic horizontal tabs

```rust
let mut tabs = TabsState::new();

MudTabs::new().show(ui, &mut tabs, |t| {
    t.panel(MudTabPanel::new("One").content(|ui| {
        ui.add(MudText::new("First panel body"));
    }));
    t.panel(MudTabPanel::new("Two").content(|ui| {
        ui.add(MudText::new("Second panel body"));
    }));
});
```

### Bottom-positioned header

```rust
MudTabs::new()
    .position(Position::Bottom)
    .show(ui, &mut tabs, |t| {
        t.text_panel("A");
        t.text_panel("B");
    });
```

### Left-positioned header

```rust
MudTabs::new()
    .position(Position::Left)
    .minimum_tab_width(120.0)
    .show(ui, &mut tabs, |t| {
        t.text_panel("General");
        t.text_panel("Advanced");
        t.text_panel("About");
    });
```

### Centered + scrollable when overflowing

```rust
MudTabs::new()
    .centered(true)
    .minimum_tab_width(160.0)
    .show(ui, &mut tabs, |t| {
        for i in 0..12 {
            t.text_panel(format!("Tab {i}"));
        }
    });
```

### Icons and badges

```rust
MudTabs::new().show(ui, &mut tabs, |t| {
    t.panel(
        MudTabPanel::new("Inbox")
            .icon(filled::INBOX)
            .badge_data("3")
            .badge_color(Color::Error)
            .content(|ui| { ui.label("Inbox…"); }),
    );
    t.panel(
        MudTabPanel::new("Archive")
            .icon(filled::ARCHIVE)
            .content(|ui| { ui.label("Archive…"); }),
    );
    t.panel(
        MudTabPanel::new("Notifications")
            .icon(filled::NOTIFICATIONS)
            .badge_dot(true)
            .badge_color(Color::Warning)
            .content(|ui| { ui.label("Notifications…"); }),
    );
});
```

### Disabled / hidden panels

```rust
MudTabs::new().show(ui, &mut tabs, |t| {
    t.panel(MudTabPanel::new("A"));
    t.panel(MudTabPanel::new("B").disabled(true));
    t.panel(MudTabPanel::new("C").visible(false));
    t.panel(MudTabPanel::new("D"));
});
```

Disabled panels are shown but not clickable; hidden panels are not rendered
and are not counted in the visible index sequence.

### Slider animation off

```rust
MudTabs::new()
    .slider_animation(false)
    .show(ui, &mut tabs, |t| { /* … */ });
```

The slider jumps instantly between tabs.

### Preview interaction (veto activation)

```rust
MudTabs::new()
    .on_preview_interaction(|i| {
        if i.target_index == 2 && has_unsaved_changes() {
            i.prevent();
        }
    })
    .show(ui, &mut tabs, |t| { /* … */ });
```

### Custom tab header

```rust
MudTabs::new().show(ui, &mut tabs, |t| {
    t.panel(
        MudTabPanel::new("Overview")
            .icon(filled::INFO)
            .content(|ui| { ui.label("Overview…"); }),
    );
    t.panel(
        MudTabPanel::new("")
            .tab_content(|ui, view| {
                ui.add(MudIcon::new(filled::SETTINGS));
                ui.add(MudText::new("Settings").bold(view.is_active));
                if view.is_active {
                    ui.add(MudText::new("·").color(Color::Primary));
                }
            })
            .content(|ui| { ui.label("Settings…"); }),
    );
});
```

### Custom tab wrapper (adds a border around the header content)

```rust
MudTabPanel::new("Custom")
    .tab_wrapper_content(|ui, _view, inner| {
        egui::Frame::default()
            .stroke(egui::Stroke::new(1.0, Color32::GRAY))
            .inner_margin(egui::Margin::same(4))
            .show(ui, |ui| inner(ui));
    });
```

The third argument (`inner`) is the "render the default header" closure.
Calling it renders the standard text + icon + badge content into the
wrapper's `Ui`.

### Header fragment before / after the tabs

```rust
MudTabs::new()
    .header(|ui| ui.add(MudButton::new("+ New Tab")))
    .header_position(TabHeaderPosition::After)
    .show(ui, &mut tabs, |t| { /* … */ });
```

### Pre-panel content

```rust
MudTabs::new()
    .pre_panel_content(|ui, view| {
        ui.add(
            MudText::new(format!("Showing: {}", view.text.unwrap_or("")))
                .typo(Typo::Caption),
        );
    })
    .show(ui, &mut tabs, |t| { /* … */ });
```

### Outlined + rounded + elevation

```rust
MudTabs::new()
    .outlined(true)
    .rounded(true)
    .elevation(2)
    .apply_effects_to_container(true)
    .show(ui, &mut tabs, |t| { /* … */ });
```

### Scrollable panel body

```rust
MudTabs::new()
    .max_height(300.0)
    .show(ui, &mut tabs, |t| {
        t.panel(MudTabPanel::new("Log").content(|ui| {
            for i in 0..200 {
                ui.add(MudText::new(format!("Line {i}")));
            }
        }));
    });
```

### Read the outcome

```rust
let (resp, _) = MudTabs::new().show(ui, &mut tabs, |t| { /* … */ });
match resp.action {
    TabAction::Activated { from, to } => log::info!("{from} → {to}"),
    TabAction::Vetoed { target } => log::warn!("activation to {target} vetoed"),
    TabAction::None => {}
}
if let Some(rect) = resp.panel_rect {
    log::debug!("panel body rect: {rect:?}");
}
```

### Programmatic navigation

```rust
tabs.next(3);     // wraps to 0 if currently 2
tabs.previous(3); // wraps to 2 if currently 0
tabs.activate(1, 3);
```

---

## Layout and slider

### Header positions

| `Position` | Header layout | Content layout |
|---|---|---|
| `Top` (default) | Horizontal row above the panel | Below the header |
| `Bottom` | Horizontal row below the panel | Above the header |
| `Left` | Vertical column, left column (~30% width) | Right column (rest) |
| `Right` | Vertical column, right column (~30% width) | Left column (rest) |

### Horizontal cell widths

`horizontal_cell_widths(natural, minimum, available)`:

1. Every cell is at least `minimum` wide.
2. If the total is **less** than `available`, all cells scale **up**
   proportionally to fill the row.
3. If the total **exceeds** `available`, cells keep their natural / minimum
   widths — the header row is wrapped in a `ScrollArea::horizontal` by the
   caller so overflowing tabs are reachable by scrolling.

### Slider

The active tab's slider is drawn as a 2 px line on the **outer** edge of the
header (top for `Top`, bottom for `Bottom`, left for `Left`, right for
`Right`).

| Setting | Behaviour |
|---------|-----------|
| `slider_animation(true)` (default) | Animates between tab positions using `egui::Context::animate_value_with_time` with a 220 ms ease. |
| `slider_animation(false)` | Jumps instantly. |
| `hide_slider(true)` | No slider is painted. |

The animation state lives in `egui::Memory` keyed by
`ui.scope_id().with(("slider_x", …))` etc. — no per-component state on
`MudTabs`.

### `Border`

When `border(true)`, a 1 px line is drawn on the edge where the header meets
the panel body (`bottom` for `Top`, `top` for `Bottom`, `right` for `Left`,
`left` for `Right`). This matches MudBlazor's `border` behaviour exactly.

### `Outlined`

When `outlined(true)`, a 1 px line is drawn around the entire header block.

### Elevation

| `elevation` | `apply_effects_to_container` | Shadow |
|---|---|---|
| `0` | either | none |
| `> 0` | `false` | Shadow on the header block only |
| `> 0` | `true` | Shadow on the whole component |

Values above `24` are clamped.

---

## Performance and memory choices

| Choice | Effect |
|--------|--------|
| `resolve_position` / `is_horizontal_header` / `horizontal_cell_widths` are pure | No `egui::Context`; unit-tested. |
| `TabsState` is `Copy` | Zero-cost snapshot. |
| `TabUi` collects panels into a temporary `Vec` | Freed at the end of `show`. |
| Panel headers are measured with an invisible probe | One extra `Ui` per `show`, not per tab. |
| Only the active panel's body runs | Non-active panel bodies are skipped (MudBlazor's `KeepPanelsAlive = false` behaviour). |
| `egui::Memory`-backed slider animation | No per-component state; `TabsState` stays `Copy`. |
| `ScrollArea::horizontal` for overflowing headers | Native scrollbar; matches MudBlazor's reachability goal without porting `PrevIcon` / `NextIcon`. |
| `MudIcon` reuse across tabs | Free thanks to its transform cache. |
| `MudText` for all titles | Theme and typography propagate automatically. |
| `MudBadge` for badges | Matches the rest of the library. |
| `PanelClass` / `TabPanelClass` resolved once per `show` | One `BsStyle` / `TwStyle` call, not one per panel. |

Not done, on purpose: virtualisation of the header. MudBlazor renders every
visible tab; the port does the same. For dozens of tabs, `centered(true) +
ScrollArea` is the intended reach.

---

## Testing

`cargo test` runs 20+ pure-function tests:

| Area | Tests |
|------|-------|
| `resolve_position` | physical variants, `Start` / `End` normalisation under RTL, `Center` → `Top` |
| `is_horizontal_header` | `Top` / `Bottom` vs `Left` / `Right` |
| `horizontal_cell_widths` | clamp to minimum, grow to fill, no shrink on overflow, empty input |
| Defaults | every `MudTabs` default matches MudBlazor |
| Builders | every field set, `minimum_tab_width` clamps negative |
| `MudTabPanel` | `new` vs `content_only`, every builder, `view` fields, `has_badge` |
| `TabsState` | default, `with_active_index`, `activate` (clamping, empty), `next` / `previous` wrap |
| `TabAction` | every variant, helper methods |
| `TabInteraction` | `new` not cancelled, `prevent` |
| `TabHeaderPosition` | default, predicates, names |
| Collector | preserves panel order |

Rendering (`show`) needs a live `egui::Context`; the pure layer is the source
of truth.

```bash
cargo test -p mud_egui_components mud_tabs
cargo test -p mud_egui_core tab_header_position
```

---

## Differences from MudBlazor

- **State is caller-owned (`TabsState`).** MudBlazor uses the bindable
  `ActivePanelIndex` parameter; the egui equivalent is a struct the caller
  owns and passes by `&mut`. This matches `MudTable` / `MudTreeView` /
  `MudChipSet` / `MudNavMenu` / `MudPagination` / `MudStepper`.
- **`KeepPanelsAlive` is a no-op.** egui panel bodies are per-frame closures;
  persistent state belongs on the caller.
- **`SortDirection` / `SortComparer` / `EnableDragAndDrop` / `OnItemDropped`
  not ported.** These are 8.14 additions; the port targets 8.13.
- **`MudDynamicTabs` not ported.** The `ShowCloseIcon` / `CloseIcon` /
  `CloseIconColor` / `OnClose` parameters that belong to it are stored on
  `MudTabPanel` for forward-compat but are not rendered.
- **Scroll buttons (`PrevIcon` / `NextIcon` / `ScrollIconColor` /
  `AlwaysShowScrollButtons`) are stored but unused.** The port uses a native
  `ScrollArea::horizontal` for the header row instead.
- **`Ripple` / `DisableRipple` are stored but unused.** The port uses a
  hover highlight instead.
- **`Tag` / `UserAttributes` not ported.** No HTML tag.
- **`Style` → `classes(..)`.** egui has no inline CSS strings.
- **`Position::Start` / `End` accepted.** MudBlazor's `Position` enum has
  these but `MudTabs` documents only `Top` / `Bottom` / `Left` / `Right`. The
  port accepts them and normalises to `Left` / `Right` under RTL so callers
  can be layout-direction-agnostic.

===== ./crates/mud_egui_components/src/surfaces/card/README.md =====

# MudCard family for `mud_egui`

A surface that groups a header, media, content and actions, plus its four
companion blocks. Ported from MudBlazor 8.13.0's
[`MudCard`](https://mudblazor.com/components/card),
`MudCardHeader`, `MudCardContent`, `MudCardActions` and `MudCardMedia`.

```rust
use mud_egui_components::surfaces::card::{
    MudCard, MudCardActions, MudCardContent, MudCardHeader, MudCardMedia,
};

MudCard::new().show(ui, |ui| {
    MudCardMedia::new()
        .image("https://picsum.photos/400/200")
        .height(160.0)
        .title("A photo")
        .show(ui);

    MudCardHeader::new().show(ui, |ui| {
        ui.add(MudText::h6("Card title"));
    });

    MudCardContent::new().show(ui, |ui| {
        ui.add(MudText::body2("Supporting copy goes here."));
    });

    MudCardActions::new().show(ui, |ui| {
        ui.add(MudButton::new("Dismiss").variant(Variant::Text));
        ui.add(MudButton::new("Open").variant(Variant::Filled));
    });
});
```

The card paints only the surface; every string inside is yours to render through
**`MudText`**, every glyph through **`MudIcon`**.

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

All five components were verified against DNDocs for MudBlazor 8.13.0 before
implementation. Together they expose **exactly eight parameters**.

### `MudCard`

| MudBlazor | Type | Default | Rust mapping |
|---|---|---|---|
| `ChildContent` | `RenderFragment?` | `null` | closure to `show(ui, \|ui\| ..)` |
| `Elevation` | `int` | **`1`** | `elevation(u8)` (clamped 0–24) |
| `Outlined` | `bool` | `false` | `outlined(bool)` |
| `Square` | `bool` | `false` | `square(bool)` |
| `Class` | `string?` | `null` | `classes(iter)` — stored, not resolved |
| `Tag` | `object?` | `null` | `tag(t)` / `tag_ref::<U>()` |

### `MudCardHeader` / `MudCardContent` / `MudCardActions`

| MudBlazor | Type | Default | Rust mapping |
|---|---|---|---|
| `ChildContent` | `RenderFragment?` | `null` | closure to `show(ui, \|ui\| ..)` |
| `Class` | `string?` | `null` | `classes(iter)` — stored |
| `Tag` | `object?` | `null` | `tag(t)` / `tag_ref::<U>()` |

### `MudCardMedia`

| MudBlazor | Type | Default | Rust mapping |
|---|---|---|---|
| `Image` | `string?` | `null` | `image(s)` |
| `Height` | `int` | **`300`** | `height(f32)` |
| `Title` | `string?` | `null` | `title(s)` |
| `Class` | `string?` | `null` | `classes(iter)` — stored |
| `Tag` | `object?` | `null` | `tag(t)` / `tag_ref::<U>()` |

### Deliberately not ported

| MudBlazor | Reason |
|---|---|
| `Style` (all five) | egui has no inline CSS strings. |
| `UserAttributes` (all five) | There is no HTML tag to splat attributes onto. |
| `Class` resolution (all five) | Stored on `base` for parity, but not resolved: `MudCard`'s fill, border and radius are owned by `Elevation` / `Outlined` / `Square`, and the companions own only their padding. Resolving a second utility frame would composite two fills or two margins. |

Nothing else was added. In particular there is no `Color`, no `Variant`, no
`Size`, no `Disabled` / `ReadOnly`, no `OnClick` and no adornment slots —
`MudCard` has none of them in MudBlazor, and the companions have only
`ChildContent`.

---

## Architecture

```
surfaces/card/
├── mud_card.rs           MudCard: surface + elevation/outline/radius
│   └── pure helpers      card_radius, card_stroke, card_shadow,
│                         pad_u8, pad_i8
├── mud_card_header.rs    MudCardHeader: 16 pt padded vertical block
├── mud_card_content.rs   MudCardContent: 16 pt padded vertical block
├── mud_card_actions.rs   MudCardActions: 8 pt padded horizontal row
├── mud_card_media.rs     MudCardMedia: MudImage with Cover fit + height
│   └── pure helpers      sane_height
└── mod.rs                re-exports
```

**Pure layer.** Every decision that can be made from numbers is a free
function: the corner radius, whether a stroke or a shadow is emitted and at what
strength, how a padding is clamped into `i8`, and how a bad media height is
repaired. All of them are unit-tested without an `egui::Context`.

**Component layer.** `MudCard::show` fetches the theme once, builds one
`egui::Frame`, and runs the caller's closure inside it. The companions build a
single `Frame` with a constant margin and run the closure inside that. None of
them holds state between frames.

**No `MudPaper` dependency.** `MudCard` could have delegated to `MudPaper`, but
`MudPaper::show` takes `&mut self` (it refreshes its Tailwind state after
drawing), which would force `MudCard` to own a long-lived `MudPaper` and break
the stateless-builder model every other component in this workspace follows.
`MudCard` therefore builds its own frame, exactly as `MudAlert` does.

---

## API reference

### `MudCard`

| Method | Type | Default | Mirrors | Description |
|---|---|---|---|---|
| `new()` | `Self` | — | — | Elevation 1, rounded, not outlined. |
| `elevation(e)` | `u8` | `1` | `Elevation` | Shadow depth, clamped to 24. `0` = flat. |
| `outlined(b)` | `bool` | `false` | `Outlined` | 1 pt border; suppresses the shadow. |
| `square(b)` | `bool` | `false` | `Square` | Zero corner radius. |
| `classes(iter)` | `iter of AsRef<str>` | none | `Class` | Stored on `ComponentBase`. |
| `tag(t)` / `tag_ref::<U>()` | `T: Any + Send + Sync` | `None` | `Tag` | Arbitrary user data. |
| `show(ui, \|ui\| ..)` | `&mut Ui, FnOnce(&mut Ui) -> R` | — | `ChildContent` | Returns `InnerResponse<R>`. |

### `MudCardHeader`

| Method | Default | Mirrors | Description |
|---|---|---|---|
| `new()` | — | — | 16 pt padding on all sides. |
| `classes(iter)` | none | `Class` | Stored. |
| `tag(t)` / `tag_ref::<U>()` | `None` | `Tag` | Arbitrary user data. |
| `show(ui, \|ui\| ..)` | — | `ChildContent` | Vertical stack inside the padding. |

### `MudCardContent`

Identical surface to `MudCardHeader` (16 pt padding, vertical stack). The two
are separate types so a call site reads like the Blazor markup and so their
padding can diverge if MudBlazor ever does.

### `MudCardActions`

| Method | Default | Mirrors | Description |
|---|---|---|---|
| `new()` | — | — | 8 pt padding, `left_to_right` row, vertically centred. |
| `classes(iter)` | none | `Class` | Stored. |
| `tag(t)` / `tag_ref::<U>()` | `None` | `Tag` | Arbitrary user data. |
| `show(ui, \|ui\| ..)` | — | `ChildContent` | Horizontal row inside the padding. |

### `MudCardMedia`

| Method | Type | Default | Mirrors | Description |
|---|---|---|---|---|
| `new()` | `Self` | — | — | No image, height 300. |
| `image(s)` | `impl Into<String>` | `None` | `Image` | URL or path. |
| `height(h)` | `f32` | `300.0` | `Height` | Strip height in points. |
| `title(s)` | `impl Into<String>` | `None` | `Title` | Forwarded to `MudImage::alt`. |
| `classes(iter)` | `iter of AsRef<str>` | none | `Class` | Stored. |
| `tag(t)` / `tag_ref::<U>()` | `T: Any + Send + Sync` | `None` | `Tag` | Arbitrary user data. |
| `show(ui)` | `&mut Ui` | — | — | Returns the image's `Response`. |

### Constants

| Name | Value | Where |
|---|---|---|
| `CARD_RADIUS` | `4.0` | `mud_card` (crate-private) |
| `OUTLINE_STROKE_WIDTH` | `1.0` | `mud_card` |
| `MAX_ELEVATION` | `24` | `mud_card` |
| `HEADER_PADDING` | `16.0` | `mud_card_header` |
| `CONTENT_PADDING` | `16.0` | `mud_card_content` |
| `ACTIONS_PADDING` | `8.0` | `mud_card_actions` |
| `DEFAULT_MEDIA_HEIGHT` | `300.0` | `mud_card_media` |

---

## Examples

### Minimal card

```rust
MudCard::new().show(ui, |ui| {
    MudCardContent::new().show(ui, |ui| {
        ui.add(MudText::body1("Just some text in a surface."));
    });
});
```

### Product card with media, title, copy and actions

```rust
MudCard::new().show(ui, |ui| {
    MudCardMedia::new()
        .image("file://assets/shoe.png")
        .height(180.0)
        .title("Trail runner")
        .show(ui);

    MudCardHeader::new().show(ui, |ui| {
        ui.add(MudText::h6("Trail runner"));
        ui.add(MudText::body2("Lightweight, 4 mm drop").color(Color::TextSecondary));
    });

    MudCardContent::new().show(ui, |ui| {
        ui.add(MudText::body2("Built for long days on loose ground."));
    });

    MudCardActions::new().show(ui, |ui| {
        ui.add(MudButton::new("Details").variant(Variant::Text));
        ui.add(MudButton::new("Add to cart").variant(Variant::Filled));
    });
});
```

### Outlined card, no shadow

```rust
MudCard::new()
    .outlined(true)
    .show(ui, |ui| {
        MudCardContent::new().show(ui, |ui| {
            ui.add(MudText::body2("Outlined cards read as selectable."));
        });
    });
```

`outlined(true)` suppresses the shadow even when `elevation` is left at its
default of 1 — that is MudBlazor's `MudPaper` rule.

### Flat card in a dense grid

```rust
MudCard::new().elevation(0).square(true).show(ui, |ui| {
    MudCardContent::new().show(ui, |ui| {
        ui.add(MudText::body2("Tiled, no gap seams."));
    });
});
```

### Elevated, floating card

```rust
MudCard::new().elevation(6).show(ui, |ui| {
    MudCardContent::new().show(ui, |ui| {
        ui.add(MudText::body1("Lifted off the page."));
    });
});
```

### Actions pushed to the trailing edge

```rust
MudCard::new().show(ui, |ui| {
    MudCardContent::new().show(ui, |ui| {
        ui.add(MudText::body2("Confirm?"));
    });
    MudCardActions::new().show(ui, |ui| {
        ui.add(MudButton::new("Cancel").variant(Variant::Text));
        ui.add(MudButton::new("Delete").color(Color::Error));
        ui.add(MudSpacer::new()); // absorbs the remaining width
    });
});
```

`MudCardActions` uses the same start-aligned row MudBlazor's stylesheet
produces (`display: flex; align-items: center`, no `justify-content`). A
`MudSpacer` is the idiomatic way to right-align the buttons.

### Header-only card (a panel)

```rust
MudCard::new().elevation(0).outlined(true).show(ui, |ui| {
    MudCardHeader::new().show(ui, |ui| {
        ui.add(MudText::subtitle2("Filters"));
    });
    MudCardContent::new().show(ui, |ui| {
        ui.checkbox(&mut self.show_archived, "Show archived");
    });
});
```

### Media strip without a URL

```rust
// Reserves 300 pt of blank space — the same box an <img> occupies before load.
MudCardMedia::new().show(ui);
```

### Tag round-trip

```rust
let card = MudCard::new().tag("product-42");
assert_eq!(card.tag_ref::<&str>(), Some(&"product-42"));
card.show(ui, |ui| { /* … */ });
```

---

## Layout and rendering

### Card surface

```
┌──────────────────────────────────────────────┐  ← Frame: fill = palette.surface
│  MudCardMedia   (height h, full width)        │     radius 4 pt (0 if Square)
├──────────────────────────────────────────────┤     shadow if elevation > 0
│  MudCardHeader  (padding 16)                  │     stroke 1 pt if Outlined
├──────────────────────────────────────────────┤
│  MudCardContent (padding 16)                  │
├──────────────────────────────────────────────┤
│  MudCardActions (padding 8, row, centred)     │
└──────────────────────────────────────────────┘
```

| Property | Rule |
|---|---|
| Fill | `palette.surface` from the active `MudTheme`. |
| Corner radius | 4 pt, or 0 with `square(true)`. |
| Outline | 1 pt in `ui.visuals().widgets.noninteractive.bg_stroke.color`; only when `outlined`. |
| Shadow | `card_shadow(elevation)`; `None` at 0 and when `outlined`. |
| Inner padding | **None.** The card is a pure surface; the companions own the spacing. |
| Width | Whatever the content asks for. Wrap the card in a sized parent (or set the child's width) to make it fill. |

### Why the card has no padding

In MudBlazor, `.mud-card` sets `overflow: hidden` and nothing else; the 16 pt
inset you see comes from `MudCardHeader` / `MudCardContent`, and the 8 pt inset
around buttons comes from `MudCardActions`. Splitting it the same way keeps a
media strip flush to the card edge, which is the whole point of
`MudCardMedia`.

### Clipping to rounded corners

egui's `Frame` does not clip its children, so a `MudCardMedia` inside a card
with a non-zero radius shows square corners where it meets the rounded edge.
MudBlazor avoids this with CSS `overflow: hidden`. The equivalent here is to
give the card an explicit width and clip the media yourself:

```rust
let card_w = 320.0;
ui.allocate_ui(vec2(card_w, 0.0), |ui| {
    MudCard::new().show(ui, |ui| {
        let rect = Rect::from_min_size(ui.cursor().min, vec2(card_w, 180.0));
        ui.scope(|ui| {
            ui.set_clip_rect(rect.intersect(ui.clip_rect()));
            MudCardMedia::new().image("file://assets/hero.png").height(180.0).show(ui);
        });
    });
});
```

This is not applied implicitly because `MudCard` sizes to its content and
therefore does not know its own final rectangle while it is still drawing.

### Density

There is one density. MudBlazor's card has no `Dense` parameter, so neither
does this port; tighten spacing by nesting a `MudStack` with a smaller
`spacing` inside `MudCardContent`.

---

## Performance and memory choices

| Choice | Effect |
|---|---|
| Theme fetched once per `MudCard::show` | One `MudTheme::fetch` and one `palette.surface.into()`. The four companions touch no theme at all. |
| One `egui::Frame` per component | No nested frames, no double compositing. |
| `bool::then` for the stroke and shadow | No `Option<Option<_>>`, no branches in the paint path. |
| Pure helpers on `Copy` values | Radius, stroke, shadow and height repair are arithmetic; unit-testable without a `Context`. |
| `Margin::same` with a constant | No per-frame string or class parsing. |
| `MudCardMedia` delegates to `MudImage` | Async loading, fallback sources and the broken-image placeholder come for free; the two caches (`PATH_CACHE` aside) are the loader's. |
| No `egui::Memory` anywhere | Nothing to invalidate, no re-entrant-`RwLock` hazard. |
| No `Vec` or `String` allocation in `show` | The only allocation is whatever `MudImage` does internally for the URI. |

Not done, on purpose: caching the resolved surface colour or the shadow. Both
depend on the theme, which can change between frames, and both are a handful of
arithmetic operations — a cache would cost more than it saves.

---

## Testing

`cargo test -p mud_egui_components mud_card` runs 20 tests across the five
files:

| File | Tests |
|---|---|
| `mud_card.rs` | radius is 0 when square; stroke only when outlined and carries its colour; shadow is `None` at 0, monotonic, clamped at 24; `pad_u8` / `pad_i8` sanitizing (`NaN`, negative, infinity, `>127`); defaults (elevation **1**); every builder; tag round-trip |
| `mud_card_header.rs` | padding constant is 16 and maps to `i8` 16; empty defaults; builders; tag round-trip |
| `mud_card_content.rs` | same, for its own 16 pt constant |
| `mud_card_actions.rs` | padding constant is 8; empty defaults; builders; tag round-trip |
| `mud_card_media.rs` | default height is **300**; `sane_height` (valid, zero, negative, `NaN`, infinity); defaults; every builder; tag round-trip |

No test creates an `egui::Context`. `MudCard::show`, the three companion `show`
methods and `MudCardMedia::show` need a live `Ui` (layout, painting, theme
lookup) and are therefore left to integration tests. Everything they delegate
to is covered above.

```bash
cargo test -p mud_egui_components mud_card
```

---

## Differences from MudBlazor

- **`Class` is stored but not resolved.** `Elevation` / `Outlined` / `Square`
  own the surface, and each companion owns exactly one padding constant.
  Resolving a utility-class frame on top would composite a second fill or a
  second margin. `classes(..)` is kept so a Blazor call site ports without
  edits, and so the field is available if class resolution is added later.
- **`Style` is not ported.** egui has no inline CSS strings.
- **`UserAttributes` is not ported.** There is no HTML tag.
- **`Elevation` is `u8`, not `int`.** Clamped to `0..=24`, matching every other
  component in this workspace. MudBlazor clamps too; the range is identical.
- **`Height` on `MudCardMedia` is `f32`, not `int`.** Points, consistent with
  `MudStack::height`, `MudPaper::height` and `MudCollapse::max_height`. The
  default is still 300.
- **`MudCardActions` has no `justify` parameter.** MudBlazor's stylesheet does
  not right-align actions either — it only centres them vertically. Insert a
  `MudSpacer` to push buttons to the trailing edge, the same technique
  `MudToolBar` documents. Adding a `justify` builder would be API invention.
- **`MudCardMedia` delegates to `MudImage`.** MudBlazor emits a bare `<img>`
  with `object-fit: cover`. `MudImage` gives the same geometry plus async
  loading, a fallback source and an error placeholder, which a card showing
  remote product photos wants anyway. `Title` maps to `MudImage::alt`.
- **No rounded-corner clipping.** MudBlazor relies on CSS `overflow: hidden`;
  egui frames do not clip. The README shows the `ui.scope` +
  `set_clip_rect` recipe. Applying it implicitly would require `MudCard` to
  know its own size before drawing it, which immediate mode does not allow.
- **No `CardHeaderContent` helper.** It appears in MudBlazor's documentation
  examples but is defined by the docs project, not by MudBlazor. It is not a
  component and is therefore not ported.

===== ./crates/mud_egui_components/src/surfaces/collapse/README.md =====

# MudCollapse for `mud_egui`

A container that animates its height and opacity to hide or reveal its content,
for [egui](https://github.com/emilk/egui). Ported from MudBlazor's
[`MudCollapse`](https://mudblazor.com/api/collapse) (MudBlazor 8.13.0).

```rust
let out = MudCollapse::new()
    .expanded(self.open)
    .id_salt("faq-1")
    .show(ui, |ui| {
        ui.add(MudText::body2("Hidden until expanded."));
    });

if let Some(open) = out.expanded_changed {
    log::info!("expanded changed to {open}");
}
if out.animation_ended {
    self.ready_to_scroll = true;
}
```

Every string inside renders through **`MudText`**, every glyph through
**`MudIcon`**.

---

## Contents

1. [Design principle: API parity, not API invention](#design-principle-api-parity-not-api-invention)
2. [Architecture](#architecture)
3. [API reference](#api-reference)
4. [Events in an immediate-mode UI](#events-in-an-immediate-mode-ui)
5. [Layout and rendering](#layout-and-rendering)
6. [Examples](#examples)
7. [Performance and memory choices](#performance-and-memory-choices)
8. [Testing](#testing)
9. [Differences from MudBlazor](#differences-from-mudblazor)

---

## Design principle: API parity, not API invention

The MudBlazor 8.13.0 parameter surface was verified against DNDocs before
implementation. `MudCollapse` has **exactly five parameters**, and all five are
ported.

### Ported parameters

| MudBlazor | Type | Default | Rust mapping |
|---|---|---|---|
| `ChildContent` | `RenderFragment?` | `null` | closure to `show(ui, \|ui\| ..)` |
| `Expanded` | `bool` | `false` | `expanded(bool)` |
| `ExpandedChanged` | `EventCallback<bool>` | — | `CollapseResponse::expanded_changed` |
| `MaxHeight` | `int?` | `null` | `max_height(f32)` → `Option<f32>` |
| `OnAnimationEnd` | `EventCallback` | — | `CollapseResponse::animation_ended` |
| `Class` | `string?` | `null` | `classes(iter)` — resolved through `flavor` |
| `Tag` | `object?` | `null` | `tag(t)` / `tag_ref::<U>()` |

### Deliberately not ported

| MudBlazor | Reason |
|---|---|
| `Style` | egui has no inline CSS strings. Utility classes are resolved through the Bootstrap or Tailwind flavour; use `classes(..)` and the typed `flavor(..)` for the same purpose. |
| `UserAttributes` | There is no HTML tag to splat attributes onto. |

`MaxHeight` becomes `Option<f32>` because CSS `int?` (pixels) and egui points
are the same unit at scale 1; `None` means "no cap", which is MudBlazor's
`null`.

---

## Architecture

```
mud_collapse.rs
├── constants          DEFAULT_DURATION, HEIGHT_EPSILON, MAX_NATURAL_HEIGHT,
│                      REM_SIZE_PT
├── pure helpers       sane_duration, sane_length, sane_max_height,
│                      is_animating, detect_animation_end,
│                      detect_expanded_change, visible_height,
│                      needs_height_update, placed_width
├── compute_progress   animate_bool_with_time, or a direct 0/1 for duration 0
├── paint_frame_bg     shadow + fill + stroke over the reserved rect
├── MudCollapse        builder
│   └── show           progress → events → one of three paths
├── CollapseResponse   response + inner + progress + event fields (Deref<Response>)
├── impl Widget        ui.add(MudCollapse::new())  (empty content)
└── mod tests          16 pure-function tests
```

**Pure layer.** Every decision that can be made from numbers — sanitizing the
duration and the cap, whether we are animating, whether the animation just
ended, whether `expanded` changed, how tall the visible strip is, whether the
cached height is stale — is a free function over `f32` and `bool`. None of them
touch `egui::Context`, so they are unit-tested without a window.

**State layer.** Three values live in `egui::Memory`, keyed by
`ui.make_persistent_id(id_salt)`:

| Key | Type | Purpose |
|---|---|---|
| `"height"` | `f32` | Last measured natural height |
| `"prev_expanded"` | `bool` | Previous frame's `expanded` |
| `"animating"` | `bool` | Whether we were animating last frame |

`id_salt` disambiguates collapses that share a parent, the same way
`MudToggleGroup` salts its ripple keys.

**Component layer.** `show` reads progress, computes the two events, picks one
of three paths, draws the content **exactly once**, and reserves the correct
height.

### The three paths

| Path | Condition | Behaviour |
|---|---|---|
| Collapsed | `!expanded && progress <= 0` | Zero-sized response; the closure is **not called**; the height cache is dropped. |
| Settled expanded | `expanded && progress >= 1` | Content drawn plainly, measured, exactly its height (capped by `max_height`) reserved. |
| Animating | otherwise | Content drawn clipped to `cached × progress`, opacity `= progress`, `cached × progress` reserved. |

---

## API reference

### `MudCollapse`

| Method | Type | Default | Mirrors | Description |
|---|---|---|---|---|
| `new()` | `Self` | — | — | Collapsed, 0.2 s, no cap. |
| `expanded(b)` | `bool` | `false` | `Expanded` | Whether the content is shown. |
| `duration(s)` | `f32` | `0.2` | *(CSS transition)* | Animation seconds; `0.0` is instant. |
| `max_height(px)` | `f32` | `None` | `MaxHeight` | Visible height cap in points. |
| `id_salt(s)` | `impl Hash + Debug` | `None` | — | Stable identity for two collapses in one parent. |
| `flavor(f)` | `StyleFlavor` | `Bootstrap` | — | Utility-class resolver. |
| `classes(iter)` | `iter of AsRef<str>` | none | `Class` | Utility classes resolved into the frame. |
| `tag(t)` / `tag_ref::<U>()` | `T: Any + Send + Sync` | `None` | `Tag` | Arbitrary user data. |
| `show(ui, \|ui\| ..)` | `&mut Ui, FnOnce(&mut Ui) -> R` | — | `ChildContent` | Returns `CollapseResponse<R>`. |

### `CollapseResponse<R>`

| Field / method | Type | Description |
|---|---|---|
| `response` | `Response` | The reserved area. `CollapseResponse` derefs to it. |
| `inner` | `Option<R>` | The closure's value; `None` while fully collapsed. |
| `progress` | `f32` | `0.0` collapsed … `1.0` expanded. |
| `expanded_changed` | `Option<bool>` | `Some(new)` on the frame `expanded` changed; `None` on the first frame and on steady frames. |
| `animation_ended` | `bool` | `true` for exactly one frame when progress reaches its target. |
| `is_fully_expanded()` | `bool` | `progress >= 1`. |
| `is_fully_collapsed()` | `bool` | `progress <= 0`. |

### Constants

| Name | Value |
|---|---|
| `DEFAULT_DURATION` | `0.2` s |
| `HEIGHT_EPSILON` | `0.1` pt |
| `MAX_NATURAL_HEIGHT` | `10_000.0` pt |

---

## Events in an immediate-mode UI

egui has no event-callback objects, so the two MudBlazor callbacks are reported
as **data about this frame**:

```rust
let out = MudCollapse::new().expanded(open).id_salt("faq-1").show(ui, |ui| {
    ui.label("…");
});

if let Some(now_open) = out.expanded_changed { /* ExpandedChanged */ }
if out.animation_ended                     { /* OnAnimationEnd  */ }
```

- `expanded_changed` is `Some(new)` only when the `expanded` argument differs
  from the previous frame's. It is **never** set on the first frame (no change
  has happened yet).
- `animation_ended` is `true` for **exactly one frame**, when progress reaches
  the target (`1.0` when expanding, `0.0` when collapsing). With
  `duration(0.0)` no animation is ever in progress, so it never fires.
- If the collapse is not shown for a frame (an ancestor is hidden), the stored
  state is not updated; events resume correctly when it is shown again.

---

## Layout and rendering

| Aspect | Behaviour |
|---|---|
| Animation | Linear progress from egui's `animate_bool_with_time`; height `= natural_height × progress`, opacity `= progress`. |
| Natural height | Measured from the child `Ui` (frame margins included) and cached; remeasured only when it changes by more than `HEIGHT_EPSILON`. |
| `max_height` | The reserved and visible height never exceed it; the content is clipped to it. |
| Width | The available width. |
| Frame | Resolved from utility classes (Bootstrap or Tailwind) and wrapped around the content: shadow, fill and stroke are painted manually over the reserved rect so the height is exact. |
| Interaction | Static container (`Sense::hover()`); hover/focus/disabled styling belongs to the content. |

### First expansion, one frame late

The natural height is learned from the previous frame. On the **first** frame
of the **first** expansion there is no cached height, so the content is drawn
invisibly (clip height 0, opacity 0), measured, and cached; the animation then
starts from a known height on the next frame. The delay is one frame (≈16 ms),
imperceptible, and after the first expansion the collapse animates correctly
forever.

### Why the frame is painted manually

`egui::Frame::show` reserves the rect its content used, which is the *natural*
height — wrong for the animating path, where only `natural × progress` should
be reserved, and wrong for `max_height`, where the content must be clipped. So
`MudCollapse` allocates exactly the visible rect first, then paints the frame's
shadow, fill and stroke into it, then clips the content child to the same rect.

---

## Examples

### Basic

```rust
ui.checkbox(&mut self.open, "Show details");
MudCollapse::new()
    .expanded(self.open)
    .id_salt("details")
    .show(ui, |ui| {
        ui.add(MudText::body2("Hidden until expanded."));
    });
```

### Reacting to events

```rust
let out = MudCollapse::new()
    .expanded(self.open)
    .id_salt("details")
    .show(ui, |ui| {
        ui.add(MudText::body2("Details"));
    });

if let Some(open) = out.expanded_changed {
    log::info!("expanded changed to {open}");
}
if out.animation_ended {
    self.scroll_into_view = true;
}
```

### Limited height (tall content stays clipped)

```rust
MudCollapse::new()
    .expanded(true)
    .max_height(120.0)
    .id_salt("log")
    .show(ui, |ui| {
        for line in &self.lines {
            ui.add(MudText::monospace(line.clone()));
        }
    });
```

### Accordion: header + collapse

```rust
for (i, item) in self.items.iter().enumerate() {
    if ui.button(&item.title).clicked() {
        self.open = if self.open == Some(i) { None } else { Some(i) };
    }
    MudCollapse::new()
        .expanded(self.open == Some(i))
        .id_salt(("faq", i))
        .classes(["p-2"])
        .show(ui, |ui| {
            ui.add(MudText::body2(&item.body));
        });
}
```

### Instant switch

```rust
MudCollapse::new()
    .expanded(self.open)
    .duration(0.0)     // no animation, no OnAnimationEnd
    .id_salt("instant")
    .show(ui, |ui| { ui.label("…"); });
```

### Tailwind surface

```rust
MudCollapse::new()
    .expanded(self.open)
    .flavor(StyleFlavor::Tailwind)
    .classes(["bg-surface", "rounded-lg", "p-4", "hover:bg-surface/90"])
    .id_salt("tw")
    .show(ui, |ui| { ui.label("…"); });
```

The `hover:` variant is fed from the previous frame's state, exactly as
`MudPaper` documents.

### Tag round-trip

```rust
let c = MudCollapse::new().duration(0.0).tag("advanced-options");
assert_eq!(c.tag_ref::<&str>(), Some(&"advanced-options"));
c.expanded(true).id_salt("adv").show(ui, |ui| { ui.label("…"); });
```

---

## Performance and memory choices

| Choice | Effect |
|---|---|
| Collapsed path skips the closure | A hidden panel costs one animation lookup and a zero-size allocation. |
| One measurement per frame | The closure is `FnOnce` and is called exactly once; heights are cached between frames. |
| Noise-tolerant cache updates | Heights are only rewritten (and a repaint requested) when they change by more than `HEIGHT_EPSILON`, so sub-pixel float noise cannot cause a repaint loop. |
| Three scal

===== ./crates/mud_egui_components/src/surfaces/expansion_panels/README.md =====

# MudExpansionPanels + MudExpansionPanel for `mud_egui`

An accordion container and its collapsible panel for
[egui](https://github.com/emilk/egui), ported from MudBlazor's
[`MudExpansionPanels`](https://mudblazor.com/components/expansion-panel) and
`MudExpansionPanel` (MudBlazor 8.13.0).

```rust
use mud_egui_components::surfaces::expansion_panels::{
    MudExpansionPanel, MudExpansionPanels,
};

MudExpansionPanels::new().show(ui, &mut self.state, |panels| {
    panels.add(MudExpansionPanel::new("Shipping"), |ui| {
        ui.add(MudText::body2("Arrives in 2–3 business days."));
    });
    panels.add(MudExpansionPanel::new("Returns"), |ui| {
        ui.add(MudText::body2("30 days, unused condition."));
    });
});
```

Every string renders through **`MudText`**, every glyph through **`MudIcon`**.

---

## Contents

1. [Design principle: API parity, not API invention](#design-principle-api-parity-not-api-invention)
2. [Architecture](#architecture)
3. [API reference — MudExpansionPanels](#api-reference--mudexpansionpanels)
4. [API reference — MudExpansionPanel](#api-reference--mudexpansionpanel)
5. [Examples](#examples)
6. [Cascading and state](#cascading-and-state)
7. [Layout and rendering](#layout-and-rendering)
8. [Performance and memory choices](#performance-and-memory-choices)
9. [Testing](#testing)
10. [Differences from MudBlazor](#differences-from-mudblazor)

---

## Design principle: API parity, not API invention

Both components were verified against DNDocs for MudBlazor 8.13.0 before
implementation. Between them they expose **nineteen** parameters, and all
nineteen are ported.

### `MudExpansionPanels`

| MudBlazor | Type | Default | Rust mapping |
|---|---|---|---|
| `ChildContent` | `RenderFragment?` | `null` | closure to `show(ui, state, \|panels\| ..)` |
| `Dense` | `bool` | `false` | `dense(bool)` |
| `Elevation` | `int` | **`1`** | `elevation(u8)` (clamped 0–24) |
| `Gutters` | `bool` | `true` | `gutters(bool)` |
| `MultiExpansion` | `bool` | `false` | `multi_expansion(bool)` |
| `Outlined` | `bool` | **`true`** | `outlined(bool)` |
| `Square` | `bool` | `false` | `square(bool)` |
| `Class` | `string?` | `null` | `classes(iter)` — stored |
| `Tag` | `object?` | `null` | `tag(t)` / `tag_ref::<U>()` |

### `MudExpansionPanel`

| MudBlazor | Type | Default | Rust mapping |
|---|---|---|---|
| `ChildContent` | `RenderFragment?` | `null` | closure to `show` / `ExpansionPanelsUi::add` |
| `Dense` | `bool` | `false` | `dense(bool)` |
| `Disabled` | `bool` | `false` | `disabled(bool)` |
| `Expanded` | `bool` | `false` | caller-owned `&mut bool` |
| `ExpandedChanged` | `EventCallback<bool>` | — | `PanelResponse::expanded_changed` |
| `Gutters` | `bool` | `true` | `gutters(bool)` |
| `HeaderClass` | `string?` | `null` | `header_class(s)` — stored |
| `HideIcon` | `bool` | `false` | `hide_icon(bool)` |
| `Icon` | `string` | `ExpandMore` | `icon(s)` |
| `KeepContentAlive` | `bool` | — | `keep_content_alive(bool)` — **inert** |
| `MaxHeight` | `int?` | `null` | `max_height(f32)` → `Option<f32>` |
| `Text` | `string?` | `null` | `new(text)` / `text(s)` |
| `TitleContent` | `RenderFragment?` | `null` | `title_content(f)` — overrides `Text` |
| `Class` | `string?` | `null` | `classes(iter)` — stored |
| `Tag` | `object?` | `null` | `tag(t)` / `tag_ref::<U>()` |

### Deliberately not ported

| MudBlazor | Reason |
|---|---|
| `Style` (both) | egui has no inline CSS strings. |
| `UserAttributes` (both) | There is no HTML tag to splat attributes onto. |
| `Class` / `HeaderClass` resolution | Stored for parity only. The container owns fill, border, shadow and radius; the panel owns padding and its divider. Resolving a second utility frame would composite two fills or two margins. |
| `Panels` (container) | It is a read-only view of the children. In egui the children are the caller's `&mut Vec<bool>`. |
| Protected `Classname` / `HeaderClassname` / `PanelContentClassname` | Internal CSS class getters with no egui analogue. |
| `NextPanelExpanded` | An internal bookkeeping flag used to decide border radii between adjacent panels. The container computes it directly from the panel index. |

### Ported as free functions

`ExpandAllAsync`, `CollapseAllAsync` and `CollapseAllExceptAsync` are methods on
the container instance in Blazor, reachable through `@ref`. In immediate mode
there is no instance to hold, so they become functions over the caller's state
slice: `expand_all(&mut [bool])`, `collapse_all(&mut [bool])`,
`collapse_all_except(&mut [bool], usize)`.

### `KeepContentAlive` is inert

In Blazor it keeps the collapsed content in the DOM. In egui the content
closure is **never called** while the panel is collapsed, and any widget state
is keyed by `Id` in `Memory`, so it survives collapse and expand on its own.
There is nothing for the flag to switch off. It is accepted and stored so a
Blazor call site ports without edits, and documented here rather than silently
dropped.

---

## Architecture

```
surfaces/expansion_panels/
├── mud_expansion_panel.rs
│   ├── constants        HEADER_HEIGHT 48, DENSE_HEADER_HEIGHT 32,
│   │                    CONTENT_PADDING_Y 16/8, GUTTER_PX 16,
│   │                    ICON_SIZE 24/20, ICON_GAP 8, PANEL_RADIUS 4,
│   │                    ICON_ROTATION_DEG 180, HAIRLINE 1
│   ├── pure helpers     header_height, content_padding_y, horizontal_padding,
│   │                    icon_size, next_expanded, icon_rotation,
│   │                    radius_u8, padding_i8, placed_width
│   ├── MudExpansionPanel  builder + show (standalone) + render (in container)
│   ├── PanelChrome      container-supplied overrides (Copy)
│   └── PanelResponse    header Response + inner + toggled + expanded_changed
└── mud_expansion_panels.rs
    ├── MudExpansionPanels  builder + show
    ├── ExpansionPanelsUi   collector: add(panel, content)
    ├── PanelEntry<'a>      panel + Box<dyn FnOnce(&mut Ui) + 'a>
    ├── PanelSetResponse    container Response + inner + toggled
    ├── bulk ops            expand_all, collapse_all, collapse_all_except
    └── pure helpers        sync_state_len, container_radius, elevation_shadow
```

**Pure layer.** Header geometry, the click-to-state rule, the icon angle,
state-vector resizing, the accordion rule and the shadow ramp are all free
functions over `f32`, `bool` and `&mut [bool]`. None touch `egui::Context`, so
they are unit-tested without a window.

**State layer.** Expansion lives in the caller's `&mut Vec<bool>`, indexed by
declaration order and resized to the panel count at the top of every frame.

**Component layer.** The container collects panels, resizes state, draws one
`Frame` around the whole stack, then renders each panel with a `PanelChrome`
describing the cascade and whether the panel is last.

**Content layer.** Each panel's content region is delegated to
[`MudCollapse`](../collapse/README.md), so the animation, the height cache and
the `MaxHeight` clamp are shared rather than reimplemented.

### Why the panel has two entry points

| Entry point | Used by | Surface |
|---|---|---|
| `MudExpansionPanel::show` | standalone panels | own fill, own 1 pt outline, 4 pt radius |
| `MudExpansionPanel::render` (`pub(crate)`) | `MudExpansionPanels` | fill only; the container draws the border, shadow and radius; the panel draws a 1 pt divider when it is not last |

A container that drew each panel's full border would show a 2 pt seam where two
panels meet, because both hairlines would be painted. Splitting the surface
this way gives the "one rounded card, internal rules" look that
`.mud-expansion-panels` produces in Blazor.

---

## API reference — `MudExpansionPanels`

| Method | Type | Default | Mirrors | Description |
|---|---|---|---|---|
| `new()` | `Self` | — | — | Accordion, elevation 1, gutters on, outlined, rounded. |
| `dense(b)` | `bool` | `false` | `Dense` | Compact padding on every panel. |
| `elevation(e)` | `u8` | `1` | `Elevation` | Stack shadow, clamped to 24. `0` = flat. |
| `gutters(b)` | `bool` | `true` | `Gutters` | Left/right padding on every panel. |
| `multi_expansion(b)` | `bool` | `false` | `MultiExpansion` | Allow several panels open at once. |
| `outlined(b)` | `bool` | `true` | `Outlined` | 1 pt border around the stack. |
| `square(b)` | `bool` | `false` | `Square` | Zero corner radius on the stack. |
| `classes(iter)` | `iter of AsRef<str>` | none | `Class` | Stored on `ComponentBase`. |
| `tag(t)` / `tag_ref::<U>()` | `T: Any + Send + Sync` | `None` | `Tag` | Arbitrary user data. |
| `show(ui, state, \|panels\| ..)` | `&mut Ui, &mut Vec<bool>, FnOnce(&mut ExpansionPanelsUi) -> R` | — | `ChildContent` | Returns `PanelSetResponse<R>`. |

### `ExpansionPanelsUi`

| Method | Description |
|---|---|
| `add(panel, content)` | Registers a panel and its content. Nothing is drawn until the builder closure returns. |
| `len()` / `is_empty()` | Number of panels registered so far. |

The content closure is `FnOnce(&mut Ui) + 'a`, where `'a` is the lifetime of the
`show` call — it can borrow local state, so it does **not** need to be
`'static`.

### `PanelSetResponse<R>`

| Field | Type | Description |
|---|---|---|
| `response` | `Response` | The whole stack, border included. |
| `inner` | `R` | The builder closure's return value. |
| `toggled` | `Option<usize>` | Index of the panel whose header toggled this frame. |

### Bulk operations

| Function | Mirrors | Description |
|---|---|---|
| `expand_all(&mut [bool])` | `ExpandAllAsync` | Opens every panel. |
| `collapse_all(&mut [bool])` | `CollapseAllAsync` | Closes every panel. |
| `collapse_all_except(&mut [bool], usize)` | `CollapseAllExceptAsync` | Closes all but one; out-of-range indices are ignored. |

---

## API reference — `MudExpansionPanel`

| Method | Type | Default | Mirrors | Description |
|---|---|---|---|---|
| `new(text)` | `impl Into<String>` | `""` | `Text` | Creates a panel with a text header. |
| `text(s)` | `impl Into<String>` | — | `Text` | Replaces the header caption. |
| `title_content(f)` | `FnMut(&mut Ui) + 'static` | `None` | `TitleContent` | Custom header; overrides `Text`. |
| `dense(b)` | `bool` | `false` | `Dense` | 32 pt header, 8 pt content padding, 20 pt icon. |
| `disabled(b)` | `bool` | `false` | `Disabled` | Header ignores clicks. |
| `gutters(b)` | `bool` | `true` | `Gutters` | 16 pt left/right padding. |
| `header_class(s)` | `impl Into<String>` | `None` | `HeaderClass` | Stored, not resolved. |
| `hide_icon(b)` | `bool` | `false` | `HideIcon` | Hides the expand glyph (clicks still work). |
| `icon(s)` | `impl Into<String>` | `ExpandMore` | `Icon` | Expand glyph override. |
| `keep_content_alive(b)` | `bool` | `false` | `KeepContentAlive` | Inert in egui. |
| `max_height(px)` | `f32` | `None` | `MaxHeight` | Content height cap in points. |
| `id_salt(s)` | `impl Hash + Debug` | `None` | — | Stable animation identity. The container sets one automatically. |
| `classes(iter)` | `iter of AsRef<str>` | none | `Class` | Stored on `ComponentBase`. |
| `tag(t)` / `tag_ref::<U>()` | `T: Any + Send + Sync` | `None` | `Tag` | Arbitrary user data. |
| `show(ui, &mut bool, \|ui\| ..)` | `&mut Ui, &mut bool, FnOnce(&mut Ui) -> R` | — | — | Standalone render; returns `PanelResponse<R>`. |

### `PanelResponse<R>`

| Field | Type | Description |
|---|---|---|
| `response` | `Response` | The header. `PanelResponse` derefs to it. |
| `inner` | `Option<R>` | The content closure's value; `None` while fully collapsed. |
| `expanded` | `bool` | The state after this frame. |
| `toggled` | `bool` | `true` when the header toggled the state this frame. |
| `expanded_changed` | `Option<bool>` | `Some(new)` on the toggling frame. Mirrors `ExpandedChanged`. |

### Constants

| Name | Value |
|---|---|
| `HEADER_HEIGHT` / `DENSE_HEADER_HEIGHT` | `48.0` / `32.0` |
| `CONTENT_PADDING_Y` / `DENSE_CONTENT_PADDING_Y` | `16.0` / `8.0` |
| `GUTTER_PX` | `16.0` |
| `ICON_SIZE` / `DENSE_ICON_SIZE` | `24.0` / `20.0` |
| `ICON_GAP` | `8.0` |
| `PANEL_RADIUS` / `CONTAINER_RADIUS` | `4.0` / `4.0` |
| `ICON_ROTATION_DEG` | `180.0` |
| `HAIRLINE` | `1.0` |
| `MAX_ELEVATION` | `24` |

---

## Examples

### Accordion (the default)

```rust
MudExpansionPanels::new().show(ui, &mut self.state, |panels| {
    panels.add(MudExpansionPanel::new("One"), |ui| { ui.label("1"); });
    panels.add(MudExpansionPanel::new("Two"), |ui| { ui.label("2"); });
});
```

Opening `Two` closes `One` automatically.

### Multi-expansion

```rust
MudExpansionPanels::new()
    .multi_expansion(true)
    .show(ui, &mut self.state, |panels| {
        for (i, section) in self.sections.iter().enumerate() {
            panels.add(MudExpansionPanel::new(&section.title), |ui| {
                ui.add(MudText::body2(&section.body));
            });
            let _ = i;
        }
    });
```

### Opening one panel from a button

```rust
if ui.button("Show returns policy").clicked() {
    collapse_all(&mut self.state);
    self.state[2] = true;          // index 2 = "Returns"
}
```

### Expand all / collapse all

```rust
ui.horizontal(|ui| {
    if ui.button("Expand all").clicked() {
        expand_all(&mut self.state);
    }
    if ui.button("Collapse all").clicked() {
        collapse_all(&mut self.state);
    }
});
```

### Flat, square, flush stack

```rust
MudExpansionPanels::new()
    .elevation(0)
    .outlined(false)
    .square(true)
    .gutters(false)
    .show(ui, &mut self.state, |panels| { /* … */ });
```

### Dense stack with a height-capped panel

```rust
MudExpansionPanels::new()
    .dense(true)
    .show(ui, &mut self.state, |panels| {
        panels.add(
            MudExpansionPanel::new("Build log").max_height(160.0),
            |ui| {
                for line in &self.log {
                    ui.add(MudText::monospace(line.clone()));
                }
            },
        );
    });
```

### Custom header

```rust
panels.add(
    MudExpansionPanel::new("")
        .title_content(|ui| {
            ui.add(MudIcon::new(filled::SHIPPING).size_px(18.0));
            ui.add(MudText::subtitle1("Shipping"));
            ui.add(MudText::caption("2–3 days").color(Color::TextSecondary));
        }),
    |ui| { ui.label("…"); },
);
```

### Disabled panel

```rust
panels.add(
    MudExpansionPanel::new("Enterprise (contact sales)").disabled(true),
    |ui| { ui.label("…"); },
);
```

The header still renders and hovers, but the state never changes.

### Custom expand glyph

```rust
MudExpansionPanel::new("Filters").icon(filled::ADD).show(ui, &mut open, |ui| {
    ui.label("…");
});
```

The glyph rotates 180° as the panel expands, so pick a symmetric one.

### Standalone panel (no container)

```rust
let mut open = false;
MudExpansionPanel::new("Standalone").show(ui, &mut open, |ui| {
    ui.label("Own fill, own border, own radius.");
});
```

### Reacting to a toggle

```rust
let out = MudExpansionPanels::new().show(ui, &mut self.state, |panels| { /* … */ });
if let Some(index) = out.toggled {
    log::info!("panel {index} toggled to {}", self.state[index]);
}
```

---

## Cascading and state

### Cascade rules

`Dense` and `Gutters` are non-nullable on both types, so the container folds
onto the panel rather than "filling in" an unset value:

| Container | Panel | Effective |
|---|---|---|
| `dense(false)` | `dense(false)` | standard |
| `dense(true)` | `dense(false)` | **dense** — container wins |
| `dense(false)` | `dense(true)` | **dense** |
| `gutters(true)` | `gutters(true)` | padded |
| `gutters(false)` | `gutters(true)` | **no padding** — container wins |
| `gutters(true)` | `gutters(false)` | **no padding** |

A panel can opt into density and opt out of gutters, but not the reverse. That
is the only reading consistent with MudBlazor's wording for non-nullable flags.

`Outlined`, `Elevation` and `Square` are container-only in 8.13.0 and apply to
the stack, not to individual panels.

### State vector

```rust
let mut state: Vec<bool> = Vec::new();   // grow it however you like
MudExpansionPanels::new().show(ui, &mut state, |panels| { /* … */ });
// After the call, state.len() == number of panels registered.
```

`show` calls `state.resize(count, false)` before drawing, so a panel added this
frame starts collapsed and a removed panel's flag is dropped. Keep the
declaration order stable if you want a given flag to keep belonging to the same
panel.

### Accordion enforcement

When `multi_expansion` is off and a panel reports `toggled` **to** expanded, the
container sets every other flag to `false` in the same frame, before those
panels are rendered — so they draw collapsed immediately, with no one-frame lag.

### `ExpandedChanged` on the panel

`PanelResponse::expanded_changed` is `Some(new)` only on the frame the header
clicked. It is never set on a programmatic change (writing `state[i] = true`
yourself), because the panel sees the flag already set — the same way an
`ExpandedChanged` callback fires in Blazor only from a user interaction or an
explicit `ExpandAsync()` call.

---

## Layout and rendering

### Header

```
LTR:  [ pad_x ][ title area …………………………… ][ gap 8 ][ icon 24 ][ pad_x ]
RTL:  not mirrored (see Differences)
```

| Density | Header height | Icon side |
|---|---|---|
| standard | 48 pt | 24 pt |
| dense | 32 pt | 20 pt |

- Hover paints `widgets.hovered.weak_bg_fill` over the header rect, behind the
  title and the icon.
- A non-disabled header gets `Sense::click()` and a pointing-hand cursor; a
  disabled header gets `Sense::hover()` only.
- `HideIcon(true)` removes both the glyph **and** its reserved column, so the
  title area grows to fill the space. Clicks still toggle.
- The icon rotates `180° × animate_bool_with_time(id, expanded, 0.2)`.

### Content

```
[ header ]
[ content, padding_y 16 (8 dense), delegated to MudCollapse ]
[ 1 pt divider, if inside a container and not last ]
```

The content region is a `MudCollapse`, so:

- it is clipped and faded while animating;
- its natural height is cached and only remeasured when it moves by more than
  0.1 pt;
- `MaxHeight` clamps both the reserved and the visible height;
- the closure is not called while fully collapsed.

### Stack

The container wraps all panels in one `egui::Frame` with:

| Container flag | Frame property |
|---|---|
| (always) | `fill = palette.surface` |
| `outlined(true)` | `stroke = 1 pt, widgets.noninteractive.bg_stroke.color` |
| `elevation > 0` | `shadow = shadow_ramp(elevation)` |
| `square(false)` | `corner_radius = 4 pt` |

Panels inside it paint no surface of their own. A 1 pt divider is drawn under
every panel except the last, in the same colour as the outline, so the stack
reads as one rounded card with internal rules.

---

## Performance and memory choices

| Choice | Effect |
|---|---|
| Two-phase render | Panels are collected before any is drawn, so accordion enforcement and the "is this the last panel" divider rule are both correct within one frame. |
| Non-`'static` content closures | `Box<dyn FnOnce(&mut Ui) + 'a>` borrows for the length of the `show` call, so a panel's content can read `&self` instead of cloning into `move` closures or `Rc<RefCell<_>>`. |
| Collapsed panels are free | `MudCollapse` skips the content closure entirely once settled, so twenty closed panels cost twenty header rows and nothing else. |
| One `Frame` for the whole stack | One fill, one stroke, one shadow — not one per panel. |
| One `MudCollapse` per panel | The height cache, the epsilon and the `MaxHeight` clamp are shared with `MudCollapse` rather than duplicated. |
| Header is one allocation | `allocate_exact_size` for the header rect plus a child `Ui` for the title; no nested layouts, no per-frame `Vec`. |
| `mem::take` for the caption | The `String` is moved into `MudText` rather than cloned. |
| Pure helpers on `Copy` values | Header geometry, the toggle rule, the icon angle and state resizing are arithmetic; unit-testable without a `Context`. |
| `state.resize` is O(delta) | Growing by one panel appends one `false`; the steady state does no work. |

---

## Testing

`cargo test -p mud_egui_components mud_expansion` runs 26 tests:

| File | Area | Tests |
|---|---|---|
| `mud_expansion_panel.rs` | Metrics | header height per density; content padding per density; gutter padding; icon size per density; `radius_u8` / `padding_i8` clamping; `placed_width` |
| | Toggle rule | `next_expanded` truth table; disabled ignores clicks |
| | Icon | rotation spans 0–180; clamps and sanitizes `NaN` |
| | Chrome | `PanelChrome::standalone` has no overrides |
| | Builders | defaults match MudBlazor; every field set; `id_salt` stability; tag round-trip |
| `mud_expansion_panels.rs` | Builders | defaults (elevation **1**, outlined **true**); every field set; tag round-trip |
| | State | `sync_state_len` grows with `false`, shrinks, is idempotent |
| | Bulk ops | `expand_all`, `collapse_all`, `collapse_all_except` (including out-of-range and empty) |
| | Chrome | container radius; shadow `None` at 0 and clamped at 24 |
| | Cascade | container dense forces dense; container gutters-off forces flush; panel can opt in to dense and out of gutters |

No test creates an `egui::Context`. `show` and `render` need a live `Ui` and are
left to integration tests; everything they delegate to is covered above.

Suggested integration tests (`egui_kittest` or a headless `Context::run`
harness):

- Clicking a header flips that panel and, in accordion mode, closes the others
  in the same frame.
- `multi_expansion(true)` lets two panels be open at once.
- A disabled panel's header click leaves the state untouched.
- `HideIcon(true)` widens the title area and still toggles.
- `MaxHeight` caps the expanded content and clips the overflow.
- A stack of three panels draws exactly two dividers.

```bash
cargo test -p mud_egui_components mud_expansion
```

---

## Differences from MudBlazor

- **`Expanded` / `ExpandedChanged` become a `&mut bool` plus a return field.**
  The caller owns the flag; `PanelResponse::expanded_changed` reports the
  toggling frame. This is the same split `MudCollapse` and `MudAlert` use, and
  it means `MudExpansionPanels` never stores hidden state.
- **`ExpandAllAsync` / `CollapseAllAsync` / `CollapseAllExceptAsync` are free
  functions over `&mut [bool]`.** There is no instance to hold in immediate
  mode; the caller owns the vector.
- **`Panels` is the caller's vector.** MudBlazor exposes a read-only list of
  child components; here the children *are* the state.
- **`Class` and `HeaderClass` are stored but not resolved.** The container owns
  fill, border, shadow and radius, and the panel owns padding and its divider.
  Resolving another utility frame would composite a second fill or a second
  margin.
- **`Style` and `UserAttributes` are not ported.** No CSS strings, no HTML tag.
- **`KeepContentAlive` is inert.** In egui the content closure is simply not
  called while collapsed, and widget state survives in `Memory` regardless.
- **`Elevation` and `Outlined` are `u8` / `bool` with the same defaults as
  MudBlazor (`1` and `true`).** Elevation is clamped to `0..=24` like every
  other component in this workspace.
- **`MaxHeight` is `f32` points, not `int` pixels.** Consistent with
  `MudCollapse`, `MudStack` and `MudCardMedia`.
- **No `Ripple` parameter.** MudBlazor 8.13.0's `MudExpansionPanel` does not
  have one; adding it would be API invention. The header's hover wash is the
  only interaction feedback.
- **One `Icon`, not an expand/collapse pair.** 8.13.0 exposes a single `Icon`
  (default `ExpandMore`), which this port rotates 180° when expanded — matching
  `.mud-expand-panel-icon`'s CSS transform. If your MudBlazor build shows a
  static icon, delete the `.rotation(..)` call in `render`; it is one line.
- **No RTL mirroring.** `MudExpansionPanel` has no `RightToLeft` parameter in
  8.13.0 (unlike `MudTimeline`, which does), so the header is always
  left-to-right. Wrap the stack in an RTL-aware parent if you need the mirror.
- **Panel surfaces are split between the container and the panel.** Standalone
  panels draw their own fill, outline and radius; inside a container the
  container draws one border, shadow and radius and the panel draws only a
  divider. This avoids the 2 pt seam you get when two hairlines meet, and
  reproduces MudBlazor's "one rounded card, internal rules" look.
- **Accordion enforcement happens before rendering.** Because panels are
  collected first, closing the others takes effect in the same frame rather
  than one frame later.

===== ./crates/mud_egui_components/src/surfaces/paper/README.md =====

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

===== ./crates/mud_egui_components/src/typography/README.md =====

# MudText for `mud_egui`

A themed, Material Design text component for [egui](https://github.com/emilk/egui), ported from MudBlazor's `MudText`. It provides a fluent builder API for applying semantic typography scales, semantic colors, text alignment, and standard text decorations.

```rust
MudText::h3("Welcome!")
    .color(Color::Primary)
    .bold(true)
    .ui(ui);

```

In addition to explicit builder methods, it features a mini utility-class engine that parses standard spacing (`p-4`, `mx-2`) and background color (`bg-primary`) strings.

---

## Contents

1. [What maps to MudBlazor, and what does not](https://www.google.com/search?q=%23what-maps-to-mudblazor-and-what-does-not)
2. [Architecture](https://www.google.com/search?q=%23architecture)
3. [API reference](https://www.google.com/search?q=%23api-reference)
4. [Examples](https://www.google.com/search?q=%23examples)
5. [Performance considerations](https://www.google.com/search?q=%23performance-considerations)
6. [Design rationale](https://www.google.com/search?q=%23design-rationale)

---

## What maps to MudBlazor, and what does not

The public MudBlazor API was checked to maintain parity. `MudText` exposes `Typo`, `Color`, `Align`, `GutterBottom`, `Inline`, `HtmlTag`, and cascading right-to-left settings.

| MudBlazor | This crate | Notes |
| --- | --- | --- |
| `Typo` | `MudText::typo(Typo)` | Also available via `h1()`..`overline()` constructors

|
| `Color` | `MudText::color(Color)` | Plus `custom_color(Color32)` to bypass the theme

|
| `Align` | `MudText::align(MudAlign)` | Has no effect on inline displays

|
| `GutterBottom` | `MudText::gutter_bottom(bool)` | Adds a `0.35em` bottom margin

|
| `Inline` | `MudText::inline(bool)` | Disables horizontal expansion and alignment

|
| `HtmlTag` | `MudText::html_tag("...")` | Inline-level tags (e.g., `span`, `b`) behave like `inline(true)`

|
| `Class` | `MudText::classes([...])` | Parsed for padding, margins, and background utility strings

|
| `UserAttributes` | `MudText::id(...)`, `aria_label(...)` | `id` is a stable salt, `aria_label` is for screen readers

|
| `ChildContent` | Closure passed to `MudText::new(..)` | Accepts `impl Into<String>`

|

**Rust-only Additions:** `opacity`, `tooltip`, `selectable`, `sense`, `id`, `aria_label`, and direct font overrides (`bold`, `italic`, `uppercase`, `font_size`, `font_family`) are `egui`-native conveniences.

**Deliberately not added:** MudBlazor's `MudText` has no value binding, callbacks, `Variant`, `Size`, `Elevation`, `Disabled`, or `ReadOnly`, so they are not invented here.

---

## Architecture

Rendering is a two-stage pipeline to separate pure logic from immediate-mode UI allocations:

1. **Resolve (`MudText::resolve_style`):** A pure function from `(component state, theme, fallback text color)` to a `ResolvedStyle`. It never touches an `egui::Ui`, never allocates (except cloning an `Arc` for font families), and is completely unit-testable. Precedence is always `builder override > theme variant`.


2. **Render (`MudText::ui` / `MudText::ui_with_theme`):** Turns the resolved style into an `egui::Label`, optionally wrapped in alignment and utility `egui::Frame` layers.



---

## API reference

### Shorthand Constructors

Instead of `MudText::new("...").typo(Typo::H1)`, you can use shorthand constructors:

* `h1` through `h6`
* `subtitle1`, `subtitle2`
* `body1`, `body2`
* `button`, `caption`, `overline`

### Modifier Methods

Every setter consumes and returns `Self` (fluent builder pattern).

| Method | Description |
| --- | --- |
| `custom_color(Color32)` | Explicitly bypasses semantic `MudTheme` colors.

|
| `truncate(bool)` | Overflows are truncated with an ellipsis (`...`).

|
| `no_wrap(bool)` | Prevents text from wrapping to a new line.

|
| `underline(bool)` / `strikethrough(bool)` | Adds text decoration lines.

|
| `selectable(bool)` | Allows the user to highlight and copy the text.

|
| `monospace(bool)` | Forces fixed-width typography.

|
| `opacity(f32)` | `[0.0, 1.0]` opacity multiplier; `NaN` is ignored.

|
| `sense(Sense)` | Attaches interaction tracking (e.g., `Sense::click()`) to the label.

|
| `tooltip(String)` | Attaches an on-hover tooltip string.

|

---

## Examples

### Themed Typography

```rust
ui.add(MudText::h1("Main Title").color(Color::Primary));
ui.add(MudText::body2("Standard paragraph text here."));

```

### Uppercase & Inline Styles

```rust
// Typo::Button defaults to uppercase; uppercase(false) disables it.
MudText::button("Submit").uppercase(false).ui(ui);

// Inline elements only take the space they need.
MudText::new("A span").html_tag("span").ui(ui);

```

### Utility Classes

```rust
MudText::h4("Highlighted")
    .classes(["p-4", "mx-2", "bg-primary"])
    .ui(ui);

```

### Capturing Interactivity

```rust
let response = MudText::body1("Click me!")
    .sense(Sense::click())
    .id("clickable-text")
    .ui(ui);

if response.clicked() {
    println!("Text was clicked!");
}

```

---

## Performance considerations

* **Zero-copy Fast Path:** The text buffer is *moved* into `egui::RichText` via `std::mem::take` (zero allocations, zero copies) unless an uppercase transform is explicitly required.


* **Shared Theme Lookups:** [`MudText::ui_with_theme`] allows a caller drawing multiple text nodes (e.g., lists or tables) to look up the `MudTheme` once and share it across all texts, avoiding per-widget context fetching.


* **Sanitized Inputs:** Invalid numeric overrides (`NaN`, infinite values, zero, or negative font sizes) are rejected during the resolution stage so rendering logic never sees them.



---

## Design rationale

* **Pure State Resolution:** `ResolvedStyle` encapsulates exact point sizes, weights, layouts, and colors. This separates complex theme precedence (e.g., specific font family vs. monospace toggle vs. explicit overrides) from rendering side effects.


* **HTML Semantic Tags:** `is_inline_html_tag` mimics web browser behaviors by automatically switching layout modes to inline when tags like `a`, `span`, or `code` are supplied.


* **No Interior Mutability:** The `MudText` builder uses plain properties, enabling intermediate variable bindings if you need to reuse partial configurations across several UI elements.

===== ./crates/mud_egui_navigation/src/README.md =====

# egui-routing

**An in-memory, type-safe URL router for [`egui`](https://github.com/emilk/egui), inspired by Avalonia's route host and Blazor's `NavigationManager`.**

Desktop and embedded `egui` apps have no address bar, but they still benefit from the mental model of URLs: a page is a *route*, parameters are *typed*, "back" and "forward" are a *history*, and "are you sure you want to leave?" is a *navigation guard*. `egui-routing` brings that model to immediate-mode UIs with a small, dependency-light core (`egui` is the only dependency) and no global state.

```text
 navigate_to("/orders/42?filter=active")
        │
        ▼
 NavigationManager ──(veto handlers may cancel)──► history updated
        │
        ▼  (polled once per frame — no events, no callbacks into the UI)
 Router::render ──► RouteTable lookup ──► Page (new or reused) inside Layout
```

---

## Table of contents

1. [Key features](#key-features)
2. [Architecture & module breakdown](#architecture--module-breakdown)
3. [Installation & project layout](#installation--project-layout)
4. [Quickstart](#quickstart)
5. [Route template grammar](#route-template-grammar)
6. [API reference summary](#api-reference-summary)
7. [Page lifecycle](#page-lifecycle)
8. [Performance & design rationale](#performance--design-rationale)
9. [Behavioural notes & fixes relative to the original port](#behavioural-notes--fixes-relative-to-the-original-port)
10. [Testing](#testing)
11. [Minimum supported Rust version](#minimum-supported-rust-version)
12. [License](#license)

---

## Key features

- **Typed route parameters.** `/orders/{id:int}` yields `RouteValue::Int(42)`. Supported constraints: `int`, `long`, `double`, `float`, `decimal`, `bool`, `guid`, `datetime`. A segment that fails its constraint simply does not match, so `/orders/abc` falls through to your not-found page.
- **Optional and catch-all segments.** `{id?}`, `{id:int?}` and `{*path}` are supported, with template validation at registration time (duplicate names, catch-all not last, required-after-optional, unknown constraints…).
- **Specificity-based matching.** Literals beat constrained parameters, which beat plain parameters, which beat optional ones, which beat catch-alls — regardless of registration order. Ambiguous registrations (`/a/{x:int}` vs `/a/{y:int}`) are rejected up front.
- **Pre-navigation vetoes (navigation guards).** Register any number of handlers with `register_veto`; each receives the target URI and the navigation kind and may call `prevent()`. Works for push, replace, back and forward. Handlers are panic-safe (a panicking handler cannot wedge the manager).
- **Page reuse.** Navigating between URIs that resolve to the same page type keeps the existing instance and calls `bind(ctx, reset_missing = true)` instead of constructing a new one. Switch it off with `.reuse_pages(false)`.
- **Query-string manipulation.** Order-preserving, case-insensitive parsing, and `NavigationManager::uri_with_query` to set/replace/remove a single parameter in one pass while preserving path and fragment.
- **Layout chrome.** A `Layout` wraps every routed page (sidebar, header, tab bar…) and lives across navigations, receiving its own `on_navigated_to` / `on_navigated_from` hooks.
- **Bounded history.** Browser-like back/forward stack with a configurable maximum size and forward-history truncation on push.
- **Frame-polled, event-free design.** No subscriptions or callbacks into the UI layer; the router compares URIs once per frame, which fits immediate-mode rendering naturally.
- **Custom not-found page** (or a built-in fallback that always shows the current path).
- **Dependency injection friendly.** `add_with::<T>(template, || T::new(services.clone()))`.
- **Safe Rust only.** No `unsafe`; no `unwrap`/`expect` in library code.

---

## Architecture & module breakdown

| File | Architectural role |
|------|--------------------|
| `navigation.rs` | **State & policy.** `NavigationManager` owns the bounded URL history under a base URI, resolves relative/absolute URIs, runs veto handlers, and exposes `uri_with_query`. Also defines `NavigationKind`, `NavigateError`, `LocationChangingContext` and `VetoHandle`. |
| `page.rs` | **Contracts.** The `Page` and `Layout` traits (lifecycle hooks + `show`) and `NavigationContext`, the read-only bundle of location, path, typed route values and query values handed to both. |
| `query.rs` | **Query-string toolkit.** Lazy borrowed iteration (`pairs` / `QueryPairs`), percent-decoding/encoding, first-value maps, `set_parameter`, single-pass `with_parameter`, `build`, and URI splitting helpers. |
| `route_table.rs` | **Registry.** `RouteTableBuilder` validates and registers templates → page factories, rejects ambiguity, and freezes them into an immutable, specificity-sorted `RouteTable` with `lookup` / `try_match`. |
| `route_template.rs` | **Parser & matcher.** Parses `/orders/{id:int}`-style templates, validates them, computes the canonical form and ambiguity *signature*, and matches paths without allocating on failure. |
| `route_value.rs` | **Value type.** `RouteValue` (`Str` / `Int` / `Float` / `Bool`) with lenient, allocation-free accessors. |
| `router.rs` | **The host.** `Router` polls the `NavigationManager` each frame, dispatches on change (lookup → context → hooks → create/reuse page) and renders the page inside the optional layout. |

Dependency direction (no cycles in the runtime types):

```text
route_value ◄── route_template ◄── route_table ◄───┐
     ▲                ▲                              │
     │                └──── query ◄── navigation     │
     └───────────── page ◄───────────────────────── router
```

(`route_table` refers to `Page` only through the `PageFactory` alias.)

---

## Installation & project layout

The sources live in a `routing` module of your crate (the files `use crate::routing::…`). Create `src/routing/mod.rs`:

```rust
pub mod navigation;
pub mod page;
pub mod query;
pub mod route_table;
pub mod route_template;
pub mod route_value;
pub mod router;
```

```toml
[dependencies]
egui = "*"      # use the version matching your eframe
eframe = "*"    # optional – any egui host works
```

---

## Quickstart

### 1. Define pages and (optionally) a layout

```rust
use crate::routing::navigation::NavigationManager;
use crate::routing::page::{Layout, NavigationContext, Page};
use egui::Ui;

#[derive(Default)]
struct HomePage;

impl Page for HomePage {
    fn show(&mut self, ui: &mut Ui, _ctx: &NavigationContext, nav: &mut NavigationManager) {
        ui.heading("Home");
        if ui.button("Open order #42").clicked() {
            // Push a new history entry. `Ok(false)` would mean a guard vetoed it.
            let _ = nav.navigate_to("/orders/42?filter=active", false);
        }
    }
}

/// A page with state bound from the route.
#[derive(Default)]
struct OrderPage {
    id: i64,
    filter: String,
}

impl Page for OrderPage {
    // Optional: copy route/query data into fields. Runs before `on_navigated_to`.
    // `reset_missing` is true when this instance is being reused.
    fn bind(&mut self, ctx: &NavigationContext, reset_missing: bool) {
        self.id = ctx.route_i64("id").unwrap_or_default();
        match ctx.query("filter") {
            Some(f) => self.filter = f.to_owned(),
            None if reset_missing => self.filter.clear(),
            None => {}
        }
    }

    fn on_navigated_to(&mut self, ctx: &NavigationContext) {
        println!("now showing {}", ctx.path);
    }

    fn on_navigated_from(&mut self) {
        println!("leaving order {}", self.id);
    }

    fn show(&mut self, ui: &mut Ui, _ctx: &NavigationContext, nav: &mut NavigationManager) {
        ui.heading(format!("Order #{}", self.id));
        ui.label(format!("filter = {:?}", self.filter));

        // Rewrite just one query parameter, keeping path + fragment.
        if ui.button("Show completed").clicked() {
            let uri = nav.uri_with_query("filter", Some("completed"));
            let _ = nav.navigate_to(&uri, true); // `true` = replace, don't grow history
        }
        ui.add_enabled_ui(nav.can_go_back(), |ui| {
            if ui.button("← Back").clicked() {
                let _ = nav.back();
            }
        });
    }
}

/// Chrome that survives navigations.
struct ShellLayout;

impl Layout for ShellLayout {
    fn show(&mut self, ui: &mut Ui, ctx: &NavigationContext, body: &mut dyn FnMut(&mut Ui)) {
        ui.horizontal(|ui| {
            ui.strong("My App");
            ui.weak(&ctx.path);
        });
        ui.separator();
        body(ui); // draw the routed page here
    }
}
```

### 2. Build the route table and the router

```rust
use crate::routing::{navigation::NavigationManager, route_table::RouteTable, router::Router};

fn build_router() -> Result<Router, String> {
    let mut routes = RouteTable::builder();
    routes.add::<HomePage>("/")?;
    routes.add::<OrderPage>("/orders/{id:int}")?;
    // Dependency injection: supply your own factory.
    // routes.add_with::<ReportPage>("/reports/{*path}", move || ReportPage::new(db.clone()))?;

    Ok(Router::new(routes.build())
        .with_layout(Box::new(ShellLayout))
        .not_found_default::<NotFoundPage>()) // or .not_found_with(|| NotFoundPage::new(..))
}

struct MyApp {
    router: Router,
    nav: NavigationManager,
}

impl MyApp {
    fn new() -> Self {
        Self {
            router: build_router().expect("valid route table"),
            // initial location, base URI, max history entries
            nav: NavigationManager::new("/", "app://my-app/", 64),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.router.render(ui, &mut self.nav); // once per frame
        });
    }
    // On newer eframe versions that provide `App::ui`, call `self.router.render(ui, &mut self.nav)` there.
}
```

### 3. Dynamic route parameters and query strings

```rust
// Template:  /orders/{id:int}          URI: /orders/42?filter=active&page=3
fn bind(&mut self, ctx: &NavigationContext, _reset: bool) {
    let id:   Option<i64>  = ctx.route_i64("id");          // Some(42)   (names are case-insensitive)
    let f:    Option<&str> = ctx.query("filter");          // Some("active")
    let page: u32          = ctx.query_parse("page").unwrap_or(1); // 3
    let raw  = ctx.route("id");                            // Some(&RouteValue::Int(42))
}
```

| Template | Matches | Captured |
|----------|---------|----------|
| `/orders/{id:int}` | `/orders/42` | `id = Int(42)` |
| `/orders/{id:int}` | `/orders/abc` | *no match* |
| `/users/{id?}` | `/users` and `/users/7` | `id` absent / `Str("7")` |
| `/files/{*path}` | `/files/a/b/c.txt` | `path = Str("a/b/c.txt")` |
| `/prices/{p:decimal}` | `/prices/9.99` | `p = Float(9.99)` |
| `/flags/{on:bool}` | `/flags/true` | `on = Bool(true)` |
| `/g/{id:guid}` | `/g/123E4567-E89B-12D3-A456-426614174000` | `id = Str("123e4567-…")` (lower-cased) |

### 4. Pre-navigation veto / navigation guards

```rust
use std::{cell::Cell, rc::Rc};

let dirty = Rc::new(Cell::new(false)); // shared with your editor page
let guard_flag = Rc::clone(&dirty);

let handle = nav.register_veto(move |change| {
    // `change.target_location` is the absolute target, `change.kind` is Push/Replace/Back/Forward.
    if guard_flag.get() && change.target_location.ends_with("/home") {
        change.prevent(); // cancels the navigation; remaining handlers are skipped
    }
});

match nav.navigate_to("/home", false) {
    Ok(true)  => { /* navigated (or already there) */ }
    Ok(false) => { /* vetoed: show a "discard changes?" dialog */ }
    Err(e)    => eprintln!("bad URI: {e}"),
}

// Later, when the guard is no longer needed:
nav.unregister_veto(handle);
```

Notes:

- `back()` / `forward()` are guarded too (`NavigationKind::Back` / `Forward`).
- Navigating to the URI that is already current is a no-op returning `Ok(true)`; handlers are not invoked.
- Handlers cannot call back into the manager (it is mutably borrowed while they run). Record the user's intent (e.g. set a flag) and navigate on a later frame.

---

## Route template grammar

| Syntax | Meaning |
|--------|---------|
| `orders` | Literal; ASCII case-insensitive |
| `{id}` | Required parameter, `RouteValue::Str` |
| `{id:int}` | Constrained parameter (`int`, `long`, `double`, `float`, `decimal`, `bool`, `guid`, `datetime`; case-insensitive) |
| `{id?}` / `{id:int?}` | Optional parameter. Only optional segments (or a catch-all) may follow an optional one |
| `{*path}` | Catch-all. Must be last; cannot be optional or constrained; captures the remaining segments joined by `/` (possibly empty) |

Names must match `[A-Za-z_][A-Za-z0-9_]*`, are case-insensitive and unique per template. `guid` validates the 8-4-4-4-12 hex shape; `datetime` checks only that the segment has ≥10 characters and starts with a digit (no `uuid` / `chrono` dependency) — parse the string in your page if you need a real date.

Empty path segments (`//`) and leading/trailing slashes are ignored. Path segments are percent-decoded; `+` is **literal** in paths (it is a space only in query strings).

---

## API reference summary

### `NavigationManager` (`navigation.rs`)

| Item | Signature | Notes |
|------|-----------|-------|
| constructor | `new(initial_uri: &str, base_uri: &str, max_history: usize) -> Self` | Base gets a trailing `/`; `max_history` clamped ≥ 1. `Default` = base `/`, 100 entries |
| navigate | `navigate_to(&mut self, uri: &str, replace: bool) -> Result<bool, NavigateError>` | `Ok(false)` ⇒ vetoed; same-URI ⇒ `Ok(true)` no-op |
| history | `back()`, `forward() -> Result<bool, NavigateError>` | `Ok(false)` at the edges or when vetoed |
| inspect | `current_uri() -> &str`, `current_relative() -> &str`, `can_go_back()`, `can_go_forward()`, `history_len()`, `history_index()`, `history() -> &[String]`, `base_uri()`, `max_history()`, `veto_count()` | `current_relative` is allocation-free |
| guards | `register_veto(impl FnMut(&mut LocationChangingContext) + 'static) -> VetoHandle`, `unregister_veto(VetoHandle)` | Handlers run in registration order |
| URI helpers | `to_absolute(&str) -> String`, `to_base_relative(&str) -> String`, `uri_with_query(&str, Option<&str>) -> String` | `to_base_relative` returns `""` for foreign URIs |

| Type | Variants / fields |
|------|-------------------|
| `NavigationKind` | `Push` (default), `Replace`, `Back`, `Forward`; `as_str()`, `Display` |
| `NavigateError` | `Empty`, `Malformed(String)`, `OutsideBase(String)`, `ReentrantCall`; implements `Error` + `Display` |
| `LocationChangingContext` | `target_location: String`, `kind: NavigationKind`, `prevent()`, `is_prevented()` |
| `VetoHandle` | Opaque token for `unregister_veto` |

### `Page`, `Layout`, `NavigationContext` (`page.rs`)

```rust
pub trait Page: Any {
    fn bind(&mut self, ctx: &NavigationContext, reset_missing: bool) {}
    fn on_navigated_to(&mut self, ctx: &NavigationContext) {}
    fn on_navigated_from(&mut self) {}
    fn show(&mut self, ui: &mut Ui, ctx: &NavigationContext, nav: &mut NavigationManager);
}

pub trait Layout {
    fn on_navigated_to(&mut self, ctx: &NavigationContext) {}
    fn on_navigated_from(&mut self) {}
    fn show(&mut self, ui: &mut Ui, ctx: &NavigationContext, body: &mut dyn FnMut(&mut Ui));
}

pub struct NavigationContext {
    pub location: String,                        // absolute URI
    pub path: String,                            // "/orders/42" (base-relative, no query)
    pub route_values: HashMap<String, RouteValue>, // lower-cased keys
    pub query: HashMap<String, String>,            // lower-cased keys, first value wins
}
// route(&str) -> Option<&RouteValue>, route_str / route_i64 / route_f64 / route_bool,
// query(&str) -> Option<&str>, has_query(&str) -> bool, query_parse::<T: FromStr>(&str) -> Option<T>
```

### `RouteTable`, `RouteTableBuilder` (`route_table.rs`)

| Item | Signature |
|------|-----------|
| builder | `RouteTable::builder() -> RouteTableBuilder` |
| register | `add<T: Page + Default>(&mut self, template: &str) -> Result<&mut Self, String>` |
| register (DI) | `add_with<T: Page>(&mut self, template: &str, factory: impl Fn() -> T + 'static) -> Result<&mut Self, String>` |
| freeze | `build(self) -> RouteTable` (stable specificity sort) |
| query | `lookup(&self, path) -> Option<(&RouteEntry, HashMap<String, RouteValue>)>`, `try_match(&self, path) -> Option<RouteMatch>`, `len()`, `is_empty()`, `iter()` |
| alias | `type PageFactory = Rc<dyn Fn() -> Box<dyn Page>>` |

### `RouteTemplate` (`route_template.rs`)

`parse(&str) -> Result<Self, String>` (also `FromStr`), `try_match(&str) -> Option<HashMap<String, RouteValue>>`, `is_match(&str) -> bool` (allocation-free), `segment_count()`, `parameter_names()`, public fields `template` (canonical form) and `signature` (ambiguity fingerprint).

### `RouteValue` (`route_value.rs`)

`Str(String) | Int(i64) | Float(f64) | Bool(bool)` with `as_str / as_i64 / as_f64 / as_bool` (lenient: `Str` parses, `Int` widens to `f64`), `to_text() -> Cow<str>`, `kind()`, `Display`, and `From` impls for `String`, `&str`, `i32`, `i64`, `f64`, `bool`.

### `query` helpers (`query.rs`)

| Function | Purpose |
|----------|---------|
| `pairs(&str) -> QueryPairs` | Lazy `(Cow<str>, Cow<str>)` iterator |
| `get_first(&str, name) -> Option<Cow<str>>` | Case-insensitive lookup |
| `parse(&str) -> Vec<(String, String)>` | Owned parse (compat) |
| `first_value_map(&str)` / `to_first_value_map(Vec<_>)` | Lower-cased first-value `HashMap` |
| `set_parameter(&mut Vec<_>, name, Option<&str>)` | In-place set/replace/remove |
| `with_parameter(&str, name, Option<&str>) -> String` | Single-pass, single-allocation equivalent |
| `build(&[(K, V)]) -> String` | Encode to `?a=1&b=2` |
| `split_uri(&str) -> (path, query, fragment)` | Slice-based split |
| `percent_decode(&str, plus_as_space) -> Cow<str>` / `encode_component_into` | Encoding primitives |

### `Router` (`router.rs`)

| Item | Signature |
|------|-----------|
| construct | `Router::new(RouteTable)`, `Router::default()` |
| configure | `with_layout(Box<dyn Layout>)`, `reuse_pages(bool)`, `not_found_with::<T>(impl Fn() -> T)`, `not_found_default::<T: Default>()` |
| frame | `render(&mut self, ui: &mut Ui, nav: &mut NavigationManager)` |
| inspect | `current_context() -> Option<&NavigationContext>`, `current_route() -> Option<&RouteTemplate>`, `current_page_type() -> Option<TypeId>` |
| teardown | `shutdown(&mut self)` |

---

## Page lifecycle

On every URI change the router runs, in this order:

| # | Hook | Target | Condition |
|---|------|--------|-----------|
| 1 | `on_navigated_from()` | Layout, then Page | Only if a page is active (skipped on the very first dispatch) |
| 2 | *(construct new page via factory)* | — | Only if not reusing |
| 3 | `bind(ctx, reset_missing)` | Page | `reset_missing = true` when the instance is reused |
| 4 | `on_navigated_to(ctx)` | Page | Always |
| 5 | `on_navigated_to(ctx)` | Layout | If a layout is configured |
| 6 | `show(...)` | Layout → Page | Every frame |

`Router::shutdown()` fires `on_navigated_from` on the layout and the page and drops them.

---

## Performance & design rationale

### Allocation strategy

- **Steady-state frames allocate nothing in the router.** The router compares `nav.current_uri()` against its stored `last_uri` as `&str`s (the buffer is reused when it changes) and passes the already-built `NavigationContext` **by reference** to the layout and page. The original per-frame `String` and `HashMap` clones are gone.
- **Matching is two-pass and allocation-free for misses.** `RouteTemplate::try_match` first validates the path with a lazy slice iterator (`PathSegments`) and `&str`/`Cow` comparisons; only when the template is known to match does it walk again to materialise captured values. Scanning N templates costs zero heap allocations for the N−1 that fail.
- **Borrowed decoding.** `percent_decode` and the query iterator return `Cow::Borrowed` unless an escape (or `+` in queries) is actually present. Catch-all tails borrow straight from the path when no normalisation is needed.
- **Single-pass query rewriting.** `with_parameter` streams the existing query into one pre-sized `String`, so `uri_with_query` does no intermediate `Vec<(String, String)>`.
- **Lazy URI resolution.** `resolve_within_base` returns a `Cow`, so the no-op "navigate to where I already am" and in-base absolute URIs do not allocate for the comparison.
- **Guards without cloning history.** `back()` / `forward()` hand the history entry to handlers by reference; a `LocationChangingContext` is only built when at least one handler exists.
- **No factory call on reuse.** The original called the page factory on every navigation and discarded the instance when reusing; the factory now runs only when a page is actually needed.
- **Case-insensitive lookups don't allocate.** `NavigationContext::route`/`query` lower-case their key through a `Cow` (borrowed when already lower-case), and name comparison uses an iterator-based Unicode-aware `eq_ignore_case`.

### Memory footprint

- A `RouteTemplate` stores its segments once; matching never copies them. `RouteEntry` / `RouteTable` are `Clone` thanks to `Rc` factories (a pointer bump, not a deep copy).
- `NavigationContext` is created once per navigation (not per frame) and holds only the data the page needs.
- History is a bounded `Vec<String>`; trimming uses a single `drain(..excess)`.

### Lifetime & ownership decisions

- **Polling instead of events.** Immediate-mode UIs redraw every frame, so a pull model (`current_uri()` comparison) avoids `Rc<RefCell<…>>` webs and re-entrancy hazards that event subscriptions would introduce.
- **Handlers can't re-enter.** Veto handlers run while the manager is mutably borrowed, so Rust's borrow checker already forbids re-entrant navigation. The `ReentrantCall` error and an RAII guard that clears the "running" flag remain as defence-in-depth (including if a handler panics).
- **`Rc` factories, `'static` handlers.** Pages and handlers are `'static` (pages are identified by `TypeId`), and the whole routing stack is single-threaded like `egui`'s UI thread — hence `Rc` rather than `Arc`, and no `Send`/`Sync` bounds.
- **Owned `RouteValue`s.** Captured values are owned so pages can keep them past the frame without lifetime plumbing; they are created once per navigation.
- **Explicit API surface.** `Cow`/slice-returning functions are *additions* (`pairs`, `with_parameter`, `current_relative`, `is_match`, …). The original owned-return functions remain.

---

## Behavioural notes & fixes relative to the original port

The refactor keeps every public type, trait and method signature (see the one exception below). The following behaviours were deliberately corrected; each is covered by a unit test.

1. **Route-parameter keys are lower-cased.** Previously `{OrderId}` was stored as `"OrderId"` while `NavigationContext::route` looked up `"orderid"`, so the value could never be found.
2. **Path decoding.** Path segments were decoded via the *query* decoder: `+` became a space and a segment containing `&` was truncated. Paths now use proper percent-decoding (`+` literal; `&`/`=` preserved).
3. **Relative URIs containing `://` in the query** (e.g. `login?next=http://x/`) were treated as absolute and rejected with `OutsideBase`. A scheme is now only recognised at the start of the URI.
4. **Reused built-in not-found page** kept showing the first missing path; its `bind` now refreshes it.
5. **`on_navigated_from` is no longer fired before the first `on_navigated_to`** (nothing had been navigated to yet).
6. **`current_route`** is updated when a page is reused (it previously kept the old template).
7. **Page factories no longer run when a page is reused.**
8. **Handlers are panic-safe.** The original moved the handler list out and would lose it (and leave the re-entrancy flag set) if a handler panicked.
9. `RouteTable::try_match` used to return a `RouteMatch` whose `factory` panicked via `unreachable!`; it now returns a working factory.

**Type-level change:** `RouteEntry::factory` / `RouteMatch::factory` are now `Rc<dyn Fn() -> Box<dyn Page>>` (alias `PageFactory`) instead of `Box<dyn Fn() -> Box<dyn Page>>`. Call syntax `(entry.factory)()` is unchanged; only code that *constructs* these structs by hand is affected.

---

## Testing

Every module has a `#[cfg(test)] mod tests` block (≈140 tests) covering:

- history, replace, bounds and forward-truncation, URI resolution and edge cases (empty, whitespace, foreign hosts, `://` in queries);
- veto handlers (push/replace/back/forward, ordering, short-circuit, unregister, target/kind reporting, panic safety) and the re-entrancy guard;
- query parsing/building/round-trips, `set_parameter` vs `with_parameter` equivalence, borrowed-vs-owned `Cow` behaviour;
- template parsing errors, all constraints (`int`, `long`, `double`, `float`, `decimal`, `bool`, `guid`, `datetime`), optional and catch-all segments, specificity ordering, ambiguity detection;
- router lifecycle ordering (`bind` → `on_navigated_to`, layout hooks, reuse vs. new page, not-found handling, shutdown).

```bash
cargo test
```

The router tests drive `Router::sync` directly, so they need no GPU, window or `egui::Context`.

> **Heads-up:** this code was written without access to a Rust toolchain in the authoring environment, so it has not been compiled or run there. Please run `cargo test` and `cargo clippy` once on your side; if anything fails to compile (most likely a small signature mismatch against your exact `egui` version) it should be a one-line fix.

---

## Minimum supported Rust version

**Rust 1.86** — `impl dyn Page { fn type_id }` upcasts `&dyn Page` to `&dyn Any`, which requires stable trait-object upcasting.

---

## License

Choose a license for your project (for example MIT OR Apache-2.0) and add the corresponding `LICENSE-*` files.

===== ./crates/mud_egui_tailwind/src/README.md =====

# Tailwind Utilities for Mud.Egui

Tailwind-style utility classes for egui widgets. Supports arbitrary values, opacity modifiers and interactive variants.

## Contents

- [Getting Started](#getting-started)
- [Utilities](#utilities)
- [Spacing Scale](#spacing-scale)
- [Specificity Rules](#specificity-rules)
- [Troubleshooting](#troubleshooting)

## Getting Started

Tailwind has variants such as `hover:` and `dark:`, so you **must** pass the current interaction and theme state into the resolver.

```rust
use mud_egui_components::foundation::base::component_base::ComponentBase;
use mud_egui_tailwind::tw_style::TwStyle;

pub struct MyCard {
    base: ComponentBase,
    style: TwStyle,
    hovered: bool, // result from the previous frame
}

impl MyCard {
    pub fn new() -> Self {
        Self {
            base: ComponentBase::new().classes([
                "bg-primary/90",
                "hover:bg-primary-dark",
                "p-4",
                "rounded-lg",
            ]),
            style: TwStyle::default(),
            hovered: false,
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        let is_dark = ui.visuals().dark_mode;

        // Resolve the classes into an egui::Frame (16.0 is the rem size).
        let frame = self
            .style
            .build_frame(&self.base, ui, self.hovered, is_dark, 16.0);

        let out = frame.show(ui, |ui| {
            ui.label("Hello Tailwind");
        });

        // Remember hover state for the next frame.
        self.hovered = out.response.hovered();
    }
}
```

**Why remember hover from the previous frame?** The frame's real rect is only known after it is drawn, and the style is needed before drawing. Reading the last frame's `response.hovered()` is accurate and avoids testing the whole available area (`ui.max_rect()`), which would report a hover for anything inside the parent.

## Utilities

| Utility | Controls | Example classes |
| --- | --- | --- |
| [TwBackground](#twbackground) | `Frame.fill` | `bg-primary/50`, `bg-[#f00]`, `hover:bg-primary` |
| [TwBorder](#twborder) | `Frame.stroke` | `border border-dashed`, `border-[3px]` |
| [TwMargin / TwPadding](#twmargin--twpadding) | outer / inner margin | `m-4`, `-mt-2`, `p-[1.5rem]` |
| [TwRounded](#twrounded) | `Frame.corner_radius` | `rounded-lg`, `rounded-tl`, `rounded-[8px]` |

### TwBackground

- **Palette:** `bg-primary`, `bg-appbar-background`
- **Aliases:** `bg-primary-foreground` (maps to `MudPalettePrimaryText`), `bg-primary-dark`, `bg-primary-light`
- **Arbitrary hex:** `bg-[#f00]`, `bg-[#ff000080]` (CSS RGBA order)
- **Opacity:** `bg-primary/50` (50%), `bg-white/[0.2]`
- **Variants (stackable):** `hover:bg-primary`, `dark:bg-surface`, `focus:bg-active`

### TwBorder

Width and style are separate classes, as in Tailwind.

- **Styles:** `border-solid`, `border-dashed`, `border-dotted`, `border-double`, `border-none`
- **Widths:** `border` (1px), `border-2` (2px), `border-[3px]`, `border-[0.125rem]`
- **Example:** `border border-dashed`

### TwMargin & TwPadding

- **Spacing:** `m-4` (1rem), `p-2.5`, `mt-px` (1 pixel)
- **Negative margins:** `-mt-2`, `-mx-4`
- **Arbitrary values:** `m-[12px]`, `p-[1.5rem]`, `mt-[12]` (unitless means px)
- **Auto margins:** `mx-auto`, `ml-auto`, `mr-auto` (returns an `egui::Align`)

### TwRounded

- **Sizes:** `rounded-none`, `rounded-sm`, `rounded-md`, `rounded-lg`, `rounded-xl`, `rounded-2xl`, `rounded-full`
- **Corners:** `rounded-t` (top), `rounded-tl` (top-left), `rounded-ss` (start-start), and so on
- **Arbitrary:** `rounded-[8px]`

## Spacing Scale

A number `n` equals `n * 0.25rem`.

| Class | Value |
| --- | --- |
| `p-px` | 1px |
| `p-1` | 0.25rem |
| `p-4` | 1rem |
| `p-8` | 2rem |

## Specificity Rules

When several classes touch the same property, the more specific one wins, regardless of order.

| Utility | Order (lowest → highest) |
| --- | --- |
| Margin / padding | `m` < `mx`/`my` < `ms`/`me` < `mt`/`mr`/`mb`/`ml` |
| Rounded | `rounded` < `s` < `e` < `t` < `r` < `b` < `l` < `ss` < `se` < `ee` < `es` < `tl` < `tr` < `br` < `bl` |

Rounded classes follow Tailwind CSS order and are resolved per corner.

## Troubleshooting

| Symptom | Likely cause / fix |
| --- | --- |
| **`hover:` classes never trigger** | Pass the hover flag to `build_frame`. Use the previous frame's `response.hovered()`, as in the example above. |
| **Opacity looks wrong** | Your theme lookup should return an unmultiplied `egui::Color32` so the parser can change the alpha channel. |
| **`mx-auto` doesn't center** | `auto` returns an `egui::Align`. Apply it to the UI layout yourself. |
| **Arbitrary value fails to parse** | Check for typos. `[12px]` is valid; `[12px` (missing bracket) is not. |
| **`dark:` classes ignored** | Pass `ui.visuals().dark_mode` (or your own theme flag) as `is_dark`. |

===== ./crates/mud_egui_theming/src/README.md =====

Here is the complete guide for the Theming system. It has been placed in a single code block so you can easily copy it into your project's documentation.

```markdown
# Theming for Mud.Egui

Welcome to the Theming system for Mud.Egui. This crate (`mud_egui_theming`) is the direct Rust/egui port of the C# `Mud.Avalonia.Theming` system.

## 🚨 The Paradigm Shift: Avalonia vs. egui

If you are coming from Avalonia, WPF, or web development, you need to unlearn the concept of a `ResourceDictionary` and `ThemeProvider` controls. 

**In Avalonia (Retained Mode):**
1. A `ThemeProvider` control listens to the OS theme.
2. It generates a massive dictionary of 500+ XAML resources.
3. It broadcasts an event (`GlobalThemeChanged`).
4. Controls with `DynamicResource` bindings update themselves.

**In egui (Immediate Mode):**
1. You hold a `MudTheme` struct in your app state.
2. Every single frame (60 FPS), you call `theme.sync(ctx)`.
3. This overwrites egui's internal `Visuals` (colors, rounding, strokes).
4. Widgets read colors directly from the theme struct.

Because of this, **we completely deleted** the following C# files:
*   `ThemeProvider.cs`
*   `ThemeBroadcaster.cs`
*   `MudThemeResourceFactory.cs`
*   `MudThemeResourceKeyTable.cs`
*   `ThemeInjectionMode.cs`
*   All `*ResourceDefinition.cs` files

They are simply not needed.

---

## Getting Started

To use the theme, initialize it in your application state and sync it every frame.

```rust
use mud_egui_theming::theme::MudTheme;

struct MyApp {
    theme: MudTheme,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            theme: MudTheme::default(), // Starts in Light mode
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 1. Sync the theme to egui's global styling every frame
        self.theme.sync(ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Hello Mud.Egui");
            
            if ui.button("Toggle Dark Mode").clicked() {
                self.theme.is_dark_mode = !self.theme.is_dark_mode;
            }
        });
    }
}
```

---

## Core Concepts

### 1. `MudTheme` (The Aggregator)
The `MudTheme` struct holds everything: your light and dark palettes, typography, layout dimensions, shadows, and breakpoints.

It also holds the global `is_dark_mode` boolean. Setting this to `true` will instantly swap all colors to the dark palette on the next `sync()` call.

### 2. `Palette` (Semantic Colors)
The `Palette` struct holds semantic colors instead of raw hex codes. You don't ask for "Blue", you ask for `theme.current_palette().primary`.

It comes with two default palettes:
*   `Palette::new_light()`: Ivory backgrounds, ink text, deep clay primary.
*   `Palette::new_dark()`: Charcoal backgrounds, ivory text, warm clay primary.

**Computed Variants:**
Just like C#, colors have computed variants:
*   `palette.primary_darken()`
*   `palette.primary_lighten()`
*   `palette.primary.with_alpha(0.5)`

### 3. `MudColor` (Color Math)
Because egui uses `Color32` (which lacks HSL math and hex parsing), we provide `MudColor`.
*   Parses Hex: `MudColor::parse("#B85533")`
*   Converts directly to egui: `let color: Color32 = mud_color.into();`

### 4. `Typography`
We ported the 14-tier Material Design type scale. It holds `FontVariant` structs (H1-H6, Body1/2, Subtitle1/2, etc.).

### 5. `LayoutProperties`
Holds the standard MudBlazor dimensions:
*   `appbar_height` (64.0)
*   `drawer_width_left` (240.0)
*   `drawer_mini_width_left` (56.0)
*   `default_border_radius` (4.0)

### 6. `MudShadows`
Provides 25 levels of elevation (`0` through `24`), mapped to `egui::epaint::Shadow`. Light mode uses black alpha shadows; Dark mode uses ivory alpha shadows to ensure visibility on dark surfaces.

### 7. `Breakpoints`
Used for responsive design. Instead of checking raw pixel widths, you check the bucket.
*   `Xs`, `Sm`, `Md`, `Lg`, `Xl`, `Xxl`.

---

## Using the Theme in Widgets

Because egui is immediate mode, widgets can easily read the global theme from the `Context`.

### Accessing the Theme
We provide a `fetch` method that safely retrieves the shared theme from `egui`'s memory.

```rust
use mud_egui_theming::theme::MudTheme;

fn my_custom_widget(ui: &mut egui::Ui) {
    // 1. Fetch the theme
    let theme = MudTheme::fetch(ui.ctx()).unwrap_or_default();
    let palette = theme.current_palette();

    // 2. Use the colors
    let text_color = palette.primary_contrast_text.into();
    let bg_color = palette.primary.into();

    ui.colored_label(text_color, "Hello, Colored Text!");
}
```

### Resolving Utility Keys
If you are using the `bs_*` or `tw_*` utility parsers (like `bg-primary` or `border-dashed`), they output string keys like `"MudPalettePrimary"`.

Use `theme.resolve(key)` to turn that string into a real `Color32`.

```rust
// Inside your TwStyle / BsStyle build_frame method:
let theme = MudTheme::fetch(ui.ctx()).unwrap_or_default();

let color = match bg_class {
    TwBg::Literal(c) => c,
    TwBg::Resource { key, alpha } => {
        // Resolves "MudPalettePrimary" to a Color32
        let base_color = theme.resolve(&key).unwrap_or(egui::Color32::TRANSPARENT);
        // apply opacity...
        base_color
    }
};
```

### Checking Breakpoints
To build responsive layouts, check the current screen width against the theme's breakpoints.

```rust
fn responsive_widget(ui: &mut egui::Ui) {
    let theme = MudTheme::fetch(ui.ctx()).unwrap_or_default();
    let screen_width = ui.ctx().screen_rect().width();
    
    let bucket = theme.breakpoints.get_bucket(screen_width);

    match bucket {
        BreakpointBucket::Xs | BreakpointBucket::Sm => {
            ui.label("Mobile View");
        }
        _ => {
            ui.label("Desktop View");
        }
    }
}
```

### Applying Shadows
Use the active shadow set to give elevation to panels or popups.

```rust
let theme = MudTheme::fetch(ui.ctx()).unwrap_or_default();
let shadow = theme.current_shadows().get(4); // Get elevation level 4

// Apply to an egui Frame
egui::Frame::window(ui.style())
    .shadow(shadow)
    .show(ui, |ui| {
        ui.label("Elevated Card");
    });
```

---

## Troubleshooting

| Symptom | Likely Cause / Fix |
|---|---|
| **Theme colors don't apply** | Did you call `self.theme.sync(ctx)` at the top of your `update()` method? |
| **Dark mode doesn't work** | Ensure `theme.is_dark_mode = true` is set, and that you called `sync(ctx)` afterwards. |
| **Colors are transparent** | Ensure `MudColor` parsing didn't fail. Hex codes should include `#` or be valid lengths (3, 4, 6, 8). |
| **Text is invisible on buttons** | You are probably using `palette.primary` for text when you should be using `palette.primary_contrast_text`. |
| **Compile error on `style`** | In newer egui versions, `ctx.style()` is gone. Use `ctx.all_styles_mut(|style| { ... })` inside `sync()`. |
```
