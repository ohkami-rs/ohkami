use std::future::Future;
use serde::Serialize;
use super::Handler;
use crate::{context::Context, response::Response};

pub trait IntoHandler<'router> {
    fn into_handlefunc(self) -> Handler<'router>;
}

impl<'router, F, Fut, T> IntoHandler<'router> for F
where
    F:   Fn(Context) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Response<T>> + Send + 'static,
    T:   Serialize,
{
    fn into_handlefunc(self) -> Handler<'router> {
        Box::new(move |c, _, _| Box::pin(async {
            let response = self(c).await;
            response.send(&mut c.stream).await
        }))
    }
}
