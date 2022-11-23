use crate::response::Response;


pub type Result<T> = std::result::Result<T, Response>;

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
impl From<sqlx::Error> for Response {
    fn from(value: sqlx::Error) -> Self {
        Self::InternalServerError(value.to_string() + ": caused by DB handling")
    }
}