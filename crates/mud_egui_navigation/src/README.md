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