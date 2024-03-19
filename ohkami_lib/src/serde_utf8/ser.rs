pub(crate) struct UTF8Serializer {
    pub(crate) output: String
}

const _: () = {
    /// Here the tuple variant is garanteed to have 0 or 1 field
    /// (by `UTFSerializer::serialize_tuple_variant` impl)
    impl serde::ser::SerializeTupleVariant for &mut UTF8Serializer {
        type Ok    = ();
        type Error = super::Error;

        fn serialize_field<T: ?Sized>(&mut self, field: &T) -> Result<(), Self::Error>
        where T: serde::Serialize {
            field.serialize(&mut **self)
        }

        fn end(self) -> Result<Self::Ok, Self::Error> {
            Ok(())
        }
    }
};

impl serde::Serializer for &mut UTF8Serializer {
    type Ok    = ();
    type Error = super::Error;

    type SerializeMap           = super::Infallible;
    type SerializeSeq           = super::Infallible;
    type SerializeStruct        = super::Infallible;
    type SerializeTupleStruct   = super::Infallible;
    type SerializeTuple         = super::Infallible;
    type SerializeStructVariant = super::Infallible;
    type SerializeTupleVariant  = Self;

    #[inline(always)]
    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.output.push_str(v);
        Ok(())
    }
    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.output.push(v);
        Ok(())
    }
    fn serialize_bytes(self, _: &[u8]) -> Result<Self::Ok, Self::Error> {
        Err(serde::ser::Error::custom("ohkami's builtin UTF-8 serializer doesn't support raw byte data !"))
    }
    
    #[inline]
    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
    #[inline]
    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where T: serde::Serialize {
        value.serialize(self)
    }

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        self.output.push_str(if v {"true"} else {"false"});
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(v.into())
    }
    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(v.into())
    }
    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(v.into())
    }
    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        self.output.push_str(&v.to_string());
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.serialize_u64(v.into())
    }
    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.serialize_u64(v.into())
    }
    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.serialize_u64(v.into())
    }
    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.output.push_str(&v.to_string());
        Ok(())
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.serialize_f64(v.into())
    }
    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        self.output.push_str(&v.to_string());
        Ok(())
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
    fn serialize_unit_struct(self, _: &'static str) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    #[inline]
    fn serialize_unit_variant(
        self,
        _name: &'static str, 
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(variant)
    }
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        if len > 1 {
            Err(serde::ser::Error::custom("ohkami's builtin UTF-8 serialier doesn't support enum with tuple variant having more than 2 fields !"))
        } else {
            Ok(self)
        }
    }
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(serde::ser::Error::custom("ohkami's builtin UTF-8 serialier doesn't support enum with struct variants !"))
    }

    #[inline(always)]
    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where T: serde::Serialize {
        value.serialize(self)
    }
    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where T: serde::Serialize {
        value.serialize(self)
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Err(serde::ser::Error::custom("ohkami's builtin UTF-8 serialier doesn't support struct (except for newtype pattern) !"))
    }
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(serde::ser::Error::custom("ohkami's builtin UTF-8 serialier doesn't support map !"))
    }
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(serde::ser::Error::custom("ohkami's builtin UTF-8 serialier doesn't support sequence !"))
    }
    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(serde::ser::Error::custom("ohkami's builtin UTF-8 serialier doesn't support tuple !"))
    }
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(serde::ser::Error::custom("ohkami's builtin UTF-8 serialier doesn't support struct (except for newtype pattern) !"))
    }
}
