use std::{pin::Pin, future::Future};
use serde::{Serialize, Deserialize};
use crate::{
    Context, Request,
    layer0_lib::{List, BufRange},
    layer1_req_res::{Response, PathParam},
};

pub(crate) const PATH_PARAMS_LIMIT: usize = 2;
type PathParams = List<BufRange, PATH_PARAMS_LIMIT>;


pub struct Handler<T: Serialize>(
    Box<dyn
        Fn(Request, Context, PathParams) -> Pin<
            Box<dyn
                Future<Output = Response<T>>
                + Send + 'static
            >
        > + Send + Sync + 'static
    >
);

pub trait IntoHandler<Args, T:Serialize> {
    fn into_handler(self) -> Handler<T>;
}


impl<F, Fut, T> IntoHandler<(Context,), T> for F
where
    F:   Fn(Context) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Response<T>> + Send + Sync + 'static,
    T:   serde::Serialize + Send,
{
    fn into_handler(self) -> Handler<T> {
        Handler(Box::new(move |_, c, _| {
            Box::pin(
                self(c)
            )
        }))
    }
}

macro_rules! with_single_path_param {
    ($( $param_type:ty ),*) => {$(
        impl<F, Fut, T> IntoHandler<(Context, $param_type), T> for F
        where
            F:   Fn(Context, $param_type) -> Fut + Send + Sync + 'static,
            Fut: Future<Output = Response<T>> + Send + Sync + 'static,
            T:   serde::Serialize + Send + 'static,
        {
            fn into_handler(self) -> Handler<T> {
                Handler(Box::new(move |req, c, params| {
                    match <$param_type as PathParam>::parse(&req.buffer[unsafe {params.list[0].assume_init_ref()}]) {
                        Ok(p1) => Box::pin(self(c, p1)),
                        Err(e) => {
                            let res = Response::Err(c.BadRequest().text(e.to_string()));
                            Box::pin(async {res})
                        },
                    }
                }))
            }
        }
    )*};
} with_single_path_param! { &str, String, u8, u16, u32, u64, u128, usize }

impl<F, Fut, T, P1:PathParam> IntoHandler<(Context, (P1,)), T> for F
where
    F:   Fn(Context, (P1,)) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Response<T>> + Send + Sync + 'static,
    T:   serde::Serialize + Send + 'static,
{
    fn into_handler(self) -> Handler<T> {
        Handler(Box::new(move |req, c, params| {
            // SAFETY: Due to the architecture of `Router`,
            // `params` has already `append`ed once before this code
            match <P1 as PathParam>::parse(&req.buffer[unsafe {params.list[0].assume_init_ref()}]) {
                Ok(p1) => Box::pin(self(c, (p1,))),
                Err(e) => {
                    let res = Response::Err(c.BadRequest().text(e.to_string()));
                    Box::pin(async {res})
                },
            }
        }))
    }
}
impl<F, Fut, T, P1:PathParam, P2:PathParam> IntoHandler<(Context, (P1, P2)), T> for F
where
    F:   Fn(Context, (P1, P2)) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Response<T>> + Send + Sync + 'static,
    T:   serde::Serialize + Send + 'static,
{
    fn into_handler(self) -> Handler<T> {
        Handler(Box::new(move |req, c, params| {
            // SAFETY: Due to the architecture of `Router`,
            // `params` has already `append`ed twice before this code
            match <P1 as PathParam>::parse(&req.buffer[unsafe {params.list[0].assume_init_ref()}]) {
                Ok(p1) => match <P2 as PathParam>::parse(&req.buffer[unsafe {params.list[1].assume_init_ref()}]) {
                    Ok(p2) => Box::pin(self(c, (p1, p2))),
                    Err(e) => {
                        let res = Response::Err(c.BadRequest().text(e.to_string()));
                        Box::pin(async move {res})
                    }
                }
                Err(e) => {
                    let res = Response::Err(c.BadRequest().text(e.to_string()));
                    Box::pin(async {res})
                },
            }
        }))
    }
}

