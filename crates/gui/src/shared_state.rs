#![allow(clippy::mutable_key_type)]

use egui::Context;
use proxy_lib::Packet;
use std::{collections::HashMap, sync::RwLock};

pub enum Event {
    StartListening,
    StopListening,
    PacketReceived,
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
    #[serde(skip)]
    pub selected_packet: Option<usize>,
    #[serde(skip)]
    pub packets: RwLock<Vec<Packet>>,
    #[serde(skip)]
    pub(super) receiver: Option<flume::Receiver<Event>>,
    #[serde(skip)]
    sender: Option<flume::Sender<Event>>,
    #[serde(skip)]
    pub ctx: Option<Context>,
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
            selected_packet: None,
            packets: RwLock::new(Vec::new()),
            receiver: Some(receiver),
            sender: Some(sender),
            ctx: None,
        }
    }
}

#[allow(unused)]
impl SharedState {
    pub fn new(ctx: Context) -> Self {
        Self {
            ctx: Some(ctx),
            ..Self::default()
        }
    }
    pub(super) fn merge(mut self, other: Self) -> Self {
        self.ctx = other.ctx;
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

    pub fn send_event(&self, event: Event) {
        if let Some(sender) = &self.sender {
            sender.send(event);
        }
    }
}
