use std::fmt::Debug;

use serde::Deserialize;
use crate::{
    result::{Result, ElseResponse},
    utils::{map::RangeMap, buffer::{Buffer, BufRange}},
    components::json::Json,
    response::Response,
};

#[cfg(feature = "sqlx")]
use async_std::sync::Arc;
#[cfg(feature = "postgres")]
use sqlx::PgPool as ConnectionPool;
#[cfg(feature = "mysql")]
use sqlx::MySqlPool as ConnectionPool;


pub struct Context {
    pub(crate) buffer: Buffer,

    pub(crate) body:        Option<Json>,
    pub(crate) param_range: Option<BufRange>,
    pub(crate) query_range: Option<RangeMap>,

    #[cfg(feature = "sqlx")]
    pub(crate) pool:  Arc<ConnectionPool>,
}

impl<'d> Context {
    pub fn body<D: Deserialize<'d>>(&'d self) -> Result<D> {
        let json = self.body.as_ref()
            ._else(|| Response::BadRequest("expected request body"))?;
        let json_struct = json.to_struct()?;
        Ok(json_struct)
    }
    pub fn param(&self/*, key: &str*/) -> Option<&str> {
        Some(self.buffer.read_str(self.param_range.as_ref()?))
    }
    pub fn query(&self, key: &str) -> Option<&str> {
        self.query_range.as_ref()?.read_match_part_of_buffer(key, &self.buffer)
    }

    #[cfg(feature = "sqlx")]
    pub fn pool(&self) -> &ConnectionPool {
        &*self.pool
    }
}

impl Debug for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "
param: {:?} (range: {:?}),
query: {:?} (range: {:?}),
body: {:?}",
            self.param(),
            self.param_range,
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
