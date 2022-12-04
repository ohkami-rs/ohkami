use crate::response::Response;


pub type Result<T> = std::result::Result<T, Response>;

pub trait ElseResponse {
    type Expect;
    fn else_response<F: FnOnce() -> Response>(self, err: F) -> Result<Self::Expect>;
}
impl<T> ElseResponse for Option<T> {
    type Expect = T;
    fn else_response<F: FnOnce() -> Response>(self, err: F) -> Result<Self::Expect> {
        self.ok_or_else(err)
    }
}
impl ElseResponse for bool {
    type Expect = ();
    fn else_response<F: FnOnce() -> Response>(self, err: F) -> Result<Self::Expect> {
        self.then_some(()).ok_or_else(err)
    }
}

pub trait ElseResponseWithErr<E> {
    type Expect;
    fn else_response<F: FnOnce(E) -> Response>(self, err: F) -> Result<Self::Expect>;
}
impl<T, E> ElseResponseWithErr<E> for std::result::Result<T, E> {
    type Expect = T;
    fn else_response<F: FnOnce(E) -> Response>(self, err: F) -> Result<Self::Expect> {
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
        if let Some(db_error) = value.as_database_error() {
            Self::InternalServerError(db_error.message().to_string() + ": caused by DB handling")
        } else {
            Self::SetUpError(&vec![value.to_string() + "caused by DB setup"])
        }
    }
}