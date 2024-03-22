use serde::de::IntoDeserializer;
use super::{File, Error};


pub(super) struct Multipart<'de> {
    parts: Vec<Part<'de>>, // vec reversed
}
pub(super) enum Part<'de> {
    File  { name: &'de str, file:    File<'de> },
    Field { name: &'de str, content: &'de str },
}

pub(super) struct NextFields<'de> {
    name: &'de str,
    item: FilesOrField<'de>,
}
pub(super) enum FilesOrField<'de> {
    Files(Vec<File<'de>>),
    Field(&'de str),
}

impl<'de> Multipart<'de> {
    pub(super) fn next(&mut self) -> NextFields<'de> {
        let mut parts_of_same_name = Vec::with_capacity(1);
        let Some(first) = self.parts.pop() else {return Vec::new()};
        if let Part::File { name, .. } = first {
            while self.parts.last()
                .is_some_and(|next| next.is_file() && next.name == first.name)
            {
                parts_of_same_name.push(unsafe {self.parts.pop().unwrap_unchecked()});
            }
        }
        parts_of_same_name.push(first);
        parts_of_same_name
    }
    pub(super) fn peek(&self) -> Option<&Part<'de>> {
        self.parts.last()
    }

    pub(super) fn parse(input: &'de [u8]) -> Result<Self, Error> {
        let mut r = ::byte_reader::Reader::new(input);

        #[inline(always)] #[allow(non_snake_case)]
        const fn UNTIL_CRLF(b: &u8) -> bool {
            *b != b'\r'
        }
        #[inline(always)]
        const fn detached<'d>(bytes: &[u8]) -> &'d [u8] {
            unsafe {std::slice::from_raw_parts(bytes.as_ptr(), bytes.len())}
        }
        #[inline(always)]
        fn detached_str<'d>(bytes: &[u8], if_not_utf8: fn()->Error) -> Result<&'d str, Error> {
            std::str::from_utf8(detached(bytes)).map_err(|_| if_not_utf8())
        }

        let boundary = {
            let _ = r.consume("--").ok_or_else(Error::ExpectedBoundary)?;
            let b = r.read_while(|b| b != &b'\r');
            detached(b)
        };

        let mut parts = Vec::new();
        while let Some(i) = r.consume_oneof(["\r\n", "--"]) {
            match i {
                0 => {
                    let mut name          = "";
                    let mut mime          = "";
                    let mut content       = &b""[..];
                    let mut filename      = None;
                    let mut mixed_boudary = None;

                    while r.consume("\r\n"/* A newline between headers and content */).is_none() {
                        let header = r.read_kebab().ok_or_else(Error::ExpectedValidHeader)?;
                        if header.eq_ignore_ascii_case("Content-Type") {
                            r.consume(": ").ok_or_else(Error::ExpectedValidHeader)?;
                            mime = detached_str(r.read_while(UNTIL_CRLF), Error::InvalidMimeType)?;
                            if mime == "multipart/mixed" {
                                r.consume(", boundary=").ok_or_else(Error::MissinSpecifyingMixedBoudary)?;
                                mixed_boudary = Some(detached(r.read_while(UNTIL_CRLF)));
                            }
                        } else if header.eq_ignore_ascii_case("Content-Disposition") {
                            r.consume(": form-data; name=").ok_or_else(Error::ExpectedFormdataAndName)?;
                            name = detached_str(
                                r.read_quoted_by(b'"', b'"').ok_or_else(Error::InvalidPartName)?,
                                Error::InvalidPartName)?;
                            if r.consume("; ").is_some() {
                                r.consume("filename=").ok_or_else(Error::ExpectedFilename)?;
                                filename = Some(detached_str(
                                    r.read_quoted_by(b'"', b'"').ok_or_else(Error::InvalidFilename)?,
                                    Error::InvalidFilename)?);
                            }
                        } else {
                            r.skip_while(|b| b != &b'\r');
                        }
                        r.consume("\r\n").ok_or_else(Error::MissingCRLF)?;
                    }

                    parts.push(match filename {
                        None => Part::Field {
                            name,
                            content: detached_str(content, Error::NotUTF8NonFileField)?,
                        },
                        Some(filename ) => Part::File {
                            name,
                            file: File { filename, mime, content }
                        },
                    })
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


const _: () = {
    impl<'de> serde::de::IntoDeserializer<'de, Error> for FilesOrField<'de> {
        type Deserializer = DeserializeFilesOrField<'de>;
        fn into_deserializer(self) -> Self::Deserializer {
            DeserializeFilesOrField {
                files_or_field: self,
                entry:          FileStructEntry::init()
            }
        }
    }
    pub(super) struct DeserializeFilesOrField<'de> {
        files_or_field: FilesOrField<'de>,
        entry:          FileStructEntry,
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

    impl<'de> serde::de::Deserializer<'de> for DeserializeFilesOrField<'de> {
        type Error = Error;

        fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: serde::de::Visitor<'de> {
            match &self.files_or_field {
                FilesOrField::Field(_) => self.deserialize_str(visitor),
                FilesOrField::Files(_) => self.deserialize_map(visitor),
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
            if let FilesOrField::Field(content) = self.files_or_field {
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
            let FilesOrField::Files(files) = &self.files_or_field
                else {return Err((|| Error::ExpectedFile())())};
            (files.len() == 1)
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

    impl<'de> serde::de::MapAccess<'de> for DeserializeFilesOrField<'de> {
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
                assert!(match &self.files_or_field {
                    FilesOrField::Files(files) => files.len() == 1,
                    _ => false
                });
            }

            let file = match &mut self.files_or_field {
                FilesOrField::Files(files) => unsafe {files.pop().unwrap_unchecked()},
                _ => unsafe {std::hint::unreachable_unchecked()}
            };

            let (k, v) = match &self.entry {
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
            self.entry.step();

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

            let DeserializeFilesOrField {
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
