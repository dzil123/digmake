use std::io::BufRead;
use std::io::BufReader;
use std::io::Read;
use std::net::{TcpListener, TcpStream};
use std::thread::sleep;
use std::time::Duration;

fn handle_client(stream: TcpStream) {
    stream.set_nonblocking(true).unwrap();
    dbg!(&stream);

    // let mut result = vec![];
    // dbg!(stream.read_to_end(&mut result).unwrap());
    // dbg!(result);
    let time = Duration::from_millis(100);
    // Read::bytes() makes a syscall for each byte, use BufRead instead
    for (i, b) in stream.bytes().enumerate() {
        print!("{:2}  ", i);
        match b {
            Ok(b) => println!("0x{0:02x}  0b{0:08b}  {0:}", b),
            Err(e) => println!("error {:?}", e),
        }
        sleep(time);
    }
    println!("closed\n");
}

fn handle_client2(mut stream: TcpStream) {
    // stream.set_nonblocking(true).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_millis(50)))
        .unwrap();

    let time = Duration::from_millis(100);
    // let mut iter = stream.bytes();
    let mut byte = 0;

    loop {
        match stream.read(std::slice::from_mut(&mut byte)) {
            Ok(value) => match value {
                0 => println!("closed"),
                1 => println!("0x{0:02x}  0b{0:08b}  {0:}", byte),
                x => println!("how how {} {}", x, byte),
            },
            Err(err) => {
                println!("Error: {:?}", err);
            }
        }
        match stream.peek(std::slice::from_mut(&mut byte)) {
            Ok(value) => match value {
                0 => println!("  closed"),
                1 => println!("  0x{0:02x}  0b{0:08b}  {0:}", byte),
                x => println!("  how how {} {}", x, byte),
            },
            Err(err) => {
                println!("  Error: {:?}", err);
            }
        }
        sleep(time);
    }
}

fn dbg<T: std::fmt::Debug>(val: T) {
    println!("{:?}", val);
}

fn handle_client_buf(stream: TcpStream) {
    let mut stream = BufReader::new(stream);
    loop {
        dbg((1, stream.capacity()));
        dbg((2, stream.buffer()));
        dbg((3, stream.fill_buf()));

        if stream.buffer().len() == 0 {
            println!("closed");
            return;
        }

        dbg((4, stream.buffer()));
        stream.consume(usize::MAX);
    }
}

/*
fn uhh() {
    let len: VarInt = bufread.read()?; // probably without serde
    bufread.consume(len of varint read);

    let len = len as i32;
    let buffer = bufread.fill_buf()?; // get buffer without copying, borrowed
    // if we call deserialize without enough data, it will give eof error
    if buffer.len() < len {
        // the buffer does not have the entire packet, which will give eof error
        // it is not possible to read more data into bufread before consuming current data
        let owned_buffer: [u8; len]; // array, vec, smolvec?
        buffer.read_exact(&mut owned_buffer)?;
        buffer = &owned_buffer;
    }

    // now we are guaranteed to have at least the entire current packet in buffer
    let type:VarInt = buffer.read();
    return (type, buffer); // match on statemachine and type to get struct type
    // todo: ensure length consumed by buffer == len?
    // also this wont work with nonblocking or with read timeout
}
*/

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:25565")?;
    dbg!(&listener);

    // accept connections and process them serially
    for stream in listener.incoming() {
        handle_client_buf(stream?);
    }
    Ok(())
}
