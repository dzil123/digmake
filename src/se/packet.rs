use super::VarInt;
use serde::Deserialize;
use std::convert::{TryFrom, TryInto};

// #[derive(Deserialize)]
// struct PacketRaw {
//     length: VarInt, // length of id + rest of packet
//     id: VarInt,
// }

// impl TryFrom<PacketRaw> for Packet {
//     type Error = String;
//     // type Error = std::num::TryFromIntError;

//     fn try_from(packet: PacketRaw) -> Result<Self, Self::Error> {
//         let id = packet.id.0;
//         let length = packet
//             .length
//             .0
//             .try_into()
//             .map_err(|err| format!("could not convert packet length i32 to u32 {:?}", err))?;

//         Ok(Packet { id, length })
//     }
// }

#[derive(Deserialize)]
// #[serde(try_from = "PacketRaw")]
pub struct Packet {
    id: i32,
    length: u32, // length of just the rest of packet, not including id
}

// impl Packet {
//     fn read(deserializer: &mut Deserializer) -> Self {
//         todo!()
//     }
// }
