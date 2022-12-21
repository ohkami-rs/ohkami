use crate::response::Response;

/// ohkami's primary Result type.\
/// Here, `Response` implements `From`
/// 
/// - std::io::Error
/// - serde_json::Error
/// - std::str::Utf8Error
/// - sqlx::Error
/// 
/// by default, so `std::result::Result`s holding them in `Err` can ne converted to this `Result` by `?`.
pub type Result<T> = std::result::Result<T, Response>;

pub trait ElseResponse {
    type Expect;
    /// Convert
    /// - `Option<T>` to `Result<T>`
    /// - `bool` to `Result<()>`
    /// 
    /// `Err` value is calculated **only if** called by `None` or `false`.
    /// This is quite useful in error handling like
    /// 
    /// ```no_run
    /// let handler = self.handler.as_ref()._else(|| Response::NotFound(None))?;
    /// 
    /// (count < 10)
    ///     ._else(|| Response::BadRequest("`count` must be less than 10"))?;
    ///     // or
    ///     ._else(|| Response::BadRequest(None))?;
    /// ```
    fn _else<F: FnOnce() -> Response>(self, err: F) -> Result<Self::Expect>;
}
impl<T> ElseResponse for Option<T> {
    type Expect = T;
    fn _else<F: FnOnce() -> Response>(self, err: F) -> Result<Self::Expect> {
        self.ok_or_else(err)
    }
}
impl ElseResponse for bool {
    type Expect = ();
    fn _else<F: FnOnce() -> Response>(self, err: F) -> Result<Self::Expect> {
        self.then_some(()).ok_or_else(err)
    }
}

pub trait ElseResponseWithErr<E> {
    type Expect;
    /// Convert `std::result::Result<T, E>` into `Result<T>`.
    /// `Err` value is calculated **only if** called by `None` or `false`.
    /// 
    /// ```no_run
    /// let user = ctx.body::<User>()?;
    /// 
    /// // or, you can add an error context message:
    /// let user = ctx.body::<User>()
    ///     ._else(|e| e.error_context("failed to get user data"))?;
    /// 
    /// // or discard original error:
    /// let user = ctx.body::<User>()
    ///     ._else(|_| Response::InternalServerError("can't get user"))?;
    ///     // or
    ///     ._else(|_| Response::InternalServerError(None))?;
    /// ```
    fn _else<F: FnOnce(E) -> Response>(self, err: F) -> Result<Self::Expect>;
}
impl<T, E> ElseResponseWithErr<E> for std::result::Result<T, E> {
    type Expect = T;
    fn _else<F: FnOnce(E) -> Response>(self, err: F) -> Result<Self::Expect> {
        self.map_err(err)
    }
}


impl From<std::io::Error> for Response {
    fn from(value: std::io::Error) -> Self {
        Self::InternalServerError(value.to_string() + ": caused by I/O")
    }
}
impl From<serde_json::Error> for Response {
    fn from(value: serde_json::Error) -> Self {
        Self::InternalServerError(value.to_string() + ": caused by json handling :: " + {
            if value.is_data() {
                "invalid json data"
            } else if value.is_eof() {
                "unexpected end of line"
            } else if value.is_io() {
                "about io"
            } else { // value.is_syntax()
                "wrong json syntax"
            }
        })
    }
}
impl From<std::str::Utf8Error> for Response {
    fn from(value: std::str::Utf8Error) -> Self {
        Self::InternalServerError(value.to_string() + ": caused by UTF-8 handling")
    }
}

#[cfg(any(feature = "postgres", feature = "mysql"))]
impl From<sqlx::Error> for Response {
    fn from(value: sqlx::Error) -> Self {
        Self::InternalServerError(value.to_string() + ": caused by DB handling")
    }
}