//! The trailing adornment of a [`MudTextField`](super::MudTextField).
//!
//! Kept in its own file per the project convention that every enum lives in
//! its own file. Public because it is a public builder parameter.
//!
//! MudBlazor's `Adornment` family (`Adornment`, `AdornmentIcon`,
//! `AdornmentText`, `AdornmentSide`, `AdornmentColor`, `AdornmentAlignment`)
//! is collapsed into one enum here: for the common case — a chevron on a
//! select, a calendar icon on a date picker, a colour swatch on a colour
//! picker — you only need to choose *what* to draw at the trailing edge.
//! The side is always trailing (`AdornmentSide::End`); the alignment is
//! always vertically centred.

/// The trailing adornment of a text field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdornmentEnd {
    /// Render an SVG glyph (e.g. a chevron, calendar, or clock).
    ///
    /// The glyph inherits the field's secondary text colour unless
    /// overridden by `adornment_end_color`.
    Icon(String),

    /// Render a short text label (e.g. a unit such as `"kg"` or `"ms"`).
    Text(String),
}

impl AdornmentEnd {
    /// Convenience constructor for [`AdornmentEnd::Icon`].
    pub fn icon(svg: impl Into<String>) -> Self {
        Self::Icon(svg.into())
    }

    /// Convenience constructor for [`AdornmentEnd::Text`].
    pub fn text(t: impl Into<String>) -> Self {
        Self::Text(t.into())
    }

    /// Whether this adornment is a glyph (used by the layout code to decide
    /// its width).
    #[inline]
    pub fn is_icon(&self) -> bool {
        matches!(self, Self::Icon(_))
    }

    /// The glyph or text content, whichever this variant holds.
    #[inline]
    pub fn content(&self) -> &str {
        match self {
            Self::Icon(s) | Self::Text(s) => s,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icon_constructor_and_variant() {
        let a = AdornmentEnd::icon("M0 0");
        assert!(a.is_icon());
        assert_eq!(a.content(), "M0 0");
        assert!(matches!(a, AdornmentEnd::Icon(_)));
    }

    #[test]
    fn text_constructor_and_variant() {
        let a = AdornmentEnd::text("kg");
        assert!(!a.is_icon());
        assert_eq!(a.content(), "kg");
        assert!(matches!(a, AdornmentEnd::Text(_)));
    }
}