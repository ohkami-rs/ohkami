#![allow(non_snake_case)]

use serde::{Serialize};
use crate::layer3_fang_handler::RouteSections;
use super::{Handler, IntoHandler};


pub struct Handlers {
    route: RouteSections,
    GET: Option<Handler>,
    PUT: Option<Handler>,
    POST: Option<Handler>,
    HEAD: Option<Handler>,
    PATCH: Option<Handler>,
    DELETE: Option<Handler>,
    OPTIONS: Option<Handler>,
} macro_rules! register {
    ($( $method:ident ),*) => {
        impl Handlers {
            $(
                pub fn $method<H: IntoHandler<Args, T>, Args, T:Serialize>(mut self, handler: H) -> Self {
                    self.$method.replace(handler.into_handler());
                    self
                }
            )*
        }
    };
} register! { GET, PUT, POST, HEAD, PATCH, DELETE, OPTIONS }

pub trait Route {
    fn route(self) -> Handlers;
} impl Route for &'static str {
    fn route(self) -> Handlers {
        Handlers {
            route: RouteSections::from_literal(self),
            GET: None,
            PUT: None,
            POST: None,
            HEAD: None,
            PATCH: None,
            DELETE: None,
            OPTIONS: None,
        }
    }
}
