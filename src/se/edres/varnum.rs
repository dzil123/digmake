// When our Deserializer sees these magic strings, it will read a varint
// This is a workaround for adding custom types to Serde
// https://github.com/alexcrichton/toml-rs/blob/e454f3960afd5e7f5918f9b9ebd63fea507e95a1/src/datetime.rs#L37-L44
pub const VARINT_FIELD: &str = "$__digmake_private_varint_field";
pub const VARINT_NAME: &str = "$__digmake_private_varint_name";

pub const VARLONG_FIELD: &str = "$__digmake_private_varlong_field";
pub const VARLONG_NAME: &str = "$__digmake_private_varlong_name";

macro_rules! impl_deserialize {
    ($mdl:ident, ($typ:ty, $typ_ident:ident, $typ_expr:expr), $ityp:ty, ($FIELD:ident, $NAME:ident), $visit_func:ident, $visit_export:ident) => {
        mod $mdl {
            use super::{$FIELD, $NAME};
            use crate::se::error::Error;
            use crate::se::$typ_ident;
            use serde::de;
            use std::fmt;

            impl<'de> de::Deserialize<'de> for $typ {
                fn deserialize<D>(deserializer: D) -> Result<$typ, D::Error>
                where
                    D: de::Deserializer<'de>,
                {
                    static FIELDS: [&str; 1] = [$FIELD];
                    deserializer.deserialize_struct($NAME, &FIELDS, VarVisitor)
                }
            }

            struct VarVisitor;

            impl<'de> de::Visitor<'de> for VarVisitor {
                type Value = $typ;

                fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                    formatter.write_str(concat!("a ", stringify!($typ)))
                }

                fn visit_map<V>(self, mut visitor: V) -> Result<$typ, V::Error>
                where
                    V: de::MapAccess<'de>,
                {
                    let value = visitor.next_key::<VarKey>()?;
                    if value.is_none() {
                        return Err(de::Error::custom(concat!(stringify!($typ), " key not found")));
                    }
                    let v: $typ = visitor.next_value()?;
                    Ok(v)
                }

                fn $visit_func<E>(self, v: $ityp) -> Result<Self::Value, E>
                where
                    E: serde::de::Error,
                {
                    Ok( $typ_expr (v))
                }
            }

            struct VarKey;

            impl<'de> de::Deserialize<'de> for VarKey {
                fn deserialize<D>(deserializer: D) -> Result<VarKey, D::Error>
                where
                    D: de::Deserializer<'de>,
                {
                    struct FieldVisitor;

                    impl<'de> de::Visitor<'de> for FieldVisitor {
                        type Value = ();

                        fn expecting(
                            &self,
                            formatter: &mut fmt::Formatter<'_>,
                        ) -> fmt::Result {
                            formatter.write_str(concat!("a valid ", stringify!($ityp $typ)))
                        }

                        fn visit_str<E>(self, s: &str) -> Result<(), E>
                        where
                            E: de::Error,
                        {
                            if s == $FIELD {
                                Ok(())
                            } else {
                                Err(de::Error::custom("expected field with custom name"))
                            }
                        }
                    }
                    deserializer.deserialize_identifier(FieldVisitor)?;
                    Ok(VarKey)
                }
            }

            pub struct VarDeserializer {
                visited: bool,
                value: $ityp,
            }

            impl VarDeserializer {
                pub fn new(value: $ityp) -> Self {
                    Self {
                        visited: false,
                        value,
                    }
                }
            }

            impl<'de> de::MapAccess<'de> for VarDeserializer {
                type Error = Error;

                fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Error>
                where
                    K: de::DeserializeSeed<'de>,
                {
                    if self.visited {
                        return Ok(None);
                    }
                    self.visited = true;
                    seed.deserialize(VarFieldDeserializer).map(Some)
                }

                fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Error>
                where
                    V: de::DeserializeSeed<'de>,
                {
                    seed.deserialize(VarContentsDeserializer(self.value))
                }
            }

            struct VarFieldDeserializer;

            impl<'de> de::Deserializer<'de> for VarFieldDeserializer {
                type Error = Error;

                fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Error>
                where
                    V: de::Visitor<'de>,
                {
                    visitor.visit_borrowed_str($FIELD)
                }

                serde::forward_to_deserialize_any! {
                    bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string seq
                    bytes byte_buf map struct option unit newtype_struct
                    ignored_any unit_struct tuple_struct tuple enum identifier
                }
            }

            struct VarContentsDeserializer($ityp);

            impl<'de> de::Deserializer<'de> for VarContentsDeserializer {
                type Error = Error;

                fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Error>
                where
                    V: de::Visitor<'de>,
                {
                    visitor. $visit_func (self.0)
                }

                serde::forward_to_deserialize_any! {
                    bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string seq
                    bytes byte_buf map struct option unit newtype_struct
                    ignored_any unit_struct tuple_struct tuple enum identifier
                }
            }
        }

        pub(super) use $mdl::VarDeserializer as $visit_export;
    };
}

impl_deserialize!(
    varint,
    (VarInt, VarInt, VarInt),
    i32,
    (VARINT_FIELD, VARINT_NAME),
    visit_i32,
    VarIntDeserializer
);

impl_deserialize!(
    varlong,
    (VarLong, VarLong, VarLong),
    i64,
    (VARLONG_FIELD, VARLONG_NAME),
    visit_i64,
    VarLongDeserializer
);
