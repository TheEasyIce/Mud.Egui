//! Private collector entry for [`MudNavMenu`](super::MudNavMenu) and
//! [`NavGroup`](super::NavGroup).
//!
//! Kept in its own file per the project convention that every enum lives in
//! its own file. `pub(crate)` — not part of the public API.

use super::{NavGroup, NavLink};

/// One collected entry: either a link or a (possibly nested) group.
pub(crate) enum NavEntry<'a> {
    /// A leaf navigation link.
    Link(NavLink<'a>),
    /// A collapsible group of links and sub-groups.
    Group(NavGroup<'a>),
}