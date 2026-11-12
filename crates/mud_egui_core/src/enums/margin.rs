//! `Margin` — density modifier for input components.
//!
//! Port of `MudBlazor.Margin` (MudBlazor 8.13.0). Used by `MudTextField`,
//! `MudSelect`, `MudAutocomplete`, and the picker family to tighten or loosen
//! the vertical space around an input control.

/// Vertical density modifier for an input component.
///
/// Mirrors MudBlazor's `Margin` enum:
///
/// * `None` — no additional vertical space (MudBlazor's default).
/// * `Dense` — a compact layout (a few pixels tighter than `None`).
/// * `Normal` — MudBlazor's roomier layout (a few pixels looser than `None`).
///
/// The actual pixel deltas are resolved by the consuming component from
/// `Size` + `Margin`; `Margin` itself carries no numbers so it can be reused
/// across inputs without inheritance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Margin {
    /// No additional vertical space. This is MudBlazor's default.
    #[default]
    None,
    /// Compact layout.
    Dense,
    /// Roomier layout.
    Normal,
}

impl Margin {
    /// Human-readable name matching the C# enum member.
    #[inline]
    pub fn name(self) -> &'static str {
        match self {
            Self::None => "None",
            Self::Dense => "Dense",
            Self::Normal => "Normal",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_none() {
        assert_eq!(Margin::default(), Margin::None);
    }

    #[test]
    fn names_match_variants() {
        assert_eq!(Margin::None.name(), "None");
        assert_eq!(Margin::Dense.name(), "Dense");
        assert_eq!(Margin::Normal.name(), "Normal");
    }
}