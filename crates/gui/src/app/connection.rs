use super::{SharedState, View, Window};

pub struct Connection {}

impl Window for Connection {
    fn new() -> Self {
        Self {}
    }

    fn name(&self) -> &'static str {
        "Connection"
    }
}

impl View for Connection {
    fn ui(&mut self, ui: &mut egui::Ui, state: &mut SharedState) {
        // The input fields will be read-only when connected

        ui.label("Listener Address");
        ui.text_edit_singleline(&mut state.listener_addr);
        ui.label("Server Address");
        ui.text_edit_singleline(&mut state.server_addr);

        // button text changes depending on start/stop state
        if ui.button("Start Listening").clicked() {
            // Send an event indicating start/stop listening
        }
    }
}
