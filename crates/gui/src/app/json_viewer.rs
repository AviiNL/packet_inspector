use super::{SharedState, Tab, View};

pub struct JsonView {}

impl Tab for JsonView {
    fn new() -> Self {
        Self {}
    }

    fn name(&self) -> &'static str {
        "Json Viewer"
    }
}

impl View for JsonView {
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut SharedState) {
        ui.heading("Json Viewer");
        // ui.text_edit_singleline((&mut state.about_text).into());
    }
}
