use std::fmt::Debug;
use serde::Deserialize;
use crate::{
    result::{Result, ElseResponse},
    utils::{range::RangeMap, buffer::Buffer},
    components::json::JSON,
    response::Response, prelude::ElseResponseWithErr
};

#[cfg(feature = "sqlx")]
use async_std::sync::Arc;
#[cfg(feature = "postgres")]
use sqlx::PgPool as ConnectionPool;
#[cfg(feature = "mysql")]
use sqlx::MySqlPool as ConnectionPool;

/// Type of context of a HTTP request.
pub struct Context {
    pub(crate) buffer:      Buffer,

    pub(crate) body:        Option<JSON>,
    pub(crate) query_range: Option<RangeMap>,

    #[cfg(feature = "sqlx")]
    pub(crate) pool:  Arc<ConnectionPool>,
}

impl<'d> Context {
    /// Try deserialize the reuqest body into Rust struct that implements `serde::Deserialize`, and return `Result</* that struct */>`. If request doesn't have body, this returns `Err(Response)` of "Bad Request".
    pub fn body<D: Deserialize<'d>>(&'d self) -> Result<D> {
        let json = self.body.as_ref()
            ._else(|| Response::BadRequest("expected request body"))?;
        let json_struct = json.to_struct()?;
        Ok(json_struct)
    }
    /// Return `Result< &str | u64 | i64 | usize >` that holds query parameter whose key matches the argument (`Err`: if param string can't be parsed).
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

    /// Return a reference of `PgPool` (if feature = "postgres") or `MySqlPool` (if feature = "mysql").
    #[cfg(feature = "sqlx")]
    pub fn pool(&self) -> &ConnectionPool {
        &*self.pool
    }
}

pub trait Query<'q> {fn parse(q: &'q str) -> Result<Self> where Self: Sized;}
impl<'q> Query<'q> for &'q str {fn parse(q: &'q str) -> Result<Self> {Ok(q)}}
impl<'q> Query<'q> for u64 {fn parse(q: &'q str) -> Result<Self> {q.parse()._else(|_| Response::BadRequest("format of query parameter is wrong"))}}
impl<'q> Query<'q> for i64 {fn parse(q: &'q str) -> Result<Self> {q.parse()._else(|_| Response::BadRequest("format of query parameter is wrong"))}}
impl<'q> Query<'q> for usize {fn parse(q: &'q str) -> Result<Self> {q.parse()._else(|_| Response::BadRequest("format of query parameter is wrong"))}}

impl Debug for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "
query: {:?} (range: {:?}),
body: {:?}",
            self.query_range.as_ref().map(|map| map.debug_fmt_with(&self.buffer)),
            self.query_range,
            self.body,
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
