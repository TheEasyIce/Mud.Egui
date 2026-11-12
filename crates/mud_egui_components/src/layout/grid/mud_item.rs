//! A single cell of a `MudGrid`.
//!
//! Port of `MudItem.cs`. Declares how much of the grid it occupies at each
//! Material Design breakpoint. Unset breakpoints inherit from the next
//! smaller one (mobile-first).
//!
//! ```ignore
//! MudItem::new().xs(12).md(6).lg("2*")   // full width, half on md, 2-star on lg+
//! ```
//!
//! # Cascade
//! Breakpoints form a ladder `xs < sm < md < lg < xl < xxl`. Resolving at a breakpoint
//! walks *down* the ladder from that breakpoint until it finds a set span; a larger
//! breakpoint's value is never used for a smaller screen. If nothing is set the item spans
//! the whole row (12 tracks).
//!
//! # Mapping to MudBlazor
//! | MudBlazor `MudItem` | Here |
//! |---------------------|------|
//! | `xs="12"` ... `xxl="3"` | [`MudItem::xs`] ... [`MudItem::xxl`] |
//! | `Class="..."` | [`MudItem::class`] for alignment utilities (`align-self-end`, `text-end`, ...); `ItemStyle` on `GridUi::styled_item` for frames |
//! | `Style="..."` | `ItemStyle::frame` on `GridUi::styled_item` (in `mud_grid.rs`) |
//! | `ChildContent` | the closure handed to `GridUi::item` |
//!
//! `MudItem` carries the six breakpoint spans plus a `&'static str` class list, so it stays
//! `Copy` and cheap to build every frame.

use std::fmt;

use super::mud_grid_span::{MudGridSpan, TRACK_COUNT};
use mud_egui_core::enums::breakpoint::Breakpoint;

/// Number of rungs on the breakpoint ladder (`xs`..`xxl`).
const LADDER_LEN: usize = 6;

/// Breakpoint names in ladder order, used by [`fmt::Display`].
const LADDER_NAMES: [&str; LADDER_LEN] = ["xs", "sm", "md", "lg", "xl", "xxl"];

/// Describes how wide a grid cell is at each breakpoint.
///
/// `Copy` and tiny (six spans), so items are cheap to build per frame and pass by value.
///
/// # Example
/// ```ignore
/// let card = MudItem::new().xs(12).sm(6).lg(4);
/// assert_eq!(card.resolve_span(Breakpoint::Md), MudGridSpan::Tracks(6));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct MudItem {
    /// Span at the smallest breakpoint; the base of the cascade.
    pub xs: MudGridSpan,
    /// Span from the `sm` breakpoint (600px) upward, unless overridden.
    pub sm: MudGridSpan,
    /// Span from the `md` breakpoint (960px) upward, unless overridden.
    pub md: MudGridSpan,
    /// Span from the `lg` breakpoint (1280px) upward, unless overridden.
    pub lg: MudGridSpan,
    /// Span from the `xl` breakpoint (1920px) upward, unless overridden.
    pub xl: MudGridSpan,
    /// Span from the `xxl` breakpoint (2560px) upward, unless overridden.
    pub xxl: MudGridSpan,
    /// Space-separated alignment utility classes, like MudBlazor's `Class="..."`.
    /// See [`MudItem::class`].
    pub class: &'static str,
}

/// Position of `breakpoint` on the ladder (`0` = `xs` ... `5` = `xxl`).
///
/// `Breakpoint::None` maps to `xs`, mirroring the C# `_ => Xs` fallback: with no breakpoint
/// information the safest assumption is the smallest screen.
#[inline]
pub(crate) fn ladder_rank(breakpoint: Breakpoint) -> usize {
    match breakpoint {
        Breakpoint::Xs | Breakpoint::None => 0,
        Breakpoint::Sm | Breakpoint::SmAndDown | Breakpoint::SmAndUp => 1,
        Breakpoint::Md | Breakpoint::MdAndDown | Breakpoint::MdAndUp => 2,
        Breakpoint::Lg | Breakpoint::LgAndDown | Breakpoint::LgAndUp => 3,
        Breakpoint::Xl | Breakpoint::XlAndDown | Breakpoint::XlAndUp => 4,
        Breakpoint::Xxl | Breakpoint::Always => 5,
    }
}

