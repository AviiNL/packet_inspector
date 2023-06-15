use std::sync::{Arc, OnceLock};

use packet_registry::Packet;
use tokio::net::TcpStream;
use tracing::Level;

use crate::{client::process, packet_registry::PacketRegistry};

mod client;
mod packet_io;
mod packet_registry;

include!(concat!(env!("OUT_DIR"), "/packets.rs"));

static PACKET_REGISTRY: OnceLock<Arc<PacketRegistry>> = OnceLock::new();

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .init();

    PACKET_REGISTRY.get_or_init(|| {
        let registry = PacketRegistry::new();
        registry.register_all(&STD_PACKETS);
        Arc::new(registry)
    });

    let listener = tokio::net::TcpListener::bind("0.0.0.0:25566")
        .await
        .unwrap();

    println!("Listening on port 25566");

    tokio::spawn(async move {
        while let Ok((client, _addr)) = listener.accept().await {
            tokio::spawn(async move {
                let server = TcpStream::connect("127.0.0.1:25560").await.unwrap();

                if let Err(e) = process(client, server).await {
                    if !e.to_string().contains("unexpected end of file") {
                        // bit meh to do it like this but it works
                        tracing::error!("Error: {:?}", e);
                    }
                }

                Ok::<(), anyhow::Error>(())
            });
        }
    });

    // consumer
    tokio::spawn(async move {
        let receiver = {
            let registry = PACKET_REGISTRY.get().unwrap();
            registry.subscribe()
        };

        while let Ok(packet) = receiver.recv() {
            log(&packet);
        }
    });

    tokio::signal::ctrl_c().await.unwrap();
}

fn log(packet: &Packet) {
    tracing::debug!(
        "{:?} -> [{:?}] 0x{:0>2X} \"{}\" {:?}",
        packet.side,
        packet.state,
        packet.id,
        packet.name,
        truncated(format!("{:?}", packet.data), 512)
    );
}

fn truncated(string: String, max_len: usize) -> String {
    if string.len() > max_len {
        format!("{}...", &string[..max_len])
    } else {
        string.to_string()
    }
}
