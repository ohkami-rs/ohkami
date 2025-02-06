use serde::de::IntoDeserializer as _;

use crate::{percent_decode_utf8, percent_decode};
use std::borrow::Cow;


pub(crate) struct URLEncodedDeserializer<'de> {
    input: &'de [u8],
    side:  ParsingSide,
}
#[derive(Debug, PartialEq, Clone, Copy)]
enum ParsingSide {
    Key,
    Value,
}

impl<'de> URLEncodedDeserializer<'de> {
    pub(crate) fn new(input: &'de [u8]) -> Self {
        Self { input, side: ParsingSide::Key }
    }
    pub(crate) fn remaining(&self) -> &'de [u8] {
        &self.input
    }
}

impl<'de> URLEncodedDeserializer<'de> {
    #[inline(always)]
    fn peek(&self) -> Result<&u8, super::Error> {
        self.input.first().ok_or_else(|| serde::de::Error::custom("can't peek: unexpected end of input"))
    }
    #[inline(always)]
    fn next(&mut self) -> Result<u8, super::Error> {
        let next = self.peek()?.clone();
        self.input = &self.input[1..];
        Ok(next)
    }
    #[inline(always)]
    unsafe fn take_n_unchecked(&mut self, n: usize) -> &'de [u8] {
        use std::slice::from_raw_parts;

        let len = self.input.len();
        let ptr = self.input.as_ptr();

        // SAFETY: Caller has to check that `0 <= mid <= self.len()`
        let (take, remaining) = (from_raw_parts(ptr, n), from_raw_parts(ptr.add(n), len - n));

        self.input = remaining;
        take
    }
    #[inline(always)]
    fn next_section(&mut self) -> Result<&'de [u8], super::Error> {
        let next_punc = self.input.iter().position(|b| matches!(b, b'=' | b'&'));

        match &self.side {
            ParsingSide::Key => match next_punc {
                /* e.g. `name` */
                None => Err(serde::de::Error::custom("invalid key-value: unexpected end of input")),

                /* e.g. `=ohkami` */
                Some(0) => Err(serde::de::Error::custom("invalid key-value: empty key")),

                /* e.g. `name=ohkami` is ok, `name&ohkami` is err */
                Some(n) => (self.input[n] == b'=')
                    .then_some(unsafe {self.take_n_unchecked(n)})
                    .ok_or_else(|| serde::de::Error::custom("invalid key-value: missing `=`"))
            },

            ParsingSide::Value => match next_punc {
                /* final value; end of whole the parsing */
                None => Ok(unsafe {self.take_n_unchecked(self.input.len())}),

                /* n = 0 is ok (e.g. `name=&age=18` is valid; `name` is empty string) */
                /* e.g. `name=ohkami&age=4` is ok, `name=ohkami=age=4` is err */
                Some(n) => (self.input[n] == b'&')
                    .then_some(unsafe {self.take_n_unchecked(n)})
                    .ok_or_else(|| serde::de::Error::custom("invalid key-value: missing `&`"))
            }
        }
    }
}

