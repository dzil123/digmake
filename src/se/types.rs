macro_rules! var_num {
    ($name:ident, $type:ty) => {
        #[derive(Copy, Clone, Debug, PartialOrd, PartialEq, Ord, Eq)]
        pub struct $name(pub $type);
    };
}

var_num!(VarInt, i32);
var_num!(VarLong, i64);

#[derive(Clone, Debug, PartialOrd, PartialEq, Ord, Eq)]
pub struct Position {
    pub x: i32, // 26 bit
    pub y: i16, // 12 bit
    pub z: i32, // 26 bit
}
