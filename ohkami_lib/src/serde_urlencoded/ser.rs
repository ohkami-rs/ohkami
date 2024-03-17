use serde::Serialize as _;
use crate::percent_encode;


pub(crate) struct URLEncodedSerializer {
    output: String,

    /// To forbid nesting maps
    init: bool,
}
impl URLEncodedSerializer {
    #[inline]
    pub(crate) const fn new() -> Self {
        Self {
            output: String::new(),
            init:   true,
        }
    }

    #[inline]
    pub(crate) fn output(self) -> String {
        self.output
    }
}

const _: () = {
    impl serde::ser::SerializeMap for &mut URLEncodedSerializer {
        type Ok = ();
        type Error = super::Error;

        fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), Self::Error>
        where T: serde::Serialize {
            if !self.output.is_empty() {
                self.output.push('&');
            }
            key.serialize(&mut **self)
        }
        fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
        where T: serde::Serialize {
            self.output.push('=');
            value.serialize(&mut **self)
        }
        fn end(self) -> Result<Self::Ok, Self::Error> {
            Ok(())
        }
    }
    impl serde::ser::SerializeStruct for &mut URLEncodedSerializer {
        type Ok    = ();
        type Error = super::Error;

        #[inline(always)]
        fn serialize_field<T: ?Sized>(
            &mut self,
            key: &'static str,
            value: &T,
        ) -> Result<(), Self::Error>
        where T: serde::Serialize {
            if !self.output.is_empty() {
                self.output.push('&');
            }
            key.serialize(&mut **self)?;
            self.output.push('=');
            value.serialize(&mut **self)?;
            Ok(())
        }
        #[inline(always)]
        fn end(self) -> Result<Self::Ok, Self::Error> {
            Ok(())
        }
    }

    impl serde::ser::SerializeSeq for &mut URLEncodedSerializer {
        type Ok    = ();
        type Error = super::Error;

        fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
        where T: serde::Serialize {
            if !self.output.ends_with('=') {
                self.output.push(',');
            }
            value.serialize(&mut **self)
        }
        fn end(self) -> Result<Self::Ok, Self::Error> {
            Ok(())
        }
    }
    impl serde::ser::SerializeTuple for &mut URLEncodedSerializer {
        type Ok    = ();
        type Error = super::Error;

        fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
        where T: serde::Serialize {
            if !self.output.ends_with('=') {
                self.output.push(',');
            }
            value.serialize(&mut **self)
        }
        fn end(self) -> Result<Self::Ok, Self::Error> {
            Ok(())
        }
    }
    impl serde::ser::SerializeTupleStruct for &mut URLEncodedSerializer {
        type Ok    = ();
        type Error = super::Error;

        fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
        where T: serde::Serialize {
            if !self.output.ends_with('=') {
                self.output.push(',');
            }
            value.serialize(&mut **self)
        }
        fn end(self) -> Result<Self::Ok, Self::Error> {
            Ok(())
        }
    }
    impl serde::ser::SerializeTupleVariant for &mut URLEncodedSerializer {
        type Ok    = ();
        type Error = super::Error;

        fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
        where T: serde::Serialize {
            if !self.output.ends_with('=') {
                self.output.push(',');
            }
            value.serialize(&mut **self)
        }
        fn end(self) -> Result<Self::Ok, Self::Error> {
            Ok(())
        }
    }
};

impl serde::Serializer for &mut URLEncodedSerializer {
    type Ok    = ();
    type Error = super::Error;

    type SerializeMap           = Self;
    type SerializeStruct        = Self;

    type SerializeSeq           = Self;
    type SerializeTuple         = Self;
    type SerializeTupleStruct   = Self;

    type SerializeTupleVariant  = super::Infallible;
    type SerializeStructVariant = super::Infallible;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        self.output.push_str(if v {"true"} else {"false"});
        Ok(())
    }

    fn serialize_bytes(self, _: &[u8]) -> Result<Self::Ok, Self::Error> {
        Err(serde::ser::Error::custom("ohkami's builtin urlencoded serializer doesn't support raw byte data !"))
    }

    #[inline(always)]
    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.output.push_str(&percent_encode(v));
        Ok(())
    }
    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.output.push(v);
        Ok(())
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.output.push_str(&v.to_string());
        Ok(())
    }
    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        self.output.push_str(&v.to_string());
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.output.push_str(&v.to_string());
        Ok(())
    }
    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.output.push_str(&v.to_string());
        Ok(())
    }
    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.output.push_str(&v.to_string());
        Ok(())
    }
    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        self.output.push_str(&v.to_string());
        Ok(())        
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.output.push_str(&v.to_string());
        Ok(())
    }
    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.output.push_str(&v.to_string());
        Ok(())
    }
    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.output.push_str(&v.to_string());
        Ok(())
    }
    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.output.push_str(&v.to_string());
        Ok(())        
    }

    #[inline]
    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where T: serde::Serialize {
        value.serialize(self)
    }

    #[inline]
    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where T: serde::Serialize {
        value.serialize(self)
    }
    #[inline]
    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    #[inline(always)]
    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        let _ = len;
        self.init.then_some({self.init = false; self})
            .ok_or_else(||serde::ser::Error::custom("ohkami's builtin urlencoded serializer doesn't support nested maps or map-like structs !"))
    }
    #[inline(always)]
    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        self.serialize_map(Some(len))
    }
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.serialize_seq(Some(len))
    }
    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.serialize_unit()
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(self)
    }
    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.serialize_seq(Some(len))
    }

    #[inline]
    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where T: serde::Serialize {
        if !self.output.is_empty() {
            self.output.push('&');
        }
        variant.serialize(&mut *self)?;
        self.output.push('=');
        value.serialize(&mut *self)?;
        Ok(())
    }
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(serde::ser::Error::custom("ohkami's builtin urlencoded serializer doesn't support enum with struct variants !"))
    }
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(serde::ser::Error::custom("ohkami's builtin urlencoded serializer doesn't support enum with tuple variants !"))
    }
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        variant.serialize(self)
    }
}
