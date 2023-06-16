use super::{SharedState, View, Window};

pub struct About {}

impl Window for About {
    fn new() -> Self {
        Self {}
    }

    fn name(&self) -> &'static str {
        "About"
    }
}

impl View for About {
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut SharedState) {
        ui.heading("Valence Packet Inspector");
        // ui.text_edit_singleline((&mut state.about_text).into());
    }
}
