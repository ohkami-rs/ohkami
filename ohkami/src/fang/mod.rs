pub mod into_fang;
pub mod route;

use std::{pin::Pin, future::Future, collections::HashMap};
use crate::{context::Context, request::Request};
use self::{route::FangsRoute, into_fang::IntoFang};


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
    pub fn before<A, F: IntoFang<A> + Clone + Sync + Send + 'static>(mut self, route: &'static str, fang: F) -> Self {
        self.0.entry(FangsRoute::parse(route))
            .and_modify(|f| f.push(fang.into_fang()))
            .or_insert(vec![fang.into_fang()]);
        self
    }
}


const _: (/* Fangs impls */) = {
    impl Clone for Fangs {
        fn clone(&self) -> Self {
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