use std::{env, fs, path::Path, process::Command};

use anyhow::Context;
use proc_macro2::TokenStream;
use quote::quote;
use serde::Deserialize;

#[derive(Deserialize)]
struct Packet {
    name: String,
    side: String,
    state: String,
    id: i32,
}

pub fn main() -> anyhow::Result<()> {
    let packets: Vec<Packet> = serde_json::from_str(include_str!("./assets/packets.json"))?;

    let mut consts = TokenStream::new();

    let len = packets.len();

    let mut p: Vec<TokenStream> = Vec::new();

    for packet in packets {
        let stripped_name = packet.name.strip_suffix("Packet").unwrap_or(&packet.name);

        let id = packet.id;
        let side = match &packet.side {
            s if s == "clientbound" => quote! { crate::packet_registry::PacketSide::Clientbound },
            s if s == "serverbound" => quote! { crate::packet_registry::PacketSide::Serverbound },
            _ => unreachable!(),
        };

        let state = match &packet.state {
            s if s == "handshaking" => quote! { crate::packet_registry::PacketState::Handshaking },
            s if s == "status" => quote! { crate::packet_registry::PacketState::Status },
            s if s == "login" => quote! { crate::packet_registry::PacketState::Login },
            s if s == "play" => quote! { crate::packet_registry::PacketState::Play },
            _ => unreachable!(),
        };

        // const STD_PACKETS = [PacketSide::Client(PacketState::Handshaking(Packet{..})), ..];
        p.push(quote! {
            crate::packet_registry::Packet {
                id: #id,
                side: #side,
                state: #state,
                name: #stripped_name,
                data: None,
            }
        });
    }

    consts.extend([quote! {
        pub const STD_PACKETS: [crate::packet_registry::Packet; #len] = [
            #(#p),*
        ];
    }]);

    write_generated_file(consts, "packets.rs")?;

    Ok(())
}

pub fn write_generated_file(content: TokenStream, out_file: &str) -> anyhow::Result<()> {
    let out_dir = env::var_os("OUT_DIR").context("failed to get OUT_DIR env var")?;
    let path = Path::new(&out_dir).join(out_file);
    let code = content.to_string();

    fs::write(&path, code)?;

    // Try to format the output for debugging purposes.
    // Doesn't matter if rustfmt is unavailable.
    let _ = Command::new("rustfmt").arg(path).output();

    Ok(())
}
