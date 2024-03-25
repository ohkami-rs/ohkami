use serde::de::IntoDeserializer;
use super::{File, Error};


#[derive(Debug, PartialEq)]
pub(super) struct Multipart<'de>(
    pub(super/* for test */) Vec<Part<'de>>,
);
#[derive(Debug, PartialEq)]
pub(super) enum Part<'de> {
    Text { name: &'de str, text: &'de str },
    File { name: &'de str, file: File<'de> },
}

#[derive(Debug, PartialEq)]
pub(super) struct Next<'de> {
    pub(crate) name: &'de str,
    pub(crate) item: TextOrFiles<'de>,
}
#[derive(Debug, PartialEq)]
pub(super) enum TextOrFiles<'de> {
    Text (&'de str),
    Files(Vec<File<'de>>),
}

impl<'de> Multipart<'de> {
    pub(super) fn next(&mut self) -> Option<Next<'de>> {
        Some(match self.0.pop()? {
            Part::Text { name, text } => Next {
                name,
                item: TextOrFiles::Text(text),
            },
            Part::File { name, file } => {
                if file.filename.is_empty() && file.content.is_empty() {
                    return Some(Next { name, item: TextOrFiles::Files(Vec::new()) })
                }

                let mut files = vec![file];
                while self.peek().is_some_and(|part| match part {
                    Part::File { name: next_name, .. } => name == *next_name,
                    Part::Text { .. } => false,
                }) {
                    let Some(Part::File { file, .. }) = self.0.pop()
                        else {unsafe {std::hint::unreachable_unchecked()}};
                    files.push(file);
                }

                Next {
                    name,
                    item: TextOrFiles::Files(files)
                }
            }
        })
    }
    pub(super) fn peek(&self) -> Option<&Part<'de>> {
        self.0.last()
    }

    pub(super) fn parse(input: &'de [u8]) -> Result<Self, Error> {
        const CRLF: &[u8] = b"\r\n";

        #[inline(always)]
        fn utf8(bytes: &[u8], if_not_utf8: fn()->Error) -> Result<&str, Error> {
            std::str::from_utf8(bytes).map_err(|_| if_not_utf8())
        }

        let mut r = ::byte_reader::Reader::new(input);

        /* including leading `--` */
        let boundary = r.read_until(CRLF);

        let mut parts = Vec::new();
        while let Some(i) = r.consume_oneof(["\r\n", "--"]) {
            match i {
                0 => {
                    let mut name     = "";
                    let mut mimetype = "";
                    let mut filename = None;
                    while r.consume("\r\n"/* A newline between headers and content */).is_none() {
                        let header = r.read_kebab().ok_or_else(Error::ExpectedValidHeader)?;
                        if header.eq_ignore_ascii_case("Content-Type") {
                            r.consume(": ").ok_or_else(Error::ExpectedValidHeader)?;
                            mimetype = utf8(r.read_until(CRLF), Error::InvalidMimeType)?;
                            (mimetype != "multipart/mixed").then_some(())
                                .ok_or_else(Error::NotSupportedMultipartMixed)?;
                        } else if header.eq_ignore_ascii_case("Content-Disposition") {
                            r.consume(": form-data; name=").ok_or_else(Error::ExpectedFormdataAndName)?;
                            name = utf8(
                                r.read_quoted_by(b'"', b'"').ok_or_else(Error::InvalidPartName)?,
                                Error::InvalidPartName)?;
                            if r.consume("; ").is_some() {
                                r.consume("filename=").ok_or_else(Error::ExpectedFilename)?;
                                filename = Some(utf8(
                                    r.read_quoted_by(b'"', b'"').ok_or_else(Error::InvalidFilename)?,
                                    Error::InvalidFilename)?);
                            }
                        } else {
                            r.skip_while(|b| b != &b'\r');
                        }
                        r.consume("\r\n").ok_or_else(Error::MissingCRLF)?;
                    }

                    let content = {
                        let before_boundary = r.read_until(boundary);
                        let before_boundary_len = before_boundary.len();
                        let Some((content, CRLF)) = (before_boundary_len >= CRLF.len()).then_some(unsafe {
                            use std::slice::from_raw_parts;

                            let ptr = before_boundary.as_ptr();
                            let mid = before_boundary_len - CRLF.len();
                            (from_raw_parts(ptr, mid), from_raw_parts(ptr.add(mid), CRLF.len()))
                        }) else {return Err((|| Error::MissingCRLF())())};

                        r.consume(boundary).ok_or_else(Error::ExpectedBoundary)?;

                        content
                    };

                    parts.push(match filename {
                        None => Part::Text {
                            name,
                            text: utf8(content, Error::NotUTF8NonFileField)?,
                        },
                        Some(filename) => Part::File {
                            name,
                            file: File { filename, mimetype, content }
                        },
                    })
                }
                1 => break,
                _ => unsafe {std::hint::unreachable_unchecked()}
            }
        }

        Ok(Self(parts))
    }
}


