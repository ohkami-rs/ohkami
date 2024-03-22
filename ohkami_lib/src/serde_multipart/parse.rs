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
        todo!()
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


struct DeserializeFileOrField<'de> {
    ff:    FileOrField<'de>,
    entry: FileStructEntry,
}

#[allow(non_camel_case_types)]
enum FileStructEntry {
    filename,
    mime,
    content,
    Finished
}
impl FileStructEntry {
    const fn init() -> Self {
        Self::filename
    }
    const fn as_str(&self) -> &'static str {
        match self {
            Self::filename => "filename",
            Self::mime     => "mime",
            Self::content  => "content",
            Self::Finished => {
                #[cfg(debug_assertions)] {
                    panic!("Unexpectedly called `FileStructEntry::as_str` for variant `Finished`")
                } #[cfg(not(debug_assertions))] {
                    unsafe {std::hint::unreachable_unchecked()}
                }
            }
        }
    }
    fn step(&mut self) {
        match &self {
            Self::filename => *self = Self::mime,
            Self::mime     => *self = Self::content,
            Self::content  => *self = Self::Finished,
            Self::Finished => ()
        }
    }
}

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

    impl<'de> serde::de::Deserializer<'de> for DeserializeFileOrField<'de> {
        type Error = Error;

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
        fn deserialize_newtype_struct<V>(
            self,
            _name: &'static str,
            visitor: V,
        ) -> Result<V::Value, Self::Error>
        where V: serde::de::Visitor<'de> {
            visitor.visit_newtype_struct(self)
        }

        fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: serde::de::Visitor<'de> {
            match &self.ff {
                FileOrField::Field(_) => self.deserialize_str(visitor),
                FileOrField::File(_)  => self.deserialize_map(visitor),
            }
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

            let k = kseed.deserialize(entry.as_str().into_deserializer())?;
            let v = vseed.deserialize()?;

            entry.step();
            
            Ok(Some((k, v)))
        }
    }
};
