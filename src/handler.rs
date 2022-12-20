use std::{future::Future, pin::Pin};
use crate::{response::Response, result::Result, context::Context, utils::range::RangeList};


pub(crate) type HandleFunc = Box<dyn Fn(Context, RangeList) -> Pin<Box<dyn Future<Output=Result<Response>> + Send >> + Send + Sync>;
pub trait Param {}

pub trait Handler<P: Param> {
    fn into_handlefunc(self) -> HandleFunc;
}




impl Param for () {}
impl<F, Fut> Handler<()> for F
where
F:   Fn(Context) -> Fut + Send + Sync + 'static,
Fut: Future<Output=Result<Response>> + Send + 'static
{
    fn into_handlefunc(self) -> HandleFunc {
        Box::new(move |ctx, _| Box::pin(self(ctx)))
    }
}

impl Param for String {}
impl<F, Fut> Handler<String> for F
where
F:   Fn(Context, String) -> Fut + Send + Sync + 'static,
Fut: Future<Output=Result<Response>> + Send + 'static
{
    fn into_handlefunc(self) -> HandleFunc {
        Box::new(move |ctx, params|
            match params.get1() {
                Some(range) => {
                    let param = ctx.buffer.read_str(&range).to_owned();
                    Box::pin(self(ctx, param))
                },
                None => unreachable!(/* --- */),
            }
        )
    }
}

macro_rules! impl_handler_with_int {
    ( $( $int_type:ty )* ) => {
        $(
            impl Param for $int_type {}
            impl<F, Fut> Handler<$int_type> for F
            where
                F:   Fn(Context, $int_type) -> Fut + Send + Sync + 'static,
                Fut: Future<Output=Result<Response>> + Send + 'static
            {
                fn into_handlefunc(self) -> HandleFunc {
                    Box::new(move |ctx, params|
                        match params.get1() {
                            Some(range) => {
                                let parsed = ctx.buffer.read_str(&range).parse::<$int_type>();
                                match parsed {
                                    Ok(param) => Box::pin(self(ctx, param)),
                                    _ => Box::pin(async {Err(Response::BadRequest("format of path param is wrong"))})
                                }
                            },
                            None => unreachable!(/* --- */),
                        }
                    )
                }
            }
        )*
    };
}
impl_handler_with_int!(u8 u32 u64 usize isize i64);

macro_rules! impl_handler_with_2ints {
    ( $( ($int1:ty, $int2:ty) )* ) => {
        $(
            impl Param for ($int1, $int2) {}
            impl<F, Fut> Handler<($int1, $int2)> for F
            where
                F:   Fn(Context, $int1, $int2) -> Fut + Send + Sync + 'static,
                Fut: Future<Output=Result<Response>> + Send + 'static
            {
                fn into_handlefunc(self) -> HandleFunc {
                    Box::new(move |ctx, params|
                        match params.get2() {
                            Some((range1, range2)) => {
                                let parsed1 = ctx.buffer.read_str(&range1).parse::<$int1>();
                                let parsed2 = ctx.buffer.read_str(&range2).parse::<$int2>();
                                match (parsed1, parsed2) {
                                    (Ok(param1), Ok(param2)) => Box::pin(self(ctx, param1, param2)),
                                    _ => Box::pin(async {Err(Response::BadRequest("format of path param is wrong"))})
                                }
                            },
                            None => unreachable!(/* --- */),
                        }
                    )
                }
            }
        )*
    };
}
impl_handler_with_2ints!(
    (u8, u8) (u8, u32) (u8,u64)  (u8,usize)  (u8, isize) (u8, i64)
    (u32,u8) (u32,u32) (u32,u64) (u32,usize) (u32,isize) (u32,i64)
);


#[cfg(test)]
mod test {
    use crate::{context::Context, response::Response, result::Result, components::json::JSON, json};
    use super::{Handler, Param, HandleFunc};

    async fn a(_: Context) -> Result<Response> {
        Response::OK(None::<JSON>)
    }
    async fn b(_: Context, id: u8) -> Result<Response> {
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