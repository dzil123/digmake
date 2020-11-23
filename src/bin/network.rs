use std::io::Read;
use std::net::{TcpListener, TcpStream};

fn handle_client(stream: TcpStream) {
    dbg!(&stream);

    // let mut result = vec![];
    // dbg!(stream.read_to_end(&mut result).unwrap());
    // dbg!(result);

    for (i, b) in stream.bytes().enumerate() {
        print!("{:2}  ", i);
        match b {
            Ok(b) => println!("0x{0:02x}  0b{0:08b}  {0:}", b),
            Err(e) => println!("error {:?}", e),
        }
    }
    println!("closed\n");
}

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:25565")?;
    dbg!(&listener);

    // accept connections and process them serially
    for stream in listener.incoming() {
        handle_client(stream?);
    }
    Ok(())
}
