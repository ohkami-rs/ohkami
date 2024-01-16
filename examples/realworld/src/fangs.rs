use ohkami::{Fang, IntoFang, Request, Response};
use crate::config;


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
        Fang(move |req: &mut Request| {
            if !self.condition.is_some_and(|cond| cond(req)) {
                return Ok(());
            }

            let payload: config::JWTPayload = config::jwt().verified(req)?;
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
        Fang(move |req: &mut Request| {
            if !self.condition.is_some_and(|cond| cond(req)) {
                return Ok(());
            }

            let payload: Option<config::JWTPayload> = config::jwt().verified(req).ok();
            req.memorize(payload);
            Ok(())
        })
    }
}

pub struct LogRequest;
impl IntoFang for LogRequest {
    fn into_fang(self) -> Fang {
        Fang(|req: &Request| {
            let method = req.method;
            let path   = req.path();

            tracing::info!("{method:<7} {path}");
        })
    }
}

pub struct LogResponse;
impl IntoFang for LogResponse {
    fn into_fang(self) -> Fang {
        Fang(|res: &Response| {
            tracing::info!("{res:?}");
        })
    }
}
