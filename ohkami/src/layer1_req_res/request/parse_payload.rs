use std::{borrow::Cow, format as f};

use serde::Deserialize;
use byte_reader::Reader;
use percent_encoding::percent_decode;

fn __unreachable__() -> ! {
    unsafe {std::hint::unreachable_unchecked()}
}




/*===== for #[Payload(JSON)] =====*/
#[inline]
pub fn parse_json<'req, T: Deserialize<'req>>(buf: &'req [u8]) -> Result<T, Cow<'static, str>> {
        serde_json::from_slice(buf)
            .map_err(|e| Cow::Owned(e.to_string()))
}




/*===== for #[Payload(FormData)] =====*/
pub struct FormPart {
    name:    String,
    content: FormContent,
} impl FormPart {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn into_content(self) -> FormContent {
        self.content
    }
}

pub enum FormContent {
    Content(Content),
    Files(Vec<File>),
    File(File),
}

pub struct Content {
    mime_type: String,
    content:   Vec<u8>,
} impl Content {
    pub fn mime_type(&self) -> &str {
        &self.mime_type
    }
    pub fn content(&self) -> &[u8] {
        &self.content
    }
} impl Content {
    pub fn text(self) -> Result<String, ::std::string::FromUtf8Error> {
        String::from_utf8(self.content)
    }
    pub unsafe fn text_unchecked(self) -> String {
        String::from_utf8_unchecked(self.content)
    }
}

pub struct File {
    name:      Option<String>,
    mime_type: String,
    content:   Vec<u8>,
} impl File {
    pub fn name(&self) -> Option<&str> {
        self.name.as_ref().map(|s| s.as_str())
    }
    pub fn mime_type(&self) -> &str {
        &self.mime_type
    }
    pub fn content(&self) -> &[u8] {
        &self.content
    }
}

/// return
/// 
/// - `Some(FormPart)` if `buf` contains a form part
/// - `None` if `buf` only contains end boundary `--boundary--`
pub fn parse_formpart(buf: &[u8], boundary: &str) -> Result<Option<FormPart>, &'static str> {
    let mut name           = String::new();
    let mut file_name      = None;
    let mut mime_type      = None;
    let mut mixed_boundary = None;
    
    let mut r = Reader::from(buf);

    r.consume("--")    .ok_or_else(EXPECTED_VALID_BOUNDARY)?;
    r.consume(boundary).ok_or_else(EXPECTED_VALID_BOUNDARY)?;
    if r.consume("\r\n").is_none() {
        // This was `--boundary--` : an end boundary
        return Ok(None)
    }
    
    while r.consume("\r\n").is_none(/* The newline just before body of this part */) {
        let header = r.read_kebab().ok_or_else(EXPECTED_VALID_HEADER)?;
        if header.eq_ignore_ascii_case("Content-Type:") {r.skip_while(|b| b==&b' ');
            let content_type = r.read_while(|b| !matches!(b, b'\r' | b','));
            if content_type == b"multipart/mixed" {
                r.consume(",")        .ok_or_else(EXPECTED_BOUNDARY)?;
                r.skip_while(|b| b==&b' ');
                r.consume("boundary=").ok_or_else(EXPECTED_BOUNDARY)?;
                mixed_boundary = Some(String::from_utf8(r.read_while(|b| b != &b'\r').to_vec()).map_err(|_| INVALID_FILENANE())?);
            } else {
                mime_type = Some(String::from_utf8(
                    content_type.to_vec()
                ).map_err(|_| INVALID_CONTENT_TYPE())?);
            }
        } else if header.eq_ignore_ascii_case("Content-Disposition:") {r.skip_while(|b| b==&b' ');
            r.consume("form-data").ok_or_else(EXPECTED_FORMDATA_AND_NAME)?;
            r.consume(";")        .ok_or_else(EXPECTED_FORMDATA_AND_NAME)?;
            r.skip_while(|b| b==&b' ');
            r.consume("name=")    .ok_or_else(EXPECTED_FORMDATA_AND_NAME)?;
            name = r.read_string().ok_or_else(EXPECTED_FORMDATA_AND_NAME)?;
            if r.peek().unwrap() != &b'\r' {
                r.skip_while(|b| b==&b' ');
                r.consume("filename=").ok_or_else(EXPECTED_FILENAME)?;
                file_name = Some(r.read_string().ok_or_else(EXPECTED_FILENAME)?)
            }
        } else {// ignore the line
            r.skip_while(|b| b != &b'\r');
        }
        r.consume("\r\n").unwrap();
    }

    if let Some(boundary) = mixed_boundary {
        let mut attachments = parse_attachments(&mut r, &boundary)?;
        let content = if attachments.len() == 1 {
            FormContent::File(unsafe {attachments.pop().unwrap_unchecked()})
        } else {
            FormContent::Files(attachments)
        };

        Ok(Some(FormPart { name, content }))
    } else {
        let mut content = Vec::new(); loop {
            let line = r.read_while(|b| b != &b'\r');

            if line.len() == 0 || is_end_boundary(line, boundary) {
                r.consume("\r\n")/* Maybe no `\r\n` if this is final part */;
                break
            }
            for b in line {content.push(*b)}
            r.consume("\r\n").unwrap();
        }
        let content = if let Some(file_name) = file_name {
            FormContent::File(File {
                name:      Some(file_name),
                mime_type: mime_type.unwrap_or_else(|| f!("text/plain")),
                content
            })
        } else {
            FormContent::Content(Content {
                mime_type: mime_type.unwrap_or_else(|| f!("text/plain")),
                content,
            })
        };

        Ok(Some(FormPart { name, content }))
    } 
}

