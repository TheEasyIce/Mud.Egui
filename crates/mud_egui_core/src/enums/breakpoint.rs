//! Browser-width breakpoints.
//!
//! Port of `MudBlazor.Breakpoint` (MudBlazor 8.13.0). The 8.13.0 enum has
//! **sixteen** variants: six simple buckets (`Xs`..`Xxl`), eight compound ranges
//! (`SmAndDown`..`XlAndUp`), and two sentinels (`None`, `Always`).
//!
//! | Variant | Pixel range (points) | Meaning |
//! |---------|----------------------|---------|
//! | `Xs` | `< 600` | A small to large phone. |
//! | `Sm` | `[600, 960)` | A small to medium tablet. |
//! | `Md` | `[960, 1280)` | A large tablet or laptop. |
//! | `Lg` | `[1280, 1920)` | A desktop computer. |
//! | `Xl` | `[1920, 2560)` | A 4K desktop monitor. |
//! | `Xxl` | `>= 2560` | An ultra-wide 4K+ monitor. |
//! | `SmAndDown` | `< 960` | Sm or smaller. |
//! | `MdAndDown` | `< 1280` | Md or smaller. |
//! | `LgAndDown` | `< 1920` | Lg or smaller. |
//! | `XlAndDown` | `< 2560` | Xl or smaller. |
//! | `SmAndUp` | `>= 600` | Sm or larger. |
//! | `MdAndUp` | `>= 960` | Md or larger. |
//! | `LgAndUp` | `>= 1280` | Lg or larger. |
//! | `XlAndUp` | `>= 1920` | Xl or larger. |
//! | `None` | never | No breakpoint applies. |
//! | `Always` | always | The breakpoint applies at every width. |
//!
//! The simple breakpoints match **one bucket**; the compound breakpoints match
//! **one range**. That is the 8.13.0 distinction — `Lg` is "a desktop computer"
//! and `LgAndUp` is "a desktop computer, or larger device" — and it is why both
//! sets exist. Earlier MudBlazor versions treated a simple breakpoint as
//! "and up"; 8.13.0 lets you be explicit.
//!
//! Two behaviours are worth calling out:
//!
//! - [`Breakpoint::to_pixels`] returns the *lower bound* of the named bucket
//!   (`Xs`→0, `Sm`→600, …) or of the `*AndUp` range (`SmAndUp`→600, …). For a
//!   `*AndDown` range it returns the *upper bound* (`SmAndDown`→960, …), because
//!   `*AndDown` means "up to and including this bucket". This is the value
//!   consumers that treat the enum as a "collapse below this width" threshold
//!   (e.g. `MudStack::breakpoint`) already expect.
//! - [`Breakpoint::matches`] returns whether a given width falls inside the
//!   breakpoint's range. `None` matches nothing; `Always` matches everything,
//!   even a non-finite width.

/// Smallest width of the `Sm` bucket, in logical points.
pub const SM_PX: f32 = 600.0;
/// Smallest width of the `Md` bucket, in logical points.
pub const MD_PX: f32 = 960.0;
/// Smallest width of the `Lg` bucket, in logical points.
pub const LG_PX: f32 = 1280.0;
/// Smallest width of the `Xl` bucket, in logical points.
pub const XL_PX: f32 = 1920.0;
/// Smallest width of the `Xxl` bucket, in logical points.
pub const XXL_PX: f32 = 2560.0;

/// A browser-width bucket used to trigger responsive behaviour.
///
/// Mirrors `MudBlazor.Breakpoint`. The default is [`Breakpoint::None`], which is
/// the value MudBlazor documents for an unconfigured cascading breakpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Breakpoint {
    /// A small to large phone. `width < 600`.
    Xs,
    /// A small to medium tablet. `[600, 960)`.
    Sm,
    /// A large tablet or laptop. `[960, 1280)`.
    Md,
    /// A desktop computer. `[1280, 1920)`.
    Lg,
    /// A high-definition or 4K desktop monitor. `[1920, 2560)`.
    Xl,
    /// An ultra-wide 4K+ monitor. `width >= 2560`.
    Xxl,
    /// Sm or smaller. `width < 960`.
    SmAndDown,
    /// Md or smaller. `width < 1280`.
    MdAndDown,
    /// Lg or smaller. `width < 1920`.
    LgAndDown,
    /// Xl or smaller. `width < 2560`.
    XlAndDown,
    /// Sm or larger. `width >= 600`.
    SmAndUp,
    /// Md or larger. `width >= 960`.
    MdAndUp,
    /// Lg or larger. `width >= 1280`.
    LgAndUp,
    /// Xl or larger. `width >= 1920`.
    XlAndUp,
    /// No breakpoint applies.
    #[default]
    None,
    /// The breakpoint applies at every width.
    Always,
}

