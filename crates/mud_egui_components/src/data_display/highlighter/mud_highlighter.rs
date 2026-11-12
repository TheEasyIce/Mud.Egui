//! `MudHighlighter` — renders `Text` with one or more `HighlightedText`
//! fragments visually emphasised.
//!
//! Port of `MudHighlighter.razor.cs` (MudBlazor 8.13.0).
//!
//! # Parity
//!
//! | MudBlazor parameter | Rust method | Default |
//! |---------------------|-------------|---------|
//! | `Text`              | [`MudHighlighter::text`] | `None` |
//! | `HighlightedText`   | [`MudHighlighter::highlighted_text`] | `None` |
//! | `HighlightedTexts`  | [`MudHighlighter::highlighted_texts`] | `None` |
//! | `CaseSensitive`     | [`MudHighlighter::case_sensitive`] | `false` |
//! | `UntilNextBoundary` | [`MudHighlighter::until_next_boundary`] | `false` |
//! | `Markup`            | **not ported** (no HTML in egui) | — |
//! | `Class`             | [`MudHighlighter::classes`] | `None` |
//! | `Style`             | *(egui has no CSS strings; use `classes`)* | — |
//! | `Tag`               | [`MudHighlighter::tag`] / [`MudHighlighter::tag_ref`] | `None` |
//!
//! # Text rendering
//!
//! Every fragment is drawn through [`MudText`], so the font family, size,
//! typography scale, and theme colour of the surrounding UI are respected.
//! Highlighted fragments receive a coloured background via a tiny
//! [`egui::Frame`] with no padding, matching the browser’s `<mark>` rendering
//! while staying theme-consistent.
//!
//! # Matching
//!
//! * Empty needles are ignored.
//! * Overlapping matches are merged so a fragment is never painted twice.
//! * `until_next_boundary` extends each match to the next `\w → \W` transition,
//!   matching MudBlazor’s regex-boundary behaviour.
//! * Case-insensitive matching uses Unicode-aware character comparison, not
//!   byte offsets, so multibyte text is safe.

use egui::{vec2, Color32, CornerRadius, Frame, Margin, Response, Sense, Ui};
use mud_egui_bootstrap::bs_style::BsStyle;
use mud_egui_core::base::component_base::ComponentBase;
use mud_egui_core::enums::{Color, StyleFlavor};
use mud_egui_tailwind::tw_style::{TwState, TwStyle};
use mud_egui_theming::theme::MudTheme;

use crate::typography::mud_text::MudText;

/// Base rem size passed to the Tailwind resolver.
const REM_SIZE_PT: f32 = 14.0;

/// Default highlight background: a semi-transparent Material yellow that works
/// in both light and dark themes.
pub const DEFAULT_HIGHLIGHT_COLOR: Color32 =
    Color32::from_rgba_premultiplied(255, 235, 59, 120);

/// One contiguous run of text, either plain or highlighted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Segment {
    /// The fragment text.
    pub text: String,
    /// Whether this fragment should be emphasised.
    pub highlighted: bool,
}

/// Splits `text` into plain and highlighted segments from the given byte
/// ranges. Ranges must be sorted, non-overlapping, and on `char` boundaries
/// (guaranteed by [`find_ranges`]).
pub fn split_segments(text: &str, ranges: &[std::ops::Range<usize>]) -> Vec<Segment> {
    if ranges.is_empty() {
        return vec![Segment {
            text: text.to_owned(),
            highlighted: false,
        }];
    }

    let mut out = Vec::with_capacity(ranges.len() * 2 + 1);
    let mut cursor = 0usize;

    for r in ranges {
        let start = r.start.min(text.len());
        let end = r.end.min(text.len());
        if start > cursor {
            out.push(Segment {
                text: text[cursor..start].to_owned(),
                highlighted: false,
            });
        }
        if end > start {
            out.push(Segment {
                text: text[start..end].to_owned(),
                highlighted: true,
            });
        }
        cursor = cursor.max(end);
    }

    if cursor < text.len() {
        out.push(Segment {
            text: text[cursor..].to_owned(),
            highlighted: false,
        });
    }

    out
}

/// Whether `ch` counts as a “word character” for boundary extension.
#[inline]
fn is_word_char(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_'
}

/// Case-insensitive character equality (Unicode-aware).
#[inline]
fn eq_ignore_case(a: char, b: char) -> bool {
    if a == b {
        return true;
    }
    a.to_lowercase().eq(b.to_lowercase())
}

