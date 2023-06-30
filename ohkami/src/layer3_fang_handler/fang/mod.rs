mod into_fang; pub use into_fang::{IntoFang};
mod global; pub use global::{GlobalFangs}; pub(crate) use global::{getGlobalFangs};

use std::{pin::Pin, future::Future, sync::Arc, any::TypeId};
use crate::{Context, Request};


#[derive(Clone)]
pub enum Fang {
    Front(FrontFang)
} impl Fang {
    pub(crate) fn id(&self) -> &TypeId {
        match self {
            Self::Front(ff) => &ff.id,
        }
    }
}

pub struct FrontFang {
    pub(crate) id: TypeId,
    pub(crate) proc: Arc<dyn
        Fn(Context, Request) -> Pin<
            Box<dyn
                Future<Output = (Context, Request)>
                + Send + 'static
            >
        > + Send + Sync + 'static
    >
} const _: () = {
    impl Clone for FrontFang {
        fn clone(&self) -> Self {
            Self {
                id: self.id.clone(),
                proc: Arc::clone(&self.proc),
            }
        }
    }

    impl Fn<(Context, Request)> for FrontFang {
        extern "rust-call" fn call(&self, (c, req): (Context, Request)) -> Self::Output {
            (&self.proc)(c, req)
        }
    } const _: (/* by */) = {
        impl FnMut<(Context, Request)> for FrontFang {
            extern "rust-call" fn call_mut(&mut self, (c, req): (Context, Request)) -> Self::Output {
                (&self.proc)(c, req)
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
                (&*(self.proc))(c, req)
            }
        }
    };
};
