//! `MudDialog` — a modal overlay that presents information, a choice, or input.

mod dialog_control;
mod dialog_options;
mod dialog_result;
mod dialog_state;
mod mud_dialog;

pub use dialog_control::DialogControl;
pub use dialog_options::DialogOptions;
pub use dialog_result::DialogResult;
pub use dialog_state::DialogState;
pub use mud_dialog::{
    dialog_max_width_px, position_origin, resolve_panel_size, DialogResponse, MudDialog,
    MAX_SCREEN_FRACTION, MIN_DIALOG_HEIGHT, MIN_DIALOG_WIDTH, PADDING_BOTTOM, PADDING_X,
    TITLE_ROW_HEIGHT,
};