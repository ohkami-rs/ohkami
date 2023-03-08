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
                + Send + 'req
            >
        > + Send + Sync + 'req
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
pub(crate) fn combine<'req>(this: &'req Fang<'req>, child: &'req Fang<'req>) -> Fang<'req> {
    Box::new(|mut c, mut request| Box::pin(async {
        (c, request) = this(c, request).await;
        (c, request) = child(c, request).await;
        (c, request)
    }))
}
pub(crate) fn combine_optional<'req>(this: Option<&'req Fang<'req>>, child: Option<&'req Fang<'req>>) -> Option<Fang<'req>> {
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

    impl<'req> IntoIterator for Fangs<'req> {
        type IntoIter = <
            HashMap<
                FangsRoute,
                Fang<'req>
            > as IntoIterator>
        ::IntoIter;

        type Item = <
            HashMap<
                FangsRoute,
                Fang<'req>
            > as IntoIterator
        >::Item;

        fn into_iter(self) -> Self::IntoIter {
            self.0.into_iter()
        }
    }
};