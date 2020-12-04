use crate::se::Position;
use serde::ser;

impl ser::Serialize for Position {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        todo!()
    }
}
