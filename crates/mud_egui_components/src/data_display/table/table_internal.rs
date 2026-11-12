//! Internal collector types for [`MudTable`](super::MudTable).
//!
//! Kept in their own file so the component module contains no enums, matching
//! the project convention that every enum lives in its own file. Both are
//! `pub(crate)` — implementation details of the table collector.

use egui::Ui;

/// Content of one header or body cell.
pub(crate) enum CellContent<'a> {
    /// Plain text, rendered through [`MudText`](crate::MudText).
    Text(String),
    /// Custom UI; the closure receives the cell's child `Ui`.
    Ui(Box<dyn FnMut(&mut Ui) + 'a>),
}

/// Which structural region a row belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RowKind {
    /// `<thead>` row.
    Header,
    /// `<tbody>` row.
    Body,
    /// `<tfoot>` row.
    Footer,
}