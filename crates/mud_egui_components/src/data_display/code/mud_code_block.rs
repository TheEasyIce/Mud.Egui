//! A code block: shows source text with syntax highlighting, an optional language label,
//! optional line numbers, horizontal scrolling and a copy button.
//!
//! You only hand it plain code text; tokenizing and colouring happen inside.
//!
//! ```ignore
//! MudCodeBlock::new("fn main() {\n    println!(\"hi\");\n}").language("rust").show(ui);
//!
//! // Language is optional; without it a small heuristic guesses one.
//! MudCodeBlock::new(json_text).line_numbers(true).max_height(240.0).show(ui);
//!
//! // Bootstrap / Tailwind utilities work like on MudPaper / MudContainer.
//! MudCodeBlock::new(code)
//!     .classes(["bg-surface", "rounded-lg", "shadow-md", "border", "p-4"])
//!     .show(ui);
//! ```
//!
//! # Layout
//! - `highlight` (pure, no egui): [`highlight::tokenize`] turns text into contiguous
//!   `(start, end, TokenKind)` byte ranges. Unit-tested without a `Context`.
//! - render path: palette selection (light/dark), `LayoutJob` building, frame, header, copy.
//!
//! The highlighter is intentionally lightweight (comments, strings, numbers, keywords,
//! types, functions, macros, punctuation). It is not a full parser, so exotic syntax
//! (raw strings, nested block comments, template literals with `${}`) is coloured
//! approximately but never panics and never drops or reorders text.
//!
//! # Styling
//!
//! Utility classes on [`MudCodeBlock::base`] are resolved by the active
//! [`MudCodeBlock::flavor`]:
//!
//! * [`StyleFlavor::Bootstrap`] (default) uses [`BsStyle`]. No variants, so no
//!   per-frame interaction state is required.
//! * [`StyleFlavor::Tailwind`] uses [`TwStyle`]. Tailwind supports `hover:`,
//!   `dark:` and `active:` variants; the previous frame's interaction state is
//!   therefore threaded through [`TwState`].
//!
//! Resolution order for the **outer** frame is:
//!
//! **classes → code-block defaults (bg / border / radius, only where unset) →
//! explicit `fill` / `stroke` / `corner_radius` overrides**. `p-*` padding from
//! classes is extracted and applied to the *content* frame instead of the outer
//! frame, so `p-4` pads the code body, not the header band.

use std::any::Any;
use std::hash::{Hash, Hasher};

use egui::{
    text::LayoutJob, Align, Color32, CornerRadius, FontId, Frame, Id, Label, Layout, Margin,
    RichText, ScrollArea, Stroke, TextFormat, Ui,
};
use mud_egui_bootstrap::bs_style::BsStyle;
use mud_egui_core::base::component_base::ComponentBase;
use mud_egui_core::enums::StyleFlavor;
use mud_egui_tailwind::tw_style::{TwState, TwStyle};
use mud_egui_theming::theme::MudTheme;

pub use highlight::TokenKind;

// =============================================================================
// Constants
// =============================================================================

/// Cap passed to `base.set_elevation` so elevation utilities behave predictably.
const MAX_ELEVATION: u8 = 24;
/// Base rem size (points) for the Tailwind resolver.
const REM_SIZE_PT: f32 = 14.0;
/// Fallback corner radius when neither a class nor an override set one.
const DEFAULT_RADIUS: u8 = 8;
/// Body font size.
const FONT_SIZE: f32 = 13.0;
/// Seconds the copy button shows "Copied" after a click.
const COPIED_FOR: f64 = 1.5;
/// Content padding when the class frame did not supply one (`p-*`).
const DEFAULT_CONTENT_PADDING: Margin = Margin {
    left: 12,
    right: 12,
    top: 10,
    bottom: 10,
};
/// Header padding.
const DEFAULT_HEADER_PADDING: Margin = Margin {
    left: 12,
    right: 12,
    top: 6,
    bottom: 6,
};

