//! A minimal calendar date type — no external dependencies.
//!
//! `SimpleDate` carries only what the [`MudDatePicker`](super::MudDatePicker)
//! needs: a year (i32), a month (1..=12), and a day (1..=31, clamped against
//! the month). All arithmetic is pure and unit-tested, so the picker's own
//! tests can reason about exact dates without needing a `chrono` dependency
//! or a system clock.
//!
//! **Formatting**: [`SimpleDate::format_iso`] produces `"YYYY-MM-DD"`. Callers
//! who want a different display format pass their own
//! `Fn(&SimpleDate) -> String` to [`MudDatePicker::date_format`](
//! super::MudDatePicker::date_format).

use std::fmt;

/// A calendar date (proleptic Gregorian).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SimpleDate {
    /// Full year (`2025`).
    pub year: i32,
    /// Month, `1..=12`.
    pub month: u8,
    /// Day of month, `1..=31` (clamped to the month).
    pub day: u8,
}

impl SimpleDate {
    /// Creates a date, clamping `month` to `1..=12` and `day` to
    /// `1..=days_in_month(year, month)`.
    pub fn new(year: i32, month: u8, day: u8) -> Self {
        let month = month.clamp(1, 12);
        let day = day.clamp(1, Self::days_in_month(year, month));
        Self { year, month, day }
    }

    /// Creates the first day of `month` in `year`.
    pub fn first_of_month(year: i32, month: u8) -> Self {
        Self::new(year, month, 1)
    }

    /// Whether the year is a Gregorian leap year.
    #[inline]
    pub fn is_leap_year(year: i32) -> bool {
        (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
    }

    /// Days in `month` of `year`.
    pub fn days_in_month(year: i32, month: u8) -> u8 {
        match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 => if Self::is_leap_year(year) { 29 } else { 28 },
            _ => 31,
        }
    }

    /// Whether this date is in a valid range (always true after `new`).
    #[inline]
    pub fn is_valid(&self) -> bool {
        (1..=12).contains(&self.month)
            && (1..=Self::days_in_month(self.year, self.month)).contains(&self.day)
    }

    /// Day-of-week index with **Monday = 0**, Sunday = 6.
    ///
    /// Uses Sakamoto's algorithm (constant time, no lookup table beyond 12
    /// entries). Valid for any Gregorian date.
    pub fn weekday_monday_based(&self) -> u8 {
        static T: [i32; 12] = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
        let y = if self.month < 3 { self.year - 1 } else { self.year };
        let m = self.month as i32;
        let d = self.day as i32;
        // Sakamoto: 0 = Sunday .. 6 = Saturday.
        let sakamoto = (y + y / 4 - y / 100 + y / 400 + T[(m - 1) as usize] + d).rem_euclid(7);
        // Monday-based: (0=Sun -> 6, 1=Mon -> 0, ..., 6=Sat -> 5)
        ((sakamoto + 6) % 7) as u8
    }

    /// Returns `self + days`, using proper Gregorian roll-over.
    pub fn add_days(self, days: i32) -> Self {
        let mut out = self;
        let step = if days >= 0 { 1 } else { -1 };
        for _ in 0..days.abs() {
            out = out.step_one_day(step);
        }
        out
    }

    /// One day forward (`step = 1`) or backward (`step = -1`).
    fn step_one_day(self, step: i32) -> Self {
        let dim = Self::days_in_month(self.year, self.month) as i32;
        let d = self.day as i32 + step;
        if d < 1 {
            let (y, m) = if self.month == 1 {
                (self.year - 1, 12)
            } else {
                (self.year, self.month - 1)
            };
            let dim_prev = Self::days_in_month(y, m);
            Self::new(y, m, dim_prev)
        } else if d > dim {
            let (y, m) = if self.month == 12 {
                (self.year + 1, 1)
            } else {
                (self.year, self.month + 1)
            };
            Self::new(y, m, 1)
        } else {
            Self::new(self.year, self.month, d as u8)
        }
    }

    /// Returns `self` shifted by `months` calendar months (day clamped).
    pub fn add_months(self, months: i32) -> Self {
        let total = self.year * 12 + (self.month as i32 - 1) + months;
        let year = total.div_euclid(12);
        let month = (total.rem_euclid(12) + 1) as u8;
        Self::new(year, month, self.day)
    }

    /// Returns `self` shifted by `years` calendar years (day clamped).
    #[inline]
    pub fn add_years(self, years: i32) -> Self {
        self.add_months(years * 12)
    }

    /// First day of the month containing `self`.
    #[inline]
    pub fn first_of_this_month(self) -> Self {
        Self::new(self.year, self.month, 1)
    }

