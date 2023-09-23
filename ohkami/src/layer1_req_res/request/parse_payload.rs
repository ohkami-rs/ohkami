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
    File(File),
    Files(Vec<File>),
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
/// - `None` if `buf` contains only `--boundary--`
pub fn parse_formpart(buf: &[u8], boundary: &str) -> Option<FormPart> {
    let mut r = Reader::new(buf);

    r.consume(&f!("--{boundary}")).expect("Expected valid form-data boundary");
    match r.consume_oneof(["\r\n", "--"]) {
        Ok(0)  => (/* continue parsing */),
        Ok(1)  => return None/* Here the `multipart/form-data` finished */,
        Ok(_)  => __unreachable__(),
        Err(e) => panic!("{e}")
    }

    let mut this = FormPart {
        name:    String::new(),
        content: FormContent::Content(Content {
            mime_type: format!("text/plain"),
            content:   Vec::new(),
        })
    };

    while let Ok(header) = r.read_kebab() {
        if header.eq_ignore_ascii_case("Content-Type") {
            r.consume(":").unwrap(); r.skip_whitespace();

            __TODO__
        } else
        if header.eq_ignore_ascii_case("Content-Disposition") {
            r.consume(":").unwrap(); r.skip_whitespace();
            match r.consume_oneof(["form-data", "attachment"]) {
                Ok(0) => {
                    r.consume(";").unwrap(); r.skip_whitespace();
                    r.consume("name=").expect("Expected `name` in form part");
                    this.name = r.read_string().unwrap();
                }
                Ok(1) => if r.consume(";").is_ok() {
                    __TODO__
                }
                Ok(_)  => __unreachable__(), Err(e) => panic!("{e}")
            }
            r.consume("\r\n").unwrap();
        }
    }

// 
    // while r.(b"\r\n").is_some() {
    //     match r.read_split_left(b':').unwrap() {
    //         b"Content-Type" | b"content-type" => {r.read(b' ');
    //             if r.read_(b"multipart/mixed").is_some() {
    //                 r.read_(b",").unwrap(); r.read(b' ');
    //                 r.read_(b"boundary=").expect("Expected `boundary=`");
    //                 let attachent_boundary = r.read_before(b'\r').unwrap();
    //                 r.read_(b"\r\n\r\n").unwrap();
// 
    //                 let mut attachments = Vec::new();
    //                 while let Some(file) = parse_attachment(&mut r, attachent_boundary) {
    //                     attachments.push(file)
    //                 }
    //                 form_part.content = FormContent::Files(attachments)
// 
    //             } else {
// 
    //             }
    //         }
    //         b"Content-Disposition" | b"content-disposition" => {r.read(b' ');
// 
    //         }
    //         _ => panic!("Expected `Content-Type` or `Content-Disposition`")
    //     }
    // }
    
    Some(this)
}

// fn parse_attachment(r: &mut Reader<&[u8]>, boundary: &[u8]) -> Option<File> {
//     r.read_(boundary).expect("Expected valid form-data boundary");
//     r.read_(b"\r\n")?; // return None if this was `--`
// 
//     let mut file = File { name: None, mime_type: format!("text/plain"), content: vec![] };
// 
//     while r.read_(b"\r\n").is_none() {
//         match r.read_split_left(b':').unwrap() {
//             b"Content-Type" | b"content-type" => {r.read(b' ');
//                 let mime_type = r.read_before(b'\r').unwrap().to_vec();
//                 file.mime_type = String::from_utf8(mime_type).expect("mime type is invalid UTF-8");
//                 r.read_(b"\r\n").unwrap()
//             }
//             b"Content-Disposition" | b"content-disposition" => {r.read(b' ');
//                 r.read_(b"attachment").expect("Expected `attachment`");
//                 if r.read(b';').is_some() {r.read(b' ');
//                     r.read_(b"filename=\"").expect("Expected `filename`");
//                     let name = r.read_split_left(b'"').expect("Found '\"' not closing");
//                     file.name.replace(percent_decode(name).decode_utf8().expect("filename is invalid UTF-8").to_string());
//                     r.read_(b"\r\n").unwrap()
//                 }
//             }
//             _ => ()
//         }
//     }
// 
//     file.content = r.read_before_(boundary).unwrap().to_vec();
//     Some(file)
// }




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
