use crate::se::Position;
use serde::ser;

impl ser::Serialize for Position {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        let x = (self.x as i64) & 0x3FFFFFF;
        let y = (self.y as i64) & 0xFFF;
        let z = (self.z as i64) & 0x3FFFFFF;

        let data = (x << 38) | (z << 12) | y;
        serializer.serialize_i64(data)
    }
}
