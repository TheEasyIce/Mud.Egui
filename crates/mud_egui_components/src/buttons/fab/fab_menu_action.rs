//! `FabMenuAction` — one entry in a [`MudMenuFab`](super::MudMenuFab)'s action
//! list.
//!
//! Port of `MudBlazor.FabMenuAction` (MudBlazor 8.13.0). The C# record has
//! three properties — `Icon` (required), `Tooltip`, `OnClick`. The Rust port
//! keeps the same shape; `on_click` is `Option<Box<dyn FnMut() + 'a>>` so a
//! call site can borrow local data with the lifetime of the enclosing
//! `MudMenuFab`.

/// One action inside a [`MudMenuFab`](super::MudMenuFab).
pub struct FabMenuAction<'a> {
    /// Material SVG path shown inside the action's circular button.
    pub icon: String,
    /// Hover tooltip. `None` shows no tooltip.
    pub tooltip: Option<String>,
    /// Fired when the action is clicked. The action menu closes afterwards.
    pub on_click: Option<Box<dyn FnMut() + 'a>>,
}

impl<'a> FabMenuAction<'a> {
    /// Creates an action with `icon` and no tooltip or callback.
    #[inline]
    pub fn new(icon: impl Into<String>) -> Self {
        Self {
            icon: icon.into(),
            tooltip: None,
            on_click: None,
        }
    }

    /// Sets the hover tooltip.
    #[inline]
    pub fn tooltip(mut self, s: impl Into<String>) -> Self {
        self.tooltip = Some(s.into());
        self
    }

    /// Sets the click callback.
    #[inline]
    pub fn on_click(mut self, f: impl FnMut() + 'a) -> Self {
        self.on_click = Some(Box::new(f));
        self
    }
}

impl<'a> std::fmt::Debug for FabMenuAction<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FabMenuAction")
            .field("icon", &self.icon)
            .field("tooltip", &self.tooltip)
            .field("on_click", &self.on_click.is_some())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    #[test]
    fn new_has_icon_only() {
        let a = FabMenuAction::new("X");
        assert_eq!(a.icon, "X");
        assert!(a.tooltip.is_none());
        assert!(a.on_click.is_none());
    }

    #[test]
    fn tooltip_and_callback_setters_store() {
        // `Cell::set` takes `&self`, so the closure does not keep `called`
        // mutably borrowed — reading `called.get()` afterwards is legal.
        let called = Cell::new(false);
        let mut a = FabMenuAction::new("X")
            .tooltip("Tip")
            .on_click(|| called.set(true));
        assert_eq!(a.tooltip.as_deref(), Some("Tip"));
        assert!(a.on_click.is_some());
        (a.on_click.as_mut().unwrap())();
        assert!(called.get());
    }
}