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
        Self {
            listener_addr: "127.0.0.1:25566".to_string(),
            server_addr: "127.0.0.1:25565".to_string(),
            is_listening: false,
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

        self
    }

    pub fn send_event(&mut self, event: Event) {
        if let Some(sender) = &self.sender {
            sender.send(event);
        }
    }
}
