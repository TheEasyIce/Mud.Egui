# MudRating for `mud_egui`

A star-style rating control for [egui](https://github.com/emilk/egui), ported
from MudBlazor’s [`MudRating`](https://mudblazor.com/components/rating)
(MudBlazor 8.13.0).

```rust
let mut score = 3;

ui.add(
    MudRating::new(&mut score)
        .color(Color::Warning)
        .max_value(5),
);

if score != 0 {
    // score is the caller-owned selected value.
}
```

Every glyph renders through **`MudIcon`**. `MudRating` paints no text of its
own — it is a row of star icons.

---

## Contents

1. [Design principle: API parity, not API invention](#design-principle-api-parity-not-api-invention)
2. [Architecture](#architecture)
3. [API reference](#api-reference)
4. [Examples](#examples)
5. [Binding and behaviour](#binding-and-behaviour)
6. [Performance and memory choices](#performance-and-memory-choices)
7. [Testing](#testing)
8. [Differences from MudBlazor](#differences-from-mudblazor)

---

## Design principle: API parity, not API invention

The MudBlazor 8.13.0 parameter surface was verified against the published API
reference before implementation. `MudRating` has **fifteen parameters**, and all
fifteen are accounted for.

### Ported parameters

| MudBlazor | Type | Default | Rust mapping |
|---|---|---|---|
| `Color` | `Color` | `Default` | `color(Color)` |
| `Disabled` | `bool` | `false` | `disabled(bool)` |
| `EmptyIcon` | `string` | `StarBorder` | `empty_icon(s)` |
| `EmptyIconColor` | `Color?` | `null` | `empty_icon_color(Color)` |
| `FullIcon` | `string` | `Star` | `full_icon(s)` |
| `FullIconColor` | `Color?` | `null` | `full_icon_color(Color)` |
| `HoveredValueChanged` | `EventCallback<int?>` | — | `RatingResponse::hovered_changed` + `hovered` |
| `MaxValue` | `int` | **`5`** | `max_value(i32)` |
| `Name` | `string` | random GUID | `name(s)` — stored, inert |
| `ReadOnly` | `bool` | `false` | `read_only(bool)` |
| `Ripple` | `bool` | `true` | `ripple(bool)` |
| `SelectedValue` | `int` | **`0`** | caller-owned `&mut i32` |
| `SelectedValueChanged` | `EventCallback<int>` | — | `RatingResponse::changed` |
| `Size` | `Size` | `Medium` | `size(Size)` |
| `Class` | `string?` | `null` | `classes(iter)` — stored, not resolved |
| `Tag` | `object?` | `null` | `tag(t)` / `tag_ref::<U>()` |

### Deliberately not ported

| MudBlazor | Reason |
|---|---|
| `RatingItemsClass` | egui has no CSS strings. |
| `RatingItemsStyle` | egui has no CSS strings. |
| `Style` | egui has no inline CSS strings. |
| `UserAttributes` | There is no HTML tag to splat attributes onto. |
| `Class` resolution | Stored on `base` for parity only. The star colours come from `Color` / `FullIconColor` / `EmptyIconColor`; a utility-class frame would composite a second fill. |

`Name` is **stored but inert**: it becomes the HTML `<input name="…">` in
Blazor, which groups radios for form submission. egui has no form submission, so
it has no effect; it is accepted so a Blazor call site ports without edits.

Nothing else was added. There is no `Variant`, no `Elevation`, no `ReadOnly`
toggle beyond the ported `read_only`, no `AdornmentStart`/`AdornmentEnd`, and
**no `HalfIncrements` or half-star mode** — MudBlazor’s `MudRating` is
integer-only and so is this port.

### There is no `HoveredValue` parameter

MudBlazor reports the hovered star through `HoveredValueChanged` and exposes no
`HoveredValue` parameter for reading. The Rust port reports both:

- `RatingResponse::hovered: i32` — the current hovered star every frame (`0`
  when nothing is hovered), so a caller can read it directly.
- `RatingResponse::hovered_changed: Option<i32>` — `Some(new)` on the frame the
  hover moved, with `0` meaning "hover left". That maps `null` (Blazor’s
  un-hovered sentinel) to `0` and gives the same edge-triggered semantics as
  `HoveredValueChanged`.

This is an extension of *reporting*, not of the component’s parameter surface.

---

## Architecture

```
inputs/rating/
└── mud_rating.rs
    ├── constants        STAR_GAP 4, RIPPLE_DURATION 0.35, RIPPLE_ALPHA 60,
    │                    EMPTY_ALPHA 0.38, DISABLED_ALPHA 0.38
    ├── pure helpers     star_pixels, row_size, star_rect, star_at,
    │                    display_value, clamp_selected, sanitize_star_count,
    │                    apply_keys (KeyAction), with_alpha
    ├── MudRating<'a>    builder + show(ui) -> RatingResponse
    ├── RatingResponse   response + changed + hovered + hovered_changed
    └── ripple           start_ripple, paint_ripple (memory-backed)
```

**Pure layer.** The star pixel table, the row size, the per-star rect, the
hit-test mapping, the display-value rule, the selection clamp, the keyboard
mapping and the star-count sanitizer are all free functions over `f32`, `i32`
and `Rect`. None touch `egui::Context`.

**Binding layer.** The caller owns the `&mut i32`; the component writes
`Some(star)` when a click or key selects one. Two-way binding becomes a
caller-owned field plus a `changed` report.

**Component layer.** `show` allocates one row rect, uses `response.hover_pos()`
and `response.interact_pointer_pos()` to determine the hovered / clicked star,
applies the keyboard action, and paints one `MudIcon` per star.

---

## API reference

### `MudRating<'a>`

| Method | Type | Default | Mirrors | Description |
|---|---|---|---|---|
| `new(selected)` | `&'a mut i32` | — | `SelectedValue` | Binds the rating to a caller-owned `i32`. |
| `max_value(n)` | `i32` | `5` | `MaxValue` | Number of stars; `<= 0` renders nothing. |
| `color(c)` | `Color` | `Default` | `Color` | Accent of filled stars. |
| `size(s)` | `Size` | `Medium` | `Size` | Star size: 8 / 16 / 24 / 32 / 48 pt. |
| `full_icon(s)` | `impl Into<String>` | `STAR` | `FullIcon` | Filled glyph. |
| `empty_icon(s)` | `impl Into<String>` | `STAR_BORDER` | `EmptyIcon` | Empty glyph. |
| `full_icon_color(c)` | `Color` | `None` | `FullIconColor` | Filled colour override. |
| `empty_icon_color(c)` | `Color` | `None` | `EmptyIconColor` | Empty colour override. |
| `ripple(b)` | `bool` | `true` | `Ripple` | Click ripple. |
| `disabled(b)` | `bool` | `false` | `Disabled` | Dims and disables. |
| `read_only(b)` | `bool` | `false` | `ReadOnly` | Disables without dimming. |
| `name(s)` | `impl Into<String>` | `None` | `Name` | Stored, inert. |
| `classes(iter)` | `iter of AsRef<str>` | none | `Class` | Stored on `ComponentBase`. |
| `tag(t)` / `tag_ref::<U>()` | `T: Any + Send + Sync` | `None` | `Tag` | Arbitrary user data. |
| `show(ui)` | `&mut Ui` | — | — | Returns `RatingResponse`. |

### `RatingResponse`

| Field | Type | Description |
|---|---|---|
| `response` | `Response` | The rating row’s `Response`. Deref target. |
| `changed` | `bool` | `true` on the frame `SelectedValue` moved. Mirrors `SelectedValueChanged`. |
| `hovered` | `i32` | Current hovered star (`1..=max_value`), or `0` when nothing is hovered. Reported every frame. |
| `hovered_changed` | `Option<i32>` | `Some(new)` only on the frame the hover moved, with `0` meaning "hover left". Mirrors `HoveredValueChanged`. |

### `Size` mapping

| `Size` | Star side |
|---|---|
| `ExtraSmall` | 8 pt |
| `Small` | 16 pt |
| `Medium` | 24 pt |
| `Large` | 32 pt |
| `ExtraLarge` | 48 pt |

The table matches `MudIcon`’s, so a `MudRating::size(Size::Large)` is the same
pixel size as a `MudIcon::size_preset(Size::Large)` elsewhere in the same
screen.

### Constants

| Name | Value |
|---|---|
| `STAR_GAP` | `4.0` |
| `RIPPLE_DURATION` | `0.35` s |
| `RIPPLE_MAX_RADIUS_FACTOR` | `1.2` |
| `RIPPLE_ALPHA` | `60` |
| `EMPTY_ALPHA` | `0.38` |
| `DISABLED_ALPHA` | `0.38` |

---

## Examples

### Basic rating

```rust
let mut score = 3;
ui.add(MudRating::new(&mut score));
```

### Warning-coloured stars

```rust
MudRating::new(&mut score).color(Color::Warning).show(ui);
```

### Custom star count

```rust
MudRating::new(&mut score).max_value(10).show(ui);
```

### Read-only display

```rust
MudRating::new(&mut avg).read_only(true).show(ui);
```

The stars are full opacity, but nothing changes on click or hover.

### Disabled

```rust
MudRating::new(&mut avg).disabled(true).show(ui);
```

Dimmed to 38 % opacity, ignores pointer and keyboard.

### Custom icons (hearts)

```rust
MudRating::new(&mut score)
    .full_icon(filled::FAVORITE)
    .empty_icon(filled::FAVORITE_BORDER)
    .color(Color::Error)
    .show(ui);
```

### Custom colours

```rust
MudRating::new(&mut score)
    .color(Color::Warning)
    .full_icon_color(Color::Success)
    .empty_icon_color(Color::Error)
    .show(ui);
```

`FullIconColor` wins over `Color` for filled stars; `EmptyIconColor` sets the
unfilled glyph. When neither override is set, filled uses `Color` and empty uses
`Color` at 38 % gamma.

### Reacting to a change

```rust
let out = MudRating::new(&mut score).show(ui);
if out.changed {
    log::info!("score = {score}");
}
```

### Reacting to hover

```rust
let out = MudRating::new(&mut score).show(ui);
if let Some(hovered) = out.hovered_changed {
    if hovered == 0 {
        log::debug!("pointer left the rating");
    } else {
        log::debug!("hovering star {hovered}");
    }
}
```

### Keyboard-only

```rust
// With the rating focused:
//   → / ↑  increment
//   ← / ↓  decrement
//   Home   1
//   End    max_value
//   Space / Enter  select the hovered star, if any
```

### Standalone read-back

```rust
let score = 4;
let mut read_only = score;
let out = MudRating::new(&mut read_only).read_only(true).show(ui);
assert_eq!(out.changed, false);
```

### Tag round-trip

```rust
let rating = MudRating::new(&mut score).tag("product-rating");
assert_eq!(rating.tag_ref::<&str>(), Some(&"product-rating"));
```

---

## Binding and behaviour

### `SelectedValue` is `i32`, and `0` means unselected

MudBlazor’s `MudRating` uses a non-nullable `int` for the selected value, with
**`0` meaning “nothing selected”**. The port keeps that:

```rust
let mut score = 0;          // unselected
ui.add(MudRating::new(&mut score));
// After clicking star 3, `score == 3`.
```

There is no way to distinguish “selected 0” from “unselected” — there is no
“selected 0”. This is exactly MudBlazor’s behaviour.

### Hover preview

While the pointer is over star *K*:

- **Interactive** (`!read_only && !disabled`): stars `1..=K` render filled,
  stars `K+1..=max` render empty. On pointer leave, the display reverts to
  `*selected`.
- **ReadOnly or Disabled**: only the selected value is ever shown; the hover
  preview is suppressed entirely.

This is the intended 8.x behaviour. The 9.x changelog records the suppression as
a bug fix that brought read-only/disabled in line with this behaviour, and the
port applies the fix.

### Selection

| Interaction | Effect |
|---|---|
| Click star K | `*selected = K`; `changed = true` |
| `→` / `↑` (focused) | `*selected += 1`, clamped to `max_value` |
| `←` / `↓` (focused) | `*selected -= 1`, clamped to `0` |
| `Home` (focused) | `*selected = 1` |
| `End` (focused) | `*selected = max_value` |
| `Space` / `Enter` (focused) | Select the hovered star, if any |

A keyboard action only fires when the rating is focused and interactive. A
read-only or disabled rating still responds to *no* keyboard input, but a caller
can move the value through the `&mut i32` binding.

### `HoveredValueChanged` semantics

The Blazor callback fires with `int?`, where `null` means the pointer left the
rating. The Rust response maps that to `Option<i32>` with `0` standing in for
`None`:

```rust
if let Some(v) = out.hovered_changed {
    match v {
        0 => log::debug!("pointer left"),
        n => log::debug!("hovering {n}"),
    }
}
```

`hovered_changed` is `Some` only on the frame the hover moved. It is never
`Some` on the first frame the rating is drawn, matching the
`ExpandedChanged`-style contract used by `MudCollapse` and `MudExpansionPanel`.

---

## Performance and memory choices

| Choice | Effect |
|---|---|
| Theme fetched once per frame | One `MudTheme::fetch` and up to three `Color::resolve` calls (accent, full, empty). |
| One `allocate_exact_size` for the row | The whole rating is one rect and one `Response`. |
| One `ui.put` per star | `MudIcon` is the only per-star allocation, and it is the icon cache’s, not the component’s. |
| Pointer read from the row’s `Response` | `hover_pos()` and `interact_pointer_pos()` are already computed by egui; no extra hit-testing. |
| No `Vec`, no `String` | The caller’s `&mut i32` is the only binding; the response is three `Copy` fields plus the `Response`. |
| Hovered change detection in `egui::Memory` | One `i32` per rating, keyed by the row’s id; the component itself holds no state. |
| Ripple in `egui::Memory` | One `f64` per in-flight star; zero cost when `ripple(false)`. |
| Pure helpers on `Copy` values | Star size, row size, hit-test and keyboard mapping are arithmetic; unit-testable without a `Context`. |
| `read_only` / `disabled` skip pointer and keyboard | No hover, no click, no `Sense::click()` registration. |

---

## Testing

`cargo test -p mud_egui_components mud_rating` runs 19 pure-function tests:

| Area | Tests |
|---|---|
| Star pixel table | every `Size`; matches `MudIcon`’s scale |
| Row size | zero stars, one star, five stars |
| Star count sanitizer | zero, negative, positive |
| Hit-test | every slot of a five-star row; outside the row; in the gap between stars; zero stride |
| Star rect | first two stars; left-to-right layout |
| Display value | hover when interactive; suppressed when read-only or disabled; nothing selected |
| Selection clamp | negative → `0`; above max → max; `0` stays `0`; zero max |
| Keyboard | no key pressed → no action |
| Builders | defaults match MudBlazor (max **5**, `SelectedValue` **0**); every field set; tag round-trip |

No test creates a live `egui::Context`; the one keyboard test uses
`Context::run_ui` with an empty `RawInput`, which is safe and allocation-free.
`show` needs a live `Ui` and is therefore left to integration tests.

Suggested integration tests (`egui_kittest` or a headless `Context::run`
harness):

- Clicking star 3 sets `*selected == 3` and `changed == true`.
- Hovering star 4 previews four filled stars; leaving restores the selected
  count.
- `read_only(true)` blocks clicks and suppresses the hover preview.
- `disabled(true)` blocks clicks, suppresses the hover preview, and dims the
  stars.
- `→` on a focused rating increments; `←` decrements; `Home` sets `1`; `End`
  sets `max_value`.
- `hovered_changed` is `Some(new)` on the frame the hover moves and `None`
  otherwise.

```bash
cargo test -p mud_egui_components mud_rating
```

---

## Differences from MudBlazor

- **`SelectedValue` / `SelectedValueChanged` become a caller-owned `&mut i32`
  plus `RatingResponse::changed`.** The caller owns the value; `changed` is
  `true` on the frame it moved. This is the same split `MudCheckBox`,
  `MudRadio` and `MudCollapse` use. The `0`-means-unselected convention is
  preserved.
- **`HoveredValueChanged` becomes two fields on `RatingResponse`.**
  `hovered: i32` reports the current hover every frame; `hovered_changed:
  Option<i32>` is `Some(new)` on the frame the hover moved, with `0` mapped to
  Blazor’s `null`. The reporting is extended; the parameter surface is
  unchanged.
- **`Name` is stored but inert.** It becomes the HTML `<input name="…">` in
  Blazor, which egui has no equivalent for. Accepted so a Blazor call site
  ports without edits, documented here rather than silently dropped.
- **`RatingItemsClass` and `RatingItemsStyle` are not ported.** No CSS strings.
- **`Style` and `UserAttributes` are not ported.** No CSS strings, no HTML tag.
- **`ReadOnly` and `Disabled` suppress the hover preview.** The 9.x changelog
  records this as a bug fix on MudBlazor’s side (“Skip hover effects when
  ReadOnly or Disabled”); the port applies the intended 8.x behaviour.
- **`Size` uses `MudIcon`’s pixel table** (8 / 16 / 24 / 32 / 48), so a
  `MudRating` with `Size::Large` matches a `MudIcon` with `Size::Large` on the
  same screen. MudBlazor’s SCSS uses the same scale.
- **No half-star mode.** MudBlazor’s `MudRating` is integer-only; there is no
  `HalfIncrements` or `AllowHalf` parameter to port, and none was added.
- **Keyboard handling covers `Home` / `End`.** `HandleKeyDownAsync` in MudBlazor
  is arrow keys only; the port adds `Home` (set to `1`) and `End` (set to
  `max_value`), which are the standard keyboard interactions for a rating input.
  They do not change the ported parameter surface.
- **Default `Color` is `Default`, and `Color::Default` resolves to
  `ui.visuals().text_color()`.** MudBlazor’s `Color.Default` on a rating
  renders plain text-coloured stars; the port does the same, and setting
  `.color(Color::Warning)` or `.color(Color::Primary)` gives the familiar
  coloured stars.