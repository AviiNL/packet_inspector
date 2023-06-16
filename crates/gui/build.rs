use std::{collections::HashMap, env, fs, path::Path, process::Command};

use anyhow::Context;
use proc_macro2::TokenStream;
use quote::quote;
use serde::Deserialize;

#[allow(unused)]
#[derive(Deserialize)]
struct Packet {
    name: String,
    side: String,
    state: String,
    id: i32,
}

pub fn main() -> anyhow::Result<()> {
    let packets: Vec<Packet> =
        serde_json::from_str(include_str!("../proxy-lib/assets/packets.json"))?;

    // HashMap<side, HashMap<state, Vec<name>>>
    let grouped_packets = HashMap::<String, HashMap<String, Vec<String>>>::new();

    let mut grouped_packets = packets
        .into_iter()
        .fold(grouped_packets, |mut acc, packet| {
            let side = match packet.side.as_str() {
                "serverbound" => "Serverbound".to_string(),
                "clientbound" => "Clientbound".to_string(),
                _ => panic!("Invalid side"),
            };

            let state = match packet.state.as_str() {
                "handshaking" => "Handshaking".to_string(),
                "status" => "Status".to_string(),
                "login" => "Login".to_string(),
                "play" => "Play".to_string(),
                _ => panic!("Invalid state"),
            };

            let name = packet
                .name
                .strip_suffix("Packet")
                .unwrap_or(&packet.name)
                .to_string();

            // lowercase the last character of name
            let name = {
                let mut chars = name.chars();
                let last_char = chars.next_back().unwrap();
                let last_char = last_char.to_lowercase().to_string();
                let mut name = chars.collect::<String>();
                name.push_str(&last_char);
                name
            };

            // if the packet is clientbound, but the name does not ends with S2c, add it
            let name = if side == "Clientbound" && !name.ends_with("S2c") {
                format!("{}S2c", name)
            } else {
                name
            };

            // same for serverbound
            let name = if side == "Serverbound" && !name.ends_with("C2s") {
                format!("{}C2s", name)
            } else {
                name
            };

            let state_map = acc.entry(side).or_insert_with(HashMap::new);
            let id_map = state_map.entry(state).or_insert_with(Vec::new);
            id_map.push(name);

            acc
        });

    let mut generated = TokenStream::new();

    for (side, state_map) in grouped_packets.iter_mut() {
        let mut side_arms = TokenStream::new();
        for (state, id_map) in state_map.iter_mut() {
            let mut match_arms = TokenStream::new();

            for name in id_map.iter_mut() {
                let name = syn::parse_str::<syn::Ident>(&name).unwrap();

                match_arms.extend(quote! {
                    #name::ID => {
                        format!("{:#?}", #name::decode(&mut data).unwrap())
                    }
                });
            }

            let state = syn::parse_str::<syn::Ident>(&state).unwrap();

            side_arms.extend(quote! {
                PacketState::#state => match packet.id {
                    #match_arms
                    _ => NOT_AVAILABLE.to_string(),
                },
            });
        }

        if side == "Clientbound" {
            side_arms.extend(quote! {
                _ => NOT_AVAILABLE.to_string(),
            });
        }

        let side = syn::parse_str::<syn::Ident>(&side).unwrap();

        generated.extend(quote! {
            PacketSide::#side => match packet.state {
                #side_arms
            },
        });
    }

    // wrap generated in a function definition
    let generated = quote! {
        use proxy_lib::Packet as ProxyPacket;
        use valence::advancement::packet::*;
        use valence::client::action::*;
        use valence::client::command::*;
        use valence::client::custom_payload::*;
        use valence::client::hand_swing::*;
        use valence::client::interact_block::*;
        use valence::client::interact_entity::*;
        use valence::client::interact_item::*;
        use valence::client::keepalive::*;
        use valence::client::movement::*;
        use valence::client::packet::structure_block::*;
        use valence::client::packet::*;
        use valence::client::resource_pack::*;
        use valence::client::settings::*;
        use valence::client::status::*;
        use valence::client::teleport::*;
        use valence::client::title::*;
        use valence::entity::packet::*;
        use valence::instance::packet::*;
        use valence::inventory::packet::synchronize_recipes::*;
        use valence::inventory::packet::*;
        use valence::network::packet::*;
        use valence::particle::*;
        use valence::player_list::packet::*;
        use valence::protocol::packet::boss_bar::*;
        use valence::protocol::packet::chat::*;
        use valence::protocol::packet::command::*;
        use valence::protocol::packet::map::*;
        use valence::protocol::packet::scoreboard::*;
        use valence::protocol::packet::sound::*;
        use valence::registry::tags::*;
        use valence::world_border::packet::*;
        use valence::protocol::{Decode, Packet};

        pub fn packet_to_string(packet: &ProxyPacket) -> String {
            let bytes = packet.data.as_ref().unwrap();
            let mut data = &bytes.clone()[..];

            match packet.side {
                #generated
            }
        }
    };

    write_generated_file(generated, "packet_to_string.rs")?;

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
