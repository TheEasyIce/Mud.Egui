//! `MudDateRangePicker` — a dual text field with a popover range calendar.

mod date_range;
mod date_range_picker_action;
mod date_range_picker_state;
mod mud_date_range_picker;
mod range_position;

pub use date_range::{format_one, inclusive_days, DateRange};
pub use date_range_picker_action::DateRangePickerAction;
pub use date_range_picker_state::DateRangePickerState;
pub use mud_date_range_picker::{
    day_at, range_contains_disabled, second_click_valid, span_count, DateRangePickerResponse,
    MudDateRangePicker, SEPARATOR_GAP, SEPARATOR_SIZE,
};