impl Breakpoint {
    /// Every variant, in MudBlazor's declaration order.
    pub const ALL: [Breakpoint; 16] = [
        Breakpoint::Xs,
        Breakpoint::Sm,
        Breakpoint::Md,
        Breakpoint::Lg,
        Breakpoint::Xl,
        Breakpoint::Xxl,
        Breakpoint::SmAndDown,
        Breakpoint::MdAndDown,
        Breakpoint::LgAndDown,
        Breakpoint::XlAndDown,
        Breakpoint::SmAndUp,
        Breakpoint::MdAndUp,
        Breakpoint::LgAndUp,
        Breakpoint::XlAndUp,
        Breakpoint::None,
        Breakpoint::Always,
    ];

    /// Lower-case name, useful for logging and test ids.
    #[inline]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Xs => "xs",
            Self::Sm => "sm",
            Self::Md => "md",
            Self::Lg => "lg",
            Self::Xl => "xl",
            Self::Xxl => "xxl",
            Self::SmAndDown => "sm-and-down",
            Self::MdAndDown => "md-and-down",
            Self::LgAndDown => "lg-and-down",
            Self::XlAndDown => "xl-and-down",
            Self::SmAndUp => "sm-and-up",
            Self::MdAndUp => "md-and-up",
            Self::LgAndUp => "lg-and-up",
            Self::XlAndUp => "xl-and-up",
            Self::None => "none",
            Self::Always => "always",
        }
    }

    /// Human-friendly label, matching MudBlazor's enum doc comments.
    #[inline]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Xs => "A small to large phone",
            Self::Sm => "A small to medium tablet",
            Self::Md => "A large tablet or laptop",
            Self::Lg => "A desktop computer",
            Self::Xl => "A high-definition or 4K desktop monitor",
            Self::Xxl => "An ultra-wide 4K+ desktop monitor",
            Self::SmAndDown => "A small to medium tablet, or smaller device",
            Self::MdAndDown => "A large tablet, laptop, tablet, or smaller device",
            Self::LgAndDown => "A desktop computer or smaller device",
            Self::XlAndDown => "A 4K desktop monitor, or smaller device",
            Self::SmAndUp => "A small to medium tablet, or larger device",
            Self::MdAndUp => "A large tablet, laptop, or larger device",
            Self::LgAndUp => "A desktop computer, or larger device",
            Self::XlAndUp => "A 4K desktop monitor, or larger device",
            Self::None => "No breakpoint applies",
            Self::Always => "Content will always be visible",
        }
    }

    /// Whether this variant is a sentinel (`None` or `Always`).
    #[inline]
    pub const fn is_sentinel(self) -> bool {
        matches!(self, Self::None | Self::Always)
    }

    /// Whether this variant names a range (`*AndUp` / `*AndDown`) rather than a
    /// single bucket.
    #[inline]
    pub const fn is_compound(self) -> bool {
        matches!(
            self,
            Self::SmAndDown
                | Self::MdAndDown
                | Self::LgAndDown
                | Self::XlAndDown
                | Self::SmAndUp
                | Self::MdAndUp
                | Self::LgAndUp
                | Self::XlAndUp
        )
    }

    /// The pixel value consumers that treat this as a **threshold** should use.
    ///
    /// - Simple buckets and `*AndUp` ranges return their **lower bound**
    ///   (`Xs`→0, `Sm`/`SmAndUp`→600, …), so a `width >= to_pixels()` test means
    ///   "this bucket or larger", which is what `MudStack::breakpoint` wants.
    /// - `*AndDown` ranges return their **upper bound**
    ///   (`SmAndDown`→960, …), so `width >= to_pixels()` means "outside the
    ///   range" and `width < to_pixels()` means "inside it", matching the
    ///   "collapse below" reading.
    /// - The sentinels return `0.0`; `None` is filtered by callers, and `Always`
    ///   as a threshold never collapses anything.
    #[inline]
    pub const fn to_pixels(self) -> f32 {
        match self {
            Self::Xs | Self::None | Self::Always => 0.0,
            Self::Sm | Self::SmAndUp => SM_PX,
            Self::Md | Self::MdAndUp => MD_PX,
            Self::Lg | Self::LgAndUp => LG_PX,
            Self::Xl | Self::XlAndUp => XL_PX,
            Self::Xxl => XXL_PX,
            Self::SmAndDown => MD_PX,
            Self::MdAndDown => LG_PX,
            Self::LgAndDown => XL_PX,
            Self::XlAndDown => XXL_PX,
        }
    }

    /// Whether a viewport `width` falls inside this breakpoint's range.
    ///
    /// `None` matches nothing; `Always` matches every width, including a
    /// non-finite one. Every other variant treats a non-finite width as
    /// "outside the range".
    #[inline]
    pub fn matches(self, width: f32) -> bool {
        match self {
            Self::Always => return true,
            Self::None => return false,
            _ => {}
        }
        if !width.is_finite() {
            return false;
        }
        let w = width.max(0.0);
        match self {
            Self::Xs => w < SM_PX,
            Self::Sm => (SM_PX..MD_PX).contains(&w),
            Self::Md => (MD_PX..LG_PX).contains(&w),
            Self::Lg => (LG_PX..XL_PX).contains(&w),
            Self::Xl => (XL_PX..XXL_PX).contains(&w),
            Self::Xxl => w >= XXL_PX,
            Self::SmAndDown => w < MD_PX,
            Self::MdAndDown => w < LG_PX,
            Self::LgAndDown => w < XL_PX,
            Self::XlAndDown => w < XXL_PX,
            Self::SmAndUp => w >= SM_PX,
            Self::MdAndUp => w >= MD_PX,
            Self::LgAndUp => w >= LG_PX,
            Self::XlAndUp => w >= XL_PX,
            // Handled above; listing them keeps the match exhaustive.
            Self::None | Self::Always => false,
        }
    }

    /// The simple bucket a width falls into (`Xs`..`Xxl`).
    ///
    /// A non-finite width falls into `Xs`, which is also the value `0.0` would
    /// map to. Compound and sentinel variants are never returned.
    #[inline]
    pub fn from_pixels(width: f32) -> Breakpoint {
        if !width.is_finite() {
            return Breakpoint::Xs;
        }
        let w = width.max(0.0);
        if w < SM_PX {
            Breakpoint::Xs
        } else if w < MD_PX {
            Breakpoint::Sm
        } else if w < LG_PX {
            Breakpoint::Md
        } else if w < XL_PX {
            Breakpoint::Lg
        } else if w < XXL_PX {
            Breakpoint::Xl
        } else {
            Breakpoint::Xxl
        }
    }
}