impl MudItem {
    /// Creates an item with every breakpoint unset (spans the full row everywhere).
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// An item that spans the whole row at every breakpoint.
    ///
    /// ```ignore
    /// grid.item(MudItem::full(), |ui| { ui.heading("Title"); });
    /// ```
    #[inline]
    pub fn full() -> Self {
        Self::new().all(MudGridSpan::FULL)
    }

    /// An item that shrinks to its content at every breakpoint.
    #[inline]
    pub fn auto() -> Self {
        Self::new().all(MudGridSpan::Auto)
    }

    /// An item that takes a proportional share of the leftover space at every breakpoint.
    #[inline]
    pub fn star(weight: f32) -> Self {
        Self::new().all(MudGridSpan::Star(weight))
    }

    /// An item sized for `count` equal columns (`12 / count` tracks, integer division).
    ///
    /// `count` is clamped to `1..=12`. For counts that do not divide 12 (5, 7, ...) the row
    /// will not be completely filled; use [`MudItem::star`] when you need exact equal shares.
    ///
    /// ```ignore
    /// let item = MudItem::equal(3);            // four tracks each
    /// assert_eq!(item.xs, MudGridSpan::Tracks(4));
    /// ```
    #[inline]
    pub fn equal(count: u8) -> Self {
        let count = count.clamp(1, TRACK_COUNT);
        Self::new().all(TRACK_COUNT / count)
    }

    /// Builds an item from a full ladder (`[xs, sm, md, lg, xl, xxl]`).
    #[inline]
    pub fn from_ladder(ladder: [MudGridSpan; LADDER_LEN]) -> Self {
        Self {
            xs: ladder[0],
            sm: ladder[1],
            md: ladder[2],
            lg: ladder[3],
            xl: ladder[4],
            xxl: ladder[5],
            class: "",
        }
    }

    /// Sets the item's utility classes, like MudBlazor's `Class="..."` on a `MudItem`.
    ///
    /// Alignment utilities are understood: `align-self-start|center|end|stretch`,
    /// `text-start|center|end`, `ms-auto`, `me-auto`, `mx-auto`, `mt-auto`, `mb-auto`,
    /// `my-auto`, `m-auto`, `w-100`, `h-100`. Other classes are ignored here (frames go through
    /// `GridUi::styled_item`).
    ///
    /// ```ignore
    /// grid.item(MudItem::new().xs(12).sm(6).class("align-self-end"), |ui| { /* ... */ });
    /// ```
    #[inline]
    pub fn class(mut self, class: &'static str) -> Self {
        self.class = class;
        self
    }

    /// Sets the span at `xs` (accepts integers, `"2*"`, `"auto"`, ... via `Into`).
    #[inline]
    pub fn xs(mut self, span: impl Into<MudGridSpan>) -> Self {
        self.xs = span.into();
        self
    }

    /// Sets the span at `sm` and upward until overridden.
    #[inline]
    pub fn sm(mut self, span: impl Into<MudGridSpan>) -> Self {
        self.sm = span.into();
        self
    }

    /// Sets the span at `md` and upward until overridden.
    #[inline]
    pub fn md(mut self, span: impl Into<MudGridSpan>) -> Self {
        self.md = span.into();
        self
    }

    /// Sets the span at `lg` and upward until overridden.
    #[inline]
    pub fn lg(mut self, span: impl Into<MudGridSpan>) -> Self {
        self.lg = span.into();
        self
    }

    /// Sets the span at `xl` and upward until overridden.
    #[inline]
    pub fn xl(mut self, span: impl Into<MudGridSpan>) -> Self {
        self.xl = span.into();
        self
    }

    /// Sets the span at `xxl` and upward.
    #[inline]
    pub fn xxl(mut self, span: impl Into<MudGridSpan>) -> Self {
        self.xxl = span.into();
        self
    }

