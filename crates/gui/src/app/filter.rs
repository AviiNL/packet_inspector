use proxy_lib::PacketState;

//
use super::{SharedState, View, Window};

pub struct Filter {}

impl Window for Filter {
    fn new() -> Self {
        Self {}
    }

    fn name(&self) -> &'static str {
        "Filters"
    }
}

impl View for Filter {
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut SharedState) {
        // only Handshake packets
        ui.heading("Handshaking");
        for p in proxy_lib::STD_PACKETS
            .iter()
            .filter(|p| p.state == PacketState::Handshaking)
        {
            if ui.checkbox(&mut true, p.name).changed() {
                // changed = true;
                ui.ctx().request_repaint();
            }
        }

        // only Status packets
        ui.heading("Status");
        for p in proxy_lib::STD_PACKETS
            .iter()
            .filter(|p| p.state == PacketState::Status)
        {
            if ui.checkbox(&mut true, p.name).changed() {
                // changed = true;
                ui.ctx().request_repaint();
            }
        }

        // only Login packets
        ui.heading("Login");
        for p in proxy_lib::STD_PACKETS
            .iter()
            .filter(|p| p.state == PacketState::Login)
        {
            if ui.checkbox(&mut true, p.name).changed() {
                // changed = true;
                ui.ctx().request_repaint();
            }
        }

        // only Play packets
        ui.heading("Play");
        for p in proxy_lib::STD_PACKETS
            .iter()
            .filter(|p| p.state == PacketState::Play)
        {
            if ui.checkbox(&mut true, p.name).changed() {
                // changed = true;
                ui.ctx().request_repaint();
            }
        }
    }
}
