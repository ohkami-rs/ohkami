use serde::Deserialize;
use sqlx::PgPool;
use crate::{
    components::json::JSON,
    response::Response, result::Result,
};


pub struct Context<'pool, 'param> {
    pub pool:        &'pool Option<PgPool>,
    pub param:       Option<&'param str>,
    pub(crate) body: Option<JSON>,
}

impl<'d, 'pool, 'param> Context<'pool, 'param> {
    pub fn request_body<D: Deserialize<'d>>(&'d self) -> Result<D> {
        let json = self.body.as_ref()
            .ok_or_else(|| Response::BadRequest("expected request body"))?;
        let json_struct = json.to_struct()?;
        Ok(json_struct)
    }
}