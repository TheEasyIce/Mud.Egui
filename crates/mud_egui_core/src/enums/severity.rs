//! Severity level of a feedback message.
//!
//! Port of `MudBlazor.Severity` (MudBlazor 8.13.0). The five levels documented
//! for `MudAlert` are `Normal`, `Info`, `Success`, `Warning` and `Error`; the
//! severity determines both the accent colour and the default icon.
//!
//! Used by `MudAlert` and, later, by `MudSnackbar`.

/// How severe a feedback message is.
///
/// Mirrors `MudBlazor.Severity`. `Normal` is the neutral default and shares its
/// icon with [`Info`](Self::Info).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Severity {
    /// Neutral message; resolves to the theme's default colour.
    Normal,
    /// Informational message.
    Info,
    /// Operation succeeded.
    Success,
    /// Something needs attention.
    Warning,
    /// Operation failed.
    Error,
}

impl Default for Severity {
    /// [`Severity::Normal`], matching MudBlazor's default.
    fn default() -> Self {
        Self::Normal
    }
}

impl Severity {
    /// Every severity, in MudBlazor's declaration order.
    pub const ALL: [Severity; 5] = [
        Severity::Normal,
        Severity::Info,
        Severity::Success,
        Severity::Warning,
        Severity::Error,
    ];

    /// Lower-case name, useful for logging and test ids.
    #[inline]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Info => "info",
            Self::Success => "success",
            Self::Warning => "warning",
            Self::Error => "error",
        }
    }

    /// Whether this is the neutral level (no semantic colour of its own).
    #[inline]
    pub const fn is_neutral(self) -> bool {
        matches!(self, Self::Normal)
    }
}

impl core::fmt::Display for Severity {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_normal() {
        assert_eq!(Severity::default(), Severity::Normal);
    }

    #[test]
    fn all_has_five_unique_levels() {
        assert_eq!(Severity::ALL.len(), 5);
        for (i, a) in Severity::ALL.iter().enumerate() {
            for b in &Severity::ALL[i + 1..] {
                assert_ne!(a, b);
            }
        }
    }

    #[test]
    fn names_are_lower_case() {
        assert_eq!(Severity::Normal.name(), "normal");
        assert_eq!(Severity::Error.name(), "error");
        for s in Severity::ALL {
            assert_eq!(s.name(), s.name().to_lowercase());
        }
    }

    #[test]
    fn only_normal_is_neutral() {
        assert!(Severity::Normal.is_neutral());
        for s in &Severity::ALL[1..] {
            assert!(!s.is_neutral());
        }
    }

    #[test]
    fn display_matches_name() {
        for s in Severity::ALL {
            assert_eq!(s.to_string(), s.name());
        }
    }
}