mod into_fang;
pub use into_fang::IntoFang;

mod route;
pub use route::FangsRoute;
pub(crate) use route::FangRoutePattern;

use std::{pin::Pin, future::Future, collections::HashMap};
use crate::{context::Context, request::Request};


pub struct Fangs(
    HashMap<
        FangsRoute,
        Vec<Fang>,
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
    pub fn before<A, F: IntoFang<A> + Clone + Sync + Send + 'static>(mut self, route: &'static str, fang: &'static F) -> Self {
        let (to_modify, to_insert) = (fang.into_fang(), fang.into_fang());
        self.0.entry(FangsRoute::parse(route))
            .and_modify(|f| f.push(to_modify))
            .or_insert(vec![to_insert]);
        self
    }

    pub(crate) fn clone(&'static self) -> Self {
        let mut fangs = HashMap::<FangsRoute, Vec<Fang>>::new();
        for (route, vec_fang) in self.0.iter() {
            let mut new_vec = Vec::with_capacity(vec_fang.len());
            for fang in vec_fang {
                let new_fang: Fang = Box::new(|c, request| Box::pin(
                    fang(c, request)
                ));
                new_vec.push(new_fang)
            }
            fangs.insert(
                route.clone(),
                new_vec
            );
        }
        Self(fangs)
    }
}


const _: (/* Fangs impls */) = {
    impl IntoIterator for Fangs {
        type IntoIter = <
            HashMap<
                FangsRoute,
                Vec<Fang>
            > as IntoIterator>
        ::IntoIter;

        type Item = <
            HashMap<
                FangsRoute,
                Vec<Fang>
            > as IntoIterator
        >::Item;

        fn into_iter(self) -> Self::IntoIter {
            self.0.into_iter()
        }
    }
};