//! The size a `MudItem` asks of its `MudGrid` at one breakpoint.
//!
//! Port of `MudGridSpan.cs`. Represents a whole number of tracks (1-12), a
//! proportional "star" share (e.g., `2*`), an `Auto` size (shrink to content),
//! or `Unset` (which cascades down to smaller breakpoints).
//!
//! # Conversions
//! A span can be built from integers, from strings, or via [`str::parse`], which is what
//! enables the terse `MudItem::new().md(6).lg("2*")` syntax:
//!
//! | Input | Result |
//! |-------|--------|
//! | `6`, `6u8`, `6usize`, `6i64`, ... | `Tracks(6)` (clamped to `1..=12`) |
//! | `"6"` | `Tracks(6)` (saturates at 12; `"0"` stays `Tracks(0)` until [`MudGridSpan::normalized`]) |
//! | `"*"`, `"2.5*"` | `Star(1.0)`, `Star(2.5)` |
//! | `"auto"` (any case) | `Auto` |
//! | `""` / whitespace | `Unset` |
//! | anything else | `From<&str>` → `Unset`; `parse` → `None`; `FromStr` → [`ParseSpanError`] |
//!
//! # Presets
//! The common Material grid fractions are available as constants:
//! [`MudGridSpan::FULL`], [`HALF`](MudGridSpan::HALF), [`THIRD`](MudGridSpan::THIRD),
//! [`QUARTER`](MudGridSpan::QUARTER) and [`SIXTH`](MudGridSpan::SIXTH).
//!
//! # Equality
//! `MudGridSpan` derives `PartialEq` over an `f32` weight, so `Star(NaN) != Star(NaN)`. Use
//! [`MudGridSpan::normalized`] first if you need NaN weights to compare equal (they become
//! `Star(0.0)`).
//!
//! # Example
//! ```ignore
//! use mud_grid_span::MudGridSpan;
//!
//! assert_eq!(MudGridSpan::from(6), MudGridSpan::HALF);
//! assert_eq!("2*".parse::<MudGridSpan>(), Ok(MudGridSpan::Star(2.0)));
//! assert_eq!(MudGridSpan::Unset.or(MudGridSpan::QUARTER), MudGridSpan::Tracks(3));
//! ```

use std::fmt;
use std::str::FromStr;

/// Total number of tracks in a standard grid row.
pub const TRACK_COUNT: u8 = 12;

/// Represents the sizing span configuration of a grid item at a specific breakpoint.
///
/// The type is `Copy` and 8 bytes wide, so it is passed and stored by value everywhere.
///
/// # Example
/// ```ignore
/// let half = MudGridSpan::Tracks(6);
/// let rest = MudGridSpan::Star(1.0);
/// let fit  = MudGridSpan::Auto;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum MudGridSpan {
    /// Not specified at this breakpoint; cascades to smaller breakpoints.
    #[default]
    Unset,

    /// A whole number of tracks (normalized to fit `1..=12`).
    Tracks(u8),

    /// A proportional share of the leftover space (with weight `>= 0.0`).
    Star(f32),

    /// Shrinks to fit the natural width of the content.
    Auto,
}

/// Clamps a star weight to a usable value: finite and strictly positive, otherwise `0.0`.
///
/// A broken weight (negative, NaN, infinite) degrades to "gets no share" rather than
/// poisoning the whole row's width arithmetic with NaN.
#[inline]
fn sanitize_weight(weight: f32) -> f32 {
    if weight.is_finite() && weight > 0.0 {
        weight
    } else {
        0.0
    }
}

/// Parses the numeric part of a star span (the text before `*`).
///
/// An empty prefix means a bare `*`, i.e. weight `1.0`. Negative and non-finite weights are
/// rejected (unlike [`sanitize_weight`], which silently repairs): bad *input text* is an error,
/// bad *in-memory values* are repaired.
fn parse_star_weight(weight_text: &str) -> Option<f32> {
    let weight_text = weight_text.trim();
    if weight_text.is_empty() {
        return Some(1.0);
    }
    weight_text.parse::<f32>().ok().filter(|w| w.is_finite() && *w >= 0.0)
}

