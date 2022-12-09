use serde::Deserialize;
use crate::{
    components::{json::JSON, consts::HASH_TABLE_SIZE},
    response::Response, result::Result, utils::hash::hash,
};

#[cfg(feature = "sqlx")]
use async_std::sync::Arc;
#[cfg(feature = "postgres")]
use sqlx::PgPool as ConnectionPool;
#[cfg(feature = "mysql")]
use sqlx::MySqlPool as ConnectionPool;


pub struct Context {
    pub(crate) param: Option<u32>,
    pub(crate) body:  Option<JSON>,
    pub(crate) query: [Option<String>; HASH_TABLE_SIZE],

    #[cfg(feature = "sqlx")]
    pub(crate) pool:  Arc<ConnectionPool>,
}

impl<'d> Context {
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
        self.query[hash(key)].as_ref().map(|value| &**value)
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
