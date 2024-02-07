use std::{future::Future, borrow::Cow};
use ohkami_lib::percent_decode_utf8;
use super::Handler;
use crate::{
    Response, Status,
    FromRequest, FromParam, Request, IntoResponse,
};
#[cfg(feature="websocket")]
use crate::websocket::WebSocketContext;


pub trait IntoHandler<Args> {
    fn into_handler(self) -> Handler;
}

#[cold] #[inline(never)] fn __bad_request(
    e: impl std::fmt::Display,
) -> std::pin::Pin<Box<impl Future<Output = Response>>> {
    Box::pin({
        let res = Response::with(Status::BadRequest).text(e.to_string());
        async {res.into()}
    })
}
#[inline(always)] fn from_param_bytes<'p, P: FromParam<'p>>(
    param_bytes_maybe_percent_encoded: &'p [u8]
) -> Result<P, Cow<'static, str>> {
    let param = percent_decode_utf8(param_bytes_maybe_percent_encoded)
        .map_err(|e| Cow::Owned(e.to_string()))?;

    <P as FromParam>::from_param(param)
        .map_err(|e| Cow::Owned(e.to_string()))
}
#[inline(always)] fn from_request<'fr, 'req, R: FromRequest<'fr>>(
    req: &'req Request
) -> Result<R, <R as FromRequest<'fr>>::Error> {
    <R as FromRequest>::from_request(unsafe {
        std::mem::transmute::<&'req _, &'fr _>(req) //
    })
}


const _: () = {
    impl<F, Body, Fut> IntoHandler<fn()->Body> for F
    where
        F:    Fn() -> Fut + Send + Sync + 'static,
        Body: IntoResponse,
        Fut:  Future<Output = Body> + Send + 'static,
    {
        fn into_handler(self) -> Handler {
            Handler::new(move |_| {
                let res = self();
                Box::pin(async move {
                    res.await.into_response()
                })
            })
        }
    }
};

const _: (/* FromParam */) = {
    macro_rules! with_single_path_param {
        ($( $param_type:ty ),*) => {$(
            impl<'req, F, Body, Fut> IntoHandler<fn($param_type)->Body> for F
            where
                F:    Fn($param_type) -> Fut + Send + Sync + 'static,
                Body: IntoResponse,
                Fut:  Future<Output = Body> + Send + 'static,
            {
                fn into_handler(self) -> Handler {
                    Handler::new(move |req|
                        match from_param_bytes(unsafe {req.path.assume_one_param()}) {
                            Ok(p1) => {
                                let res = self(p1);
                                Box::pin(async move {res.await.into_response()})
                            }
                            Err(e) => __bad_request(e)
                        }
                    )
                }
            }
        )*};
    } with_single_path_param! {
        String, u8, u16, u32, u64, u128, usize
    }
    impl<'req, F, Body, Fut> IntoHandler<fn(&'req str)->Body> for F
    where
        F:    Fn(&'req str) -> Fut + Send + Sync + 'static,
        Body: IntoResponse,
        Fut:  Future<Output = Body> + Send + 'static,
    {
        fn into_handler(self) -> Handler {
            Handler::new(move |req|
                match from_param_bytes(unsafe {req.path.assume_one_param()}) {
                    Ok(p1) => {
                        let res = self(p1);
                        Box::pin(async move {res.await.into_response()})
                    },
                    Err(e) => __bad_request(e)
                }
            )
        }
    }
    #[cfg(test)] fn __() {
        async fn h1(_param: String) -> Response {todo!()}
        async fn h2(_param: &str) -> Response {todo!()}
    
        let _ = h1.into_handler();
        let _ = h2.into_handler();
    }

    impl<'req, F, Body, Fut, P1:FromParam<'req>> IntoHandler<fn((P1,))->Body> for F
    where
        F:    Fn((P1,)) -> Fut + Send + Sync + 'static,
        Body: IntoResponse,
        Fut:  Future<Output = Body> + Send + 'static,
    {
        fn into_handler(self) -> Handler {
            Handler::new(move |req|
                // SAFETY: Due to the architecture of `Router`,
                // `params` has already `append`ed once before this code
                match from_param_bytes(unsafe {req.path.assume_one_param()}) {
                    Ok(p1) => {
                        let res = self((p1,));
                        Box::pin(async move {res.await.into_response()})
                    }
                    Err(e) => __bad_request(e)
                }
            )
        }
    }

    impl<'req, F, Fut, Body:IntoResponse, P1:FromParam<'req>, P2:FromParam<'req>> IntoHandler<fn((P1, P2))->Body> for F
    where
        F:   Fn((P1, P2)) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Body> + Send + 'static,
    {
        fn into_handler(self) -> Handler {
            Handler::new(move |req| {
                let (p1, p2) = unsafe {req.path.assume_two_params()};
                match (from_param_bytes::<P1>(p1), from_param_bytes::<P2>(p2)) {
                    (Ok(p1), Ok(p2)) => {
                        let res = self((p1, p2));
                        Box::pin(async move {res.await.into_response()})
                    }
                    (Err(e), _) | (_, Err(e)) => __bad_request(e),
                }
            })
        }
    }
};

