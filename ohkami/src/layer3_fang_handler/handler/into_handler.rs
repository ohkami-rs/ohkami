use std::future::Future;
use super::Handler;
use crate::{
    Context,
    Response,
    layer1_req_res::{FromRequest, FromBuffer as PathParam},
};


pub trait IntoHandler<Args> {
    fn into_handler(self) -> Handler;
}

const _: (/* only Context */) = {
    impl<F, Fut> IntoHandler<(Context,)> for F
    where
        F:   Fn(Context) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + Sync + 'static,
    {
        fn into_handler(self) -> Handler {
            Handler::new(false, move |_, c, _|
                Box::pin({
                    let res = self(c);
                    async {res.await}
                })
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
                    Handler::new(false, move |_, c, params|
                        match <$param_type as PathParam>::parse(unsafe {params.assume_init_first().as_bytes()}) {
                            Ok(p1) => Box::pin({
                                let res = self(c, p1);
                                async {res.await}
                            }),
                            Err(e) => Box::pin({
                                let res = c.BadRequest().text(e.to_string());
                                async {res}
                            }),
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
            Handler::new(false, move |_, c, params|
                // SAFETY: Due to the architecture of `Router`,
                // `params` has already `append`ed once before this code
                match <P1 as PathParam>::parse(unsafe {params.assume_init_first().as_bytes()}) {
                    Ok(p1) => Box::pin({
                        let res = self(c, (p1,));
                        async {res.await}
                    }),
                    Err(e) => Box::pin({
                        let res = c.BadRequest().text(e.to_string());
                        async {res}
                    }),
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
            Handler::new(false, move |_, c, params| {
                let (p1_range, p2_range) = params.assume_init_extract();
                // SAFETY: Due to the architecture of `Router`,
                // `params` has already `append`ed twice before this code
                match <P1 as PathParam>::parse(unsafe {p1_range.as_bytes()}) {
                    Ok(p1) => match <P2 as PathParam>::parse(unsafe {p2_range.as_bytes()}) {
                        Ok(p2) => Box::pin({
                            let res = self(c, (p1, p2));
                            async {res.await}
                        }),
                        Err(e) => Box::pin({
                            let res = c.BadRequest().text(e.to_string());
                            async {res}
                        })
                    }
                    Err(e) => Box::pin({
                        let res = c.BadRequest().text(e.to_string());
                        async {res}
                    }),
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
            Handler::new(false, move |req, c, _|
                match Item1::parse(&req) {
                    Ok(item1) => Box::pin({
                        let res = self(c, item1);
                        async {res.await}
                    }),
                    Err(e) => Box::pin({
                        let res = c.BadRequest().text(e.to_string());
                        async {res}
                    })
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
            Handler::new(false, move |req, c, _|
                match Item1::parse(&req) {
                    Ok(item1) => match Item2::parse(&req) {
                        Ok(item2) => Box::pin({
                            let res = self(c, item1, item2);
                            async {res.await}
                        }),
                        Err(e) => Box::pin({
                            let res = c.BadRequest().text(e.to_string());
                            async {res}
                        })
                    }
                    Err(e) => Box::pin({
                        let res = c.BadRequest().text(e.to_string());
                        async {res}
                    })
                }
            )
        }
    }

    impl<F, Fut, Item1:FromRequest, Item2:FromRequest, Item3:FromRequest> IntoHandler<(Context, Item1, Item2, Item3)> for F
    where
        F:   Fn(Context, Item1, Item2, Item3) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + Sync + 'static,
    {
        fn into_handler(self) -> Handler {
            Handler::new(false, move |req, c, _|
                match Item1::parse(&req) {
                    Ok(item1) => match Item2::parse(&req) {
                        Ok(item2) => match Item3::parse(&req) {
                            Ok(item3) => Box::pin({
                                let res = self(c, item1, item2, item3);
                                async {res.await}
                            }),
                            Err(e) => Box::pin({
                                let res = c.BadRequest().text(e.to_string());
                                async {res}
                            })
                        }
                        Err(e) => Box::pin({
                            let res = c.BadRequest().text(e.to_string());
                            async {res}
                        })
                    }
                    Err(e) => Box::pin({
                        let res = c.BadRequest().text(e.to_string());
                        async {res}
                    })
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
                    Handler::new(false, move |req, c, params|
                        // SAFETY: Due to the architecture of `Router`,
                        // `params` has already `append`ed once before this code
                        match <$param_type as PathParam>::parse(unsafe {params.assume_init_first().as_bytes()}) {
                            Ok(p1) => match Item1::parse(&req) {
                                Ok(item1) => Box::pin({
                                    let res = self(c, p1, item1);
                                    async {res.await}
                                }),
                                Err(e) => Box::pin({
                                    let res = c.BadRequest().text(e.to_string());
                                    async {res}
                                })
                            }
                            Err(e) => Box::pin({
                                let res = c.BadRequest().text(e.to_string());
                                async {res}
                            }),
                        }
                    )
                }
            }

            impl<F, Fut, Item1:FromRequest, Item2:FromRequest> IntoHandler<(Context, $param_type, Item1, Item2)> for F
            where
                F:   Fn(Context, $param_type, Item1, Item2) -> Fut + Send + Sync + 'static,
                Fut: Future<Output = Response> + Send + Sync + 'static,
            {
                fn into_handler(self) -> Handler {
                    Handler::new(false, move |req, c, params|
                        // SAFETY: Due to the architecture of `Router`,
                        // `params` has already `append`ed once before this code
                        match <$param_type as PathParam>::parse(unsafe {params.assume_init_first().as_bytes()}) {
                            Ok(p1) => match Item1::parse(&req) {
                                Ok(item1) => match Item2::parse(&req) {
                                    Ok(item2) => Box::pin({
                                        let res = self(c, p1, item1, item2);
                                        async {res.await}
                                    }),
                                    Err(e) => Box::pin({
                                        let res = c.BadRequest().text(e.to_string());
                                        async {res}
                                    })
                                }
                                Err(e) => Box::pin({
                                    let res = c.BadRequest().text(e.to_string());
                                    async {res}
                                })
                            }
                            Err(e) => Box::pin({
                                let res = c.BadRequest().text(e.to_string());
                                async {res}
                            }),
                        }
                    )
                }
            }

            impl<F, Fut, Item1:FromRequest, Item2:FromRequest, Item3:FromRequest> IntoHandler<(Context, $param_type, Item1, Item2, Item3)> for F
            where
                F:   Fn(Context, $param_type, Item1, Item2, Item3) -> Fut + Send + Sync + 'static,
                Fut: Future<Output = Response> + Send + Sync + 'static,
            {
                fn into_handler(self) -> Handler {
                    Handler::new(false, move |req, c, params|
                        // SAFETY: Due to the architecture of `Router`,
                        // `params` has already `append`ed once before this code
                        match <$param_type as PathParam>::parse(unsafe {params.assume_init_first().as_bytes()}) {
                            Ok(p1) => match Item1::parse(&req) {
                                Ok(item1) => match Item2::parse(&req) {
                                    Ok(item2) => match Item3::parse(&req) {
                                        Ok(item3) => Box::pin({
                                            let res = self(c, p1, item1, item2, item3);
                                            async {res.await}
                                        }),
                                        Err(e) => Box::pin({
                                            let res = c.BadRequest().text(e.to_string());
                                            async {res}
                                        })
                                    }
                                    Err(e) => Box::pin({
                                        let res = c.BadRequest().text(e.to_string());
                                        async {res}
                                    })
                                }
                                Err(e) => Box::pin({
                                    let res = c.BadRequest().text(e.to_string());
                                    async {res}
                                })
                            }
                            Err(e) => Box::pin({
                                let res = c.BadRequest().text(e.to_string());
                                async {res}
                            }),
                        }
                    )
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
                Handler::new(false, move |req, c, params|
                    // SAFETY: Due to the architecture of `Router`,
                    // `params` has already `append`ed once before this code
                    match P1::parse(unsafe {params.assume_init_first().as_bytes()}) {
                        Ok(p1) => match Item1::parse(&req) {
                            Ok(item1) => Box::pin({
                                let res = self(c, (p1,), item1);
                                async {res.await}
                            }),
                            Err(e) => Box::pin({
                                let res = c.BadRequest().text(e.to_string());
                                async {res}
                            })
                        }
                        Err(e) => Box::pin({
                            let res = c.BadRequest().text(e.to_string());
                            async {res}
                        }),
                    }
                )
            }
        }

        impl<F, Fut, P1:PathParam, Item1:FromRequest, Item2:FromRequest> IntoHandler<(Context, (P1,), Item1, Item2)> for F
        where
            F:   Fn(Context, (P1,), Item1, Item2) -> Fut + Send + Sync + 'static,
            Fut: Future<Output = Response> + Send + Sync + 'static,
        {
            fn into_handler(self) -> Handler {
                Handler::new(false, move |req, c, params|
                    // SAFETY: Due to the architecture of `Router`,
                    // `params` has already `append`ed once before this code
                    match P1::parse(unsafe {params.assume_init_first().as_bytes()}) {
                        Ok(p1) => match Item1::parse(&req) {
                            Ok(item1) => match Item2::parse(&req) {
                                Ok(item2) => Box::pin({
                                    let res = self(c, (p1,), item1, item2);
                                    async {res.await}
                                }),
                                Err(e) => Box::pin({
                                    let res = c.BadRequest().text(e.to_string());
                                    async {res}
                                })
                            }
                            Err(e) => Box::pin({
                                let res = c.BadRequest().text(e.to_string());
                                async {res}
                            })
                        }
                        Err(e) => Box::pin({
                            let res = c.BadRequest().text(e.to_string());
                            async {res}
                        }),
                    }
                )
            }
        }

        impl<F, Fut, P1:PathParam, Item1:FromRequest, Item2:FromRequest, Item3:FromRequest> IntoHandler<(Context, (P1,), Item1, Item2, Item3)> for F
        where
            F:   Fn(Context, (P1,), Item1, Item2, Item3) -> Fut + Send + Sync + 'static,
            Fut: Future<Output = Response> + Send + Sync + 'static,
        {
            fn into_handler(self) -> Handler {
                Handler::new(false, move |req, c, params|
                    // SAFETY: Due to the architecture of `Router`,
                    // `params` has already `append`ed once before this code
                    match P1::parse(unsafe {params.assume_init_first().as_bytes()}) {
                        Ok(p1) => match Item1::parse(&req) {
                            Ok(item1) => match Item2::parse(&req) {
                                Ok(item2) => match Item3::parse(&req) {
                                    Ok(item3) => Box::pin({
                                        let res = self(c, (p1,), item1, item2, item3);
                                        async {res.await}
                                    }),
                                    Err(e) => Box::pin({
                                        let res = c.BadRequest().text(e.to_string());
                                        async {res}
                                    })
                                }
                                Err(e) => Box::pin({
                                    let res = c.BadRequest().text(e.to_string());
                                    async {res}
                                })
                            }
                            Err(e) => Box::pin({
                                let res = c.BadRequest().text(e.to_string());
                                async {res}
                            })
                        }
                        Err(e) => Box::pin({
                            let res = c.BadRequest().text(e.to_string());
                            async {res}
                        }),
                    }
                )
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
            Handler::new(false, move |req, c, params| {
                let (p1_range, p2_range) = params.assume_init_extract();

                // SAFETY: Due to the architecture of `Router`,
                // `params` has already `append`ed twice before this code
                match <P1 as PathParam>::parse(unsafe {p1_range.as_bytes()}) {
                    Ok(p1) => match <P2 as PathParam>::parse(unsafe {p2_range.as_bytes()}) {
                        Ok(p2) => match Item1::parse(&req) {
                            Ok(item1) => Box::pin({
                                let res = self(c, (p1, p2), item1);
                                async {res.await}
                            }),
                            Err(e) => Box::pin({
                                let res = c.BadRequest().text(e.to_string());
                                async {res}
                            })
                        }
                        Err(e) => Box::pin({
                            let res = c.BadRequest().text(e.to_string());
                            async {res}
                        })
                    } 
                    Err(e) => Box::pin({
                        let res = c.BadRequest().text(e.to_string());
                        async {res}
                    }),
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
            Handler::new(false, move |req, c, params| {
                let (p1_range, p2_range) = params.assume_init_extract();

                // SAFETY: Due to the architecture of `Router`,
                // `params` has already `append`ed twice before this code
                match <P1 as PathParam>::parse(unsafe {p1_range.as_bytes()}) {
                    Ok(p1) => match <P2 as PathParam>::parse(unsafe {p2_range.as_bytes()}) {
                        Ok(p2) => match Item1::parse(&req) {
                            Ok(item1) => match Item2::parse(&req) {
                                Ok(item2) => Box::pin({
                                    let res = self(c, (p1, p2), item1, item2);
                                    async {res.await}
                                }),
                                Err(e) => Box::pin({
                                    let res = c.BadRequest().text(e.to_string());
                                    async {res}
                                })
                            }
                            Err(e) => Box::pin({
                                let res = c.BadRequest().text(e.to_string());
                                async {res}
                            })
                        }
                        Err(e) => Box::pin({
                            let res = c.BadRequest().text(e.to_string());
                            async {res}
                        })
                    } 
                    Err(e) => Box::pin({
                        let res = c.BadRequest().text(e.to_string());
                        async {res}
                    }),
                }
            })
        }
    }

    impl<F, Fut, P1:PathParam, P2:PathParam, Item1:FromRequest, Item2:FromRequest, Item3:FromRequest> IntoHandler<(Context, (P1, P2), Item1, Item2, Item3)> for F
    where
        F:   Fn(Context, (P1, P2), Item1, Item2, Item3) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + Sync + 'static,
    {
        fn into_handler(self) -> Handler {
            Handler::new(false, move |req, c, params| {
                let (p1_range, p2_range) = params.assume_init_extract();

                // SAFETY: Due to the architecture of `Router`,
                // `params` has already `append`ed twice before this code
                match <P1 as PathParam>::parse(unsafe {p1_range.as_bytes()}) {
                    Ok(p1) => match <P2 as PathParam>::parse(unsafe {p2_range.as_bytes()}) {
                        Ok(p2) => match Item1::parse(&req) {
                            Ok(item1) => match Item2::parse(&req) {
                                Ok(item2) => match Item3::parse(&req) {
                                    Ok(item3) => Box::pin({
                                        let res = self(c, (p1, p2), item1, item2, item3);
                                        async {res.await}
                                    }),
                                    Err(e) => Box::pin({
                                        let res = c.BadRequest().text(e.to_string());
                                        async {res}
                                    })
                                }
                                Err(e) => Box::pin({
                                    let res = c.BadRequest().text(e.to_string());
                                    async {res}
                                })
                            }
                            Err(e) => Box::pin({
                                let res = c.BadRequest().text(e.to_string());
                                async {res}
                            })
                        }
                        Err(e) => Box::pin({
                            let res = c.BadRequest().text(e.to_string());
                            async {res}
                        })
                    } 
                    Err(e) => Box::pin({
                        let res = c.BadRequest().text(e.to_string());
                        async {res}
                    }),
                }
            })
        }
    }
};
