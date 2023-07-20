mod handlers; pub use handlers::{Handlers, ByAnother, Route};
mod into_handler; pub use into_handler::{IntoHandler};

use std::{pin::Pin, future::Future};
use crate::{
    Context, Request,
    layer0_lib::{List, BufRange},
    layer1_req_res::{Response},
};

pub(crate) const PATH_PARAMS_LIMIT: usize = 2;
pub(crate) type PathParams = List<BufRange, PATH_PARAMS_LIMIT>;


pub struct Handler(
    Box<dyn
        Fn(Request, Context, PathParams) -> Pin<
            Box<dyn
                Future<Output = Response>
                + Send + 'static
            >
        > + Send + Sync + 'static
    >
); const _: () = {
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
                    Future<Output = Response>
                    + Send + 'static
                >
            >;
            extern "rust-call" fn call_once(self, (req, c, params): (Request, Context, PathParams)) -> Self::Output {
                self.0(req, c, params)
            }
        }
    };
};
