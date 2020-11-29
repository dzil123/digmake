#![allow(non_snake_case, dead_code)]

use digmake::se::{from_bytes_debug, VarInt};
use serde::Deserialize;

type Optional<T> = Option<T>;

pub fn test_ClientHandshake() {
    #[derive(Deserialize, Debug)]
    struct ClientHandshake<'a> {
        packet_id: u8,
        client_version: VarInt,
        server_hostname: &'a str,
        server_port: u16,
        next_state: VarInt,
    }

    let DATA = vec![
        0x00, 0xf2, 0x05, 0x09, 0x6c, 0x6f, 0x63, 0x61, 0x6c, 0x68, 0x6f, 0x73, 0x74, 0x63, 0xdd,
        0x01,
    ];

    let data = from_bytes_debug::<ClientHandshake>(&DATA);

    dbg!(&data);

    let data = data.1.unwrap();
    let host = data.server_hostname;
    // std::mem::drop(DATA);

    dbg!(DATA.as_ptr(), host.as_ptr());
}

pub fn test_enum() {
    #[derive(Deserialize, Debug)]
    struct Main {
        value: Enum,
    }

    #[derive(Deserialize, Debug)]
    enum Enum {
        Empty,         // variant 0x00
        Alfa(i32),     // variant 0x01
        Bravo(String), // variant 0x02, etc
        Charlie(u16, i16),
        Delta(u8, i16, VarInt),
        TooLong(i32, i32), // expected to fail with EOF
    }

    for DATA in (0..6) // for each possible enum variant
        .into_iter()
        .inspect(|val| println!("trying 0x{:02x}", val))
        .map(|val| vec![val, 0x03, 0x00, 0x12, 0x56])
    {
        let data = from_bytes_debug::<Main>(&DATA);

        println!("{:?}", &data);
    }
}

pub fn test_optional() {
    // note: 'u16' isnt special, you can swap out for any type, tuple, struct, whatever (with the correct data of course)
    // only the Packet struct is used in the actual code, the other structs are for demo only

    // Our idiomatic Rust struct:
    #[derive(Deserialize, Debug)]
    struct Packet {
        widgets: Option<u16>,
    }

    // How wiki.vg would describe this struct:
    struct PacketWiki {
        has_widgets: bool,      // Whether the packet has widgets
        widgets: Optional<u16>, // The number of widgets in the packet. Not present if previous boolean is false.
    }

    // possible ways to parse this ^:
    struct PacketWithoutWidgets {
        has_widgets: bool, // 0x00 literal
    }
    // OR
    struct PacketWithWidgets {
        has_widgets: bool, // 0x01 literal
        widgets: u16,
    }

    // So when the deserializer is asked for an Option<T>, it reads a bool first, then tells the visitor that.
    // If its true, the visitor will then ask us to deserialize T

    let DATA_WITHOUT_WIDGETS = vec![0x00];
    let DATA_WITH_WIDGETS = vec![0x01, 0x44, 0x44];

    println!(
        "Without widgets:\n    {:?}",
        from_bytes_debug::<Packet>(&DATA_WITHOUT_WIDGETS)
    );
    println!(
        "With widgets:\n    {:?}",
        from_bytes_debug::<Packet>(&DATA_WITH_WIDGETS)
    );

    /*
    Runtime result:
        Without widgets:
            ([], Ok(Packet { widgets: None }))
        With widgets:
            ([], Ok(Packet { widgets: Some(17476) }))
    */
}

pub fn test_Position() {
    use digmake::se::Position;

    let DATA = vec![
        // x i26                               | z 26
        0b00000000, 0b00000000, 0b00000000, 0b00100000,
        //                           | y 12
        0b00000000, 0b00000000, 0b00000000, 0b00000000,
    ];

    let data = from_bytes_debug::<Position>(&DATA);
    dbg!(&data);
    dbg!(data.1.unwrap());
}

pub fn test_bytearry() {
    let DATA = vec![63, 231, 92, 12];

    let data = from_bytes_debug::<&[u8]>(&DATA);
    dbg!(&data);
    dbg!(data.1.unwrap());
}

pub fn test_uuid() {
    use serde::{self, Deserialize};

    #[derive(Deserialize)]
    #[serde(remote = "uuid::Uuid")]
    struct Uuid(
        #[serde(getter = "uuid::Uuid::as_bytes")] [u8; 16], //
    );

    impl From<Uuid> for uuid::Uuid {
        fn from(uuid: Uuid) -> Self {
            Self::from_bytes(uuid.0)
        }
    }

    #[derive(Deserialize, Debug)]
    struct Struct {
        #[serde(with = "Uuid")]
        value: uuid::Uuid,
    }

    let DATA = vec![
        63, 231, 92, 12, //
        63, 231, 92, 12, //
        63, 231, 92, 12, //
        63, 231, 92, 12,
    ];

    let data = from_bytes_debug::<Struct>(&DATA);
    dbg!(&data);
    dbg!(data.1.unwrap());
}

/*
pub fn test_varint() {
    use serde::{self, Deserialize};

    #[derive(Deserialize)]
    struct VarInt {
        foo: i8,
    }

    #[derive(Deserialize)]
    struct Struct {
        #[serde(with = "VarInt")]
        value: i32,
    }
}
*/

pub fn test_vec() {
    use serde::{self, Deserialize};

    #[derive(Deserialize, Debug)]
    struct Struct<'a> {
        foo: Vec<(i8, i16)>,
        bar: &'a [u8],
    }

    let DATA = vec![
        2, // len of foo
        1, 2, 3, // foo[0]
        4, 5, 6, // foo[1]
        // bytearray: "The length of this array must be inferred from the packet length."
        7, 8, // rest of data is put into bar
    ];

    let data = from_bytes_debug::<Struct>(&DATA);
    dbg!(&data);
    dbg!(data.1.unwrap());
}
