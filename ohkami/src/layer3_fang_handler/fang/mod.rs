use std::{pin::Pin, future::Future};
use crate::{Context, Request};


pub enum Fang {
    Before(
        Box<dyn
            Fn(Context, Request) -> Pin<
                Box<dyn
                    Future<Output = (Context, Request)>
                    + Send + 'static
                >
            > + Send + Sync + 'static
        >
    ),
} impl Fang {

}


pub trait IntoFang<Args> {
    fn into_fang(self) -> Fang;
}

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
};

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


