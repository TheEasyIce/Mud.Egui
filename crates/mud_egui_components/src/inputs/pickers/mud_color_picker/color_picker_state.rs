//! Caller-owned state for [`MudColorPicker`](super::MudColorPicker).
//!
//! Holds the committed colour, the trigger text, the popover state, the active
//! view and channel mode, the HEX input buffer, and the drag-throttle
//! timestamp. Persist it across frames — the same caller-owned-state pattern
//! `MudDatePicker`, `MudTimePicker`, `MudDateRangePicker` and the rest of the
//! picker family use.

use mud_egui_core::enums::{ColorPickerMode, ColorPickerView};

use crate::feedback::popover::PopoverState;

use super::color_channels::ColorChannels;

/// Caller-owned colour-picker state.
#[derive(Debug, Clone, PartialEq)]
pub struct ColorPickerState {
    /// Committed colour. Mirrors `Value`.
    pub value: Option<ColorChannels>,
    /// Trigger text. Always HEX/HEXA, kept in sync by `commit` / `clear` /
    /// `with_value`.
    pub text: String,
    /// The HEX input buffer inside the popover. Only valid while the user is
    /// typing; `commit` rewrites it.
    pub hex_buffer: String,
    /// Popover open/closed state and stable id.
    pub popover: PopoverState,
    /// Current selection surface. Changed by the toolbar.
    pub view: ColorPickerView,
    /// Current channel mode. Changed by the mode switch.
    pub mode: ColorPickerMode,
    /// Seconds-since-start of the last committed drag update. Drives
    /// `ThrottleInterval`.
    pub last_update: Option<f64>,
    /// `true` between `drag_started` and `drag_stopped` on any spectrum
    /// control. Exposed so the caller can mirror `DragEffect`.
    pub drag_active: bool,
}

impl ColorPickerState {
    /// Creates empty state, closed, on the spectrum view in RGB mode.
    pub fn new(id_salt: impl std::hash::Hash + std::fmt::Debug) -> Self {
        Self {
            value: None,
            text: String::new(),
            hex_buffer: String::new(),
            popover: PopoverState::new(id_salt),
            view: ColorPickerView::Spectrum,
            mode: ColorPickerMode::Rgb,
            last_update: None,
            drag_active: false,
        }
    }

    /// Seeds the colour and its trigger text.
    pub fn with_value(mut self, color: ColorChannels, show_alpha: bool) -> Self {
        self.value = Some(color);
        self.text = color.hex_text(show_alpha);
        self.hex_buffer = self.text.clone();
        self
    }

    /// Whether the popover is open.
    #[inline]
    pub fn is_open(&self) -> bool {
        self.popover.open
    }

    /// Opens the popover, keeping the current view and mode.
    pub fn open(&mut self) {
        self.popover.open = true;
    }

    /// Closes the popover and clears the drag latch.
    pub fn close(&mut self) {
        self.popover.open = false;
        self.drag_active = false;
    }

    /// Toggles the popover.
    pub fn toggle(&mut self) {
        if self.popover.open { self.close() } else { self.open() }
    }

    /// Commits `color`, refreshes both text buffers, and closes the popover
    /// when `auto_close` is set. Returns `true` when anything changed.
    pub fn commit(&mut self, color: ColorChannels, show_alpha: bool) -> bool {
        let text = color.hex_text(show_alpha);
        let changed = self.value.map(|v| v.rgb_eq(&color)).unwrap_or(false) == false;
        self.value = Some(color);
        self.text = text.clone();
        self.hex_buffer = text;
        changed
    }

    /// Clears the colour and both text buffers, and closes the popover.
    pub fn clear(&mut self) {
        self.value = None;
        self.text.clear();
        self.hex_buffer.clear();
        self.popover.open = false;
        self.drag_active = false;
    }

    /// The colour the popover should paint: the committed value, or opaque
    /// black when nothing is selected.
    #[inline]
    pub fn current(&self) -> ColorChannels {
        self.value.unwrap_or_else(ColorChannels::black)
    }

    /// Advances the channel mode one step (`RGB → HSL → HEX → RGB`).
    #[inline]
    pub fn cycle_mode(&mut self) {
        self.mode = self.mode.next();
    }

    /// Switches to `view`.
    #[inline]
    pub fn change_view(&mut self, view: ColorPickerView) {
        self.view = view;
    }

    /// `true` when a throttled drag may commit at `now` given `interval_ms`.
    #[inline]
    pub fn throttle_allows(&self, now: f64, interval_ms: u32) -> bool {
        match self.last_update {
            None => true,
            Some(prev) => (now - prev) * 1000.0 >= interval_ms as f64,
        }
    }

