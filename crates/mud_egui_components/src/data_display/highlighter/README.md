# MudHighlighter for `mud_egui`

A themed text highlighter for [egui](https://github.com/emilk/egui), ported
from MudBlazor's [`MudHighlighter`](https://mudblazor.com/components/highlighter).
Renders a block of text with one or more search fragments visually emphasised,
using the existing **`MudText`** widget for every fragment so typography and
theme updates propagate automatically.

```rust
MudHighlighter::new("The quick brown fox jumps over the lazy dog")
    .highlighted_text("fox")
    .show(ui);
```

---

## Contents

1. [Design principle: API parity, not API invention](#design-principle-api-parity-not-api-invention)
2. [Architecture](#architecture)
3. [API reference](#api-reference)
4. [Visual behaviour](#visual-behaviour)
5. [Examples](#examples)
6. [Matching algorithm](#matching-algorithm)
7. [Performance and memory choices](#performance-and-memory-choices)
8. [Testing](#testing)
9. [Differences from MudBlazor](#differences-from-mudblazor)

---

## Design principle: API parity, not API invention

The MudBlazor `MudHighlighter` API was checked against the published API docs
and the source (`MudHighlighter.razor.cs`) before this port. The real component
has these parameters plus the shared `MudComponentBase` members:

| MudBlazor parameter | Type | Default |
|---------------------|------|---------|
| `Text` | `string?` | `null` |
| `HighlightedText` | `string?` | `null` |
| `HighlightedTexts` | `IEnumerable<string>?` | `null` |
| `CaseSensitive` | `bool` | `false` |
| `UntilNextBoundary` | `bool` | `false` |
| `Markup` | `bool` | `false` |
| `Class` (base) | `string?` | `null` |
| `Style` (base) | `string?` | `null` |
| `Tag` (base) | `object?` | `null` |
| `UserAttributes` (base) | `Dictionary<string, object>?` | — |

**`Markup` is deliberately not ported.** It tells Blazor to render `Text` as
an HTML `RenderFragment`, which requires a browser DOM. egui has no HTML
parser, so adding a no-op `markup(bool)` would be API invention. If you need
styled runs inside the text, compose multiple `MudHighlighter` / `MudText`
widgets inside a `ui.horizontal_wrapped`.

**`UserAttributes` is not ported** because egui has no HTML tag to splat
attributes onto.

**`Style` maps to `classes(..)`** (utility classes) because egui has no
inline CSS string mechanism. This matches the pattern used by `MudDivider`,
`MudGrid`, and `MudDataGrid`.

---

## Architecture

```
mud_highlighter.rs
├── Segment              — one plain or highlighted text run
├── find_ranges(..)      — pure: needles + text → sorted, merged byte ranges
├── split_segments(..)   — pure: text + ranges → Vec<Segment>
├── MudHighlighter       — builder + rendering
│   └── show(ui) -> Response
│       ├── resolve root frame  (BsStyle / TwStyle)
│       ├── find_ranges + split_segments
│       └── horizontal_wrapped: MudText per segment,
│           highlighted segments wrapped in a zero-padding Frame
└── mod tests            — pure-function tests (no egui context)
```

The module follows the same **pure-layer / thin-painter** split used by
`MudGrid` and `MudDataGrid`: all matching and segmentation logic is free
functions covered by unit tests, and `show` only resolves the theme, calls
those functions, and paints.

---

## API reference

### `MudHighlighter`

| Method | Type | Default | MudBlazor equivalent | Description |
|--------|------|---------|----------------------|-------------|
| `new(text)` | `impl Into<String>` | `""` | `Text` | Creates a highlighter for `text`. |
| `text(s)` | `impl Into<String>` | — | `Text` | Replaces the full text. |
| `highlighted_text(s)` | `impl Into<String>` | none | `HighlightedText` | Adds one fragment. |
| `highlighted_texts(iter)` | `IntoIterator<Item = impl Into<String>>` | none | `HighlightedTexts` | Adds several fragments. |
| `case_sensitive(b)` | `bool` | `false` | `CaseSensitive` | Match case exactly. |
| `until_next_boundary(b)` | `bool` | `false` | `UntilNextBoundary` | Extend each match to the next word boundary. |
| `highlight_color(c)` | `Color32` | `DEFAULT_HIGHLIGHT_COLOR` | *(extension)* | Background colour for highlighted fragments. |
| `classes(iter)` | `iter of AsRef<str>` | none | `Class` | Utility classes on the root frame. |
| `flavor(f)` | `StyleFlavor` | `Bootstrap` | — | Utility-class resolver. |
| `tag(t)` / `tag_ref::<U>()` | `T: Any + Send + Sync` | `None` | `Tag` | Attach and read user data. |
| `show(ui)` | `&mut Ui` | — | — | Render and return the root frame's `Response`. |

### Free functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `find_ranges` | `(&str, &[String], bool, bool) -> Vec<Range<usize>>` | Pure matcher; returns sorted, merged byte ranges. |
| `split_segments` | `(&str, &[Range<usize>]) -> Vec<Segment>` | Splits text into plain / highlighted runs. |

### `Segment`

| Field | Type | Description |
|-------|------|-------------|
| `text` | `String` | The fragment text. |
| `highlighted` | `bool` | Whether this fragment is emphasised. |

### Constants

| Name | Value | Description |
|------|-------|-------------|
| `DEFAULT_HIGHLIGHT_COLOR` | `rgba(255, 235, 59, 120)` | Semi-transparent Material Yellow that works in light and dark themes. |

---

## Visual behaviour

| Case | Behaviour |
|------|-----------|
| No matches | The full text renders as a single plain `MudText`. |
| One match | Text is split into up to three runs (before, match, after); the match receives the highlight background. |
| Overlapping matches | Merged into one contiguous highlighted range so nothing is painted twice. |
| `until_next_boundary` | Each match extends to the next `\w → \W` transition (matching regex `\b`). |
| `case_sensitive(false)` | Unicode-aware case-insensitive matching; multibyte text is safe. |
| Wrapping | Segments render inside `ui.horizontal_wrapped` with zero inter-item spacing, so the highlighted runs wrap naturally across lines. |
| Highlight background | A zero-padding `Frame` with a 2 px corner radius; the fill is `highlight_color`. |

Every text run is drawn through **`MudText::new(..).inline(true)`**, so the
font family, size, weight, and theme colour come from the surrounding UI
exactly as they do for any other `MudText`. Changing the theme propagates
immediately — no separate font configuration is needed.

---

## Examples

### Basic single-fragment highlight

```rust
MudHighlighter::new("The quick brown fox")
    .highlighted_text("quick")
    .show(ui);
```

### Multiple fragments

```rust
MudHighlighter::new("Red, green, and blue")
    .highlighted_texts(["red", "blue"])
    .show(ui);
```

### Case-sensitive matching

```rust
MudHighlighter::new("Rust and rust")
    .highlighted_text("Rust")
    .case_sensitive(true)
    .show(ui);
```

### Until-next-boundary (whole-word emphasis)

```rust
MudHighlighter::new("foobar barbazqux end")
    .highlighted_text("bar")
    .until_next_boundary(true)
    .show(ui);
// Highlights "barbazqux" (from the match to the next whitespace).
```

### Custom highlight colour

```rust
MudHighlighter::new("Styled highlight")
    .highlighted_text("highlight")
    .highlight_color(Color32::from_rgba_unmultiplied(0, 200, 83, 140))
    .show(ui);
```

### Inside a search-results list

```rust
for item in results {
    MudHighlighter::new(&item.name)
        .highlighted_text(&query)
        .show(ui);
    ui.add_space(4.0);
}
```

### Composing with other theme widgets

```rust
ui.horizontal(|ui| {
    ui.add(MudText::new("Result:").bold(true));
    MudHighlighter::new("Ada Lovelace")
        .highlighted_text("Love")
        .show(ui);
});
```

Because every fragment is a `MudText`, the highlighter inherits the same
typography context as its neighbours.

### Tag round-trip

```rust
let h = MudHighlighter::new("data")
    .highlighted_text("at")
    .tag("search-result-42");

assert_eq!(h.tag_ref::<&str>(), Some(&"search-result-42"));
h.show(ui);
```

---

## Matching algorithm

1. **Collect raw matches.** For each non-empty needle, scan the text
   character-by-character (Unicode-safe) and record every start/end pair.
   Case-insensitive matching uses per-character case folding, not
   `to_lowercase()` on the whole string, so byte offsets stay valid for
   multibyte text.
2. **Extend to boundary (optional).** If `until_next_boundary` is enabled,
   each match’s end advances while the next character is a word character
   (`\w`: alphanumeric or `_`), stopping at the first non-word character or the
   end of the text. This matches MudBlazor’s regex-boundary behaviour.
3. **Sort by start offset.** Deterministic ordering for overlapping matches.
4. **Merge overlapping ranges.** If the next range starts at or before the
   previous range’s end, extend the previous range’s end. This guarantees every
   byte belongs to at most one segment and prevents double-painted highlights.
5. **Split into segments.** Walk the sorted ranges and emit alternating
   plain / highlighted `Segment`s, including leading and trailing plain runs
   when the first match does not start at byte 0 or the last match does not end
   at `text.len()`.

Complexity: `O(n × m × k)` where `n` is text length in chars, `m` is the number
of needles, and `k` is the needle length. For typical highlighter use cases
(short needles, bounded text) this is linear in practice.

---

## Performance and memory choices

- **No retained render tree.** Columns and segments are built every frame
  from borrowed data; nothing accumulates across frames.
- **Zero-clone match results.** `find_ranges` returns ranges (24 bytes each),
  not substrings. `split_segments` allocates one `String` per segment —
  unavoidable because `MudText` owns its caption — but this is proportional to
  the number of matches, not the text length.
- **`horizontal_wrapped` with `item_spacing.x = 0`.** Segments flow without
  gaps and wrap naturally, without needing a single laid-out `LayoutJob`.
- **Pure functions.** `find_ranges` and `split_segments` take slices and
  return owned vectors; they do not touch `egui::Context` and are unit-tested
  without a window.
- **No per-frame allocation for plain text.** When there are no matches,
  `split_segments` returns a single `Segment` borrowing nothing extra; the
  cost is one `String` clone of the full text (same as any `MudText`).
- **Theme fetched once per frame.** `MudTheme::fetch` is called at the top of
  `show` and reused for both the frame resolver and the highlight colour
  lookup.
- **Unicode-safe.** All offsets are derived from `char_indices`, never from
  byte arithmetic on a lowercased copy, so multibyte text cannot panic or
  mis-slice.

Not done, on purpose: caching the match results across frames. The needles or
text can change every frame (as in a live search box), so a cache would need
invalidation on every call; a direct scan is simpler and has no steady-state
cost.

---

## Testing

`cargo test` runs the module’s tests:

| Area | Tests |
|------|-------|
| `find_ranges` | empty needles, empty text, simple match, multiple occurrences, case-insensitive default, case-sensitive, `until_next_boundary` at word end / whitespace / punctuation, overlapping merge, multiple needles, multibyte text, needle longer than text |
| `split_segments` | no ranges, leading / middle / trailing range, whole-text range |
| Builders | defaults match MudBlazor, every builder sets its field, `text(..)` replaces the previous value, tag round-trips by type |

There are **no** rendering tests because `show` needs a live `egui::Context`;
the pure layer is the source of truth for matching and segmentation, exactly as
in `MudGrid` and `MudDataGrid`.

```bash
cargo test -p mud_egui_components mud_highlighter
```

---

## Differences from MudBlazor

- **`Markup` not ported.** egui has no HTML parser; the parameter has no
  meaningful equivalent. Compose `MudHighlighter` / `MudText` widgets for
  styled runs instead.
- **`Style` → `classes(..)`.** egui has no inline CSS strings; utility classes
  are resolved through the Bootstrap or Tailwind flavour, same as every other
  `mud_egui` component.
- **`highlight_color(..)` is an extension.** MudBlazor relies on the browser’s
  default `<mark>` styling (yellow). egui has no browser default, so the port
  ships a theme-friendly default and an explicit override.
- **Case-insensitive matching is Unicode-aware.** MudBlazor uses
  `StringComparison.OrdinalIgnoreCase` (culture-invariant); this port folds
  per character, which is correct for the common cases (ASCII, Latin-1,
  Cyrillic, Greek) and never panics on multibyte input.
- **`until_next_boundary` extends to the right only.** This matches
  MudBlazor’s regex `\b` usage: the match starts where the needle starts and
  extends forward to the next boundary.