//! A minimal time-of-day type — no external dependencies.
//!
//! `SimpleTime` carries only what the [`MudTimePicker`](super::MudTimePicker)
//! needs: hours (`0..=23`), minutes (`0..=59`) and seconds (`0..=59`). All
//! arithmetic is pure and unit-tested, so the picker's own tests can reason
//! about exact times without a `chrono` dependency or a system clock.
//!
//! **Seconds** are carried for parity with `TimeSpan`: a seeded value keeps
//! them, but the dial only selects hours and minutes, so a new selection
//! sets seconds to `0`.
//!
//! **Formatting**: [`SimpleTime::format`] evaluates a MudBlazor-style
//! `TimeFormat` template (`HH`, `H`, `hh`, `h`, `mm`, `m`, `ss`, `s`, `tt`).
//! [`SimpleTime::format_default`] produces `"hh:mm tt"` in 12-hour mode and
//! `"HH:mm"` in 24-hour mode, matching MudBlazor's documented defaults
//! [reference:5].

use std::fmt;

/// Seconds in a day.
pub const SECONDS_PER_DAY: u32 = 86_400;
/// Minutes in a day.
pub const MINUTES_PER_DAY: u32 = 1_440;

/// A time of day (proleptic, no timezone).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct SimpleTime {
    /// Hour, `0..=23`.
    pub hours: u8,
    /// Minute, `0..=59`.
    pub minutes: u8,
    /// Second, `0..=59`.
    pub seconds: u8,
}

impl SimpleTime {
    /// Creates a time, clamping every field into its range.
    pub fn new(hours: u8, minutes: u8, seconds: u8) -> Self {
        Self {
            hours: hours.min(23),
            minutes: minutes.min(59),
            seconds: seconds.min(59),
        }
    }

    /// Creates a time from hours and minutes; seconds are `0`.
    #[inline]
    pub fn hm(hours: u8, minutes: u8) -> Self {
        Self::new(hours, minutes, 0)
    }

    /// Midnight.
    #[inline]
    pub fn midnight() -> Self {
        Self::new(0, 0, 0)
    }

    /// `true` when the hour is before noon.
    #[inline]
    pub fn is_am(&self) -> bool {
        self.hours < 12
    }

    /// `true` when the hour is noon or later.
    #[inline]
    pub fn is_pm(&self) -> bool {
        !self.is_am()
    }

    /// Hour on a 12-hour clock, `1..=12` (`0` and `12` both map to `12`).
    #[inline]
    pub fn hour_12(&self) -> u8 {
        match self.hours % 12 {
            0 => 12,
            h => h,
        }
    }

    /// Total minutes since midnight.
    #[inline]
    pub fn total_minutes(&self) -> u16 {
        (self.hours as u16) * 60 + self.minutes as u16
    }

    /// Builds a time from minutes since midnight (`0..=1439`).
    #[inline]
    pub fn from_total_minutes(total: u16) -> Self {
        let total = total % MINUTES_PER_DAY as u16;
        Self::hm((total / 60) as u8, (total % 60) as u8)
    }

    /// Returns `self` shifted by `delta` minutes, wrapping at midnight.
    pub fn add_minutes(self, delta: i32) -> Self {
        let total = (self.total_minutes() as i32 + delta)
            .rem_euclid(MINUTES_PER_DAY as i32) as u16;
        Self::from_total_minutes(total)
    }

    /// Replaces the hour, keeping minutes and seconds.
    #[inline]
    pub fn with_hours(self, hours: u8) -> Self {
        Self::new(hours, self.minutes, self.seconds)
    }

    /// Replaces the minute, keeping hours and seconds.
    #[inline]
    pub fn with_minutes(self, minutes: u8) -> Self {
        Self::new(self.hours, minutes, self.seconds)
    }

    /// Flips AM/PM, keeping the minute and second.
    #[inline]
    pub fn toggle_am_pm(self) -> Self {
        let h = if self.hours >= 12 { self.hours - 12 } else { self.hours + 12 };
        self.with_hours(h)
    }

    /// `HH:MM` (24-hour, zero-padded).
    pub fn format_hhmm(&self) -> String {
        format!("{:02}:{:02}", self.hours, self.minutes)
    }

    /// The default display text: `hh:mm tt` in 12-hour mode, `HH:mm`
    /// otherwise — MudBlazor's documented defaults[reference:6].
    pub fn format_default(&self, am_pm: bool) -> String {
        if am_pm {
            self.format("hh:mm tt")
        } else {
            self.format_hhmm()
        }
    }

    /// Evaluates a MudBlazor-style `TimeFormat` template.
    ///
    /// Recognised tokens, longest match first:
    ///
    /// | Token | Output |
    /// |-------|--------|
    /// | `HH`  | 24-hour, zero-padded (`00`..`23`) |
    /// | `H`   | 24-hour (`0`..`23`) |
    /// | `hh`  | 12-hour, zero-padded (`01`..`12`) |
    /// | `h`   | 12-hour (`1`..`12`) |
    /// | `mm`  | minutes, zero-padded |
    /// | `m`   | minutes |
    /// | `ss`  | seconds, zero-padded |
    /// | `s`   | seconds |
    /// | `tt`  | `AM` / `PM` |
    ///
    /// Unknown characters are copied verbatim, so `"HH:mm"`, `"h:mm tt"` and
    /// `"HH时mm分"` all round-trip. An empty template falls back to
    /// [`format_default`](Self::format_default).
    pub fn format(&self, template: &str) -> String {
        if template.is_empty() {
            return self.format_default(self.is_am());
        }
        let mut out = String::with_capacity(template.len() + 4);
        let bytes = template.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            let rest = &template[i..];
            if rest.starts_with("HH") {
                out.push_str(&format!("{:02}", self.hours));
                i += 2;
            } else if rest.starts_with("hh") {
                out.push_str(&format!("{:02}", self.hour_12()));
                i += 2;
            } else if rest.starts_with("mm") {
                out.push_str(&format!("{:02}", self.minutes));
                i += 2;
            } else if rest.starts_with("ss") {
                out.push_str(&format!("{:02}", self.seconds));
                i += 2;
            } else if rest.starts_with("tt") {
                out.push_str(if self.is_am() { "AM" } else { "PM" });
                i += 2;
            } else if rest.starts_with('H') {
                out.push_str(&self.hours.to_string());
                i += 1;
            } else if rest.starts_with('h') {
                out.push_str(&self.hour_12().to_string());
                i += 1;
            } else if rest.starts_with('m') {
                out.push_str(&self.minutes.to_string());
                i += 1;
            } else if rest.starts_with('s') {
                out.push_str(&self.seconds.to_string());
                i += 1;
            } else {
                let ch = rest.chars().next().unwrap();
                out.push(ch);
                i += ch.len_utf8();
            }
        }
        out
    }
}