pub(super/* for test */) fn parse_attachments(r: &mut Reader<&[u8]>, boundary: &str) -> Result<Vec<File>, &'static str> {
    let mut attachments = Vec::new();
    loop {
        r.consume("--").ok_or_else(EXPECTED_VALID_BOUNDARY)?;
        r.consume(boundary).ok_or_else(EXPECTED_VALID_BOUNDARY)?;
        if r.consume("\r\n").is_some() {
            let mut file = File { name: None, mime_type: f!("text/plain"), content: vec![] };
            loop {
                if r.consume("\r\n").is_some() {break}

                let header = r.read_kebab().ok_or_else(EXPECTED_VALID_HEADER)?;
                if header.eq_ignore_ascii_case("Content-Type") {
                    r.consume(":").unwrap(); r.skip_while(|b| b==&b' ');
                    file.mime_type = String::from_utf8(
                        r.read_while(|b| b != &b'\r').to_vec()
                    ).map_err(|_| INVALID_CONTENT_TYPE())?;
                    r.consume("\r\n").unwrap();
                } else if header.eq_ignore_ascii_case("Content-Disposition") {
                    r.consume(":").unwrap(); r.skip_while(|b| b==&b' ');
                    r.consume("attachment").ok_or_else(EXPECTED_ATTACHMENT)?;
                    if r.consume(";").is_some() {r.skip_while(|b| b==&b' ');
                        r.consume("filename=").ok_or_else(EXPECTED_FILENAME)?;
                        file.name = Some(r.read_string().ok_or_else(INVALID_FILENANE)?);
                        r.consume("\r\n").unwrap();
                    }
                } else {// ignore the line
                    r.skip_while(|b| b != &b'\r');
                    r.consume("\r\n").unwrap();
                }
            }

            loop {
                let line = r.read_while(|b| b != &b'\r');
                if is_end_boundary(line, boundary) {
                    r.consume("\r\n")/* Maybe no `\r\n` if this is final part */;
                    break
                } else if line.len() == 0 {
                    
                }
                for b in line {
                    file.content.push(*b)
                }
            }
        }
        r.consume("--").unwrap();
        break Ok(attachments)
    }
}

fn is_end_boundary(line: &[u8], boundary: &str) -> bool {
    use std::slice::from_raw_parts as raw;
    line.len() == (2 + boundary.len() + 2) && unsafe {
        let p = line.as_ptr();
        raw(p, 2) == &[b'-', b'-'] &&
        raw(p, boundary.len()) == boundary.as_bytes() &&
        raw(p, 2) == &[b'-', b'-']
    }
}

