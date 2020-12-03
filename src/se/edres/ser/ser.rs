use super::error::SerError;
use crate::se::{
    error::{Error, Result},
    Input, VarInt, VarLong,
};
use serde::{ser, Serialize, Serializer as _};
use std::convert::{TryFrom, TryInto};

impl Serialize for VarInt {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        todo!()
    }
}

pub struct Serializer<'a> {
    output: &'a mut Vec<u8>,
}

impl<'a> Serializer<'a> {
    fn serialize_variant_as_u8(&mut self, variant: u32) -> Result<()> {
        // the spec has most enums have a VarInt variant, but they're all 0 <= x < 128
        // so just serialize a u8 and call it a day
        if variant >= 128 {
            Err(SerError::LargeVariant(variant))?;
        }

        let variant: u8 = variant.try_into().unwrap(); // should not panic due to above check

        self.serialize_u8(variant)
    }

    //     fn clone<F>(&mut self, func: F) -> Result
    //     where
    //         F: FnMut(Self) -> Result,
    //     {
    //         func(Self {
    //             output: self.output,
    //         })
    //     }

    //     fn foo(mut self) -> Result<()>{
    //         self.clone().serialize_bool(false)?;
    //         self.serialize_bool(false)?;
    //         Ok(())
    //     }

    //     fn serialize_bool(self, v: bool) -> Result<()>{
    //         self.output.push(if v { 1 } else { 0 });
    //         Ok(())
    //     }
}

macro_rules! impl_serialize_num {
    ($fn_name:ident, $typ:ty) => {
        fn $fn_name(self, v: $typ) -> Result<()> {
            self.output.extend_from_slice(&v.to_be_bytes());
            Ok(())
        }
    };
}

impl<'a, 'b> ser::Serializer for &'b mut Serializer<'a> {
    type Ok = ();
    type Error = Error;

    type SerializeSeq = VecSerializer<'a, 'b>;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<()> {
        self.output.push(if v { 1 } else { 0 });
        Ok(())
    }

    impl_serialize_num!(serialize_i8, i8);
    impl_serialize_num!(serialize_i16, i16);
    impl_serialize_num!(serialize_i32, i32);
    impl_serialize_num!(serialize_i64, i64);
    impl_serialize_num!(serialize_i128, i128);

    impl_serialize_num!(serialize_u8, u8);
    impl_serialize_num!(serialize_u16, u16);
    impl_serialize_num!(serialize_u32, u32);
    impl_serialize_num!(serialize_u64, u64);
    impl_serialize_num!(serialize_u128, u128);

    impl_serialize_num!(serialize_f32, f32);
    impl_serialize_num!(serialize_f64, f64);

    fn serialize_char(self, v: char) -> Result<()> {
        Err(SerError::InvalidType("char"))?
    }

    fn serialize_str(self, v: &str) -> Result<()> {
        todo!()
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        self.output.extend_from_slice(v);
        Ok(())
    }

    fn serialize_none(self) -> Result<()> {
        self.serialize_bool(false)
    }

    fn serialize_some<T>(self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.serialize_bool(true)?;
        value.serialize(self)?;
        Ok(())
    }

    fn serialize_unit(self) -> Result<()> {
        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        Ok(())
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
    ) -> Result<()> {
        self.serialize_variant_as_u8(variant_index)
    }

    fn serialize_newtype_struct<T: ?Sized>(self, _name: &'static str, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        value: &T,
    ) -> Result<()>
    where
        T: Serialize,
    {
        self.serialize_variant_as_u8(variant_index)?;
        value.serialize(self)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        todo!()
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        todo!()
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        todo!()
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        todo!()
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap> {
        todo!()
    }

    fn serialize_struct(self, name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        todo!()
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        todo!()
    }

    #[inline]
    fn is_human_readable(&self) -> bool {
        false
    }
}

impl<'a, 'b> ser::SerializeTuple for &'b mut Serializer<'a> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        // value.serialize(&mut **self)
        value.serialize(*self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, 'b> ser::SerializeTupleStruct for &'b mut Serializer<'a> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        // value.serialize(&mut **self)
        value.serialize(*self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, 'b> ser::SerializeTupleVariant for &'b mut Serializer<'a> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, 'b> ser::SerializeMap for &'b mut Serializer<'a> {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        panic!()
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        panic!()
    }

    fn end(self) -> Result<()> {
        panic!()
    }
}

impl<'a, 'b> ser::SerializeStruct for &'b mut Serializer<'a> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        panic!()
    }

    fn end(self) -> Result<()> {
        panic!()
    }
}

impl<'a, 'b> ser::SerializeStructVariant for &'b mut Serializer<'a> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        panic!()
    }

    fn end(self) -> Result<()> {
        panic!()
    }
}

struct VecSerializer<'a, 'b> {
    parent: &'b mut Serializer<'a>,
    len: i32,
    output: Vec<u8>,
    serializer: Serializer<'a>,
}

impl<'a, 'b> VecSerializer<'a, 'b> {
    fn new(parent: &'b mut Serializer<'a>, len: Option<usize>) -> Self {
        let mut output = match len {
            Some(len) => Vec::with_capacity(len),
            None => Vec::new(),
        };

        Self {
            parent,
            output,
            len: 0,
            serializer: Serializer {
                // output: &mut output,
                // output: &mut parent.output,
                output: todo!(),
            },
        }
    }
}

impl<'a, 'b> ser::SerializeSeq for VecSerializer<'a, 'b> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + serde::Serialize,
    {
        self.len += 1;
        value.serialize(&mut self.serializer)
    }

    fn end(mut self) -> Result<()> {
        VarInt(self.len).serialize(&mut *self.parent)?;
        self.parent.output.append(&mut self.output);

        Ok(())
    }
}
