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