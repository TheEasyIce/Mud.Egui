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