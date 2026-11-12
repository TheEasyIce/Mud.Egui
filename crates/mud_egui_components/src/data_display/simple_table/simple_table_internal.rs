//! Internal collector types for [`MudSimpleTable`](super::MudSimpleTable).
//!
//! Kept in their own file so the component module contains no enums, matching
//! the project convention that every enum lives in its own file. These are
//! `pub(crate)` — implementation details of the table collector, not part of
//! the public API.

use egui::Ui;

/// Content of one table cell.
pub(crate) enum CellContent<'a> {
    /// A plain text cell, rendered through [`MudText`](crate::MudText).
    Text(String),
    /// A custom-UI cell; the closure receives the cell's child `Ui`.
    Ui(Box<dyn FnMut(&mut Ui) + 'a>),
}

/// Which structural region a row belongs to.
pub(crate) enum RowKind {
    /// Header row (`<thead>` / `<th>` styling).
    Header,
    /// Data row (`<tbody>` / `<td>` styling).
    Body,
    /// Footer row (`<tfoot>` styling).
    Footer,
}