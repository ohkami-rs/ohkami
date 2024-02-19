use ohkami::{builtin::JWT, FrontFang, BackFang, IntoResponse, Request, Response};
use sqlx::PgPool;
use crate::{config, errors::RealWorldError};


pub struct Auth {
    condition: Option<fn(&Request)->bool>
}
impl Auth {
    pub fn with_condition(condition: fn(&Request)->bool) -> Self {
        Auth { condition: Some(condition) }
    }
}
impl Default for Auth {
    fn default() -> Self {
        Auth { condition: None }
    }
}
impl FrontFang for Auth {
    type Error = Response;
    async fn bite(&self, req: &mut Request) -> Result<(), Response> {
        if self.condition.is_some_and(|cond| !cond(req)) {
            return Ok(());
        }

        let secret = config::JWT_SECRET_KEY()
            .map_err(RealWorldError::into_response)?;
        let payload: config::JWTPayload = JWT::default(secret).verified(req)?;
        req.memorize(payload);
        Ok(())
    }
}

pub struct OptionalAuth {
    condition: Option<fn(&Request)->bool>,
}
impl OptionalAuth {
    pub fn with_condition(condition: fn(&Request)->bool) -> Self {
        Self { condition: Some(condition) }
    }
}
impl Default for OptionalAuth {
    fn default() -> Self {
        Self { condition: None }
    }
}
impl FrontFang for OptionalAuth {
    type Error = RealWorldError;
    async fn bite(&self, req: &mut Request) -> Result<(), Self::Error> {
        if self.condition.is_some_and(|cond| !cond(req)) {
            return Ok(());
        }

        let secret = config::JWT_SECRET_KEY()?;
        let payload: Option<config::JWTPayload> = JWT::default(secret).verified(req).ok();
        req.memorize(payload);
        Ok(())
    }
}

pub struct LogRequest;
impl FrontFang for LogRequest {
    type Error = std::convert::Infallible;
    fn bite(&self, req: &mut Request) -> impl std::future::Future<Output = Result<(), Self::Error>> + Send {
        let method = req.method();
        let path   = req.path();

        tracing::info!("{method:<7} {path}");

        async {Ok(())}
    }
}

pub struct LogResponse;
impl BackFang for LogResponse {
    type Error = std::convert::Infallible;
    fn bite(&self, res: &mut Response, _: &Request) -> impl std::future::Future<Output = Result<(), Self::Error>> + Send {
        tracing::info!("{res:?}");
        async {Ok(())}
    }
}

pub struct ConnectionPool(PgPool);
impl FrontFang for ConnectionPool {
    type Error = std::convert::Infallible;
    fn bite(&self, req: &mut Request) -> impl std::future::Future<Output = Result<(), Self::Error>> + Send {
        req.memorize(self.0.clone());
        async {Ok(())}
    }
}
impl From<PgPool> for ConnectionPool {
    fn from(pool: PgPool) -> Self {
        Self(pool)
    }
}
