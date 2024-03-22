mod de;
mod file;
mod parse;

#[cfg(test)]
mod _test;


pub use file::File;

#[inline(always)]
pub fn from_bytes<'de, D: serde::Deserialize<'de>>(input: &'de [u8]) -> Result<D, Error> {
    let mut d = de::MultipartDesrializer::new(input)?;
    D::deserialize(&mut d)
}


use std::borrow::Cow;
#[derive(Debug)]
pub struct Error(Cow<'static, str>);
const _: () = {
    impl std::fmt::Display for Error {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str(&self.0)
        }
    }
    impl std::error::Error for Error {}

    impl serde::ser::Error for Error {
        fn custom<T>(msg:T) -> Self where T:std::fmt::Display {
            Self(Cow::Owned(msg.to_string()))
        }
    }
    impl serde::de::Error for Error {
        fn custom<T>(msg:T) -> Self where T:std::fmt::Display {
            Self(Cow::Owned(msg.to_string()))
        }
    }
};
#[allow(non_snake_case)]
impl Error {
    const fn ExpectedValidBoundary() -> Self {
        Self(Cow::Borrowed("Expected valid multipart boundary"))
    }
    const fn ExpectedBoundary() -> Self {
        Self(Cow::Borrowed("Expected multipart boundary"))
    }
    const fn MissingCRLF() -> Self {
        Self(Cow::Borrowed("Missing CRLF in multipart"))
    }
    const fn ExpectedAttachment() -> Self {
        Self(Cow::Borrowed("Expected `attachment`"))
    }
    const fn ExpectedFile() -> Self {
        Self(Cow::Borrowed("Expected file but found non-file field in multipart"))
    }
    const fn ExpectedFilename() -> Self {
        Self(Cow::Borrowed("Expected `filename=\"...\"`"))
    }
    const fn ExpectedValidHeader() -> Self {
        Self(Cow::Borrowed("Expected `Content-Type` or `Content-Disposition` header in multipart section"))
    }
    const fn ExpectedFormdataAndName() -> Self {
        Self(Cow::Borrowed("Expected `form-data; name=\"...\"` after `Content-Disposition: `"))
    }
    const fn InvalidFilename() -> Self {
        Self(Cow::Borrowed("Invalid filename"))
    }
}
