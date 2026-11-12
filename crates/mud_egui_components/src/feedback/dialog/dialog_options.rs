//! `DialogOptions` — display options for a [`MudDialog`](super::MudDialog).
//!
//! Port of `MudBlazor.DialogOptions` (MudBlazor 8.13.0). The C# type is an
//! **immutable record** since v8.0; the Rust port is a `Clone` struct whose
//! fields are replaced wholesale by
//! [`DialogControl::set_options`](super::DialogControl::set_options), the
//! `SetOptionsAsync` equivalent.
//!
//! Every option is `Option<T>` in C# so the provider's default can win. The
//! Rust port resolves the `null` case to a concrete value in
//! [`Default`](Self::default), so a caller can read the field directly.

use mud_egui_core::enums::{DialogPosition, MaxWidth};

/// Display options for a dialog.
#[derive(Debug, Clone, PartialEq)]
pub struct DialogOptions {
    /// Close the dialog when the backdrop is clicked. Defaults to `true`.
    pub backdrop_click: bool,
    /// Custom class on the dialog background. Stored, not resolved.
    pub background_class: Option<String>,
    /// Show an explicit close (×) button in the header. Defaults to `false`.
    pub close_button: bool,
    /// Close the dialog on `Escape`. Defaults to `true`.
    pub close_on_escape_key: bool,
    /// Fill the whole screen. Defaults to `false`.
    pub full_screen: bool,
    /// Stretch to the maximum width. Defaults to `false`.
    pub full_width: bool,
    /// Cap the dialog width. Defaults to `None` (a 90 % screen fraction).
    pub max_width: Option<MaxWidth>,
    /// Hide the header row. Defaults to `false`.
    pub no_header: bool,
    /// Where the dialog sits. Defaults to `None` → `DialogPosition::Center`.
    pub position: Option<DialogPosition>,
}

impl Default for DialogOptions {
    fn default() -> Self {
        Self {
            backdrop_click: true,
            background_class: None,
            close_button: false,
            close_on_escape_key: true,
            full_screen: false,
            full_width: false,
            max_width: None,
            no_header: false,
            position: None,
        }
    }
}

impl DialogOptions {
    /// Creates the MudBlazor defaults.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// `backdrop_click`.
    #[inline]
    pub fn with_backdrop_click(mut self, b: bool) -> Self {
        self.backdrop_click = b;
        self
    }

    /// `background_class`.
    #[inline]
    pub fn with_background_class(mut self, s: impl Into<String>) -> Self {
        self.background_class = Some(s.into());
        self
    }

    /// `close_button`.
    #[inline]
    pub fn with_close_button(mut self, b: bool) -> Self {
        self.close_button = b;
        self
    }

    /// `close_on_escape_key`.
    #[inline]
    pub fn with_close_on_escape_key(mut self, b: bool) -> Self {
        self.close_on_escape_key = b;
        self
    }

    /// `full_screen`.
    #[inline]
    pub fn with_full_screen(mut self, b: bool) -> Self {
        self.full_screen = b;
        self
    }

    /// `full_width`.
    #[inline]
    pub fn with_full_width(mut self, b: bool) -> Self {
        self.full_width = b;
        self
    }

    /// `max_width`.
    #[inline]
    pub fn with_max_width(mut self, w: MaxWidth) -> Self {
        self.max_width = Some(w);
        self
    }

    /// `no_header`.
    #[inline]
    pub fn with_no_header(mut self, b: bool) -> Self {
        self.no_header = b;
        self
    }

    /// `position`.
    #[inline]
    pub fn with_position(mut self, p: DialogPosition) -> Self {
        self.position = Some(p);
        self
    }

    /// The position actually used, resolving `None` to `Center`.
    #[inline]
    pub fn effective_position(&self) -> DialogPosition {
        self.position.unwrap_or(DialogPosition::Center)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_mudblazor() {
        let o = DialogOptions::default();
        assert!(o.backdrop_click);
        assert!(o.background_class.is_none());
        assert!(!o.close_button);
        assert!(o.close_on_escape_key);
        assert!(!o.full_screen);
        assert!(!o.full_width);
        assert!(o.max_width.is_none());
        assert!(!o.no_header);
        assert!(o.position.is_none());
    }

    #[test]
    fn effective_position_defaults_to_center() {
        assert_eq!(
            DialogOptions::default().effective_position(),
            DialogPosition::Center
        );
        assert_eq!(
            DialogOptions::default()
                .with_position(DialogPosition::TopCenter)
                .effective_position(),
            DialogPosition::TopCenter
        );
    }

    #[test]
    fn every_with_setter_stores_its_field() {
        let o = DialogOptions::new()
            .with_backdrop_click(false)
            .with_background_class("dialog-bg")
            .with_close_button(true)
            .with_close_on_escape_key(false)
            .with_full_screen(true)
            .with_full_width(true)
            .with_max_width(MaxWidth::Md)
            .with_no_header(true)
            .with_position(DialogPosition::BottomRight);

        assert!(!o.backdrop_click);
        assert_eq!(o.background_class.as_deref(), Some("dialog-bg"));
        assert!(o.close_button);
        assert!(!o.close_on_escape_key);
        assert!(o.full_screen);
        assert!(o.full_width);
        assert_eq!(o.max_width, Some(MaxWidth::Md));
        assert!(o.no_header);
        assert_eq!(o.position, Some(DialogPosition::BottomRight));
    }

    #[test]
    fn clone_is_independent() {
        let a = DialogOptions::new().with_close_button(true);
        let mut b = a.clone();
        b.close_button = false;
        assert!(a.close_button);
        assert!(!b.close_button);
    }
}