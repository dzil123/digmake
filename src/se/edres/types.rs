use crate::se::Position;
use serde::de;

impl<'de> de::Deserialize<'de> for Position {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let data = u64::deserialize(deserializer)?;
        dbg!(data);

        let mut x = dbg!(dbg!(data >> 38) as i32);
        let mut y = (data & 0xFFF) as i16;
        let mut z = (data << 26 >> 38) as i32;

        if x >= 2i32.pow(25) {
            x -= 2i32.pow(26);
        }
        if y >= 2i16.pow(11) {
            y -= 2i16.pow(12)
        }
        if z >= 2i32.pow(25) {
            z -= 2i32.pow(26)
        }

        dbg!(x);

        Ok(Position { x, y, z })
    }
}
