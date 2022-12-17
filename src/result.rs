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
    /// ```no_run
    /// async fn handler(ctx: Context) -> Result<Response> {
    ///     let sleep_time = ctx.param()
    ///         ._else(|| Response::BadRequest("Expected sleeping duration as path parameter."))?
    ///         .parse::<u64>()
    ///         ._else(|_| Response::BadRequest("`time` must be a zero or positive integer."))?;
    ///     (sleep_time < 30)
    ///         ._else(|| Response::BadRequest("Sorry, please request a sleeping duration (sec) less than 30."))?;
    ///     
    ///     std::thread::sleep(
    ///         std::time::Duration::from_secs(sleep_time)
    ///     );
    ///     
    ///     Response::OK("Hello, I'm sleepy...")
    /// }
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
    /// Convert `std::result::Result<T, E>` to `Result<T>`. `Err` value is calculated **only if** original `std::result::Result` is `Err`, and this calculation can use that `Err` like
    /// ```no_run
    /// let sleep_time = ctx.param().unwrap()
    ///         .parse::<u64>()
    ///         ._else(|err| Response::BadRequest(err.to_string()))?;
    /// ```
    /// If you discard one, use `_` :
    /// ```no_run
    /// let sleep_time = ctx.param().unwrap()
    ///         .parse::<u64>()
    ///         ._else(|_| Response::BadRequest("`time` must be a zero or positive integer."))?;
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