// ---- highlight:begin ---------------------------------------------------------------
/// Pure tokenizer. No egui types, so it can be tested anywhere.
pub mod highlight {
    /// What a run of text is, for colouring.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum TokenKind {
        Plain,
        Comment,
        String,
        Number,
        Keyword,
        Type,
        Function,
        Macro,
        Punct,
    }

    struct Spec {
        keywords: &'static [&'static str],
        line_comments: &'static [&'static str],
        block_comments: bool,
        backtick_strings: bool,
        rust_chars: bool,
        macros: bool,
    }

    const RUST: Spec = Spec {
        keywords: &[
            "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum",
            "extern", "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod",
            "move", "mut", "pub", "ref", "return", "self", "Self", "static", "struct", "super",
            "trait", "true", "type", "unsafe", "use", "where", "while",
        ],
        line_comments: &["//"],
        block_comments: true,
        backtick_strings: false,
        rust_chars: true,
        macros: true,
    };
    const JS: Spec = Spec {
        keywords: &[
            "async", "await", "break", "case", "catch", "class", "const", "continue", "default",
            "delete", "do", "else", "export", "extends", "false", "finally", "for", "from",
            "function", "if", "import", "in", "instanceof", "interface", "let", "new", "null",
            "of", "return", "static", "super", "switch", "this", "throw", "true", "try", "type",
            "typeof", "undefined", "var", "void", "while", "yield",
        ],
        line_comments: &["//"],
        block_comments: true,
        backtick_strings: true,
        rust_chars: false,
        macros: false,
    };
    const PY: Spec = Spec {
        keywords: &[
            "and", "as", "assert", "async", "await", "break", "class", "continue", "def", "del",
            "elif", "else", "except", "False", "finally", "for", "from", "global", "if", "import",
            "in", "is", "lambda", "None", "nonlocal", "not", "or", "pass", "raise", "return",
            "True", "try", "while", "with", "yield",
        ],
        line_comments: &["#"],
        block_comments: false,
        backtick_strings: false,
        rust_chars: false,
        macros: false,
    };
    const CS: Spec = Spec {
        keywords: &[
            "abstract", "async", "await", "base", "bool", "break", "case", "catch", "class",
            "const", "continue", "default", "else", "enum", "false", "finally", "for", "foreach",
            "if", "in", "interface", "internal", "namespace", "new", "null", "override",
            "private", "protected", "public", "readonly", "return", "static", "string", "struct",
            "switch", "this", "throw", "true", "try", "using", "var", "virtual", "void", "while",
        ],
        line_comments: &["//"],
        block_comments: true,
        backtick_strings: false,
        rust_chars: false,
        macros: false,
    };
    const JSON: Spec = Spec {
        keywords: &["true", "false", "null"],
        line_comments: &[],
        block_comments: false,
        backtick_strings: false,
        rust_chars: false,
        macros: false,
    };
    const TOML: Spec = Spec {
        keywords: &["true", "false"],
        line_comments: &["#"],
        block_comments: false,
        backtick_strings: false,
        rust_chars: false,
        macros: false,
    };
    const SHELL: Spec = Spec {
        keywords: &[
            "case", "do", "done", "elif", "else", "esac", "fi", "for", "function", "if", "in",
            "then", "while", "export", "echo", "cd", "cargo", "git",
        ],
        line_comments: &["#"],
        block_comments: false,
        backtick_strings: true,
        rust_chars: false,
        macros: false,
    };
    const GENERIC: Spec = Spec {
        keywords: &[
            "if", "else", "for", "while", "return", "true", "false", "null", "class", "function",
        ],
        line_comments: &["//", "#"],
        block_comments: true,
        backtick_strings: false,
        rust_chars: false,
        macros: false,
    };

    fn spec_for(lang: &str) -> &'static Spec {
        match lang.trim().to_ascii_lowercase().as_str() {
            "rust" | "rs" => &RUST,
            "js" | "javascript" | "ts" | "typescript" | "jsx" | "tsx" => &JS,
            "py" | "python" => &PY,
            "cs" | "csharp" | "c#" => &CS,
            "json" | "jsonc" => &JSON,
            "toml" | "ini" | "yaml" | "yml" => &TOML,
            "sh" | "bash" | "shell" | "zsh" | "console" => &SHELL,
            _ => &GENERIC,
        }
    }

    /// Normalised name of a language the highlighter knows, or `None`.
    pub fn is_known_language(lang: &str) -> bool {
        !std::ptr::eq(spec_for(lang), &GENERIC)
    }

    /// A cheap guess for when no language was given.
    pub fn guess_language(code: &str) -> &'static str {
        let t = code.trim_start();
        if t.starts_with('{') || t.starts_with('[') {
            "json"
        } else if t.contains("fn ") || t.contains("let mut ") || t.contains("use std::") || t.contains("impl ") {
            "rust"
        } else if t.contains("def ") || t.starts_with("import ") && !t.contains(';') {
            "python"
        } else if t.contains("console.log") || t.contains("=> ") || t.contains("function ") {
            "js"
        } else if t.contains("namespace ") || t.contains("public class") {
            "cs"
        } else if t.starts_with("$ ") || t.starts_with("cargo ") || t.starts_with("git ") {
            "sh"
        } else {
            "text"
        }
    }

    #[inline]
    fn is_ident_start(b: u8) -> bool {
        b.is_ascii_alphabetic() || b == b'_'
    }
    #[inline]
    fn is_ident_part(b: u8) -> bool {
        b.is_ascii_alphanumeric() || b == b'_'
    }

    /// Splits `code` into contiguous `(start, end, kind)` byte ranges covering all of it.
    /// Adjacent ranges of the same kind are merged. Concatenating the ranges reproduces
    /// `code` exactly. Never panics on any input (ranges always fall on char boundaries).
    pub fn tokenize(code: &str, lang: &str) -> Vec<(usize, usize, TokenKind)> {
        let spec = spec_for(lang);
        let b = code.as_bytes();
        let n = b.len();
        let mut out: Vec<(usize, usize, TokenKind)> = Vec::new();
        let mut i = 0;

        let push = |out: &mut Vec<(usize, usize, TokenKind)>, s: usize, e: usize, k: TokenKind| {
            if s >= e {
                return;
            }
            if let Some(last) = out.last_mut() {
                if last.2 == k && last.1 == s {
                    last.1 = e;
                    return;
                }
            }
            out.push((s, e, k));
        };

        while i < n {
            let c = b[i];

            // Line comment.
            if let Some(_) = spec.line_comments.iter().find(|p| code[i..].starts_with(**p)) {
                let end = code[i..].find('\n').map_or(n, |o| i + o);
                push(&mut out, i, end, TokenKind::Comment);
                i = end;
                continue;
            }
            // Block comment.
            if spec.block_comments && code[i..].starts_with("/*") {
                let end = code[i + 2..].find("*/").map_or(n, |o| i + 2 + o + 2);
                push(&mut out, i, end, TokenKind::Comment);
                i = end;
                continue;
            }
            // Strings (and Rust char literals).
            let is_quote = c == b'"'
                || (c == b'\'' && {
                if spec.rust_chars {
                    // 'x' or '\n' (a char literal), not a lifetime like 'a.
                    (i + 2 < n && b[i + 2] == b'\'' && b[i + 1] != b'\\')
                        || (i + 1 < n && b[i + 1] == b'\\')
                } else {
                    true
                }
            })
                || (c == b'`' && spec.backtick_strings);
            if is_quote {
                let mut j = i + 1;
                while j < n {
                    if b[j] == b'\\' {
                        j += 2;
                        continue;
                    }
                    if b[j] == c {
                        j += 1;
                        break;
                    }
                    // Single-line quotes end at a newline; backticks may span lines.
                    if b[j] == b'\n' && c != b'`' {
                        break;
                    }
                    j += 1;
                }
                let mut end = j.min(n);
                while !code.is_char_boundary(end) {
                    end += 1;
                }
                push(&mut out, i, end, TokenKind::String);
                i = end;
                continue;
            }
            // Numbers.
            if c.is_ascii_digit() {
                let mut j = i + 1;
                while j < n && (is_ident_part(b[j]) || (b[j] == b'.' && j + 1 < n && b[j + 1].is_ascii_digit())) {
                    j += 1;
                }
                push(&mut out, i, j, TokenKind::Number);
                i = j;
                continue;
            }
            // Identifiers, keywords, types, functions, macros.
            if is_ident_start(c) {
                let mut j = i + 1;
                while j < n && is_ident_part(b[j]) {
                    j += 1;
                }
                let word = &code[i..j];
                let kind;
                let mut end = j;
                if spec.macros && j < n && b[j] == b'!' && !(j + 1 < n && b[j + 1] == b'=') {
                    kind = TokenKind::Macro;
                    end = j + 1;
                } else if spec.keywords.contains(&word) {
                    kind = TokenKind::Keyword;
                } else {
                    let mut k = j;
                    while k < n && (b[k] == b' ' || b[k] == b'\t') {
                        k += 1;
                    }
                    if k < n && b[k] == b'(' {
                        kind = TokenKind::Function;
                    } else if c.is_ascii_uppercase() {
                        kind = TokenKind::Type;
                    } else {
                        kind = TokenKind::Plain;
                    }
                }
                push(&mut out, i, end, kind);
                i = end;
                continue;
            }
            // Punctuation (ASCII) or anything else (whitespace, non-ASCII) as plain.
            if c.is_ascii_punctuation() {
                push(&mut out, i, i + 1, TokenKind::Punct);
                i += 1;
            } else {
                let len = code[i..].chars().next().map_or(1, |ch| ch.len_utf8());
                push(&mut out, i, i + len, TokenKind::Plain);
                i += len;
            }
        }
        out
    }
}
// ---- highlight:end -----------------------------------------------------------------