impl MudGridSpan {
    /// The whole row (12 tracks).
    pub const FULL: Self = MudGridSpan::Tracks(TRACK_COUNT);
    /// Half a row (6 tracks).
    pub const HALF: Self = MudGridSpan::Tracks(TRACK_COUNT / 2);
    /// A third of a row (4 tracks).
    pub const THIRD: Self = MudGridSpan::Tracks(TRACK_COUNT / 3);
    /// A quarter of a row (3 tracks).
    pub const QUARTER: Self = MudGridSpan::Tracks(TRACK_COUNT / 4);
    /// A sixth of a row (2 tracks).
    pub const SIXTH: Self = MudGridSpan::Tracks(TRACK_COUNT / 6);

    /// Builds a `Tracks` span, clamping the count into `1..=12`.
    ///
    /// Usable in `const` contexts, unlike the `From` impls.
    ///
    /// ```ignore
    /// const SIDEBAR: MudGridSpan = MudGridSpan::fixed(3);
    /// assert_eq!(MudGridSpan::fixed(0), MudGridSpan::Tracks(1));
    /// ```
    #[inline]
    pub const fn fixed(tracks: u8) -> Self {
        let t = if tracks < 1 {
            1
        } else if tracks > TRACK_COUNT {
            TRACK_COUNT
        } else {
            tracks
        };
        MudGridSpan::Tracks(t)
    }

    /// Builds a `Star` span with the given weight (repaired later by [`normalized`](Self::normalized)).
    #[inline]
    pub fn star(weight: f32) -> Self {
        MudGridSpan::Star(weight)
    }

    /// Normalizes the span variant:
    /// - `Unset` converts to the full track count (12).
    /// - Track counts are clamped to `1..=12`.
    /// - Invalid (negative, NaN, or infinite) star weights fall back to `0.0`.
    ///
    /// The result is never `Unset`, which lets the layout math treat that variant as
    /// unreachable. Normalizing is idempotent.
    #[inline]
    pub fn normalized(self) -> Self {
        match self {
            MudGridSpan::Tracks(t) => MudGridSpan::Tracks(t.clamp(1, TRACK_COUNT)),
            MudGridSpan::Star(w) => MudGridSpan::Star(sanitize_weight(w)),
            MudGridSpan::Auto => MudGridSpan::Auto,
            MudGridSpan::Unset => MudGridSpan::Tracks(TRACK_COUNT),
        }
    }

    /// Returns `self` if it is set, otherwise `fallback`.
    ///
    /// This is the single step of the mobile-first cascade: folding it from the smallest
    /// breakpoint upward yields the effective span at every breakpoint.
    #[inline]
    pub fn or(self, fallback: Self) -> Self {
        if self.is_set() {
            self
        } else {
            fallback
        }
    }

    /// Returns `true` if the span is explicitly set (i.e., not `Unset`).
    #[inline]
    pub const fn is_set(self) -> bool {
        !matches!(self, MudGridSpan::Unset)
    }

    /// Returns `true` if the span represents a fixed track count.
    #[inline]
    pub const fn is_tracks(self) -> bool {
        matches!(self, MudGridSpan::Tracks(_))
    }

    /// Returns `true` if the span represents a proportional star weight.
    #[inline]
    pub const fn is_star(self) -> bool {
        matches!(self, MudGridSpan::Star(_))
    }

    /// Returns `true` if the span is set to auto-sizing.
    #[inline]
    pub const fn is_auto(self) -> bool {
        matches!(self, MudGridSpan::Auto)
    }

    /// Returns the track count, or `0` if this is not a `Tracks` span.
    #[inline]
    pub const fn tracks(self) -> u8 {
        if let MudGridSpan::Tracks(t) = self {
            t
        } else {
            0
        }
    }

