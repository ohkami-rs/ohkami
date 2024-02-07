use ohkami::{builtin::JWT, Fang, IntoFang, IntoResponse, Request, Response};
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
impl IntoFang for Auth {
    fn into_fang(self) -> Fang {
        Fang::front(move |req: &mut Request| {
            if self.condition.is_some_and(|cond| !cond(req)) {
                return Ok(());
            }

            let secret = config::JWT_SECRET_KEY()
                .map_err(RealWorldError::into_response)?;
            let payload: config::JWTPayload = JWT::default(secret).verified(req)?;
            req.memorize(payload);
            Ok(())
        })
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
impl IntoFang for OptionalAuth {
    fn into_fang(self) -> Fang {
        Fang::front(move |req: &mut Request| {
            if self.condition.is_some_and(|cond| !cond(req)) {
                return Ok(());
            }

            let secret = config::JWT_SECRET_KEY()
                .map_err(RealWorldError::into_response)?;
            let payload: Option<config::JWTPayload> = JWT::default(secret).verified(req).ok();
            req.memorize(payload);
            Ok(())
        })
    }
}

pub struct LogRequest;
impl IntoFang for LogRequest {
    fn into_fang(self) -> Fang {
        Fang::front(|req: &Request| {
            let method = req.method();
            let path   = req.path();

            tracing::info!("{method:<7} {path}");
        })
    }
}

pub struct LogResponse;
impl IntoFang for LogResponse {
    fn into_fang(self) -> Fang {
        Fang::back(|res: &Response| {
            tracing::info!("{res:?}");
        })
    }
}

pub struct ConnectionPool(PgPool);
impl IntoFang for ConnectionPool {
    fn into_fang(self) -> Fang {
        Fang::front(move |req: &mut Request| {
            req.memorize(self.0.clone())
        })
    }
}
impl From<PgPool> for ConnectionPool {
    fn from(pool: PgPool) -> Self {
        Self(pool)
    }
}
