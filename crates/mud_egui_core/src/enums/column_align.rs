//! `ColumnAlign` — horizontal alignment of a data-grid column's header and
//! cell content.
//!
//! Port of `MudBlazor.Align` (renamed here to avoid clashing with
//! [`egui::Align`]).

/// Horizontal alignment for a column.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ColumnAlign {
    /// Left-aligned (default).
    #[default]
    Left,
    /// Centered.
    Center,
    /// Right-aligned (typically used for numbers).
    Right,
}

impl ColumnAlign {
    /// Maps to the corresponding [`egui::Align`] value for child layouts.
    #[inline]
    pub fn to_egui(self) -> egui::Align {
        match self {
            Self::Left => egui::Align::Min,
            Self::Center => egui::Align::Center,
            Self::Right => egui::Align::Max,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_left() {
        assert_eq!(ColumnAlign::default(), ColumnAlign::Left);
    }

    #[test]
    fn to_egui_maps_every_variant() {
        assert_eq!(ColumnAlign::Left.to_egui(), egui::Align::Min);
        assert_eq!(ColumnAlign::Center.to_egui(), egui::Align::Center);
        assert_eq!(ColumnAlign::Right.to_egui(), egui::Align::Max);
    }
}