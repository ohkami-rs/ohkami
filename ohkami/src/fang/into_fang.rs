use std::future::Future;
use crate::context::Context;
use super::Fang;

pub trait IntoFang<A> {
    fn into_fang(&'static self) -> Fang;
}

impl<F, Fut> IntoFang<(Context,)> for F
where
    F:   Fn(Context) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Context> + Send,
{
    fn into_fang(&'static self) -> Fang {
        Box::new(move |c, request| Box::pin(async {
            (self(c).await, request)
        }))
    }
}

impl<F, Fut> IntoFang<()> for F
where
    F:   Fn(&Context) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = ()> + Send,
{
    fn into_fang(&'static self) -> Fang {
        Box::new(|c, request| Box::pin(async {
            self(&c).await;
            (c, request)
        }))
    }
}
