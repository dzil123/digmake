macro_rules! var_num {
    ($name:ident, $type:ty) => {
        #[derive(Copy, Clone, PartialOrd, PartialEq, Ord, Eq)]
        pub struct $name(pub $type);

        impl std::fmt::Debug for $name {
            #[inline(always)]
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                std::fmt::Debug::fmt(&self.0, f)
            }
        }

        impl std::fmt::Display for $name {
            #[inline(always)]
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                std::fmt::Display::fmt(&self.0, f)
            }
        }
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
