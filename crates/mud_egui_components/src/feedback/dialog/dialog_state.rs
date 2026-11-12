//! Caller-owned state for [`MudDialog`](super::MudDialog).
//!
//! The egui equivalent of MudBlazor's `IDialogService` bookkeeping plus the
//! inline-dialog `IsVisible` two-way binding. Holds the open flag, a stable
//! id, the current title and options (the `SetTitleAsync` /
//! `SetOptionsAsync` overrides), and the result of the closing frame.
//!
//! Persist it across frames — the same caller-owned-state pattern
//! `MudDatePicker`, `MudColorPicker`, `MudTable` and the rest of the library
//! use.
//!
//! **Not `Clone`**: `DialogResult` carries a `Box<dyn Any + Send + Sync>`
//! payload that has no cloneable form. `DialogState` is passed by `&mut` to
//! `show`; every mutation happens in place.

use egui::Id;

use super::dialog_options::DialogOptions;
use super::dialog_result::DialogResult;

/// Caller-owned dialog state.
#[derive(Debug)]
pub struct DialogState {
    /// Whether the dialog is open. Mirrors `IsVisible`.
    pub open: bool,
    /// Stable identity for the backdrop, the panel, and the size cache.
    pub id: Id,
    /// Title override set by [`set_title`](Self::set_title). When `Some`, it
    /// wins over the builder's `title` — the `SetTitleAsync` equivalent.
    pub title: Option<String>,
    /// Options override set by [`set_options`](Self::set_options). Starts from
    /// the builder's options — the `SetOptionsAsync` equivalent.
    pub options: DialogOptions,
    /// Result of the frame that closed the dialog. Drained by
    /// [`take_result`](Self::take_result) or by the closing `show` call.
    pub result: Option<DialogResult>,
    /// `true` until the next `show` claims focus for the opening frame.
    focus_requested: bool,
}

impl DialogState {
    /// Creates a closed dialog with a stable id.
    pub fn new(id_salt: impl std::hash::Hash + std::fmt::Debug) -> Self {
        Self {
            open: false,
            id: Id::new(id_salt),
            title: None,
            options: DialogOptions::default(),
            result: None,
            focus_requested: false,
        }
    }

    /// Creates an already-open dialog. Mirrors showing a dialog on first
    /// frame.
    pub fn opened(id_salt: impl std::hash::Hash + std::fmt::Debug) -> Self {
        let mut s = Self::new(id_salt);
        s.open();
        s
    }

    /// Whether the dialog is open.
    #[inline]
    pub fn is_open(&self) -> bool {
        self.open
    }

    /// Opens the dialog. Mirrors `IsVisible = true`.
    pub fn open(&mut self) {
        self.open = true;
        self.result = None;
        self.focus_requested = true;
    }

    /// Opens the dialog with an explicit title and options. The
    /// `MudDialog.Show(title, options)` equivalent for inline dialogs.
    pub fn open_with(&mut self, title: Option<String>, options: DialogOptions) {
        self.title = title;
        self.options = options;
        self.open();
    }

    /// Closes with [`DialogResult::ok`]. Mirrors `IMudDialogInstance.Close()`.
    #[inline]
    pub fn close(&mut self) {
        self.close_with(DialogResult::ok());
    }

    /// Closes with [`DialogResult::cancel`]. Mirrors `IMudDialogInstance.Cancel()`.
    #[inline]
    pub fn cancel(&mut self) {
        self.close_with(DialogResult::cancel());
    }

    /// Closes with a custom result. Mirrors
    /// `IMudDialogInstance.Close(DialogResult)`.
    pub fn close_with(&mut self, result: DialogResult) {
        self.open = false;
        self.result = Some(result);
    }

    /// Overrides the title. Mirrors `IMudDialogInstance.SetTitleAsync`.
    #[inline]
    pub fn set_title(&mut self, title: Option<String>) {
        self.title = title;
    }

    /// Overrides the options. Mirrors `IMudDialogInstance.SetOptionsAsync`.
    #[inline]
    pub fn set_options(&mut self, options: DialogOptions) {
        self.options = options;
    }

    /// Takes the closing frame's result, if any. Mirrors awaiting
    /// `IDialogReference.Result`.
    #[inline]
    pub fn take_result(&mut self) -> Option<DialogResult> {
        self.result.take()
    }

    /// `true` on the opening frame only. Used by `show` to request focus.
    pub(crate) fn take_focus_request(&mut self) -> bool {
        let was = self.focus_requested;
        self.focus_requested = false;
        was
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- construction -----------------------------------------------------

    #[test]
    fn new_starts_closed_with_defaults() {
        let s = DialogState::new("t");
        assert!(!s.is_open());
        assert!(s.title.is_none());
        assert_eq!(s.options, DialogOptions::default());
        assert!(s.result.is_none());
    }

    #[test]
    fn opened_starts_open() {
        let s = DialogState::opened("t");
        assert!(s.is_open());
    }

    // --- open / close -----------------------------------------------------

    #[test]
    fn open_requests_focus() {
        let mut s = DialogState::new("t");
        s.open();
        assert!(s.take_focus_request());
        assert!(!s.take_focus_request(), "focus is claimed once");
    }

    #[test]
    fn open_with_sets_title_and_options() {
        let mut s = DialogState::new("t");
        let opts = DialogOptions::new().with_close_button(true);
        s.open_with(Some("Hello".into()), opts.clone());
        assert!(s.is_open());
        assert_eq!(s.title.as_deref(), Some("Hello"));
        assert_eq!(s.options, opts);
    }

    #[test]
    fn close_defaults_to_ok() {
        let mut s = DialogState::opened("t");
        s.close();
        assert!(!s.is_open());
        let r = s.take_result().expect("a result");
        assert!(r.is_ok());
        assert!(!r.is_canceled());
    }

    #[test]
    fn cancel_defaults_to_canceled() {
        let mut s = DialogState::opened("t");
        s.cancel();
        let r = s.take_result().expect("a result");
        assert!(r.is_canceled());
    }

    #[test]
    fn close_with_stores_the_payload() {
        let mut s = DialogState::opened("t");
        s.close_with(DialogResult::ok_with(7u32));
        let r = s.take_result().expect("a result");
        assert_eq!(r.data::<u32>(), Some(&7));
    }

    #[test]
    fn take_result_drains_it() {
        let mut s = DialogState::opened("t");
        s.close();
        assert!(s.take_result().is_some());
        assert!(s.take_result().is_none());
    }

    #[test]
    fn open_clears_a_stale_result() {
        let mut s = DialogState::opened("t");
        s.close();
        s.open();
        assert!(s.result.is_none());
    }

    // --- overrides --------------------------------------------------------

    #[test]
    fn set_title_and_options_override() {
        let mut s = DialogState::new("t");
        s.set_title(Some("New".into()));
        s.set_options(DialogOptions::new().with_full_width(true));
        assert_eq!(s.title.as_deref(), Some("New"));
        assert!(s.options.full_width);
    }

    #[test]
    fn set_title_accepts_none_to_revert() {
        let mut s = DialogState::new("t");
        s.set_title(Some("X".into()));
        s.set_title(None);
        assert!(s.title.is_none());
    }

    // --- id ---------------------------------------------------------------

    #[test]
    fn id_is_stable_across_constructions() {
        assert_eq!(DialogState::new("x").id, DialogState::new("x").id);
        assert_ne!(DialogState::new("x").id, DialogState::new("y").id);
    }
}