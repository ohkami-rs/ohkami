use std::future::Future;

use serde::Serialize;

use crate::{context::Context, response::Response};

use super::HandleFunc;

pub trait IntoHandleFunc<'router> {
    fn into_handlefunc(self) -> HandleFunc<'router>;
}

impl<'router, F, Fut, T> IntoHandleFunc<'router> for F
where
    F:   Fn(Context) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Response<T>> + Send + 'static,
    T:   Serialize,
{
    fn into_handlefunc(self) -> HandleFunc<'router> {
        Box::new(move |stream, c, _| Box::pin(async {
            let response = self(c).await;
            response.send(stream).await
        }))
    }
}
