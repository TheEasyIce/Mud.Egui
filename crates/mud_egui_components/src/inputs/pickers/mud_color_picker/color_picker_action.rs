//! `ColorPickerAction` — what a [`MudColorPicker`](super::MudColorPicker) did
//! this frame.
//!
//! **Extension**: MudBlazor reports these outcomes through three callbacks
//! (`PickerOpened`, `PickerClosed`, `ValueChanged`). The port promotes the
//! outcome to one enum so a caller reads it from the return value — the same
//! split `DatePickerAction`, `TimePickerAction` and `DateRangePickerAction`
//! use.
//!
//! Kept in its own file per Rule 2.

use super::color_channels::ColorChannels;

/// What a [`MudColorPicker`](super::MudColorPicker) did this frame.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ColorPickerAction {
    /// No interaction this frame.
    #[default]
    None,
    /// The popover opened.
    Opened,
    /// The popover closed.
    Closed,
    /// A colour was committed. Mirrors `ValueChanged`.
    ColorSelected(ColorChannels),
    /// The value was cleared.
    Cleared,
}

impl ColorPickerAction {
    /// `true` when the bound colour changed (`ColorSelected` / `Cleared`).
    #[inline]
    pub fn changed_value(&self) -> bool {
        matches!(self, Self::ColorSelected(_) | Self::Cleared)
    }

    /// `true` when the popover open/close state changed.
    #[inline]
    pub fn toggled_open(&self) -> bool {
        matches!(self, Self::Opened | Self::Closed)
    }

    /// The committed colour, when this frame committed one.
    #[inline]
    pub fn selected(&self) -> Option<ColorChannels> {
        match self {
            Self::ColorSelected(c) => Some(*c),
            _ => None,
        }
    }

    /// Human-readable name, matching the variant.
    #[inline]
    pub fn name(&self) -> &'static str {
        match self {
            Self::None => "None",
            Self::Opened => "Opened",
            Self::Closed => "Closed",
            Self::ColorSelected(_) => "ColorSelected",
            Self::Cleared => "Cleared",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn red() -> ColorChannels {
        ColorChannels::from_rgba(255, 0, 0, 255)
    }

    #[test]
    fn default_is_none() {
        assert_eq!(ColorPickerAction::default(), ColorPickerAction::None);
    }

    #[test]
    fn changed_value_only_for_colour_mutations() {
        assert!(ColorPickerAction::ColorSelected(red()).changed_value());
        assert!(ColorPickerAction::Cleared.changed_value());
        assert!(!ColorPickerAction::None.changed_value());
        assert!(!ColorPickerAction::Opened.changed_value());
        assert!(!ColorPickerAction::Closed.changed_value());
    }

    #[test]
    fn toggled_open_only_for_open_and_close() {
        assert!(ColorPickerAction::Opened.toggled_open());
        assert!(ColorPickerAction::Closed.toggled_open());
        assert!(!ColorPickerAction::ColorSelected(red()).toggled_open());
    }

    #[test]
    fn selected_returns_the_colour_only_for_color_selected() {
        assert_eq!(ColorPickerAction::ColorSelected(red()).selected(), Some(red()));
        assert_eq!(ColorPickerAction::None.selected(), None);
        assert_eq!(ColorPickerAction::Cleared.selected(), None);
    }

    #[test]
    fn names_match_variants() {
        assert_eq!(ColorPickerAction::None.name(), "None");
        assert_eq!(ColorPickerAction::Opened.name(), "Opened");
        assert_eq!(ColorPickerAction::Closed.name(), "Closed");
        assert_eq!(ColorPickerAction::ColorSelected(red()).name(), "ColorSelected");
        assert_eq!(ColorPickerAction::Cleared.name(), "Cleared");
    }
}