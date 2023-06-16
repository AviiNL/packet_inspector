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
    fn ui(&mut self, ui: &mut egui::Ui, state: &mut SharedState) {
        state.packets.read().unwrap().iter().for_each(|packet| {
            ui.label(format!("Packet: {}", packet.name));
        });
    }
}
