use eframe::egui;
use mud_egui_navigation::navigation::NavigationManager;
use mud_egui_navigation::page::{NavigationContext, Page};

#[derive(Default)]
pub struct HomePage;

impl Page for HomePage {
    fn show(&mut self, ui: &mut egui::Ui, _ctx: &NavigationContext, nav: &mut NavigationManager) {

    }
}