impl core::fmt::Display for Breakpoint {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f32 = 1e-3;

    #[test]
    fn default_is_none() {
        assert_eq!(Breakpoint::default(), Breakpoint::None);
    }

    #[test]
    fn all_has_sixteen_unique_variants() {
        assert_eq!(Breakpoint::ALL.len(), 16);
        for (i, a) in Breakpoint::ALL.iter().enumerate() {
            for b in &Breakpoint::ALL[i + 1..] {
                assert_ne!(a, b);
            }
        }
    }

    #[test]
    fn names_are_lowercase_and_unique() {
        let mut seen = std::collections::BTreeSet::new();
        for b in Breakpoint::ALL {
            assert!(seen.insert(b.name()), "duplicate name: {}", b.name());
            assert_eq!(b.name(), b.name().to_lowercase());
        }
    }

    #[test]
    fn display_matches_name() {
        for b in Breakpoint::ALL {
            assert_eq!(b.to_string(), b.name());
        }
    }

    #[test]
    fn sentinel_classification() {
        assert!(Breakpoint::None.is_sentinel());
        assert!(Breakpoint::Always.is_sentinel());
        for b in Breakpoint::ALL {
            if !b.is_sentinel() {
                assert!(!b.is_sentinel());
            }
        }
    }

    #[test]
    fn compound_classification() {
        for b in [
            Breakpoint::SmAndDown,
            Breakpoint::MdAndDown,
            Breakpoint::LgAndDown,
            Breakpoint::XlAndDown,
            Breakpoint::SmAndUp,
            Breakpoint::MdAndUp,
            Breakpoint::LgAndUp,
            Breakpoint::XlAndUp,
        ] {
            assert!(b.is_compound(), "{} should be compound", b.name());
        }
        for b in [
            Breakpoint::Xs,
            Breakpoint::Sm,
            Breakpoint::Md,
            Breakpoint::Lg,
            Breakpoint::Xl,
            Breakpoint::Xxl,
            Breakpoint::None,
            Breakpoint::Always,
        ] {
            assert!(!b.is_compound(), "{} should not be compound", b.name());
        }
    }

