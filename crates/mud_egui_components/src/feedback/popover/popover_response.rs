//! The result of one [`MudPopover::show`](super::MudPopover::show) frame.

use egui::Response;

/// The result of one [`MudPopover::show`](super::MudPopover::show) frame.
///
/// When the popover is closed, `response` is the parent `Ui`'s zero-sized
/// hover response — inspect `open` to distinguish.
#[derive(Debug)]
pub struct PopoverResponse {
    /// Whether the popover is open **after** this frame's dismiss handling
    /// was applied.
    pub open: bool,
    /// The popover's own `egui::Response`. Zero-sized when closed.
    pub response: Response,
}

impl PopoverResponse {
    /// Whether the popover closed this frame (i.e. `open` is `false` now).
    #[inline]
    pub fn is_closed(&self) -> bool {
        !self.open
    }
}

#[cfg(test)]
mod tests {
    // PopoverResponse wraps egui::Response which is not easily
    // constructible without a Ui. The is_closed helper is trivially correct;
    // exercising it via a live frame is not worth a headless test.
}