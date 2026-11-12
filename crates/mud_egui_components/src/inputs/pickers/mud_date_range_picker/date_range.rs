//! `DateRange` — a pair of optional dates, the value type of
//! [`MudDateRangePicker`](super::MudDateRangePicker).
//!
//! Port of `MudBlazor.DateRange` (MudBlazor 8.13.0). `DateRange` derives from
//! `Range<DateTime?>` and exposes `Start` ("the earliest date") and `End`
//! ("the most recent date"), both nullable. An empty range has both `None`;
//! a half-open range (one click of the two-click selection) has one.
//!
//! **Ordering.** The C# constructor assigns its arguments as given; the
//! *picker* is what normalises so `Start <= End`. This type keeps that split:
//! [`DateRange::new`] preserves the arguments, and
//! [`DateRange::normalized`] performs the ordering, so a caller can inspect
//! the click order if it wants to.
//!
//! **Day counts are inclusive** — a range whose start and end are the same
//! day has a count of `1`, matching `MinDays` / `MaxDays`
//! ("Inclusive of the selected date").
//!
//! Kept in its own file because it is a first-class public type, not a field
//! of the component.

use std::fmt;
use crate::inputs::pickers::mud_date_picker::SimpleDate;

/// Two optional dates forming a range.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct DateRange {
    /// The earliest date. `None` when nothing is selected.
    pub start: Option<SimpleDate>,
    /// The most recent date. `None` when nothing is selected, or when the
    /// two-click selection is waiting for its second click.
    pub end: Option<SimpleDate>,
}

impl DateRange {
    /// Creates a range from `start` and `end` without reordering them.
    ///
    /// Mirrors the C# `DateRange(DateTime? start, DateTime? end)`
    /// constructor, which also assigns as given. Use
    /// [`normalized`](Self::normalized) to order the pair.
    #[inline]
    pub fn new(start: Option<SimpleDate>, end: Option<SimpleDate>) -> Self {
        Self { start, end }
    }

    /// An empty range — nothing selected.
    #[inline]
    pub fn empty() -> Self {
        Self::default()
    }

    /// A single-day range: `start == end == date`.
    #[inline]
    pub fn single(date: SimpleDate) -> Self {
        Self { start: Some(date), end: Some(date) }
    }

    /// `true` when neither endpoint is set.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.start.is_none() && self.end.is_none()
    }

    /// `true` when both endpoints are set.
    #[inline]
    pub fn is_complete(&self) -> bool {
        self.start.is_some() && self.end.is_some()
    }

    /// `true` when both endpoints are set and equal.
    #[inline]
    pub fn is_same_day(&self) -> bool {
        matches!((self.start, self.end), (Some(s), Some(e)) if s == e)
    }

    /// Returns the range with `start <= end`.
    ///
    /// Mirrors the picker's "automatically orders dates so Start <= End,
    /// regardless of click order" rule. A half-open or empty range is
    /// returned unchanged.
    pub fn normalized(self) -> Self {
        match (self.start, self.end) {
            (Some(s), Some(e)) if s > e => Self { start: Some(e), end: Some(s) },
            _ => self,
        }
    }

    /// `true` when `date` lies inside the range (inclusive on both ends).
    ///
    /// A half-open range (`start` only) contains exactly its start day,
    /// which is how the first click of the two-click selection is shown.
    pub fn contains(&self, date: &SimpleDate) -> bool {
        match (self.start, self.end) {
            (Some(s), Some(e)) => {
                let (s, e) = if s <= e { (s, e) } else { (e, s) };
                *date >= s && *date <= e
            }
            (Some(s), None) => *date == s,
            _ => false,
        }
    }

    /// Number of calendar days in the range, inclusive. `None` unless both
    /// endpoints are set.
    ///
    /// `2024-01-01 ..= 2024-01-01` is `1`; `2024-01-01 ..= 2024-01-31` is
    /// `31`, matching `MinDays` / `MaxDays`'s inclusive counting.
    pub fn day_count(&self) -> Option<i32> {
        match (self.start, self.end) {
            (Some(s), Some(e)) => Some(inclusive_days(s, e)),
            _ => None,
        }
    }

    /// Formats both endpoints with `format`, falling back to ISO 8601.
    ///
    /// Mirrors the pair of texts `MudRangeInput` shows: an empty string for
    /// an unset endpoint.
    pub fn formatted(
        &self,
        format: Option<&dyn Fn(&SimpleDate) -> String>,
    ) -> (String, String) {
        (format_one(self.start, format), format_one(self.end, format))
    }
}

