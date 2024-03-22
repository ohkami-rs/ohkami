use serde::de::IntoDeserializer;

use super::{File, Error};


pub(super) struct Multipart<'de> {
    parts: Vec<Part<'de>>, // vec reversed
}
pub(super) struct Part<'de> {
    pub(super) name:    &'de str,
    pub(super) content: FileOrField<'de>,
}
pub(super) enum FileOrField<'de> {
    File(File<'de>),
    Field(&'de str),
}

impl<'de> Multipart<'de> {
    pub(super) fn next(&mut self) -> Option<Part<'de>> {
        self.parts.pop()
    }
    pub(super) fn peek(&self) -> Option<&Part<'de>> {
        self.parts.last()
    }

    pub(super) fn parse(input: &'de [u8]) -> Result<Self, Error> {
        let mut r = ::byte_reader::Reader::new(input);

        let boundary = {
            let _ = r.consume("--").ok_or_else(Error::ExpectedBoundary)?;
            let b = r.read_while(|b| b != &b'\r');
            // SAFETY:
            // 1. `b` has lifetime the same lifetime as `input` or `r`
            // 2. `r` never mutate `input` bytes themselves
            unsafe {std::slice::from_raw_parts(b.as_ptr(), b.len())}
        };

        let mut parts = Vec::new();
        while let Some(i) = r.consume_oneof(["\r\n", "--"]) {
            match i {
                0 => {
                    
                }
                1 => {
                    r.consume(boundary).ok_or_else(Error::ExpectedBoundary)?;
                    break
                }
                _ => unsafe {std::hint::unreachable_unchecked()}
            }
        }
        Ok(Self {parts})
    }
}

/*

        let mut r = Reader::new(input);

        r.consume("--").ok_or_else(Error::ExpectedValidBoundary)?;
        // SAFETY:
        // 1. What `boundary` refers to is `input`, that keeps alive
        //    for 'de, the same lifetime as `Self`
        // 2. `r` never changes the content of `input`

        let boundary = {
            let b = r.read_while(|b| b != &b'\r');
            unsafe {std::slice::from_raw_parts(b.as_ptr(), b.len())}
        };

        r.consume("\r\n").ok_or_else(Error::MissingCRLF)?;

*/


const _: () = {
    impl<'de> serde::de::IntoDeserializer<'de, Error> for FileOrField<'de> {
        type Deserializer = DeserializeFileOrField<'de>;
        fn into_deserializer(self) -> Self::Deserializer {
            DeserializeFileOrField {
                ff:    self,
                entry: FileStructEntry::init()
            }
        }
    }
    pub(super) struct DeserializeFileOrField<'de> {
        ff:    FileOrField<'de>,
        entry: FileStructEntry,
    }
    #[allow(non_camel_case_types)]
    enum FileStructEntry {
        filename,
        mime,
        content,
        __finished
    } impl FileStructEntry {
        const fn init() -> Self {
            Self::filename
        }
        fn step(&mut self) {
            match &self {
                Self::filename => *self = Self::mime,
                Self::mime     => *self = Self::content,
                Self::content  => *self = Self::__finished,
                Self::__finished => ()
            }
        }
    }

    impl<'de> serde::de::Deserializer<'de> for DeserializeFileOrField<'de> {
        type Error = Error;

        fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: serde::de::Visitor<'de> {
            match &self.ff {
                FileOrField::Field(_) => self.deserialize_str(visitor),
                FileOrField::File(_)  => self.deserialize_map(visitor),
            }
        }
        fn deserialize_newtype_struct<V>(
            self,
            _name: &'static str,
            visitor: V,
        ) -> Result<V::Value, Self::Error>
        where V: serde::de::Visitor<'de> {
            visitor.visit_newtype_struct(self)
        }

        fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: serde::de::Visitor<'de> {
            if let FileOrField::Field(content) = self.ff {
                visitor.visit_str(content)
            } else {
                Err((|| Error::ExpectedNonFileField())())
            }
        }

        fn deserialize_struct<V>(
            self,
            _name: &'static str,
            _fields: &'static [&'static str],
            visitor: V,
        ) -> Result<V::Value, Self::Error>
        where V: serde::de::Visitor<'de> {
            self.deserialize_map(visitor)
        }
        fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: serde::de::Visitor<'de> {
            matches!(&self.ff, FileOrField::File(_))
                .then_some(visitor.visit_map(self)?)
                .ok_or_else(Error::ExpectedFile)
        }

        serde::forward_to_deserialize_any! {
            i8 i16 i32 i64 u8 u16 u32 u64 f32 f64
            string char bool
            bytes byte_buf
            seq enum option identifier
            unit unit_struct tuple tuple_struct
            ignored_any
        }
    }

    impl<'de> serde::de::MapAccess<'de> for DeserializeFileOrField<'de> {
        type Error = Error;

        fn next_entry_seed<K, V>(
            &mut self,
            kseed: K,
            vseed: V,
        ) -> Result<Option<(K::Value, V::Value)>, Self::Error>
        where
            K: serde::de::DeserializeSeed<'de>,
            V: serde::de::DeserializeSeed<'de>,
        {
            #[cfg(debug_assertions)] {
                // This SHOULD be already checked in `deserialize_map`,
                // BEFORE `next_entry_seed` here
                assert!(matches!(&self.ff, FileOrField::File(_)));
            }

            let DeserializeFileOrField {
                ff: FileOrField::File(file), entry
            } = self else {unsafe {std::hint::unreachable_unchecked()}};

            let (k, v) = match entry {
                FileStructEntry::__finished => return Ok(None),

                FileStructEntry::filename => (
                    kseed.deserialize("filename".into_deserializer())?,
                    vseed.deserialize(file.filename.into_deserializer())?
                ),
                FileStructEntry::mime     => (
                    kseed.deserialize("mime".into_deserializer())?,
                    vseed.deserialize(file.mime.into_deserializer())?
                ),
                FileStructEntry::content  => (
                    kseed.deserialize("content".into_deserializer())?,
                    vseed.deserialize(file.content.into_deserializer())?
                ),
            };
            entry.step();

            Ok(Some((k, v)))
        }

        fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
        where K: serde::de::DeserializeSeed<'de> {
            #[cfg(debug_assertions)] {
                // This SHOULD be already checked in `deserialize_map`,
                // BEFORE `next_entry_seed` here
                assert!(matches!(&self.ff, FileOrField::File(_)));
            }

            seed.deserialize(match &self.entry {
                FileStructEntry::__finished => return Ok(None),

                FileStructEntry::filename => "filename",
                FileStructEntry::mime     => "mime",
                FileStructEntry::content  => "content",
            }.into_deserializer()).map(Some)
        }
        fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
        where V: serde::de::DeserializeSeed<'de> {
            #[cfg(debug_assertions)] {
                // This SHOULD be already checked in `deserialize_map`,
                // BEFORE `next_entry_seed` here
                assert!(matches!(&self.ff, FileOrField::File(_)));
            }

            let DeserializeFileOrField {
                ff: FileOrField::File(file), entry
            } = self else {unsafe {std::hint::unreachable_unchecked()}};

            let v = match entry {
                FileStructEntry::__finished => unsafe {// SAFETY: already checked in `next_key_seed`
                    std::hint::unreachable_unchecked()
                },

                FileStructEntry::filename => seed.deserialize(file.filename.into_deserializer()),
                FileStructEntry::mime     => seed.deserialize(file.mime.into_deserializer()),
                FileStructEntry::content  => seed.deserialize(file.content.into_deserializer()),
            };
            entry.step();

            v
        }
    }
};
