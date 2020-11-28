use digmake::se::{from_bytes, from_bytes_debug, Error, Input, Result, VarInt};
use std::fmt::{Debug, Display};
use std::io::{BufRead, BufReader, Read};

fn read_first<T>(slice: &[T]) -> &[T] {
    &slice[..10.min(slice.len())]
}

fn read_packet<'a, T: serde::de::Deserialize<'a>>(buffer: Input<'a>) -> Result<T> {
    println!("Packet of type {}:", std::any::type_name::<T>(),);
    let (rest_input, packet) = from_bytes_debug(buffer);
    let packet = packet?;
    if rest_input.len() > 0 {
        println!(
            "warning: unread length {}: {:?}",
            rest_input.len(),
            read_first(rest_input)
        );
    }

    Ok(packet)
}

fn show_packet_dbg<T: Debug>(packet: T) {
    println!("{:#?}", packet);
}
fn show_packet_dbg_min<T: Debug>(packet: T) {
    println!("{:?}", packet);
}
fn show_packet_dsp<T: Display>(packet: T) {
    println!("{}", packet);
}

struct Data<'a> {
    is_server: bool,
    data: &'a str,
}

impl<'a> Data<'a> {
    fn new(is_server: bool, data: &'a str) -> Self {
        Data { is_server, data }
    }
}

fn do_one_packet<T: BufRead>(mut reader: &mut T, is_server: bool) -> Result<()> {
    println!();
    digmake::read_packet(&mut reader, |packet_id, buffer| {
        println!("Packet id: {}", packet_id);
        println!("    {:?}...", &buffer[..10.min(buffer.len())]);
        print!("Sent by ");
        match is_server {
            true => println!("server"),
            false => println!("client"),
        }

        if buffer.len() == 0 {
            println!("ignoring len 0 packet");
            return Ok(());
        }

        match (packet_id, is_server) {
            (0x00, false) => {
                #[derive(serde::Deserialize, Debug)]
                struct Handshake<'a> {
                    protocol_version: VarInt,
                    address: &'a str,
                    port: u16,
                    next_state: VarInt,
                }

                #[derive(serde::Deserialize, Debug)]
                struct LoginStart<'a> {
                    name: &'a str,
                }

                match read_packet::<Handshake>(buffer) {
                    Ok(packet) => show_packet_dbg(packet),
                    Err(_) => {
                        println!("failed 0x00 client packet as Handshake, try LoginStart");
                        let packet: LoginStart = read_packet(buffer)?;
                        show_packet_dbg(packet);
                    }
                }
            }
            (0x00, true) => {
                #[derive(serde::Deserialize, Debug)]
                struct Response<'a> {
                    json: &'a str,
                }

                let packet: Response = read_packet(buffer)?;
                show_packet_dsp(packet.json);
            }
            (0x01, is_pong) => {
                #[derive(serde::Deserialize, Debug)]
                struct PingPong(i64);

                let packet: PingPong = read_packet(buffer)?;
                show_packet_dbg(packet);
            }
            _ => {
                println!("unknown packet");
            }
        }
        Ok(())
    })
}

fn is_reader_not_eof<T: BufRead>(reader: &mut T) -> Result<bool> {
    Ok(reader.fill_buf()?.len() > 0) // if buffer is empty, and attempt to read more into it read 0 bytes, then eof
}

fn do_one_data(data: &Data) -> Result<()> {
    let bytes = hex::decode(&*data.data)?;
    let mut reader = BufReader::new(&*bytes);

    while is_reader_not_eof(&mut reader)? {
        do_one_packet(&mut reader, data.is_server)?;
    }

    Ok(())
}

fn do_all_data(datas: &[Data]) -> Result<()> {
    for data in datas {
        do_one_data(data)?;
    }

    Ok(())
}

fn get_data() -> Vec<Data<'static>> {
    todo!()
}

fn main() {
    let data = get_data();
    do_all_data(&data).unwrap();
}