/// Returns every match of `needles` inside `text`, optionally extended to the
/// next word boundary and optionally case-sensitive.
///
/// The result is sorted by start offset and contains no overlapping ranges.
pub fn find_ranges(
    text: &str,
    needles: &[String],
    case_sensitive: bool,
    until_next_boundary: bool,
) -> Vec<std::ops::Range<usize>> {
    if text.is_empty() || needles.is_empty() {
        return Vec::new();
    }

    // Collect raw matches.
    let mut raw: Vec<std::ops::Range<usize>> = Vec::new();
    for needle in needles {
        if needle.is_empty() {
            continue;
        }
        let needle_chars: Vec<char> = needle.chars().collect();
        if needle_chars.len() > text.chars().count() {
            continue;
        }

        let hay: Vec<(usize, char)> = text.char_indices().collect();
        if let Some(last_start) = hay.len().checked_sub(needle_chars.len()) {
            for i in 0..=last_start {
                let mut ok = true;
                for (j, &nc) in needle_chars.iter().enumerate() {
                    let hc = hay[i + j].1;
                    let matched = if case_sensitive {
                        hc == nc
                    } else {
                        eq_ignore_case(hc, nc)
                    };
                    if !matched {
                        ok = false;
                        break;
                    }
                }
                if ok {
                    let start = hay[i].0;
                    let end_idx = i + needle_chars.len();
                    let mut end = if end_idx < hay.len() {
                        hay[end_idx].0
                    } else {
                        text.len()
                    };

                    if until_next_boundary {
                        // Extend while the next character is still a word char.
                        while end < text.len() {
                            let ch = text[end..].chars().next().unwrap_or('\0');
                            if !is_word_char(ch) {
                                break;
                            }
                            end += ch.len_utf8();
                        }
                    }

                    raw.push(start..end);
                }
            }
        }
    }

    // Sort and merge overlapping ranges.
    raw.sort_by_key(|r| (r.start, r.end));
    let mut merged: Vec<std::ops::Range<usize>> = Vec::with_capacity(raw.len());
    for r in raw {
        if let Some(last) = merged.last_mut() {
            if r.start <= last.end {
                last.end = last.end.max(r.end);
                continue;
            }
        }
        merged.push(r);
    }
    merged
}

/// Renders `Text` with one or more highlighted fragments.
///
/// # Example
///
/// ```ignore
/// MudHighlighter::new("The quick brown fox")
///     .highlighted_text("quick")
///     .show(ui);
/// ```
#[must_use = "MudHighlighter does nothing until you call `.show(ui)`"]
pub struct MudHighlighter {
    /// The full text.
    pub text: String,
    /// Fragments to emphasise. Populated by [`highlighted_text`](Self::highlighted_text)
    /// and [`highlighted_texts`](Self::highlighted_texts).
    pub highlighted_texts: Vec<String>,
    /// Whether matching is case-sensitive.
    pub case_sensitive: bool,
    /// Whether each match extends to the next word boundary.
    pub until_next_boundary: bool,
    /// Background colour for highlighted fragments.
    pub highlight_color: Color32,

    /// Utility classes for the root frame.
    pub base: ComponentBase,
    /// Bootstrap or Tailwind resolver.
    pub flavor: StyleFlavor,
    /// Bootstrap resolver state.
    pub bs_style: BsStyle,
    /// Tailwind resolver state.
    pub tw_style: TwStyle,
    /// Tailwind interaction state.
    pub tw_state: TwState,

    tag_value: Option<std::sync::Arc<dyn std::any::Any + Send + Sync>>,
}

impl Default for MudHighlighter {
    fn default() -> Self {
        Self {
            text: String::new(),
            highlighted_texts: Vec::new(),
            case_sensitive: false,
            until_next_boundary: false,
            highlight_color: DEFAULT_HIGHLIGHT_COLOR,
            base: ComponentBase::new(),
            flavor: StyleFlavor::default(),
            bs_style: BsStyle::default(),
            tw_style: TwStyle::default(),
            tw_state: TwState::default(),
            tag_value: None,
        }
    }
}