const _: () = {
    impl<'de> serde::de::IntoDeserializer<'de, Error> for TextOrFiles<'de> {
        type Deserializer = DeserializeFilesOrField<'de>;
        fn into_deserializer(self) -> Self::Deserializer {
            DeserializeFilesOrField {
                text_ot_files: self
            }
        }
    }
    pub(super) struct DeserializeFilesOrField<'de> {
        text_ot_files: TextOrFiles<'de>
    }

    impl<'de> serde::de::Deserializer<'de> for DeserializeFilesOrField<'de> {
        type Error = Error;

        fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: serde::de::Visitor<'de> {
            match &self.text_ot_files {
                TextOrFiles::Text(_)  => self.deserialize_str(visitor),
                TextOrFiles::Files(_) => self.deserialize_map(visitor),
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
            if let TextOrFiles::Text(text) = self.text_ot_files {
                visitor.visit_borrowed_str(text)
            } else {
                Err((|| Error::ExpectedNonFileField())())
            }
        }
        fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: serde::de::Visitor<'de> {
            self.deserialize_str(visitor)
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
        fn deserialize_map<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
        where V: serde::de::Visitor<'de> {
            let TextOrFiles::Files(files) = &mut self.text_ot_files else {
                return Err((|| Error::ExpectedFile())())
            };
            (files.len() == 1)
                .then_some({
                    let file = unsafe {files.pop().unwrap_unchecked()};
                    visitor.visit_map(file.into_deserializer())?
                })
                .ok_or_else(Error::UnexpectedMultipleFiles)
        }

        fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: serde::de::Visitor<'de> {
            let TextOrFiles::Files(_) = &self.text_ot_files else {
                return Err((|| Error::ExpectedFile())())
            };
            visitor.visit_seq(self)
        }

        fn deserialize_option<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
        where V: serde::de::Visitor<'de> {
            match &mut self.text_ot_files {
                TextOrFiles::Files(files) => match files.len() {
                    0 => visitor.visit_none(),
                    1 => visitor.visit_some(unsafe {files.pop().unwrap_unchecked()}.into_deserializer()),
                    _ => Err((|| Error::UnexpectedMultipleFiles())())
                },
                TextOrFiles::Text(text) => match text.len() {
                    0 => visitor.visit_none(),
                    _ => visitor.visit_some(serde::de::value::BorrowedStrDeserializer::new(text)),
                }
            }
        }

        serde::forward_to_deserialize_any! {
            i8 i16 i32 i64 u8 u16 u32 u64 f32 f64
            char bool
            bytes byte_buf
            enum identifier
            unit unit_struct tuple tuple_struct
            ignored_any
        }
    }

    impl<'de> serde::de::SeqAccess<'de> for DeserializeFilesOrField<'de> {
        type Error = Error;

        fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
        where T: serde::de::DeserializeSeed<'de> {
            #[cfg(debug_assertions)] {
                // This SHOULD be already checked in `deserialize_map`,
                // BEFORE `next_entry_seed` here
                assert!(match &self.text_ot_files {
                    TextOrFiles::Files(_) => true,
                    _ => false
                });
            }
            
            let TextOrFiles::Files(files) = &mut self.text_ot_files else {
                unsafe {std::hint::unreachable_unchecked()}
            };

            let next = match files.pop() {
                Some(file) => file,
                None       => return Ok(None),
            };

            seed.deserialize(next.into_deserializer()).map(Some)
        }
    }
};
