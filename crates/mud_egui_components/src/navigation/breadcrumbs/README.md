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