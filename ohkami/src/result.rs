use crate::response::{Response, ResponseWriter};


pub type Result<T> = std::result::Result<T, Response>;

pub trait ElseResponse {
    type Expect;
    fn _else<F: FnOnce() -> ()>(self, error_response: F) -> std::result::Result<Self::Expect, ()>;
}
impl<T> ElseResponse for Option<T> {
    type Expect = T;
    #[inline] fn _else<F: FnOnce() -> ()>(self, error_response: F) -> std::result::Result<Self::Expect, ()> {
        self.ok_or_else(error_response)
    }
}
impl ElseResponse for bool {
    type Expect = ();
    #[inline] fn _else<F: FnOnce() -> ()>(self, error_response: F) -> std::result::Result<Self::Expect, ()> {
        self.then(|| ()).ok_or_else(error_response)
    }
}

pub trait ElseResponseWithErr<E> {
    type Expect;
    fn _else<F: FnOnce(E) -> ()>(self, error_response: F) -> std::result::Result<Self::Expect, ()>;
}
impl<T, E> ElseResponseWithErr<E> for std::result::Result<T, E> {
    type Expect = T;
    #[inline] fn _else<F: FnOnce(E) -> ()>(self, error_response: F) -> std::result::Result<Self::Expect, ()> {
        self.map_err(error_response)
    }
}


// impl From<std::io::Error> for Response {
//     fn from(value: std::io::Error) -> Self {
//         Self::InternalServerError(value.to_string() + ": caused by I/O")
//     }
// }
// impl From<serde_json::Error> for Response {
//     fn from(value: serde_json::Error) -> Self {
//         Self::InternalServerError(value.to_string() + ": caused by json handling :: " + {
//             if value.is_data() {
//                 "invalid json data"
//             } else if value.is_eof() {
//                 "unexpected end of line"
//             } else if value.is_io() {
//                 "about io"
//             } else { // value.is_syntax()
//                 "wrong json syntax"
//             }
//         })
//     }
// }
// impl From<std::str::Utf8Error> for Response {
//     fn from(value: std::str::Utf8Error) -> Self {
//         Self::InternalServerError(value.to_string() + ": caused by UTF-8 handling")
//     }
// }
// 
// #[cfg(any(feature = "postgres", feature = "mysql"))]
// impl From<sqlx::Error> for Response {
//     fn from(value: sqlx::Error) -> Self {
//         Self::InternalServerError(value.to_string() + ": caused by DB handling")
//     }
// }
