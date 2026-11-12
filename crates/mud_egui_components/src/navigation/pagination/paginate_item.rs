//! Private enum for the layout produced by [`paginate_layout`](
//! super::mud_pagination::paginate_layout).
//!
//! Kept in its own file per the project convention that every enum lives in
//! its own file. `pub(crate)` — not part of the public API.

/// One entry in the rendered pagination layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PaginateItem {
    /// A clickable page number (1-based).
    Page(u32),
    /// A non-clickable gap between two non-contiguous page groups.
    Ellipsis,
}