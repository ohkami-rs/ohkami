pub fn to_string(value: &impl serde::Serialize) -> Result<String, Error> {
    let mut s = UTF8Serializer { output: String::new() };
    value.serialize(&mut s)?;
    Ok(s.output)
}

pub fn from_str<'de, D: serde::Deserialize<'de>>(input: &'de str) -> Result<D, Error> {
    let mut d = UTF8Deserializer { input };
    let t = D::deserialize(&mut d)?;
    if d.input.is_empty() {
        Ok(t)
    } else {
        Err((||serde::de::Error::custom(format!("Unexpected trailing charactors: {}", d.input)))())
    }
}


struct UTF8Serializer { output: String }
const _: () = {
    /// Here the tuple variant is garanteed to have 0 or 1 field
    /// (by `UTFSerializer::serialize_tuple_variant` impl)
    impl serde::ser::SerializeTupleVariant for &mut UTF8Serializer {
        type Ok    = ();
        type Error = Error;

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
    type Error = Error;

    type SerializeMap           = Infallible;
    type SerializeSeq           = Infallible;
    type SerializeStruct        = Infallible;
    type SerializeTupleStruct   = Infallible;
    type SerializeTuple         = Infallible;
    type SerializeStructVariant = Infallible;
    type SerializeTupleVariant  = Self;

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

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.output.push(v);
        Ok(())
    }
    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.output.push_str(v);
        Ok(())
    }
    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        let v = std::str::from_utf8(v).map_err(|e| serde::ser::Error::custom(e))?;
        self.serialize_str(v)
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where T: serde::Serialize {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
    fn serialize_unit_struct(self, _: &'static str) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

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


struct UTF8Deserializer<'de> { input: &'de str }
const _: () = {
    impl<'de> UTF8Deserializer<'de> {
        #[inline(always)]
        fn input(&mut self) -> &str {
            let mut input = "";
            std::mem::swap(&mut self.input, &mut input);
            input
        }
    }
};
impl<'de, 'u> serde::Deserializer<'de> for &'u mut UTF8Deserializer<'de> {
    type Error = Error;
    
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: serde::de::Visitor<'de> {
        self.deserialize_str(visitor)
    }
    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: serde::de::Visitor<'de> {
        self.deserialize_any(visitor)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: serde::de::Visitor<'de> {
        visitor.visit_str(self.input())
    }
    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: serde::de::Visitor<'de> {
        self.deserialize_str(visitor)
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where  V: serde::de::Visitor<'de> {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: serde::de::Visitor<'de> {
        match self.input() {
            "true" => visitor.visit_bool(true),
            "false" => visitor.visit_bool(false),
            other => Err(
                (|| serde::de::Error::custom(
                    format!("Expected `true` or `false`, but found `{other}`"))
                )()
            )
        }
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: serde::de::Visitor<'de> {
        self.deserialize_bytes(visitor)
    }
    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: serde::de::Visitor<'de> {
        visitor.visit_bytes(self.input().as_bytes())
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: serde::de::Visitor<'de> {
        let mut chars = self.input.chars();
        match (chars.next(), chars.next()) {
            (Some(c), None) => {self.input = ""; visitor.visit_char(c)},
            _ => Err((|| serde::de::Error::custom(
                format!("Expected a single char, but got single")
            ))())
        }
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where V: serde::de::Visitor<'de> {
        use serde::de::IntoDeserializer;

        visitor.visit_enum(self.input().into_deserializer())
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: serde::de::Visitor<'de> {
        visitor.visit_f32(self.input().parse()
            .map_err(|e| serde::de::Error::custom(
                format!("Can't parse input as a f32: {e}")
            ))?
        )
    }
    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: serde::de::Visitor<'de> {
        visitor.visit_f64(self.input().parse()
            .map_err(|e| serde::de::Error::custom(
                format!("Can't parse input as a f64: {e}")
            ))?
        )
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: serde::de::Visitor<'de> {
        visitor.visit_i8(self.input().parse()
            .map_err(|e| serde::de::Error::custom(
                format!("Can't parse input as a i8: {e}")
            ))?
        )
    }
    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: serde::de::Visitor<'de> {
        visitor.visit_i16(self.input().parse()
            .map_err(|e| serde::de::Error::custom(
                format!("Can't parse input as a i16: {e}")
            ))?
        )
    }
    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: serde::de::Visitor<'de> {
        visitor.visit_i32(self.input().parse()
            .map_err(|e| serde::de::Error::custom(
                format!("Can't parse input as a i32: {e}")
            ))?
        )
    }
    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: serde::de::Visitor<'de> {
        visitor.visit_i64(self.input().parse()
            .map_err(|e| serde::de::Error::custom(
                format!("Can't parse input as a i64: {e}")
            ))?
        )
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: serde::de::Visitor<'de> {
        visitor.visit_u8(self.input().parse()
            .map_err(|e| serde::de::Error::custom(
                format!("Can't parse input as a u8: {e}")
            ))?
        )
    }
    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: serde::de::Visitor<'de> {
        visitor.visit_u16(self.input().parse()
            .map_err(|e| serde::de::Error::custom(
                format!("Can't parse input as a u16: {e}")
            ))?
        )
    }
    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: serde::de::Visitor<'de> {
        visitor.visit_u32(self.input().parse()
            .map_err(|e| serde::de::Error::custom(
                format!("Can't parse input as a u32: {e}")
            ))?
        )
    }
    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: serde::de::Visitor<'de> {
        visitor.visit_u64(self.input().parse()
            .map_err(|e| serde::de::Error::custom(
                format!("Can't parse input as a u64: {e}")
            ))?
        )
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: serde::de::Visitor<'de> {
        self.deserialize_str(visitor)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: serde::de::Visitor<'de> {
        if self.input.is_empty() {
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    fn deserialize_map<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where V: serde::de::Visitor<'de> {
        Err(serde::de::Error::custom("ohkami's builtin UTF-8 deserializer doesn't support maps"))
    }
    fn deserialize_seq<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where V: serde::de::Visitor<'de> {
        Err(serde::de::Error::custom("ohkami's builtin UTF-8 deserializer doesn't support sequences"))
    }
    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where V: serde::de::Visitor<'de> {
        Err(serde::de::Error::custom("ohkami's builtin UTF-8 deserializer doesn't support structs except for newtypes !"))
    }
    fn deserialize_tuple<V>(self, _len: usize, _visitor: V) -> Result<V::Value, Self::Error>
    where V: serde::de::Visitor<'de> {
        Err(serde::de::Error::custom("ohkami's builtin UTF-8 deserializer doesn't support tuples"))
    }
    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where V: serde::de::Visitor<'de> {
        Err(serde::de::Error::custom("ohkami's builtin UTF-8 deserializer doesn't support structs except for newtypes !"))
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: serde::de::Visitor<'de> {
        if self.input.is_empty() {
            visitor.visit_unit()
        } else {
            Err(serde::de::Error::custom(
                format!("Expected empty input, but got {}", self.input)
            ))
        }
    }
    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where V: serde::de::Visitor<'de> {
        self.deserialize_unit(visitor)
    }
}




