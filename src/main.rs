use std::fmt::{Binary, Display, LowerHex};

fn main() {
    println!("{}", 253u8);
    println!("{}", 253u8 as i8);
    return;

    let values = [i32::MAX, i32::MIN, -1];
    for &x in &values {
        dbg(x);
        dbg2(&serialize_int(x));
    }
}

fn dbg2(res: &[u8]) {
    println!("{:?}", res);
    for x in res {
        dbg(x);
    }
    println!("------------\n");
}

fn dbg<T>(x: T)
where
    T: Display + Binary + LowerHex,
{
    println!("{}", x);
    println!("{:08x}", x);
    println!("{:032b}", x);
    println!();
}

fn serialize_int(value: i32) -> Vec<u8> {
    // keep the same bits, need unsigned for "logical" right shift semantics
    let mut value = value as u32;
    let mut res = vec![];

    loop {
        let mut temp: u8 = (value & 0b01111111) as u8;
        value >>= 7;

        if value != 0 {
            temp |= 0b10000000;
        }
        res.push(temp);
        if value == 0 {
            break;
        }
    }

    res
}

fn deserialize_int(value: Vec<u8>) -> i32 {
    unimplemented!();
}
