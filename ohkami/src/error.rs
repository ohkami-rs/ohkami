use std::fmt::Display;

use crate::response::ErrResponse;

pub enum Error {
    IO(String),
    ConstValue(String),
    Validation(String),
    Others(String),
}
impl Error {
    pub(crate) fn in_const_value(const_name: &'static str) -> Self {
        Self::ConstValue(format!(
            "Error cause: mistake in setting an internal const value `{const_name}`. If you see this error, please report to GitHub issue."
        ))
    }
}
const _: (/* Error impls */) = {
    impl Display for Error {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            writeln!(f, "{}", match self {
                Self::IO(msg) => msg,
                Self::ConstValue(msg) => msg,
                Self::Validation(msg) => msg,
                Self::Others(msg) => msg,
            })
        }
    }
    impl From<std::io::Error> for Error {
        fn from(value: std::io::Error) -> Self {
            Self::IO(format!("{value}"))
        }
    }
};

pub trait ElseResponse {
    type Expect;
    fn _else<F: FnOnce() -> ErrResponse>(self, error_response: F) -> Result<Self::Expect, ErrResponse>;
}
impl<T> ElseResponse for Option<T> {
    type Expect = T;
    #[inline] fn _else<F: FnOnce() -> ErrResponse>(self, error_response: F) -> Result<Self::Expect, ErrResponse> {
        self.ok_or_else(error_response)
    }
}
impl ElseResponse for bool {
    type Expect = ();
    #[inline] fn _else<F: FnOnce() -> ErrResponse>(self, error_response: F) -> Result<Self::Expect, ErrResponse> {
        self.then(|| ()).ok_or_else(error_response)
    }
}

pub trait ElseResponseWithErr<E> {
    type Expect;
    fn _else<F: FnOnce(E) -> ErrResponse>(self, error_response: F) -> Result<Self::Expect, ErrResponse>;
}
impl<T, E> ElseResponseWithErr<E> for std::result::Result<T, E> {
    type Expect = T;
    #[inline] fn _else<F: FnOnce(E) -> ErrResponse>(self, error_response: F) -> Result<Self::Expect, ErrResponse> {
        self.map_err(error_response)
    }
}