    /// Sets the same span at every breakpoint.
    ///
    /// Replaces anything set earlier on this item.
    #[inline]
    pub fn all(self, span: impl Into<MudGridSpan>) -> Self {
        let span = span.into();
        // Only the spans are replaced; utility classes set earlier are kept.
        Self { xs: span, sm: span, md: span, lg: span, xl: span, xxl: span, class: self.class }
    }

    /// Sets the span at a breakpoint chosen at run time.
    ///
    /// `Breakpoint::None` writes to `xs`, matching [`resolve_span`](Self::resolve_span).
    ///
    /// ```ignore
    /// let item = MudItem::new().at(Breakpoint::Lg, 4);
    /// assert_eq!(item.lg, MudGridSpan::Tracks(4));
    /// ```
    #[inline]
    pub fn at(mut self, breakpoint: Breakpoint, span: impl Into<MudGridSpan>) -> Self {
        let span = span.into();
        match ladder_rank(breakpoint) {
            0 => self.xs = span,
            1 => self.sm = span,
            2 => self.md = span,
            3 => self.lg = span,
            4 => self.xl = span,
            _ => self.xxl = span,
        }
        self
    }

    /// The raw ladder `[xs, sm, md, lg, xl, xxl]`, exactly as set (no cascade, no normalizing).
    #[inline]
    pub fn ladder(&self) -> [MudGridSpan; LADDER_LEN] {
        [self.xs, self.sm, self.md, self.lg, self.xl, self.xxl]
    }

    /// The raw span stored for `breakpoint` (no cascade, no normalizing).
    ///
    /// `Breakpoint::None` reads `xs`.
    #[inline]
    pub fn get(&self, breakpoint: Breakpoint) -> MudGridSpan {
        self.ladder()[ladder_rank(breakpoint)]
    }

    /// Returns `true` when no breakpoint has been set (the item always spans the full row).
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.ladder().iter().all(|s| !s.is_set())
    }

    /// Resolves the active span for the given breakpoint, cascading down to
    /// smaller breakpoints while the current one is unset. If nothing is set,
    /// the item spans the full row (12 tracks). The result is normalized.
    ///
    /// The ladder is a fixed-size stack array of `Copy` values, so resolving never allocates.
    pub fn resolve_span(&self, breakpoint: Breakpoint) -> MudGridSpan {
        let ladder = self.ladder();
        let rank = ladder_rank(breakpoint);

        ladder[..=rank]
            .iter()
            .rev()
            .copied()
            .find(|s| s.is_set())
            .unwrap_or(MudGridSpan::Unset)
            .normalized()
    }

    /// The cascade resolved at *every* breakpoint at once: `result[rank]` equals
    /// `resolve_span` for that rank.
    ///
    /// Useful for previews, debugging overlays and for hoisting resolution out of a loop that
    /// visits many breakpoints.
    ///
    /// ```ignore
    /// let table = MudItem::new().xs(12).md(6).resolve_all();
    /// assert_eq!(table[1], MudGridSpan::Tracks(12)); // sm inherits xs
    /// assert_eq!(table[2], MudGridSpan::Tracks(6));  // md
    /// ```
    pub fn resolve_all(&self) -> [MudGridSpan; LADDER_LEN] {
        let mut out = [MudGridSpan::FULL; LADDER_LEN];
        let mut carry = MudGridSpan::Unset;
        for (slot, span) in out.iter_mut().zip(self.ladder()) {
            carry = span.or(carry);
            *slot = carry.normalized();
        }
        out
    }
}

/// Lets a bare span stand in for an item that uses it at every breakpoint, so
/// `GridUi::item(MudGridSpan::HALF, ...)` works.
impl From<MudGridSpan> for MudItem {
    #[inline]
    fn from(span: MudGridSpan) -> Self {
        MudItem::new().all(span)
    }
}

/// Formats like the Blazor attributes, listing only the breakpoints that are set:
/// `xs=12 md=6 lg=2*`. An empty item formats as the empty string.
impl fmt::Display for MudItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut first = true;
        for (name, span) in LADDER_NAMES.iter().zip(self.ladder()) {
            if span.is_set() {
                if !first {
                    f.write_str(" ")?;
                }
                first = false;
                write!(f, "{name}={span}")?;
            }
        }
        Ok(())
    }
}

