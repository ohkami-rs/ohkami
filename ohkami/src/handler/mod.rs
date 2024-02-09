mod handlers;     pub use handlers::{Handlers, ByAnother, Route};
mod into_handler; pub use into_handler::{IntoHandler};

use std::{
    pin::Pin,
    sync::Arc,
    future::Future,
};
use crate::{Request, Response};


#[derive(Clone)]
pub struct Handler {
    pub(crate) proc: Arc<dyn
        Fn(&mut Request) -> Pin<
            Box<dyn
                Future<Output = Response>
                + Send + 'static
            >
        > + Send + Sync + 'static
    >
}

impl Handler {
    fn new(
        proc: (
            impl Fn(&mut Request) -> Pin<
                Box<dyn
                    Future<Output = Response>
                    + Send + 'static
                >
            > + Send + Sync + 'static
        )
    ) -> Self {
        Self {
            proc: Arc::new(proc),
        }
    }
}
