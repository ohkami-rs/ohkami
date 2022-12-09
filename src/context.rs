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


pub struct Context<'q> {
    pub(crate) param: Option<u32>,  // Option<&'ctx str>,
    pub(crate) query: Option<HashMap<&'q str, &'q str>>,
    pub(crate) body:  Option<JSON>,

    #[cfg(feature = "sqlx")]
    pub(crate) pool:  Arc<ConnectionPool>,
}

impl<'d, 'q> Context<'q> {
    pub fn request_body<D: Deserialize<'d>>(&'d self) -> Result<D> {
        let json = self.body.as_ref()
            .ok_or_else(|| Response::BadRequest("expected request body"))?;
        let json_struct = json.to_struct()?;
        Ok(json_struct)
    }
    pub fn param(&self) -> Option<u32> {
        self.param
    }
    pub fn query(&self, key: &str) -> Option<&&str> {
        self.query.as_ref()?.get(key)
    }

    #[cfg(feature = "sqlx")]
    pub fn pool(&self) -> &ConnectionPool {
        &*self.pool
    }
}
