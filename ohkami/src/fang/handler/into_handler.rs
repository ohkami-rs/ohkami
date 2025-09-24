use super::super::{BoxedFPC, Fang, middleware::Fangs};
use super::{Handler, SendOnThreaded, SendOnThreadedFuture, SendSyncOnThreaded};
use crate::{FromRequest, IntoResponse, Request, Response};
use std::{future::Future, pin::Pin};

#[cfg(feature = "openapi")]
use crate::openapi;

pub trait IntoHandler<T> {
    fn n_pathparams(&self) -> usize;
    fn into_handler(self) -> Handler;
}

#[cold]
#[inline(never)]
fn error_response(e: Response) -> Pin<Box<dyn SendOnThreadedFuture<Response>>> {
    Box::pin(async { e })
}

/* FIXME: omit unsafe... */
#[inline(always)]
fn from_request<'fr, 'req, R: FromRequest<'fr>>(req: &'req Request) -> Result<R, Response> {
    <R as FromRequest>::from_request(unsafe { std::mem::transmute::<&'req _, &'fr _>(req) })
        .ok_or_else(|| Response::BadRequest().with_text("missing something expected in request"))?
        .map_err(IntoResponse::into_response)
}

#[cfg(feature = "openapi")]
fn get_type_identifier<F>() -> &'static str {
    let type_name = std::any::type_name::<F>();

    let type_path = if type_name.ends_with('>') {
        /* `type_name` has generics like `playground::handler<alloc::string::String>` */
        /* ref: <https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=e02e32853dddf5385769d1718c481814> */
        let (type_path, _ /*generics*/) = type_name
            .rsplit_once('<')
            .expect("unexpectedly independent `>` in std::any::type_name");
        type_path
    } else {
        type_name
    };

    let (_ /*path from crate root*/, type_ident) = type_path
        .rsplit_once("::")
        .expect("unexpected format of std::any::type_name");
    type_ident
}

#[cfg(feature = "openapi")]
fn with_default_operation_id<F>(op: openapi::Operation) -> openapi::Operation {
    let type_ident = get_type_identifier::<F>();

    /* when like `Ohkami::new(("/".GET(|| async {"Hello, world!"}),))` */
    if type_ident == "{{closure}}" {
        op
    } else {
        op.operation_id(type_ident)
    }
}

/*
 * `IntoHandler` implementations for async functions
 */

impl<F, Fut, Res> IntoHandler<fn() -> Res> for F
where
    F: Fn() -> Fut + SendSyncOnThreaded + 'static,
    Res: IntoResponse,
    Fut: Future<Output = Res> + SendOnThreaded + 'static,
{
    fn n_pathparams(&self) -> usize {
        0
    }

    fn into_handler(self) -> Handler {
        Handler::new(
            move |_| {
                let res = self();
                Box::pin(async move { res.await.into_response() })
            },
            #[cfg(feature = "openapi")]
            {
                with_default_operation_id::<F>(openapi::Operation::with(Res::openapi_responses()))
            },
        )
    }
}

impl<'req, F, Fut, Res, Req1> IntoHandler<fn(Req1) -> Res> for F
where
    F: Fn(Req1) -> Fut + SendSyncOnThreaded + 'static,
    Res: IntoResponse,
    Fut: Future<Output = Res> + SendOnThreaded + 'static,
    Req1: FromRequest<'req>,
{
    fn n_pathparams(&self) -> usize {
        Req1::n_pathparams()
    }

    fn into_handler(self) -> Handler {
        Handler::new(
            move |req| match from_request::<Req1>(req) {
                Ok(req1) => {
                    let res = self(req1);
                    Box::pin(async move { res.await.into_response() })
                }
                Err(e) => error_response(e),
            },
            #[cfg(feature = "openapi")]
            {
                with_default_operation_id::<F>(openapi::Operation::with(Res::openapi_responses()))
                    .inbound(Req1::openapi_inbound())
            },
        )
    }
}

impl<'req, F, Fut, Res, Req1, Req2> IntoHandler<fn(Req1, Req2) -> Res> for F
where
    F: Fn(Req1, Req2) -> Fut + SendSyncOnThreaded + 'static,
    Res: IntoResponse,
    Fut: Future<Output = Res> + SendOnThreaded + 'static,
    Req1: FromRequest<'req>,
    Req2: FromRequest<'req>,
{
    fn n_pathparams(&self) -> usize {
        Req1::n_pathparams().max(Req2::n_pathparams())
    }

    fn into_handler(self) -> Handler {
        Handler::new(
            move |req| match (from_request::<Req1>(req), from_request::<Req2>(req)) {
                (Ok(req1), Ok(req2)) => {
                    let res = self(req1, req2);
                    Box::pin(async move { res.await.into_response() })
                }
                (Err(e), _) | (_, Err(e)) => error_response(e),
            },
            #[cfg(feature = "openapi")]
            {
                with_default_operation_id::<F>(openapi::Operation::with(Res::openapi_responses()))
                    .inbound(Req1::openapi_inbound())
                    .inbound(Req2::openapi_inbound())
            },
        )
    }
}