impl<'u, 'de> serde::Deserializer<'de> for &'u mut URLEncodedDeserializer<'de> {
    type Error = super::Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: serde::de::Visitor<'de> {
        self.deserialize_map(visitor)
    }

    /// when the visitor visits value of unkown key
    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: serde::de::Visitor<'de> {
        #[cfg(debug_assertions)] {
            assert!(matches!(self.side, ParsingSide::Value));
        }
        let _ = self.next_section();

        self.side = ParsingSide::Key;

        visitor.visit_unit()
    }

    #[inline(always)]
    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: serde::de::Visitor<'de> {
        #[cfg(debug_assertions)] {
            assert!(self.side == ParsingSide::Key);
        }

        visitor.visit_map(AmpersandSeparated::new(self))
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

    #[inline]
    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where V: serde::de::Visitor<'de> {
        visitor.visit_enum(Enum::new(self))
    }

    #[inline(always)]
    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: serde::de::Visitor<'de> {
        /*
            Here we don't put

            ```
            #[cfg(debug_assertions)] {
                assert!(self.side == ParsingSide::Key);
            }
            ```
            because `deserialize_identifier` can be called by value-place enums
            like `enum Gender { Male, Female, Other }`.
        */

        self.deserialize_str(visitor)
    }
    #[inline(always)]
    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: serde::de::Visitor<'de> {
        let section = self.next_section()?;
        match percent_decode_utf8(section).map_err(|e|
            serde::de::Error::custom(format!("Expected to be decoded to an UTF-8, but got `{}`: {e}", section.escape_ascii()))
        )? {
            Cow::Borrowed(str) => visitor.visit_borrowed_str(str),
            Cow::Owned(string) => visitor.visit_string(string)
                .map_err(|e: Self::Error| {
                    #[cfg(debug_assertions)] {
                        serde::de::Error::custom(format!(
                            "{e}. [DEBUG] maybe you need to use `String` or `Cow<str>` \
                            instead of `&str` to accept percent-decoded value"
                        ))
                    }
                    #[cfg(not(debug_assertions))] {
                        e
                    }
                }),
        }
    }
    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: serde::de::Visitor<'de> {
        self.deserialize_str(visitor)
    }
    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: serde::de::Visitor<'de> {
        let section = self.next_section()?;
        let decoded = percent_decode_utf8(section).map_err(|e|
            serde::de::Error::custom(format!("Expected to be decoded to an UTF-8, but got `{}`: {e}", section.escape_ascii()))
        )?;
        let mut chars = decoded.chars();
        let (Some(ch), None) = (chars.next(), chars.next()) else {
            return Err((|| serde::de::Error::custom(
                format!("Expected a single charactor, but got `{}`", section.escape_ascii())
            ))())
        };
        visitor.visit_char(ch)
    }

    #[inline]
    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where V: serde::de::Visitor<'de> {
        visitor.visit_newtype_struct(self)
    }

    #[inline]
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: serde::de::Visitor<'de> {
        if self.input.iter().position(|b| b==&b'&').unwrap_or(self.input.len()) == 0 {
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: serde::de::Visitor<'de> {
        if self.input.iter().position(|b| b==&b'&').unwrap_or(self.input.len()) == 0 {
            visitor.visit_unit()
        } else {
            Err((|| serde::de::Error::custom(format!(
                "Expected an empty value for an unit, but got `{}`",
                match self.next_section() {
                    Ok(section) => section.escape_ascii().to_string(),
                    Err(e) => e.to_string()
                }
            )))())
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

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: serde::de::Visitor<'de> {
        visitor.visit_seq(CommaSeparated::new(self))
    }
    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where V: serde::de::Visitor<'de> {
        self.deserialize_seq(visitor)
    }
    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where V: serde::de::Visitor<'de> {
        self.deserialize_seq(visitor)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: serde::de::Visitor<'de> {
        #[cfg(debug_assertions)] {
            assert!(self.side == ParsingSide::Value);
        }

        match percent_decode(self.next_section().unwrap()) {
            Cow::Borrowed(slice) => visitor.visit_bytes(slice),
            Cow::Owned(byte_vec) => visitor.visit_byte_buf(byte_vec),
        }
    }
    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: serde::de::Visitor<'de> {
        #[cfg(debug_assertions)] {
            assert!(self.side == ParsingSide::Value);
        }

        self.deserialize_bytes(visitor)
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: serde::de::Visitor<'de> {
        #[cfg(debug_assertions)] {
            assert!(self.side == ParsingSide::Value);
        }

        match self.next_section().unwrap() {
            b"true"  => visitor.visit_bool(true),
            b"false" => visitor.visit_bool(false),
            other   => Err(serde::de::Error::custom(format!(
                "Expected `true` or `false`, but got `{}`",
                other.escape_ascii()
            )))
        }
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: serde::de::Visitor<'de> {
        #[cfg(debug_assertions)] {
            assert!(self.side == ParsingSide::Value);
        }

        let section = self.next_section().unwrap();
        let section = std::str::from_utf8(section)
            .map_err(|_| serde::de::Error::custom(
                format!("Expected a number, but got `{}`", section.escape_ascii())
            ))?;
        visitor.visit_f32(
            section.parse().map_err(|_| serde::de::Error::custom(
                format!("Expected a number, but got `{section}`")
            ))?
        )
    }
    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: serde::de::Visitor<'de> {
        #[cfg(debug_assertions)] {
            assert!(self.side == ParsingSide::Value);
        }

        let section = self.next_section().unwrap();
        let section = std::str::from_utf8(section)
            .map_err(|_| serde::de::Error::custom(
                format!("Expected a number, but got `{}`", section.escape_ascii())
            ))?;
        visitor.visit_f64(
            section.parse().map_err(|_| serde::de::Error::custom(
                format!("Expected a number, but got `{section}`")
            ))?
        )
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: serde::de::Visitor<'de> {
        #[cfg(debug_assertions)] {
            assert!(self.side == ParsingSide::Value);
        }

        let section = self.next_section().unwrap();
        let section = std::str::from_utf8(section)
            .map_err(|_| serde::de::Error::custom(
                format!("Expected an integer, but got `{}`", section.escape_ascii())
            ))?;
        visitor.visit_i8(
            section.parse().map_err(|_| serde::de::Error::custom(
                format!("Expected an integer, but got `{section}`")
            ))?
        )
    }
    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: serde::de::Visitor<'de> {
        #[cfg(debug_assertions)] {
            assert!(self.side == ParsingSide::Value);
        }

        let section = self.next_section().unwrap();
        let section = std::str::from_utf8(section)
            .map_err(|_| serde::de::Error::custom(
                format!("Expected an integer, but got `{}`", section.escape_ascii())
            ))?;
        visitor.visit_i16(
            section.parse().map_err(|_| serde::de::Error::custom(
                format!("Expected an integer, but got `{section}`")
            ))?
        )
    }
    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: serde::de::Visitor<'de> {
        #[cfg(debug_assertions)] {
            assert!(self.side == ParsingSide::Value);
        }

        let section = self.next_section().unwrap();
        let section = std::str::from_utf8(section)
            .map_err(|_| serde::de::Error::custom(
                format!("Expected an integer, but got `{}`", section.escape_ascii())
            ))?;
        visitor.visit_i32(
            section.parse().map_err(|_| serde::de::Error::custom(
                format!("Expected an integer, but got `{section}`")
            ))?
        )
    }
    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: serde::de::Visitor<'de> {
        #[cfg(debug_assertions)] {
            assert!(self.side == ParsingSide::Value);
        }

        let section = self.next_section().unwrap();
        let section = std::str::from_utf8(section)
            .map_err(|_| serde::de::Error::custom(
                format!("Expected an integer, but got `{}`", section.escape_ascii())
            ))?;
        visitor.visit_i64(
            section.parse().map_err(|_| serde::de::Error::custom(
                format!("Expected an integer, but got `{section}`")
            ))?
        )
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: serde::de::Visitor<'de> {
        #[cfg(debug_assertions)] {
            assert!(self.side == ParsingSide::Value);
        }

        let section = self.next_section().unwrap();
        let section = std::str::from_utf8(section)
            .map_err(|_| serde::de::Error::custom(
                format!("Expected an integer, but got `{}`", section.escape_ascii())
            ))?;
        visitor.visit_u8(
            section.parse().map_err(|_| serde::de::Error::custom(
                format!("Expected an integer, but got `{section}`")
            ))?
        )
    }
    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: serde::de::Visitor<'de> {
        #[cfg(debug_assertions)] {
            assert!(self.side == ParsingSide::Value);
        }

        let section = self.next_section().unwrap();
        let section = std::str::from_utf8(section)
            .map_err(|_| serde::de::Error::custom(
                format!("Expected an integer, but got `{}`", section.escape_ascii())
            ))?;
        visitor.visit_u16(
            section.parse().map_err(|_| serde::de::Error::custom(
                format!("Expected an integer, but got `{section}`")
            ))?
        )
    }
    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: serde::de::Visitor<'de> {
        #[cfg(debug_assertions)] {
            assert!(self.side == ParsingSide::Value);
        }

        let section = self.next_section().unwrap();
        let section = std::str::from_utf8(section)
            .map_err(|_| serde::de::Error::custom(
                format!("Expected an integer, but got `{}`", section.escape_ascii())
            ))?;
        visitor.visit_u32(
            section.parse().map_err(|_| serde::de::Error::custom(
                format!("Expected an integer, but got `{section}`")
            ))?
        )
    }
    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: serde::de::Visitor<'de> {
        #[cfg(debug_assertions)] {
            assert!(self.side == ParsingSide::Value);
        }

        let section = self.next_section().unwrap();
        let section = std::str::from_utf8(section)
            .map_err(|_| serde::de::Error::custom(
                format!("Expected an integer, but got `{}`", section.escape_ascii())
            ))?;
        visitor.visit_u64(
            section.parse().map_err(|_| serde::de::Error::custom(
                format!("Expected an integer, but got `{section}`")
            ))?
        )
    }
}


