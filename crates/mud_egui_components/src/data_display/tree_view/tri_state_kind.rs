//! Private tri-state value computed during rendering.
//!
//! Kept in its own file per the project convention that every enum lives in
//! its own file. `pub(crate)` — not part of the public API.

/// Effective checked state of a tree node in `Multi` + `TriState` mode.
///
/// * [`Unchecked`](Self::Unchecked) — no descendant is checked.
/// * [`Indeterminate`](Self::Indeterminate) — some descendants are checked,
///   some are not.
/// * [`Checked`](Self::Checked) — every descendant is checked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TriStateKind {
    /// Nothing checked.
    Unchecked,
    /// Mixed checked / unchecked.
    Indeterminate,
    /// All checked.
    Checked,
}