pub mod into_fang;
pub mod route;

use std::{pin::Pin, future::Future, collections::HashMap};
use crate::{context::Context, request::Request};
use self::{route::FangsRoute, into_fang::IntoFang};


pub struct Fangs<'req>(
    HashMap<
        FangsRoute,
        Fang<'req>,
    >
);
pub type Fang<'req> =
    Box<dyn
        Fn(Context, Request<'req>) -> Pin<
            Box<dyn
                Future<Output = (Context, Request<'req>)>
                + Send
            >
        > + Send + Sync
    >
;


impl<'req> Fangs<'req> {
    pub fn new() -> Self {
        Fangs(HashMap::new())
    }
    pub fn before<F: IntoFang<'req>>(mut self, route: &'static str, fang: F) -> Self {
        self.0.entry(FangsRoute::parse(route))
            .and_modify(|f| *f = combine(f, &fang.into_fang()))
            .or_insert(fang.into_fang());
        self
    }
}
fn combine<'req>(this: &'req Fang<'req>, another: &'req Fang<'req>) -> Fang<'req> {
    Box::new(|c, request| Box::pin(async {
        (c, request) = this(c, request).await;
        (c, request) = another(c, request).await;
        (c, request)
    }))
}


const _: (/* Fangs impls */) = {
    impl<'req> Clone for Fangs<'req> {
        fn clone(&self) -> Self {
            let mut fangs = HashMap::<FangsRoute, Fang<'req>>::new();
            for (route, fang) in self.0.iter() {
                fangs.insert(
                    route.clone(),
                    Box::new(|c, request| Box::pin(async {
                        fang(c, request).await
                    }))
                );
            }
            Self(fangs)
        }
    }
};