    /// Last day of the month containing `self`.
    #[inline]
    pub fn last_of_this_month(self) -> Self {
        Self::new(self.year, self.month, Self::days_in_month(self.year, self.month))
    }

    /// Whether `self` is before `other` on the calendar.
    #[inline]
    pub fn is_before(&self, other: &Self) -> bool {
        (self.year, self.month, self.day) < (other.year, other.month, other.day)
    }

    /// Whether `self` is after `other` on the calendar.
    #[inline]
    pub fn is_after(&self, other: &Self) -> bool {
        (self.year, self.month, self.day) > (other.year, other.month, other.day)
    }

    /// `true` when `self` is strictly inside `min..=max`. Either bound may be
    /// `None`, in which case that bound is unbounded.
    #[inline]
    pub fn in_range(&self, min: Option<&Self>, max: Option<&Self>) -> bool {
        if let Some(m) = min {
            if self.is_before(m) {
                return false;
            }
        }
        if let Some(m) = max {
            if self.is_after(m) {
                return false;
            }
        }
        true
    }

    /// Clamps `self` into `min..=max`.
    #[inline]
    pub fn clamped(self, min: Option<&Self>, max: Option<&Self>) -> Self {
        if let Some(m) = min {
            if self.is_before(m) {
                return *m;
            }
        }
        if let Some(m) = max {
            if self.is_after(m) {
                return *m;
            }
        }
        self
    }

    /// `YYYY-MM-DD` (ISO 8601).
    pub fn format_iso(&self) -> String {
        format!("{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }

    /// Builds the 42-day (6-week) grid that displays `year` / `month`,
    /// starting on `first_day_of_week` (0 = Monday, 6 = Sunday).
    ///
    /// Returns `None` if `first_day_of_week` is out of range.
    pub fn calendar_grid(year: i32, month: u8, first_day_of_week: u8) -> Option<Vec<SimpleDate>> {
        if first_day_of_week > 6 {
            return None;
        }
        let first = Self::first_of_month(year, month);
        let wd = first.weekday_monday_based();
        let offset = ((wd + 7 - first_day_of_week) % 7) as i32;
        let start = first.add_days(-offset);
        let mut out = Vec::with_capacity(42);
        let mut d = start;
        for _ in 0..42 {
            out.push(d);
            d = d.add_days(1);
        }
        Some(out)
    }

    /// `true` when `self` and `other` fall in the same calendar month.
    #[inline]
    pub fn same_month(&self, other: &Self) -> bool {
        self.year == other.year && self.month == other.month
    }
}

impl fmt::Display for SimpleDate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.format_iso())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_clamps_invalid_month_and_day() {
        let d = SimpleDate::new(2024, 0, 0);
        assert_eq!(d.month, 1);
        assert_eq!(d.day, 1);
        let d = SimpleDate::new(2024, 13, 99);
        assert_eq!(d.month, 12);
        assert_eq!(d.day, 31);
        let d = SimpleDate::new(2024, 2, 30);
        assert_eq!(d.day, 29); // leap
        let d = SimpleDate::new(2023, 2, 30);
        assert_eq!(d.day, 28); // non-leap
    }

    #[test]
    fn leap_years() {
        assert!(SimpleDate::is_leap_year(2000));
        assert!(SimpleDate::is_leap_year(2024));
        assert!(!SimpleDate::is_leap_year(1900));
        assert!(!SimpleDate::is_leap_year(2023));
    }

    #[test]
    fn days_in_month_returns_expected() {
        assert_eq!(SimpleDate::days_in_month(2024, 1), 31);
        assert_eq!(SimpleDate::days_in_month(2024, 2), 29);
        assert_eq!(SimpleDate::days_in_month(2023, 2), 28);
        assert_eq!(SimpleDate::days_in_month(2024, 4), 30);
        assert_eq!(SimpleDate::days_in_month(2024, 12), 31);
    }

    #[test]
    fn weekday_monday_based_matches_known_dates() {
        // 2024-01-01 was a Monday.
        assert_eq!(SimpleDate::new(2024, 1, 1).weekday_monday_based(), 0);
        // 2024-01-07 was a Sunday.
        assert_eq!(SimpleDate::new(2024, 1, 7).weekday_monday_based(), 6);
        // 2025-01-01 is a Wednesday.
        assert_eq!(SimpleDate::new(2025, 1, 1).weekday_monday_based(), 2);
    }

    #[test]
    fn add_days_rolls_over_month_and_year() {
        assert_eq!(SimpleDate::new(2024, 1, 31).add_days(1), SimpleDate::new(2024, 2, 1));
        assert_eq!(SimpleDate::new(2024, 2, 29).add_days(1), SimpleDate::new(2024, 3, 1));
        assert_eq!(SimpleDate::new(2024, 12, 31).add_days(1), SimpleDate::new(2025, 1, 1));
        assert_eq!(SimpleDate::new(2024, 1, 1).add_days(-1), SimpleDate::new(2023, 12, 31));
    }