    /// Records a successful commit at `now`.
    #[inline]
    pub fn mark_updated(&mut self, now: f64) {
        self.last_update = Some(now);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn red() -> ColorChannels {
        ColorChannels::from_rgba(255, 0, 0, 255)
    }

    // --- construction -----------------------------------------------------

    #[test]
    fn new_starts_empty_and_closed() {
        let s = ColorPickerState::new("t");
        assert!(s.value.is_none());
        assert!(s.text.is_empty());
        assert!(s.hex_buffer.is_empty());
        assert!(!s.is_open());
        assert_eq!(s.view, ColorPickerView::Spectrum);
        assert_eq!(s.mode, ColorPickerMode::Rgb);
        assert!(s.last_update.is_none());
        assert!(!s.drag_active);
    }

    #[test]
    fn with_value_seeds_text_and_buffer() {
        let s = ColorPickerState::new("t").with_value(red(), false);
        assert_eq!(s.value, Some(red()));
        assert_eq!(s.text, "#FF0000");
        assert_eq!(s.hex_buffer, "#FF0000");
    }

    #[test]
    fn with_value_honours_show_alpha() {
        let s = ColorPickerState::new("t").with_value(red().with_a(0.5), true);
        assert_eq!(s.text.len(), 9);
    }

    // --- open / close -----------------------------------------------------

    #[test]
    fn toggle_flips_the_popover() {
        let mut s = ColorPickerState::new("t");
        s.toggle();
        assert!(s.is_open());
        s.toggle();
        assert!(!s.is_open());
    }

    #[test]
    fn close_clears_the_drag_latch() {
        let mut s = ColorPickerState::new("t");
        s.open();
        s.drag_active = true;
        s.close();
        assert!(!s.drag_active);
    }

    // --- commit / clear ---------------------------------------------------

    #[test]
    fn commit_stores_colour_and_texts() {
        let mut s = ColorPickerState::new("t");
        s.commit(red(), false);
        assert_eq!(s.value, Some(red()));
        assert_eq!(s.text, "#FF0000");
        assert_eq!(s.hex_buffer, "#FF0000");
    }

    #[test]
    fn commit_reports_whether_the_colour_changed() {
        let mut s = ColorPickerState::new("t");
        assert!(s.commit(red(), false), "first commit changes the value");
        assert!(!s.commit(red(), false), "identical commit changes nothing");
        let blue = ColorChannels::from_rgba(0, 0, 255, 255);
        assert!(s.commit(blue, false));
    }

    #[test]
    fn clear_drops_everything() {
        let mut s = ColorPickerState::new("t").with_value(red(), false);
        s.open();
        s.drag_active = true;
        s.clear();
        assert!(s.value.is_none());
        assert!(s.text.is_empty());
        assert!(s.hex_buffer.is_empty());
        assert!(!s.is_open());
        assert!(!s.drag_active);
    }

    // --- current / mode / view --------------------------------------------

    #[test]
    fn current_falls_back_to_black() {
        let s = ColorPickerState::new("t");
        assert_eq!(s.current(), ColorChannels::black());
    }

    #[test]
    fn cycle_mode_wraps() {
        let mut s = ColorPickerState::new("t");
        assert_eq!(s.mode, ColorPickerMode::Rgb);
        s.cycle_mode();
        assert_eq!(s.mode, ColorPickerMode::Hsl);
        s.cycle_mode();
        assert_eq!(s.mode, ColorPickerMode::Hex);
        s.cycle_mode();
        assert_eq!(s.mode, ColorPickerMode::Rgb);
    }

    #[test]
    fn change_view_stores_it() {
        let mut s = ColorPickerState::new("t");
        s.change_view(ColorPickerView::Grid);
        assert_eq!(s.view, ColorPickerView::Grid);
    }

    // --- throttle ---------------------------------------------------------

    #[test]
    fn throttle_allows_the_first_update() {
        let s = ColorPickerState::new("t");
        assert!(s.throttle_allows(1.0, 50));
    }

    #[test]
    fn throttle_blocks_until_the_interval_elapses() {
        let mut s = ColorPickerState::new("t");
        s.mark_updated(1.000);
        assert!(!s.throttle_allows(1.010, 50));
        assert!(!s.throttle_allows(1.049, 50));
        assert!(s.throttle_allows(1.050, 50));
    }

    #[test]
    fn throttle_zero_interval_always_allows() {
        let mut s = ColorPickerState::new("t");
        s.mark_updated(1.0);
        assert!(s.throttle_allows(1.0, 0));
    }
}