impl MudHighlighter {
    /// Creates a highlighter for `text` with no fragments highlighted.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            ..Default::default()
        }
    }

    /// Sets the full text (replaces any previous value).
    #[inline]
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self
    }

    /// Adds one highlighted fragment (`HighlightedText`).
    #[inline]
    pub fn highlighted_text(mut self, fragment: impl Into<String>) -> Self {
        self.highlighted_texts.push(fragment.into());
        self
    }

    /// Adds several highlighted fragments (`HighlightedTexts`).
    pub fn highlighted_texts<I, S>(mut self, fragments: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.highlighted_texts
            .extend(fragments.into_iter().map(Into::into));
        self
    }

    /// Makes matching case-sensitive.
    #[inline]
    pub fn case_sensitive(mut self, b: bool) -> Self {
        self.case_sensitive = b;
        self
    }

    /// Extends each match to the next word boundary.
    #[inline]
    pub fn until_next_boundary(mut self, b: bool) -> Self {
        self.until_next_boundary = b;
        self
    }

    /// Sets the highlight background colour (extension; MudBlazor relies on
    /// the browser's default `<mark>` styling).
    #[inline]
    pub fn highlight_color(mut self, color: Color32) -> Self {
        self.highlight_color = color;
        self
    }

    /// Adds utility classes to the root frame.
    #[inline]
    pub fn classes<S: AsRef<str>>(mut self, classes: impl IntoIterator<Item = S>) -> Self {
        self.base = self.base.classes(classes);
        self
    }

    /// Selects the utility-class resolver.
    #[inline]
    pub fn flavor(mut self, f: StyleFlavor) -> Self {
        self.flavor = f;
        self
    }

    /// Attaches arbitrary user data (`Tag`).
    pub fn tag<T: std::any::Any + Send + Sync>(mut self, value: T) -> Self {
        self.tag_value = Some(std::sync::Arc::new(value));
        self
    }

    /// Reads the tag back if it is of type `U`.
    pub fn tag_ref<U: std::any::Any + Send + Sync>(&self) -> Option<&U> {
        self.tag_value.as_ref()?.downcast_ref::<U>()
    }

    /// Renders the highlighter and returns the root frame's [`Response`].
    pub fn show(mut self, ui: &mut Ui) -> Response {
        let theme = MudTheme::fetch(ui.ctx()).unwrap_or_default();
        let palette = theme.current_palette();

        // Resolve the root frame (same pattern as the other components).
        let lookup = |key: &str| palette.resolve(key).map(Color32::from);
        let mut tw_state = self.tw_state;
        tw_state.dark = ui.visuals().dark_mode;
        let frame = match self.flavor {
            StyleFlavor::Bootstrap => self.bs_style.build_frame(&self.base, &lookup),
            StyleFlavor::Tailwind => {
                self.tw_style
                    .build_frame(&self.base, tw_state, REM_SIZE_PT, &lookup)
            }
        };

        let ranges = find_ranges(
            &self.text,
            &self.highlighted_texts,
            self.case_sensitive,
            self.until_next_boundary,
        );
        let segments = split_segments(&self.text, &ranges);
        let highlight_bg = self.highlight_color;

        let inner = frame.show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                // Remove inter-widget gaps so segments appear contiguous,
                // matching a single inline text run.
                ui.spacing_mut().item_spacing.x = 0.0;

                for segment in segments {
                    if segment.highlighted {
                        // Use a zero-padding frame around MudText so the
                        // background hugs the glyphs without shifting layout.
                        Frame::default()
                            .fill(highlight_bg)
                            .corner_radius(CornerRadius::same(2))
                            .inner_margin(Margin::ZERO)
                            .show(ui, |ui| {
                                ui.add(MudText::new(segment.text).inline(true));
                            });
                    } else {
                        ui.add(MudText::new(segment.text).inline(true));
                    }
                }
            });
        });

        inner.response
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn needles(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| (*s).to_string()).collect()
    }

    // --- find_ranges -------------------------------------------------------

    #[test]
    fn no_needles_is_empty() {
        assert!(find_ranges("hello", &[], false, false).is_empty());
    }

    #[test]
    fn empty_needle_is_ignored() {
        assert!(find_ranges("hello", &needles(&[""]), false, false).is_empty());
    }

    #[test]
    fn simple_match() {
        let r = find_ranges("hello world", &needles(&["world"]), false, false);
        assert_eq!(r, vec![6..11]);
    }

    #[test]
    fn multiple_occurrences() {
        let r = find_ranges("a a a", &needles(&["a"]), false, false);
        assert_eq!(r, vec![0..1, 2..3, 4..5]);
    }

    #[test]
    fn case_insensitive_by_default() {
        let r = find_ranges("Hello WORLD", &needles(&["world"]), false, false);
        assert_eq!(r, vec![6..11]);
    }

    #[test]
    fn case_sensitive_respected() {
        let r = find_ranges("Hello WORLD", &needles(&["world"]), true, false);
        assert!(r.is_empty());
    }

    #[test]
    fn until_next_boundary_extends_to_word_end() {
        let r = find_ranges("foo barbazqux end", &needles(&["bar"]), false, true);
        assert_eq!(r, vec![4..13]); // "barbazqux"
    }

    #[test]
    fn until_next_boundary_stops_at_whitespace() {
        let r = find_ranges("foo bar baz", &needles(&["bar"]), false, true);
        assert_eq!(r, vec![4..7]);
    }

    #[test]
    fn until_next_boundary_stops_at_punctuation() {
        let r = find_ranges("foo bar, baz", &needles(&["bar"]), false, true);
        assert_eq!(r, vec![4..7]);
    }

    #[test]
    fn overlapping_matches_are_merged() {
        let r = find_ranges("abcabc", &needles(&["abc", "bca"]), false, false);
        // "abc" @ 0..3 and "bca" @ 1..4 overlap → merged to 0..4.
        // "abc" @ 3..6 also present.
        assert_eq!(r, vec![0..4, 3..6]);
    }

    #[test]
    fn multiple_needles_are_all_found() {
        let r = find_ranges("red green blue", &needles(&["red", "blue"]), false, false);
        assert_eq!(r, vec![0..3, 10..14]);
    }

    #[test]
    fn unicode_is_handled_by_char_boundary() {
        let r = find_ranges("héllo wörld", &needles(&["wörld"]), false, false);
        assert_eq!(r.len(), 1);
        assert_eq!(&"héllo wörld"[r[0].clone()], "wörld");
    }

    #[test]
    fn needle_longer_than_text_returns_empty() {
        assert!(find_ranges("hi", &needles(&["hello"]), false, false).is_empty());
    }

    // --- split_segments ----------------------------------------------------

    #[test]
    fn no_ranges_returns_single_plain_segment() {
        let segs = split_segments("hello", &[]);
        assert_eq!(segs.len(), 1);
        assert!(!segs[0].highlighted);
        assert_eq!(segs[0].text, "hello");
    }

    #[test]
    fn single_range_splits_into_three() {
        let segs = split_segments("hello world", &[6..11]);
        assert_eq!(segs.len(), 2); // "hello " + "world" (no leading range)
        assert_eq!(segs[0].text, "hello ");
        assert!(!segs[0].highlighted);
        assert_eq!(segs[1].text, "world");
        assert!(segs[1].highlighted);
    }

    #[test]
    fn middle_range_splits_into_three() {
        let segs = split_segments("foo bar baz", &[4..7]);
        assert_eq!(segs.len(), 3);
        assert_eq!(segs[0].text, "foo ");
        assert!(!segs[0].highlighted);
        assert_eq!(segs[1].text, "bar");
        assert!(segs[1].highlighted);
        assert_eq!(segs[2].text, " baz");
        assert!(!segs[2].highlighted);
    }

    #[test]
    fn leading_range_has_no_leading_plain_segment() {
        let segs = split_segments("bar baz", &[0..3]);
        assert_eq!(segs.len(), 2);
        assert_eq!(segs[0].text, "bar");
        assert!(segs[0].highlighted);
        assert_eq!(segs[1].text, " baz");
        assert!(!segs[1].highlighted);
    }

    #[test]
    fn trailing_range_has_no_trailing_plain_segment() {
        let segs = split_segments("foo bar", &[4..7]);
        assert_eq!(segs.len(), 2);
        assert_eq!(segs[0].text, "foo ");
        assert!(!segs[0].highlighted);
        assert_eq!(segs[1].text, "bar");
        assert!(segs[1].highlighted);
    }

    #[test]
    fn whole_text_highlighted_is_one_segment() {
        let segs = split_segments("hello", &[0..5]);
        assert_eq!(segs.len(), 1);
        assert!(segs[0].highlighted);
        assert_eq!(segs[0].text, "hello");
    }

    // --- builders ---------------------------------------------------------

    #[test]
    fn defaults_match_mudblazor() {
        let h = MudHighlighter::new("x");
        assert!(h.highlighted_texts.is_empty());
        assert!(!h.case_sensitive);
        assert!(!h.until_next_boundary);
        assert_eq!(h.highlight_color, DEFAULT_HIGHLIGHT_COLOR);
    }

    #[test]
    fn builders_set_every_field() {
        let h = MudHighlighter::new("full")
            .highlighted_text("a")
            .highlighted_texts(["b", "c"])
            .case_sensitive(true)
            .until_next_boundary(true)
            .highlight_color(Color32::RED)
            .classes(["x"])
            .flavor(StyleFlavor::Tailwind)
            .tag(7u32);

        assert_eq!(h.text, "full");
        assert_eq!(h.highlighted_texts, vec!["a", "b", "c"]);
        assert!(h.case_sensitive);
        assert!(h.until_next_boundary);
        assert_eq!(h.highlight_color, Color32::RED);
        assert_eq!(h.tag_ref::<u32>(), Some(&7));
        assert!(h.tag_ref::<&str>().is_none());
    }

    #[test]
    fn text_replaces_previous_value() {
        let h = MudHighlighter::new("old").text("new");
        assert_eq!(h.text, "new");
    }
}