    /// Returns the star weight, or `0.0` if this is not a `Star` span.
    #[inline]
    pub fn weight(self) -> f32 {
        if let MudGridSpan::Star(w) = self {
            w
        } else {
            0.0
        }
    }

    /// How many of the row's 12 tracks this span *reserves when packing items into rows*.
    ///
    /// `Tracks(n)` reserves `n` (clamped to `1..=12`); `Star`, `Auto` and `Unset` reserve one
    /// track each so that a row can never hold more than 12 flexible items.
    ///
    /// ```ignore
    /// assert_eq!(MudGridSpan::Tracks(99).track_need(), 12);
    /// assert_eq!(MudGridSpan::Auto.track_need(), 1);
    /// ```
    #[inline]
    pub fn track_need(self) -> u8 {
        match self {
            MudGridSpan::Tracks(t) => t.clamp(1, TRACK_COUNT),
            _ => 1,
        }
    }

    /// Parses string inputs into a `MudGridSpan`.
    ///
    /// # Supported Formats
    /// - `""` or whitespace -> `Unset`
    /// - `"auto"` (case-insensitive) -> `Auto`
    /// - `"*"` or `"2.5*"` -> `Star` (defaults to weight `1.0` if just `*`)
    /// - `"6"` -> `Tracks` (saturates at 12)
    ///
    /// # Returns
    /// `None` for malformed input, including negative, non-finite, or invalid star weights.
    pub fn parse(text: &str) -> Option<Self> {
        let trimmed = text.trim();

        // Empty string maps to Unset
        if trimmed.is_empty() {
            return Some(MudGridSpan::Unset);
        }

        // Case-insensitive check for "auto"
        if trimmed.eq_ignore_ascii_case("auto") {
            return Some(MudGridSpan::Auto);
        }

        // Star-sizing suffix ('*'), with an optional numeric weight prefix
        if let Some(weight_text) = trimmed.strip_suffix('*') {
            return parse_star_weight(weight_text).map(MudGridSpan::Star);
        }

        // Otherwise, attempt to parse as a whole number of tracks
        trimmed
            .parse::<u32>()
            .ok()
            .map(|t| MudGridSpan::Tracks(t.min(u32::from(TRACK_COUNT)) as u8))
    }
}

// =============================================================================
// Display / FromStr (round-trips with `parse`)
// =============================================================================

impl fmt::Display for MudGridSpan {
    /// Formats the span in the same syntax [`MudGridSpan::parse`] accepts, so
    /// `span.to_string().parse()` round-trips. `Unset` formats as the empty string.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MudGridSpan::Unset => Ok(()),
            MudGridSpan::Tracks(t) => write!(f, "{t}"),
            MudGridSpan::Star(w) if *w == 1.0 => f.write_str("*"),
            MudGridSpan::Star(w) => write!(f, "{w}*"),
            MudGridSpan::Auto => f.write_str("Auto"),
        }
    }
}

/// Error returned when a string cannot be successfully parsed into a valid grid span.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParseSpanError;

impl fmt::Display for ParseSpanError {
    /// Describes the accepted syntax so the message is actionable on its own.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid grid span (expected \"1\"..\"12\", \"*\", \"2*\" or \"auto\")")
    }
}

impl std::error::Error for ParseSpanError {}

impl FromStr for MudGridSpan {
    type Err = ParseSpanError;

    /// Strict parsing: malformed input is an error rather than silently `Unset`.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s).ok_or(ParseSpanError)
    }
}

// =============================================================================
// Conversions (enabling syntax like `item.md(6)` and `item.md("2*")`)
// =============================================================================

