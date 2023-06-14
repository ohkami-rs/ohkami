// mod fangs; pub(crate) use fangs::{Fangs}; pub use fangs::{public};
mod into_fang; pub use into_fang::{IntoFang};

use std::{pin::Pin, future::Future, sync::Arc};
use crate::{Context, Request};


#[derive(Clone)]
pub enum Fang {
    Front(FrontFang)
}

pub struct FrontFang(
    pub(crate) Arc<dyn
        Fn(Context, Request) -> Pin<
            Box<dyn
                Future<Output = (Context, Request)>
                + Send + 'static
            >
        > + Send + Sync + 'static
    >
); const _: () = {
    impl Clone for FrontFang {
        fn clone(&self) -> Self {
            Self(Arc::clone(&self.0))
        }
    }

    impl Fn<(Context, Request)> for FrontFang {
        extern "rust-call" fn call(&self, (c, req): (Context, Request)) -> Self::Output {
            (&self.0)(c, req)
        }
    } const _: (/* by */) = {
        impl FnMut<(Context, Request)> for FrontFang {
            extern "rust-call" fn call_mut(&mut self, (c, req): (Context, Request)) -> Self::Output {
                (&self.0)(c, req)
            }
        }
        impl FnOnce<(Context, Request)> for FrontFang {
            type Output = Pin<
                Box<dyn
                    Future<Output = (Context, Request)>
                    + Send + 'static
                >
            >;
            extern "rust-call" fn call_once(self, (c, req): (Context, Request)) -> Self::Output {
                (&*(self.0))(c, req)
            }
        }
    };
};
