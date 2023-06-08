mod fangs; pub use fangs::{Fangs};
mod into_fang; pub use into_fang::{IntoFang};

use std::{pin::Pin, future::Future};
use crate::{Context, Request};


pub enum Fang {
    Before(
        Box<dyn
            Fn(Context, Request) -> Pin<
                Box<dyn
                    Future<Output = (Context, Request)>
                    + Send + 'static
                >
            > + Send + Sync + 'static
        >
    ),
} impl Fang {
    pub(crate) fn clone(self) -> (Self, Self) {
        match self {
            Self::Before(f) => {
                let f: &'static _ = Box::leak(f);
                (
                    Self::Before(Box::new(|c, req| Box::pin(f(c, req)))),
                    Self::Before(Box::new(|c, req| Box::pin(f(c, req)))),
                )
            }
        }
    }
}
