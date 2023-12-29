mod handlers;     pub use handlers::{Handlers, ByAnother, Route};
mod into_handler; pub use into_handler::{IntoHandler};

#[cfg(test)] use std::sync::Arc;

use std::{
    pin::Pin,
    future::Future,
};
use crate::{
    Context, Request,
    layer1_req_res::{Response},
};


#[cfg(not(test))]
pub struct Handler {
    #[cfg(feature="websocket")] pub(crate) requires_upgrade: bool,
    pub(crate) proc: Box<dyn
        Fn(Context, &mut Request) -> Pin<
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
    #[cfg(feature="websocket")] pub(crate) requires_upgrade: bool,
    pub(crate) proc: Arc<dyn
        Fn(Context, &mut Request) -> Pin<
            Box<dyn
                Future<Output = Response>
                + Send + 'static
            >
        > + Send + Sync + 'static
    >
}


impl Handler {
    fn new(
        proc: (impl Fn(Context, &mut Request) -> Pin<
            Box<dyn
                Future<Output = Response>
                + Send + 'static
            >
            > + Send + Sync + 'static
        )
    ) -> Self {
        #[cfg(not(test))] {Self {
            #[cfg(feature="websocket")] requires_upgrade: false,
            proc: Box::new(proc),
        }}
        #[cfg(test)] {Self {
            #[cfg(feature="websocket")] requires_upgrade: false,
            proc: Arc::new(proc),
        }}
    }

    #[cfg(feature="websocket")] fn requires_upgrade(mut self) -> Self {
        self.requires_upgrade = true;
        self
    }
}
