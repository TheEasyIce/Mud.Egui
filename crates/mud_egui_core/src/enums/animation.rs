//! Animation style for a skeleton placeholder.
//!
//! Port of `MudBlazor.Animation` (MudBlazor 8.13.0). The 8.13.0 enum has three
//! variants: `False = 0`, `Pulse = 1`, `Wave = 2`. The `False` variant cannot
//! be named `False` in Rust (reserved keyword), so the port maps it to
//! [`Animation::Off`]; the numeric values are preserved in the doc comments
//! for reference.
//!
//! | MudBlazor | Rust | Meaning |
//! |-----------|------|---------|
//! | `False = 0` | [`Off`](Self::Off) | No animation. |
//! | `Pulse = 1` | [`Pulse`](Self::Pulse) | Opacity fades 1.0 → 0.7 → 1.0 over 1.3 s. |
//! | `Wave = 2` | [`Wave`](Self::Wave) | A 200 pt gradient band sweeps left-to-right over 1.0 s. |
//!
//! Used by `MudSkeleton`. No other component in this workspace reads it.

/// The animation style a skeleton placeholder plays while it is visible.
///
/// Mirrors `MudBlazor.Animation` with `False` renamed to [`Off`](Self::Off) to
/// avoid the Rust keyword. `Pulse` is the default, matching MudBlazor's
/// `Animation.Pulse`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Animation {
    /// No animation. Equivalent to MudBlazor's `Animation.False = 0`.
    Off,
    /// The placeholder's opacity fades 1.0 → 0.7 → 1.0 in a 1.3 s loop.
    /// Equivalent to MudBlazor's `Animation.Pulse = 1`. **Default.**
    #[default]
    Pulse,
    /// A 200 pt gradient band sweeps across the placeholder left-to-right in a
    /// 1.0 s loop. Equivalent to MudBlazor's `Animation.Wave = 2`.
    Wave,
}

impl Animation {
    /// Every variant, in MudBlazor's declaration order.
    pub const ALL: [Animation; 3] = [Animation::Off, Animation::Pulse, Animation::Wave];

    /// Lower-case name, useful for logging and test ids.
    #[inline]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Pulse => "pulse",
            Self::Wave => "wave",
        }
    }

    /// Whether this variant animates at all.
    #[inline]
    pub const fn is_animated(self) -> bool {
        !matches!(self, Self::Off)
    }
}

impl core::fmt::Display for Animation {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_pulse() {
        assert_eq!(Animation::default(), Animation::Pulse);
    }

    #[test]
    fn all_has_three_unique_variants() {
        assert_eq!(Animation::ALL.len(), 3);
        for (i, a) in Animation::ALL.iter().enumerate() {
            for b in &Animation::ALL[i + 1..] {
                assert_ne!(a, b);
            }
        }
    }

    #[test]
    fn names_are_lowercase_and_unique() {
        let mut seen = std::collections::BTreeSet::new();
        for a in Animation::ALL {
            assert!(seen.insert(a.name()), "duplicate name: {}", a.name());
            assert_eq!(a.name(), a.name().to_lowercase());
        }
    }

    #[test]
    fn display_matches_name() {
        for a in Animation::ALL {
            assert_eq!(a.to_string(), a.name());
        }
    }

    #[test]
    fn only_off_is_not_animated() {
        assert!(!Animation::Off.is_animated());
        assert!(Animation::Pulse.is_animated());
        assert!(Animation::Wave.is_animated());
    }
}