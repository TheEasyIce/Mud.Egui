//! `MudLayout` and `MudMainContent` — the full-screen layout shell.

mod layout_state;
mod mud_layout;
mod mud_main_content;

pub use layout_state::LayoutState;
pub use mud_layout::MudLayout;
pub use mud_main_content::{compute_main_rect, MudMainContent};