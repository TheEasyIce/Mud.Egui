# MudCodeBlock (egui)

A code block for Mud.Egui. You pass **plain code text**; it tokenizes, colours and lays it out
for you. It shows an optional header (language label + Copy button), optional line numbers, and
scrolls horizontally instead of wrapping.

> **Scope note.** `MudCodeBlock` is **not** a port of a MudBlazor component. MudBlazor has no
> built-in code block (its docs site uses a custom one). This is an original Mud.Egui component
> and deliberately has no `Variant`, `Color`, `Elevation` or click events.

The only dependency is `egui` (written against 0.37).

## Table of contents

- [Quick start](#quick-start)
- [API reference](#api-reference)
- [Supported languages](#supported-languages)
- [Token colours](#token-colours)
- [Architecture](#architecture)
- [Limitations](#limitations)
- [Testing](#testing)

## Quick start

```rust
// Text in, highlighted block out.
MudCodeBlock::new("fn main() {\n    println!(\"hi\");\n}")
    .language("rust")
    .show(ui);
```

```rust
// Language is optional (a small heuristic guesses it); cap the height and add line numbers.
MudCodeBlock::new(json_text)
    .line_numbers(true)
    .max_height(240.0)
    .show(ui);
```

```rust
// Inside a MudStack: wrap it in `stack.item` so the stack's gap does not leak into it.
MudStack::new().spacing(4).show(ui, |stack| {
    stack.item(|ui| { MudCodeBlock::new(snippet_a).language("rust").show(ui); });
    stack.item(|ui| { MudCodeBlock::new(snippet_b).language("cs").show(ui); });
});
```

## API reference

| Parameter | Type | Default | Builder method | Notes |
|---|---|---|---|---|
| `code` | `String` | — | `MudCodeBlock::new(impl Into<String>)` | Raw text. Trailing newlines are trimmed so no empty last line appears. |
| `language` | `Option<String>` | `None` | `.language(impl Into<String>)` | Case-insensitive name or alias. `None` guesses from the text. Shown as the header label. |
| `header` | `bool` | `true` | `.header(bool)` | Shows the language label and Copy button. |
| `line_numbers` | `bool` | `false` | `.line_numbers(bool)` | Right-aligned gutter; not selectable, so copying by hand does not include numbers. |
| `max_height` | `Option<f32>` | `None` | `.max_height(f32)` | Body scrolls vertically past this height. |
| `id_salt` | `Option<Id>` | `None` | `.id_salt(impl Hash + Debug)` | Disambiguates scroll position and "Copied" state of two blocks with identical text. |

| Method | Signature | Notes |
|---|---|---|
| `show` | `fn show(self, ui: &mut Ui) -> egui::Response` | Draws the block; returns the outer frame's response. |

### Copy button

Clicking **Copy** puts the full code text (without line numbers) on the clipboard through
`ctx.copy_text`, then shows **Copied** for 1.5 seconds. It requests a repaint so the label
reverts without needing input.

### Public highlighter

The tokenizer is public and independent of egui:

```rust
use mud_code_block::highlight::{tokenize, guess_language, is_known_language, TokenKind};

for (start, end, kind) in tokenize(code, "rust") {
    println!("{kind:?}: {}", &code[start..end]);
}
```

`tokenize` returns contiguous `(start, end, kind)` byte ranges that cover the whole input;
concatenating them reproduces the text exactly. Adjacent ranges of the same kind are merged.

## Supported languages

| Names / aliases | Notes |
|---|---|
| `rust`, `rs` | Keywords, `name!` macros, char literals vs. lifetimes, `//` and `/* */` comments. |
| `js`, `javascript`, `ts`, `typescript`, `jsx`, `tsx` | Backtick strings, `//` and `/* */`. |
| `py`, `python` | `#` comments. |
| `cs`, `csharp`, `c#` | `//` and `/* */`. |
| `json`, `jsonc` | Strings, numbers, `true`/`false`/`null`. |
| `toml`, `ini`, `yaml`, `yml` | `#` comments, strings, numbers, booleans. |
| `sh`, `bash`, `shell`, `zsh`, `console` | `#` comments, common commands as keywords. |
| anything else | Generic fallback: `//`, `#` and `/* */` comments, common keywords. |

## Token colours

Two built-in palettes are chosen from `ui.visuals().dark_mode`.

| Token | Meaning |
|---|---|
| Comment (italic) | Line and block comments |
| String | `"..."`, `'...'`, backtick strings, Rust char literals |
| Number | Digit-led runs such as `42`, `1.5`, `0xFF`, `1_000` |
| Keyword | Per-language keyword list |
| Type | Capitalised identifiers that are not keywords |
| Function | An identifier directly followed by `(` |
| Macro | Rust `name!` |
| Punct | ASCII punctuation |
| Plain | Everything else (identifiers, whitespace, non-ASCII) |

## Architecture

```
┌──────────────────────────────────────────────────────────────┐
│  Render path (needs a live egui::Context)                     │
│  MudCodeBlock::show                                           │
│  palette (light/dark) → build_job (LayoutJob) → Frame/header  │
│  → ScrollArea → Label (+ optional line-number Label)          │
└──────────────────────────────────────────────────────────────┘
                              │ calls
                              ▼
┌──────────────────────────────────────────────────────────────┐
│  highlight (pure, no egui types — unit-tested directly)       │
│  tokenize · guess_language · is_known_language · TokenKind    │
└──────────────────────────────────────────────────────────────┘
```

Text is built as one `LayoutJob` with `wrap.max_width = f32::INFINITY` so lines never wrap;
horizontal overflow is handled by the `ScrollArea`. The code label is selectable; the line
numbers are a separate non-selectable label using the same monospace font and size, so rows line
up.

## Limitations

- **Not a full parser.** Raw strings (`r#"..."#`), nested block comments and template-literal
  interpolation (`${}`) are coloured approximately. Text is never dropped, reordered or
  panicked on.
- **No semantic highlighting.** "Type" means "capitalised identifier" and "Function" means
  "followed by `(`".
- **Language guess is a heuristic.** Pass `.language(..)` when you know it.
- **Very long single lines** are laid out as one galley; extremely large inputs (hundreds of
  thousands of lines) should be paginated or capped with `max_height`.
- If you need exact, grammar-based highlighting for many languages, use `egui_extras` with its
  `syntect` feature instead; it is heavier but complete.

## Testing

The tokenizer has unit tests that need no `egui::Context`:

- lossless, contiguous ranges on empty, non-ASCII, unterminated-string and unterminated-comment
  input, across every language;
- Rust keywords, types, functions, macros, numbers;
- strings containing `//`, escapes not ending a string, lifetimes vs. char literals;
- `!=` not treated as a macro; block comments spanning lines;
- Python comments and keywords; JSON values; language lookup and guessing.

Run with:

```sh
cargo test
```

The render path (`MudCodeBlock::show`) needs a live `Ui` and is not covered by these tests.