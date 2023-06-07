mod handlers;

use serde::{Serialize};
use std::{pin::Pin, future::Future, marker::PhantomData};
use crate::{
    Context, Request,
    layer0_lib::{List, BufRange},
    layer1_req_res::{Response, PathParam, Queries, Payload},
};

pub(crate) const PATH_PARAMS_LIMIT: usize = 2;
type PathParams = List<BufRange, PATH_PARAMS_LIMIT>;


pub struct Handler(
    Box<dyn
        Fn(Request, Context, PathParams) -> Pin<
            Box<dyn
                Future<Output = Response<()>>
                + Send + 'static
            >
        > + Send + Sync + 'static
    >
); const _: () = {
    impl<T:Serialize> Response<T> {
        pub(crate) fn into_unit(self) -> Response<()> {
            match self {
                Self::Ok(s, _) => Response::Ok(s, PhantomData),
                Self::Err(err) => Response::Err(err),
            }
        }
    }

    impl Fn<(Request, Context, PathParams)> for Handler {
        extern "rust-call" fn call(&self, (req, c, params): (Request, Context, PathParams)) -> Self::Output {
            self.0(req, c, params)
        }
    } const _: (/* with */) = {
        impl FnMut<(Request, Context, PathParams)> for Handler {
            extern "rust-call" fn call_mut(&mut self, (req, c, params): (Request, Context, PathParams)) -> Self::Output {
                self.0(req, c, params)
            }
        }
        impl FnOnce<(Request, Context, PathParams)> for Handler {
            type Output = Pin<
                Box<dyn
                    Future<Output = Response<()>>
                    + Send + 'static
                >
            >;
            extern "rust-call" fn call_once(self, (req, c, params): (Request, Context, PathParams)) -> Self::Output {
                self.0(req, c, params)
            }
        }
    };
};


pub trait IntoHandler<Args, T:Serialize> {
    fn into_handler(self) -> Handler;
}

const _: (/* only Context */) = {
    impl<F, Fut, T> IntoHandler<(Context,), ()> for F
    where
        F:   Fn(Context) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response<T>> + Send + Sync + 'static,
        T:   serde::Serialize + Send,
    {
        fn into_handler(self) -> Handler {
            Handler(Box::new(move |_, c, _|
                Box::pin({
                    let res = self(c);
                    async {res.await.into_unit()}
                })
            ))
        }
    }
};

const _: (/* PathParam */) = {
    macro_rules! with_single_path_param {
        ($( $param_type:ty ),*) => {$(
            impl<F, Fut, T> IntoHandler<(Context, $param_type), ()> for F
            where
                F:   Fn(Context, $param_type) -> Fut + Send + Sync + 'static,
                Fut: Future<Output = Response<T>> + Send + Sync + 'static,
                T:   serde::Serialize + Send + 'static,
            {
                fn into_handler(self) -> Handler {
                    Handler(Box::new(move |req, c, params|
                        match <$param_type as PathParam>::parse(&req.buffer[unsafe {params.list[0].assume_init_ref()}]) {
                            Ok(p1) => Box::pin({
                                let res = self(c, p1);
                                async {res.await.into_unit()}
                            }),
                            Err(e) => Box::pin({
                                let res = Response::Err(c.BadRequest().text(e.to_string()));
                                async {res}
                            }),
                        }
                    ))
                }
            }
        )*};
    } with_single_path_param! {
        &str, String, u8, u16, u32, u64, u128, usize
    }

    impl<F, Fut, T, P1:PathParam> IntoHandler<(Context, (P1,)), ()> for F
    where
        F:   Fn(Context, (P1,)) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response<T>> + Send + Sync + 'static,
        T:   serde::Serialize + Send + 'static,
    {
        fn into_handler(self) -> Handler {
            Handler(Box::new(move |req, c, params|
                // SAFETY: Due to the architecture of `Router`,
                // `params` has already `append`ed once before this code
                match <P1 as PathParam>::parse(&req.buffer[unsafe {params.list[0].assume_init_ref()}]) {
                    Ok(p1) => Box::pin({
                        let res = self(c, (p1,));
                        async {res.await.into_unit()}
                    }),
                    Err(e) => Box::pin({
                        let res = Response::Err(c.BadRequest().text(e.to_string()));
                        async {res}
                    }),
                }
            ))
        }
    }

    impl<F, Fut, T, P1:PathParam, P2:PathParam> IntoHandler<(Context, (P1, P2)), ()> for F
    where
        F:   Fn(Context, (P1, P2)) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response<T>> + Send + Sync + 'static,
        T:   serde::Serialize + Send + 'static,
    {
        fn into_handler(self) -> Handler {
            Handler(Box::new(move |req, c, params|
                // SAFETY: Due to the architecture of `Router`,
                // `params` has already `append`ed twice before this code
                match <P1 as PathParam>::parse(&req.buffer[unsafe {params.list[0].assume_init_ref()}]) {
                    Ok(p1) => match <P2 as PathParam>::parse(&req.buffer[unsafe {params.list[1].assume_init_ref()}]) {
                        Ok(p2) => Box::pin({
                            let res = self(c, (p1, p2));
                            async {res.await.into_unit()}
                        }),
                        Err(e) => Box::pin({
                            let res = Response::Err(c.BadRequest().text(e.to_string()));
                            async {res}
                        })
                    }
                    Err(e) => Box::pin({
                        let res = Response::Err(c.BadRequest().text(e.to_string()));
                        async {res}
                    }),
                }
            ))
        }
    }
};

