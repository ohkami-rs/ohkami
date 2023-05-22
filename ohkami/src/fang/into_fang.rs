use std::future::Future;
use crate::context::Context;
use super::Fang;

pub trait IntoFang<A>: Clone {
    fn into_fang(self) -> Fang;
}

impl<F, Fut> IntoFang<(Context,)> for F
where
    F:   Clone + Fn(Context) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Context> + Send,
{
    fn into_fang(self) -> Fang {
        Box::new(move |c, request| Box::pin(async {
            (self(c).await, request)
        }))
    }
}

impl<F, Fut> IntoFang<()> for F
where
    F:   Clone + Fn(&Context) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = ()> + Send,
{
    fn into_fang(self) -> Fang {
        Box::new(|c, request| Box::pin(async {
            self(&c).await;
            (c, request)
        }))
    }
}