// =============================================================================
// Tests
// =============================================================================

/// Unit tests for the cascade and builders. Nothing here touches egui.
#[cfg(test)]
mod tests {
    use super::*;

    const ALL: [Breakpoint; 6] = [
        Breakpoint::Xs,
        Breakpoint::Sm,
        Breakpoint::Md,
        Breakpoint::Lg,
        Breakpoint::Xl,
        Breakpoint::Xxl,
    ];

    #[test]
    fn unset_item_spans_full_row() {
        let item = MudItem::new();
        for bp in [Breakpoint::Xs, Breakpoint::Md, Breakpoint::Xxl] {
            assert_eq!(item.resolve_span(bp), MudGridSpan::Tracks(12));
        }
    }

    #[test]
    fn cascades_down_from_larger_breakpoints() {
        let item = MudItem::new().xs(12).md(6);
        assert_eq!(item.resolve_span(Breakpoint::Xs), MudGridSpan::Tracks(12));
        assert_eq!(item.resolve_span(Breakpoint::Sm), MudGridSpan::Tracks(12));
        assert_eq!(item.resolve_span(Breakpoint::Md), MudGridSpan::Tracks(6));
        assert_eq!(item.resolve_span(Breakpoint::Lg), MudGridSpan::Tracks(6));
        assert_eq!(item.resolve_span(Breakpoint::Xxl), MudGridSpan::Tracks(6));
    }

    #[test]
    fn small_breakpoints_never_use_larger_values() {
        let item = MudItem::new().lg(4);
        assert_eq!(item.resolve_span(Breakpoint::Md), MudGridSpan::Tracks(12));
        assert_eq!(item.resolve_span(Breakpoint::Lg), MudGridSpan::Tracks(4));
    }

    #[test]
    fn star_and_auto_cascade() {
        let item = MudItem::new().sm("2*").xl("auto");
        assert_eq!(item.resolve_span(Breakpoint::Md), MudGridSpan::Star(2.0));
        assert_eq!(item.resolve_span(Breakpoint::Xl), MudGridSpan::Auto);
        assert_eq!(item.resolve_span(Breakpoint::Xxl), MudGridSpan::Auto);
    }

    #[test]
    fn results_are_normalized() {
        assert_eq!(MudItem::new().xs(0).resolve_span(Breakpoint::Xs), MudGridSpan::Tracks(1));
        assert_eq!(MudItem::new().xs(99).resolve_span(Breakpoint::Xs), MudGridSpan::Tracks(12));
    }

    #[test]
    fn all_sets_every_breakpoint() {
        let item = MudItem::new().all(4);
        for bp in [Breakpoint::Xs, Breakpoint::Sm, Breakpoint::Md, Breakpoint::Lg, Breakpoint::Xl, Breakpoint::Xxl] {
            assert_eq!(item.resolve_span(bp), MudGridSpan::Tracks(4));
        }
    }

    #[test]
    fn ladder_rank_mapping() {
        assert_eq!(ladder_rank(Breakpoint::None), 0);
        assert_eq!(ladder_rank(Breakpoint::Xs), 0);
        assert_eq!(ladder_rank(Breakpoint::Sm), 1);
        assert_eq!(ladder_rank(Breakpoint::Md), 2);
        assert_eq!(ladder_rank(Breakpoint::Lg), 3);
        assert_eq!(ladder_rank(Breakpoint::Xl), 4);
        assert_eq!(ladder_rank(Breakpoint::Xxl), 5);
        assert_eq!(LADDER_LEN, ALL.len());
        assert_eq!(LADDER_NAMES.len(), LADDER_LEN);
    }

    #[test]
    fn none_breakpoint_behaves_like_xs() {
        let item = MudItem::new().xs(3).md(6);
        assert_eq!(item.resolve_span(Breakpoint::None), MudGridSpan::Tracks(3));
        assert_eq!(MudItem::new().resolve_span(Breakpoint::None), MudGridSpan::Tracks(12));
    }