    // --- to_pixels --------------------------------------------------------

    #[test]
    fn to_pixels_for_simple_buckets() {
        assert!((Breakpoint::Xs.to_pixels() - 0.0).abs() < EPS);
        assert!((Breakpoint::Sm.to_pixels() - SM_PX).abs() < EPS);
        assert!((Breakpoint::Md.to_pixels() - MD_PX).abs() < EPS);
        assert!((Breakpoint::Lg.to_pixels() - LG_PX).abs() < EPS);
        assert!((Breakpoint::Xl.to_pixels() - XL_PX).abs() < EPS);
        assert!((Breakpoint::Xxl.to_pixels() - XXL_PX).abs() < EPS);
    }

    #[test]
    fn to_pixels_for_and_up_uses_lower_bound() {
        assert!((Breakpoint::SmAndUp.to_pixels() - SM_PX).abs() < EPS);
        assert!((Breakpoint::MdAndUp.to_pixels() - MD_PX).abs() < EPS);
        assert!((Breakpoint::LgAndUp.to_pixels() - LG_PX).abs() < EPS);
        assert!((Breakpoint::XlAndUp.to_pixels() - XL_PX).abs() < EPS);
    }

    #[test]
    fn to_pixels_for_and_down_uses_upper_bound() {
        assert!((Breakpoint::SmAndDown.to_pixels() - MD_PX).abs() < EPS);
        assert!((Breakpoint::MdAndDown.to_pixels() - LG_PX).abs() < EPS);
        assert!((Breakpoint::LgAndDown.to_pixels() - XL_PX).abs() < EPS);
        assert!((Breakpoint::XlAndDown.to_pixels() - XXL_PX).abs() < EPS);
    }

    #[test]
    fn to_pixels_for_sentinels_is_zero() {
        assert_eq!(Breakpoint::None.to_pixels(), 0.0);
        assert_eq!(Breakpoint::Always.to_pixels(), 0.0);
    }

    // --- matches ----------------------------------------------------------

    #[test]
    fn sentinels_match_nothing_and_everything() {
        assert!(!Breakpoint::None.matches(0.0));
        assert!(!Breakpoint::None.matches(500.0));
        assert!(!Breakpoint::None.matches(f32::NAN));
        assert!(Breakpoint::Always.matches(0.0));
        assert!(Breakpoint::Always.matches(10_000.0));
        assert!(Breakpoint::Always.matches(f32::NAN));
        assert!(Breakpoint::Always.matches(f32::INFINITY));
    }

    #[test]
    fn simple_buckets_match_their_range_only() {
        // Xs: < 600
        assert!(Breakpoint::Xs.matches(0.0));
        assert!(Breakpoint::Xs.matches(599.9));
        assert!(!Breakpoint::Xs.matches(600.0));
        // Sm: [600, 960)
        assert!(Breakpoint::Sm.matches(600.0));
        assert!(Breakpoint::Sm.matches(959.9));
        assert!(!Breakpoint::Sm.matches(599.9));
        assert!(!Breakpoint::Sm.matches(960.0));
        // Md: [960, 1280)
        assert!(Breakpoint::Md.matches(960.0));
        assert!(!Breakpoint::Md.matches(1280.0));
        // Lg: [1280, 1920)
        assert!(Breakpoint::Lg.matches(1280.0));
        assert!(!Breakpoint::Lg.matches(1920.0));
        // Xl: [1920, 2560)
        assert!(Breakpoint::Xl.matches(1920.0));
        assert!(!Breakpoint::Xl.matches(2560.0));
        // Xxl: >= 2560
        assert!(!Breakpoint::Xxl.matches(2559.9));
        assert!(Breakpoint::Xxl.matches(2560.0));
        assert!(Breakpoint::Xxl.matches(10_000.0));
    }

