use std::future::Future;
use super::Handler;
use crate::{
    Context,
    Response,
    layer1_req_res::{FromRequest, FromBuffer as PathParam},
};
#[cfg(feature="websocket")]
use crate::websocket::WebSocketContext;


pub trait IntoHandler<Args> {
    fn into_handler(self) -> Handler;
}

#[cold] fn __bad_request(
    c: &Context,
    e: std::borrow::Cow<'static, str>,
) -> std::pin::Pin<Box<impl Future<Output = Response>>> {
    Box::pin({
        let res = c.BadRequest().text(e.to_string());
        async {res}
    })
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

const _: (/* PathParam */) = {
    macro_rules! with_single_path_param {
        ($( $param_type:ty ),*) => {$(
            impl<F, Fut> IntoHandler<(Context, $param_type)> for F
            where
                F:   Fn(Context, $param_type) -> Fut + Send + Sync + 'static,
                Fut: Future<Output = Response> + Send + Sync + 'static,
            {
                fn into_handler(self) -> Handler {
                    Handler::new(move |_, c, params|
                        match <$param_type as PathParam>::parse(unsafe {params.assume_init_first().as_bytes()}) {
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

    impl<F, Fut, P1:PathParam> IntoHandler<(Context, (P1,))> for F
    where
        F:   Fn(Context, (P1,)) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + Sync + 'static,
    {
        fn into_handler(self) -> Handler {
            Handler::new(move |_, c, params|
                // SAFETY: Due to the architecture of `Router`,
                // `params` has already `append`ed once before this code
                match <P1 as PathParam>::parse(unsafe {params.assume_init_first().as_bytes()}) {
                    Ok(p1) => Box::pin(self(c, (p1,))),
                    Err(e) => __bad_request(&c, e)
                }
            )
        }
    }

    impl<F, Fut, P1:PathParam, P2:PathParam> IntoHandler<(Context, (P1, P2))> for F
    where
        F:   Fn(Context, (P1, P2)) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + Sync + 'static,
    {
        fn into_handler(self) -> Handler {
            Handler::new(move |_, c, params| {
                let (p1, p2) = params.assume_init_extract();
                let (p1, p2) = unsafe {(p1.as_bytes(), p2.as_bytes())};
                match (<P1 as PathParam>::parse(p1), <P2 as PathParam>::parse(p2)) {
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
                    (Ok(item1), Ok(item2))    => Box::pin(self(c, item1, item2)),
                    (Err(e), _) | (_, Err(e)) => __bad_request(&c, e),
                }
            )
        }
    }
};

const _: (/* single PathParam and FromRequest items */) = {
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

                        match (<$param_type as PathParam>::parse(p1), Item1::parse(req)) {
                            (Ok(p1), Ok(item1))       => Box::pin(self(c, p1, item1)),
                            (Err(e), _) | (_, Err(e)) => __bad_request(&c, e),
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

                        match (<$param_type as PathParam>::parse(p1), Item1::parse(req), Item2::parse(req)) {
                            (Ok(p1), Ok(item1), Ok(item2))         => Box::pin(self(c, p1, item1, item2)),
                            (Err(e),_,_)|(_,Err(e),_)|(_,_,Err(e)) => __bad_request(&c, e),
                        }
                    })
                }
            }
        )*};
    } with_single_path_param_and_from_request_items! {
        String, u8, u16, u32, u64, u128, usize
    }
};

const _: (/* one PathParam and FromRequest items */) = {
    impl<F, Fut, P1:PathParam, Item1:FromRequest> IntoHandler<(Context, (P1,), Item1)> for F
        where
            F:   Fn(Context, (P1,), Item1) -> Fut + Send + Sync + 'static,
            Fut: Future<Output = Response> + Send + Sync + 'static,
        {
            fn into_handler(self) -> Handler {
                Handler::new(move |req, c, params| {
                    // SAFETY: Due to the architecture of `Router`,
                    // `params` has already `append`ed once before this code
                    let p1 = unsafe {params.assume_init_first().as_bytes()};

                    match (P1::parse(p1), Item1::parse(req)) {
                        (Ok(p1), Ok(item1))   => Box::pin(self(c, (p1,), item1)),
                        (Err(e),_)|(_,Err(e)) => __bad_request(&c, e)
                    }
                })
            }
        }

        impl<F, Fut, P1:PathParam, Item1:FromRequest, Item2:FromRequest> IntoHandler<(Context, (P1,), Item1, Item2)> for F
        where
            F:   Fn(Context, (P1,), Item1, Item2) -> Fut + Send + Sync + 'static,
            Fut: Future<Output = Response> + Send + Sync + 'static,
        {
            fn into_handler(self) -> Handler {
                Handler::new(move |req, c, params| {
                    // SAFETY: Due to the architecture of `Router`,
                    // `params` has already `append`ed once before this code
                    let p1 = unsafe {params.assume_init_first().as_bytes()};

                    match (P1::parse(p1), Item1::parse(req), Item2::parse(req)) {
                        (Ok(p1), Ok(item1), Ok(item2))         => Box::pin(self(c, (p1,), item1, item2)),
                        (Err(e),_,_)|(_,Err(e),_)|(_,_,Err(e)) => __bad_request(&c, e),
                    }
                })
            }
        }
};

const _: (/* two PathParams and FromRequest items */) = {
    impl<F, Fut, P1:PathParam, P2:PathParam, Item1:FromRequest> IntoHandler<(Context, (P1, P2), Item1)> for F
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

                match (P1::parse(p1), P2::parse(p2), Item1::parse(req)) {
                    (Ok(p1), Ok(p2), Ok(item1))            => Box::pin(self(c, (p1, p2), item1)),
                    (Err(e),_,_)|(_,Err(e),_)|(_,_,Err(e)) => __bad_request(&c, e),
                }
            })
        }
    }

    impl<F, Fut, P1:PathParam, P2:PathParam, Item1:FromRequest, Item2:FromRequest> IntoHandler<(Context, (P1, P2), Item1, Item2)> for F
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

                match (P1::parse(p1), P2::parse(p2), Item1::parse(req), Item2::parse(req)) {
                    (Ok(p1), Ok(p2), Ok(item1), Ok(item2))                      => Box::pin(self(c, (p1, p2), item1, item2)),
                    (Err(e),_,_,_)|(_,Err(e),_,_)|(_,_,Err(e),_)|(_,_,_,Err(e)) => __bad_request(&c, e),
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

    impl<F, Fut, P1:PathParam> IntoHandler<(WebSocketContext, P1)> for F
    where
        F:   Fn(WebSocketContext, P1) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + Sync + 'static,
    {
        fn into_handler(self) -> Handler {
            Handler::new(move |req, c, params| {
                let p1 = unsafe {params.assume_init_first().as_bytes()};
                match P1::parse(p1) {
                    Ok(p1) => match WebSocketContext::new(c, req) {
                        Ok(wsc)  => Box::pin(self(wsc, p1)),
                        Err(res) => (|| Box::pin(async {res}))(),
                    }
                    Err(e) => __bad_request(&c, e),
                }
            }).requires_upgrade()
        }
    }
    impl<F, Fut, P1:PathParam, P2:PathParam> IntoHandler<(WebSocketContext, P1, P2)> for F
    where
        F:   Fn(WebSocketContext, P1, P2) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + Sync + 'static,
    {
        fn into_handler(self) -> Handler {
            Handler::new(move |req, c, params| {
                let (p1, p2) = params.assume_init_extract();
                let (p1, p2) = unsafe {(p1.as_bytes(), p2.as_bytes())};
                match (P1::parse(p1), P2::parse(p2)) {
                    (Ok(p1), Ok(p2)) => match WebSocketContext::new(c, req) {
                        Ok(wsc)  => Box::pin(self(wsc, p1, p2)),
                        Err(res) => (|| Box::pin(async {res}))(),
                    }
                    (Err(e),_)|(_,Err(e)) => __bad_request(&c, e),
                }
            }).requires_upgrade()
        }
    }
    impl<F, Fut, P1:PathParam> IntoHandler<(WebSocketContext, (P1,))> for F
    where
        F:   Fn(WebSocketContext, (P1,)) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + Sync + 'static,
    {
        fn into_handler(self) -> Handler {
            Handler::new(move |req, c, params| {
                let p1 = unsafe {params.assume_init_first().as_bytes()};
                match P1::parse(p1) {
                    Ok(p1) => match WebSocketContext::new(c, req) {
                        Ok(wsc)  => Box::pin(self(wsc, (p1,))),
                        Err(res) => (|| Box::pin(async {res}))(),
                    }
                    Err(e) => __bad_request(&c, e),
                }
            }).requires_upgrade()
        }
    }
    impl<F, Fut, P1:PathParam, P2:PathParam> IntoHandler<(WebSocketContext, (P1, P2))> for F
    where
        F:   Fn(WebSocketContext, (P1, P2)) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + Sync + 'static,
    {
        fn into_handler(self) -> Handler {
            Handler::new(move |req, c, params| {
                let (p1, p2) = params.assume_init_extract();
                let (p1, p2) = unsafe {(p1.as_bytes(), p2.as_bytes())};
                match (P1::parse(p1), P2::parse(p2)) {
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
