use digmake::se::Position;
use digmake::se::{from_bytes_debug, Input, Result, VarInt};
use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use serde::{self, Deserialize};

mod customvec {
    // Default Vec impl is a VarInt length followed by an array
    // This is for Vecs with the prefixed length of a different type

    use serde::de::Expected;
    use std::fmt;
    // why isnt serde::de::Expected implemented for more types
    struct Index(usize);

    impl Expected for Index {
        fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
            fmt::Display::fmt(&self.0, fmt)
        }
    }

    macro_rules! customvec_impl {
        ($name:ident, $type:ty) => {
            pub mod $name {
                use super::Index;
                use serde::de::{Deserialize, Deserializer, Error, SeqAccess, Unexpected, Visitor};
                use serde::ser::{Serialize, Serializer};
                use std::convert::TryFrom;
                use std::marker::PhantomData;

                pub fn deserialize<'de, D, T>(de: D) -> std::result::Result<Vec<T>, D::Error>
                where
                    D: Deserializer<'de>,
                    T: Deserialize<'de>,
                {
                    struct CustomVecVisitor<T> {
                        marker: PhantomData<T>,
                    }

                    impl<'de, T> Visitor<'de> for CustomVecVisitor<T>
                    where
                        T: Deserialize<'de>,
                    {
                        type Value = Vec<T>;

                        fn expecting(
                            &self,
                            formatter: &mut std::fmt::Formatter,
                        ) -> std::fmt::Result {
                            formatter.write_str(concat!(
                                "an array prefixed with its length as ",
                                stringify!($type)
                            ))
                        }

                        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                        where
                            A: SeqAccess<'de>,
                        {
                            let len: $type = match seq.next_element()? {
                                Some(x) => x,
                                None => {
                                    return Err(Error::missing_field(concat!(
                                        stringify!($type),
                                        " len"
                                    )))
                                }
                            };

                            let len = match usize::try_from(len) {
                                Ok(x) => x,
                                Err(_) => {
                                    return Err(Error::invalid_value(
                                        Unexpected::Signed(len.into()),
                                        &concat!("an ", stringify!($type), " > 0"),
                                    ))
                                }
                            };

                            let mut values = Vec::with_capacity(len);

                            for i in 0..len {
                                let val = match seq.next_element()? {
                                    Some(x) => x,
                                    None => {
                                        return Err(Error::invalid_length(len, &Index(i)));
                                    }
                                };

                                values.push(val);
                            }

                            Ok(values)
                        }
                    }

                    let visitor = CustomVecVisitor {
                        marker: PhantomData,
                    };

                    // the length is unknown and not actually 0,
                    // but this is good enough since my Deserializer doesnt look at the length param
                    // only works if (len, item0, item1, ...) == (len, (item0, item1, ...))
                    de.deserialize_tuple(0, visitor)
                }

                pub fn serialize<T, S>(val: &T, serializer: S) -> Result<S::Ok, S::Error>
                where
                    T: Serialize,
                    S: Serializer,
                {
                    todo!()
                }
            }
        };
    }

    customvec_impl!(short, i16);
    customvec_impl!(int, i32);
}

#[macro_use]
mod bigarray {
    // inspired by https://stackoverflow.com/a/48976823
    pub trait BigArray<'de>: Sized {
        fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
        where
            D: serde::de::Deserializer<'de>;

        fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
        where
            S: serde::ser::Serializer;
    }

    #[macro_export]
    macro_rules! big_array {
        ($len:expr) => {
            use serde::de::{Deserialize, Deserializer, Error, SeqAccess, Visitor};
            use serde::ser::{Serialize, Serializer};
            use std::fmt;
            use std::marker::PhantomData;

            impl<'de, T> BigArray<'de> for PhantomData<T>
            where
                T: Deserialize<'de>,
            {
                fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
                where
                    D: Deserializer<'de>,
                {
                    struct ArrayVisitor<T> {
                        element: PhantomData<T>,
                    }

                    impl<'de, T> Visitor<'de> for ArrayVisitor<T>
                    where
                        T: Deserialize<'de>,
                    {
                        type Value = PhantomData<T>;

                        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                            formatter.write_fmt(format_args!(
                                concat!("{} items of ", stringify!(T)),
                                $len
                            ))
                        }

                        fn visit_seq<A>(
                            self,
                            mut seq: A,
                        ) -> std::result::Result<Self::Value, A::Error>
                        where
                            A: SeqAccess<'de>,
                        {
                            for i in 0..$len {
                                let _: T = seq.next_element()?.ok_or_else(|| {
                                    Error::invalid_length(i + 1, &format!("{}", $len).as_str())
                                })?;
                            }
                            Ok(PhantomData)
                        }
                    }

                    let visitor = ArrayVisitor {
                        element: PhantomData,
                    };
                    deserializer.deserialize_tuple($len, visitor)
                }

                fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
                where
                    S: Serializer,
                {
                    todo!()
                }
            }
        };
    }
}

fn blocked_on(feature: &'static str) {
    println!("parsing is blocked on {} feature", feature);
}

fn blocked_on_nbt() {
    blocked_on("nbt");
}

