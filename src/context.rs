use std::fmt::Debug;
use serde::{Deserialize, Serialize};
use crate::{
    result::{Result, ElseResponse, ElseResponseWithErr},
    utils::{range::RangeMap, buffer::Buffer},
    components::{json::JSON, status::Status, headers::{HeaderMap, HeaderKey}},
    response::{Response, message::ErrorMessage, body::{Body, ResponseBody}},
};

#[cfg(feature = "sqlx")]
use async_std::sync::Arc;
#[cfg(feature = "postgres")]
use sqlx::PgPool as ConnectionPool;
#[cfg(feature = "mysql")]
use sqlx::MySqlPool as ConnectionPool;

/// Type of context of a HTTP request.
pub struct Context {
    pub req: RequestContext,
    pub(crate) additional_headers: String,

    #[cfg(feature = "sqlx")]
    pub(crate) pool:  Arc<ConnectionPool>,
}
pub struct RequestContext {
    pub(crate) buffer:      Buffer,

    // pub(crate) body:        Option<JSON>,
    pub(crate) query_range: Option<RangeMap>,
    pub(crate) headers:     HeaderMap,
}

impl Context {
    #[allow(non_snake_case)]
    pub fn OK<B: ResponseBody>(&self, body: B) -> Result<Response> {
        Ok(Response {
            additional_headers: self.additional_headers.to_owned(),
            status: Status::OK,
            body: body.as_body()?
        })
    }
    #[allow(non_snake_case)]
    pub fn Created<T: Serialize + for <'d> Deserialize<'d>>(&self, created: JSON<T>) -> Result<Response> {
        Ok(Response {
            additional_headers: self.additional_headers.to_owned(),
            status: Status::Created,
            body: Some(Body::application_json(created.ser()?))
        })
    }
    #[allow(non_snake_case)]
    pub fn NoContent() -> Result<Response> {
        Ok(Response {
            additional_headers: String::new(),
            status: Status::Created,
            body: None
        })
    }

    /// Generate `Response` value that represents a HTTP response of `404 Not Found`.
    /// `String`, `&str` or `Option<String>` can be argument of this.
    #[allow(non_snake_case)]
    pub fn NotFound<Msg: ErrorMessage>(&self, msg: Msg) -> Response {
        Response {
            additional_headers: self.additional_headers.to_owned(),
            status: Status::NotFound,
            body:   msg.as_message()
        }
    }
    /// Generate `Response` value that represents a HTTP response of `400 Bad Request`.
    /// `String`, `&str` or `Option<String>` can be argument of this.
    #[allow(non_snake_case)]
    pub fn BadRequest<Msg: ErrorMessage>(&self, msg: Msg) -> Response {
        Response {
            additional_headers: self.additional_headers.to_owned(),
            status:             Status::BadRequest,
            body:               msg.as_message(),
        }
    }
    /// Generate `Response` value that represents a HTTP response of `500 Internal Server Error`.
    /// `String`, `&str` or `Option<String>` can be argument of this.
    #[allow(non_snake_case)]
    pub fn InternalServerError<Msg: ErrorMessage>(&self, msg: Msg) -> Response {
        Response {
            additional_headers: self.additional_headers.to_owned(),
            status:             Status::InternalServerError,
            body:               msg.as_message(),
        }
    }
    /// Generate `Response` value that represents a HTTP response of `501 Not Implemented`.
    /// `String`, `&str` or `Option<String>` can be argument of this.
    #[allow(non_snake_case)]
    pub fn NotImplemented<Msg: ErrorMessage>(&self, msg: Msg) -> Response {
        Response {
            additional_headers: self.additional_headers.to_owned(),
            status:             Status::NotImplemented,
            body:               msg.as_message(),
        }
    }
    /// Generate `Response` value that represents a HTTP response of `403 Forbidden`.
    /// `String`, `&str` or `Option<String>` can be argument of this.
    #[allow(non_snake_case)]
    pub fn Forbidden<Msg: ErrorMessage>(&self, msg: Msg) -> Response {
        Response {
            additional_headers: self.additional_headers.to_owned(),
            status:             Status::Forbidden,
            body:               msg.as_message(),
        }
    }
    /// Generate `Response` value that represents a HTTP response of `401 Unauthorized`.
    /// `String`, `&str` or `Option<String>` can be argument of this.
    #[allow(non_snake_case)]
    pub fn Unauthorized<Msg: ErrorMessage>(&self, msg: Msg) -> Response {
        Response {
            additional_headers: self.additional_headers.to_owned(),
            status:             Status::Unauthorized,
            body:               msg.as_message(),
        }
    }