    #[test]
    fn add_months_clamps_day() {
        // 31 Jan + 1 month → 29 Feb (leap)
        assert_eq!(
            SimpleDate::new(2024, 1, 31).add_months(1),
            SimpleDate::new(2024, 2, 29)
        );
        // 31 Mar + 1 month → 30 Apr
        assert_eq!(
            SimpleDate::new(2024, 3, 31).add_months(1),
            SimpleDate::new(2024, 4, 30)
        );
        // 2024-01 + 12 → 2025-01
        assert_eq!(
            SimpleDate::new(2024, 1, 15).add_months(12),
            SimpleDate::new(2025, 1, 15)
        );
        // 2024-01 + -1 → 2023-12
        assert_eq!(
            SimpleDate::new(2024, 1, 15).add_months(-1),
            SimpleDate::new(2023, 12, 15)
        );
    }

    #[test]
    fn add_years_shifts_year() {
        assert_eq!(
            SimpleDate::new(2024, 1, 1).add_years(5),
            SimpleDate::new(2029, 1, 1)
        );
        assert_eq!(
            SimpleDate::new(2024, 2, 29).add_years(1), // 2024 has leap Feb
            SimpleDate::new(2025, 2, 28)                 // 2025 clamps
        );
    }

    #[test]
    fn first_and_last_of_this_month() {
        let d = SimpleDate::new(2024, 2, 15);
        assert_eq!(d.first_of_this_month(), SimpleDate::new(2024, 2, 1));
        assert_eq!(d.last_of_this_month(), SimpleDate::new(2024, 2, 29));
    }

    #[test]
    fn is_before_after_and_range() {
        let a = SimpleDate::new(2024, 1, 1);
        let b = SimpleDate::new(2024, 1, 2);
        assert!(a.is_before(&b));
        assert!(!a.is_after(&b));
        let min = SimpleDate::new(2024, 1, 1);
        let max = SimpleDate::new(2024, 12, 31);
        assert!(a.in_range(Some(&min), Some(&max)));
        assert!(SimpleDate::new(2025, 1, 1).in_range(Some(&min), Some(&max)) == false);
        assert!(a.in_range(None, None));
    }

    #[test]
    fn clamped_bounds() {
        let min = SimpleDate::new(2024, 6, 1);
        let max = SimpleDate::new(2024, 8, 31);
        assert_eq!(SimpleDate::new(2024, 1, 1).clamped(Some(&min), Some(&max)), min);
        assert_eq!(SimpleDate::new(2024, 12, 31).clamped(Some(&min), Some(&max)), max);
        let mid = SimpleDate::new(2024, 7, 15);
        assert_eq!(mid.clamped(Some(&min), Some(&max)), mid);
    }

    #[test]
    fn format_iso_is_padded() {
        assert_eq!(SimpleDate::new(2024, 1, 5).format_iso(), "2024-01-05");
        assert_eq!(SimpleDate::new(999, 12, 31).format_iso(), "0999-12-31");
    }

    #[test]
    fn display_uses_iso() {
        assert_eq!(SimpleDate::new(2024, 3, 14).to_string(), "2024-03-14");
    }

    #[test]
    fn calendar_grid_is_42_days_and_starts_on_correct_weekday() {
        let grid = SimpleDate::calendar_grid(2024, 1, 0).unwrap();
        assert_eq!(grid.len(), 42);
        // 2024-01-01 was a Monday, so first_day_of_week=0 → grid starts on
        // 2024-01-01.
        assert_eq!(grid[0], SimpleDate::new(2024, 1, 1));
        // Last cell should be 2024-02-11 (42 days after start - 1).
        assert_eq!(grid[41], SimpleDate::new(2024, 2, 11));
    }

    #[test]
    fn calendar_grid_with_sunday_start() {
        let grid = SimpleDate::calendar_grid(2024, 1, 6).unwrap(); // Sunday = 6
        // First cell is the Sunday before 2024-01-01 (Mon) → 2023-12-31.
        assert_eq!(grid[0], SimpleDate::new(2023, 12, 31));
    }

    #[test]
    fn calendar_grid_rejects_bad_first_day() {
        assert!(SimpleDate::calendar_grid(2024, 1, 7).is_none());
    }

    #[test]
    fn same_month_works() {
        let a = SimpleDate::new(2024, 3, 1);
        let b = SimpleDate::new(2024, 3, 31);
        let c = SimpleDate::new(2024, 4, 1);
        assert!(a.same_month(&b));
        assert!(!a.same_month(&c));
    }
}