// =============================================================================
// Palette
// =============================================================================

#[derive(Clone, Copy)]
struct Palette {
    bg: Color32,
    header_bg: Color32,
    border: Color32,
    text: Color32,
    muted: Color32,
    comment: Color32,
    keyword: Color32,
    string: Color32,
    number: Color32,
    ty: Color32,
    function: Color32,
    macro_: Color32,
    punct: Color32,
}

const DARK: Palette = Palette {
    bg: Color32::from_rgb(0x1c, 0x1d, 0x22),
    header_bg: Color32::from_rgb(0x26, 0x27, 0x2d),
    border: Color32::from_rgb(0x3a, 0x3b, 0x42),
    text: Color32::from_rgb(0xd6, 0xde, 0xeb),
    muted: Color32::from_rgb(0x8a, 0x8f, 0x98),
    comment: Color32::from_rgb(0x7a, 0x84, 0x90),
    keyword: Color32::from_rgb(0xc7, 0x92, 0xea),
    string: Color32::from_rgb(0xc3, 0xe8, 0x8d),
    number: Color32::from_rgb(0xf7, 0x8c, 0x6c),
    ty: Color32::from_rgb(0xff, 0xcb, 0x6b),
    function: Color32::from_rgb(0x82, 0xaa, 0xff),
    macro_: Color32::from_rgb(0x89, 0xdd, 0xff),
    punct: Color32::from_rgb(0x9d, 0xa9, 0xbf),
};

const LIGHT: Palette = Palette {
    bg: Color32::from_rgb(0xf6, 0xf8, 0xfa),
    header_bg: Color32::from_rgb(0xea, 0xee, 0xf2),
    border: Color32::from_rgb(0xd0, 0xd7, 0xde),
    text: Color32::from_rgb(0x1f, 0x23, 0x28),
    muted: Color32::from_rgb(0x65, 0x6d, 0x76),
    comment: Color32::from_rgb(0x6e, 0x77, 0x81),
    keyword: Color32::from_rgb(0xcf, 0x22, 0x2e),
    string: Color32::from_rgb(0x0a, 0x30, 0x69),
    number: Color32::from_rgb(0x05, 0x50, 0xae),
    ty: Color32::from_rgb(0x95, 0x38, 0x00),
    function: Color32::from_rgb(0x82, 0x50, 0xdf),
    macro_: Color32::from_rgb(0x11, 0x63, 0x29),
    punct: Color32::from_rgb(0x57, 0x60, 0x6a),
};

