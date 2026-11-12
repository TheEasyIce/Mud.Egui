use eframe::egui;
use mud_egui_navigation::navigation::NavigationManager;
use mud_egui_navigation::page::{NavigationContext, Page};

#[derive(Default)]
pub struct NotFoundPage;

impl Page for NotFoundPage {
    fn show(&mut self, ui: &mut egui::Ui, ctx: &NavigationContext, nav: &mut NavigationManager) {
        ui.heading("Page not found");
        ui.label(format!("Path: {}", ctx.path));
        if ui.button("Home").clicked() {
            let _ = nav.navigate_to("/", false);
        }
    }
}