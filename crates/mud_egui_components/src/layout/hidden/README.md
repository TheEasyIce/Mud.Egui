# MudHidden for `mud_egui`

A visibility gate for [egui](https://github.com/emilk/egui): renders its content
only when the available width falls inside (or, with `invert`, outside) a
breakpoint. Ported from MudBlazor's
[`MudHidden`](https://mudblazor.com/components/hidden) (MudBlazor 8.13.0).

```rust
// Show the wide layout at Md and above.
MudHidden::new()
.breakpoint(Breakpoint::MdAndUp)
.show(ui, |ui| {
ui.add(MudText::body1("Wide layout"));
});

// Show the compact layout below Md.
MudHidden::new()
.breakpoint(Breakpoint::MdAndUp)
.invert(true)
.show(ui, |ui| {
ui.add(MudText::body1("Compact layout"));
});
```

The content closure supplies every string and glyph; `MudHidden` paints none of
its own. Route them through **`MudText`** and **`MudIcon`** as usual.

---

## Contents

1. [Design principle: API parity, not API invention](#design-principle-api-parity-not-api-invention)
2. [Architecture](#architecture)
3. [API reference](#api-reference)
4. [Examples](#examples)
5. [Visibility pipeline](#visibility-pipeline)
6. [Performance and memory choices](#performance-and-memory-choices)
7. [Testing](#testing)
8. [Differences from MudBlazor](#differences-from-mudblazor)

---

## Design principle: API parity, not API invention

The MudBlazor 8.13.0 parameter surface was verified against DNDocs before
implementation. `MudHidden` has **five parameters**, and all five are ported.

### Ported parameters

| MudBlazor | Type | Default | Rust mapping |
|---|---|---|---|
| `Breakpoint` | `Breakpoint` | `None` | `breakpoint(Breakpoint)` |
| `Invert` | `bool` | `false` | `invert(bool)` |
| `ChildContent` | `RenderFragment?` | `null` | closure to `show(ui, \|ui\| ..)` |
| `Hidden` | `bool` | **`true`** | `HiddenResponse::hidden` |
| `HiddenChanged` | `EventCallback<bool>` | — | `HiddenResponse::changed` |
| `Class` | `string?` | `null` | `classes(iter)` — stored, not resolved |
| `Tag` | `object?` | `null` | `tag(t)` / `tag_ref::<U>()` |

### Deliberately not ported

| MudBlazor | Reason |
|---|---|
| `Style` | egui has no inline CSS strings. `MudHidden` renders no element of its own, so there is nothing to style. |
| `UserAttributes` | There is no HTML tag to splat attributes onto. |
| `Class` resolution | Stored on `base` for parity. `MudHidden` adds no wrapper frame, so a class list has no surface to paint; the child content's own classes apply instead. |
| `CurrentBreakpointFromProvider` | A cascading parameter. egui has no cascading-parameter system; the port reads `ui.available_width()` (see [Differences from MudBlazor](#differences-from-mudblazor)). |

### Extensions

| Extension | Type | Default | Purpose |
|---|---|---|---|
| `id_salt(impl Hash + Debug)` | `Id` | `None` | Distinguishes two `MudHidden`s in one parent, whose `HiddenChanged` edges would otherwise share a memory key. Same shape as `MudCollapse::id_salt`. |

### `Hidden` is an output, not a binding

`Hidden` is a two-way bindable parameter in Blazor, but the component **computes**
it from `(Breakpoint, Invert, width)` every frame — the caller never controls
it. The port therefore exposes it as `HiddenResponse::hidden`, the same call
`MudCollapse` makes for `expanded_changed`.

### `Breakpoint` gained nine variants

The 8.13.0 `Breakpoint` enum has **sixteen** variants, not the seven the
workspace previously had: six simple buckets (`Xs`..`Xxl`), eight compound ranges
(`SmAndDown`..`XlAndUp`), and two sentinels (`None`, `Always`). The port extends
`mud_egui_core::enums::Breakpoint` to match — the extension is purely additive,
and every existing `to_pixels()` / `>=` use of the enum keeps working unchanged.

The distinction the compound variants exist for is:

| Variant | Matches |
|---|---|
| `Lg` | the `[1280, 1920)` bucket only |
| `LgAndUp` | `width >= 1280` |
| `LgAndDown` | `width < 1920` |

So `Breakpoint::Lg` hides only when the viewport is a desktop but not a
4K monitor; `Breakpoint::LgAndUp` hides on desktop and larger. Use the compound
variants for the common "hide at and above" / "hide at and below" cases.

---

## Architecture

```
layout/hidden/
├── mud_hidden.rs
│   ├── MudHidden          builder + show(ui, add_content) + Widget impl
│   ├── HiddenResponse<R>  response + hidden + changed + inner
│   └── compute_hidden     pure visibility rule
└── mod.rs                 re-exports

mud_egui_core/src/enums/
└── breakpoint.rs          sixteen variants + matches / to_pixels / from_pixels
```

**Pure layer.** `compute_hidden(breakpoint, invert, width)` is the whole
decision. `Breakpoint::matches(width)` is the range test it delegates to. Both
are unit-testable without an `egui::Context`.

**State layer.** One `bool` per instance lives in `egui::Memory`, keyed by
`<ui.id().with(id_salt)>.mud_hidden_prev`. It exists only so `HiddenChanged` can
fire once per flip rather than every frame.

**Component layer.** `show` reads the available width, computes the hidden
state, updates the memory, and either allocates a zero-sized response (hidden)
or runs the content closure in a `ui.scope` (visible). The closure is **never**
called while hidden — that is the component's whole purpose.

---

## API reference

### `MudHidden`

| Method | Type | Default | Mirrors | Description |
|---|---|---|---|---|
| `new()` | `Self` | — | — | No breakpoint, not inverted. |
| `breakpoint(b)` | `Breakpoint` | `None` | `Breakpoint` | The range to test against. |
| `invert(b)` | `bool` | `false` | `Invert` | Hide when the range matches instead of showing. |
| `id_salt(s)` | `impl Hash + Debug` | `None` | *(extension)* | Distinguish two instances in one parent. |
| `classes(iter)` | `iter of AsRef<str>` | none | `Class` | Stored on `ComponentBase`. |
| `tag(t)` / `tag_ref::<U>()` | `T: Any + Send + Sync` | `None` | `Tag` | Arbitrary user data. |
| `show(ui, \|ui\| ..)` | `&mut Ui, FnOnce(&mut Ui) -> R` | — | `ChildContent` | Returns `HiddenResponse<R>`. |
| `ui.add(..)` | — | — | — | Evaluates the rule but draws no content. |

### `HiddenResponse<R>`

| Field | Type | Description |
|---|---|---|
| `response` | `Response` | The content's response when visible; a zero-sized hover response when hidden. Deref target. |
| `hidden` | `bool` | The computed hidden state. Mirrors `Hidden`. |
| `changed` | `bool` | `true` on the frame the hidden state flipped. Mirrors `HiddenChanged`; never set on the first frame. |
| `inner` | `Option<R>` | The content closure's value; `None` when hidden. |

`HiddenResponse` implements `Deref<Target = Response>`, so `out.hovered()`,
`out.rect` and friends work directly.

### `Breakpoint`

| Variant | Matches (width, points) | Doc label |
|---|---|---|
| `Xs` | `< 600` | A small to large phone |
| `Sm` | `[600, 960)` | A small to medium tablet |
| `Md` | `[960, 1280)` | A large tablet or laptop |
| `Lg` | `[1280, 1920)` | A desktop computer |
| `Xl` | `[1920, 2560)` | A high-definition or 4K desktop monitor |
| `Xxl` | `>= 2560` | An ultra-wide 4K+ desktop monitor |
| `SmAndDown` | `< 960` | Sm or smaller |
| `MdAndDown` | `< 1280` | Md or smaller |
| `LgAndDown` | `< 1920` | Lg or smaller |
| `XlAndDown` | `< 2560` | Xl or smaller |
| `SmAndUp` | `>= 600` | Sm or larger |
| `MdAndUp` | `>= 960` | Md or larger |
| `LgAndUp` | `>= 1280` | Lg or larger |
| `XlAndUp` | `>= 1920` | Xl or larger |
| `None` | never | No breakpoint applies |
| `Always` | always | Content will always be visible |

Helpers: `ALL`, `name()`, `label()`, `is_sentinel()`, `is_compound()`,
`to_pixels()`, `matches(width)`, `from_pixels(width)`, `Display`.

### Constants

| Name | Value | Meaning |
|---|---|---|
| `SM_PX` | `600.0` | Smallest `Sm` width |
| `MD_PX` | `960.0` | Smallest `Md` width |
| `LG_PX` | `1280.0` | Smallest `Lg` width |
| `XL_PX` | `1920.0` | Smallest `Xl` width |
| `XXL_PX` | `2560.0` | Smallest `Xxl` width |

---

## Examples

### Show only at Md and above

```rust
MudHidden::new()
.breakpoint(Breakpoint::MdAndUp)
.show(ui, |ui| {
ui.add(MudText::body1("Sidebar"));
});
```

### Show only below Md (the inverse)

```rust
MudHidden::new()
.breakpoint(Breakpoint::MdAndUp)
.invert(true)
.show(ui, |ui| {
ui.add(MudText::body1("Hamburger menu"));
});
```

### Show only in the Md bucket (not and-up)

```rust
MudHidden::new()
.breakpoint(Breakpoint::Md)
.show(ui, |ui| {
ui.add(MudText::body2("Tablet-only panel"));
});
```

`Breakpoint::Md` matches only `[960, 1280)`; use `MdAndUp` to show at Md and
larger.

### Show only on phones (up to Sm)

```rust
MudHidden::new()
.breakpoint(Breakpoint::SmAndDown)
.invert(true)
.show(ui, |ui| {
ui.add(MudText::body1("Mobile layout"));
});
```

`SmAndDown` matches `< 960`; inverting makes it hide below 960 and show at
960 and above. To show only at `< 960`, drop the `.invert(true)`.

### Hide at Lg and larger

```rust
MudHidden::new()
.breakpoint(Breakpoint::LgAndUp)
.show(ui, |ui| {
ui.add(MudText::body2("Visible on tablets and phones"));
});
```

At `Lg`, `Xl` and `Xxl`, `LgAndUp` matches and the content is hidden; below
`Lg`, it does not match and the content is visible.

### Observing `HiddenChanged`

```rust
let out = MudHidden::new()
.breakpoint(Breakpoint::MdAndUp)
.show(ui, |ui| {
ui.add(MudText::body1("Content"));
});

if out.changed {
log::info!("hidden = {}", out.hidden);
}
```

`changed` is `true` for exactly one frame per flip; the previous value is kept
in `egui::Memory` keyed by the widget id.

### Two gates in one parent

```rust
MudHidden::new()
.id_salt("wide")
.breakpoint(Breakpoint::LgAndUp)
.show(ui, |ui| { /* … */ });

MudHidden::new()
.id_salt("narrow")
.breakpoint(Breakpoint::SmAndDown)
.show(ui, |ui| { /* … */ });
```

Without distinct salts the two share a memory key and their `changed` flags
would cross.

### `Always` (never hidden)

```rust
MudHidden::new()
.breakpoint(Breakpoint::Always)
.show(ui, |ui| {
ui.add(MudText::body2("Always visible"));
});
```

`Always` is a sentinel: it is never hidden, even with `invert(true)`.

---

## Visibility pipeline

```
available_width ──► Breakpoint::matches(width) ──► XOR invert ──► hidden?
                                                                 │
                          ┌──────────────────────────────────────┴──────────┐
                          ▼                                                 ▼
                     hidden == true                                    hidden == false
                     allocate Vec2::ZERO                               ui.scope(add_content)
                     Sense::hover()                                    Response = content's
                     inner = None                                      inner = Some(R)
```

`Always` short-circuits to `hidden = false` regardless of `invert` — that is the
variant's documented meaning, "Content will always be visible".

`None` matches nothing, so with the default `invert = false` the content is
visible. That is the least surprising reading of "no breakpoint specified".

### Per-frame cost

| Path | Work |
|---|---|
| Hidden | One `available_width()` read, one `matches` call, one memory read, one `insert_temp` of a `bool`, one zero-sized allocation. The content closure is **not called**. |
| Visible | The above, plus a `ui.scope` and the closure. |

No `Vec`, no `String`, no theme lookup.

---

## Performance and memory choices

| Choice | Effect |
|---|---|
| Content closure is `FnOnce` | The closure runs **at most once**, and never when hidden. This is the whole point of the component. |
| `Breakpoint::matches` is pure | One match arm and a half-open range check; no allocation. |
| One `bool` in `egui::Memory` | The only persisted state; read and written once per frame. |
| Zero-sized hover response when hidden | Takes no layout space and steals no clicks. |
| No theme fetch | `MudHidden` resolves no colours; the child content owns its own theming. |
| `id_salt` for disambiguation | Two instances in one parent do not share their `changed` edge. |

---

## Testing

`cargo test -p mud_egui_components mud_hidden` runs 15 pure-function tests:

| Area | Tests |
|---|---|
| Visibility rule | `None` with/without invert; `Always` never hidden even inverted; simple bucket hide-inside vs show-inside; `*AndDown` and `*AndUp` both ways; `Xs` and `Xxl` boundary cases |
| Builders | Defaults match MudBlazor; every field set; `id_salt` stability |

Plus 21 tests in `mud_egui_core` for `Breakpoint`:

| Area | Tests |
|---|---|
| `ALL` / `name` / `Display` | Sixteen unique variants; lowercase names; `Display` matches `name` |
| Classification | `is_sentinel`, `is_compound` |
| `to_pixels` | Simple lower bounds; `*AndUp` lower bounds; `*AndDown` upper bounds; sentinels are `0.0` |
| `matches` | Sentinels; every simple bucket at its boundaries; every compound at its boundaries; non-finite width rejected except by `Always`; negative width treated as zero |
| `from_pixels` | Round-trip at each lower bound; non-finite repair |

No test creates an `egui::Context`. `MudHidden::show` needs a live `Ui` (for
`available_width`, the scope, and memory) and is therefore left to integration
tests; everything it delegates to is covered above.

Suggested integration tests (`egui_kittest` or a headless `Context::run`):

- A `MudHidden` with `Breakpoint::MdAndUp` draws its content at 1280 pt and
  draws nothing at 599 pt.
- The content closure is not called when hidden (use a counter in a `Cell`).
- `invert(true)` swaps the two cases.
- `changed` is `true` for exactly one frame after a resize crosses the
  boundary, and `false` on steady frames.
- Two `MudHidden`s in one parent with distinct `id_salt`s track their own
  `changed` edges independently.
- `Breakpoint::Always` draws its content at every width, even with `invert(true)`.

```bash
cargo test -p mud_egui_core breakpoint
cargo test -p mud_egui_components mud_hidden
```

---

## Differences from MudBlazor

- **The width comes from `ui.available_width()`, not the browser window.**
  MudBlazor uses JavaScript (`IBrowserViewportService`) to observe the browser
  window's size. egui has no browser window, so the port reads the **container**
  width instead. That is the immediate-mode equivalent and is arguably more
  correct inside a split panel or a narrow side column, where the available
  width is what actually matters.
- **`Hidden` is an output, not a two-way binding.** The component computes it
  from `(Breakpoint, Invert, width)` every frame; the port returns it on
  `HiddenResponse::hidden` instead of taking a `&mut bool`. This is the same
  split `MudCollapse` uses for `expanded_changed`.
- **`HiddenChanged` is a response field, not a callback.** `changed` is `true`
  for exactly one frame per flip, keyed in `egui::Memory`. The initial frame
  never reports `changed` (there is no previous value to compare against),
  which is the same contract `MudCollapse::expanded_changed` and
  `MudRating::hovered_changed` use.
- **`Class` is stored but not resolved.** `MudHidden` renders no wrapper
  element, so a class list has no surface to paint. `classes(..)` is kept for
  parity; the child content's own classes apply instead.
- **`Style` and `UserAttributes` are not ported.** No inline CSS strings, no
  HTML tag.
- **`CurrentBreakpointFromProvider` is not ported.** It is a cascading
  parameter; egui has no cascading-parameter system. The port reads the
  available width directly, which is equivalent for the only thing the
  parameter is used for.
- **`Breakpoint` gained nine variants.** The 8.13.0 enum has sixteen, not
  seven; the port extends `mud_egui_core::enums::Breakpoint` to match. The
  extension is additive — existing `to_pixels()` / `>=` comparisons keep
  working — but it is a change to a shared enum, not just to `MudHidden`.
  `Breakpoint::Lg` and `Breakpoint::LgAndUp` are now genuinely different (the
  bucket versus the range), whereas a pre-8.13 port that only had seven
  variants had to treat `Lg` as "and up".
- **`Always` is never hidden, even with `invert(true)`.** That is the variant's
  documented meaning ("Content will always be visible"). A naive
  `matches(width) != invert` would hide it when inverted; the port special-cases
  it instead.
- **`None` shows the content by default.** `matches(None, _)` is `false`, so
  `hidden = false != false = false` — visible. MudBlazor's `Hidden` field
  defaults to `true`, but the field is overwritten by the first
  `UpdateAsync`; the steady-state behaviour is what the port matches, and
  "no breakpoint specified → show the content" is the least surprising reading.