impl Palette {
    fn color(&self, kind: TokenKind) -> Color32 {
        match kind {
            TokenKind::Plain => self.text,
            TokenKind::Comment => self.comment,
            TokenKind::String => self.string,
            TokenKind::Number => self.number,
            TokenKind::Keyword => self.keyword,
            TokenKind::Type => self.ty,
            TokenKind::Function => self.function,
            TokenKind::Macro => self.macro_,
            TokenKind::Punct => self.punct,
        }
    }
}

/// Builds the coloured, non-wrapping text for `code`.
fn build_job(code: &str, lang: &str, pal: &Palette) -> LayoutJob {
    let mut job = LayoutJob::default();
    job.wrap.max_width = f32::INFINITY;
    for (s, e, kind) in highlight::tokenize(code, lang) {
        job.append(
            &code[s..e],
            0.0,
            TextFormat {
                font_id: FontId::monospace(FONT_SIZE),
                color: pal.color(kind),
                italics: kind == TokenKind::Comment,
                ..Default::default()
            },
        );
    }
    job
}

// =============================================================================
// Private helpers
// =============================================================================

/// `true` when `m` has no padding on any side.
#[inline]
fn margin_is_zero(m: Margin) -> bool {
    m.left == 0 && m.right == 0 && m.top == 0 && m.bottom == 0
}

/// `true` when every corner of `r` is zero.
#[inline]
fn radius_is_zero(r: CornerRadius) -> bool {
    r.nw == 0 && r.ne == 0 && r.sw == 0 && r.se == 0
}

/// Tiny deterministic FNV-1a hasher used for the style cache.
struct Fnv1a(u64);

impl Fnv1a {
    const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;

    #[inline]
    fn new() -> Self {
        Self(Self::OFFSET)
    }
}

impl Hasher for Fnv1a {
    #[inline]
    fn finish(&self) -> u64 {
        self.0
    }
    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.0 ^= u64::from(b);
            self.0 = self.0.wrapping_mul(Self::PRIME);
        }
    }
}

/// Fingerprints the handful of egui visuals that realistically change when a
/// theme is switched.
fn visuals_fingerprint(v: &egui::Visuals) -> u64 {
    let mut h = Fnv1a::new();
    v.dark_mode.hash(&mut h);
    v.panel_fill.to_array().hash(&mut h);
    v.extreme_bg_color.to_array().hash(&mut h);
    v.selection.bg_fill.to_array().hash(&mut h);
    v.hyperlink_color.to_array().hash(&mut h);
    h.finish()
}

// =============================================================================
// Component
// =============================================================================

/// A highlighted, copyable code block. Give it text; it does the rest.
///
/// Supports Bootstrap / Tailwind utility classes on [`Self::base`] exactly like
/// `MudPaper` and `MudContainer`.
#[must_use = "MudCodeBlock does nothing until you call `.show(ui)`"]
pub struct MudCodeBlock {
    // ---- content ----
    pub code: String,
    /// Language name (`rust`, `js`, `python`, `cs`, `json`, `toml`, `sh`, ...). `None` guesses.
    pub language: Option<String>,
    /// Shows the copy button and language label header. Default `true`.
    pub header: bool,
    /// Shows a line-number gutter. Default `false`.
    pub line_numbers: bool,
    /// Caps the height; the body scrolls vertically beyond it. Default none.
    pub max_height: Option<f32>,
    /// Fixed body height: always exactly this tall (scrolls if the code is taller, leaves
    /// empty space if shorter). Wins over `max_height`. Default none.
    pub height: Option<f32>,
    /// Disambiguates the scroll area / copy state of two blocks with equal text.
    pub id_salt: Option<Id>,

    // ---- styling (mirrors MudPaper / MudContainer) ----
    /// Shared component state (utility classes, elevation). Like `MudPaper::base`.
    pub base: ComponentBase,
    /// Which utility-class system to resolve [`Self::base`] against.
    pub flavor: StyleFlavor,
    /// Bootstrap resolver cache.
    pub bs_style: BsStyle,
    /// Tailwind resolver cache.
    pub tw_style: TwStyle,
    /// Tailwind interaction state fed to the resolver (from the previous frame).
    pub tw_state: TwState,
    /// Explicit background, beats `bg-*` utilities.
    pub fill: Option<Color32>,
    /// Explicit border stroke, beats `border-*` utilities.
    pub stroke: Option<Stroke>,
    /// Explicit corner radius, beats `rounded-*` utilities.
    pub corner_radius: Option<CornerRadius>,
    /// Opt-in: memoize the class-resolved frame across frames.
    pub memoize_style: bool,
    /// Arbitrary user data (like `MudPaper::tag`).
    pub tag: Option<Box<dyn Any + Send + Sync>>,

    // ---- internal ----
    frame_cache: Option<(u64, Frame)>,
    cache_epoch: u64,
}

