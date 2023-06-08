use std::future::Future;
use super::Fang;
use crate::{
    Context,
    Request,
};


pub trait IntoFang<Args> {
    fn into_fang(self) -> Fang;
}

const _: (/* Before */) = {
    const _: (/* only Context */) = {
        impl<F, Fut> IntoFang<(&Context,)> for F
        where
            F:   Fn(&Context) -> Fut + Send + Sync + 'static,
            Fut: Future<Output = ()> + Send + 'static,
        {
            fn into_fang(self) -> Fang {
                Fang::Before(Box::new(move |c, req|
                    Box::pin({
                        let out = self(&c);
                        async {out.await; (c, req)}
                    })
                ))
            }
        }

        impl<F, Fut> IntoFang<(&mut Context,)> for F
        where
            F:   Fn(&mut Context) -> Fut + Send + Sync + 'static,
            Fut: Future<Output = ()> + Send + 'static,
        {
            fn into_fang(self) -> Fang {
                Fang::Before(Box::new(move |mut c, req|
                    Box::pin({
                        let out = self(&mut c);
                        async {out.await; (c, req)}
                    })
                ))
            }
        }
    };

    const _: (/* with Request */) = {
        impl<F, Fut> IntoFang<(&Context, &Request)> for F
        where
            F:   Fn(&Context, &Request) -> Fut + Send + Sync + 'static,
            Fut: Future<Output = ()> + Send + 'static,
        {
            fn into_fang(self) -> Fang {
                Fang::Before(Box::new(move |c, req|
                    Box::pin({
                        let out = self(&c, &req);
                        async {out.await; (c, req)}
                    })
                ))
            }
        }

        impl<F, Fut> IntoFang<(&mut Context, &Request)> for F
        where
            F:   Fn(&mut Context, &Request) -> Fut + Send + Sync + 'static,
            Fut: Future<Output = ()> + Send + 'static,
        {
            fn into_fang(self) -> Fang {
                Fang::Before(Box::new(move |mut c, req|
                    Box::pin({
                        let out = self(&mut c, &req);
                        async {out.await; (c, req)}
                    })
                ))
            }
        }
    };
};
