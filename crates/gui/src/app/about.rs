use super::{View, Window};

pub struct About {}

impl Window for About {
    fn name(&self) -> &'static str {
        "About"
    }
}

impl View for About {
    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("egui");
        ui.label("egui is designed to be easy to use, portable, and fast.");
    }
}
