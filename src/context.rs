use serde::Deserialize;
use crate::{
    components::json::JSON,
    response::Response, result::Result,
};

#[cfg(not(any(feature = "postgres", feature = "mysql")))]
pub struct Context {
    pub path_param:  Option<u32>,  // Option<&'ctx str>,
    pub(crate) body: Option<JSON>,
}
#[cfg(not(any(feature = "postgres", feature = "mysql")))]
impl<'d> Context {
    pub fn request_body<D: Deserialize<'d>>(&'d self) -> Result<D> {
        let json = self.body.as_ref()
            .ok_or_else(|| Response::BadRequest("expected request body"))?;
        let json_struct = json.to_struct()?;
        Ok(json_struct)
    }
}

#[cfg(feature = "postgres")]
pub struct Context<'ctx> {
    pub(crate) pool: Option<&'ctx sqlx::PgPool>,
    pub path_param:  Option<u32>,  // Option<&'ctx str>,
    pub(crate) body: Option<JSON>,
}
#[cfg(feature = "postgres")]
impl<'d, 'ctx> Context<'ctx> {
    pub fn request_body<D: Deserialize<'d>>(&'d self) -> Result<D> {
        let json = self.body.as_ref()
            .ok_or_else(|| Response::BadRequest("expected request body"))?;
        let json_struct = json.to_struct()?;
        Ok(json_struct)
    }
    pub fn pool(&self) -> &sqlx::PgPool {
        self.pool.unwrap()
    }
}

#[cfg(feature = "mysql")]
pub struct Context<'ctx> {
    pub(crate) pool: Option<&'ctx sqlx::MySqlPool>,
    pub path_param:  Option<u32>,  // Option<&'ctx str>,
    pub(crate) body: Option<JSON>,
}
#[cfg(feature = "mysql")]
impl<'d, 'ctx> Context<'ctx> {
    pub fn request_body<D: Deserialize<'d>>(&'d self) -> Result<D> {
        let json = self.body.as_ref()
            .ok_or_else(|| Response::BadRequest("expected request body"))?;
        let json_struct = json.to_struct()?;
        Ok(json_struct)
    }
    pub fn pool(&self) -> &sqlx::MySqlPool {
        self.pool.unwrap()
    }
}