/// Generates the clamping integer conversions into `MudGridSpan`.
///
/// One macro keeps the clamping rule (`1..=12`, never panic in a UI path) in exactly one
/// place for every integer width. `i32` is included so that an un-suffixed literal such as
/// `item.md(6)` (which defaults to `i32`) compiles.
macro_rules! impl_from_integer {
    ($($t:ty),*) => {$(
        impl From<$t> for MudGridSpan {
            #[inline]
            fn from(t: $t) -> Self {
                MudGridSpan::Tracks(t.max(1).min(TRACK_COUNT as $t) as u8)
            }
        }
    )*};
}
impl_from_integer!(i8, i16, i32, i64, isize, u8, u16, u32, u64, usize);

/// Invalid strings automatically convert to `Unset`.
/// To handle parsing errors explicitly, use [`str::parse`] instead.
impl From<&str> for MudGridSpan {
    fn from(s: &str) -> Self {
        MudGridSpan::parse(s).unwrap_or(MudGridSpan::Unset)
    }
}

/// Same lenient behaviour as the `&str` conversion, for owned strings.
impl From<String> for MudGridSpan {
    fn from(s: String) -> Self {
        MudGridSpan::from(s.as_str())
    }
}

/// Same lenient behaviour as the `&str` conversion, for borrowed owned strings.
impl From<&String> for MudGridSpan {
    fn from(s: &String) -> Self {
        MudGridSpan::from(s.as_str())
    }
}

// =============================================================================
// Tests
// =============================================================================

