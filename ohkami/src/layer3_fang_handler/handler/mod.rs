mod handlers;     pub use handlers::{Handlers, ByAnother, Route};
mod into_handler; pub use into_handler::{IntoHandler};

#[cfg(test)] use std::sync::Arc;

use std::{
    pin::Pin,
    future::Future,
};
use crate::{
    Context, Request,
    layer0_lib::{List, Slice},
    layer1_req_res::{Response},
};

pub(crate) const PATH_PARAMS_LIMIT: usize = 2;
pub(crate) type PathParams = List<Slice, PATH_PARAMS_LIMIT>;


#[cfg(not(test))]
pub struct Handler {
    pub(crate) requires_upgrade: bool,
    pub(crate) proc: Box<dyn
        Fn(&mut Request, Context, PathParams) -> Pin<
            Box<dyn
                Future<Output = Response>
                + Send + 'static
            >
        > + Send + Sync + 'static
    >
}

#[cfg(test)]
#[derive(Clone)]
pub struct Handler {
    pub(crate) requires_upgrade: bool,
    pub(crate) proc: Arc<dyn
        Fn(&mut Request, Context, PathParams) -> Pin<
            Box<dyn
                Future<Output = Response>
                + Send + 'static
            >
        > + Send + Sync + 'static
    >
}


impl Handler {
    fn new(requires_upgrade: bool, proc: (impl
        Fn(&mut Request, Context, PathParams) -> Pin<
            Box<dyn
                Future<Output = Response>
                + Send + 'static
            >
            > + Send + Sync + 'static
        )
    ) -> Self {
        #[cfg(not(test))] {Self { requires_upgrade, proc: Box::new(proc) }}
        #[cfg(test)]      {Self { requires_upgrade, proc: Arc::new(proc) }}
    }
}
