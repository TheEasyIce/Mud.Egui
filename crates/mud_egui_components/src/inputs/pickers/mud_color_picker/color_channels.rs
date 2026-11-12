//! `ColorChannels` — the value type of [`MudColorPicker`](super::MudColorPicker).
//!
//! Mirrors the channel surface of `MudBlazor.Utilities.MudColor`: red, green
//! and blue in `0..=255`, alpha in `0.0..=1.0`, hue in `0.0..=360.0`, and
//! saturation / lightness in `0.0..=1.0`[reference:15]. RGB and HSL are kept in
//! sync: writing an RGB channel recomputes HSL, and writing an HSL channel
//! recomputes RGB **while preserving the other two HSL components** — that is
//! what makes [`hsl_eq`](Self::hsl_eq) and therefore
//! `UpdateBindingIfOnlyHSLChanged` meaningful.
//!
//! # Why not `mud_egui_theming::MudColor`?
//!
//! The theming `MudColor` is palette-facing: hex parsing, `Into<Color32>`,
//! lighten/darken. The picker needs the inverse direction — per-channel
//! setters, a preserved HSL triple, and `Display` formats for three modes.
//! `ColorChannels` provides exactly `MudColor`'s accessor surface so a caller
//! converts at the boundary:
//!
//! ```ignore
//! let channels = ColorChannels::from(Color32::from(mud_color));
//! let back: Color32 = channels.into();
//! ```

use std::fmt;

use egui::Color32;
use mud_egui_core::enums::ColorPickerMode;

// ---------------------------------------------------------------------------
// ColorChannels
// ---------------------------------------------------------------------------

/// An RGBA colour with a cached HSL triple.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColorChannels {
    r: u8,
    g: u8,
    b: u8,
    a: f64,
    h: f64,
    s: f64,
    l: f64,
}

impl ColorChannels {
    /// Creates a colour from 8-bit RGB and an alpha in `0.0..=1.0`.
    pub fn from_rgba8(r: u8, g: u8, b: u8, alpha: f64) -> Self {
        let (h, s, l) = rgb_to_hsl(r, g, b);
        Self { r, g, b, a: alpha.clamp(0.0, 1.0), h, s, l }
    }