impl fmt::Display for SimpleTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.format_hhmm())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- construction -----------------------------------------------------

    #[test]
    fn new_clamps_every_field() {
        let t = SimpleTime::new(99, 99, 99);
        assert_eq!((t.hours, t.minutes, t.seconds), (23, 59, 59));
        let t = SimpleTime::new(0, 0, 0);
        assert_eq!((t.hours, t.minutes, t.seconds), (0, 0, 0));
    }

    #[test]
    fn hm_sets_zero_seconds() {
        let t = SimpleTime::hm(9, 30);
        assert_eq!(t.seconds, 0);
        assert_eq!((t.hours, t.minutes), (9, 30));
    }

    // --- AM / PM ----------------------------------------------------------

    #[test]
    fn is_am_pm_splits_at_noon() {
        assert!(SimpleTime::hm(0, 0).is_am());
        assert!(SimpleTime::hm(11, 59).is_am());
        assert!(SimpleTime::hm(12, 0).is_pm());
        assert!(SimpleTime::hm(23, 59).is_pm());
    }

    #[test]
    fn hour_12_maps_midnight_and_noon_to_twelve() {
        assert_eq!(SimpleTime::hm(0, 0).hour_12(), 12);
        assert_eq!(SimpleTime::hm(12, 0).hour_12(), 12);
        assert_eq!(SimpleTime::hm(1, 0).hour_12(), 1);
        assert_eq!(SimpleTime::hm(13, 0).hour_12(), 1);
        assert_eq!(SimpleTime::hm(23, 0).hour_12(), 11);
    }

    // --- minutes arithmetic -----------------------------------------------

    #[test]
    fn total_minutes_round_trips() {
        for h in [0u8, 9, 12, 23] {
            for m in [0u8, 15, 45, 59] {
                let t = SimpleTime::hm(h, m);
                assert_eq!(SimpleTime::from_total_minutes(t.total_minutes()), t);
            }
        }
    }

    #[test]
    fn add_minutes_wraps_at_midnight() {
        assert_eq!(SimpleTime::hm(23, 30).add_minutes(45), SimpleTime::hm(0, 15));
        assert_eq!(SimpleTime::hm(0, 15).add_minutes(-45), SimpleTime::hm(23, 30));
        assert_eq!(SimpleTime::hm(12, 0).add_minutes(1440), SimpleTime::hm(12, 0));
    }

    // --- with_* -----------------------------------------------------------

    #[test]
    fn with_hours_and_minutes_keep_the_rest() {
        let t = SimpleTime::new(9, 30, 15);
        assert_eq!(t.with_hours(17), SimpleTime::new(17, 30, 15));
        assert_eq!(t.with_minutes(45), SimpleTime::new(9, 45, 15));
    }

    #[test]
    fn toggle_am_pm_flips_by_twelve() {
        assert_eq!(SimpleTime::hm(9, 15).toggle_am_pm(), SimpleTime::hm(21, 15));
        assert_eq!(SimpleTime::hm(21, 15).toggle_am_pm(), SimpleTime::hm(9, 15));
        assert_eq!(SimpleTime::hm(0, 0).toggle_am_pm(), SimpleTime::hm(12, 0));
        assert_eq!(SimpleTime::hm(12, 0).toggle_am_pm(), SimpleTime::hm(0, 0));
    }

    // --- formatting -------------------------------------------------------

    #[test]
    fn format_hhmm_is_padded() {
        assert_eq!(SimpleTime::hm(9, 5).format_hhmm(), "09:05");
        assert_eq!(SimpleTime::hm(23, 59).format_hhmm(), "23:59");
    }

    #[test]
    fn format_default_uses_am_pm_flag() {
        assert_eq!(SimpleTime::hm(18, 32).format_default(false), "18:32");
        assert_eq!(SimpleTime::hm(18, 32).format_default(true), "06:32 PM");
        assert_eq!(SimpleTime::hm(6, 5).format_default(true), "06:05 AM");
    }

    #[test]
    fn format_tokens_substitute() {
        let t = SimpleTime::new(18, 32, 7);
        assert_eq!(t.format("HH:mm"), "18:32");
        assert_eq!(t.format("H:m"), "18:32");
        assert_eq!(t.format("h:mm tt"), "6:32 PM");
        assert_eq!(t.format("hh:mm:ss tt"), "06:32:07 PM");
        assert_eq!(t.format("H时m分"), "18时32分");
        assert_eq!(t.format(""), "18:32");
    }

    #[test]
    fn display_uses_24_hour() {
        assert_eq!(SimpleTime::hm(9, 5).to_string(), "09:05");
    }
}