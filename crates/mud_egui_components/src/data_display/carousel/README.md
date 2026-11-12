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