const _: (/* FromRequest items */) = {
    impl<'req, F, Fut, Body:IntoResponse, Item1:FromRequest<'req>> IntoHandler<fn(Item1)->Body> for F
    where
        F:   Fn(Item1) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Body> + Send + 'static,
    {
        fn into_handler(self) -> Handler {
            Handler::new(move |req|
                match from_request::<Item1>(req) {
                    Ok(item1) => {
                        let res = self(item1);
                        Box::pin(async move {res.await.into_response()})
                    }
                    Err(e)    => __bad_request(e)
                }
            )
        }
    }

    impl<'req, F, Fut, Body:IntoResponse, Item1:FromRequest<'req>, Item2:FromRequest<'req>> IntoHandler<fn(Item1, Item2)->Body> for F
    where
        F:   Fn(Item1, Item2) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Body> + Send + 'static,
    {
        fn into_handler(self) -> Handler {
            Handler::new(move |req|
                match (from_request::<Item1>(req), from_request::<Item2>(req)) {
                    (Ok(item1), Ok(item2)) => {
                        let res = self(item1, item2);
                        Box::pin(async move {res.await.into_response()})
                    }
                    (Err(e), _) => __bad_request(e),
                    (_, Err(e)) => __bad_request(e),
                }
            )
        }
    }

    impl<'req, F, Fut, Body:IntoResponse, Item1:FromRequest<'req>, Item2:FromRequest<'req>, Item3:FromRequest<'req>> IntoHandler<fn(Item1, Item2, Item3)->Body> for F
    where
        F:   Fn(Item1, Item2, Item3) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Body> + Send + 'static,
    {
        fn into_handler(self) -> Handler {
            Handler::new(move |req|
                match (from_request::<Item1>(req), from_request::<Item2>(req), from_request::<Item3>(req)) {
                    (Ok(item1), Ok(item2), Ok(item3)) => {
                        let res = self(item1, item2, item3);
                        Box::pin(async move {res.await.into_response()})
                    }
                    (Err(e), _, _) => __bad_request(e),
                    (_, Err(e), _) => __bad_request(e),
                    (_, _, Err(e)) => __bad_request(e),
                }
            )
        }
    }
};

