use std::{pin::Pin, future::Future};
use serde::{Serialize, Deserialize};
use crate::{
    Context, Request,
    layer0_lib::{List, BufRange},
    layer1_req_res::{Response},
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
        Handler(
            Box::new(move |_, c, _| {
                Box::pin(
                    self(c)
                )
            })
        )
    }
}

impl<F, Fut, T,
    P1:for<'de>Deserialize<'de>,
> IntoHandler<(Context, (P1,)), T> for F
where
    F:   Fn(Context, P1) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Response<T>> + Send + Sync + 'static,
    T:   serde::Serialize + Send,
{
    fn into_handler(self) -> Handler<T> {
        Handler(
            Box::new(move |req, c, params| {
                let p1 = serde_json::from_slice(
                    &req.buffer[
                        // SAFETY: Router の仕組み上, これが呼ばれた時点で
                        // params は１回 append されている
                        unsafe {params.list[0].assume_init_ref()}
                    ]
                ).expect("Failed to deserialize");

                Box::pin(self(c, p1))
            })
        )
    }
}

impl<F, Fut, T,
    P1:for<'de>Deserialize<'de>,
    P2:for<'de>Deserialize<'de>,
> IntoHandler<(Context, (P1, P2)), T> for F
where
    F:   Fn(Context, (P1, P2)) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Response<T>> + Send + Sync + 'static,
    T:   serde::Serialize + Send,
{
    fn into_handler(self) -> Handler<T> {
        Handler(
            Box::new(move |req, c, params| {
                // SAFETY: 上と同様. これが呼ばれた時点で,
                // Router の仕組み上２回 append されている
                let p1 = serde_json::from_slice(
                    &req.buffer[
                        unsafe {params.list[0].assume_init_ref()}
                    ]
                ).expect("Failed to deserialize");

                let p2 = serde_json::from_slice(
                    &req.buffer[
                        unsafe {params.list[1].assume_init_ref()}
                    ]
                ).expect("Failed to deserialize");

                Box::pin(self(c, (p1, p2)))
            })
        )
    }
}

#[cfg(test)] #[test] fn check_deserialize_tuple() {
    let _: String = match serde_json::to_string(&(42usize, 24usize)) {
        Ok(s)  => dbg!(s),
        Err(e) => panic!("{e}"),
    };
    let _: (usize, usize) = match serde_json::from_str("[42, 24]") {
        Ok(de) => dbg!(de),
        Err(e) => panic!("{e}"),
    };
}