    #[test]
    fn each_setter_writes_only_its_field() {
        let item = MudItem::new().xs(1).sm(2).md(3).lg(4).xl(5).xxl(6);
        assert_eq!(item.xs, MudGridSpan::Tracks(1));
        assert_eq!(item.sm, MudGridSpan::Tracks(2));
        assert_eq!(item.md, MudGridSpan::Tracks(3));
        assert_eq!(item.lg, MudGridSpan::Tracks(4));
        assert_eq!(item.xl, MudGridSpan::Tracks(5));
        assert_eq!(item.xxl, MudGridSpan::Tracks(6));
        // And each breakpoint resolves to its own value.
        for (bp, expected) in ALL.iter().zip(1u8..=6) {
            assert_eq!(item.resolve_span(*bp), MudGridSpan::Tracks(expected));
        }
    }

    #[test]
    fn setters_accept_various_inputs() {
        let item = MudItem::new().xs(6usize).sm(4u8).md("*").lg("auto").xl("3*").xxl(12u32);
        assert_eq!(item.xs, MudGridSpan::Tracks(6));
        assert_eq!(item.sm, MudGridSpan::Tracks(4));
        assert_eq!(item.md, MudGridSpan::Star(1.0));
        assert_eq!(item.lg, MudGridSpan::Auto);
        assert_eq!(item.xl, MudGridSpan::Star(3.0));
        assert_eq!(item.xxl, MudGridSpan::Tracks(12));
    }

    #[test]
    fn empty_string_unsets_and_cascades_again() {
        let item = MudItem::new().xs(3).md("");
        assert_eq!(item.md, MudGridSpan::Unset);
        assert_eq!(item.resolve_span(Breakpoint::Md), MudGridSpan::Tracks(3));
        // Garbage strings also become Unset.
        assert_eq!(MudItem::new().xs(3).md("garbage").resolve_span(Breakpoint::Md), MudGridSpan::Tracks(3));
    }

    #[test]
    fn all_overrides_previous_settings() {
        let item = MudItem::new().md(6).all(2);
        for bp in ALL {
            assert_eq!(item.resolve_span(bp), MudGridSpan::Tracks(2));
        }
        // Setting a breakpoint after `all` wins at that breakpoint; `all` also set `xl`/`xxl`
        // explicitly, so those keep their own value instead of inheriting from `lg`.
        let item = item.lg(8);
        assert_eq!(item.resolve_span(Breakpoint::Md), MudGridSpan::Tracks(2));
        assert_eq!(item.resolve_span(Breakpoint::Lg), MudGridSpan::Tracks(8));
        assert_eq!(item.resolve_span(Breakpoint::Xl), MudGridSpan::Tracks(2));
        assert_eq!(item.resolve_span(Breakpoint::Xxl), MudGridSpan::Tracks(2));
    }

    #[test]
    fn default_equals_new_and_is_copy() {
        assert_eq!(MudItem::new(), MudItem::default());
        let a = MudItem::new().xs(4);
        let b = a; // Copy
        assert_eq!(a, b);
        assert_ne!(a, MudItem::new());
    }

    #[test]
    fn at_writes_the_matching_field() {
        let item = MudItem::new()
            .at(Breakpoint::Xs, 1)
            .at(Breakpoint::Sm, 2)
            .at(Breakpoint::Md, 3)
            .at(Breakpoint::Lg, 4)
            .at(Breakpoint::Xl, 5)
            .at(Breakpoint::Xxl, 6);
        assert_eq!(item, MudItem::new().xs(1).sm(2).md(3).lg(4).xl(5).xxl(6));
        // `None` writes to xs.
        assert_eq!(MudItem::new().at(Breakpoint::None, 7).xs, MudGridSpan::Tracks(7));
        // Strings work too.
        assert_eq!(MudItem::new().at(Breakpoint::Md, "2*").md, MudGridSpan::Star(2.0));
    }