impl Default for MudCodeBlock {
    fn default() -> Self {
        Self {
            code: String::new(),
            language: None,
            header: true,
            line_numbers: false,
            max_height: None,
            height: None,
            id_salt: None,
            base: ComponentBase::new(),
            flavor: StyleFlavor::Bootstrap,
            bs_style: BsStyle::default(),
            tw_style: TwStyle::default(),
            tw_state: TwState::default(),
            fill: None,
            stroke: None,
            corner_radius: None,
            memoize_style: false,
            tag: None,
            frame_cache: None,
            cache_epoch: 0,
        }
    }
}

impl MudCodeBlock {
    pub fn new(code: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            ..Default::default()
        }
    }

    // ---- content builders (unchanged) ----

    pub fn language(mut self, language: impl Into<String>) -> Self {
        self.language = Some(language.into());
        self
    }

    pub fn header(mut self, header: bool) -> Self {
        self.header = header;
        self
    }

    pub fn line_numbers(mut self, on: bool) -> Self {
        self.line_numbers = on;
        self
    }

    pub fn max_height(mut self, h: f32) -> Self {
        self.max_height = Some(h);
        self
    }

    /// Fixed body height in points (always this tall). Use [`Self::max_height`] to only cap it.
    pub fn height(mut self, h: f32) -> Self {
        self.height = Some(h.max(0.0));
        self
    }

    pub fn id_salt(mut self, salt: impl std::hash::Hash + std::fmt::Debug) -> Self {
        self.id_salt = Some(Id::new(salt));
        self
    }

    // ---- styling builders (new, mirror MudPaper / MudContainer) ----

    /// Selects the utility-class resolver. Defaults to [`StyleFlavor::Bootstrap`].
    ///
    /// Switching to [`StyleFlavor::Tailwind`] enables `hover:`, `dark:` and
    /// `active:` classes to be resolved.
    #[inline]
    pub fn flavor(mut self, flavor: StyleFlavor) -> Self {
        self.flavor = flavor;
        self
    }

    /// Adds utility classes (e.g. `["bg-surface", "p-4", "rounded-lg", "shadow-md"]`).
    /// Mirrors `MudPaper::classes` / `MudContainer::classes`.
    #[inline]
    pub fn classes<S: AsRef<str>>(mut self, classes: impl IntoIterator<Item = S>) -> Self {
        self.base = self.base.classes(classes);
        self
    }

    /// Adds one space-separated class string, exactly like `Class="a b c"`.
    #[inline]
    pub fn class(self, classes: impl AsRef<str>) -> Self {
        self.classes([classes])
    }

    /// Convenience setter for elevation (0..=24). Mirrors `MudContainer::elevation`.
    /// Backed by `ComponentBase::set_elevation`, so `elevation-*` utilities and
    /// the automatic elevation-shadow path both pick it up.
    #[inline]
    pub fn elevation(mut self, level: u8) -> Self {
        self.base.set_elevation(level.min(MAX_ELEVATION));
        self
    }

    /// Explicit background; overrides any `bg-*` class.
    #[inline]
    pub fn fill(mut self, fill: Color32) -> Self {
        self.fill = Some(fill);
        self
    }

    /// Explicit border stroke; overrides any `border-*` class.
    #[inline]
    pub fn stroke(mut self, stroke: Stroke) -> Self {
        self.stroke = Some(stroke);
        self
    }

    /// Explicit corner radius; overrides any `rounded-*` class.
    #[inline]
    pub fn corner_radius(mut self, r: CornerRadius) -> Self {
        self.corner_radius = Some(r);
        self
    }

    /// Opts in to memoizing the class-resolved frame across frames.
    ///
    /// The cache key covers: flavour, classes, elevation, ripple flag, dark
    /// mode, a fingerprint of key egui visuals, the explicit
    /// `fill`/`stroke`/`corner_radius` overrides, and, for Tailwind, the
    /// hover/active/dark variant bits. It does **not** observe `MudTheme`
    /// values that live outside egui's visuals; if you mutate the theme
    /// palette independently, call [`Self::invalidate_style_cache`].
    #[inline]
    pub fn memoize_style(mut self, enabled: bool) -> Self {
        self.memoize_style = enabled;
        self
    }

    /// Attaches arbitrary user data (like `MudPaper::tag`).
    #[inline]
    pub fn tag<T: Any + Send + Sync>(mut self, tag: T) -> Self {
        self.tag = Some(Box::new(tag));
        self
    }

    /// Reads user data attached with [`Self::tag`].
    #[inline]
    pub fn tag_ref<T: Any + Send + Sync>(&self) -> Option<&T> {
        self.tag.as_ref().and_then(|b| b.downcast_ref::<T>())
    }

    /// Discards the memoized frame so the next frame re-resolves against the
    /// current theme. Mirrors `MudContainer::invalidate_style_cache`.
    pub fn invalidate_style_cache(&mut self) {
        self.frame_cache = None;
        self.cache_epoch = self.cache_epoch.wrapping_add(1);
    }

    // ---- frame resolution ----

    /// Resolves the active flavour's classes into an `egui::Frame`, filling in
    /// code-block defaults only where the classes left fields unset, then
    /// applying explicit overrides. Uses the memoized copy when
    /// [`Self::memoize_style`] is on and the key matches.
    fn resolve_frame(&mut self, ui: &Ui, pal: &Palette) -> Frame {
        if !self.memoize_style {
            return self.resolve_frame_uncached(ui, pal);
        }
        let key = self.style_key(ui.visuals().dark_mode, visuals_fingerprint(ui.visuals()));
        if let Some((cached_key, cached_frame)) = &self.frame_cache {
            if *cached_key == key {
                return cached_frame.clone();
            }
        }
        let frame = self.resolve_frame_uncached(ui, pal);
        self.frame_cache = Some((key, frame.clone()));
        frame
    }

    /// Resolves the frame from scratch.
    fn resolve_frame_uncached(&mut self, ui: &Ui, pal: &Palette) -> Frame {
        let theme = MudTheme::fetch(ui.ctx()).unwrap_or_default();
        let palette = theme.current_palette();
        let lookup = |key: &str| palette.resolve(key).map(Color32::from);

        let mut frame = match self.flavor {
            StyleFlavor::Bootstrap => self.bs_style.build_frame(&self.base, &lookup),
            StyleFlavor::Tailwind => self
                .tw_style
                .build_frame(&self.base, self.tw_state, REM_SIZE_PT, &lookup),
        };

        // Fill in code-block defaults ONLY where the classes left the field
        // untouched, so `bg-surface rounded-md shadow-md` etc. win cleanly.
        if frame.fill == Color32::TRANSPARENT {
            frame = frame.fill(pal.bg);
        }
        if frame.stroke == Stroke::NONE {
            frame = frame.stroke(Stroke::new(1.0, pal.border));
        }
        if radius_is_zero(frame.corner_radius) {
            frame = frame.corner_radius(CornerRadius::same(DEFAULT_RADIUS));
        }

        // Explicit overrides beat both classes and defaults.
        if let Some(fill) = self.fill {
            frame = frame.fill(fill);
        }
        if let Some(stroke) = self.stroke {
            frame = frame.stroke(stroke);
        }
        if let Some(r) = self.corner_radius {
            frame = frame.corner_radius(r);
        }

        frame
    }

    /// Computes the memoization key for the class-resolved frame.
    ///
    /// Bootstrap has no variants, so interaction state is deliberately
    /// excluded; Tailwind includes the hover/active/dark bits it consumes.
    fn style_key(&self, dark: bool, visuals_fp: u64) -> u64 {
        let mut h = Fnv1a::new();
        (self.flavor as u8).hash(&mut h);
        dark.hash(&mut h);
        visuals_fp.hash(&mut h);
        self.cache_epoch.hash(&mut h);
        self.base.elevation_value().hash(&mut h);
        self.base.ripple_enabled().hash(&mut h);
        if self.flavor == StyleFlavor::Tailwind {
            self.tw_state.hovered.hash(&mut h);
            self.tw_state.active.hash(&mut h);
            self.tw_state.dark.hash(&mut h);
        }
        for class in self.base.class_names().iter() {
            class.hash(&mut h);
        }
        self.fill.map(|c| c.to_array()).hash(&mut h);
        self.stroke
            .map(|s| (s.width.to_bits(), s.color.to_array()))
            .hash(&mut h);
        self.corner_radius
            .map(|r| (r.nw, r.ne, r.sw, r.se))
            .hash(&mut h);
        h.finish()
    }

    // ---- rendering ----

    /// Draws the block and returns the response of its outer frame.
    ///
    /// Takes `&mut self` (like `MudPaper::show` / `MudContainer::show`) so the
    /// Tailwind hover/active state and the frame memoization cache can persist
    /// across frames when the block is stored on a struct field.
    pub fn show(&mut self, ui: &mut Ui) -> egui::Response {
        let pal = if ui.visuals().dark_mode { DARK } else { LIGHT };
        self.tw_state.dark = ui.visuals().dark_mode;

        // Snapshot content fields so the render closure doesn't touch `self`.
        let code = self.code.trim_end_matches(['\n', '\r']).to_owned();
        let header = self.header;
        let line_numbers = self.line_numbers;
        let max_height = self.max_height;
        let height = self.height;
        let lang = self
            .language
            .clone()
            .unwrap_or_else(|| highlight::guess_language(&code).to_owned());
        let label = self.language.clone().unwrap_or_else(|| lang.clone());

        let id = ui.id().with(self.id_salt.unwrap_or_else(|| Id::new(&code)));
        let copied_id = id.with("copied_at");

        // --- resolve the outer frame from classes + defaults + overrides ----
        let outer_frame = self.resolve_frame(ui, &pal);

        // Extract any class-supplied padding for the *content* frame, so `p-4`
        // pads the code body rather than the header band. Explicit `p-0` from
        // classes still gives the body its default padding, which matches how
        // users expect a "no padding" code block to look.
        let content_padding = if margin_is_zero(outer_frame.inner_margin) {
            DEFAULT_CONTENT_PADDING
        } else {
            outer_frame.inner_margin
        };
        let outer_radius = outer_frame.corner_radius;

        // Outer frame has no inner margin of its own: header + content supply
        // their own padding.
        let outer_frame = Frame {
            inner_margin: Margin::ZERO,
            ..outer_frame
        };

        // --- draw ---------------------------------------------------------
        let response = outer_frame
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.spacing_mut().item_spacing.y = 0.0;

                if header {
                    Frame::new()
                        .fill(pal.header_bg)
                        .corner_radius(CornerRadius {
                            nw: outer_radius.nw,
                            ne: outer_radius.ne,
                            sw: 0,
                            se: 0,
                        })
                        .inner_margin(DEFAULT_HEADER_PADDING)
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(label.clone())
                                        .small()
                                        .color(pal.muted)
                                        .monospace(),
                                );
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    let now = ui.input(|i| i.time);
                                    let at: Option<f64> =
                                        ui.ctx().data(|d| d.get_temp(copied_id));
                                    let showing = at.is_some_and(|t| now - t < COPIED_FOR);
                                    let text = if showing { "Copied" } else { "Copy" };
                                    let resp = ui.add(
                                        egui::Button::new(
                                            RichText::new(text).small().color(pal.muted),
                                        )
                                            .frame(false),
                                    );
                                    if resp.clicked() {
                                        ui.ctx().copy_text(code.clone());
                                        ui.ctx()
                                            .data_mut(|d| d.insert_temp(copied_id, now));
                                        ui.ctx().request_repaint_after(
                                            std::time::Duration::from_secs_f64(COPIED_FOR),
                                        );
                                    } else if showing {
                                        ui.ctx().request_repaint_after(
                                            std::time::Duration::from_millis(200),
                                        );
                                    }
                                });
                            });
                        });
                }

                Frame::new()
                    .inner_margin(content_padding)
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        let mut scroll = ScrollArea::both()
                            .id_salt(id.with("scroll"))
                            .auto_shrink([false, true]);
                        if let Some(h) = height {
                            // Fixed: fill exactly `h` (no shrink), scroll if taller.
                            scroll = scroll
                                .auto_shrink([false, false])
                                .max_height(h)
                                .min_scrolled_height(h);
                        } else if let Some(h) = max_height {
                            scroll = scroll.max_height(h);
                        }
                        scroll.show(ui, |ui| {
                            ui.horizontal_top(|ui| {
                                ui.spacing_mut().item_spacing.x = 14.0;
                                if line_numbers {
                                    let count = code.lines().count().max(1);
                                    let width = count.to_string().len();
                                    let numbers: String = (1..=count)
                                        .map(|n| format!("{n:>width$}"))
                                        .collect::<Vec<_>>()
                                        .join("\n");
                                    ui.add(
                                        Label::new(
                                            RichText::new(numbers)
                                                .monospace()
                                                .size(FONT_SIZE)
                                                .color(pal.muted),
                                        )
                                            .selectable(false)
                                            .extend(),
                                    );
                                }
                                ui.add(
                                    Label::new(build_job(&code, &lang, &pal))
                                        .selectable(true)
                                        .extend(),
                                );
                            });
                        });
                    });
            })
            .response;

        // Persist interaction flags for next frame's Tailwind variants.
        self.tw_state.hovered = response.hovered();
        self.tw_state.active = response.is_pointer_button_down_on();

        response
    }
}