#[allow(non_snake_case)] const fn EXPECTED_VALID_BOUNDARY() -> &'static str {
    "Expected valid form-data boundary"
}
#[allow(non_snake_case)] const fn EXPECTED_ATTACHMENT() -> &'static str {
    "Expected `attachment`"
}
#[allow(non_snake_case)] const fn EXPECTED_FILENAME() -> &'static str {
    "Expected `filename=\"...\"`"
}
#[allow(non_snake_case)] const fn EXPECTED_BOUNDARY() -> &'static str {
    "Expected `boundary=\"...\"`"
}
#[allow(non_snake_case)] const fn EXPEXTED_CRLF() -> &'static str {
    "Expected `\\r\\n`"
}
#[allow(non_snake_case)] const fn EXPECTED_VALID_HEADER() -> &'static str {
    "Expected `Content-Type` or `Content-Disposition`"
}
#[allow(non_snake_case)] const fn EXPECTED_FORMDATA_AND_NAME() -> &'static str {
    "Expected `form-data; name=\"...\"`"
}
#[allow(non_snake_case)] const fn INVALID_FILENANE() -> &'static str {
    "Invalid filename"
}
#[allow(non_snake_case)] const fn INVALID_CONTENT_TYPE() -> &'static str {
    "Invalid Content-Type"
}


/*===== for #[Payload(URLEncoded)] =====*/
/* Thanks: https://github.com/servo/rust-url/blob/master/form_urlencoded/src/lib.rs */

/// Convert a byte string in the `application/x-www-form-urlencoded` syntax
/// into a iterator of (name, value) pairs.
///
/// Use `parse(input.as_bytes())` to parse a `&str` string.
///
/// The names and values are percent-decoded. For instance, `%23first=%25try%25` will be
/// converted to `[("#first", "%try%")]`.
#[inline]
pub fn parse_urlencoded(input: &[u8]) -> Parse<'_> {
    Parse { input }
}
/// The return type of `parse()`.
#[derive(Copy, Clone)]
pub struct Parse<'a> {
    input: &'a [u8],
} const _: () = {
    impl<'a> Iterator for Parse<'a> {
        type Item = (Cow<'a, str>, Cow<'a, str>);

        fn next(&mut self) -> Option<Self::Item> {
            loop {
                if self.input.is_empty() {
                    return None;
                }
                let mut split2 = self.input.splitn(2, |&b| b == b'&');
                let sequence = split2.next().unwrap();
                self.input = split2.next().unwrap_or(&[][..]);
                if sequence.is_empty() {
                    continue;
                }
                let mut split2 = sequence.splitn(2, |&b| b == b'=');
                let name = split2.next().unwrap();
                let value = split2.next().unwrap_or(&[][..]);
                return Some((decode(name), decode(value)));
            }
        }
    }

    fn decode(input: &[u8]) -> Cow<'_, str> {
        let replaced = replace_plus(input);
        decode_utf8_lossy(match percent_decode(&replaced).into() {
            Cow::Owned(vec) => Cow::Owned(vec),
            Cow::Borrowed(_) => replaced,
        })
    }

    /// Replace b'+' with b' '
    fn replace_plus(input: &[u8]) -> Cow<'_, [u8]> {
        match input.iter().position(|&b| b == b'+') {
            None => Cow::Borrowed(input),
            Some(first_position) => {
                let mut replaced = input.to_owned();
                replaced[first_position] = b' ';
                for byte in &mut replaced[first_position + 1..] {
                    if *byte == b'+' {
                        *byte = b' ';
                    }
                }
                Cow::Owned(replaced)
            }
        }
    }

    fn decode_utf8_lossy(input: Cow<'_, [u8]>) -> Cow<'_, str> {
        // Note: This function is duplicated in `percent_encoding/lib.rs`.
        match input {
            Cow::Borrowed(bytes) => String::from_utf8_lossy(bytes),
            Cow::Owned(bytes) => {
                match String::from_utf8_lossy(&bytes) {
                    Cow::Borrowed(utf8) => {
                        // If from_utf8_lossy returns a Cow::Borrowed, then we can
                        // be sure our original bytes were valid UTF-8. This is because
                        // if the bytes were invalid UTF-8 from_utf8_lossy would have
                        // to allocate a new owned string to back the Cow so it could
                        // replace invalid bytes with a placeholder.

                        // First we do a debug_assert to confirm our description above.
                        let raw_utf8: *const [u8] = utf8.as_bytes();
                        debug_assert!(raw_utf8 == &*bytes as *const [u8]);

                        // Given we know the original input bytes are valid UTF-8,
                        // and we have ownership of those bytes, we re-use them and
                        // return a Cow::Owned here.
                        Cow::Owned(unsafe { String::from_utf8_unchecked(bytes) })
                    }
                    Cow::Owned(s) => Cow::Owned(s),
                }
            }
        }
    }
};




