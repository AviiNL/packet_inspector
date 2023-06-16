#![allow(clippy::mutable_key_type)]

use proxy_lib::Packet;
use std::collections::HashMap;

pub enum Event {
    StartListening,
    StopListening,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct SharedState {
    pub listener_addr: String,
    pub server_addr: String,
    #[serde(skip)]
    pub is_listening: bool,

    pub packet_filter: HashMap<Packet, bool>,

    // pub listener_addr: String,
    // pub server_addr: String,
    // #[serde(skip)]
    // current_packet: Packet
    // #[serde(skip)]
    // all_packets: Vec<Packet>
    #[serde(skip)]
    pub(super) receiver: Option<flume::Receiver<Event>>,
    #[serde(skip)]
    sender: Option<flume::Sender<Event>>,
}

impl Default for SharedState {
    fn default() -> Self {
        let (sender, receiver) = flume::unbounded();

        let mut packet_filter = HashMap::new();

        for p in proxy_lib::STD_PACKETS.iter() {
            packet_filter.insert(p.clone(), true);
        }

        Self {
            listener_addr: "127.0.0.1:25566".to_string(),
            server_addr: "127.0.0.1:25565".to_string(),
            is_listening: false,
            packet_filter,
            receiver: Some(receiver),
            sender: Some(sender),
        }
    }
}

#[allow(unused)]
impl SharedState {
    pub(super) fn merge(mut self, other: Self) -> Self {
        self.sender = other.sender;
        self.receiver = other.receiver;

        // make a backup of self.packet_filter

        let mut packet_filter = HashMap::new();
        // iterate over proxy_lib::STD_PACKETS
        for p in proxy_lib::STD_PACKETS.iter() {
            // if the packet is in the current packet_filter
            if let Some(v) = self
                .packet_filter
                .iter()
                .find(|(k, _)| k.id == p.id && k.side == p.side && k.state == p.state)
                .map(|(_, v)| v)
            {
                // insert it into packet_filter
                packet_filter.insert(p.clone(), *v);
            } else {
                // otherwise insert it into packet_filter with a default value of true
                packet_filter.insert(p.clone(), true);
            }
        }

        self.packet_filter = packet_filter;

        self
    }

    pub fn send_event(&mut self, event: Event) {
        if let Some(sender) = &self.sender {
            sender.send(event);
        }
    }
}