const _: (/* Queries */) = {
    impl<F, Fut, T, Q:Queries> IntoHandler<(Context, Q), ()> for F
    where
        F:   Fn(Context, Q) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response<T>> + Send + Sync + 'static,
        T:   serde::Serialize + Send + 'static,
    {
        fn into_handler(self) -> Handler {
            Handler(Box::new(move |req, c, _|
                match <Q as Queries>::parse(&req) {
                    Ok(q) => Box::pin({
                        let res = self(c, q);
                        async {res.await.into_unit()}
                    }),
                    Err(e) => Box::pin({
                        let res = Response::Err(c.BadRequest().text(e.to_string()));
                        async {res}
                    })
                }
            ))
        }
    }
};

const _: (/* PathParam and Queries */) = {
    macro_rules! with_single_path_param_and_queries {
        ($( $param_type:ty ),*) => {$(
            impl<F, Fut, T, Q:Queries> IntoHandler<(Context, $param_type, Q), ()> for F
            where
                F:   Fn(Context, $param_type, Q) -> Fut + Send + Sync + 'static,
                Fut: Future<Output = Response<T>> + Send + Sync + 'static,
                T:   serde::Serialize + Send + 'static,
            {
                fn into_handler(self) -> Handler {
                    Handler(Box::new(move |req, c, params|
                        // SAFETY: Due to the architecture of `Router`,
                        // `params` has already `append`ed once before this code
                        match <$param_type as PathParam>::parse(&req.buffer[unsafe {params.list[0].assume_init_ref()}]) {
                            Ok(p1) => match <Q as Queries>::parse(&req) {
                                Ok(q) => Box::pin({
                                    let res = self(c, p1, q);
                                    async {res.await.into_unit()}
                                }),
                                Err(e) => Box::pin({
                                    let res = Response::Err(c.BadRequest().text(e.to_string()));
                                    async {res}
                                })
                            }
                            Err(e) => Box::pin({
                                let res = Response::Err(c.BadRequest().text(e.to_string()));
                                async {res}
                            }),
                        }
                    ))
                }
            }
        )*};
    } with_single_path_param_and_queries! {
        &str, String, u8, u16, u32, u64, u128, usize
    }

    impl<F, Fut, T, P1:PathParam, Q:Queries> IntoHandler<(Context, (P1,), Q), ()> for F
    where
        F:   Fn(Context, (P1,), Q) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response<T>> + Send + Sync + 'static,
        T:   serde::Serialize + Send + 'static,
    {
        fn into_handler(self) -> Handler {
            Handler(Box::new(move |req, c, params|
                // SAFETY: Due to the architecture of `Router`,
                // `params` has already `append`ed once before this code
                match <P1 as PathParam>::parse(&req.buffer[unsafe {params.list[0].assume_init_ref()}]) {
                    Ok(p1) => match <Q as Queries>::parse(&req) {
                        Ok(q) => Box::pin({
                            let res = self(c, (p1,), q);
                            async {res.await.into_unit()} 
                        }),
                        Err(e) => Box::pin({
                            let res = Response::Err(c.BadRequest().text(e.to_string()));
                            async {res}
                        })
                    } 
                    Err(e) => Box::pin({
                        let res = Response::Err(c.BadRequest().text(e.to_string()));
                        async {res}
                    }),
                }
            ))
        }
    }

    impl<F, Fut, T, P1:PathParam, P2:PathParam, Q:Queries> IntoHandler<(Context, (P1, P2), Q), ()> for F
    where
        F:   Fn(Context, (P1, P2), Q) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response<T>> + Send + Sync + 'static,
        T:   serde::Serialize + Send + 'static,
    {
        fn into_handler(self) -> Handler {
            Handler(Box::new(move |req, c, params|
                // SAFETY: Due to the architecture of `Router`,
                // `params` has already `append`ed twice before this code
                match <P1 as PathParam>::parse(&req.buffer[unsafe {params.list[0].assume_init_ref()}]) {
                    Ok(p1) => match <P2 as PathParam>::parse(&req.buffer[unsafe {params.list[1].assume_init_ref()}]) {
                        Ok(p2) => match <Q as Queries>::parse(&req) {
                            Ok(q) => Box::pin({
                                let res = self(c, (p1, p2), q);
                                async {res.await.into_unit()}
                            }),
                            Err(e) => Box::pin({
                                let res = Response::Err(c.BadRequest().text(e.to_string()));
                                async {res}
                            })
                        }
                        Err(e) => Box::pin({
                            let res = Response::Err(c.BadRequest().text(e.to_string()));
                            async {res}
                        })
                    } 
                    Err(e) => Box::pin({
                        let res = Response::Err(c.BadRequest().text(e.to_string()));
                        async {res}
                    }),
                }
            ))
        }
    }
};

