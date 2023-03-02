use std::future::Future;

use serde::Serialize;

use crate::{context::Context, response::Response};

use super::HandleFunc;

pub trait IntoHandleFunc<'buf> {
    fn into_handlefunc(self) -> HandleFunc<'buf>;
}

impl<'buf, F, Fut, T> IntoHandleFunc<'buf> for F
where
    F:   Fn(Context) -> Fut,
    Fut: Future<Output = Response<T>>,
    T:   Serialize,
{
    fn into_handlefunc(self) -> HandleFunc<'buf> {
           
    }
}

