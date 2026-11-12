//! The result of one [`MudDrawer::show`](super::MudDrawer::show) frame.

use egui::Response;

/// The result of one [`MudDrawer::show`](super::MudDrawer::show) frame.
#[derive(Debug)]
pub struct DrawerResponse {
    /// The drawer panel's own `Response`. Zero-sized when fully hidden.
    pub response: Response,
    /// `true` on the frame the drawer opened.
    pub opened: bool,
    /// `true` on the frame the drawer closed.
    pub closed: bool,
    /// Whether the drawer painted anything this frame.
    pub visible: bool,
    /// Width (or height, for top/bottom anchors) the drawer reserved in the
    /// layout this frame. `0` for overlay drawers.
    pub reserved_extent: f32,
}