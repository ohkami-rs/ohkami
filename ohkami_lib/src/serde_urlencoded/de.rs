use crate::{percent_decode_utf8, percent_decode};
use std::borrow::Cow;


pub(crate) struct URLEncodedDeserializer<'de> {
    pub(crate) input: &'de str
}

impl<'de> URLEncodedDeserializer<'de> {
    #[inline(always)]
    fn peek(&self) -> Result<char, super::Error> {
        self.input.chars().next().ok_or_else(|| serde::de::Error::custom("Unexpected end of input"))
    }
    #[inline(always)]
    fn next(&mut self) -> Result<char, super::Error> {
        let next = self.peek()?;
        self.input = &self.input[next.len_utf8()..];
        Ok(next)
    }
    #[inline(always)]
    fn next_section(&mut self) -> Result<&'de str, super::Error> {
        if self.input.is_empty() {
            return Err((|| serde::de::Error::custom("Unexpected end of input"))())
        }
        let section = &self.input[..self.input.find(&['=', '&']).unwrap_or(self.input.len())];
        self.input = &self.input[section.len()..];
        Ok(section)
    }
}

impl<'u, 'de> serde::Deserializer<'de> for &'u mut URLEncodedDeserializer<'de> {
    type Error = super::Error;

    #[inline(always)]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: serde::de::Visitor<'de> {
        self.deserialize_map(visitor)
    }
    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: serde::de::Visitor<'de> {
        self.deserialize_any(visitor)
    }

    #[inline(always)]
    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where V: serde::de::Visitor<'de> {
        self.deserialize_map(visitor)
    }
    #[inline(always)]
    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: serde::de::Visitor<'de> {
        visitor.visit_map(AmpersandSeparated::new(self))
    }

    #[inline]
    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: serde::de::Visitor<'de> {
        let section = self.next_section()?;
        match percent_decode_utf8(section.as_bytes()).map_err(|e|
            serde::de::Error::custom(format!("Expected to be decoded to an UTF-8, but got `{section}`: {e}"))
        )? {
            Cow::Borrowed(str) => visitor.visit_borrowed_str(str),
            Cow::Owned(string) => visitor.visit_string(string),
        }
    }
    #[inline(always)]
    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: serde::de::Visitor<'de> {
        self.deserialize_str(visitor)
    }
    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: serde::de::Visitor<'de> {
        let section = self.next_section()?;
        let mut chars = section.chars();
        let (Some(ch), None) = (chars.next(), chars.next()) else {
            return Err((|| serde::de::Error::custom(
                format!("Expected a single charactor, but got `{section}`")
            ))())
        };
        visitor.visit_char(ch)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: serde::de::Visitor<'de> {
        match percent_decode(self.next_section()?.as_bytes()) {
            Cow::Borrowed(slice) => visitor.visit_bytes(slice),
            Cow::Owned(byte_vec) => visitor.visit_byte_buf(byte_vec),
        }
    }
    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: serde::de::Visitor<'de> {
        self.deserialize_bytes(visitor)
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: serde::de::Visitor<'de> {
        match self.next_section()? {
            "true"  => visitor.visit_bool(true),
            "false" => visitor.visit_bool(false),
            other   => Err(serde::de::Error::custom(format!(
                "Expected `true` or `false`, but got `{other}`"
            )))
        }
    }
}

struct AmpersandSeparated<'amp, 'de:'amp> {
    de:    &'amp mut URLEncodedDeserializer<'de>,
    first: bool,
}
impl<'amp, 'de> AmpersandSeparated<'amp, 'de> {
    #[inline(always)]
    fn new(de: &'amp mut URLEncodedDeserializer<'de>) -> Self {
        Self { de, first: true }
    }
}
const _: () = {
    impl<'amp, 'de> serde::de::MapAccess<'de> for AmpersandSeparated<'amp, 'de> {
        type Error = super::Error;

        #[inline]
        fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
        where K: serde::de::DeserializeSeed<'de> {
            if self.de.input.is_empty() {
                return Ok(None)
            }
            if !self.first && self.de.next()? != '&' {
                return Err((|| serde::de::Error::custom("missing &"))())
            }
            self.first = false;
            seed.deserialize(&mut *self.de).map(Some)
        }
        #[inline]
        fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
        where V: serde::de::DeserializeSeed<'de> {
            if self.de.next()? != '=' {
                return Err((|| serde::de::Error::custom("missing ="))())
            }
            seed.deserialize(&mut *self.de)
        }
    }
};
