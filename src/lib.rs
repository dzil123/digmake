pub mod logic;
pub mod se;
mod util;

use crate::se::VarInt;
use se::{Error, Result};
use std::io::{BufRead, BufReader};
use std::net::TcpStream;

// todo: custom enum for packet problems

/*
struct Packet {
    packet_len: VarInt, // length of next two fields
    packet_id: VarInt,
    data: Vec<u8>
}
*/

fn read_packet_id_len<T>(mut reader: &mut T) -> Result<(i32, usize)>
where
    T: BufRead,
{
    let packet_len = VarInt::_parse_as_usize(&mut reader)?;
    // util::count_reads counts the number of bytes read from the reader by the closure
    let (packet_id, id_len) = util::count_reads(&mut reader, |reader| VarInt::_parse(reader));
    let packet_id = packet_id?;
    println!(
        "packet_len: {}, packet_id: 0x{:02X}, id_len: {}",
        packet_len, packet_id, id_len
    );

    let packet_len = packet_len.checked_sub(id_len).ok_or_else(|| {
        Error::Packet(format!(
            "length of packet id varint ({}) is larger than length of entire packet ({})",
            id_len, packet_len
        ))
    })?;

    Ok((packet_id, packet_len))
}

pub fn read_packet<T, F>(mut reader: &mut T, mut handler: F) -> Result<()>
where
    T: BufRead,
    F: FnMut(i32, &[u8]) -> Result<()>,
{
    let (packet_id, packet_len) = read_packet_id_len(&mut reader)?;

    if packet_len == 0 {
        handler(packet_id, &[])
    } else {
        let temp_buffer = reader.fill_buf()?;
        if temp_buffer.len() >= packet_len {
            // we are lucky, the entire packet is in the buffer
            // we can pass the buffer into the deserializer with 0 copies
            let result = handler(packet_id, &temp_buffer[0..packet_len]);
            reader.consume(packet_len);
            result
        } else {
            // the entire packet is not in the buffer
            // it is not possible to read more into the buffer until it is emptied

            let mut storage = vec![0u8; packet_len]; // allocate memory on the heap
            reader.read_exact(&mut storage)?; // possibly multiple syscalls
            handler(packet_id, &mut storage)
        }
    }
}

struct Example {
    reader: BufReader<TcpStream>,
}

impl Example {
    fn new(stream: TcpStream) -> Self {
        Self {
            reader: BufReader::new(stream),
        }
    }

    fn read_packet(&mut self) -> Result<()> {
        let reader = &mut self.reader;

        read_packet(reader, |packet_id, buffer| {
            match packet_id {
                0x00 => {
                    #[derive(serde::Deserialize, Debug)]
                    struct Handshake<'a> {
                        protocol_version: VarInt,
                        address: &'a str,
                        port: u16,
                        next_state: VarInt,
                    }

                    let packet: Handshake = se::from_bytes(buffer)?;
                    dbg!(&packet);
                    assert_eq!(packet.protocol_version.0, 754);
                    assert_eq!(packet.next_state.0, 1); // status

                    // read_packet(reader, |packet_id, buffer| {
                    //     assert_eq!(packet_id, 0);
                    //     let packet: () = se::from_bytes(buffer)?;
                    //     Ok(())
                    // })?;

                    // self.reader.get_mut().write_all(&[0xFF, 0x00])?;
                }
                _ => return Err(Error::Packet("unknown packet type".to_string())),
            }
            Ok(())
        })
    }
}
