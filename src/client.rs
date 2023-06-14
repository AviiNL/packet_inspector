use std::sync::Arc;

use tokio::net::TcpStream;
use valence_core::protocol::Decode;
use valence_network::packet::{
    HandshakeC2s, HandshakeNextState, LoginCompressionS2c, LoginSuccessS2c,
};

use crate::{
    logger::Logger,
    packet_io::{Direction, PacketIo},
    packet_registry::PacketState,
};

const TRUNCATE_LENGTH: usize = 255;

pub async fn process(
    client: TcpStream,
    server: TcpStream,
    _logger: Arc<Logger>,
) -> anyhow::Result<()> {
    let mut current_state = PacketState::Handshaking;

    let client = PacketIo::new(client, Direction::Serverbound);
    let server = PacketIo::new(server, Direction::Clientbound);

    let (mut client_reader, mut client_writer) = client.split();
    let (mut server_reader, mut server_writer) = server.split();

    // handle handshake and compression packets manually before starting the play loops
    let packet = client_reader.recv_packet_raw().await?;
    tracing::debug!(
        "client to server: {:?} {:?}",
        current_state,
        truncated(format!("{:?}", packet), TRUNCATE_LENGTH)
    );
    server_writer.send_packet_raw(&packet).await?;

    let mut r = &packet.body[..];
    let handshake = HandshakeC2s::decode(&mut r)?;

    match handshake.next_state {
        HandshakeNextState::Status => {
            current_state = PacketState::Status;
        }
        HandshakeNextState::Login => {
            current_state = PacketState::Login;

            let packet = client_reader.recv_packet_raw().await?; // LoginHelloC2s
            tracing::debug!(
                "client to server: {:?} {:?}",
                current_state,
                truncated(format!("{:?}", packet), TRUNCATE_LENGTH)
            );
            server_writer.send_packet_raw(&packet).await?; // LoginHelloC2

            // server replies with LoginCompressionS2c, which we need to intercept
            let packet = server_reader.recv_packet_raw().await?;
            let mut r = &packet.body[..];
            let compression = LoginCompressionS2c::decode(&mut r)?; // if this fails, encryption is enabled and we can not continue

            let threshold = compression.threshold.0 as u32;
            client_reader.set_compression(Some(threshold));
            client_writer.set_compression(Some(threshold));
            server_reader.set_compression(Some(threshold));
            server_writer.set_compression(Some(threshold));

            tracing::debug!(
                "server to client: {:?} {:?}",
                current_state,
                truncated(format!("{:?}", packet), TRUNCATE_LENGTH)
            );
            client_writer.send_packet_raw(&packet).await?;

            // expect a LoginSuccessS2c from the server
            let packet = server_reader.recv_packet_raw().await?;
            tracing::debug!(
                "server to client: {:?} {:?}",
                current_state,
                truncated(format!("{:?}", packet), TRUNCATE_LENGTH)
            );
            client_writer.send_packet_raw(&packet).await?;

            current_state = PacketState::Play;
        }
    }

    // these loops _should_ only contain Play packets
    let c2s = tokio::spawn(async move {
        loop {
            // client to server handling
            let packet = client_reader.recv_packet_raw().await?;
            tracing::debug!(
                "client to server: {:?} {:?}",
                current_state,
                truncated(format!("{:?}", packet), TRUNCATE_LENGTH)
            );
            server_writer.send_packet_raw(&packet).await?;
        }

        #[allow(unreachable_code)]
        Ok::<(), anyhow::Error>(())
    });

    let s2c = tokio::spawn(async move {
        loop {
            // server to client handling
            let packet = server_reader.recv_packet_raw().await?; // the server isnt giving us a reply
            tracing::debug!(
                "server to client: {:?} {:?}",
                current_state,
                truncated(format!("{:?}", packet), TRUNCATE_LENGTH)
            );
            client_writer.send_packet_raw(&packet).await?;
        }

        #[allow(unreachable_code)]
        Ok::<(), anyhow::Error>(())
    });

    // wait for either to finish
    tokio::select! {
        res = c2s => res?,
        res = s2c => res?,
    }
}

fn truncated(string: String, max_len: usize) -> String {
    if string.len() > max_len {
        format!("{}...", &string[..max_len])
    } else {
        string.to_string()
    }
}