impl egui::Widget for MudCodeBlock {
    /// Draws the block. Prefer storing the block on a field and calling
    /// [`MudCodeBlock::show`] when you want caching or Tailwind state to persist.
    fn ui(mut self, ui: &mut Ui) -> egui::Response {
        self.show(ui)
    }
}

#[cfg(test)]
mod tests {
    use super::highlight::{guess_language, is_known_language, tokenize};
    use super::TokenKind as K;

    fn kinds(code: &str, lang: &str) -> Vec<(String, super::TokenKind)> {
        tokenize(code, lang)
            .into_iter()
            .map(|(s, e, k)| (code[s..e].to_string(), k))
            .collect()
    }

    fn rebuilt(code: &str, lang: &str) -> String {
        tokenize(code, lang).into_iter().map(|(s, e, _)| &code[s..e]).collect()
    }

    #[test]
    fn ranges_are_contiguous_and_lossless() {
        for lang in ["rust", "js", "python", "cs", "json", "toml", "sh", "text"] {
            for code in [
                "fn main() { let x = 1.5; // hi\n}",
                "naïve ünï 日本語 \"str\" 'a",
                "/* open",
                "\"unterminated",
                "",
                "a\\",
                "'\\",
            ] {
                assert_eq!(rebuilt(code, lang), code, "lang {lang} code {code:?}");
            }
        }
    }