impl<'req, F, Fut, Res, Req1, Req2, Req3> IntoHandler<fn(Req1, Req2, Req3) -> Res> for F
where
    F: Fn(Req1, Req2, Req3) -> Fut + SendSyncOnThreaded + 'static,
    Res: IntoResponse,
    Fut: Future<Output = Res> + SendOnThreaded + 'static,
    Req1: FromRequest<'req>,
    Req2: FromRequest<'req>,
    Req3: FromRequest<'req>,
{
    fn n_pathparams(&self) -> usize {
        Req1::n_pathparams()
            .max(Req2::n_pathparams())
            .max(Req3::n_pathparams())
    }

    fn into_handler(self) -> Handler {
        Handler::new(
            move |req| match (
                from_request::<Req1>(req),
                from_request::<Req2>(req),
                from_request::<Req3>(req),
            ) {
                (Ok(req1), Ok(req2), Ok(req3)) => {
                    let res = self(req1, req2, req3);
                    Box::pin(async move { res.await.into_response() })
                }
                (Err(e), _, _) | (_, Err(e), _) | (_, _, Err(e)) => error_response(e),
            },
            #[cfg(feature = "openapi")]
            {
                with_default_operation_id::<F>(openapi::Operation::with(Res::openapi_responses()))
                    .inbound(Req1::openapi_inbound())
                    .inbound(Req2::openapi_inbound())
                    .inbound(Req3::openapi_inbound())
            },
        )
    }
}

impl<'req, F, Fut, Res, Req1, Req2, Req3, Req4> IntoHandler<fn(Req1, Req2, Req3, Req4) -> Res> for F
where
    F: Fn(Req1, Req2, Req3, Req4) -> Fut + SendSyncOnThreaded + 'static,
    Res: IntoResponse,
    Fut: Future<Output = Res> + SendOnThreaded + 'static,
    Req1: FromRequest<'req>,
    Req2: FromRequest<'req>,
    Req3: FromRequest<'req>,
    Req4: FromRequest<'req>,
{
    fn n_pathparams(&self) -> usize {
        Req1::n_pathparams()
            .max(Req2::n_pathparams())
            .max(Req3::n_pathparams())
            .max(Req4::n_pathparams())
    }

    fn into_handler(self) -> Handler {
        Handler::new(
            move |req| match (
                from_request::<Req1>(req),
                from_request::<Req2>(req),
                from_request::<Req3>(req),
                from_request::<Req4>(req),
            ) {
                (Ok(req1), Ok(req2), Ok(req3), Ok(req4)) => {
                    let res = self(req1, req2, req3, req4);
                    Box::pin(async move { res.await.into_response() })
                }
                (Err(e), _, _, _) | (_, Err(e), _, _) | (_, _, Err(e), _) | (_, _, _, Err(e)) => {
                    error_response(e)
                }
            },
            #[cfg(feature = "openapi")]
            {
                with_default_operation_id::<F>(openapi::Operation::with(Res::openapi_responses()))
                    .inbound(Req1::openapi_inbound())
                    .inbound(Req2::openapi_inbound())
                    .inbound(Req3::openapi_inbound())
                    .inbound(Req4::openapi_inbound())
            },
        )
    }
}

impl<'req, F, Fut, Body, Req1, Req2, Req3, Req4, Req5>
    IntoHandler<fn(Req1, Req2, Req3, Req4, Req5) -> Body> for F