/// Unit tests for the span type. Nothing here touches egui.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_unset() {
        assert_eq!(MudGridSpan::default(), MudGridSpan::Unset);
        assert!(!MudGridSpan::Unset.is_set());
        assert!(MudGridSpan::Auto.is_set());
        assert!(MudGridSpan::Tracks(1).is_set());
        assert!(MudGridSpan::Star(0.0).is_set());
    }

    #[test]
    fn track_count_is_twelve() {
        assert_eq!(TRACK_COUNT, 12);
    }

    #[test]
    fn presets_are_the_expected_fractions() {
        assert_eq!(MudGridSpan::FULL, MudGridSpan::Tracks(12));
        assert_eq!(MudGridSpan::HALF, MudGridSpan::Tracks(6));
        assert_eq!(MudGridSpan::THIRD, MudGridSpan::Tracks(4));
        assert_eq!(MudGridSpan::QUARTER, MudGridSpan::Tracks(3));
        assert_eq!(MudGridSpan::SIXTH, MudGridSpan::Tracks(2));
        // Every preset divides the row evenly.
        for preset in [MudGridSpan::HALF, MudGridSpan::THIRD, MudGridSpan::QUARTER, MudGridSpan::SIXTH] {
            assert_eq!(TRACK_COUNT % preset.tracks(), 0);
        }
    }

    #[test]
    fn fixed_is_const_and_clamps() {
        const SIDEBAR: MudGridSpan = MudGridSpan::fixed(3);
        assert_eq!(SIDEBAR, MudGridSpan::Tracks(3));
        assert_eq!(MudGridSpan::fixed(0), MudGridSpan::Tracks(1));
        assert_eq!(MudGridSpan::fixed(12), MudGridSpan::Tracks(12));
        assert_eq!(MudGridSpan::fixed(200), MudGridSpan::Tracks(12));
        assert_eq!(MudGridSpan::star(2.0), MudGridSpan::Star(2.0));
    }

    #[test]
    fn or_falls_back_only_when_unset() {
        assert_eq!(MudGridSpan::Unset.or(MudGridSpan::Auto), MudGridSpan::Auto);
        assert_eq!(MudGridSpan::Tracks(3).or(MudGridSpan::Auto), MudGridSpan::Tracks(3));
        assert_eq!(MudGridSpan::Auto.or(MudGridSpan::Tracks(3)), MudGridSpan::Auto);
        // A zero-weight star is still "set".
        assert_eq!(MudGridSpan::Star(0.0).or(MudGridSpan::Tracks(1)), MudGridSpan::Star(0.0));
        assert_eq!(MudGridSpan::Unset.or(MudGridSpan::Unset), MudGridSpan::Unset);
    }

    #[test]
    fn track_need_matches_packing_rules() {
        assert_eq!(MudGridSpan::Tracks(0).track_need(), 1);
        assert_eq!(MudGridSpan::Tracks(5).track_need(), 5);
        assert_eq!(MudGridSpan::Tracks(99).track_need(), 12);
        assert_eq!(MudGridSpan::Star(3.0).track_need(), 1);
        assert_eq!(MudGridSpan::Auto.track_need(), 1);
        assert_eq!(MudGridSpan::Unset.track_need(), 1);
    }

    #[test]
    fn normalization() {
        assert_eq!(MudGridSpan::Unset.normalized(), MudGridSpan::Tracks(12));
        assert_eq!(MudGridSpan::Tracks(0).normalized(), MudGridSpan::Tracks(1));
        assert_eq!(MudGridSpan::Tracks(99).normalized(), MudGridSpan::Tracks(12));
        assert_eq!(MudGridSpan::Star(-2.0).normalized(), MudGridSpan::Star(0.0));
        assert_eq!(MudGridSpan::Star(f32::NAN).normalized(), MudGridSpan::Star(0.0));
        assert_eq!(MudGridSpan::Star(f32::INFINITY).normalized(), MudGridSpan::Star(0.0));
        assert_eq!(MudGridSpan::Star(2.5).normalized(), MudGridSpan::Star(2.5));
        assert_eq!(MudGridSpan::Auto.normalized(), MudGridSpan::Auto);
    }

    #[test]
    fn normalization_edge_values() {
        assert_eq!(MudGridSpan::Star(0.0).normalized(), MudGridSpan::Star(0.0));
        assert_eq!(MudGridSpan::Star(-0.0).normalized(), MudGridSpan::Star(0.0));
        assert_eq!(MudGridSpan::Star(f32::NEG_INFINITY).normalized(), MudGridSpan::Star(0.0));
        assert_eq!(MudGridSpan::Tracks(1).normalized(), MudGridSpan::Tracks(1));
        assert_eq!(MudGridSpan::Tracks(12).normalized(), MudGridSpan::Tracks(12));
    }

    #[test]
    fn normalization_is_idempotent_and_never_unset() {
        for span in [
            MudGridSpan::Unset,
            MudGridSpan::Tracks(0),
            MudGridSpan::Tracks(200),
            MudGridSpan::Star(f32::NAN),
            MudGridSpan::Star(3.0),
            MudGridSpan::Auto,
        ] {
            let once = span.normalized();
            assert_ne!(once, MudGridSpan::Unset);
            assert_eq!(once.normalized(), once);
        }
    }

    #[test]
    fn nan_star_is_not_equal_to_itself_until_normalized() {
        let nan = MudGridSpan::Star(f32::NAN);
        assert_ne!(nan, nan);
        assert_eq!(nan.normalized(), nan.normalized());
    }

    #[test]
    fn parsing() {
        assert_eq!(MudGridSpan::parse(""), Some(MudGridSpan::Unset));
        assert_eq!(MudGridSpan::parse("  "), Some(MudGridSpan::Unset));
        assert_eq!(MudGridSpan::parse("6"), Some(MudGridSpan::Tracks(6)));
        assert_eq!(MudGridSpan::parse("300"), Some(MudGridSpan::Tracks(12)));
        assert_eq!(MudGridSpan::parse("*"), Some(MudGridSpan::Star(1.0)));
        assert_eq!(MudGridSpan::parse("2.5*"), Some(MudGridSpan::Star(2.5)));
        assert_eq!(MudGridSpan::parse(" 3 * "), Some(MudGridSpan::Star(3.0)));
        assert_eq!(MudGridSpan::parse("AUTO"), Some(MudGridSpan::Auto));
    }

    #[test]
    fn parsing_details() {
        // Whitespace and case around `auto`
        assert_eq!(MudGridSpan::parse("  Auto  "), Some(MudGridSpan::Auto));
        assert_eq!(MudGridSpan::parse("aUtO"), Some(MudGridSpan::Auto));
        // "0" is accepted verbatim (normalization raises it to 1 later)
        assert_eq!(MudGridSpan::parse("0"), Some(MudGridSpan::Tracks(0)));
        assert_eq!(MudGridSpan::parse("0").unwrap().normalized(), MudGridSpan::Tracks(1));
        // Explicit plus sign and exponent notation are accepted by Rust's number parsers
        assert_eq!(MudGridSpan::parse("+5"), Some(MudGridSpan::Tracks(5)));
        assert_eq!(MudGridSpan::parse("1e2*"), Some(MudGridSpan::Star(100.0)));
        // A zero star weight is valid input
        assert_eq!(MudGridSpan::parse("0*"), Some(MudGridSpan::Star(0.0)));
        // Boundaries
        assert_eq!(MudGridSpan::parse("12"), Some(MudGridSpan::Tracks(12)));
        assert_eq!(MudGridSpan::parse("13"), Some(MudGridSpan::Tracks(12)));
    }

    #[test]
    fn parsing_rejects_garbage() {
        for bad in ["abc", "-1", "-1*", "inf*", "NaN*", "1.5", "**"] {
            assert_eq!(MudGridSpan::parse(bad), None, "{bad:?} should be rejected");
        }
        assert!("nope".parse::<MudGridSpan>().is_err());
    }

    #[test]
    fn parsing_rejects_overflow_and_mixed_input() {
        // Larger than u32::MAX cannot be parsed as a track count at all
        assert_eq!(MudGridSpan::parse("4294967296"), None);
        assert_eq!(MudGridSpan::parse("auto*"), None);
        assert_eq!(MudGridSpan::parse("2 3"), None);
        assert_eq!(MudGridSpan::parse("*2"), None);
    }

    #[test]
    fn display_round_trips() {
        for span in [
            MudGridSpan::Tracks(4),
            MudGridSpan::Star(1.0),
            MudGridSpan::Star(2.5),
            MudGridSpan::Auto,
            MudGridSpan::Unset,
        ] {
            assert_eq!(span.to_string().parse::<MudGridSpan>(), Ok(span));
        }
    }

    #[test]
    fn display_formats() {
        assert_eq!(MudGridSpan::Unset.to_string(), "");
        assert_eq!(MudGridSpan::Tracks(7).to_string(), "7");
        assert_eq!(MudGridSpan::Star(1.0).to_string(), "*");
        assert_eq!(MudGridSpan::Star(2.0).to_string(), "2*");
        assert_eq!(MudGridSpan::Star(0.5).to_string(), "0.5*");
        assert_eq!(MudGridSpan::Auto.to_string(), "Auto");
        // Zero-weight star also round-trips
        assert_eq!("0*".parse::<MudGridSpan>(), Ok(MudGridSpan::Star(0.0)));
        assert_eq!(MudGridSpan::Star(0.0).to_string().parse::<MudGridSpan>(), Ok(MudGridSpan::Star(0.0)));
    }

    #[test]
    fn parse_error_is_descriptive() {
        let err = ParseSpanError;
        let msg = err.to_string();
        assert!(msg.contains("invalid grid span"));
        assert!(msg.contains("auto"));
        // Usable as a std error trait object
        let boxed: Box<dyn std::error::Error> = Box::new(err);
        assert!(!boxed.to_string().is_empty());
    }

    #[test]
    fn conversions() {
        assert_eq!(MudGridSpan::from(6), MudGridSpan::Tracks(6));
        assert_eq!(MudGridSpan::from(-3), MudGridSpan::Tracks(1)); // Clamped to 1
        assert_eq!(MudGridSpan::from(99), MudGridSpan::Tracks(12));
        assert_eq!(MudGridSpan::from(200u8), MudGridSpan::Tracks(12));
        assert_eq!(MudGridSpan::from(7usize), MudGridSpan::Tracks(7));
        assert_eq!(MudGridSpan::from("2*"), MudGridSpan::Star(2.0));
        assert_eq!(MudGridSpan::from("garbage"), MudGridSpan::Unset);
    }

    #[test]
    fn unsigned_conversions_clamp_both_ends() {
        assert_eq!(MudGridSpan::from(0u8), MudGridSpan::Tracks(1));
        assert_eq!(MudGridSpan::from(0u16), MudGridSpan::Tracks(1));
        assert_eq!(MudGridSpan::from(0u32), MudGridSpan::Tracks(1));
        assert_eq!(MudGridSpan::from(0usize), MudGridSpan::Tracks(1));
        assert_eq!(MudGridSpan::from(u16::MAX), MudGridSpan::Tracks(12));
        assert_eq!(MudGridSpan::from(u32::MAX), MudGridSpan::Tracks(12));
        assert_eq!(MudGridSpan::from(usize::MAX), MudGridSpan::Tracks(12));
        assert_eq!(MudGridSpan::from(i32::MIN), MudGridSpan::Tracks(1));
        assert_eq!(MudGridSpan::from(i32::MAX), MudGridSpan::Tracks(12));
        assert_eq!(MudGridSpan::from(""), MudGridSpan::Unset);
        assert_eq!(MudGridSpan::from("AUTO"), MudGridSpan::Auto);
    }

    #[test]
    fn signed_and_wide_conversions_clamp() {
        assert_eq!(MudGridSpan::from(-5i8), MudGridSpan::Tracks(1));
        assert_eq!(MudGridSpan::from(100i8), MudGridSpan::Tracks(12));
        assert_eq!(MudGridSpan::from(7i16), MudGridSpan::Tracks(7));
        assert_eq!(MudGridSpan::from(i64::MIN), MudGridSpan::Tracks(1));
        assert_eq!(MudGridSpan::from(i64::MAX), MudGridSpan::Tracks(12));
        assert_eq!(MudGridSpan::from(isize::MIN), MudGridSpan::Tracks(1));
        assert_eq!(MudGridSpan::from(isize::MAX), MudGridSpan::Tracks(12));
        assert_eq!(MudGridSpan::from(0u64), MudGridSpan::Tracks(1));
        assert_eq!(MudGridSpan::from(u64::MAX), MudGridSpan::Tracks(12));
    }

    #[test]
    fn owned_string_conversions_are_lenient() {
        assert_eq!(MudGridSpan::from(String::from("3*")), MudGridSpan::Star(3.0));
        assert_eq!(MudGridSpan::from(&String::from("auto")), MudGridSpan::Auto);
        assert_eq!(MudGridSpan::from(String::from("nope")), MudGridSpan::Unset);
    }

    #[test]
    fn accessors() {
        assert_eq!(MudGridSpan::Tracks(5).tracks(), 5);
        assert_eq!(MudGridSpan::Auto.tracks(), 0);
        assert_eq!(MudGridSpan::Star(3.0).weight(), 3.0);
        assert!(MudGridSpan::Auto.is_auto());
        assert!(MudGridSpan::Star(1.0).is_star());
        assert!(MudGridSpan::Tracks(1).is_tracks());
    }

    #[test]
    fn accessors_are_exclusive() {
        assert_eq!(MudGridSpan::Star(3.0).tracks(), 0);
        assert_eq!(MudGridSpan::Tracks(3).weight(), 0.0);
        assert_eq!(MudGridSpan::Unset.weight(), 0.0);
        assert_eq!(MudGridSpan::Unset.tracks(), 0);
        for span in [MudGridSpan::Unset, MudGridSpan::Auto, MudGridSpan::Tracks(2), MudGridSpan::Star(1.0)] {
            let kinds = [span.is_tracks(), span.is_star(), span.is_auto()];
            assert!(kinds.iter().filter(|k| **k).count() <= 1);
        }
        assert!(!MudGridSpan::Unset.is_tracks() && !MudGridSpan::Unset.is_star() && !MudGridSpan::Unset.is_auto());
    }
}