/*=====*/




#[cfg(test)] #[allow(unused)] struct T1 {
    id:   usize,
    name: String,
    age:  u8,
}
#[cfg(test)] fn __1(buf: &[u8]) -> Result<T1, ::std::borrow::Cow<'static, str>> {
    let mut id   = ::std::option::Option::<usize>::None;
    let mut name = ::std::option::Option::<String>::None;
    let mut age  = ::std::option::Option::<u8>::None;

    for (k, v) in crate::__internal__::parse_urlencoded(buf) {
        match &*k {
            "id"   => id.replace(<usize as crate::__internal__::FromBuffer>::parse(v.as_bytes())?)
                .map_or(::std::result::Result::Ok(()), |_|
                    ::std::result::Result::Err(::std::borrow::Cow::Borrowed("duplicated key: `id`"))
                )?,
            "name" => name.replace(<String as crate::__internal__::FromBuffer>::parse(v.as_bytes())?)
                .map_or(::std::result::Result::Ok(()), |_|
                    ::std::result::Result::Err(::std::borrow::Cow::Borrowed("duplicated key: `name`"))
                )?,
            "age"  => age.replace(<u8 as crate::__internal__::FromBuffer>::parse(v.as_bytes())?)
                .map_or(::std::result::Result::Ok(()), |_|
                    ::std::result::Result::Err(::std::borrow::Cow::Borrowed("duplicated key: `age`")),
                )?,
            unexpected => return Err(::std::borrow::Cow::Owned(format!("unexpected key: `{unexpected}`"))),
        }
    }

    ::std::result::Result::Ok(T1 {
        id:   id.ok_or_else(|| ::std::borrow::Cow::Borrowed("`id` is not found"))?,
        name: name.ok_or_else(|| ::std::borrow::Cow::Borrowed("`name` is not found"))?,
        age:  age.ok_or_else(|| ::std::borrow::Cow::Borrowed("`age` is not found"))?,
    })
}

#[cfg(test)] #[allow(unused)] struct T2 {
    id:   usize,
    name: String,
    age:  Option<u8>,
}
#[cfg(test)] fn __2(buf: &[u8]) -> ::std::result::Result<T2, ::std::borrow::Cow<'static, str>> {
    let mut id   = ::std::option::Option::<usize>::None;
    let mut name = ::std::option::Option::<String>::None;
    let mut age  = ::std::option::Option::<u8>::None;

    for (k, v) in crate::__internal__::parse_urlencoded(buf) {
        match &*k {
            "id"   => id.replace(<usize as crate::__internal__::FromBuffer>::parse(v.as_bytes())?)
                .map_or(::std::result::Result::Ok(()), |_|
                    ::std::result::Result::Err(::std::borrow::Cow::Borrowed("duplicated key: `id`"))
                )?,
            "name" => name.replace(<String as crate::__internal__::FromBuffer>::parse(v.as_bytes())?)
                .map_or(::std::result::Result::Ok(()), |_|
                    ::std::result::Result::Err(::std::borrow::Cow::Borrowed("duplicated key: `name`"))
                )?,
            "age"  => age.replace(<u8 as crate::__internal__::FromBuffer>::parse(v.as_bytes())?)
                .map_or(::std::result::Result::Ok(()), |_|
                    ::std::result::Result::Err(::std::borrow::Cow::Borrowed("duplicated key: `age`")),
                )?,
            unexpected => return Err(::std::borrow::Cow::Owned(format!("unexpected key: `{unexpected}`"))),
        }
    }

    ::std::result::Result::Ok(T2 {
        id:   id.ok_or_else(|| ::std::borrow::Cow::Borrowed("`id` is not found"))?,
        name: name.ok_or_else(|| ::std::borrow::Cow::Borrowed("`name` is not found"))?,
        age:  age,
    })
}
