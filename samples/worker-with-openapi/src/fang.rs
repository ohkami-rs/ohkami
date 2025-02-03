use crate::Bindings;
use ohkami::{Request, Response, FromRequest, serde::json};
use ohkami::prelude::{FangAction, Deserialize};

#[cfg(feature="openapi")]
use ohkami::openapi;


/// memorize `TokenAuthed`
#[derive(Clone)]
pub(super) struct TokenAuth;

pub(super) struct TokenAuthed {
    pub(super) user_id:   i32,
    pub(super) user_name: String,
}

#[derive(Deserialize)]
struct TokenSchema<'req> {
    user_id: i32,
    token:   &'req str,
}

impl FangAction for TokenAuth {
    async fn fore<'a>(&'a self, req: &'a mut Request) -> Result<(), Response> {
        let authorization_bearer = req.headers
            .Authorization().ok_or_else(Response::BadRequest)?
            .strip_prefix("Bearer ").ok_or_else(Response::BadRequest)?;

        let TokenSchema { user_id, token } =
            json::from_str(authorization_bearer)
                .inspect_err(|e| worker::console_error!("Failed to parse TokenSchema `{authorization_bearer}`: {e}"))
                .map_err(|_| Response::Unauthorized())?;

        let Bindings { DB, .. } = FromRequest::from_request(req).unwrap()?;
        let user_name = DB.prepare("SELECT name FROM users WHERE id = ? AND token = ?")
            .bind(&[user_id.into(), token.into()])?
            .first::<String>(Some("name")).await?
            .ok_or_else(Response::Unauthorized)?;

        req.context.set(TokenAuthed { user_id, user_name });
        Ok(())
    }

    #[cfg(feature="openapi")]
    fn openapi_map_operation(&self, operation: openapi::Operation) -> openapi::Operation {
        operation.security(
            openapi::SecurityScheme::Bearer("tokenAuth", Some("JSON (user_id, token)")),
            &[]
        )
    }
}


#[derive(Clone)]
pub(super) struct Logger;
impl FangAction for Logger {
    async fn fore<'a>(&'a self, req: &'a mut Request) -> Result<(), Response> {
        worker::console_log!("{req:?}");
        Ok(())
    }
    async fn back<'a>(&'a self, res: &'a mut Response) {
        worker::console_log!("{res:?}");
    }
}
