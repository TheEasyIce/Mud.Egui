//! `MudColorPicker` — a text field with a popover colour picker.

mod color_channels;
mod color_picker_action;
mod color_picker_state;
mod mud_color_picker;

pub use color_channels::ColorChannels;
pub use color_picker_action::ColorPickerAction;
pub use color_picker_state::ColorPickerState;
pub use mud_color_picker::{
    default_palette, ColorPickerResponse, MudColorPicker, DEFAULT_PALETTE,
    DEFAULT_THROTTLE_MS, PLANE_SIZE, QUICK_COLOR_COUNT, STRIP_HEIGHT, SWATCH_SIZE,
};