const _: (/* single FromParam and FromRequest items */) = {
    macro_rules! with_single_path_param_and_from_request_items {
        ($( $param_type:ty ),*) => {$(
            impl<'req, F, Fut, Body:IntoResponse, Item1:FromRequest<'req>> IntoHandler<fn($param_type, Item1)->Body> for F
            where
                F:   Fn($param_type, Item1) -> Fut + Send + Sync + 'static,
                Fut: Future<Output = Body> + Send + 'static,
            {
                fn into_handler(self) -> Handler {
                    Handler::new(move |req| {
                        // SAFETY: Due to the architecture of `Router`,
                        // `params` has already `append`ed once before this code
                        let p1 = unsafe {req.path.assume_one_param()};

                        match (from_param_bytes(p1), from_request(req)) {
                            (Ok(p1), Ok(item1)) => {
                                let res = self(p1, item1);
                                Box::pin(async move {res.await.into_response()})
                            },
                            (Err(e), _) => __bad_request(e),
                            (_, Err(e)) => __bad_request(e),
                        }
                    })
                }
            }

            impl<'req, F, Fut, Body:IntoResponse, Item1:FromRequest<'req>, Item2:FromRequest<'req>> IntoHandler<fn($param_type, Item1, Item2)->Body> for F
            where
                F:   Fn($param_type, Item1, Item2) -> Fut + Send + Sync + 'static,
                Fut: Future<Output = Body> + Send + 'static,
            {
                fn into_handler(self) -> Handler {
                    Handler::new(move |req| {
                        // SAFETY: Due to the architecture of `Router`,
                        // `params` has already `append`ed once before this code
                        let p1 = unsafe {req.path.assume_one_param()};

                        match (from_param_bytes(p1), from_request::<Item1>(req), from_request::<Item2>(req)) {
                            (Ok(p1), Ok(item1), Ok(item2)) => {
                                let res = self(p1, item1, item2);
                                Box::pin(async move {res.await.into_response()})
                            }
                            (Err(e),_,_) => __bad_request(e),
                            (_,Err(e),_) => __bad_request(e),
                            (_,_,Err(e)) => __bad_request(e),
                        }
                    })
                }
            }

            impl<'req, F, Fut, Body:IntoResponse, Item1:FromRequest<'req>, Item2:FromRequest<'req>, Item3:FromRequest<'req>> IntoHandler<fn($param_type, Item1, Item2, Item3)->Body> for F
            where
                F:   Fn($param_type, Item1, Item2, Item3) -> Fut + Send + Sync + 'static,
                Fut: Future<Output = Body> + Send + 'static,
            {
                fn into_handler(self) -> Handler {
                    Handler::new(move |req| {
                        // SAFETY: Due to the architecture of `Router`,
                        // `params` has already `append`ed once before this code
                        let p1 = unsafe {req.path.assume_one_param()};

                        match (from_param_bytes(p1), from_request::<Item1>(req), from_request::<Item2>(req), from_request::<Item3>(req)) {
                            (Ok(p1), Ok(item1), Ok(item2), Ok(item3)) => {
                                let res = self(p1, item1, item2, item3);
                                Box::pin(async move {res.await.into_response()})
                            }
                            (Err(e),_,_,_) => __bad_request(e),
                            (_,Err(e),_,_) => __bad_request(e),
                            (_,_,Err(e),_) => __bad_request(e),
                            (_,_,_,Err(e)) => __bad_request(e),
                        }
                    })
                }
            }
        )*};
    } with_single_path_param_and_from_request_items! {
        String, &'req str, u8, u16, u32, u64, u128, usize
    }
};

const _: (/* one FromParam and FromRequest items */) = {
    impl<'req, F, Fut, Body:IntoResponse, P1:FromParam<'req>, Item1:FromRequest<'req>> IntoHandler<fn((P1,), Item1)->Body> for F
    where
        F:   Fn((P1,), Item1) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Body> + Send + 'static,
    {
        fn into_handler(self) -> Handler {
            Handler::new(move |req| {
                // SAFETY: Due to the architecture of `Router`,
                // `params` has already `append`ed once before this code
                let p1 = unsafe {req.path.assume_one_param()};

                match (from_param_bytes(p1), from_request::<Item1>(req)) {
                    (Ok(p1), Ok(item1)) => {
                        let res = self((p1,), item1);
                        Box::pin(async move {res.await.into_response()})
                    }
                    (Err(e),_) => __bad_request(e),
                    (_,Err(e)) => __bad_request(e),
                }
            })
        }
    }

    impl<'req, F, Fut, Body:IntoResponse, P1:FromParam<'req>, Item1:FromRequest<'req>, Item2:FromRequest<'req>> IntoHandler<fn((P1,), Item1, Item2)->Body> for F
    where
        F:   Fn((P1,), Item1, Item2) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Body> + Send + 'static,
    {
        fn into_handler(self) -> Handler {
            Handler::new(move |req| {
                // SAFETY: Due to the architecture of `Router`,
                // `params` has already `append`ed once before this code
                let p1 = unsafe {req.path.assume_one_param()};

                match (from_param_bytes(p1), from_request::<Item1>(req), from_request::<Item2>(req)) {
                    (Ok(p1), Ok(item1), Ok(item2)) => {
                        let res = self((p1,), item1, item2);
                        Box::pin(async move {res.await.into_response()})
                    }
                    (Err(e),_,_) => __bad_request(e),
                    (_,Err(e),_) => __bad_request(e),
                    (_,_,Err(e)) => __bad_request(e),
                }
            })
        }
    }

    impl<'req, F, Fut, Body:IntoResponse, P1:FromParam<'req>, Item1:FromRequest<'req>, Item2:FromRequest<'req>, Item3:FromRequest<'req>> IntoHandler<fn((P1,), Item1, Item2, Item3)->Body> for F
    where
        F:   Fn((P1,), Item1, Item2, Item3) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Body> + Send + 'static,
    {
        fn into_handler(self) -> Handler {
            Handler::new(move |req| {
                // SAFETY: Due to the architecture of `Router`,
                // `params` has already `append`ed once before this code
                let p1 = unsafe {req.path.assume_one_param()};
                
                match (from_param_bytes(p1), from_request::<Item1>(req), from_request::<Item2>(req), from_request::<Item3>(req)) {
                    (Ok(p1), Ok(item1), Ok(item2), Ok(item3)) => {
                        let res = self((p1,), item1, item2, item3);
                        Box::pin(async move {res.await.into_response()})
                    }
                    (Err(e),_,_,_) => __bad_request(e),
                    (_,Err(e),_,_) => __bad_request(e),
                    (_,_,Err(e),_) => __bad_request(e),
                    (_,_,_,Err(e)) => __bad_request(e),
                }
            })
        }
    }
};

