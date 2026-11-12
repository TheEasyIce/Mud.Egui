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