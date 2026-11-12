use egui::Ui;
use mud_egui_navigation::page::{Layout, NavigationContext};

pub struct BlankLayout {
    
}


impl Layout for BlankLayout {
    fn show(&mut self, ui: &mut Ui, ctx: &NavigationContext, body: &mut dyn FnMut(&mut Ui)) {
        body(ui);
    }
}