/// Formats one optional date with `format`, falling back to ISO 8601.
pub fn format_one(
    date: Option<SimpleDate>,
    format: Option<&dyn Fn(&SimpleDate) -> String>,
) -> String {
    match date {
        None => String::new(),
        Some(d) => match format {
            Some(f) => f(&d),
            None => d.format_iso(),
        },
    }
}

/// Inclusive number of days between `a` and `b`, in either order.
///
/// `inclusive_days(a, a) == 1`.
pub fn inclusive_days(a: SimpleDate, b: SimpleDate) -> i32 {
    (julian_day(b) - julian_day(a)).abs() + 1
}

/// Julian day number (Fliegel–Van Flandern). `O(1)`, valid for the whole
/// proleptic Gregorian range the picker supports.
fn julian_day(d: SimpleDate) -> i32 {
    let a = (14 - d.month as i32).div_euclid(12);
    let y = d.year + 4800 - a;
    let m = d.month as i32 + 12 * a - 3;
    d.day as i32
        + (153 * m + 2).div_euclid(5)
        + 365 * y
        + y.div_euclid(4)
        - y.div_euclid(100)
        + y.div_euclid(400)
        - 32045
}

impl fmt::Display for DateRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} ..= {}",
            self.start.map(|d| d.format_iso()).unwrap_or_else(|| "—".into()),
            self.end.map(|d| d.format_iso()).unwrap_or_else(|| "—".into()),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u8, day: u8) -> SimpleDate {
        SimpleDate::new(y, m, day)
    }

    // --- constructors -----------------------------------------------------

    #[test]
    fn empty_and_default_agree() {
        assert_eq!(DateRange::empty(), DateRange::default());
        assert!(DateRange::empty().is_empty());
        assert!(!DateRange::empty().is_complete());
    }

    #[test]
    fn new_preserves_argument_order() {
        let r = DateRange::new(Some(d(2024, 5, 1)), Some(d(2024, 1, 1)));
        assert_eq!(r.start, Some(d(2024, 5, 1)));
        assert_eq!(r.end, Some(d(2024, 1, 1)));
    }

    #[test]
    fn single_is_one_day() {
        let r = DateRange::single(d(2024, 3, 14));
        assert!(r.is_complete());
        assert!(r.is_same_day());
        assert_eq!(r.day_count(), Some(1));
    }

    // --- completeness -----------------------------------------------------

    #[test]
    fn half_open_range_is_not_complete() {
        let r = DateRange::new(Some(d(2024, 3, 14)), None);
        assert!(!r.is_empty());
        assert!(!r.is_complete());
        assert_eq!(r.day_count(), None);
    }

    #[test]
    fn is_same_day_requires_both_ends() {
        assert!(!DateRange::new(Some(d(2024, 3, 14)), None).is_same_day());
        assert!(!DateRange::new(None, None).is_same_day());
        assert!(DateRange::new(Some(d(2024, 3, 14)), Some(d(2024, 3, 14))).is_same_day());
    }

    // --- normalization ----------------------------------------------------

    #[test]
    fn normalized_orders_a_reversed_range() {
        let r = DateRange::new(Some(d(2024, 5, 1)), Some(d(2024, 1, 1))).normalized();
        assert_eq!(r.start, Some(d(2024, 1, 1)));
        assert_eq!(r.end, Some(d(2024, 5, 1)));
    }

    #[test]
    fn normalized_is_idempotent_and_keeps_partial_ranges() {
        let r = DateRange::new(Some(d(2024, 1, 1)), Some(d(2024, 5, 1)));
        assert_eq!(r.normalized(), r);
        assert_eq!(DateRange::empty().normalized(), DateRange::empty());
        let half = DateRange::new(Some(d(2024, 5, 1)), None);
        assert_eq!(half.normalized(), half);
    }

    // --- contains ---------------------------------------------------------

    #[test]
    fn contains_is_inclusive_on_both_ends() {
        let r = DateRange::new(Some(d(2024, 1, 1)), Some(d(2024, 1, 31)));
        assert!(r.contains(&d(2024, 1, 1)));
        assert!(r.contains(&d(2024, 1, 31)));
        assert!(r.contains(&d(2024, 1, 15)));
        assert!(!r.contains(&d(2023, 12, 31)));
        assert!(!r.contains(&d(2024, 2, 1)));
    }

    #[test]
    fn contains_handles_a_reversed_range() {
        let r = DateRange::new(Some(d(2024, 1, 31)), Some(d(2024, 1, 1)));
        assert!(r.contains(&d(2024, 1, 15)));
    }

    #[test]
    fn contains_a_half_open_range_only_its_start() {
        let r = DateRange::new(Some(d(2024, 1, 15)), None);
        assert!(r.contains(&d(2024, 1, 15)));
        assert!(!r.contains(&d(2024, 1, 16)));
        assert!(!DateRange::empty().contains(&d(2024, 1, 15)));
    }

    // --- day counts -------------------------------------------------------

    #[test]
    fn inclusive_days_counts_both_ends() {
        assert_eq!(inclusive_days(d(2024, 1, 1), d(2024, 1, 1)), 1);
        assert_eq!(inclusive_days(d(2024, 1, 1), d(2024, 1, 2)), 2);
        assert_eq!(inclusive_days(d(2024, 1, 1), d(2024, 1, 31)), 31);
        assert_eq!(inclusive_days(d(2024, 1, 31), d(2024, 1, 1)), 31);
    }

    #[test]
    fn inclusive_days_spans_months_and_years() {
        assert_eq!(inclusive_days(d(2024, 1, 31), d(2024, 2, 1)), 2);
        assert_eq!(inclusive_days(d(2024, 2, 28), d(2024, 3, 1)), 3); // leap
        assert_eq!(inclusive_days(d(2023, 2, 28), d(2023, 3, 1)), 2);
        assert_eq!(inclusive_days(d(2024, 12, 31), d(2025, 1, 1)), 2);
        assert_eq!(inclusive_days(d(2020, 1, 1), d(2030, 1, 1)), 3653); // 2 leap days
    }

    #[test]
    fn day_count_uses_the_inclusive_span() {
        assert_eq!(
            DateRange::new(Some(d(2024, 3, 1)), Some(d(2024, 3, 7))).day_count(),
            Some(7)
        );
        assert_eq!(
            DateRange::new(Some(d(2024, 3, 7)), Some(d(2024, 3, 1))).day_count(),
            Some(7)
        );
    }

    // --- formatting -------------------------------------------------------

    #[test]
    fn formatted_uses_the_closure() {
        let fmt = |d: &SimpleDate| format!("{:02}/{:02}/{:04}", d.day, d.month, d.year);
        let r = DateRange::new(Some(d(2024, 3, 14)), Some(d(2024, 3, 20)));
        assert_eq!(r.formatted(Some(&fmt)), ("14/03/2024".into(), "20/03/2024".into()));
    }

    #[test]
    fn formatted_falls_back_to_iso_and_blanks_the_missing_end() {
        let r = DateRange::new(Some(d(2024, 3, 14)), None);
        assert_eq!(r.formatted(None), ("2024-03-14".into(), String::new()));
    }

    #[test]
    fn display_shows_em_dashes_for_missing_ends() {
        let r = DateRange::new(Some(d(2024, 3, 14)), None);
        assert_eq!(r.to_string(), "2024-03-14 ..= —");
        assert_eq!(DateRange::empty().to_string(), "— ..= —");
    }
}