const _: (/* two PathParams and FromRequest items */) = {
    impl<'req, F, Fut, Body:IntoResponse, P1:FromParam<'req>, P2:FromParam<'req>, Item1:FromRequest<'req>> IntoHandler<fn((P1, P2), Item1)->Body> for F
    where
        F:   Fn((P1, P2), Item1) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Body> + Send + 'static,
    {
        fn into_handler(self) -> Handler {
            Handler::new(move |req| {
                // SAFETY: Due to the architecture of `Router`,
                // `params` has already `append`ed twice before this code
                let (p1, p2) = unsafe {req.path.assume_two_params()};

                match (from_param_bytes(p1), from_param_bytes(p2), from_request::<Item1>(req)) {
                    (Ok(p1), Ok(p2), Ok(item1)) => {
                        let res = self((p1, p2), item1); 
                        Box::pin(async move {res.await.into_response()})
                    }
                    (Err(e),_,_) => __bad_request(e),
                    (_,Err(e),_) => __bad_request(e),
                    (_,_,Err(e)) => __bad_request(e),
                }
            })
        }
    }

    impl<'req, F, Fut, Body:IntoResponse, P1:FromParam<'req>, P2:FromParam<'req>, Item1:FromRequest<'req>, Item2:FromRequest<'req>> IntoHandler<fn((P1, P2), Item1, Item2)->Body> for F
    where
        F:   Fn((P1, P2), Item1, Item2) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Body> + Send + 'static,
    {
        fn into_handler(self) -> Handler {
            Handler::new(move |req| {
                // SAFETY: Due to the architecture of `Router`,
                // `params` has already `append`ed twice before this code
                let (p1, p2) = unsafe {req.path.assume_two_params()};

                match (from_param_bytes(p1), from_param_bytes(p2), from_request::<Item1>(req), from_request::<Item2>(req)) {
                    (Ok(p1), Ok(p2), Ok(item1), Ok(item2)) => {
                        let res = self((p1, p2), item1, item2);
                        Box::pin(async move {res.await.into_response()})
                    }
                    (Err(e),_,_,_) => __bad_request(e),
                    (_,Err(e),_,_) => __bad_request(e),
                    (_,_,Err(e),_) => __bad_request(e),
                    (_,_,_,Err(e)) => __bad_request(e),
                }
            })
        }
    }

    impl<'req, F, Fut, Body:IntoResponse, P1:FromParam<'req>, P2:FromParam<'req>, Item1:FromRequest<'req>, Item2:FromRequest<'req>, Item3:FromRequest<'req>> IntoHandler<fn((P1, P2), Item1, Item2, Item3)->Body> for F
    where
        F:   Fn((P1, P2), Item1, Item2, Item3) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Body> + Send + 'static,
    {
        fn into_handler(self) -> Handler {
            Handler::new(move |req| {
                // SAFETY: Due to the architecture of `Router`,
                // `params` has already `append`ed twice before this code
                let (p1, p2) = unsafe {req.path.assume_two_params()};

                match (from_param_bytes(p1), from_param_bytes(p2), from_request::<Item1>(req), from_request::<Item2>(req), from_request::<Item3>(req)) {
                    (Ok(p1), Ok(p2), Ok(item1), Ok(item2), Ok(item3)) => {
                        let res = self((p1, p2), item1, item2, item3);
                        Box::pin(async move {res.await.into_response()})
                    }
                    (Err(e),_,_,_,_) => __bad_request(e),
                    (_,Err(e),_,_,_) => __bad_request(e),
                    (_,_,Err(e),_,_) => __bad_request(e),
                    (_,_,_,Err(e),_) => __bad_request(e),
                    (_,_,_,_,Err(e)) => __bad_request(e),
                }
            })
        }
    }
};

