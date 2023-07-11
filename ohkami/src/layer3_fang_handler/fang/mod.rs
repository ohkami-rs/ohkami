mod into_fang; pub use into_fang::{IntoFang};
mod global; pub use global::{GlobalFangs}; pub(crate) use global::{getGlobalFangs};

use std::{any::TypeId, sync::Arc};
use crate::{Context, Request, Response};


#[derive(Clone)]
pub struct Fang {
    id:   TypeId,
    proc: FangProc,
}
#[derive(Clone)]
pub enum FangProc {
    Front(FrontFang),
    Back (BackFang),
}
#[derive(Clone)]
pub struct FrontFang(pub(crate) Arc<dyn
    Fn(Context, Request) -> Result<(Context, Request), Response>
>);
#[derive(Clone)]
pub struct BackFang(pub(crate) Arc<dyn
    Fn(Response) -> Response
>);


impl Fang {
    pub(crate) fn id(&self) -> &TypeId {
        &self.id
    }

    pub(crate) fn into_front(self) -> FrontFang {
        match self.proc {
            FangProc::Front(f) => f,
            FangProc::Back (_) => panic!("Called `into_front` for back fang")
        }
    }
    pub(crate) fn into_back(self) -> BackFang {
        match self.proc {
            FangProc::Back (b) => b,
            FangProc::Front(_) => panic!("Called `into_back` for front fang")
        }
    }
}

impl Fn<(Context, Request)> for FrontFang {
    extern "rust-call" fn call(&self, (c, req): (Context, Request)) -> Self::Output {
        self.0(c, req)
    }
} const _: (/* by */) = {
    impl FnOnce<(Context, Request)> for FrontFang {
        type Output = Result<(Context, Request), Response>;
        extern "rust-call" fn call_once(self, (c, req): (Context, Request)) -> Self::Output {
            self.0(c, req)
        }
    }
    impl FnMut<(Context, Request)> for FrontFang {
        extern "rust-call" fn call_mut(&mut self, (c, req): (Context, Request)) -> Self::Output {
            self.0(c, req)
        }
    }
};

impl Fn<(Response,)> for BackFang {
    extern "rust-call" fn call(&self, (res,): (Response,)) -> Self::Output {
        self.0(res)
    }
} const _: (/* by */) = {
    impl FnOnce<(Response,)> for BackFang {
        type Output = Response;
        extern "rust-call" fn call_once(self, (res,): (Response,)) -> Self::Output {
            self.0(res)
        }
    }
    impl FnMut<(Response,)> for BackFang {
        extern "rust-call" fn call_mut(&mut self, (res,): (Response,)) -> Self::Output {
            self.0(res)
        }
    }
};
