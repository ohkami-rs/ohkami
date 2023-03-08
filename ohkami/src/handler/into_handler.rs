use std::future::Future;
use serde::Serialize;
use super::Handler;
use crate::{context::Context, response::Response};

pub trait IntoHandler<'router> {
    fn into_handlefunc(self) -> Handler<'router>;
}

impl<'router, F, Fut, T> IntoHandler<'router> for F
where
    F:   Fn(Context) -> Fut + Send + Sync + 'router,
    Fut: Future<Output = Response<T>> + Send + 'router,
    T:   Serialize,
{
    fn into_handlefunc(self) -> Handler<'router> {
        Box::new(move |mut stream, c, _, _| Box::pin(async {
            let response = self(c).await;
            response.send(&mut stream).await
        }))
    }
}
