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