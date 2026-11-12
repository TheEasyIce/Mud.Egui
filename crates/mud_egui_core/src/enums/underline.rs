//! `Underline` — how a link's underline is displayed.
//!
//! Port of `MudBlazor.Underline` (MudBlazor 8.13.0).

/// Specifies when an underline is drawn under a link.
///
/// Mirrors MudBlazor's `Underline` enum, whose C# values are
/// `None = 0`, `Hover = 1`, `Always = 2`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Underline {
    /// No underline is ever displayed.
    #[allow(non_upper_case_globals)]
    None,
    /// Underline is displayed only while the pointer is over the link. This
    /// is MudBlazor's default.
    #[default]
    Hover,
    /// Underline is always displayed.
    Always,
}

impl Underline {
    /// Whether an underline should be painted in the given state.
    #[inline]
    pub fn should_paint(self, hovered: bool) -> bool {
        match self {
            Self::None => false,
            Self::Hover => hovered,
            Self::Always => true,
        }
    }

    /// Human-readable name matching the C# enum member.
    #[inline]
    pub fn name(self) -> &'static str {
        match self {
            Self::None => "None",
            Self::Hover => "Hover",
            Self::Always => "Always",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_hover() {
        assert_eq!(Underline::default(), Underline::Hover);
    }

    #[test]
    fn none_never_paints() {
        assert!(!Underline::None.should_paint(false));
        assert!(!Underline::None.should_paint(true));
    }

    #[test]
    fn hover_paints_only_when_hovered() {
        assert!(!Underline::Hover.should_paint(false));
        assert!(Underline::Hover.should_paint(true));
    }

    #[test]
    fn always_paints_in_both_states() {
        assert!(Underline::Always.should_paint(false));
        assert!(Underline::Always.should_paint(true));
    }

    #[test]
    fn name_matches_variant() {
        assert_eq!(Underline::None.name(), "None");
        assert_eq!(Underline::Hover.name(), "Hover");
        assert_eq!(Underline::Always.name(), "Always");
    }
}