const _: (/* Payload */) = {
    impl<F, Fut, T, P:Payload> IntoHandler<(Context, ((P,),)), ()> for F
    where
        F:   Fn(Context, P) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response<T>> + Send + Sync + 'static,
        T:   serde::Serialize + Send + 'static,
    {
        fn into_handler(self) -> Handler {
            Handler(Box::new(move |req, c, _|
                match <P as Payload>::parse(&req) {
                    Ok(p) => Box::pin({
                        let res = self(c, p);
                        async {res.await.into_unit()}
                    }),
                    Err(e) => Box::pin({
                        let res = Response::Err(c.BadRequest().text(e.to_string()));
                        async {res}
                    })
                }
            ))
        }
    }
};

const _: (/* Queries and Payload */) = {
    impl<F, Fut, T, Q:Queries, P:Payload> IntoHandler<(Context, Q, ((P,),)), ()> for F
    where
        F:   Fn(Context, Q, P) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response<T>> + Send + Sync + 'static,
        T:   serde::Serialize + Send + 'static,
    {
        fn into_handler(self) -> Handler {
            Handler(Box::new(move |req, c, _|
                match <Q as Queries>::parse(&req) {
                    Ok(q) => match <P as Payload>::parse(&req) {
                        Ok(p) => Box::pin({
                            let res = self(c, q, p);
                            async {res.await.into_unit()}
                        }),
                        Err(e) => Box::pin({
                            let res = Response::Err(c.BadRequest().text(e.to_string()));
                            async {res}
                        })
                    }
                    Err(e) => Box::pin({
                        let res = Response::Err(c.BadRequest().text(e.to_string()));
                        async {res}
                    })
                }
            ))
        }
    }
};
const _: (/* PathParam and Payload */) = {
    macro_rules! with_single_path_param_and_payload {
        ($( $param_type:ty ),*) => {$(
            impl<F, Fut, T, P:Payload> IntoHandler<(Context, $param_type, ((P,),)), ()> for F
            where
                F:   Fn(Context, $param_type, P) -> Fut + Send + Sync + 'static,
                Fut: Future<Output = Response<T>> + Send + Sync + 'static,
                T:   serde::Serialize + Send + 'static,
            {
                fn into_handler(self) -> Handler {
                    Handler(Box::new(move |req, c, params|
                        // SAFETY: Due to the architecture of `Router`,
                        // `params` has already `append`ed once before this code
                        match <$param_type as PathParam>::parse(&req.buffer[unsafe {params.list[0].assume_init_ref()}]) {
                            Ok(p1) => match <P as Payload>::parse(&req) {
                                Ok(p) => Box::pin({
                                    let res = self(c, p1, p);
                                    async {res.await.into_unit()}
                                }),
                                Err(e) => Box::pin({
                                    let res = Response::Err(c.BadRequest().text(e.to_string()));
                                    async {res}
                                })
                            } 
                            Err(e) => Box::pin({
                                let res = Response::Err(c.BadRequest().text(e.to_string()));
                                async {res}
                            }),
                        }
                    ))
                }
            }
        )*};
    } with_single_path_param_and_payload! {
        &str, String, u8, u16, u32, u64, u128, usize
    }

    impl<F, Fut, T, P1:PathParam, P:Payload> IntoHandler<(Context, (P1,), ((P,),)), ()> for F
    where
        F:   Fn(Context, (P1,), P) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response<T>> + Send + Sync + 'static,
        T:   serde::Serialize + Send + 'static,
    {
        fn into_handler(self) -> Handler {
            Handler(Box::new(move |req, c, params|
                // SAFETY: Due to the architecture of `Router`,
                // `params` has already `append`ed once before this code
                match <P1 as PathParam>::parse(&req.buffer[unsafe {params.list[0].assume_init_ref()}]) {
                    Ok(p1) => match <P as Payload>::parse(&req) {
                        Ok(p) => Box::pin({
                            let res = self(c, (p1,), p);
                            async {res.await.into_unit()}
                        }),
                        Err(e) => Box::pin({
                            let res = Response::Err(c.BadRequest().text(e.to_string()));
                            async {res}
                        })
                    } 
                    Err(e) => Box::pin({
                        let res = Response::Err(c.BadRequest().text(e.to_string()));
                        async {res}
                    }),
                }
            ))
        }
    }

    impl<F, Fut, T, P1:PathParam, P2:PathParam, P:Payload> IntoHandler<(Context, (P1, P2), ((P,),)), ()> for F
    where
        F:   Fn(Context, (P1, P2), P) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response<T>> + Send + Sync + 'static,
        T:   serde::Serialize + Send + 'static,
    {
        fn into_handler(self) -> Handler {
            Handler(Box::new(move |req, c, params|
                // SAFETY: Due to the architecture of `Router`,
                // `params` has already `append`ed twice before this code
                match <P1 as PathParam>::parse(&req.buffer[unsafe {params.list[0].assume_init_ref()}]) {
                    Ok(p1) => match <P2 as PathParam>::parse(&req.buffer[unsafe {params.list[1].assume_init_ref()}]) {
                        Ok(p2) => match <P as Payload>::parse(&req) {
                            Ok(p) => Box::pin({
                                let res = self(c, (p1, p2), p);
                                async {res.await.into_unit()}
                            }),
                            Err(e) => Box::pin({
                                let res = Response::Err(c.BadRequest().text(e.to_string()));
                                async {res}
                            })
                        }
                        Err(e) => Box::pin({
                            let res = Response::Err(c.BadRequest().text(e.to_string()));
                            async {res}
                        })
                    }
                    Err(e) => Box::pin({
                        let res = Response::Err(c.BadRequest().text(e.to_string()));
                        async {res}
                    }),
                }
            ))
        }
    }
};

