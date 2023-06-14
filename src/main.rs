use std::sync::{Arc, OnceLock};

use tokio::net::TcpStream;
use tracing::Level;

use crate::{client::process, packet_registry::PacketRegistry};

mod client;
mod logger;
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

    let logger = Arc::new(logger::Logger::new());

    while let Ok((client, _addr)) = listener.accept().await {
        let logger = logger.clone();
        tokio::spawn(async move {
            let server = TcpStream::connect("127.0.0.1:25560").await.unwrap();

            if let Err(e) = process(client, server, logger).await {
                tracing::error!("Error: {:?}", e);
            }

            Ok::<(), anyhow::Error>(())
        });
    }
}
