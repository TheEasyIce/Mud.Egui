//! `DialogControl` — the handle a dialog's content uses to talk back.
//!
//! The egui analogue of MudBlazor's `[CascadingParameter] IMudDialogInstance`:
//! the content closure receives `&mut DialogControl` and calls
//! [`close`](Self::close), [`cancel`](Self::cancel),
//! [`close_with`](Self::close_with), [`set_title`](Self::set_title) or
//! [`set_options`](Self::set_options). Requests are **deferred**: `show`
//! applies them after the panel has finished drawing, so the content can
//! borrow `DialogControl` freely without conflicting with the caller's
//! `&mut DialogState`.
//!
//! Kept in its own file because it is a first-class public type used inside
//! every dialog body.
//!
//! **Not `Clone`**: `DialogResult` carries a `Box<dyn Any + Send + Sync>`
//! payload that has no cloneable form. The handle is passed by `&mut` to
//! `show`; it is never cloned.

use super::dialog_options::DialogOptions;
use super::dialog_result::DialogResult;

/// A dialog content's handle back to its host.
#[derive(Debug)]
pub struct DialogControl {
    /// The options in force for this frame. Copy of
    /// [`DialogState::options`](super::DialogState::options).
    pub options: DialogOptions,
    /// The title in force for this frame.
    pub title: Option<String>,
    close_request: Option<DialogResult>,
    title_request: Option<Option<String>>,
    options_request: Option<DialogOptions>,
}

impl DialogControl {
    /// Creates a control pre-loaded with the current title and options.
    #[inline]
    pub fn new(options: DialogOptions, title: Option<String>) -> Self {
        Self {
            options,
            title,
            close_request: None,
            title_request: None,
            options_request: None,
        }
    }

    /// Closes the dialog with [`DialogResult::ok`]. The
    /// `MudDialog.Close()` equivalent.
    #[inline]
    pub fn close(&mut self) {
        self.close_request = Some(DialogResult::ok());
    }

    /// Closes the dialog with [`DialogResult::cancel`]. The
    /// `MudDialog.Cancel()` equivalent.
    #[inline]
    pub fn cancel(&mut self) {
        self.close_request = Some(DialogResult::cancel());
    }

    /// Closes the dialog with a custom result. The
    /// `MudDialog.Close(DialogResult)` equivalent.
    #[inline]
    pub fn close_with(&mut self, result: DialogResult) {
        self.close_request = Some(result);
    }

    /// `true` when a close has already been requested this frame.
    #[inline]
    pub fn is_close_requested(&self) -> bool {
        self.close_request.is_some()
    }

    /// Overrides the title. The `SetTitleAsync` equivalent.
    #[inline]
    pub fn set_title(&mut self, title: Option<String>) {
        self.title = title.clone();
        self.title_request = Some(title);
    }

    /// Overrides the options. The `SetOptionsAsync` equivalent.
    #[inline]
    pub fn set_options(&mut self, options: DialogOptions) {
        self.options = options.clone();
        self.options_request = Some(options);
    }

    /// Takes the pending close request, if any.
    #[inline]
    pub fn take_close_request(&mut self) -> Option<DialogResult> {
        self.close_request.take()
    }

    /// Takes the pending title request, if any.
    #[inline]
    pub fn take_title_request(&mut self) -> Option<Option<String>> {
        self.title_request.take()
    }

    /// Takes the pending options request, if any.
    #[inline]
    pub fn take_options_request(&mut self) -> Option<DialogOptions> {
        self.options_request.take()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctl() -> DialogControl {
        DialogControl::new(DialogOptions::default(), Some("T".into()))
    }

    #[test]
    fn new_starts_with_no_requests() {
        let c = ctl();
        assert!(!c.is_close_requested());
        assert_eq!(c.title.as_deref(), Some("T"));
        assert_eq!(c.options, DialogOptions::default());
    }

    #[test]
    fn close_requests_ok() {
        let mut c = ctl();
        c.close();
        assert!(c.is_close_requested());
        let r = c.take_close_request().expect("a result");
        assert!(r.is_ok());
        assert!(!c.is_close_requested(), "request is drained");
    }

    #[test]
    fn cancel_requests_cancel() {
        let mut c = ctl();
        c.cancel();
        assert!(c.take_close_request().expect("a result").is_canceled());
    }

    #[test]
    fn close_with_carries_the_payload() {
        let mut c = ctl();
        c.close_with(DialogResult::ok_with(9u8));
        assert_eq!(c.take_close_request().and_then(|mut r| r.take_data::<u8>()), Some(9));
    }

    #[test]
    fn set_title_updates_and_records() {
        let mut c = ctl();
        c.set_title(Some("New".into()));
        assert_eq!(c.title.as_deref(), Some("New"));
        assert_eq!(c.take_title_request(), Some(Some("New".into())));
        assert_eq!(c.take_title_request(), None);
    }

    #[test]
    fn set_options_updates_and_records() {
        let mut c = ctl();
        let opts = DialogOptions::new().with_close_button(true);
        c.set_options(opts.clone());
        assert!(c.options.close_button);
        assert_eq!(c.take_options_request(), Some(opts));
    }

    #[test]
    fn last_close_wins() {
        let mut c = ctl();
        c.close();
        c.cancel();
        assert!(c.take_close_request().expect("a result").is_canceled());
    }
}