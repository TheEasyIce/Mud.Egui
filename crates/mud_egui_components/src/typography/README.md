# MudText for `mud_egui`

A themed, Material Design text component for [egui](https://github.com/emilk/egui), ported from MudBlazor's `MudText`. It provides a fluent builder API for applying semantic typography scales, semantic colors, text alignment, and standard text decorations.

```rust
MudText::h3("Welcome!")
    .color(Color::Primary)
    .bold(true)
    .ui(ui);

```

In addition to explicit builder methods, it features a mini utility-class engine that parses standard spacing (`p-4`, `mx-2`) and background color (`bg-primary`) strings.

---

## Contents

1. [What maps to MudBlazor, and what does not](https://www.google.com/search?q=%23what-maps-to-mudblazor-and-what-does-not)
2. [Architecture](https://www.google.com/search?q=%23architecture)
3. [API reference](https://www.google.com/search?q=%23api-reference)
4. [Examples](https://www.google.com/search?q=%23examples)
5. [Performance considerations](https://www.google.com/search?q=%23performance-considerations)
6. [Design rationale](https://www.google.com/search?q=%23design-rationale)

---

## What maps to MudBlazor, and what does not

The public MudBlazor API was checked to maintain parity. `MudText` exposes `Typo`, `Color`, `Align`, `GutterBottom`, `Inline`, `HtmlTag`, and cascading right-to-left settings.

| MudBlazor | This crate | Notes |
| --- | --- | --- |
| `Typo` | `MudText::typo(Typo)` | Also available via `h1()`..`overline()` constructors

|
| `Color` | `MudText::color(Color)` | Plus `custom_color(Color32)` to bypass the theme

|
| `Align` | `MudText::align(MudAlign)` | Has no effect on inline displays

|
| `GutterBottom` | `MudText::gutter_bottom(bool)` | Adds a `0.35em` bottom margin

|
| `Inline` | `MudText::inline(bool)` | Disables horizontal expansion and alignment

|
| `HtmlTag` | `MudText::html_tag("...")` | Inline-level tags (e.g., `span`, `b`) behave like `inline(true)`

|
| `Class` | `MudText::classes([...])` | Parsed for padding, margins, and background utility strings

|
| `UserAttributes` | `MudText::id(...)`, `aria_label(...)` | `id` is a stable salt, `aria_label` is for screen readers

|
| `ChildContent` | Closure passed to `MudText::new(..)` | Accepts `impl Into<String>`

|

**Rust-only Additions:** `opacity`, `tooltip`, `selectable`, `sense`, `id`, `aria_label`, and direct font overrides (`bold`, `italic`, `uppercase`, `font_size`, `font_family`) are `egui`-native conveniences.

**Deliberately not added:** MudBlazor's `MudText` has no value binding, callbacks, `Variant`, `Size`, `Elevation`, `Disabled`, or `ReadOnly`, so they are not invented here.

---

## Architecture

Rendering is a two-stage pipeline to separate pure logic from immediate-mode UI allocations:

1. **Resolve (`MudText::resolve_style`):** A pure function from `(component state, theme, fallback text color)` to a `ResolvedStyle`. It never touches an `egui::Ui`, never allocates (except cloning an `Arc` for font families), and is completely unit-testable. Precedence is always `builder override > theme variant`.


2. **Render (`MudText::ui` / `MudText::ui_with_theme`):** Turns the resolved style into an `egui::Label`, optionally wrapped in alignment and utility `egui::Frame` layers.



---

## API reference

### Shorthand Constructors

Instead of `MudText::new("...").typo(Typo::H1)`, you can use shorthand constructors:

* `h1` through `h6`
* `subtitle1`, `subtitle2`
* `body1`, `body2`
* `button`, `caption`, `overline`

### Modifier Methods

Every setter consumes and returns `Self` (fluent builder pattern).

| Method | Description |
| --- | --- |
| `custom_color(Color32)` | Explicitly bypasses semantic `MudTheme` colors.

|
| `truncate(bool)` | Overflows are truncated with an ellipsis (`...`).

|
| `no_wrap(bool)` | Prevents text from wrapping to a new line.

|
| `underline(bool)` / `strikethrough(bool)` | Adds text decoration lines.

|
| `selectable(bool)` | Allows the user to highlight and copy the text.

|
| `monospace(bool)` | Forces fixed-width typography.

|
| `opacity(f32)` | `[0.0, 1.0]` opacity multiplier; `NaN` is ignored.

|
| `sense(Sense)` | Attaches interaction tracking (e.g., `Sense::click()`) to the label.

|
| `tooltip(String)` | Attaches an on-hover tooltip string.

|

---

## Examples

### Themed Typography

```rust
ui.add(MudText::h1("Main Title").color(Color::Primary));
ui.add(MudText::body2("Standard paragraph text here."));

```

### Uppercase & Inline Styles

```rust
// Typo::Button defaults to uppercase; uppercase(false) disables it.
MudText::button("Submit").uppercase(false).ui(ui);

// Inline elements only take the space they need.
MudText::new("A span").html_tag("span").ui(ui);

```

### Utility Classes

```rust
MudText::h4("Highlighted")
    .classes(["p-4", "mx-2", "bg-primary"])
    .ui(ui);

```

### Capturing Interactivity

```rust
let response = MudText::body1("Click me!")
    .sense(Sense::click())
    .id("clickable-text")
    .ui(ui);

if response.clicked() {
    println!("Text was clicked!");
}

```

---

## Performance considerations

* **Zero-copy Fast Path:** The text buffer is *moved* into `egui::RichText` via `std::mem::take` (zero allocations, zero copies) unless an uppercase transform is explicitly required.


* **Shared Theme Lookups:** [`MudText::ui_with_theme`] allows a caller drawing multiple text nodes (e.g., lists or tables) to look up the `MudTheme` once and share it across all texts, avoiding per-widget context fetching.


* **Sanitized Inputs:** Invalid numeric overrides (`NaN`, infinite values, zero, or negative font sizes) are rejected during the resolution stage so rendering logic never sees them.



---

## Design rationale

* **Pure State Resolution:** `ResolvedStyle` encapsulates exact point sizes, weights, layouts, and colors. This separates complex theme precedence (e.g., specific font family vs. monospace toggle vs. explicit overrides) from rendering side effects.


* **HTML Semantic Tags:** `is_inline_html_tag` mimics web browser behaviors by automatically switching layout modes to inline when tags like `a`, `span`, or `code` are supplied.


* **No Interior Mutability:** The `MudText` builder uses plain properties, enabling intermediate variable bindings if you need to reuse partial configurations across several UI elements.