use std::{future::Future, borrow::Cow};
use super::Handler;
use crate::{
    Context,
    Response,
    layer1_req_res::{FromRequest, FromParam},
};
#[cfg(feature="websocket")]
use crate::websocket::WebSocketContext;


pub trait IntoHandler<Args> {
    fn into_handler(self) -> Handler;
}

#[cold] fn __bad_request(
    c: &Context,
    e: impl std::fmt::Display,
) -> std::pin::Pin<Box<impl Future<Output = Response>>> {
    Box::pin({
        let res = c.BadRequest().text(e.to_string());
        async {res}
    })
}

#[inline] fn from_param_bytes<P: FromParam>(bytes: &[u8]) -> Result<P, Cow<'static, str>> {
    let param = std::str::from_utf8(bytes)
        .map_err(|utf8err| Cow::Owned(utf8err.to_string()))?;
    <P as FromParam>::from_param(param)
        .map_err(|e| e.to_string().into())
}

const _: (/* only Context */) = {
    impl<F, Fut> IntoHandler<(Context,)> for F
    where
        F:   Fn(Context) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + Sync + 'static,
    {
        fn into_handler(self) -> Handler {
            Handler::new(move |_, c, _|
                Box::pin(self(c))
            )
        }
    }
};

const _: (/* FromParam */) = {
    macro_rules! with_single_path_param {
        ($( $param_type:ty ),*) => {$(
            impl<F, Fut> IntoHandler<(Context, $param_type)> for F
            where
                F:   Fn(Context, $param_type) -> Fut + Send + Sync + 'static,
                Fut: Future<Output = Response> + Send + Sync + 'static,
            {
                fn into_handler(self) -> Handler {
                    Handler::new(move |_, c, params|
                        match from_param_bytes::<$param_type>(unsafe {params.assume_init_first().as_bytes()}) {
                            Ok(p1) => Box::pin(self(c, p1)),
                            Err(e) => __bad_request(&c, e)
                        }
                    )
                }
            }
        )*};
    } with_single_path_param! {
        String, u8, u16, u32, u64, u128, usize
    }

    impl<F, Fut, P1:FromParam> IntoHandler<(Context, (P1,))> for F
    where
        F:   Fn(Context, (P1,)) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + Sync + 'static,
    {
        fn into_handler(self) -> Handler {
            Handler::new(move |_, c, params|
                // SAFETY: Due to the architecture of `Router`,
                // `params` has already `append`ed once before this code
                match from_param_bytes(unsafe {params.assume_init_first().as_bytes()}) {
                    Ok(p1) => Box::pin(self(c, (p1,))),
                    Err(e) => __bad_request(&c, e)
                }
            )
        }
    }

    impl<F, Fut, P1:FromParam, P2:FromParam> IntoHandler<(Context, (P1, P2))> for F
    where
        F:   Fn(Context, (P1, P2)) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + Sync + 'static,
    {
        fn into_handler(self) -> Handler {
            Handler::new(move |_, c, params| {
                let (p1, p2) = params.assume_init_extract();
                let (p1, p2) = unsafe {(p1.as_bytes(), p2.as_bytes())};
                match (from_param_bytes::<P1>(p1), from_param_bytes::<P2>(p2)) {
                    (Ok(p1), Ok(p2))          => Box::pin(self(c, (p1, p2))),
                    (Err(e), _) | (_, Err(e)) => __bad_request(&c, e),
                }
            })
        }
    }
};

const _: (/* FromRequest items */) = {
    impl<F, Fut, Item1:FromRequest> IntoHandler<(Context, Item1)> for F
    where
        F:   Fn(Context, Item1) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + Sync + 'static,
    {
        fn into_handler(self) -> Handler {
            Handler::new(move |req, c, _|
                match Item1::parse(req) {
                    Ok(item1) => Box::pin(self(c, item1)),
                    Err(e)    => __bad_request(&c, e)
                }
            )
        }
    }

    impl<F, Fut, Item1:FromRequest, Item2:FromRequest> IntoHandler<(Context, Item1, Item2)> for F
    where
        F:   Fn(Context, Item1, Item2) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + Sync + 'static,
    {
        fn into_handler(self) -> Handler {
            Handler::new(move |req, c, _|
                match (Item1::parse(req), Item2::parse(req)) {
                    (Ok(item1), Ok(item2)) => Box::pin(self(c, item1, item2)),
                    (Err(e), _) => __bad_request(&c, e),
                    (_, Err(e)) => __bad_request(&c, e),
                }
            )
        }
    }
};

