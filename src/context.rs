use serde::Deserialize;
use sqlx::PgPool;
use crate::{
    components::json::JSON,
    response::Response, result::Result,
};


pub struct Context<'ctx> {
    pub(crate) pool: Option<&'ctx PgPool>,
    pub path_param:  Option<u32>,  // Option<&'ctx str>,
    pub(crate) body: Option<JSON>,
}

impl<'d, 'ctx> Context<'ctx> {
    pub fn request_body<D: Deserialize<'d>>(&'d self) -> Result<D> {
        let json = self.body.as_ref()
            .ok_or_else(|| Response::BadRequest("expected request body"))?;
        let json_struct = json.to_struct()?;
        Ok(json_struct)
    }
    pub fn pool(&self) -> &PgPool {
        self.pool.unwrap()
    }
}