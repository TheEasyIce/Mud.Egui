//! Private load-phase enum for [`MudImage`](super::MudImage).
//!
//! Kept in its own file per the project convention that every enum lives in
//! its own file. `pub(crate)` — not part of the public API.

/// The resolved state of one image source in the current frame.
///
/// Produced by [`MudImage::poll_source`](super::MudImage) from the raw
/// [`egui::load::TextureLoadResult`] the loader system returns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ImageLoadPhase {
    /// The image is still being fetched / decoded. Show a placeholder and
    /// request a repaint.
    Pending,
    /// The image is ready. Paint it.
    Ready,
    /// The image failed to load (network error, decode error, missing file).
    Failed,
    /// No source is configured — nothing to paint for this slot.
    Absent,
}

impl ImageLoadPhase {
    /// Whether the phase requires a repaint on the next frame.
    #[inline]
    pub(crate) fn needs_repaint(self) -> bool {
        matches!(self, Self::Pending)
    }

    /// Whether the image can be painted at its natural size.
    #[inline]
    pub(crate) fn is_ready(self) -> bool {
        matches!(self, Self::Ready)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pending_needs_repaint() {
        assert!(ImageLoadPhase::Pending.needs_repaint());
        assert!(!ImageLoadPhase::Ready.needs_repaint());
        assert!(!ImageLoadPhase::Failed.needs_repaint());
        assert!(!ImageLoadPhase::Absent.needs_repaint());
    }

    #[test]
    fn only_ready_is_ready() {
        assert!(ImageLoadPhase::Ready.is_ready());
        assert!(!ImageLoadPhase::Pending.is_ready());
        assert!(!ImageLoadPhase::Failed.is_ready());
        assert!(!ImageLoadPhase::Absent.is_ready());
    }
}