    #[test]
    fn rust_basics() {
        let k = kinds("let x = Foo::new(1);", "rust");
        assert!(k.contains(&("let".into(), K::Keyword)));
        assert!(k.contains(&("Foo".into(), K::Type)));
        assert!(k.contains(&("new".into(), K::Function)));
        assert!(k.contains(&("1".into(), K::Number)));
    }

    #[test]
    fn rust_macros_comments_and_strings() {
        let k = kinds("println!(\"a // b\"); // end", "rust");
        assert!(k.contains(&("println!".into(), K::Macro)));
        assert!(k.contains(&("\"a // b\"".into(), K::String)));
        assert!(k.contains(&("// end".into(), K::Comment)));
    }

    #[test]
    fn rust_lifetimes_are_not_strings() {
        let k = kinds("fn f<'a>(x: &'a str) {}", "rust");
        assert!(!k.iter().any(|(_, kind)| *kind == K::String));
        let c = kinds("let c = 'x';", "rust");
        assert!(c.contains(&("'x'".into(), K::String)));
    }

    #[test]
    fn not_equal_is_not_a_macro() {
        let k = kinds("a != b", "rust");
        assert!(!k.iter().any(|(_, kind)| *kind == K::Macro));
    }

    #[test]
    fn block_comment_spans_lines() {
        let k = kinds("/* a\nb */ x", "rust");
        assert_eq!(k[0], ("/* a\nb */".into(), K::Comment));
    }