const _: (/* single FromParam and FromRequest items */) = {
    macro_rules! with_single_path_param_and_from_request_items {
        ($( $param_type:ty ),*) => {$(
            impl<F, Fut, Item1:FromRequest> IntoHandler<(Context, $param_type, Item1)> for F
            where
                F:   Fn(Context, $param_type, Item1) -> Fut + Send + Sync + 'static,
                Fut: Future<Output = Response> + Send + Sync + 'static,
            {
                fn into_handler(self) -> Handler {
                    Handler::new(move |req, c, params| {
                        // SAFETY: Due to the architecture of `Router`,
                        // `params` has already `append`ed once before this code
                        let p1 = unsafe {params.assume_init_first().as_bytes()};

                        match (from_param_bytes(p1), Item1::parse(req)) {
                            (Ok(p1), Ok(item1)) => Box::pin(self(c, p1, item1)),
                            (Err(e), _) => __bad_request(&c, e),
                            (_, Err(e)) => __bad_request(&c, e),
                        }
                    })
                }
            }

            impl<F, Fut, Item1:FromRequest, Item2:FromRequest> IntoHandler<(Context, $param_type, Item1, Item2)> for F
            where
                F:   Fn(Context, $param_type, Item1, Item2) -> Fut + Send + Sync + 'static,
                Fut: Future<Output = Response> + Send + Sync + 'static,
            {
                fn into_handler(self) -> Handler {
                    Handler::new(move |req, c, params| {
                        // SAFETY: Due to the architecture of `Router`,
                        // `params` has already `append`ed once before this code
                        let p1 = unsafe {params.assume_init_first().as_bytes()};

                        match (from_param_bytes(p1), Item1::parse(req), Item2::parse(req)) {
                            (Ok(p1), Ok(item1), Ok(item2)) => Box::pin(self(c, p1, item1, item2)),
                            (Err(e),_,_) => __bad_request(&c, e),
                            (_,Err(e),_) => __bad_request(&c, e),
                            (_,_,Err(e)) => __bad_request(&c, e),
                        }
                    })
                }
            }
        )*};
    } with_single_path_param_and_from_request_items! {
        String, u8, u16, u32, u64, u128, usize
    }
};

const _: (/* one FromParam and FromRequest items */) = {
    impl<F, Fut, P1:FromParam, Item1:FromRequest> IntoHandler<(Context, (P1,), Item1)> for F
        where
            F:   Fn(Context, (P1,), Item1) -> Fut + Send + Sync + 'static,
            Fut: Future<Output = Response> + Send + Sync + 'static,
        {
            fn into_handler(self) -> Handler {
                Handler::new(move |req, c, params| {
                    // SAFETY: Due to the architecture of `Router`,
                    // `params` has already `append`ed once before this code
                    let p1 = unsafe {params.assume_init_first().as_bytes()};

                    match (from_param_bytes(p1), Item1::parse(req)) {
                        (Ok(p1), Ok(item1)) => Box::pin(self(c, (p1,), item1)),
                        (Err(e),_) => __bad_request(&c, e),
                        (_,Err(e)) => __bad_request(&c, e),
                    }
                })
            }
        }

        impl<F, Fut, P1:FromParam, Item1:FromRequest, Item2:FromRequest> IntoHandler<(Context, (P1,), Item1, Item2)> for F
        where
            F:   Fn(Context, (P1,), Item1, Item2) -> Fut + Send + Sync + 'static,
            Fut: Future<Output = Response> + Send + Sync + 'static,
        {
            fn into_handler(self) -> Handler {
                Handler::new(move |req, c, params| {
                    // SAFETY: Due to the architecture of `Router`,
                    // `params` has already `append`ed once before this code
                    let p1 = unsafe {params.assume_init_first().as_bytes()};

                    match (from_param_bytes(p1), Item1::parse(req), Item2::parse(req)) {
                        (Ok(p1), Ok(item1), Ok(item2)) => Box::pin(self(c, (p1,), item1, item2)),
                        (Err(e),_,_) => __bad_request(&c, e),
                        (_,Err(e),_) => __bad_request(&c, e),
                        (_,_,Err(e)) => __bad_request(&c, e),
                    }
                })
            }
        }
};

const _: (/* two PathParams and FromRequest items */) = {
    impl<F, Fut, P1:FromParam, P2:FromParam, Item1:FromRequest> IntoHandler<(Context, (P1, P2), Item1)> for F
    where
        F:   Fn(Context, (P1, P2), Item1) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + Sync + 'static,
    {
        fn into_handler(self) -> Handler {
            Handler::new(move |req, c, params| {
                // SAFETY: Due to the architecture of `Router`,
                // `params` has already `append`ed twice before this code
                let (p1, p2) = params.assume_init_extract();
                let (p1, p2) = unsafe {(p1.as_bytes(), p2.as_bytes())};

                match (from_param_bytes(p1), from_param_bytes(p2), Item1::parse(req)) {
                    (Ok(p1), Ok(p2), Ok(item1)) => Box::pin(self(c, (p1, p2), item1)),
                    (Err(e),_,_) => __bad_request(&c, e),
                    (_,Err(e),_) => __bad_request(&c, e),
                    (_,_,Err(e)) => __bad_request(&c, e),
                }
            })
        }
    }

    impl<F, Fut, P1:FromParam, P2:FromParam, Item1:FromRequest, Item2:FromRequest> IntoHandler<(Context, (P1, P2), Item1, Item2)> for F
    where
        F:   Fn(Context, (P1, P2), Item1, Item2) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + Sync + 'static,
    {
        fn into_handler(self) -> Handler {
            Handler::new(move |req, c, params| {
                // SAFETY: Due to the architecture of `Router`,
                // `params` has already `append`ed twice before this code
                let (p1, p2) = params.assume_init_extract();
                let (p1, p2) = unsafe {(p1.as_bytes(), p2.as_bytes())};

                match (from_param_bytes(p1), from_param_bytes(p2), Item1::parse(req), Item2::parse(req)) {
                    (Ok(p1), Ok(p2), Ok(item1), Ok(item2)) => Box::pin(self(c, (p1, p2), item1, item2)),
                    (Err(e),_,_,_) => __bad_request(&c, e),
                    (_,Err(e),_,_) => __bad_request(&c, e),
                    (_,_,Err(e),_) => __bad_request(&c, e),
                    (_,_,_,Err(e)) => __bad_request(&c, e),
                }
            })
        }
    }
};

