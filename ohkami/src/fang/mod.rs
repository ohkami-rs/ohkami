pub mod into_fang;
pub mod route;

use std::{pin::Pin, future::Future, collections::HashMap};
use crate::{context::Context, request::Request};
use self::{route::FangsRoute, into_fang::IntoFang};


pub struct Fangs(
    HashMap<
        FangsRoute,
        Fang,
    >
);
pub type Fang =
    Box<dyn
        Fn(Context, Request) -> Pin<
            Box<dyn
                Future<Output = (Context, Request)>
                + Send + 'static
            >
        > + Send + Sync + 'static
    >
;


impl Fangs {
    pub fn new() -> Self {
        Fangs(HashMap::new())
    }
    pub fn before<F: IntoFang + Clone + Sync + Send>(mut self, route: &'static str, fang: F) -> Self {
        let route = FangsRoute::parse(route);
        match self.0.remove(&route) {
            None => {self.0.insert(route, fang.clone().into_fang());},
            Some(f) => {
                self.0.insert(route, Box::new(|mut c, mut request| Box::pin(async {
                    (c, request) = f(c, request).await;
                    fang.into_fang()(c, request).await
                })));
            },
        }
        self
    }
}
pub(crate) fn combine(this: Fang, child: Fang) -> Fang {
    Box::new(move |mut c, mut request| Box::pin(async {
        (c, request) = this(c, request).await;
        (c, request) = child(c, request).await;
        (c, request)
    }))
}
pub(crate) fn combine_optional(this: Option<Fang>, child: Option<Fang>) -> Option<Fang> {
    match (this, child) {
        (Some(this_fang), Some(child_fang)) => Some(
            Box::new(|mut c, mut request| Box::pin(async {
                (c, request) = this_fang(c, request).await;
                (c, request) = child_fang(c, request).await;
                (c, request)
            }))
        ),
        (Some(this_fang), None) => Some(
            Box::new(|c, request| Box::pin(
                this_fang(c, request)
            ))
        ),
        (None, Some(child_fang)) => Some(
            Box::new(|c, request| Box::pin(
                child_fang(c, request)
            ))
        ),
        (None, None) => None,
    }
    
}


const _: (/* Fangs impls */) = {
    impl Clone for Fangs {
        fn clone(&self) -> Self {
            let mut fangs = HashMap::<FangsRoute, Fang>::new();
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

    impl IntoIterator for Fangs {
        type IntoIter = <
            HashMap<
                FangsRoute,
                Fang
            > as IntoIterator>
        ::IntoIter;

        type Item = <
            HashMap<
                FangsRoute,
                Fang
            > as IntoIterator
        >::Item;

        fn into_iter(self) -> Self::IntoIter {
            self.0.into_iter()
        }
    }
};