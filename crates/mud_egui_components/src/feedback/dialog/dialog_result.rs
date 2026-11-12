//! `DialogResult` — what a dialog returned when it closed.
//!
//! Port of `MudBlazor.DialogResult` (MudBlazor 8.13.0). The C# class exposes
//! three constructors (`Ok()`, `Ok<T>(T)`, `Cancel()`) and two properties
//! (`.Canceled`, `.Data`). The Rust port keeps the same shape.
//!
//! `.Data` is a `Box<dyn Any + Send + Sync>` because the payload type is
//! chosen by the caller, exactly like the generic `Ok<T>` overload. Read it
//! with [`DialogResult::data::<T>`] or take it with
//! [`DialogResult::take_data::<T>`].
//!
//! **Not `PartialEq`**: `dyn Any` does not implement equality. Compare
//! [`is_canceled`](Self::is_canceled) and the downcast payload instead.

use std::any::Any;
use std::fmt;

/// What a dialog returned when it closed.
pub struct DialogResult {
    canceled: bool,
    data: Option<Box<dyn Any + Send + Sync>>,
}

impl DialogResult {
    /// A successful result with no payload. Mirrors `DialogResult.Ok()`.
    #[inline]
    pub fn ok() -> Self {
        Self { canceled: false, data: None }
    }

    /// A successful result carrying `data`. Mirrors `DialogResult.Ok<T>(T)`.
    #[inline]
    pub fn ok_with<T: Any + Send + Sync>(data: T) -> Self {
        Self { canceled: false, data: Some(Box::new(data)) }
    }

    /// A cancelled result. Mirrors `DialogResult.Cancel()`.
    #[inline]
    pub fn cancel() -> Self {
        Self { canceled: true, data: None }
    }

    /// `true` for [`cancel`](Self::cancel). Mirrors `DialogResult.Canceled`.
    #[inline]
    pub fn is_canceled(&self) -> bool {
        self.canceled
    }

    /// `true` for [`ok`](Self::ok) and [`ok_with`](Self::ok_with).
    #[inline]
    pub fn is_ok(&self) -> bool {
        !self.canceled
    }

    /// `true` when the result carries a payload.
    #[inline]
    pub fn has_data(&self) -> bool {
        self.data.is_some()
    }

    /// Borrows the payload as `T`, or `None` when absent or of another type.
    #[inline]
    pub fn data<T: Any + Send + Sync>(&self) -> Option<&T> {
        self.data.as_ref().and_then(|b| b.downcast_ref::<T>())
    }

    /// Takes the payload as `T`, or `None` when absent or of another type.
    pub fn take_data<T: Any + Send + Sync>(&mut self) -> Option<T> {
        self.data
            .take()
            .and_then(|b| b.downcast::<T>().ok())
            .map(|b| *b)
    }
}

impl Default for DialogResult {
    /// Mirrors MudBlazor's `default(DialogResult)`, which is `Cancel()`.
    fn default() -> Self {
        Self::cancel()
    }
}

impl fmt::Debug for DialogResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DialogResult")
            .field("canceled", &self.canceled)
            .field("has_data", &self.data.is_some())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ok_is_not_canceled_and_has_no_data() {
        let r = DialogResult::ok();
        assert!(r.is_ok());
        assert!(!r.is_canceled());
        assert!(!r.has_data());
        assert_eq!(r.data::<i32>(), None);
    }

    #[test]
    fn cancel_is_canceled_and_has_no_data() {
        let r = DialogResult::cancel();
        assert!(r.is_canceled());
        assert!(!r.is_ok());
        assert!(!r.has_data());
    }

    #[test]
    fn default_is_cancel() {
        assert!(DialogResult::default().is_canceled());
    }

    #[test]
    fn ok_with_carries_a_payload() {
        let r = DialogResult::ok_with(42u32);
        assert!(r.is_ok());
        assert!(r.has_data());
        assert_eq!(r.data::<u32>(), Some(&42));
        assert_eq!(r.data::<i32>(), None);
    }

    #[test]
    fn take_data_consumes_the_payload() {
        let mut r = DialogResult::ok_with("saved".to_string());
        assert_eq!(r.take_data::<String>(), Some("saved".to_string()));
        assert!(!r.has_data());
        assert_eq!(r.take_data::<String>(), None);
    }

    #[test]
    fn take_data_returns_none_for_the_wrong_type() {
        let mut r = DialogResult::ok_with(1u8);
        assert_eq!(r.take_data::<String>(), None);
    }

    #[test]
    fn debug_does_not_leak_the_payload() {
        let r = DialogResult::ok_with(7);
        let s = format!("{r:?}");
        assert!(s.contains("canceled: false"));
        assert!(s.contains("has_data: true"));
        assert!(!s.contains('7'));
    }
}