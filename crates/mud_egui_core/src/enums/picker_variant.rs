//! `PickerVariant` — the display variant of a picker.
//!
//! Port of `MudBlazor.PickerVariant` (MudBlazor 8.13.0). Controls whether a
//! picker renders inline in the page, in a dialog, or as a static calendar.

/// The display variant of a picker.
///
/// | Variant | Behaviour |
/// |---------|-----------|
/// | `Inline` | The picker is always visible as an inline calendar/clock/wheel. The trigger text field is optional. |
/// | `Dialog` (default) | The picker opens in a popover below the trigger. |
/// | `Static` | The picker is always visible as a static panel with no input. |
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PickerVariant {
    /// Pickers render inline, permanently visible.
    Inline,
    /// Pickers open in a popover below the trigger. MudBlazor's default.
    #[default]
    Dialog,
    /// Static panel with no input field.
    Static,
}

impl PickerVariant {
    /// Human-readable name matching the C# enum member.
    #[inline]
    pub fn name(self) -> &'static str {
        match self {
            Self::Inline => "Inline",
            Self::Dialog => "Dialog",
            Self::Static => "Static",
        }
    }

    /// Whether the picker requires a trigger text field.
    #[inline]
    pub fn uses_input(self) -> bool {
        !matches!(self, Self::Static)
    }

    /// Whether the picker opens in a popover.
    #[inline]
    pub fn is_dialog(self) -> bool {
        matches!(self, Self::Dialog)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_dialog() {
        assert_eq!(PickerVariant::default(), PickerVariant::Dialog);
    }

    #[test]
    fn names_match_variants() {
        assert_eq!(PickerVariant::Inline.name(), "Inline");
        assert_eq!(PickerVariant::Dialog.name(), "Dialog");
        assert_eq!(PickerVariant::Static.name(), "Static");
    }

    #[test]
    fn uses_input_matches_variant() {
        assert!(PickerVariant::Inline.uses_input());
        assert!(PickerVariant::Dialog.uses_input());
        assert!(!PickerVariant::Static.uses_input());
    }

    #[test]
    fn is_dialog_only_for_dialog() {
        assert!(!PickerVariant::Inline.is_dialog());
        assert!(PickerVariant::Dialog.is_dialog());
        assert!(!PickerVariant::Static.is_dialog());
    }
}