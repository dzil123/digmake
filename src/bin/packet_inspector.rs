use digmake::se::{from_bytes, from_bytes_debug, Error, Input, Result, VarInt};
use std::fmt::{Debug, Display};
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;

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

struct Data {
    is_server: bool,
    data: Vec<u8>,
}

impl Data {
    fn new(is_server: bool, data: Vec<u8>) -> Self {
        Data { is_server, data }
    }
}

fn do_one_packet<T: BufRead>(mut reader: &mut T, is_server: bool) -> Result<()> {
    println!();
    digmake::read_packet(&mut reader, |packet_id, buffer| {
        println!("Packet id: 0x{:02X}", packet_id);
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

                #[derive(serde::Deserialize, Debug)]
                struct TeleportConfirm {
                    teleport_id: VarInt,
                }

                match read_packet::<Handshake>(buffer) {
                    Ok(packet) => show_packet_dbg(packet),
                    Err(_) => match read_packet::<LoginStart>(buffer) {
                        Ok(packet) => show_packet_dbg(packet),
                        Err(_) => show_packet_dbg(read_packet::<TeleportConfirm>(buffer)?),
                    },
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
    let mut reader = BufReader::new(&*data.data);

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

fn remove_whitespace(s: &str) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect()
}

// read from a wireshark tcp stream dump in yaml format
fn read_data_from_file<P: AsRef<Path>>(filename: P) -> Vec<Data> {
    let data: serde_yaml::Value = {
        let file = File::open(filename).unwrap();
        let data = serde_yaml::from_reader(&file);
        file.sync_all().unwrap();
        data.unwrap()
    };

    let data = data.as_mapping().unwrap();
    let mut output = Vec::with_capacity(data.len()); // data.len() == tcp packets, upper limit on number of Datas
    let mut data = data.into_iter();

    let mut read_one_physical_packet = || -> Option<Data> {
        let (key, value) = match data.next() {
            Some(val) => val,
            None => return None,
        };
        let (key, value) = (key.as_str().unwrap(), value.as_str().unwrap());

        let is_server = {
            // wow i hope this is consistent
            if key.starts_with("peer0") {
                false
            } else if key.starts_with("peer1") {
                true
            } else {
                panic!()
            }
        };
        let value = remove_whitespace(value);
        println!(
            "read physical packet: {} {:?}...",
            is_server,
            value.chars().take(50).collect::<String>()
        );
        let packet_data = base64::decode(&value).unwrap();

        Some(Data::new(is_server, packet_data))
    };

    // combine all consecutive packets from one peer into a single packet
    // because logical mc packets can span many physical tcp packets.
    // and although each Data struct can have many mc packets,
    // a packet cannot be split into multiple Data structs.

    if let Some(mut data) = read_one_physical_packet() {
        loop {
            let mut next_data = match read_one_physical_packet() {
                Some(d) => d,
                None => {
                    output.push(data);
                    break;
                }
            };

            if data.is_server == next_data.is_server {
                data.data.append(&mut next_data.data);
            } else {
                output.push(data);
                data = next_data;
            }
        }
    }

    output
}

fn main() {
    let data = read_data_from_file("packet_full.yaml");
    do_all_data(&data).unwrap();
}