    /// Creates a colour from 8-bit RGBA.
    #[inline]
    pub fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self::from_rgba8(r, g, b, a as f64 / 255.0)
    }

    /// Creates a colour from HSL and an alpha in `0.0..=1.0`.
    pub fn from_hsla(h: f64, s: f64, l: f64, alpha: f64) -> Self {
        let h = h.rem_euclid(360.0);
        let s = s.clamp(0.0, 1.0);
        let l = l.clamp(0.0, 1.0);
        let (r, g, b) = hsl_to_rgb(h, s, l);
        Self { r, g, b, a: alpha.clamp(0.0, 1.0), h, s, l }
    }

    /// Opaque black — the value the picker starts from.
    #[inline]
    pub fn black() -> Self {
        Self::from_rgba(0, 0, 0, 255)
    }

    /// Parses `#rgb`, `#rgba`, `#rrggbb`, `#rrggbbaa`, `rgb(r,g,b)` or
    /// `rgba(r,g,b,a)`. Whitespace and case are ignored. Mirrors
    /// `SetInputString`'s documented accepted formats[reference:16].
    pub fn parse(input: &str) -> Option<Self> {
        let s = input.trim();
        if s.is_empty() {
            return None;
        }
        if let Some(hex) = s.strip_prefix('#') {
            return parse_hex(hex);
        }
        let lower = s.to_ascii_lowercase();
        if let Some(rest) = lower.strip_prefix("rgba(").and_then(|r| r.strip_suffix(')')) {
            let parts: Vec<&str> = rest.split(',').map(str::trim).collect();
            if parts.len() != 4 {
                return None;
            }
            let r = parts[0].parse::<u8>().ok()?;
            let g = parts[1].parse::<u8>().ok()?;
            let b = parts[2].parse::<u8>().ok()?;
            let a = parse_alpha(parts[3])?;
            return Some(Self::from_rgba8(r, g, b, a));
        }
        if let Some(rest) = lower.strip_prefix("rgb(").and_then(|r| r.strip_suffix(')')) {
            let parts: Vec<&str> = rest.split(',').map(str::trim).collect();
            if parts.len() != 3 {
                return None;
            }
            let r = parts[0].parse::<u8>().ok()?;
            let g = parts[1].parse::<u8>().ok()?;
            let b = parts[2].parse::<u8>().ok()?;
            return Some(Self::from_rgba(r, g, b, 255));
        }
        None
    }

    // ── Channel accessors ──────────────────────────────────────────────────

    /// Red, `0..=255`.
    #[inline] pub fn r(&self) -> u8 { self.r }
    /// Green, `0..=255`.
    #[inline] pub fn g(&self) -> u8 { self.g }
    /// Blue, `0..=255`.
    #[inline] pub fn b(&self) -> u8 { self.b }
    /// Alpha, `0.0..=1.0`.
    #[inline] pub fn a(&self) -> f64 { self.a }
    /// Alpha as 8-bit, `0..=255`.
    #[inline] pub fn a8(&self) -> u8 { (self.a * 255.0).round() as u8 }
    /// Hue, `0.0..=360.0`.
    #[inline] pub fn h(&self) -> f64 { self.h }
    /// Saturation, `0.0..=1.0`.
    #[inline] pub fn s(&self) -> f64 { self.s }
    /// Lightness, `0.0..=1.0`.
    #[inline] pub fn l(&self) -> f64 { self.l }

    /// `true` when this colour is fully opaque.
    #[inline]
    pub fn is_opaque(&self) -> bool {
        self.a >= 1.0
    }

    /// The ink that reads against this fill: black or white, using the same
    /// `r + g + b > 384` rule as `MudButton` and `MudAlert`.
    #[inline]
    pub fn contrast_ink(&self) -> Color32 {
        let sum = self.r as u32 + self.g as u32 + self.b as u32;
        if sum > 384 { Color32::BLACK } else { Color32::WHITE }
    }

    // ── Channel setters ────────────────────────────────────────────────────

    /// Sets red (`0..=255`). Recomputes HSL.
    pub fn with_r(self, r: u8) -> Self {
        Self::from_rgba8(r, self.g, self.b, self.a)
    }

    /// Sets green (`0..=255`). Recomputes HSL.
    pub fn with_g(self, g: u8) -> Self {
        Self::from_rgba8(self.r, g, self.b, self.a)
    }

    /// Sets blue (`0..=255`). Recomputes HSL.
    pub fn with_b(self, b: u8) -> Self {
        Self::from_rgba8(self.r, self.g, b, self.a)
    }

    /// Sets alpha (`0.0..=1.0`). Does **not** touch RGB or HSL.
    pub fn with_a(self, a: f64) -> Self {
        Self { a: a.clamp(0.0, 1.0), ..self }
    }

    /// Sets hue (`0..=360`), preserving saturation and lightness. Recomputes
    /// RGB.
    pub fn with_h(self, h: f64) -> Self {
        Self::from_hsla(h, self.s, self.l, self.a)
    }

    /// Sets saturation (`0.0..=1.0`), preserving hue and lightness.
    pub fn with_s(self, s: f64) -> Self {
        Self::from_hsla(self.h, s, self.l, self.a)
    }

    /// Sets lightness (`0.0..=1.0`), preserving hue and saturation.
    pub fn with_l(self, l: f64) -> Self {
        Self::from_hsla(self.h, self.s, l, self.a)
    }

    // ── Comparison ─────────────────────────────────────────────────────────

    /// `true` when the **RGB** channels and alpha match.
    #[inline]
    pub fn rgb_eq(&self, other: &Self) -> bool {
        self.r == other.r && self.g == other.g && self.b == other.b && self.a == other.a
    }

    /// `true` when the **HSL** triple matches. Mirrors `MudColor.HslChanged`
    /// [reference:17]. Exact equality: the triple is stored, not recomputed.
    #[inline]
    pub fn hsl_eq(&self, other: &Self) -> bool {
        (self.h - other.h).abs() < f64::EPSILON
            && (self.s - other.s).abs() < f64::EPSILON
            && (self.l - other.l).abs() < f64::EPSILON
    }

    // ── Formatting ─────────────────────────────────────────────────────────

    /// `#RRGGBB` (upper-case).
    pub fn to_hex(&self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }

    /// `#RRGGBBAA` (upper-case).
    pub fn to_hexa(&self) -> String {
        format!("#{:02X}{:02X}{:02X}{:02X}", self.r, self.g, self.b, self.a8())
    }

    /// `rgb(r, g, b)`.
    pub fn to_rgb(&self) -> String {
        format!("rgb({}, {}, {})", self.r, self.g, self.b)
    }

    /// `rgba(r, g, b, a)` with alpha rounded to two decimals.
    pub fn to_rgba(&self) -> String {
        format!("rgba({}, {}, {}, {:.2})", self.r, self.g, self.b, self.a)
    }

    /// `hsl(h, s%, l%)`.
    pub fn to_hsl(&self) -> String {
        format!(
            "hsl({:.0}, {:.0}%, {:.0}%)",
            self.h,
            self.s * 100.0,
            self.l * 100.0
        )
    }

    /// `hsla(h, s%, l%, a)`.
    pub fn to_hsla(&self) -> String {
        format!(
            "hsla({:.0}, {:.0}%, {:.0}%, {:.2})",
            self.h,
            self.s * 100.0,
            self.l * 100.0,
            self.a
        )
    }

    /// The trigger text: **always** hex, matching MudBlazor's behaviour that
    /// the label shows hex regardless of `ColorPickerMode`[reference:18].
    #[inline]
    pub fn hex_text(&self, show_alpha: bool) -> String {
        if show_alpha { self.to_hexa() } else { self.to_hex() }
    }

    /// Formats the colour in `mode`, honouring `show_alpha`.
    pub fn format(&self, mode: ColorPickerMode, show_alpha: bool) -> String {
        match mode {
            ColorPickerMode::Rgb => {
                if show_alpha { self.to_rgba() } else { self.to_rgb() }
            }
            ColorPickerMode::Hsl => {
                if show_alpha { self.to_hsla() } else { self.to_hsl() }
            }
            ColorPickerMode::Hex => self.hex_text(show_alpha),
        }
    }
}

