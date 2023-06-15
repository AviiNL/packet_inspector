use std::sync::{Arc, RwLock};

use tokio::net::TcpStream;
use valence_core::protocol::{decode::PacketFrame, Decode, Packet};
use valence_network::packet::{
    HandshakeC2s, HandshakeNextState, LoginCompressionS2c, LoginSuccessS2c,
};

use crate::{
    logger::Logger,
    packet_io::PacketIo,
    packet_registry::{PacketSide, PacketState},
    PACKET_REGISTRY,
};

pub async fn process(
    client: TcpStream,
    server: TcpStream,
    _logger: Arc<Logger>,
) -> anyhow::Result<()> {
    let client = PacketIo::new(client, PacketSide::Serverbound);
    let server = PacketIo::new(server, PacketSide::Clientbound);

    let (mut client_reader, mut client_writer) = client.split();
    let (mut server_reader, mut server_writer) = server.split();

    let current_state_inner = Arc::new(RwLock::new(PacketState::Handshaking));
    let threshold_inner = Arc::new(RwLock::new(None));

    let current_state = current_state_inner.clone();
    let threshold = threshold_inner.clone();
    let c2s = tokio::spawn(async move {
        loop {
            client_reader.set_compression(*threshold.read().unwrap());
            server_writer.set_compression(*threshold.read().unwrap());
            // client to server handling
            let packet = client_reader.recv_packet_raw().await?;

            let state = {
                let state = current_state.read().unwrap();
                *state
            };

            if state == PacketState::Handshaking {
                if let Some(handshake) = extrapolate_packet::<HandshakeC2s>(&packet) {
                    *current_state.write().unwrap() = match handshake.next_state {
                        HandshakeNextState::Status => PacketState::Status,
                        HandshakeNextState::Login => PacketState::Login,
                    };
                }
            }

            let state = {
                let state = current_state.read().unwrap();
                *state
            };

            PACKET_REGISTRY.get().unwrap().process(
                crate::packet_registry::PacketSide::Serverbound,
                state,
                *threshold.read().unwrap(),
                &packet,
            )?;

            server_writer.send_packet_raw(&packet).await?;
        }

        #[allow(unreachable_code)]
        Ok::<(), anyhow::Error>(())
    });

    let current_state = current_state_inner.clone();
    let threshold = threshold_inner.clone();
    let s2c = tokio::spawn(async move {
        loop {
            server_reader.set_compression(*threshold.read().unwrap());
            client_writer.set_compression(*threshold.read().unwrap());
            // server to client handling
            let packet = server_reader.recv_packet_raw().await?;

            let state = {
                let state = current_state.read().unwrap();
                *state
            };

            if state == PacketState::Login {
                if let Some(compression) = extrapolate_packet::<LoginCompressionS2c>(&packet) {
                    if compression.threshold.0 >= 0 {
                        *threshold.write().unwrap() = Some(compression.threshold.0 as u32);
                    }
                };

                if let Some(_) = extrapolate_packet::<LoginSuccessS2c>(&packet) {
                    *current_state.write().unwrap() = PacketState::Play;
                };
            }

            PACKET_REGISTRY.get().unwrap().process(
                crate::packet_registry::PacketSide::Clientbound,
                state,
                *threshold.read().unwrap(),
                &packet,
            )?;

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

fn extrapolate_packet<'a, P>(packet: &'a PacketFrame) -> Option<P>
where
    P: Packet + Decode<'a> + Clone,
{
    if packet.id != P::ID {
        return None;
    }

    let mut r = &packet.body[..];
    let packet = P::decode(&mut r).ok()?;
    Some(packet.clone())
}