const _: (/* PathParam and Queries and Payload */) = {
    macro_rules! with_single_path_param_and_queries_and_payload {
        ($( $param_type:ty ),*) => {$(
            impl<F, Fut, T, Q:Queries, P:Payload> IntoHandler<(Context, $param_type, Q, ((P,),)), ()> for F
            where
                F:   Fn(Context, $param_type, Q, P) -> Fut + Send + Sync + 'static,
                Fut: Future<Output = Response<T>> + Send + Sync + 'static,
                T:   serde::Serialize + Send + 'static,
            {
                fn into_handler(self) -> Handler {
                    Handler(Box::new(move |req, c, params|
                        // SAFETY: Due to the architecture of `Router`,
                        // `params` has already `append`ed once before this code
                        match <$param_type as PathParam>::parse(&req.buffer[unsafe {params.list[0].assume_init_ref()}]) {
                            Ok(p1) => match <Q as Queries>::parse(&req) {
                                Ok(q) => match <P as Payload>::parse(&req) {
                                    Ok(p) => Box::pin({
                                        let res = self(c, p1, q, p);
                                        async {res.await.into_unit()}
                                    }),
                                    Err(e) => Box::pin({
                                        let res = Response::Err(c.BadRequest().text(e.to_string()));
                                        async {res}
                                    })
                                }
                                Err(e) => Box::pin({
                                    let res = Response::Err(c.BadRequest().text(e.to_string()));
                                    async {res}
                                })
                            } 
                            Err(e) => Box::pin({
                                let res = Response::Err(c.BadRequest().text(e.to_string()));
                                async {res}
                            }),
                        }
                    ))
                }
            }
        )*};
    } with_single_path_param_and_queries_and_payload! {
        &str, String, u8, u16, u32, u64, u128, usize
    }

    impl<F, Fut, T, P1:PathParam, Q:Queries, P:Payload> IntoHandler<(Context, (P1,), Q, ((P,),)), ()> for F
    where
        F:   Fn(Context, (P1,), Q, P) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response<T>> + Send + Sync + 'static,
        T:   serde::Serialize + Send + 'static,
    {
        fn into_handler(self) -> Handler {
            Handler(Box::new(move |req, c, params|
                // SAFETY: Due to the architecture of `Router`,
                // `params` has already `append`ed once before this code
                match <P1 as PathParam>::parse(&req.buffer[unsafe {params.list[0].assume_init_ref()}]) {
                    Ok(p1) => match <Q as Queries>::parse(&req) {
                        Ok(q) => match <P as Payload>::parse(&req) {
                            Ok(p) => Box::pin({
                                let res = self(c, (p1,), q, p);
                                async {res.await.into_unit()}
                            }),
                            Err(e) => Box::pin({
                                let res = Response::Err(c.BadRequest().text(e.to_string()));
                                async {res}
                            })
                        }
                        Err(e) => Box::pin({
                            let res = Response::Err(c.BadRequest().text(e.to_string()));
                            async {res}
                        })
                    } 
                    Err(e) => Box::pin({
                        let res = Response::Err(c.BadRequest().text(e.to_string()));
                        async {res}
                    }),
                }
            ))
        }
    }

    impl<F, Fut, T, P1:PathParam, P2:PathParam, Q:Queries, P:Payload> IntoHandler<(Context, (P1,P2), Q, ((P,),)), ()> for F
    where
        F:   Fn(Context, (P1, P2), Q, P) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response<T>> + Send + Sync + 'static,
        T:   serde::Serialize + Send + 'static,
    {
        fn into_handler(self) -> Handler {
            Handler(Box::new(move |req, c, params|
                // SAFETY: Due to the architecture of `Router`,
                // `params` has already `append`ed twice before this code
                match <P1 as PathParam>::parse(&req.buffer[unsafe {params.list[0].assume_init_ref()}]) {
                    Ok(p1) => match <P2 as PathParam>::parse(&req.buffer[unsafe {params.list[1].assume_init_ref()}]) {
                        Ok(p2) => match <Q as Queries>::parse(&req) {
                            Ok(q) => match <P as Payload>::parse(&req) {
                                Ok(p) => Box::pin({
                                    let res = self(c, (p1, p2), q, p);
                                    async {res.await.into_unit()}
                                }),
                                Err(e) => Box::pin({
                                    let res = Response::Err(c.BadRequest().text(e.to_string()));
                                    async {res}
                                })
                            }
                            Err(e) => Box::pin({
                                let res = Response::Err(c.BadRequest().text(e.to_string()));
                                async {res}
                            })
                        }
                        Err(e) => Box::pin({
                            let res = Response::Err(c.BadRequest().text(e.to_string()));
                            async {res}
                        })
                    }
                    Err(e) => Box::pin({
                        let res = Response::Err(c.BadRequest().text(e.to_string()));
                        async {res}
                    }),
                }
            ))
        }
    }
};
