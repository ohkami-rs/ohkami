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
pub fn parse_formpart(buf: &[u8], boundary: &str) -> Option<FormPart> {
    let mut name           = String::new();
    let mut file_name      = None;
    let mut mime_type      = None;
    let mut mixed_boundary = None;
    
    let mut r = Reader::new(buf);

    r.consume("--").expect("Expected valid form-data boundary");
    r.consume(boundary).expect("Expected valid form-data boundary");
    r.consume("\r\n")/* return None if this wasn't `\r\n` (so was `--`) */?;

    while r.consume("\r\n"/* `\r\n` just before body of this part */).is_none() {
        let header = r.read_kebab().unwrap();

        if header.eq_ignore_ascii_case("Content-Type:") {
            r.skip_whitespace();

            let content_type = r.read_while(|b| !matches!(b, b'\r' | b','));
            if content_type == b"multipart/mixed" {
                r.consume(",").expect("Expected `boundary=\"...\"`");
                r.skip_whitespace();
                r.consume("boundary").expect("Expected `boundary=\"...\"");
                mixed_boundary = Some(r.read_while(|b| b != &b'\r'))
            } else {
                mime_type = Some(String::from_utf8(
                    content_type.to_vec()
                ).expect("Invalid Content-Type"));
            }

            r.consume("\r\n").unwrap();
        }

        else if header.eq_ignore_ascii_case("Content-Disposition:") {
            r.skip_whitespace();
            match r.consume_oneof(["form-data", "attachment"]).expect("Expected `form-data` or `attchment`") {
                0 => {
                    r.consume(";").unwrap(); r.skip_whitespace();
                    r.consume("name=").expect("Expected `name` in form part");
                    name = r.read_string().unwrap();
                }
                1 => {
                    if r.consume(";").is_some() {r.skip_whitespace();
                        r.consume("filename=").expect("Expected `filename`");
                        file_name = Some(r.read_string().unwrap());
                    }
                }
                _  => __unreachable__()
            }
            r.consume("\r\n").unwrap();
        }

        else {// ignore the line
            r.skip_while(|b| b != &b'\r');
            r.consume("\r\n").unwrap();
        }
    }

    if let Some(boundary_bytes) = mixed_boundary {
        let mut attachments = parse_attachments(&mut r, boundary_bytes);
        let content = if attachments.len() == 1 {
            FormContent::File(unsafe {attachments.pop().unwrap_unchecked()})
        } else {
            FormContent::Files(attachments)
        };

        Some(FormPart { name, content })
    } else {
        let mut content = Vec::new(); loop {
            let line = r.read_while(|b| b != &b'\r');

            if is_end_boundary(line, boundary) {
                r.consume("\r\n")?/* Maybe no `\r\n` if this is final part */;
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

        Some(FormPart { name, content })
    } 
}

fn parse_attachments(r: &mut Reader<&[u8]>, boundary: &[u8]) -> Vec<File> {
    let mut attachments = Vec::new();
    loop {
        r.consume("--").expect("Expected valid form-data boundary");
        r.consume(boundary).expect("Expected valid form-data boundary");
        if r.consume("\r\n").is_some() {
            let mut file = File { name: None, mime_type: f!("text/plain"), content: vec![] };
            loop {
                if r.consume("\r\n").is_some() {break attachments.push(file)}

                let header = r.read_kebab().expect("Expected `Content-Type` or `Content-Disposition`");
                if header.eq_ignore_ascii_case("Content-Type") {
                    r.consume(":").unwrap(); r.skip_whitespace();
                    file.mime_type = String::from_utf8(
                        r.read_while(|b| b != &b'\r').to_vec()
                    ).expect("Invalid Content-Type");
                    r.consume("\r\n").unwrap();
                } else if header.eq_ignore_ascii_case("Content-Disposition") {
                    r.consume(":").unwrap(); r.skip_whitespace();
                    r.consume("attachment").expect("Expected `attachment`");
                    if r.consume(";").is_some() {r.skip_whitespace();
                        r.consume("filename=").expect("Expected `filename=`");
                        file.name = Some(r.read_string().expect("Invalid filename"));
                        r.consume("\r\n").unwrap();
                    }
                } else {// ignore the line
                    r.skip_while(|b| b != &b'\r');
                    r.consume("\r\n").unwrap();
                }
            }
        }
        r.consume("--").unwrap();
        break attachments
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


#[cfg(test)] fn test_is_end_boundary() {
    assert_eq!(is_end_boundary(b"--abcdef", "abcdef"), false);
    assert_eq!(is_end_boundary(b"--abcdef--", "abcdef"), true);
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
