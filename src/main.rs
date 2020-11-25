use digmake::se::{from_bytes, VarInt};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct ClientHandshake<'a> {
    packet_id: u8,
    client_version: VarInt,
    server_hostname: &'a str,
    server_port: u16,
    next_state: VarInt,
}

fn main() {
    let DATA: Vec<u8> = vec![
        0x00, 0xf2, 0x05, 0x09, 0x6c, 0x6f, 0x63, 0x61, 0x6c, 0x68, 0x6f, 0x73, 0x74, 0x63, 0xdd,
        0x01,
    ];
    let data: Result<ClientHandshake, _> = from_bytes(&DATA);

    dbg!(&data);

    let data = data.unwrap();
    let host = data.server_hostname;
    // std::mem::drop(DATA);

    dbg!(DATA.as_ptr(), host.as_ptr());
}