    #[test]
    fn and_down_matches_up_to_and_including_the_named_bucket() {
        assert!(Breakpoint::SmAndDown.matches(0.0));
        assert!(Breakpoint::SmAndDown.matches(959.9));
        assert!(!Breakpoint::SmAndDown.matches(960.0));
        assert!(Breakpoint::MdAndDown.matches(1279.9));
        assert!(!Breakpoint::MdAndDown.matches(1280.0));
        assert!(Breakpoint::LgAndDown.matches(1919.9));
        assert!(!Breakpoint::LgAndDown.matches(1920.0));
        assert!(Breakpoint::XlAndDown.matches(2559.9));
        assert!(!Breakpoint::XlAndDown.matches(2560.0));
    }

    #[test]
    fn and_up_matches_from_the_named_bucket_upwards() {
        assert!(!Breakpoint::SmAndUp.matches(599.9));
        assert!(Breakpoint::SmAndUp.matches(600.0));
        assert!(Breakpoint::SmAndUp.matches(10_000.0));
        assert!(!Breakpoint::MdAndUp.matches(959.9));
        assert!(Breakpoint::MdAndUp.matches(960.0));
        assert!(!Breakpoint::LgAndUp.matches(1279.9));
        assert!(Breakpoint::LgAndUp.matches(1280.0));
        assert!(!Breakpoint::XlAndUp.matches(1919.9));
        assert!(Breakpoint::XlAndUp.matches(1920.0));
    }

    #[test]
    fn non_finite_width_matches_nothing_but_always() {
        for b in Breakpoint::ALL {
            if b == Breakpoint::Always {
                continue;
            }
            assert!(!b.matches(f32::NAN), "{} should not match NaN", b.name());
            assert!(
                !b.matches(f32::INFINITY),
                "{} should not match infinity",
                b.name()
            );
            assert!(
                !b.matches(f32::NEG_INFINITY),
                "{} should not match -infinity",
                b.name()
            );
        }
    }

    #[test]
    fn negative_width_is_treated_as_zero() {
        assert!(Breakpoint::Xs.matches(-10.0));
        assert!(!Breakpoint::Sm.matches(-10.0));
        assert!(Breakpoint::SmAndDown.matches(-10.0));
        assert!(!Breakpoint::SmAndUp.matches(-10.0));
    }

    // --- from_pixels ------------------------------------------------------

    #[test]
    fn from_pixels_round_trips_every_simple_bucket_at_its_lower_bound() {
        assert_eq!(Breakpoint::from_pixels(0.0), Breakpoint::Xs);
        assert_eq!(Breakpoint::from_pixels(SM_PX), Breakpoint::Sm);
        assert_eq!(Breakpoint::from_pixels(MD_PX), Breakpoint::Md);
        assert_eq!(Breakpoint::from_pixels(LG_PX), Breakpoint::Lg);
        assert_eq!(Breakpoint::from_pixels(XL_PX), Breakpoint::Xl);
        assert_eq!(Breakpoint::from_pixels(XXL_PX), Breakpoint::Xxl);
        assert_eq!(Breakpoint::from_pixels(10_000.0), Breakpoint::Xxl);
    }

    #[test]
    fn from_pixels_repairs_non_finite() {
        assert_eq!(Breakpoint::from_pixels(f32::NAN), Breakpoint::Xs);
        assert_eq!(Breakpoint::from_pixels(f32::NEG_INFINITY), Breakpoint::Xs);
        assert_eq!(Breakpoint::from_pixels(f32::INFINITY), Breakpoint::Xxl);
    }

    #[test]
    fn every_simple_bucket_matches_itself_via_from_pixels() {
        // Smoke test: for each simple bucket, from_pixels(at its lower bound)
        // returns that bucket, and matches() agrees.
        for b in [
            Breakpoint::Xs,
            Breakpoint::Sm,
            Breakpoint::Md,
            Breakpoint::Lg,
            Breakpoint::Xl,
            Breakpoint::Xxl,
        ] {
            let w = b.to_pixels();
            assert_eq!(Breakpoint::from_pixels(w), b, "round-trip failed for {}", b.name());
            assert!(b.matches(w), "{} should match its own lower bound", b.name());
        }
    }
}