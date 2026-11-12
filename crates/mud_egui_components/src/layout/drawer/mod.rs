//! `MudDrawer` and `MudDrawerHeader` — navigation panels docked to a layout edge.

mod drawer_response;
mod drawer_state;
mod effective_variant;
mod mud_drawer;
mod mud_drawer_header;
mod side;

pub use drawer_response::DrawerResponse;
pub use drawer_state::DrawerState;
pub use mud_drawer::{
    MudDrawer, DEFAULT_HEIGHT, DEFAULT_MINI_HEIGHT, DEFAULT_MINI_WIDTH, DEFAULT_WIDTH,
};
pub use mud_drawer_header::{
    MudDrawerHeader, DENSE_DRAWER_HEADER_HEIGHT, DRAWER_HEADER_HEIGHT,
    DRAWER_HEADER_PADDING,
};