use std::{pin::Pin, future::Future};
use crate::{context::Context, test::Method};

pub(crate) type MiddlewareFunc = Box<dyn Fn(&mut Context) -> Pin<Box<dyn Future<Output=()> + Send>> + Send + Sync>;
trait MiddlewareClone:
    Fn(&mut Context) -> Pin<Box<dyn Future<Output=()> + Send>> + Send + Sync
    + Clone
{}

pub trait MiddlewareArg {}
pub trait MiddlewareProcess<Arg: MiddlewareArg> {
    fn into_middleware_func(self) -> MiddlewareFunc;
}
impl MiddlewareArg for () {}
impl<F, Fut> MiddlewareProcess<()> for F
where
    F:   Fn() -> Fut + Send + Sync + 'static,
    Fut: Future<Output=()> + Send + 'static,
{
    fn into_middleware_func(self) -> MiddlewareFunc {
        Box::new(move |_| Box::pin(self()))
    }
}
impl MiddlewareArg for (&Context,) {}
impl<F, Fut> MiddlewareProcess<(&Context,)> for F
where
    F:   Fn(&Context) -> Fut + Send + Sync + 'static,
    Fut: Future<Output=()> + Send + 'static,
{
    fn into_middleware_func(self) -> MiddlewareFunc {
        Box::new(move |ctx| Box::pin(self(ctx)))
    }
}
impl MiddlewareArg for &Context {}
impl<F, Fut> MiddlewareProcess<&Context> for F
where
    F:   Fn(&mut Context) -> Fut + Send + Sync + 'static,
    Fut: Future<Output=()> + Send + 'static,
{
    fn into_middleware_func(self) -> MiddlewareFunc {
        Box::new(move |ctx| Box::pin(self(ctx)))
    }
}

pub struct Middleware(
    pub(crate) Vec<(Method,/*route*/&'static str, MiddlewareFunc)>
); impl Middleware {
    pub fn init() -> Self {
        Self(Vec::new())
    }
    #[allow(non_snake_case)]
    pub fn ANY<P: MiddlewareProcess<Arg> + Clone, Arg: MiddlewareArg>(mut self, route: &'static str, proccess: P) -> Self {
        self.0.push((Method::GET, route, proccess.clone().into_middleware_func()));
        self.0.push((Method::POST, route, proccess.clone().into_middleware_func()));
        self.0.push((Method::PATCH, route, proccess.clone().into_middleware_func()));
        self.0.push((Method::DELETE, route, proccess.clone().into_middleware_func()));
        self
    }
    #[allow(non_snake_case)]
    pub fn GET<P: MiddlewareProcess<Arg>, Arg: MiddlewareArg>(mut self, route: &'static str, proccess: P) -> Self {
        self.0.push((Method::GET, route, proccess.into_middleware_func()));
        self
    }
    #[allow(non_snake_case)]
    pub fn POST<P: MiddlewareProcess<Arg>, Arg: MiddlewareArg>(mut self, route: &'static str, proccess: P) -> Self {
        self.0.push((Method::POST, route, proccess.into_middleware_func()));
        self
    }
    #[allow(non_snake_case)]
    pub fn PATCH<P: MiddlewareProcess<Arg>, Arg: MiddlewareArg>(mut self, route: &'static str, proccess: P) -> Self {
        self.0.push((Method::PATCH, route, proccess.into_middleware_func()));
        self
    }
    #[allow(non_snake_case)]
    pub fn DELETE<P: MiddlewareProcess<Arg>, Arg: MiddlewareArg>(mut self, route: &'static str, proccess: P) -> Self {
        self.0.push((Method::DELETE, route, proccess.into_middleware_func()));
        self
    }

    pub(crate) fn merge(mut self, mut another: Self) -> Self {
        self.0.append(&mut another.0);
        Self(self.0)
    }
    pub(crate) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