#[cfg(feature="websocket")]
const _: (/* requires upgrade to websocket */) = {
    impl<'req, F, Fut, Body:IntoResponse> IntoHandler<fn(WebSocketContext,)->Body> for F
    where
        F:   Fn(WebSocketContext) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Body> + Send + 'static,
    {
        fn into_handler(self) -> Handler {
            Handler::new(move |req| {
                match WebSocketContext::new(req) {
                    Ok(wsc)  => {
                        let res = self(wsc);
                        Box::pin(async move {res.await.into_response()})
                    }
                    Err(res) => (|| Box::pin(async {res}))(),
                }
            }).requires_upgrade()
        }
    }

    impl<'req, F, Fut, Body:IntoResponse, P1:FromParam<'req>> IntoHandler<fn(WebSocketContext, P1)->Body> for F
    where
        F:   Fn(WebSocketContext, P1) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Body> + Send + 'static,
    {
        fn into_handler(self) -> Handler {
            Handler::new(move |req| {
                let p1 = unsafe {req.path.assume_one_param()};
                match from_param_bytes(p1) {
                    Ok(p1) => match WebSocketContext::new(req) {
                        Ok(wsc)  => {
                            let res = self(wsc, p1);
                            Box::pin(async move {res.await.into_response()})
                        }
                        Err(res) => (|| Box::pin(async {res}))(),
                    }
                    Err(e) => __bad_request(e),
                }
            }).requires_upgrade()
        }
    }
    impl<'req, F, Fut, Body:IntoResponse, P1:FromParam<'req>, P2:FromParam<'req>> IntoHandler<fn(WebSocketContext, P1, P2)->Body> for F
    where
        F:   Fn(WebSocketContext, P1, P2) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Body> + Send + 'static,
    {
        fn into_handler(self) -> Handler {
            Handler::new(move |req| {
                let (p1, p2) = unsafe {req.path.assume_two_params()};
                match (from_param_bytes(p1), from_param_bytes(p2)) {
                    (Ok(p1), Ok(p2)) => match WebSocketContext::new(req) {
                        Ok(wsc)  => {
                            let res = self(wsc, p1, p2);
                            Box::pin(async move {res.await.into_response()})
                        }
                        Err(res) => (|| Box::pin(async {res}))(),
                    }
                    (Err(e),_)|(_,Err(e)) => __bad_request(e),
                }
            }).requires_upgrade()
        }
    }
    impl<'req, F, Fut, Body:IntoResponse, P1:FromParam<'req>> IntoHandler<fn(WebSocketContext, (P1,))->Body> for F
    where
        F:   Fn(WebSocketContext, (P1,)) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Body> + Send + 'static,
    {
        fn into_handler(self) -> Handler {
            Handler::new(move |req| {
                let p1 = unsafe {req.path.assume_one_param()};
                match from_param_bytes(p1) {
                    Ok(p1) => match WebSocketContext::new(req) {
                        Ok(wsc)  => {
                            let res = self(wsc, (p1,));
                            Box::pin(async move {res.await.into_response()})
                        }
                        Err(res) => (|| Box::pin(async {res}))(),
                    }
                    Err(e) => __bad_request(e),
                }
            }).requires_upgrade()
        }
    }
    impl<'req, F, Fut, Body:IntoResponse, P1:FromParam<'req>, P2:FromParam<'req>> IntoHandler<fn(WebSocketContext, (P1, P2))->Body> for F
    where
        F:   Fn(WebSocketContext, (P1, P2)) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Body> + Send + 'static,
    {
        fn into_handler(self) -> Handler {
            Handler::new(move |req| {
                let (p1, p2) = unsafe {req.path.assume_two_params()};
                match (from_param_bytes(p1), from_param_bytes(p2)) {
                    (Ok(p1), Ok(p2)) => match WebSocketContext::new(req) {
                        Ok(wsc)  => {
                            let res = self(wsc, (p1, p2));
                            Box::pin(async move {res.await.into_response()})
                        }
                        Err(res) => (|| Box::pin(async {res}))(),
                    }
                    (Err(e),_)|(_,Err(e)) => __bad_request(e),
                }
            }).requires_upgrade()
        }
    }
};
