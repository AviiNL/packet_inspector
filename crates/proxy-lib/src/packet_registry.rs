use std::sync::RwLock;

use bytes::Bytes;
use valence_core::protocol::decode::PacketFrame;

pub struct PacketRegistry {
    packets: RwLock<Vec<Packet>>,
    receiver: flume::Receiver<Packet>,
    sender: flume::Sender<Packet>,
}

#[allow(unused)]
impl PacketRegistry {
    pub fn new() -> Self {
        let (sender, receiver) = flume::unbounded::<Packet>();

        Self {
            packets: RwLock::new(Vec::new()),
            receiver,
            sender,
        }
    }

    pub fn subscribe(&self) -> flume::Receiver<Packet> {
        self.receiver.clone()
    }

    pub fn register(&self, packet: Packet) {
        self.packets.write().unwrap().push(packet);
    }

    // register_all(takes an array of packets)
    pub fn register_all(&self, packets: &[Packet]) {
        self.packets.write().unwrap().extend_from_slice(packets);
    }

    fn get_specific_packet(&self, side: PacketSide, state: PacketState, packet_id: i32) -> Packet {
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

    pub fn process(
        &self,
        side: PacketSide,
        state: PacketState,
        threshold: Option<u32>,
        packet: &PacketFrame,
    ) -> anyhow::Result<()> {
        let mut p = self.get_specific_packet(side, state, packet.id);
        p.data = Some(packet.body.clone().freeze());

        // store in received_packets
        self.sender.send(p)?;

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct Packet {
    pub side: PacketSide,
    pub state: PacketState,
    pub id: i32,
    pub name: &'static str,
    /// Uncompressed packet data
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
