//! Internal collector entry for [`ListUi`](super::ListUi).
//!
//! Kept in its own file so the component module contains no enums, matching
//! the project convention that every enum lives in its own file. This one is
//! `pub(crate)` — it is an implementation detail of the list collector, not
//! part of the public API.

use super::{MudListItem, MudListSubheader};

/// One collected entry: either a list item or a subheader.
pub(crate) enum ListEntry<'a, T> {
    /// A normal or nested list item.
    Item(MudListItem<'a, T>),
    /// A non-interactive subheader row.
    Subheader(MudListSubheader),
}