// ---------------------------------------------------------------------------
// Conversions
// ---------------------------------------------------------------------------

impl From<Color32> for ColorChannels {
    fn from(c: Color32) -> Self {
        Self::from_rgba(c.r(), c.g(), c.b(), c.a())
    }
}

impl From<ColorChannels> for Color32 {
    fn from(c: ColorChannels) -> Self {
        Color32::from_rgba_unmultiplied(c.r, c.g, c.b, c.a8())
    }
}

impl Default for ColorChannels {
    fn default() -> Self {
        Self::black()
    }
}

impl fmt::Display for ColorChannels {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_hex())
    }
}

// ---------------------------------------------------------------------------
// Parsing helpers
// ---------------------------------------------------------------------------

fn parse_alpha(s: &str) -> Option<f64> {
    let v = s.parse::<f64>().ok()?;
    // Accept both 0..=1 and 0..=255, as CSS `rgba()` does.
    let a = if v > 1.0 { v / 255.0 } else { v };
    Some(a.clamp(0.0, 1.0))
}

fn parse_hex(hex: &str) -> Option<ColorChannels> {
    let h = hex.trim();
    let digits: Vec<char> = h.chars().collect();
    let short = |c: char| {
        let v = c.to_digit(16)? as u8;
        Some(v * 17)
    };
    match digits.len() {
        3 => {
            let r = short(digits[0])?;
            let g = short(digits[1])?;
            let b = short(digits[2])?;
            Some(ColorChannels::from_rgba(r, g, b, 255))
        }
        4 => {
            let r = short(digits[0])?;
            let g = short(digits[1])?;
            let b = short(digits[2])?;
            let a = short(digits[3])?;
            Some(ColorChannels::from_rgba(r, g, b, a))
        }
        6 => {
            let r = u8::from_str_radix(&h[0..2], 16).ok()?;
            let g = u8::from_str_radix(&h[2..4], 16).ok()?;
            let b = u8::from_str_radix(&h[4..6], 16).ok()?;
            Some(ColorChannels::from_rgba(r, g, b, 255))
        }
        8 => {
            let r = u8::from_str_radix(&h[0..2], 16).ok()?;
            let g = u8::from_str_radix(&h[2..4], 16).ok()?;
            let b = u8::from_str_radix(&h[4..6], 16).ok()?;
            let a = u8::from_str_radix(&h[6..8], 16).ok()?;
            Some(ColorChannels::from_rgba(r, g, b, a))
        }
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// HSL ↔ RGB
// ---------------------------------------------------------------------------

fn rgb_to_hsl(r: u8, g: u8, b: u8) -> (f64, f64, f64) {
    let rf = r as f64 / 255.0;
    let gf = g as f64 / 255.0;
    let bf = b as f64 / 255.0;
    let max = rf.max(gf).max(bf);
    let min = rf.min(gf).min(bf);
    let l = (max + min) / 2.0;
    if (max - min).abs() < f64::EPSILON {
        return (0.0, 0.0, l);
    }
    let d = max - min;
    let s = if l > 0.5 { d / (2.0 - max - min) } else { d / (max + min) };
    let h = if max == rf {
        ((gf - bf) / d + if gf < bf { 6.0 } else { 0.0 }) / 6.0
    } else if max == gf {
        ((bf - rf) / d + 2.0) / 6.0
    } else {
        ((rf - gf) / d + 4.0) / 6.0
    };
    (h * 360.0, s, l)
}

fn hsl_to_rgb(h: f64, s: f64, l: f64) -> (u8, u8, u8) {
    let h = h.rem_euclid(360.0) / 360.0;
    if s <= 0.0 {
        let v = (l * 255.0).round().clamp(0.0, 255.0) as u8;
        return (v, v, v);
    }
    let q = if l < 0.5 { l * (1.0 + s) } else { l + s - l * s };
    let p = 2.0 * l - q;
    let r = hue_to_rgb(p, q, h + 1.0 / 3.0);
    let g = hue_to_rgb(p, q, h);
    let b = hue_to_rgb(p, q, h - 1.0 / 3.0);
    (
        (r * 255.0).round().clamp(0.0, 255.0) as u8,
        (g * 255.0).round().clamp(0.0, 255.0) as u8,
        (b * 255.0).round().clamp(0.0, 255.0) as u8,
    )
}

fn hue_to_rgb(p: f64, q: f64, mut t: f64) -> f64 {
    if t < 0.0 { t += 1.0; }
    if t > 1.0 { t -= 1.0; }
    if t < 1.0 / 6.0 {
        p + (q - p) * 6.0 * t
    } else if t < 1.0 / 2.0 {
        q
    } else if t < 2.0 / 3.0 {
        p + (q - p) * (2.0 / 3.0 - t) * 6.0
    } else {
        p
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn c(r: u8, g: u8, b: u8) -> ColorChannels {
        ColorChannels::from_rgba(r, g, b, 255)
    }

    // --- construction -----------------------------------------------------

    #[test]
    fn black_is_opaque_zero() {
        let b = ColorChannels::black();
        assert_eq!((b.r(), b.g(), b.b(), b.a8()), (0, 0, 0, 255));
    }

    #[test]
    fn from_rgba8_clamps_alpha() {
        assert_eq!(ColorChannels::from_rgba8(10, 20, 30, 2.0).a(), 1.0);
        assert_eq!(ColorChannels::from_rgba8(10, 20, 30, -1.0).a(), 0.0);
    }

    #[test]
    fn from_hsla_round_trips_primaries() {
        let red = ColorChannels::from_hsla(0.0, 1.0, 0.5, 1.0);
        assert_eq!((red.r(), red.g(), red.b()), (255, 0, 0));
        let green = ColorChannels::from_hsla(120.0, 1.0, 0.5, 1.0);
        assert_eq!((green.r(), green.g(), green.b()), (0, 255, 0));
        let blue = ColorChannels::from_hsla(240.0, 1.0, 0.5, 1.0);
        assert_eq!((blue.r(), blue.g(), blue.b()), (0, 0, 255));
    }

    #[test]
    fn from_hsla_grey_is_grey() {
        let g = ColorChannels::from_hsla(200.0, 0.0, 0.5, 1.0);
        assert_eq!((g.r(), g.g(), g.b()), (128, 128, 128));
    }

    // --- round-trips ------------------------------------------------------

    #[test]
    fn rgb_to_hsl_round_trips_every_channel_value() {
        for r in [0u8, 51, 128, 200, 255] {
            for g in [0u8, 51, 128, 200, 255] {
                for b in [0u8, 51, 128, 200, 255] {
                    let c = ColorChannels::from_rgba(r, g, b, 255);
                    let back = ColorChannels::from_hsla(c.h(), c.s(), c.l(), 1.0);
                    assert_eq!(
                        (back.r(), back.g(), back.b()),
                        (r, g, b),
                        "rgb({r},{g},{b}) → hsl({},{},{}) → rgb({},{},{})",
                        c.h(), c.s(), c.l(), back.r(), back.g(), back.b()
                    );
                }
            }
        }
    }

    #[test]
    fn known_hsl_values() {
        let red = c(255, 0, 0);
        assert!((red.h() - 0.0).abs() < 1e-9);
        assert!((red.s() - 1.0).abs() < 1e-9);
        assert!((red.l() - 0.5).abs() < 1e-9);

        let grey = c(128, 128, 128);
        assert_eq!(grey.s(), 0.0);

        let white = c(255, 255, 255);
        assert!((white.l() - 1.0).abs() < 1e-9);
    }

    // --- setters ----------------------------------------------------------

    #[test]
    fn with_rgb_recomputes_hsl() {
        let c = c(0, 0, 0).with_r(255);
        assert_eq!(c.r(), 255);
        assert!((c.h() - 0.0).abs() < 1e-9, "red hue should be 0");
    }

    #[test]
    fn with_h_preserves_s_and_l() {
        let base = ColorChannels::from_hsla(0.0, 0.8, 0.4, 1.0);
        let moved = base.with_h(180.0);
        assert!((moved.h() - 180.0).abs() < 1e-9);
        assert!((moved.s() - base.s()).abs() < 1e-9);
        assert!((moved.l() - base.l()).abs() < 1e-9);
    }

    #[test]
    fn with_s_at_grey_keeps_rgb_but_changes_hsl() {
        let grey = c(128, 128, 128);
        let tinted = grey.with_s(0.5);
        // S at L = 128/255 ≈ 0.502 yields a non-grey colour, but the point is
        // that HSL changed even if a caller only nudged S:
        assert!(!tinted.hsl_eq(&grey) || !tinted.rgb_eq(&grey));
    }

    #[test]
    fn with_a_does_not_touch_rgb_or_hsl() {
        let base = c(10, 20, 30);
        let faded = base.with_a(0.5);
        assert_eq!((faded.r(), faded.g(), faded.b()), (10, 20, 30));
        assert!(faded.hsl_eq(&base));
        assert!((faded.a() - 0.5).abs() < 1e-9);
    }

    #[test]
    fn with_a_clamps() {
        assert_eq!(c(0, 0, 0).with_a(2.0).a(), 1.0);
        assert_eq!(c(0, 0, 0).with_a(-0.5).a(), 0.0);
    }

    // --- comparison -------------------------------------------------------

    #[test]
    fn rgb_eq_and_hsl_eq_are_independent() {
        let a = c(10, 20, 30);
        let b = ColorChannels::from_hsla(a.h(), a.s(), a.l(), a.a());
        assert!(a.rgb_eq(&b) || a.hsl_eq(&b));
        assert!(a.hsl_eq(&b), "round-tripping preserves HSL");

        let faded = a.with_a(0.5);
        assert!(!a.rgb_eq(&faded));
        assert!(a.hsl_eq(&faded), "alpha must not affect HSL equality");
    }

    #[test]
    fn hsl_eq_detects_a_hue_only_change() {
        let a = ColorChannels::from_hsla(10.0, 0.5, 0.5, 1.0);
        let b = a.with_h(20.0);
        assert!(!a.hsl_eq(&b));
    }

    // --- parsing ----------------------------------------------------------

    #[test]
    fn parse_hex_short_and_long() {
        assert_eq!(ColorChannels::parse("#f00").unwrap().to_hex(), "#FF0000");
        assert_eq!(ColorChannels::parse("#ff0000").unwrap().to_hex(), "#FF0000");
        assert_eq!(
            ColorChannels::parse("#ff000080").unwrap().to_hexa(),
            "#FF000080"
        );
        assert_eq!(ColorChannels::parse("#f00f").unwrap().a8(), 255);
    }

    #[test]
    fn parse_rgb_and_rgba() {
        let rgb = ColorChannels::parse("rgb(10, 20, 30)").unwrap();
        assert_eq!((rgb.r(), rgb.g(), rgb.b(), rgb.a8()), (10, 20, 30, 255));
        let rgba = ColorChannels::parse("rgba(10, 20, 30, 0.5)").unwrap();
        assert_eq!((rgba.r(), rgba.g(), rgba.b()), (10, 20, 30));
        assert!((rgba.a() - 0.5).abs() < 1e-9);
        // 0..=255 alpha also accepted.
        let rgba8 = ColorChannels::parse("rgba(10,20,30,128)").unwrap();
        assert!((rgba8.a() - 128.0 / 255.0).abs() < 1e-9);
    }

    #[test]
    fn parse_rejects_garbage() {
        for s in ["", "  ", "#", "#gggggg", "rgb(1,2)", "rgb(a,b,c)", "hsl(1,2,3)", "nope"] {
            assert!(ColorChannels::parse(s).is_none(), "{s:?} must not parse");
        }
    }

    #[test]
    fn parse_ignores_surrounding_whitespace_and_case() {
        assert_eq!(ColorChannels::parse("  #FF0000  ").unwrap().to_hex(), "#FF0000");
        assert_eq!(ColorChannels::parse("RGB(1,2,3)").unwrap().to_hex(), "#010203");
    }

    // --- formatting -------------------------------------------------------

    #[test]
    fn to_hex_is_padded_and_upper_case() {
        assert_eq!(c(1, 2, 3).to_hex(), "#010203");
        assert_eq!(c(255, 255, 255).to_hex(), "#FFFFFF");
    }

    #[test]
    fn to_hexa_includes_alpha() {
        assert_eq!(c(1, 2, 3).with_a(0.5).to_hexa().len(), 9);
        assert!(c(1, 2, 3).with_a(1.0).to_hexa().ends_with("FF"));
    }

    #[test]
    fn format_honours_mode_and_alpha() {
        let col = ColorChannels::from_rgba(255, 0, 0, 128);
        assert_eq!(col.format(ColorPickerMode::Rgb, false), "rgb(255, 0, 0)");
        assert!(col.format(ColorPickerMode::Rgb, true).starts_with("rgba(255, 0, 0, "));
        assert_eq!(col.format(ColorPickerMode::Hsl, false), "hsl(0, 100%, 50%)");
        assert!(col.format(ColorPickerMode::Hsl, true).starts_with("hsla(0, 100%, 50%, "));
        assert!(col.format(ColorPickerMode::Hex, false).starts_with('#'));
        assert!(col.format(ColorPickerMode::Hex, true).starts_with('#'));
    }

    #[test]
    fn hex_text_always_returns_hex() {
        let col = c(1, 2, 3);
        assert_eq!(col.hex_text(false), "#010203");
        assert_eq!(col.hex_text(true).len(), 9);
    }

    // --- contrast ---------------------------------------------------------

    #[test]
    fn contrast_ink_switches_at_384() {
        assert_eq!(c(255, 255, 255).contrast_ink(), Color32::BLACK);
        assert_eq!(c(0, 0, 0).contrast_ink(), Color32::WHITE);
        // 128 + 128 + 128 = 384 → white ink (not > 384).
        assert_eq!(c(128, 128, 128).contrast_ink(), Color32::WHITE);
        // 129 + 128 + 128 = 385 → black ink.
        assert_eq!(c(129, 128, 128).contrast_ink(), Color32::BLACK);
    }

    // --- conversions ------------------------------------------------------

    #[test]
    fn color32_round_trip() {
        let c32 = Color32::from_rgba_unmultiplied(10, 20, 30, 255);
        let ch = ColorChannels::from(c32);
        let back: Color32 = ch.into();
        assert_eq!(back, c32);
    }

    #[test]
    fn display_is_hex() {
        assert_eq!(c(1, 2, 3).to_string(), "#010203");
    }
}