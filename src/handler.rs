use std::{future::Future, pin::Pin};
use crate::{response::Response, result::Result, context::Context, utils::map::RangeList};


pub(crate) type HandleFunc = Box<dyn Fn(Context, RangeList) -> Pin<Box<dyn Future<Output=Result<Response>> + Send >> + Send + Sync>;
pub trait Handler<P: Param> {
    fn into_handlefunc(self) -> HandleFunc;
}

impl<F, Fut> Handler<()> for F
where
    F:   Fn(Context) -> Fut + Send + Sync + 'static,
    Fut: Future<Output=Result<Response>> + Send + 'static
{
    fn into_handlefunc(self) -> HandleFunc {
        Box::new(move |ctx, _| Box::pin(self(ctx)))
    }
}

impl<F, Fut> Handler<usize> for F
where
    F:   Fn(Context, usize) -> Fut + Send + Sync + 'static,
    Fut: Future<Output=Result<Response>> + Send + 'static
{
    fn into_handlefunc(self) -> HandleFunc {
        Box::new(move |ctx, params|
            match params.get(0) {
                Some(range) => {
                    let param = ctx.buffer.read_str(&range).parse::<usize>().unwrap();
                    Box::pin(self(ctx, param))
                },
                None => unreachable!(/* --- */),
            }
        )
    }
}

pub trait Param {}
impl Param for () {}
impl Param for usize {}

#[cfg(test)]
mod test {
    use crate::{context::Context, response::Response, result::Result, components::json::JSON, json};
    use super::{Handler, Param, HandleFunc};

    async fn a(_: Context) -> Result<Response> {
        Response::OK(None::<JSON>)
    }
    async fn b(_: Context, id: usize) -> Result<Response> {
        Response::OK(json!("id": id))
    }

    struct Handlers(
        Vec<HandleFunc>
    );impl Handlers {
        fn new() -> Self {
            Self(Vec::new())
        }
        fn push<H: Handler<P> + 'static, P: Param>(&mut self, handler: H) {
            self.0.push(handler.into_handlefunc())
        }
    }

    #[test]
    fn different_signature_handlers() {
        let mut handlers = Handlers::new();

        handlers.push(a);
        handlers.push(b);
    }
}