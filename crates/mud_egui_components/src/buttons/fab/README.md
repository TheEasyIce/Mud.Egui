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