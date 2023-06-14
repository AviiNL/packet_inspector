use std::sync::RwLock;

use bytes::Bytes;

pub struct PacketRegistry {
    pub packets: RwLock<Vec<Packet>>,
}

#[allow(unused)]
impl PacketRegistry {
    pub fn new() -> Self {
        Self {
            packets: RwLock::new(Vec::new()),
        }
    }

    pub fn register(&self, packet: Packet) {
        self.packets.write().unwrap().push(packet);
    }

    // register_all(takes an array of packets)
    pub fn register_all(&self, packets: &[Packet]) {
        self.packets.write().unwrap().extend_from_slice(packets);
    }

    pub fn get_specific_packet(
        &self,
        side: PacketSide,
        state: PacketState,
        packet_id: i32,
    ) -> Packet {
        self.packets
            .read()
            .unwrap()
            .iter()
            .find(|packet| packet.id == packet_id && packet.side == side && packet.state == state)
            .unwrap_or(&Packet {
                side,
                state,
                id: packet_id,
                name: "Unknown Packet",
                data: None,
            })
            .clone()
    }
}

#[derive(Clone, Debug)]
pub struct Packet {
    pub side: PacketSide,
    pub state: PacketState,
    pub id: i32,
    pub name: &'static str,
    pub data: Option<Bytes>,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum PacketState {
    Handshaking,
    Status,
    Login,
    Play,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum PacketSide {
    Clientbound,
    Serverbound,
}