where
    F: Fn(Req1, Req2, Req3, Req4, Req5) -> Fut + SendSyncOnThreaded + 'static,
    Body: IntoResponse,
    Fut: Future<Output = Body> + SendOnThreaded + 'static,
    Req1: FromRequest<'req>,
    Req2: FromRequest<'req>,
    Req3: FromRequest<'req>,
    Req4: FromRequest<'req>,
    Req5: FromRequest<'req>,
{
    fn n_pathparams(&self) -> usize {
        Req1::n_pathparams()
            .max(Req2::n_pathparams())
            .max(Req3::n_pathparams())
            .max(Req4::n_pathparams())
            .max(Req5::n_pathparams())
    }

    fn into_handler(self) -> Handler {
        Handler::new(
            move |req| match (
                from_request::<Req1>(req),
                from_request::<Req2>(req),
                from_request::<Req3>(req),
                from_request::<Req4>(req),
                from_request::<Req5>(req),
            ) {
                (Ok(req1), Ok(req2), Ok(req3), Ok(req4), Ok(req5)) => {
                    let res = self(req1, req2, req3, req4, req5);
                    Box::pin(async move { res.await.into_response() })
                }
                (Err(e), _, _, _, _)
                | (_, Err(e), _, _, _)
                | (_, _, Err(e), _, _)
                | (_, _, _, Err(e), _)
                | (_, _, _, _, Err(e)) => error_response(e),
            },
            #[cfg(feature = "openapi")]
            {
                with_default_operation_id::<F>(openapi::Operation::with(Body::openapi_responses()))
                    .inbound(Req1::openapi_inbound())
                    .inbound(Req2::openapi_inbound())
                    .inbound(Req3::openapi_inbound())
                    .inbound(Req4::openapi_inbound())
                    .inbound(Req5::openapi_inbound())
            },
        )
    }
}

impl<'req, F, Fut, Body, Req1, Req2, Req3, Req4, Req5, Req6>
    IntoHandler<fn(Req1, Req2, Req3, Req4, Req5, Req6) -> Body> for F
where
    F: Fn(Req1, Req2, Req3, Req4, Req5, Req6) -> Fut + SendSyncOnThreaded + 'static,
    Body: IntoResponse,
    Fut: Future<Output = Body> + SendOnThreaded + 'static,
    Req1: FromRequest<'req>,
    Req2: FromRequest<'req>,
    Req3: FromRequest<'req>,
    Req4: FromRequest<'req>,
    Req5: FromRequest<'req>,
    Req6: FromRequest<'req>,
{
    fn n_pathparams(&self) -> usize {
        Req1::n_pathparams()
            .max(Req2::n_pathparams())
            .max(Req3::n_pathparams())
            .max(Req4::n_pathparams())
            .max(Req5::n_pathparams())
            .max(Req6::n_pathparams())
    }

    fn into_handler(self) -> Handler {
        Handler::new(
            move |req| match (
                from_request::<Req1>(req),
                from_request::<Req2>(req),
                from_request::<Req3>(req),
                from_request::<Req4>(req),
                from_request::<Req5>(req),
                from_request::<Req6>(req),
            ) {
                (Ok(req1), Ok(req2), Ok(req3), Ok(req4), Ok(req5), Ok(req6)) => {
                    let res = self(req1, req2, req3, req4, req5, req6);
                    Box::pin(async move { res.await.into_response() })
                }
                (Err(e), _, _, _, _, _)
                | (_, Err(e), _, _, _, _)
                | (_, _, Err(e), _, _, _)
                | (_, _, _, Err(e), _, _)
                | (_, _, _, _, Err(e), _)
                | (_, _, _, _, _, Err(e)) => error_response(e),
            },
            #[cfg(feature = "openapi")]
            {
                with_default_operation_id::<F>(openapi::Operation::with(Body::openapi_responses()))
                    .inbound(Req1::openapi_inbound())
                    .inbound(Req2::openapi_inbound())
                    .inbound(Req3::openapi_inbound())
                    .inbound(Req4::openapi_inbound())
                    .inbound(Req5::openapi_inbound())
                    .inbound(Req6::openapi_inbound())
            },
        )
    }
}

/*
 * `IntoHandler` implementations for handlers with local fangs
 */

#[diagnostic::do_not_recommend]
impl<H: IntoHandler<T>, T, F1> IntoHandler<(F1, H, T)> for (F1, H)
where
    F1: Fang<BoxedFPC>,
{
    fn n_pathparams(&self) -> usize {
        self.1.n_pathparams()
    }

    fn into_handler(self) -> Handler {
        let (f, h) = self;
        let h = h.into_handler();
        Handler {
            proc: Fangs::build(&f, h.proc),
            #[cfg(feature = "openapi")]
            openapi_operation: Fangs::openapi_map_operation(&f, h.openapi_operation),
        }
    }
}

