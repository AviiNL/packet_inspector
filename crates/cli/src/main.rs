use proxy_lib::Packet;
use proxy_lib::Proxy;
use tracing::Level;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .init();

    let proxy = Proxy::new("0.0.0.0:25566".parse()?, "127.0.0.1:25565".parse()?);
    let receiver = proxy.subscribe();

    tokio::spawn(async move {
        proxy.run().await?;

        Ok::<(), anyhow::Error>(())
    });

    // consumer
    tokio::spawn(async move {
        while let Ok(packet) = receiver.recv_async().await {
            log(&packet);
        }
    });

    tokio::signal::ctrl_c().await.unwrap();

    Ok(())
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
