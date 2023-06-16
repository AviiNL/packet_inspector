use super::{SharedState, Tab, View};

pub struct PacketList {}

impl Tab for PacketList {
    fn new() -> Self {
        Self {}
    }

    fn name(&self) -> &'static str {
        "Packets"
    }
}

impl View for PacketList {
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut SharedState) {
        ui.heading("Streaming Packet List");
        ui.label("Imagine a table-like view thats scrolling down as fast as it can when packets are being intercepted");
        // ui.text_edit_singleline((&mut state.about_text).into());
    }
}
