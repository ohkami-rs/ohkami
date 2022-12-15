use std::ops::Range;
use serde::Deserialize;
use crate::{
    result::{Result, ElseResponse},
    utils::{map::RangeMap, buffer::Buffer},
    components::json::JSON,
    response::Response,
};

#[cfg(feature = "sqlx")]
use async_std::sync::Arc;
#[cfg(feature = "postgres")]
use sqlx::PgPool as ConnectionPool;
#[cfg(feature = "mysql")]
use sqlx::MySqlPool as ConnectionPool;


pub struct Context {
    buffer: Buffer,

    pub(crate) body:        Option<JSON>,
    pub(crate) param_range: Option<Range<usize>>,
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
        Some(self.buffer.read_str(self.param_range?))
    }
    pub fn query(&self, key: &str) -> Option<&str> {
        self.query_range?.read_match_part_of_buffer(key, &self.buffer)
    }

    #[cfg(feature = "sqlx")]
    pub fn pool(&self) -> &ConnectionPool {
        &*self.pool
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