#[diagnostic::do_not_recommend]
impl<H: IntoHandler<T>, T, F1, F2> IntoHandler<(F1, F2, H, T)> for (F1, F2, H)
where
    F1: Fang<F2::Proc>,
    F2: Fang<BoxedFPC>,
{
    fn n_pathparams(&self) -> usize {
        self.2.n_pathparams()
    }

    fn into_handler(self) -> Handler {
        let (f1, f2, h) = self;
        let h = h.into_handler();
        let f = (f1, f2);
        Handler {
            proc: Fangs::build(&f, h.proc),
            #[cfg(feature = "openapi")]
            openapi_operation: Fangs::openapi_map_operation(&f, h.openapi_operation),
        }
    }
}

#[diagnostic::do_not_recommend]
impl<H: IntoHandler<T>, T, F1, F2, F3> IntoHandler<(F1, F2, F3, H, T)> for (F1, F2, F3, H)
where
    F1: Fang<F2::Proc>,
    F2: Fang<F3::Proc>,
    F3: Fang<BoxedFPC>,
{
    fn n_pathparams(&self) -> usize {
        self.3.n_pathparams()
    }

    fn into_handler(self) -> Handler {
        let (f1, f2, f3, h) = self;
        let h = h.into_handler();
        let f = (f1, f2, f3);
        Handler {
            proc: Fangs::build(&f, h.proc),
            #[cfg(feature = "openapi")]
            openapi_operation: Fangs::openapi_map_operation(&f, h.openapi_operation),
        }
    }
}

#[diagnostic::do_not_recommend]
impl<H: IntoHandler<T>, T, F1, F2, F3, F4> IntoHandler<(F1, F2, F3, F4, H, T)>
    for (F1, F2, F3, F4, H)
where
    F1: Fang<F2::Proc>,
    F2: Fang<F3::Proc>,
    F3: Fang<F4::Proc>,
    F4: Fang<BoxedFPC>,
{
    fn n_pathparams(&self) -> usize {
        self.4.n_pathparams()
    }

    fn into_handler(self) -> Handler {
        let (f1, f2, f3, f4, h) = self;
        let h = h.into_handler();
        let f = (f1, f2, f3, f4);
        Handler {
            proc: Fangs::build(&f, h.proc),
            #[cfg(feature = "openapi")]
            openapi_operation: Fangs::openapi_map_operation(&f, h.openapi_operation),
        }
    }
}

#[cfg(test)]
#[test]
fn handler_args() {
    use crate::claw::param::{FromParam, Path};

    async fn h0() -> &'static str {
        ""
    }
    async fn h1(Path(_param): Path<String>) -> Response {
        todo!()
    }
    async fn h2(Path(_param): Path<&str>) -> Response {
        todo!()
    }
    async fn h3(Path(_params): Path<(&str, u64)>) -> Response {
        todo!()
    }

    struct P1<'req>(std::marker::PhantomData<&'req ()>);
    #[cfg(feature = "openapi")]
    impl<'p> crate::openapi::Schema for P1<'p> {
        fn schema() -> impl Into<crate::openapi::SchemaRef> {
            crate::openapi::string()
        }
    }
    impl<'p> FromParam<'p> for P1<'p> {
        type Error = std::convert::Infallible;
        fn from_param(_param: std::borrow::Cow<'p, str>) -> Result<Self, Self::Error> {
            Ok(Self(Default::default()))
        }
    }
    async fn h4(Path(_param): Path<P1<'_>>) -> String {
        format!("")
    }

    /// https://github.com/ohkami-rs/ohkami/issues/550
    #[cfg(feature = "rt_glommio")]
    async fn echo_id(Path(id): Path<String>) -> String {
        let executor = glommio::executor();
        executor.spawn_blocking(move || id).await
    }

    #[cfg(feature = "rt_worker")]
    struct SomeJS {
        _ptr: *const u8,
    }
    #[cfg(feature = "rt_worker")]
    impl<'req> FromRequest<'req> for SomeJS {
        type Error = std::convert::Infallible;
        fn from_request(_: &'req Request) -> Option<Result<Self, Self::Error>> {
            None
        }
    }
    #[cfg(feature = "rt_worker")]
    async fn h5(_: SomeJS) -> String {
        format!("")
    }

    macro_rules! assert_handlers {
        ( $($function:ident)* ) => {
            $( let _ = IntoHandler::into_handler($function); )*
        };
    }

    assert_handlers! { h0 h1 h2 h3 h4 }

    #[cfg(feature = "rt_glommio")]
    assert_handlers! { echo_id }

    #[cfg(feature = "rt_worker")]
    assert_handlers! { h5 }
}

#[cfg(feature = "openapi")]
#[cfg(test)]
#[test]
fn test_get_type_ident() {
    assert_eq!(get_type_identifier::<String>(), "String");
    assert_eq!(get_type_identifier::<std::sync::Arc<String>>(), "Arc");
}
