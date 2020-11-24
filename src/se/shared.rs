// pub struct VarInt(i32);
// pub struct VarLong(i64);

macro_rules! var_num {
    ($name:ident, $type:ty) => {
        #[derive(Copy, Clone, Debug, PartialOrd, PartialEq, Ord, Eq)]
        pub struct $name(pub $type);
    };
}

var_num!(VarInt, i32);
var_num!(VarLong, i64);
