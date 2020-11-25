use super::error::Error;
use crate::se::{VarInt, VarLong};
use serde::de;

// When our Deserializer sees these magic strings, it will read a varint
// This is a workaround for adding custom types to Serde
// https://github.com/alexcrichton/toml-rs/blob/e454f3960afd5e7f5918f9b9ebd63fea507e95a1/src/datetime.rs#L37-L44
pub const VARINT_FIELD: &str = "$__digmake_private_varint_field";
pub const VARINT_NAME: &str = "$__digmake_private_varint_name";

impl<'de> de::Deserialize<'de> for VarInt {
    fn deserialize<D>(deserializer: D) -> Result<VarInt, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct VarIntVisitor;

        impl<'de> de::Visitor<'de> for VarIntVisitor {
            type Value = VarInt;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a VarInt")
            }

            fn visit_map<V>(self, mut visitor: V) -> Result<VarInt, V::Error>
            where
                V: de::MapAccess<'de>,
            {
                // dbg!("VarInt MapAccess", std::any::type_name::<V>());

                let value = visitor.next_key::<VarIntKey>()?;
                if value.is_none() {
                    return Err(de::Error::custom("VarInt key not found"));
                }
                let v: VarInt = visitor.next_value()?;
                Ok(v)
            }

            fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                // dbg!("VarInt Error", std::any::type_name::<E>());

                Ok(VarInt(v))
            }
        }
        // dbg!("VarInt Deserializer", std::any::type_name::<D>());

        static FIELDS: [&str; 1] = [VARINT_FIELD];
        deserializer.deserialize_struct(VARINT_NAME, &FIELDS, VarIntVisitor)
    }
}

struct VarIntKey;

impl<'de> de::Deserialize<'de> for VarIntKey {
    fn deserialize<D>(deserializer: D) -> Result<VarIntKey, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct FieldVisitor;

        impl<'de> de::Visitor<'de> for FieldVisitor {
            type Value = ();

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a valid i32 VarInt 1-5 bytes")
            }

            fn visit_str<E>(self, s: &str) -> Result<(), E>
            where
                E: de::Error,
            {
                // dbg!("VarIntKey Deserializer 2", std::any::type_name::<E>());
                if s == VARINT_FIELD {
                    Ok(())
                } else {
                    Err(de::Error::custom("expected field with custom name"))
                }
            }
        }
        // dbg!("VarIntKey Deserializer", std::any::type_name::<D>());

        deserializer.deserialize_identifier(FieldVisitor)?;
        Ok(VarIntKey)
    }
}

pub(super) struct VarIntDeserializer {
    visited: bool,
    value: i32,
}

impl VarIntDeserializer {
    pub(super) fn new(value: i32) -> Self {
        Self {
            visited: false,
            value,
        }
    }
}

impl<'de> de::MapAccess<'de> for VarIntDeserializer {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        // dbg!(
        //     "VarIntDeserializer DeserializeSeed",
        //     std::any::type_name::<K>()
        // );
        if self.visited {
            return Ok(None);
        }
        self.visited = true;
        seed.deserialize(VarIntFieldDeserializer).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        // dbg!(
        //     "VarIntDeserializer DeserializeSeed 2",
        //     std::any::type_name::<V>()
        // );
        seed.deserialize(VarIntContentsDeserializer(self.value))
    }
}

struct VarIntFieldDeserializer;

impl<'de> de::Deserializer<'de> for VarIntFieldDeserializer {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: de::Visitor<'de>,
    {
        // dbg!(
        //     "VarIntFieldDeserializer Visitor",
        //     std::any::type_name::<V>()
        // );
        visitor.visit_borrowed_str(VARINT_FIELD)
    }

    serde::forward_to_deserialize_any! {
        bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string seq
        bytes byte_buf map struct option unit newtype_struct
        ignored_any unit_struct tuple_struct tuple enum identifier
    }
}

struct VarIntContentsDeserializer(i32);

impl<'de> de::Deserializer<'de> for VarIntContentsDeserializer {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: de::Visitor<'de>,
    {
        // dbg!(
        //     "VarIntContentsDeserializer Visitor",
        //     std::any::type_name::<V>()
        // );
        visitor.visit_i32(self.0)
    }

    serde::forward_to_deserialize_any! {
        bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string seq
        bytes byte_buf map struct option unit newtype_struct
        ignored_any unit_struct tuple_struct tuple enum identifier
    }
}
