use digmake::se::{from_bytes, VarInt};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct MyStruct {
    bruh: VarInt,
}

fn main() {
    let DATA: Vec<u8> = vec![0xFF, 'a' as u8];
    let data: Result<MyStruct, _> = from_bytes(&DATA);

    dbg!(data);
}