    #[test]
    fn python_hash_comment_and_keywords() {
        let k = kinds("def f(): # note\n    return None", "python");
        assert!(k.contains(&("def".into(), K::Keyword)));
        assert!(k.contains(&("# note".into(), K::Comment)));
        assert!(k.contains(&("None".into(), K::Keyword)));
    }

    #[test]
    fn json_values() {
        let k = kinds("{\"a\": [1, true, null]}", "json");
        assert!(k.contains(&("\"a\"".into(), K::String)));
        assert!(k.contains(&("true".into(), K::Keyword)));
        assert!(k.contains(&("null".into(), K::Keyword)));
    }

    #[test]
    fn escapes_do_not_end_strings() {
        let k = kinds(r#""a\"b" c"#, "js");
        assert_eq!(k[0], (r#""a\"b""#.into(), K::String));
    }

    #[test]
    fn language_lookup_and_guess() {
        assert!(is_known_language("RS"));
        assert!(!is_known_language("brainfuck"));
        assert_eq!(guess_language("{\"a\":1}"), "json");
        assert_eq!(guess_language("fn main() {}"), "rust");
        assert_eq!(guess_language("hello"), "text");
    }

    // ---- styling builder / defaults tests ----

    use super::*;

    #[test]
    fn defaults_are_code_block_like() {
        let c = MudCodeBlock::new("x");
        assert!(matches!(c.flavor, StyleFlavor::Bootstrap));
        assert!(c.base.class_names().is_empty());
        assert!(c.fill.is_none() && c.stroke.is_none() && c.corner_radius.is_none());
        assert!(!c.memoize_style);
        assert!(c.tag.is_none());
        assert!(c.header);
        assert!(!c.line_numbers);
    }

    #[test]
    fn styling_builders_set_fields() {
        let c = MudCodeBlock::new("x")
            .flavor(StyleFlavor::Tailwind)
            .classes(["bg-surface", "p-4"])
            .elevation(4)
            .fill(Color32::BLACK)
            .stroke(Stroke::new(2.0, Color32::RED))
            .corner_radius(CornerRadius::same(12))
            .memoize_style(true)
            .tag("code");

        assert!(matches!(c.flavor, StyleFlavor::Tailwind));
        assert_eq!(c.base.class_names(), ["bg-surface", "p-4"]);
        assert_eq!(c.base.elevation_value(), 4);
        assert_eq!(c.fill, Some(Color32::BLACK));
        assert_eq!(c.stroke.map(|s| s.width), Some(2.0));
        assert_eq!(c.corner_radius, Some(CornerRadius::same(12)));
        assert!(c.memoize_style);
        assert_eq!(c.tag_ref::<&str>(), Some(&"code"));
    }

    #[test]
    fn class_builder_accepts_space_separated_and_dedups() {
        let c = MudCodeBlock::new("x")
            .classes(["bg-surface p-4"])
            .classes(["p-4", "rounded"]);
        assert_eq!(c.base.class_names(), ["bg-surface", "p-4", "rounded"]);
    }

    #[test]
    fn elevation_clamps_to_24() {
        let c = MudCodeBlock::new("x").elevation(200);
        assert_eq!(c.base.elevation_value(), 24);
    }

    #[test]
    fn invalidate_style_cache_bumps_epoch() {
        let mut c = MudCodeBlock::new("x");
        c.invalidate_style_cache();
        assert_eq!(c.cache_epoch, 1);
        assert!(c.frame_cache.is_none());
    }

    #[test]
    fn style_key_changes_with_classes_dark_and_overrides() {
        let base = MudCodeBlock::new("x").classes(["p-4"]);
        let k = base.style_key(true, 7);
        assert_ne!(k, MudCodeBlock::new("x").classes(["p-5"]).style_key(true, 7));
        assert_ne!(k, base.style_key(false, 7));
        assert_ne!(
            k,
            MudCodeBlock::new("x")
                .classes(["p-4"])
                .fill(Color32::BLACK)
                .style_key(true, 7)
        );
        assert_ne!(
            k,
            MudCodeBlock::new("x")
                .classes(["p-4"])
                .corner_radius(CornerRadius::same(4))
                .style_key(true, 7)
        );
    }

    #[test]
    fn tag_round_trips_by_type() {
        let c = MudCodeBlock::new("x").tag(42_u32);
        assert_eq!(c.tag_ref::<u32>(), Some(&42));
        assert!(c.tag_ref::<String>().is_none());
    }

    #[test]
    fn margin_and_radius_helpers() {
        assert!(margin_is_zero(Margin::ZERO));
        assert!(!margin_is_zero(Margin::same(1)));
        assert!(radius_is_zero(CornerRadius::same(0)));
        assert!(!radius_is_zero(CornerRadius::same(4)));
    }
}