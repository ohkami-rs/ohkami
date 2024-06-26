use std::{borrow::Cow, str::Utf8Error};


#[inline(always)] pub fn percent_decode_utf8(input: &[u8]) -> Result<Cow<'_, str>, Utf8Error> {
    ::percent_encoding::percent_decode(input).decode_utf8()
}
#[inline(always)] pub fn percent_decode(input: &[u8]) -> Cow<'_, [u8]> {
    ::percent_encoding::percent_decode(input).into()
}

#[inline(always)] pub fn percent_encode(input: &str) -> Cow<'_, str> {
    ::percent_encoding::percent_encode(input.as_bytes(), ::percent_encoding::NON_ALPHANUMERIC).into()
}
