use super::{SharedState, Tab, View};

pub struct About {}

impl Tab for About {
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
