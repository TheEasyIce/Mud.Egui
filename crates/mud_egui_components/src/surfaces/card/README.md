# MudCard family for `mud_egui`

A surface that groups a header, media, content and actions, plus its four
companion blocks. Ported from MudBlazor 8.13.0's
[`MudCard`](https://mudblazor.com/components/card),
`MudCardHeader`, `MudCardContent`, `MudCardActions` and `MudCardMedia`.

```rust
use mud_egui_components::surfaces::card::{
    MudCard, MudCardActions, MudCardContent, MudCardHeader, MudCardMedia,
};

MudCard::new().show(ui, |ui| {
    MudCardMedia::new()
        .image("https://picsum.photos/400/200")
        .height(160.0)
        .title("A photo")
        .show(ui);

    MudCardHeader::new().show(ui, |ui| {
        ui.add(MudText::h6("Card title"));
    });

    MudCardContent::new().show(ui, |ui| {
        ui.add(MudText::body2("Supporting copy goes here."));
    });

    MudCardActions::new().show(ui, |ui| {
        ui.add(MudButton::new("Dismiss").variant(Variant::Text));
        ui.add(MudButton::new("Open").variant(Variant::Filled));
    });
});
```

The card paints only the surface; every string inside is yours to render through
**`MudText`**, every glyph through **`MudIcon`**.

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

All five components were verified against DNDocs for MudBlazor 8.13.0 before
implementation. Together they expose **exactly eight parameters**.

### `MudCard`

| MudBlazor | Type | Default | Rust mapping |
|---|---|---|---|
| `ChildContent` | `RenderFragment?` | `null` | closure to `show(ui, \|ui\| ..)` |
| `Elevation` | `int` | **`1`** | `elevation(u8)` (clamped 0–24) |
| `Outlined` | `bool` | `false` | `outlined(bool)` |
| `Square` | `bool` | `false` | `square(bool)` |
| `Class` | `string?` | `null` | `classes(iter)` — stored, not resolved |
| `Tag` | `object?` | `null` | `tag(t)` / `tag_ref::<U>()` |

### `MudCardHeader` / `MudCardContent` / `MudCardActions`

| MudBlazor | Type | Default | Rust mapping |
|---|---|---|---|
| `ChildContent` | `RenderFragment?` | `null` | closure to `show(ui, \|ui\| ..)` |
| `Class` | `string?` | `null` | `classes(iter)` — stored |
| `Tag` | `object?` | `null` | `tag(t)` / `tag_ref::<U>()` |

### `MudCardMedia`

| MudBlazor | Type | Default | Rust mapping |
|---|---|---|---|
| `Image` | `string?` | `null` | `image(s)` |
| `Height` | `int` | **`300`** | `height(f32)` |
| `Title` | `string?` | `null` | `title(s)` |
| `Class` | `string?` | `null` | `classes(iter)` — stored |
| `Tag` | `object?` | `null` | `tag(t)` / `tag_ref::<U>()` |

### Deliberately not ported

| MudBlazor | Reason |
|---|---|
| `Style` (all five) | egui has no inline CSS strings. |
| `UserAttributes` (all five) | There is no HTML tag to splat attributes onto. |
| `Class` resolution (all five) | Stored on `base` for parity, but not resolved: `MudCard`'s fill, border and radius are owned by `Elevation` / `Outlined` / `Square`, and the companions own only their padding. Resolving a second utility frame would composite two fills or two margins. |

Nothing else was added. In particular there is no `Color`, no `Variant`, no
`Size`, no `Disabled` / `ReadOnly`, no `OnClick` and no adornment slots —
`MudCard` has none of them in MudBlazor, and the companions have only
`ChildContent`.

---

## Architecture

```
surfaces/card/
├── mud_card.rs           MudCard: surface + elevation/outline/radius
│   └── pure helpers      card_radius, card_stroke, card_shadow,
│                         pad_u8, pad_i8
├── mud_card_header.rs    MudCardHeader: 16 pt padded vertical block
├── mud_card_content.rs   MudCardContent: 16 pt padded vertical block
├── mud_card_actions.rs   MudCardActions: 8 pt padded horizontal row
├── mud_card_media.rs     MudCardMedia: MudImage with Cover fit + height
│   └── pure helpers      sane_height
└── mod.rs                re-exports
```

**Pure layer.** Every decision that can be made from numbers is a free
function: the corner radius, whether a stroke or a shadow is emitted and at what
strength, how a padding is clamped into `i8`, and how a bad media height is
repaired. All of them are unit-tested without an `egui::Context`.

**Component layer.** `MudCard::show` fetches the theme once, builds one
`egui::Frame`, and runs the caller's closure inside it. The companions build a
single `Frame` with a constant margin and run the closure inside that. None of
them holds state between frames.

**No `MudPaper` dependency.** `MudCard` could have delegated to `MudPaper`, but
`MudPaper::show` takes `&mut self` (it refreshes its Tailwind state after
drawing), which would force `MudCard` to own a long-lived `MudPaper` and break
the stateless-builder model every other component in this workspace follows.
`MudCard` therefore builds its own frame, exactly as `MudAlert` does.

---

## API reference

### `MudCard`

| Method | Type | Default | Mirrors | Description |
|---|---|---|---|---|
| `new()` | `Self` | — | — | Elevation 1, rounded, not outlined. |
| `elevation(e)` | `u8` | `1` | `Elevation` | Shadow depth, clamped to 24. `0` = flat. |
| `outlined(b)` | `bool` | `false` | `Outlined` | 1 pt border; suppresses the shadow. |
| `square(b)` | `bool` | `false` | `Square` | Zero corner radius. |
| `classes(iter)` | `iter of AsRef<str>` | none | `Class` | Stored on `ComponentBase`. |
| `tag(t)` / `tag_ref::<U>()` | `T: Any + Send + Sync` | `None` | `Tag` | Arbitrary user data. |
| `show(ui, \|ui\| ..)` | `&mut Ui, FnOnce(&mut Ui) -> R` | — | `ChildContent` | Returns `InnerResponse<R>`. |

### `MudCardHeader`

| Method | Default | Mirrors | Description |
|---|---|---|---|
| `new()` | — | — | 16 pt padding on all sides. |
| `classes(iter)` | none | `Class` | Stored. |
| `tag(t)` / `tag_ref::<U>()` | `None` | `Tag` | Arbitrary user data. |
| `show(ui, \|ui\| ..)` | — | `ChildContent` | Vertical stack inside the padding. |

### `MudCardContent`

Identical surface to `MudCardHeader` (16 pt padding, vertical stack). The two
are separate types so a call site reads like the Blazor markup and so their
padding can diverge if MudBlazor ever does.

### `MudCardActions`

| Method | Default | Mirrors | Description |
|---|---|---|---|
| `new()` | — | — | 8 pt padding, `left_to_right` row, vertically centred. |
| `classes(iter)` | none | `Class` | Stored. |
| `tag(t)` / `tag_ref::<U>()` | `None` | `Tag` | Arbitrary user data. |
| `show(ui, \|ui\| ..)` | — | `ChildContent` | Horizontal row inside the padding. |

### `MudCardMedia`

| Method | Type | Default | Mirrors | Description |
|---|---|---|---|---|
| `new()` | `Self` | — | — | No image, height 300. |
| `image(s)` | `impl Into<String>` | `None` | `Image` | URL or path. |
| `height(h)` | `f32` | `300.0` | `Height` | Strip height in points. |
| `title(s)` | `impl Into<String>` | `None` | `Title` | Forwarded to `MudImage::alt`. |
| `classes(iter)` | `iter of AsRef<str>` | none | `Class` | Stored. |
| `tag(t)` / `tag_ref::<U>()` | `T: Any + Send + Sync` | `None` | `Tag` | Arbitrary user data. |
| `show(ui)` | `&mut Ui` | — | — | Returns the image's `Response`. |

### Constants

| Name | Value | Where |
|---|---|---|
| `CARD_RADIUS` | `4.0` | `mud_card` (crate-private) |
| `OUTLINE_STROKE_WIDTH` | `1.0` | `mud_card` |
| `MAX_ELEVATION` | `24` | `mud_card` |
| `HEADER_PADDING` | `16.0` | `mud_card_header` |
| `CONTENT_PADDING` | `16.0` | `mud_card_content` |
| `ACTIONS_PADDING` | `8.0` | `mud_card_actions` |
| `DEFAULT_MEDIA_HEIGHT` | `300.0` | `mud_card_media` |

---

## Examples

### Minimal card

```rust
MudCard::new().show(ui, |ui| {
    MudCardContent::new().show(ui, |ui| {
        ui.add(MudText::body1("Just some text in a surface."));
    });
});
```

### Product card with media, title, copy and actions

```rust
MudCard::new().show(ui, |ui| {
    MudCardMedia::new()
        .image("file://assets/shoe.png")
        .height(180.0)
        .title("Trail runner")
        .show(ui);

    MudCardHeader::new().show(ui, |ui| {
        ui.add(MudText::h6("Trail runner"));
        ui.add(MudText::body2("Lightweight, 4 mm drop").color(Color::TextSecondary));
    });

    MudCardContent::new().show(ui, |ui| {
        ui.add(MudText::body2("Built for long days on loose ground."));
    });

    MudCardActions::new().show(ui, |ui| {
        ui.add(MudButton::new("Details").variant(Variant::Text));
        ui.add(MudButton::new("Add to cart").variant(Variant::Filled));
    });
});
```

### Outlined card, no shadow

```rust
MudCard::new()
    .outlined(true)
    .show(ui, |ui| {
        MudCardContent::new().show(ui, |ui| {
            ui.add(MudText::body2("Outlined cards read as selectable."));
        });
    });
```

`outlined(true)` suppresses the shadow even when `elevation` is left at its
default of 1 — that is MudBlazor's `MudPaper` rule.

### Flat card in a dense grid

```rust
MudCard::new().elevation(0).square(true).show(ui, |ui| {
    MudCardContent::new().show(ui, |ui| {
        ui.add(MudText::body2("Tiled, no gap seams."));
    });
});
```

### Elevated, floating card

```rust
MudCard::new().elevation(6).show(ui, |ui| {
    MudCardContent::new().show(ui, |ui| {
        ui.add(MudText::body1("Lifted off the page."));
    });
});
```

### Actions pushed to the trailing edge

```rust
MudCard::new().show(ui, |ui| {
    MudCardContent::new().show(ui, |ui| {
        ui.add(MudText::body2("Confirm?"));
    });
    MudCardActions::new().show(ui, |ui| {
        ui.add(MudButton::new("Cancel").variant(Variant::Text));
        ui.add(MudButton::new("Delete").color(Color::Error));
        ui.add(MudSpacer::new()); // absorbs the remaining width
    });
});
```

`MudCardActions` uses the same start-aligned row MudBlazor's stylesheet
produces (`display: flex; align-items: center`, no `justify-content`). A
`MudSpacer` is the idiomatic way to right-align the buttons.

### Header-only card (a panel)

```rust
MudCard::new().elevation(0).outlined(true).show(ui, |ui| {
    MudCardHeader::new().show(ui, |ui| {
        ui.add(MudText::subtitle2("Filters"));
    });
    MudCardContent::new().show(ui, |ui| {
        ui.checkbox(&mut self.show_archived, "Show archived");
    });
});
```

### Media strip without a URL

```rust
// Reserves 300 pt of blank space — the same box an <img> occupies before load.
MudCardMedia::new().show(ui);
```

### Tag round-trip

```rust
let card = MudCard::new().tag("product-42");
assert_eq!(card.tag_ref::<&str>(), Some(&"product-42"));
card.show(ui, |ui| { /* … */ });
```

---

## Layout and rendering

### Card surface

```
┌──────────────────────────────────────────────┐  ← Frame: fill = palette.surface
│  MudCardMedia   (height h, full width)        │     radius 4 pt (0 if Square)
├──────────────────────────────────────────────┤     shadow if elevation > 0
│  MudCardHeader  (padding 16)                  │     stroke 1 pt if Outlined
├──────────────────────────────────────────────┤
│  MudCardContent (padding 16)                  │
├──────────────────────────────────────────────┤
│  MudCardActions (padding 8, row, centred)     │
└──────────────────────────────────────────────┘
```

| Property | Rule |
|---|---|
| Fill | `palette.surface` from the active `MudTheme`. |
| Corner radius | 4 pt, or 0 with `square(true)`. |
| Outline | 1 pt in `ui.visuals().widgets.noninteractive.bg_stroke.color`; only when `outlined`. |
| Shadow | `card_shadow(elevation)`; `None` at 0 and when `outlined`. |
| Inner padding | **None.** The card is a pure surface; the companions own the spacing. |
| Width | Whatever the content asks for. Wrap the card in a sized parent (or set the child's width) to make it fill. |

### Why the card has no padding

In MudBlazor, `.mud-card` sets `overflow: hidden` and nothing else; the 16 pt
inset you see comes from `MudCardHeader` / `MudCardContent`, and the 8 pt inset
around buttons comes from `MudCardActions`. Splitting it the same way keeps a
media strip flush to the card edge, which is the whole point of
`MudCardMedia`.

### Clipping to rounded corners

egui's `Frame` does not clip its children, so a `MudCardMedia` inside a card
with a non-zero radius shows square corners where it meets the rounded edge.
MudBlazor avoids this with CSS `overflow: hidden`. The equivalent here is to
give the card an explicit width and clip the media yourself:

```rust
let card_w = 320.0;
ui.allocate_ui(vec2(card_w, 0.0), |ui| {
    MudCard::new().show(ui, |ui| {
        let rect = Rect::from_min_size(ui.cursor().min, vec2(card_w, 180.0));
        ui.scope(|ui| {
            ui.set_clip_rect(rect.intersect(ui.clip_rect()));
            MudCardMedia::new().image("file://assets/hero.png").height(180.0).show(ui);
        });
    });
});
```

This is not applied implicitly because `MudCard` sizes to its content and
therefore does not know its own final rectangle while it is still drawing.

### Density

There is one density. MudBlazor's card has no `Dense` parameter, so neither
does this port; tighten spacing by nesting a `MudStack` with a smaller
`spacing` inside `MudCardContent`.

---

## Performance and memory choices

| Choice | Effect |
|---|---|
| Theme fetched once per `MudCard::show` | One `MudTheme::fetch` and one `palette.surface.into()`. The four companions touch no theme at all. |
| One `egui::Frame` per component | No nested frames, no double compositing. |
| `bool::then` for the stroke and shadow | No `Option<Option<_>>`, no branches in the paint path. |
| Pure helpers on `Copy` values | Radius, stroke, shadow and height repair are arithmetic; unit-testable without a `Context`. |
| `Margin::same` with a constant | No per-frame string or class parsing. |
| `MudCardMedia` delegates to `MudImage` | Async loading, fallback sources and the broken-image placeholder come for free; the two caches (`PATH_CACHE` aside) are the loader's. |
| No `egui::Memory` anywhere | Nothing to invalidate, no re-entrant-`RwLock` hazard. |
| No `Vec` or `String` allocation in `show` | The only allocation is whatever `MudImage` does internally for the URI. |

Not done, on purpose: caching the resolved surface colour or the shadow. Both
depend on the theme, which can change between frames, and both are a handful of
arithmetic operations — a cache would cost more than it saves.

---

## Testing

`cargo test -p mud_egui_components mud_card` runs 20 tests across the five
files:

| File | Tests |
|---|---|
| `mud_card.rs` | radius is 0 when square; stroke only when outlined and carries its colour; shadow is `None` at 0, monotonic, clamped at 24; `pad_u8` / `pad_i8` sanitizing (`NaN`, negative, infinity, `>127`); defaults (elevation **1**); every builder; tag round-trip |
| `mud_card_header.rs` | padding constant is 16 and maps to `i8` 16; empty defaults; builders; tag round-trip |
| `mud_card_content.rs` | same, for its own 16 pt constant |
| `mud_card_actions.rs` | padding constant is 8; empty defaults; builders; tag round-trip |
| `mud_card_media.rs` | default height is **300**; `sane_height` (valid, zero, negative, `NaN`, infinity); defaults; every builder; tag round-trip |

No test creates an `egui::Context`. `MudCard::show`, the three companion `show`
methods and `MudCardMedia::show` need a live `Ui` (layout, painting, theme
lookup) and are therefore left to integration tests. Everything they delegate
to is covered above.

```bash
cargo test -p mud_egui_components mud_card
```

---

## Differences from MudBlazor

- **`Class` is stored but not resolved.** `Elevation` / `Outlined` / `Square`
  own the surface, and each companion owns exactly one padding constant.
  Resolving a utility-class frame on top would composite a second fill or a
  second margin. `classes(..)` is kept so a Blazor call site ports without
  edits, and so the field is available if class resolution is added later.
- **`Style` is not ported.** egui has no inline CSS strings.
- **`UserAttributes` is not ported.** There is no HTML tag.
- **`Elevation` is `u8`, not `int`.** Clamped to `0..=24`, matching every other
  component in this workspace. MudBlazor clamps too; the range is identical.
- **`Height` on `MudCardMedia` is `f32`, not `int`.** Points, consistent with
  `MudStack::height`, `MudPaper::height` and `MudCollapse::max_height`. The
  default is still 300.
- **`MudCardActions` has no `justify` parameter.** MudBlazor's stylesheet does
  not right-align actions either — it only centres them vertically. Insert a
  `MudSpacer` to push buttons to the trailing edge, the same technique
  `MudToolBar` documents. Adding a `justify` builder would be API invention.
- **`MudCardMedia` delegates to `MudImage`.** MudBlazor emits a bare `<img>`
  with `object-fit: cover`. `MudImage` gives the same geometry plus async
  loading, a fallback source and an error placeholder, which a card showing
  remote product photos wants anyway. `Title` maps to `MudImage::alt`.
- **No rounded-corner clipping.** MudBlazor relies on CSS `overflow: hidden`;
  egui frames do not clip. The README shows the `ui.scope` +
  `set_clip_rect` recipe. Applying it implicitly would require `MudCard` to
  know its own size before drawing it, which immediate mode does not allow.
- **No `CardHeaderContent` helper.** It appears in MudBlazor's documentation
  examples but is defined by the docs project, not by MudBlazor. It is not a
  component and is therefore not ported.