#[derive(Debug)]
pub struct Error(String);
const _: () = {
    impl std::fmt::Display for Error {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str(&self.0)
        }
    }
    impl std::error::Error for Error {}

    impl serde::ser::Error for Error {
        fn custom<T>(msg:T) -> Self where T:std::fmt::Display {
            Self(msg.to_string())
        }
    }
    impl serde::de::Error for Error {
        fn custom<T>(msg:T) -> Self where T:std::fmt::Display {
            Self(msg.to_string())
        }
    }
};



enum Infallible {}
const _: () = {
    impl serde::ser::SerializeMap for Infallible {
        type Ok    = ();
        type Error = Error;

        fn serialize_key<T: ?Sized>(&mut self, _: &T) -> Result<(), Self::Error>
        where T: serde::Serialize {match *self {}}

        fn serialize_value<T: ?Sized>(&mut self, _: &T) -> Result<(), Self::Error>
        where T: serde::Serialize {match *self {}}

        fn end(self) -> Result<Self::Ok, Self::Error> {match self {}}
    }

    impl serde::ser::SerializeSeq for Infallible {
        type Ok    = ();
        type Error = Error;

        fn serialize_element<T: ?Sized>(&mut self, _: &T) -> Result<(), Self::Error>
        where T: serde::Serialize {match *self {}}

        fn end(self) -> Result<Self::Ok, Self::Error> {match self {}}
    }

    impl serde::ser::SerializeStruct for Infallible {
        type Ok    = ();
        type Error = Error;

        fn serialize_field<T: ?Sized>(&mut self, _: &'static str, _: &T) -> Result<(), Self::Error>
        where T: serde::Serialize {match *self {}}

        fn end(self) -> Result<Self::Ok, Self::Error> {match self {}}
    }

    impl serde::ser::SerializeStructVariant for Infallible {
        type Ok    = ();
        type Error = Error;

        fn serialize_field<T: ?Sized>(&mut self, _: &'static str, _: &T) -> Result<(), Self::Error>
        where T: serde::Serialize {match *self {}}

        fn end(self) -> Result<Self::Ok, Self::Error> {match self {}}
    }

    impl serde::ser::SerializeTuple for Infallible {
        type Ok    = ();
        type Error = Error;

        fn serialize_element<T: ?Sized>(&mut self, _: &T) -> Result<(), Self::Error>
        where T: serde::Serialize {match *self {}}

        fn end(self) -> Result<Self::Ok, Self::Error> {match self {}}
    }

    impl serde::ser::SerializeTupleStruct for Infallible {
        type Ok    = ();
        type Error = Error;

        fn serialize_field<T: ?Sized>(&mut self, _: &T) -> Result<(), Self::Error>
        where T: serde::Serialize {match *self {}}

        fn end(self) -> Result<Self::Ok, Self::Error> {match self {}}
    }
};
