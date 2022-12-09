use std::collections::HashMap;
use serde::Deserialize;
use crate::{
    components::json::JSON,
    response::Response, result::Result,
};

#[cfg(feature = "sqlx")]
use async_std::sync::Arc;
#[cfg(feature = "postgres")]
use sqlx::PgPool as ConnectionPool;
#[cfg(feature = "mysql")]
use sqlx::MySqlPool as ConnectionPool;


pub struct Context {
    pub(crate) param: Option<u32>,  // Option<&'ctx str>,
    pub(crate) body:  Option<JSON>,
    pub(crate) query: Option<HashMap<*const u8, String>>,//[String; HASH_TABLE_SIZE]>,

    #[cfg(feature = "sqlx")]
    pub(crate) pool:  Arc<ConnectionPool>,
}

impl<'d, 'q> Context {
    pub fn request_body<D: Deserialize<'d>>(&'d self) -> Result<D> {
        let json = self.body.as_ref()
            .ok_or_else(|| Response::BadRequest("expected request body"))?;
        let json_struct = json.to_struct()?;
        Ok(json_struct)
    }
    pub fn param(&self) -> Option<u32> {
        self.param
    }
    pub fn query(&self, key: &str) -> Option<&str> {
        self.query.as_ref()?
            .get(&key.as_ptr()).map(|s| &**s)
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
        let string2 = String::from("abcdef");
        assert_eq!(string, "abcdef");
    }

}
