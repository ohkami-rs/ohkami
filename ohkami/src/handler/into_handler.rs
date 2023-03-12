use std::future::Future;
use serde::Serialize;
use super::Handler;
use crate::{context::Context, response::Response};

pub trait IntoHandler {
    fn into_handlefunc(&'static self) -> Handler;
}

impl<F, Fut, T> IntoHandler for F
where
    F:   Fn(Context) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Response<T>> + Send + 'static,
    T:   Serialize,
{
    fn into_handlefunc(&'static self) -> Handler {
        Box::new(move |mut stream, c, _, _| Box::pin(async move {
            let response = self(c).await;
            response.send(&mut stream).await
        }))
    }
}