#[cfg(feature="websocket")]
const _: (/* requires upgrade to websocket */) = {
    impl<F, Fut> IntoHandler<(WebSocketContext,)> for F
    where
        F:   Fn(WebSocketContext) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + Sync + 'static,
    {
        fn into_handler(self) -> Handler {
            Handler::new(move |req, c, _| {
                match WebSocketContext::new(c, req) {
                    Ok(wsc)  => Box::pin(self(wsc)),
                    Err(res) => (|| Box::pin(async {res}))(),
                }
            }).requires_upgrade()
        }
    }

    impl<F, Fut, P1:FromParam> IntoHandler<(WebSocketContext, P1)> for F
    where
        F:   Fn(WebSocketContext, P1) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + Sync + 'static,
    {
        fn into_handler(self) -> Handler {
            Handler::new(move |req, c, params| {
                let p1 = unsafe {params.assume_init_first().as_bytes()};
                match from_param_bytes(p1) {
                    Ok(p1) => match WebSocketContext::new(c, req) {
                        Ok(wsc)  => Box::pin(self(wsc, p1)),
                        Err(res) => (|| Box::pin(async {res}))(),
                    }
                    Err(e) => __bad_request(&c, e),
                }
            }).requires_upgrade()
        }
    }
    impl<F, Fut, P1:FromParam, P2:FromParam> IntoHandler<(WebSocketContext, P1, P2)> for F
    where
        F:   Fn(WebSocketContext, P1, P2) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + Sync + 'static,
    {
        fn into_handler(self) -> Handler {
            Handler::new(move |req, c, params| {
                let (p1, p2) = params.assume_init_extract();
                let (p1, p2) = unsafe {(p1.as_bytes(), p2.as_bytes())};
                match (from_param_bytes(p1), from_param_bytes(p2)) {
                    (Ok(p1), Ok(p2)) => match WebSocketContext::new(c, req) {
                        Ok(wsc)  => Box::pin(self(wsc, p1, p2)),
                        Err(res) => (|| Box::pin(async {res}))(),
                    }
                    (Err(e),_)|(_,Err(e)) => __bad_request(&c, e),
                }
            }).requires_upgrade()
        }
    }
    impl<F, Fut, P1:FromParam> IntoHandler<(WebSocketContext, (P1,))> for F
    where
        F:   Fn(WebSocketContext, (P1,)) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + Sync + 'static,
    {
        fn into_handler(self) -> Handler {
            Handler::new(move |req, c, params| {
                let p1 = unsafe {params.assume_init_first().as_bytes()};
                match from_param_bytes(p1) {
                    Ok(p1) => match WebSocketContext::new(c, req) {
                        Ok(wsc)  => Box::pin(self(wsc, (p1,))),
                        Err(res) => (|| Box::pin(async {res}))(),
                    }
                    Err(e) => __bad_request(&c, e),
                }
            }).requires_upgrade()
        }
    }
    impl<F, Fut, P1:FromParam, P2:FromParam> IntoHandler<(WebSocketContext, (P1, P2))> for F
    where
        F:   Fn(WebSocketContext, (P1, P2)) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + Sync + 'static,
    {
        fn into_handler(self) -> Handler {
            Handler::new(move |req, c, params| {
                let (p1, p2) = params.assume_init_extract();
                let (p1, p2) = unsafe {(p1.as_bytes(), p2.as_bytes())};
                match (from_param_bytes(p1), from_param_bytes(p2)) {
                    (Ok(p1), Ok(p2)) => match WebSocketContext::new(c, req) {
                        Ok(wsc)  => Box::pin(self(wsc, (p1, p2))),
                        Err(res) => (|| Box::pin(async {res}))(),
                    }
                    (Err(e),_)|(_,Err(e)) => __bad_request(&c, e),
                }
            }).requires_upgrade()
        }
    }
};
