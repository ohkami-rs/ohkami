pub(crate) struct UTF8Deserializer<'de> {
    pub(crate) input: &'de str
}

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

impl<'de> serde::Deserializer<'de> for &mut UTF8Deserializer<'de> {
    type Error = super::Error;
    
    #[inline(always)]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: serde::de::Visitor<'de> {
        self.deserialize_str(visitor)
    }
    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: serde::de::Visitor<'de> {
        self.deserialize_any(visitor)
    }

    #[inline(always)]
    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: serde::de::Visitor<'de> {
        visitor.visit_str(self.input())
    }
    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: serde::de::Visitor<'de> {
        self.deserialize_str(visitor)
    }

    #[inline(always)]
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

    #[inline]
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

    #[inline]
    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: serde::de::Visitor<'de> {
        self.deserialize_str(visitor)
    }

    #[inline]
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
