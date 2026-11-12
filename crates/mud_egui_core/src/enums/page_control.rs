//! `PageControl` — the four navigation control buttons of a pagination bar.
//!
//! Port of `MudBlazor.Page` (MudBlazor 8.13.0). **Renamed** to `PageControl`
//! because `Page` is already the trait name in `mud_egui_navigation::page`
//! and the two would otherwise clash in every `use` statement.
//!
//! This enum is what `MudPagination::ControlButtonClicked` passes to its
//! callback in MudBlazor.

/// One of the four control buttons in a pagination bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PageControl {
    /// Navigate to the first page.
    #[default]
    First,
    /// Navigate one page back.
    Previous,
    /// Navigate one page forward.
    Next,
    /// Navigate to the last page.
    Last,
}

impl PageControl {
    /// Human-readable name, matching the C# enum member.
    #[inline]
    pub fn name(self) -> &'static str {
        match self {
            Self::First => "First",
            Self::Previous => "Previous",
            Self::Next => "Next",
            Self::Last => "Last",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_first() {
        assert_eq!(PageControl::default(), PageControl::First);
    }

    #[test]
    fn name_matches_variant() {
        assert_eq!(PageControl::First.name(), "First");
        assert_eq!(PageControl::Previous.name(), "Previous");
        assert_eq!(PageControl::Next.name(), "Next");
        assert_eq!(PageControl::Last.name(), "Last");
    }
}