#[derive(Deserialize, serde::Serialize)]
#[serde(remote = "uuid::Uuid")]
struct Uuid(#[serde(getter = "uuid::Uuid::as_bytes")] [u8; 16]);

impl From<Uuid> for uuid::Uuid {
    fn from(uuid: Uuid) -> Self {
        Self::from_bytes(uuid.0)
    }
}

fn read_first<T>(slice: &[T]) -> &[T] {
    &slice[..10.min(slice.len())]
}

fn read_packet<'a, T>(buffer: Input<'a>) -> Result<T>
where
    T: serde::Deserialize<'a> + serde::Serialize,
{
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

    let seri = digmake::se::serialize(&packet).unwrap();

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

fn serialize_test<T>(data: T, buffer: &[u8])
where
    T: serde::Serialize + Debug,
{
    show_packet_dbg(&buffer);
    show_packet_dbg(&data);

    use digmake::se::serialize;

    let output = serialize(data).unwrap();
    show_packet_dbg(&output);

    panic!();
}

fn do_one_packet<T: BufRead>(
    mut reader: &mut T,
    is_server: bool,
) -> std::result::Result<i32, digmake::se::Error> {
    digmake::read_packeta(&mut reader, |packet_id, buffer| {
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
                #[derive(serde::Deserialize, serde::Serialize, Debug)]
                struct Handshake<'a> {
                    protocol_version: VarInt,
                    address: &'a str,
                    port: u16,
                    next_state: VarInt,
                }

                #[derive(serde::Deserialize, serde::Serialize, Debug)]
                struct LoginStart<'a> {
                    name: &'a str,
                }

                #[derive(serde::Deserialize, serde::Serialize, Debug)]
                struct TeleportConfirm {
                    teleport_id: VarInt,
                }

                match read_packet::<Handshake>(buffer) {
                    Ok(packet) => {
                        show_packet_dbg(packet)
                        // serialize_test(packet, buffer);
                    }
                    Err(_) => match read_packet::<LoginStart>(buffer) {
                        Ok(packet) => show_packet_dbg(packet),
                        Err(_) => show_packet_dbg(read_packet::<TeleportConfirm>(buffer)?),
                    },
                }
            }
            (0x00, true) => {
                #[derive(serde::Deserialize, serde::Serialize, Debug)]
                struct Response<'a> {
                    json: &'a str,
                }

                let packet: Response = read_packet(buffer)?;
                show_packet_dsp(packet.json);
            }
            (0x01, _is_pong) => {
                #[derive(serde::Deserialize, serde::Serialize, Debug)]
                struct PingPong(i64);

                let packet: PingPong = read_packet(buffer)?;
                show_packet_dbg(packet);
            }
            (0x02, true) => {
                #[derive(serde::Deserialize, serde::Serialize, Debug)]
                struct LoginSuccess<'a> {
                    #[serde(with = "Uuid")]
                    uuid: uuid::Uuid,
                    username: &'a str,
                }

                let packet: LoginSuccess = read_packet(buffer)?;
                show_packet_dbg(packet);
            }
            (0x02, false) => {
                #[derive(serde::Deserialize, serde::Serialize, Debug)]
                struct SpawnLivingEntity {
                    entity_id: VarInt,
                    #[serde(with = "Uuid")]
                    entity_uuid: uuid::Uuid,
                    entity_type: VarInt,
                    x: i64,
                    y: i64,
                    z: i64,
                    yaw: u8,
                    pitch: u8,
                    head_pitch: u8,
                    velocity_x: i16,
                    velocity_y: i16,
                    velocity_z: i16,
                }

                let packet: SpawnLivingEntity = read_packet(buffer)?;
                show_packet_dbg(packet);
            }
            (0x05, false) => {
                #[derive(serde::Deserialize, serde::Serialize, Debug)]
                enum ChatMode {
                    Enabled,
                    CommandsOnly,
                    Hidden,
                }

                #[derive(serde::Deserialize, serde::Serialize, Debug)]
                enum Hand {
                    Left,
                    Right,
                }

                #[derive(serde::Deserialize, serde::Serialize, Debug)]
                struct ClientSettings {
                    locale: String,
                    view_distance: u8, // chunks
                    chat_mode: ChatMode,
                    chat_colors: bool,
                    displayed_skin: u8, // bitmask on skin parts
                    main_hand: Hand,
                }

                let packet: ClientSettings = read_packet(buffer)?;
                show_packet_dbg(packet);
            }
            (0x0B, false) => {
                #[derive(serde::Deserialize, serde::Serialize, Debug)]
                struct PluginMessageClient<'a> {
                    channel: String,
                    data: &'a [u8],
                }

                let packet: PluginMessageClient = read_packet(buffer)?;
                show_packet_dbg(packet);
            }
            (0x0D, true) => {
                #[derive(serde::Deserialize, serde::Serialize, Debug)]
                enum Difficulty {
                    Peaceful,
                    Easy,
                    Normal,
                    Hard,
                }

                #[derive(serde::Deserialize, serde::Serialize, Debug)]
                struct ServerDifficulty {
                    difficulty: Difficulty,
                    locked: bool,
                }

                let packet: ServerDifficulty = read_packet(buffer)?;
                show_packet_dbg(packet);
            }
            (0x10, true) => {
                #[derive(serde::Deserialize, serde::Serialize, Debug)]
                struct DeclareCommands {
                    node_len: VarInt,
                    // nodes: Vec<Node>,
                    // root_index: VarInt,
                }

                let packet: DeclareCommands = read_packet(buffer)?;
                println!("this packet impossible to parse: https://wiki.vg/Command_Data");
                show_packet_dbg(packet);
            }
            (0x12, false) => {
                #[derive(serde::Deserialize, serde::Serialize, Debug)]
                struct PlayerPosition {
                    pos: (f64, f64, f64),
                    on_ground: bool,
                }

                let packet: PlayerPosition = read_packet(buffer)?;
                show_packet_dbg(packet);
            }
            (0x13, false) => {
                #[derive(serde::Deserialize, serde::Serialize, Debug)]
                struct PlayerPosition {
                    pos: (f64, f64, f64),
                    yaw: f32,
                    pitch: f32,
                    on_ground: bool,
                }

                let packet: PlayerPosition = read_packet(buffer)?;
                show_packet_dbg(packet);
            }
            (0x13, true) => {
                #[derive(serde::Deserialize, serde::Serialize, Debug)]
                struct Slot {
                    item_id: VarInt,
                    item_count: u8,
                    nbt: (),
                }

                #[derive(serde::Deserialize, serde::Serialize, Debug)]
                struct WindowItems {
                    window_id: u8,
                    #[serde(with = "customvec::short")]
                    slots: Vec<Option<Slot>>,
                }

                let packet: WindowItems = read_packet(buffer)?;
                blocked_on_nbt();
                show_packet_dbg(packet);
            }
            (0x15, false) => {
                #[derive(serde::Deserialize, serde::Serialize, Debug)]
                struct PlayerMovement {
                    on_ground: bool,
                }

                let packet: PlayerMovement = read_packet(buffer)?;
                show_packet_dbg(packet);
            }
            (0x15, true) => {
                #[derive(serde::Deserialize, serde::Serialize, Debug)]
                struct Slot {
                    item_id: VarInt,
                    item_count: u8,
                    nbt: (),
                }

                #[derive(serde::Deserialize, serde::Serialize, Debug)]
                struct SetSlotInWindow {
                    window_id: i8,
                    slot: i16,
                    data: Option<Slot>,
                }

                let packet: SetSlotInWindow = read_packet(buffer)?;
                blocked_on_nbt();
                show_packet_dbg(packet);
            }
            (0x17, true) => {
                #[derive(serde::Deserialize, serde::Serialize, Debug)]
                struct PluginMessageServer<'a> {
                    channel: String,
                    data: &'a [u8],
                }

                let packet: PluginMessageServer = read_packet(buffer)?;
                show_packet_dbg(packet);
            }
            (0x1A, false) => {
                #[derive(serde_repr::Deserialize_repr, serde_repr::Serialize_repr, Debug)]
                #[repr(u8)]
                enum PlayerAbilities {
                    NotFlying = 0x00,
                    Flying = 0x02,
                }

                let packet: PlayerAbilities = read_packet(buffer)?;
                show_packet_dbg(packet);
            }
            (0x1A, true) => {
                #[derive(serde::Deserialize, serde::Serialize, Debug)]
                struct EntityStatus {
                    id: i32,
                    status: u8,
                }

                let packet: EntityStatus = read_packet(buffer)?;
                show_packet_dbg(packet);
            }
            (0x1F, true) | (0x10, false) => {
                #[derive(serde::Deserialize, serde::Serialize, Debug)]
                struct KeepAlive(i64);

                let packet: KeepAlive = read_packet(buffer)?;
                show_packet_dbg(packet);
            }
            (0x20, true) => {
                #[derive(serde::Deserialize, serde::Serialize, Debug)]
                struct ChunkData {
                    chunk_x: i32,
                    chunk_y: i32,
                    full_chunk: bool,
                    primary_bit_mask: VarInt,
                }

                let packet: ChunkData = read_packet(buffer)?;
                blocked_on_nbt();
                show_packet_dbg_min(packet);
            }
            (0x23, true) => {
                #[derive(serde::Deserialize, serde::Serialize)]
                struct LightArray(Vec<u8>);

                impl Debug for LightArray {
                    fn fmt(
                        &self,
                        fmt: &mut std::fmt::Formatter<'_>,
                    ) -> std::result::Result<(), std::fmt::Error> {
                        fmt.write_fmt(format_args!("[u8; {}]", self.0.len()))
                    }
                }

                #[derive(serde::Deserialize, serde::Serialize, Debug)]
                struct UpdateLight {
                    chunk_x: VarInt,
                    chunk_y: VarInt,
                    trust_edges: bool,
                    sky_light_mask: VarInt,
                    block_light_mask: VarInt,
                    empty_sky_light_mask: VarInt,
                    empty_block_light_mask: VarInt,
                    sky_light: LightArray,   // always 2048,
                    block_light: LightArray, // always 2048,
                }

                let packet: UpdateLight = read_packet(buffer)?;
                show_packet_dbg_min(packet);
            }
            (0x24, true) => {
                #[derive(serde_repr::Deserialize_repr, serde_repr::Serialize_repr, Debug)]
                #[repr(i8)]
                enum Gamemode {
                    Survival = 0,
                    Creative = 1,
                    Adventure = 2,
                    Spectator = 3,
                }

                #[derive(serde_repr::Deserialize_repr, serde_repr::Serialize_repr, Debug)]
                #[repr(i8)]
                enum PreviousGamemode {
                    Survival = 0,
                    Creative = 1,
                    Adventure = 2,
                    Spectator = 3,
                    None = -1,
                }

                // hack to skip a hardcoded # of bytes of unparsed nbt to read the rest of the packet
                use crate::bigarray::BigArray;
                big_array!(30746);

                #[derive(serde::Deserialize, serde::Serialize, Debug)]
                struct JoinGame {
                    entity_id: i32,
                    is_hardcore: bool,
                    gamemode: Gamemode,
                    prev_gamemode: PreviousGamemode,
                    worlds: Vec<String>,
                    #[serde(with = "BigArray")]
                    unparsed_nbt: std::marker::PhantomData<u8>,
                    spawn_world: String,
                    hashed_seed: i64,
                    max_players: VarInt,
                    view_distance: VarInt,
                    reduced_debug: bool,
                    not_immediate_respawn: bool,
                    is_debug: bool,
                    is_flat: bool,
                }

                let packet: JoinGame = read_packet(buffer)?;
                blocked_on_nbt();
                show_packet_dbg(packet);
            }
            (0x30, true) => {
                #[derive(serde::Deserialize, serde::Serialize, Debug)]
                struct PlayerAbilities {
                    flags: u8, // bitfield
                    fly_speed: f32,
                    fov_modifier: f32,
                }

                let packet: PlayerAbilities = read_packet(buffer)?;
                show_packet_dbg(packet);
            }
            (0x32, true) => {
                #[derive(serde::Deserialize, serde::Serialize, Debug)]
                struct Properties {
                    name: String,
                    value: String,
                    signature: Option<String>,
                }

                #[derive(serde_repr::Deserialize_repr, serde_repr::Serialize_repr, Debug)]
                #[repr(i8)]
                enum Gamemode {
                    Survival = 0,
                    Creative = 1,
                    Adventure = 2,
                    Spectator = 3,
                }

                #[derive(serde::Deserialize, serde::Serialize, Debug)]
                struct Add {
                    #[serde(with = "Uuid")]
                    uuid: uuid::Uuid,
                    name: String,
                    properties: Vec<Properties>,
                    gamemode: Gamemode,
                    ping: VarInt, // time, in ms
                    display_name: Option<String>,
                }

                #[derive(serde::Deserialize, serde::Serialize, Debug)]
                struct UpdateGamemode {
                    #[serde(with = "Uuid")]
                    uuid: uuid::Uuid,
                    gamemode: VarInt,
                }

                #[derive(serde::Deserialize, serde::Serialize, Debug)]
                struct UpdateLatency {
                    #[serde(with = "Uuid")]
                    uuid: uuid::Uuid,
                    ping: VarInt,
                }

                #[derive(serde::Deserialize, serde::Serialize, Debug)]
                struct UpdateDisplayName {
                    #[serde(with = "Uuid")]
                    uuid: uuid::Uuid,
                    display_name: Option<String>,
                }

                #[derive(serde::Deserialize, serde::Serialize, Debug)]
                struct RemovePlayer {
                    #[serde(with = "Uuid")]
                    uuid: uuid::Uuid,
                }

                #[derive(serde::Deserialize, serde::Serialize, Debug)]
                enum PlayerInfo {
                    Add(Vec<Add>),
                    UpdateGamemode(Vec<UpdateGamemode>),
                    UpdateLatency(Vec<UpdateLatency>),
                    UpdateDisplayName(Vec<UpdateDisplayName>),
                    RemovePlayer(Vec<RemovePlayer>),
                }

                let packet: PlayerInfo = read_packet(buffer)?;
                show_packet_dbg(packet);
            }
            (0x34, true) => {
                #[derive(serde::Deserialize, serde::Serialize, Debug)]
                struct PlayerPositionAndLook {
                    pos: (f64, f64, f64),
                    yaw: f32,
                    pitch: f32,
                    flags: u8,
                    teleport_id: VarInt,
                }

                let packet: PlayerPositionAndLook = read_packet(buffer)?;
                show_packet_dbg(packet);
            }
            (0x35, true) => {
                #[derive(serde::Deserialize, serde::Serialize, Debug)]
                struct Common {
                    crafting_book: bool,
                    crafting_filter: bool,
                    smelting_book: bool,
                    smelting_filter: bool,
                    blast_furnace_book: bool,
                    blast_furnace_filter: bool,
                    smoker_book: bool,
                    smoker_filter: bool,
                    recipe_ids: Vec<String>,
                }

                #[derive(serde::Deserialize, serde::Serialize, Debug)]
                enum Action {
                    Init(Common, Vec<String>),
                    Add(Common),
                    Remove(Common),
                }

                #[derive(serde::Deserialize, serde::Serialize, Debug)]
                struct UnlockRecipes {
                    action: Action,
                }

                let packet: UnlockRecipes = read_packet(buffer)?;
                show_packet_dbg(packet);
            }
            (0x3D, true) => {
                #[derive(serde::Deserialize, serde::Serialize, Debug)]
                enum WorldBorder {
                    Diameter(f64),
                    LerpSize {
                        old_diameter: f64,
                        new_diameter: f64,
                        // speed: VarLong,
                    },
                    Center {
                        x: f64,
                        z: f64,
                    },
                    Initialize {
                        x: f64,
                        z: f64,
                        old_diameter: f64,
                        new_diameter: f64,
                        // speed: VarLong,
                        // etc
                    },
                    WarningTime(VarInt),
                    WarningBlocks(VarInt),
                }

                let packet: WorldBorder = read_packet(buffer)?;
                show_packet_dbg(packet);
            }
            (0x3F, true) => {
                #[derive(serde::Deserialize, serde::Serialize, Debug)]
                struct HeldItemChange {
                    slot: u8, // which slot player selected, 0-8
                }

                let packet: HeldItemChange = read_packet(buffer)?;
                show_packet_dbg(packet);
            }
            (0x40, true) => {
                #[derive(serde::Deserialize, serde::Serialize, Debug)]
                struct UpdateViewPosition {
                    chunk_x: VarInt,
                    chunk_z: VarInt,
                }

                let packet: UpdateViewPosition = read_packet(buffer)?;
                show_packet_dbg(packet);
            }
            (0x42, true) => {
                #[derive(serde::Deserialize, serde::Serialize, Debug)]
                struct SpawnPosition(Position);

                let packet: SpawnPosition = read_packet(buffer)?;
                show_packet_dbg(packet);
            }
            (0x44, true) => {
                #[derive(serde::Deserialize, serde::Serialize, Debug)]
                struct EntityMetadata {
                    entity_id: VarInt,
                    // impossible to parse
                }

                let packet: EntityMetadata = read_packet(buffer)?;
                blocked_on("0xff terminator vec");
                show_packet_dbg(packet);
            }
            (0x48, true) => {
                #[derive(serde::Deserialize, serde::Serialize, Debug)]
                struct SetXP {
                    xp_bar: f32, // 0-1
                    level: VarInt,
                    total_xp: VarInt,
                }

                let packet: SetXP = read_packet(buffer)?;
                show_packet_dbg(packet);
            }
            (0x49, true) => {
                #[derive(serde::Deserialize, serde::Serialize, Debug)]
                struct UpdateHealth {
                    health: f32,
                    food: VarInt,
                    saturation: f32,
                }

                let packet: UpdateHealth = read_packet(buffer)?;
                show_packet_dbg(packet);
            }
            (0x4E, true) => {
                #[derive(serde::Deserialize, serde::Serialize, Debug)]
                struct TimeUpdate {
                    world_age: i64,
                    time_of_day: i64,
                }

                let packet: TimeUpdate = read_packet(buffer)?;
                show_packet_dbg(packet);
            }
            (0x57, true) => {
                #[derive(serde::Deserialize, serde::Serialize, Debug)]
                enum Frame {
                    Task,
                    Challenge,
                    Goal,
                }

                #[allow(non_camel_case_types)]
                #[derive(serde::Deserialize, serde::Serialize, Debug)]
                enum Flags {
                    None,
                    Background { texture: String },
                    Toast,
                }

                #[derive(serde::Deserialize, serde::Serialize, Debug)]
                struct Display {
                    title: String,
                    desc: String,
                    icon: Option<()>, // nbt
                    frame: Frame,
                    flags: Flags,
                    coords: (f32, f32),
                }

                #[derive(serde::Deserialize, serde::Serialize, Debug)]
                struct Advancement {
                    name: String,
                    parent: Option<String>,
                    display: Option<Display>,
                    criteria: Vec<String>,
                    requirements: Vec<Vec<String>>,
                }

                #[derive(serde::Deserialize, serde::Serialize, Debug)]
                struct Progress {
                    name: String,
                    criteria: Vec<(String, Option<i64>)>, // name, achieved?, date of achieving
                }

                #[derive(serde::Deserialize, serde::Serialize, Debug)]
                struct Advancements {
                    reset_clear: bool,
                    advancements: Vec<Advancement>,
                    removed: Vec<String>,
                    progress: Vec<Progress>,
                }

                let packet: Advancements = read_packet(buffer)?;
                blocked_on_nbt();
                show_packet_dbg(packet);
            }
            (0x58, true) => {
                #[derive(serde::Deserialize, serde::Serialize, Debug)]
                enum Operation {
                    AbsoluteAdd, // value += amount
                    PercentAdd,  // value += amount * value
                    Multiply,    // value *= amount
                }

                #[derive(serde::Deserialize, serde::Serialize, Debug)]
                struct Modifier {
                    #[serde(with = "Uuid")]
                    uuid: uuid::Uuid,
                    amount: f64,
                    operation: Operation,
                }

                #[derive(serde::Deserialize, serde::Serialize, Debug)]
                struct Property {
                    key: String,
                    value: f64,
                    modifiers: Vec<Modifier>,
                }

                #[derive(serde::Deserialize, serde::Serialize, Debug)]
                struct EntityProperties {
                    entity_id: VarInt,
                    #[serde(with = "customvec::int")]
                    properties: Vec<Property>,
                }

                let packet: EntityProperties = read_packet(buffer)?;
                show_packet_dbg(packet);
            }
            (0x5A, true) => {
                #[derive(serde::Deserialize, serde::Serialize, Debug)]
                struct Recipe;

                let packet: Vec<Recipe> = read_packet(buffer)?;
                blocked_on("string enum");
                println!("number of recipes read: {}", packet.len());
                // show_packet_dbg_min(packet);
            }
            (0x5B, true) => {
                #[derive(serde::Deserialize, serde::Serialize, Debug)]
                struct Tag {
                    name: String,
                    entries: Vec<VarInt>,
                }

                #[derive(serde::Deserialize, serde::Serialize, Debug)]
                struct Tags {
                    blocks: Vec<Tag>,
                    items: Vec<Tag>,
                    fluids: Vec<Tag>,
                    entities: Vec<Tag>,
                }

                let packet: Tags = read_packet(buffer)?;
                println!("packet fully parsed; display suppressed due to large size");
                println!(
                    "Tags {{ blocks: {}, items: {}, fluids: {}, entities: {} }}",
                    packet.blocks.len(),
                    packet.items.len(),
                    packet.fluids.len(),
                    packet.entities.len()
                );
                // show_packet_dbg_min(packet);
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

static mut PACKET_TYPE_COUNTER: Option<HashMap<(i32, bool), usize>> = None;

fn do_one_data(data: &Data) -> Result<()> {
    let mut reader = BufReader::new(&*data.data);

    while is_reader_not_eof(&mut reader)? {
        let packet_id = do_one_packet(&mut reader, data.is_server)?;
        println!();

        unsafe {
            if let None = PACKET_TYPE_COUNTER {
                PACKET_TYPE_COUNTER = Some(HashMap::new());
            }
            let map = PACKET_TYPE_COUNTER.as_mut().unwrap();
            let key = (packet_id, data.is_server);
            if map.contains_key(&key) {
                *map.get_mut(&key).unwrap() += 1;
            } else {
                map.insert(key, 1);
            }
        }
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
        // println!(
        //     "read physical packet: {} {:?}...",
        //     is_server,
        //     value.chars().take(50).collect::<String>()
        // );
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
    println!();
    unsafe {
        for (key, value) in PACKET_TYPE_COUNTER.as_mut().unwrap().iter() {
            println!("({:02X}, {}): {}", key.0, key.1, value);
        }
    }
}