    #[test]
    fn get_returns_raw_unresolved_values() {
        let item = MudItem::new().xs(12).md(6);
        assert_eq!(item.get(Breakpoint::Xs), MudGridSpan::Tracks(12));
        assert_eq!(item.get(Breakpoint::Sm), MudGridSpan::Unset); // raw: no cascade
        assert_eq!(item.get(Breakpoint::Md), MudGridSpan::Tracks(6));
        assert_eq!(item.get(Breakpoint::None), MudGridSpan::Tracks(12));
        // Raw values are not normalized either.
        assert_eq!(MudItem::new().xs(0u8).get(Breakpoint::Xs), MudGridSpan::Tracks(1)); // From<u8> clamps already
        assert_eq!(MudItem::new().xs("0").get(Breakpoint::Xs), MudGridSpan::Tracks(0)); // parse keeps 0
    }

    #[test]
    fn ladder_round_trips() {
        let item = MudItem::new().xs(1).sm("auto").md("2*").xl(9);
        assert_eq!(MudItem::from_ladder(item.ladder()), item);
        let ladder = item.ladder();
        assert_eq!(ladder[0], MudGridSpan::Tracks(1));
        assert_eq!(ladder[1], MudGridSpan::Auto);
        assert_eq!(ladder[2], MudGridSpan::Star(2.0));
        assert_eq!(ladder[3], MudGridSpan::Unset);
        assert_eq!(ladder[4], MudGridSpan::Tracks(9));
        assert_eq!(ladder[5], MudGridSpan::Unset);
    }

    #[test]
    fn is_empty_reports_untouched_items() {
        assert!(MudItem::new().is_empty());
        assert!(!MudItem::new().xxl(1).is_empty());
        assert!(!MudItem::auto().is_empty());
        // Clearing the only set value makes it empty again.
        assert!(MudItem::new().md(6).md("").is_empty());
    }

    #[test]
    fn presets() {
        for bp in ALL {
            assert_eq!(MudItem::full().resolve_span(bp), MudGridSpan::Tracks(12));
            assert_eq!(MudItem::auto().resolve_span(bp), MudGridSpan::Auto);
            assert_eq!(MudItem::star(2.0).resolve_span(bp), MudGridSpan::Star(2.0));
            assert_eq!(MudItem::equal(3).resolve_span(bp), MudGridSpan::Tracks(4));
        }
        assert_eq!(MudItem::equal(0).xs, MudGridSpan::Tracks(12)); // clamped to one column
        assert_eq!(MudItem::equal(12).xs, MudGridSpan::Tracks(1));
        assert_eq!(MudItem::equal(200).xs, MudGridSpan::Tracks(1)); // clamped to 12 columns
        assert_eq!(MudItem::equal(5).xs, MudGridSpan::Tracks(2)); // integer division
    }

    #[test]
    fn resolve_all_matches_resolve_span() {
        let items = [
            MudItem::new(),
            MudItem::new().xs(12).md(6),
            MudItem::new().lg(4),
            MudItem::new().sm("2*").xl("auto"),
            MudItem::new().md(0).xxl(99),
            MudItem::new().md(6).all(2).lg(8),
        ];
        for item in items {
            let table = item.resolve_all();
            for bp in ALL {
                assert_eq!(table[ladder_rank(bp)], item.resolve_span(bp));
            }
        }
    }

    #[test]
    fn from_span_applies_everywhere() {
        let item: MudItem = MudGridSpan::HALF.into();
        assert_eq!(item, MudItem::new().all(6));
        assert_eq!(MudItem::from(MudGridSpan::Auto), MudItem::auto());
    }

    #[test]
    fn display_lists_only_set_breakpoints() {
        assert_eq!(MudItem::new().to_string(), "");
        assert_eq!(MudItem::new().xs(12).md(6).lg("2*").to_string(), "xs=12 md=6 lg=2*");
        assert_eq!(MudItem::new().xxl("auto").to_string(), "xxl=Auto");
        assert_eq!(MudItem::new().sm("*").to_string(), "sm=*");
        assert_eq!(
            MudItem::new().all(3).to_string(),
            "xs=3 sm=3 md=3 lg=3 xl=3 xxl=3"
        );
    }
}