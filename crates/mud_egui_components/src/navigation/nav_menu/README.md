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