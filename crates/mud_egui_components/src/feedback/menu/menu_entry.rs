//! Private collector entry for [`MudMenu`](super::MudMenu).
//!
//! Kept in its own file per the project convention that every enum lives in
//! its own file. `pub(crate)` — not part of the public API.

use super::{MenuSeparator, MenuItem};

/// One collected menu entry.
pub(crate) enum MenuEntry<'a> {
    /// A clickable menu item.
    Item(MenuItem<'a>),
    /// A non-clickable divider between items.
    Separator(MenuSeparator),
}