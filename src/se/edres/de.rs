use super::{error::err, varint, Error, Result};
use crate::se::{
    mon::{Parse, ParseB},
    Input, VarInt, VarLong,
};
use nom::number::complete as nom_num;
use serde::{
    self,
    de::{
        DeserializeSeed, Deserializer as SDeserializer, EnumAccess, SeqAccess, VariantAccess,
        Visitor,
    },
};

pub(super) struct Deserializer<'de> {
    #[allow(dead_code)]
    pub(super) original_input: Input<'de>,
    pub(super) input: Input<'de>,
}

impl<'de> Deserializer<'de> {
    pub(super) fn new(input: Input<'de>) -> Self {
        Self {
            original_input: input,
            input,
        }
    }

    fn update<T>(&mut self, parsed: (Input<'de>, T)) -> T {
        self.input = parsed.0;
        parsed.1
    }
}

impl<'de, 'a> SDeserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        err("cannot deserialize any")
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_bool(self.update(bool::parse(self.input)?))
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i8(self.update(nom_num::be_i8(self.input)?))
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i16(self.update(nom_num::be_i16(self.input)?))
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i32(self.update(nom_num::be_i32(self.input)?))
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i64(self.update(nom_num::be_i64(self.input)?))
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u8(self.update(nom_num::be_u8(self.input)?))
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u16(self.update(nom_num::be_u16(self.input)?))
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u32(self.update(nom_num::be_u32(self.input)?))
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u64(self.update(nom_num::be_u64(self.input)?))
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_f32(self.update(nom_num::be_f32(self.input)?))
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_f64(self.update(nom_num::be_f64(self.input)?))
    }

    fn deserialize_char<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_borrowed_str(self.update(String::parse(self.input)?))
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_bytes<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_byte_buf<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let is_some = self.update(bool::parse(self.input)?);
        if is_some {
            visitor.visit_some(self)
        } else {
            visitor.visit_none()
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(self)
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        err("cannot deserialize map")
        // let x = visitor.visit_map(MapAccesser { de: &mut self })?;
        // Ok(x)
    }

    fn deserialize_struct<V>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if name == varint::VARINT_NAME && fields == [varint::VARINT_FIELD] {
            let value = self.update(VarInt::parse(self.input)?);
            return visitor.visit_map(varint::VarIntDeserializer::new(value));
        }

        // dbg!(name, fields);
        // self.deserialize_map(visitor)
        self.deserialize_seq(visitor)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // dbg!(
        //     "alfa",
        //     std::any::type_name::<V>(),
        //     std::any::type_name::<V::Value>()
        // );
        visitor.visit_enum(self)
        // visitor.visit_enum(*self)
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // called by enum visitor derive to determine descriminant
        //   (from std::mem::discriminant()) (afaict sequential u64)
        // implement by reading one byte (upcast to u64):
        // will work for enums with underlying type bool, u8, varint (0 <= x < 128)
        // otherwise, idk how to do this
        self.deserialize_u8(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }

    #[inline]
    fn is_human_readable(&self) -> bool {
        false
    }
}

impl<'de> SeqAccess<'de> for Deserializer<'de> {
    type Error = <&'de mut Self as SDeserializer<'de>>::Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        // dbg!(
        //     "bravo",
        //     std::any::type_name::<T>(),
        //     std::any::type_name::<T::Value>()
        // );

        if self.input.len() == 0 {
            return Ok(None);
        }

        seed.deserialize(self).map(Some)
    }
}

impl<'de, 'a> EnumAccess<'de> for &'a mut Deserializer<'de> {
    type Error = <Self as SDeserializer<'de>>::Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: DeserializeSeed<'de>,
    {
        // dbg!(
        //     "charlie",
        //     std::any::type_name::<V>(),
        //     std::any::type_name::<V::Value>()
        // );
        seed.deserialize(&mut *self).map(|val| (val, self))
    }
}

impl<'de, 'a> VariantAccess<'de> for &'a mut Deserializer<'de> {
    type Error = <Self as SDeserializer<'de>>::Error;

    /*
    enum Foo {
        One,
        Two(),
        Three(T),
        Four(U, V),
        Five { val: W },
    }
    */

    // Handles Foo::One
    fn unit_variant(self) -> Result<()> {
        // dbg!("delta");
        Ok(())
    }

    // Handles Foo::Three
    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: DeserializeSeed<'de>,
    {
        // dbg!(
        //     "echo",
        //     std::any::type_name::<T>(),
        //     std::any::type_name::<T::Value>()
        // );
        seed.deserialize(self)
    }

    // Handles Foo::{ Two, Four }
    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // dbg!(
        //     "foxtrot",
        //     std::any::type_name::<V>(),
        //     std::any::type_name::<V::Value>()
        // );
        self.deserialize_seq(visitor)
    }

    // Handles Foo::Five
    fn struct_variant<V>(self, _fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // dbg!(
        //     "golf",
        //     std::any::type_name::<V>(),
        //     std::any::type_name::<V::Value>()
        // );
        self.deserialize_seq(visitor)
    }
}