struct AmpersandSeparated<'amp, 'de:'amp> {
    de:    &'amp mut URLEncodedDeserializer<'de>,
    first: bool,
}
impl<'amp, 'de> AmpersandSeparated<'amp, 'de> {
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
            if !self.first && self.de.next()? != b'&' {
                return Err((|| serde::de::Error::custom("missing `&`"))())
            }
            self.first = false;

            self.de.side = ParsingSide::Key;
            seed.deserialize(&mut *self.de).map(Some)
        }
        #[inline]
        fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
        where V: serde::de::DeserializeSeed<'de> {
            if self.de.next()? != b'=' {
                return Err((|| serde::de::Error::custom("missing `=`"))())
            }

            self.de.side = ParsingSide::Value;
            seed.deserialize(&mut *self.de)
        }
    }
};

struct Enum<'e, 'de:'e> {
    de: &'e mut URLEncodedDeserializer<'de>
}
impl<'e, 'de> Enum<'e, 'de> {
    fn new(de: &'e mut URLEncodedDeserializer<'de>) -> Self {
        Self { de }
    }
}
const _: () = {
    impl<'e, 'de> serde::de::EnumAccess<'de> for Enum<'e, 'de> {
        type Variant = Self;
        type Error   = super::Error;

        fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
        where V: serde::de::DeserializeSeed<'de> {
            Ok((
                seed.deserialize(self.de.next_section().unwrap().into_deserializer())?,
                self,
            ))
        }
    }

    impl<'e, 'de> serde::de::VariantAccess<'de> for Enum<'e, 'de> {
        type Error = super::Error;

        fn unit_variant(self) -> Result<(), Self::Error> {
            Ok(())
        }

        fn newtype_variant_seed<T>(self, _seed: T) -> Result<T::Value, Self::Error>
        where T: serde::de::DeserializeSeed<'de> {
            // seed.deserialize(self.de)
            
            Err(serde::de::Error::custom("ohkami's builtin urlencoded deserializer doesn't support enum with newtype variants !"))
        }

        fn struct_variant<V>(
            self,
            _fields: &'static [&'static str],
            _visitor: V,
        ) -> Result<V::Value, Self::Error>
        where V: serde::de::Visitor<'de> {
            Err(serde::de::Error::custom("ohkami's builtin urlencoded deserializer doesn't support enum with struct variants !"))
        }

        fn tuple_variant<V>(self, _len: usize, _visitor: V) -> Result<V::Value, Self::Error>
        where V: serde::de::Visitor<'de> {
            Err(serde::de::Error::custom("ohkami's builtin urlencoded deserializer doesn't support enum with tuple variants !"))
        }
    }
};

struct CommaSeparated<'de> {
    section: &'de [u8],
    first:   bool,
}
impl<'de> CommaSeparated<'de> {
    fn new(de: &mut URLEncodedDeserializer<'de>) -> Self {
        Self {
            section: de.next_section().unwrap(),
            first:   true,
        }
    }
}
const _: () = {
    impl<'de> serde::de::SeqAccess<'de> for CommaSeparated<'de> {
        type Error = super::Error;

        fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
        where T: serde::de::DeserializeSeed<'de> {
            if self.section.is_empty() {
                return Ok(None)
            }
            if !self.first && self.section.first() == Some(&b',') {
                return Err(serde::de::Error::custom("missing ,"))
            }
            self.first = false;

            let size = self.section.iter().position(|b| b==&b',').unwrap_or(self.section.len());
            let (element, remaining) = self.section.split_at(size);
            self.section = remaining;

            seed.deserialize(element.into_deserializer()).map(Some)
        }
    }
};
