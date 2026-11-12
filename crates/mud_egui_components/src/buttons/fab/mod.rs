//! `MudFab` and `MudMenuFab` — floating action buttons.

mod fab_menu_action;
mod menu_fab_state;
mod mud_fab;
mod mud_menu_fab;

pub use fab_menu_action::FabMenuAction;
pub use menu_fab_state::MenuFabState;
pub use mud_fab::{FabMetrics, MudFab, DEFAULT_BOTTOM, DEFAULT_RIGHT};
pub use mud_menu_fab::{
    action_side, total_height, MenuFabResponse, MudMenuFab, ACTION_GAP, ACTION_INSET,
    TRIGGER_OFFSET,
};