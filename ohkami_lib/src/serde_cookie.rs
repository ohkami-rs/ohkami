mod de;

#[cfg(test)]
mod _test;

#[inline(always)]
pub fn from_str<'de, D: serde::Deserialize<'de>>(input: &'de str) -> Result<D, Error> {
    let mut d = de::CookieDeserializer::new(input);
    let t = D::deserialize(&mut d)?;
    if d.remaining().is_empty() {
        Ok(t)
    } else {
        Err((||serde::de::Error::custom(format!("Unexpected trailing charactors: {}", d.remaining().escape_ascii())))())
    }
}

#[derive(Debug)]
pub struct Error(String);
const _: () = {
    impl std::fmt::Display for Error {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str(&self.0)
        }
    }
    impl std::error::Error for Error {}

    impl serde::ser::Error for Error {
        fn custom<T>(msg:T) -> Self where T:std::fmt::Display {
            Self(msg.to_string())
        }
    }
    impl serde::de::Error for Error {
        fn custom<T>(msg:T) -> Self where T:std::fmt::Display {
            Self(msg.to_string())
        }
    }
};
