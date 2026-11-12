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