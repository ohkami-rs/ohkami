use ohkami::{builtin::fang::JWT, Fang, FangProc, IntoResponse, Request, Response};
use ohkami::serde::{Serialize, Deserialize};
use sqlx::PgPool;
use crate::config;
use std::marker::PhantomData;


pub struct Auth<
    Payload: Serialize + for<'de> Deserialize<'de> + Send + Sync + 'static
        = config::JWTPayload
> {
    condition: Option<fn(&Request)->bool>,
    __payload: PhantomData<Payload>,
}
impl<
    Payload: Serialize + for<'de> Deserialize<'de> + Send + Sync + 'static
> Auth<Payload> {
    pub fn with_condition(condition: fn(&Request)->bool) -> Self {
        Auth {
            condition: Some(condition),
            __payload: PhantomData
        }
    }
}
impl Default for Auth {
    fn default() -> Self {
        Auth {
            condition: None,
            __payload: PhantomData
        }
    }
}
const _: () = {
    impl<
        I: FangProc,
        Payload: Serialize + for<'de> Deserialize<'de> + Send + Sync + 'static
    > Fang<I> for Auth<Payload> {
        type Proc = AuthProc<I, Payload>;
        fn chain(&self, inner: I) -> Self::Proc {
            AuthProc {
                inner,
                auth: Auth {
                    condition: self.condition,
                    __payload: PhantomData
                }
            }
        }
    }

    pub struct AuthProc<I: FangProc, Payload: Serialize + for<'de> Deserialize<'de> + Send + Sync + 'static> {
        inner: I,
        auth:  Auth<Payload>,
    }
    impl<
        I: FangProc,
        Payload: Serialize + for<'de> Deserialize<'de> + Send + Sync + 'static,
    > FangProc for AuthProc<I, Payload> {
        async fn bite<'b>(&'b self, req: &'b mut Request) -> Response {
            if self.auth.condition.is_some_and(|cond| !cond(req)) {
                return self.inner.bite(req).await;
            }
            
            let secret  = match config::JWT_SECRET_KEY() {
                Ok(sk) => sk,
                Err(e) => return e.into_response()
            };
            let payload = match JWT::<Payload>::default(secret).verified(req) {
                Ok(pl) => pl,
                Err(e) => return e.into_response()
            };
            req.memorize(payload);
            
            self.inner.bite(req).await
        }
    }
};

// pub struct OptionalAuth {
//     condition: Option<fn(&Request)->bool>,
// }
// impl OptionalAuth {
//     pub fn with_condition(condition: fn(&Request)->bool) -> Self {
//         Self { condition: Some(condition) }
//     }
// }
// impl Default for OptionalAuth {
//     fn default() -> Self {
//         Self { condition: None }
//     }
// }
// const _: () = {
//     impl<I: FangProc> Fang<I> for OptionalAuth {
//         type Proc = OptionalAuthProc<I>;
//         fn chain(&self, inner: I) -> Self::Proc {
//             OptionalAuthProc {
//                 optional_auth: Self { condition: self.condition },
//                 inner
//             }
//         }
//     }
// 
//     struct OptionalAuthProc<I: FangProc> {
//         inner: I,
//         optional_auth: OptionalAuth
//     }
// 
//     impl<I: FangProc> FangProc for OptionalAuthProc<I> {
//         async fn bite<'b>(&'b self, req: &'b mut Request) -> Response {
//             if self.optional_auth.condition.is_some_and(|cond| !cond(req)) {
//                 return self.inner.bite(req).await;
//             }
//             
//             let secret = config::JWT_SECRET_KEY()?;
//             let payload: Option<config::JWTPayload> = JWT::default(secret).verified(req).ok();
//             req.memorize(payload);
//             Ok(())
//         }
//     }
// };

pub struct Logger;
const _: () = {
    impl<I: FangProc> Fang<I> for Logger {
        type Proc = LoggerProc<I>;
        fn chain(&self, inner: I) -> Self::Proc {
            LoggerProc { inner }
        }
    }

    pub struct LoggerProc<I: FangProc> {
        inner: I
    }

    impl<I: FangProc> FangProc for LoggerProc<I> {
        async fn bite<'b>(&'b self, req: &'b mut Request) -> Response {
            let method = req.method();
            let path   = req.path();

            tracing::info!("\n[req]\n{method:<7} {path}");

            let res = self.inner.bite(req).await;

            tracing::info!("\n[res]\n{res:?}");

            res
        }
    }
};

pub struct ConnectionPool(PgPool);
impl From<PgPool> for ConnectionPool {
    fn from(pool: PgPool) -> Self {
        Self(pool)
    }
}
const _: () = {
    impl<I: FangProc> Fang<I> for ConnectionPool {
        type Proc = ConnectionPoolProc<I>;
        fn chain(&self, inner: I) -> Self::Proc {
            ConnectionPoolProc {
                inner,
                pool: self.0.clone()
            }
        }
    }

    pub struct ConnectionPoolProc<I: FangProc> {
        pool:  PgPool,
        inner: I,
    }
    impl<I: FangProc> FangProc for ConnectionPoolProc<I> {
        fn bite<'b>(&'b self, req: &'b mut Request) -> impl std::future::Future<Output = Response> + Send + 'b {
            req.memorize(self.pool.clone());
            self.inner.bite(req)
        }
    }
};
