use super::{SharedState, View, Window};

pub struct HexView {}

impl Window for HexView {
    fn new() -> Self {
        Self {}
    }

    fn name(&self) -> &'static str {
        "Hex Viewer"
    }
}

impl View for HexView {
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut SharedState) {
        ui.heading("Hex Viewer");
        // ui.text_edit_singleline((&mut state.about_text).into());
    }
}
