use crate::response::err::ErrResponse;

pub enum Error {
    IO(String),
    ConstValue(String),
    Validation(String),
}
impl Error {
    pub(crate) fn in_const_value(const_name: &'static str) -> Self {
        Self::ConstValue(format!(
            "Error cause: mistake in setting an internal const value `{const_name}`. If you see this error, please report to GitHub issue."
        ))
    }
}

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