    pub fn header<Key: HeaderKey>(&mut self, key: Key, value: &'static str) {
        self.additional_headers += key.as_key_str();
        self.additional_headers += ": ";
        self.additional_headers += value;
        self.additional_headers += "\n"
    }

    /// Return a reference of `PgPool` (if feature = "postgres") or `MySqlPool` (if feature = "mysql").
    #[cfg(feature = "sqlx")]
    pub fn pool(&self) -> &ConnectionPool {
        &*self.pool
    }
}
impl<'d> RequestContext {
    /*
        /// Try deserialize the reuqest body into Rust struct that implements `serde::Deserialize`, and return `Result</* that struct */>`. If request doesn't have body, this returns `Err(Response)` of "Bad Request".
        pub fn body<D: Deserialize<'d>>(&'d self) -> Result<D> {
            let json = self.body.as_ref()
                ._else(|| Response::BadRequest("expected request body"))?;
            let json_struct = json.to_struct()?;
            Ok(json_struct)
        }
    */
    
    /// Return `Result< &str | u8 | u64 | i32 | i64 | usize >` that holds query parameter whose key matches the argument (`Err`: if param string can't be parsed).
    /// ```no_run
    /// let count = ctx.query("count")?;
    /// ```
    pub fn query<'ctx, Q: Query<'ctx>>(&'ctx self, key: &str) -> Result<Q> {
        Query::parse(
            self.query_range
                .as_ref()
                ._else(|| Response::BadRequest(format!("expected query param `{key}`")))?
                .read_match_part_of_buffer(key, &self.buffer)
                ._else(|| Response::BadRequest(format!("expected query param `{key}`")))?
        )
    }

    /// key: &'static str | Header
    pub fn header<K: HeaderKey>(&self, key: K) -> Option<&str> {
        self.headers.get(key, &self.buffer)
    }
}

pub trait Query<'q> {fn parse(q: &'q str) -> Result<Self> where Self: Sized;}
impl<'q> Query<'q> for &'q str {fn parse(q: &'q str) -> Result<Self> {Ok(q)}}
impl<'q> Query<'q> for u8 {fn parse(q: &'q str) -> Result<Self> {q.parse()._else(|_| Response::BadRequest("format of query parameter is wrong"))}}
impl<'q> Query<'q> for u64 {fn parse(q: &'q str) -> Result<Self> {q.parse()._else(|_| Response::BadRequest("format of query parameter is wrong"))}}
impl<'q> Query<'q> for i64 {fn parse(q: &'q str) -> Result<Self> {q.parse()._else(|_| Response::BadRequest("format of query parameter is wrong"))}}
impl<'q> Query<'q> for i32 {fn parse(q: &'q str) -> Result<Self> {q.parse()._else(|_| Response::BadRequest("format of query parameter is wrong"))}}
impl<'q> Query<'q> for usize {fn parse(q: &'q str) -> Result<Self> {q.parse()._else(|_| Response::BadRequest("format of query parameter is wrong"))}}

impl Debug for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "query: {:?} (range: {:?}), {}",
            self.req.query_range.as_ref().map(|map| map.debug_fmt_with(&self.req.buffer)),
            self.req.query_range,
            self.req.headers.debug_fmt_with(&self.req.buffer),
        )
    }
}


#[cfg(test)]
mod test {
    #[test]
    fn how_str_as_ptr_works() {
        assert_eq!("abc".as_ptr(), "abc".as_ptr());

        let abc = "abc";
        let abc2 = "abc";
        assert_eq!(abc.as_ptr(), abc2.as_ptr());

        let string = String::from("abcdef");
        // let string2 = String::from("abcdef");
        assert_eq!(string, "abcdef");
    }

}
