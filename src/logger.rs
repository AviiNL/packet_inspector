use std::sync::RwLock;

use valence_core::protocol::decode::PacketFrame;

use crate::packet_registry::PacketSide;

pub struct Logger {
    pub packets: RwLock<Vec<(String, PacketSide, PacketFrame)>>,
}

impl Logger {
    pub fn new() -> Self {
        Self {
            packets: RwLock::new(Vec::new()),
        }
    }

    pub fn log(&self, name: String, direction: PacketSide, packet: PacketFrame) {
        tracing::info!("Packet: {} {:?} {:?}", name, direction, packet.id);

        self.packets
            .write()
            .unwrap()
            .push((name, direction, packet));
    }
}
