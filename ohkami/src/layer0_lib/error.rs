use std::borrow::Cow;


pub enum Error {
    IO(Cow<'static, str>),
    Parse(Cow<'static, str>),
}

impl Error {
    pub(crate) fn as_str(&self) -> &str {
        match self {
            Self::IO(cow) => &cow,
            Self::Parse(cow) => &cow,
        }
    }

    pub fn to_string(&self) -> String {
        self.as_str().to_owned()
    }
}
