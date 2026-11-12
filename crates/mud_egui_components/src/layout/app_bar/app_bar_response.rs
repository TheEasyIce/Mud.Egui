//! The result of one [`MudAppBar::show`](super::MudAppBar::show) frame.

use egui::Response;

/// The result of one [`MudAppBar::show`](super::MudAppBar::show) frame.
#[derive(Debug)]
pub struct AppBarResponse {
    /// The bar's own `Response`.
    pub response: Response,
    /// The bar's height in points (64 normally, 48 when `dense`).
    pub height: f32,
}