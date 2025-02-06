use ohkami::fang::{JWT, FangAction};
use ohkami::{IntoResponse, Request, Response};


/// memorizes `crate::config::JWTPayload`
#[derive(Clone)]
pub struct Auth {
    /// When `true`, not reject the request when it doesn't have any credential
    /// and just let it go without JWTPayload
    optional: bool,
}
impl Auth {
    pub fn required() -> Self {
        Self { optional: false }
    }
    pub fn optional() -> Self {
        Self { optional: true }
    }
}
impl FangAction for Auth {
    async fn fore<'a>(&'a self, req: &'a mut Request) -> Result<(), Response> {
        if req.headers.Authorization().is_none() && self.optional {
            return Ok(());
        }

        let secret = crate::config::JWT_SECRET_KEY()
            .map_err(IntoResponse::into_response)?;
        let payload = JWT::<crate::config::JWTPayload>::default(secret)
            .verified(req)
            .map_err(IntoResponse::into_response)?;
        req.context.set(payload);
        Ok(())
    }
}

#[derive(Clone)]
pub struct Logger;
impl FangAction for Logger {
    async fn fore<'a>(&'a self, req: &'a mut Request) -> Result<(), Response> {
        tracing::info!("req = {:<7} {}", req.method, req.path.str());
        Ok(())
    }

    async fn back<'a>(&'a self, res: &'a mut Response) {
        tracing::info!("res = {res:?}");
    }
}
