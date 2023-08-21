mod into_fang; pub use into_fang::{IntoFang};
pub mod builtin;

use std::{any::TypeId, sync::Arc};
use crate::{Context, Request, Response};


#[derive(Clone)]
pub struct Fang {
    pub(crate) id:   TypeId,
    pub(crate) proc: FangProc,
}
#[derive(Clone)]
pub enum FangProc {
    Front(FrontFang),
    Back (BackFang),
}
#[derive(Clone)]
pub struct FrontFang(pub(crate) Arc<dyn
    Fn(Context, Request) -> Result<(Context, Request), Response>
    + Send
    + Sync
    + 'static
>);
#[derive(Clone)]
pub struct BackFang(pub(crate) Arc<dyn
    Fn(Response) -> Response
    + Send
    + Sync
    + 'static
>);


impl Fang {
    pub(crate) fn id(&self) -> &TypeId {
        &self.id
    }
}
impl Fang {
    /// <br/>
    /// 
    /// available `f` signatures :
    /// 
    /// - to make *back fang* : `(Response) -> Response`
    /// 
    /// - to make *front fang*: `(&mut Context, Request) -> Request` or, return `Result<Request, Response>` instead of `Request` to early return error response
    pub fn new<Args>(f: impl IntoFang<Args>) -> Self {
        f.into_fang()
            .unwrap()//
    }
}

// impl Fn<(Context, Request)> for FrontFang {
//     extern "rust-call" fn call(&self, (c, req): (Context, Request)) -> Self::Output {
//         self.0(c, req)
//     }
// } const _: (/* by */) = {
//     impl FnOnce<(Context, Request)> for FrontFang {
//         type Output = Result<(Context, Request), Response>;
//         extern "rust-call" fn call_once(self, (c, req): (Context, Request)) -> Self::Output {
//             self.0(c, req)
//         }
//     }
//     impl FnMut<(Context, Request)> for FrontFang {
//         extern "rust-call" fn call_mut(&mut self, (c, req): (Context, Request)) -> Self::Output {
//             self.0(c, req)
//         }
//     }
// };
// 
// impl Fn<(Response,)> for BackFang {
//     extern "rust-call" fn call(&self, (res,): (Response,)) -> Self::Output {
//         self.0(res)
//     }
// } const _: (/* by */) = {
//     impl FnOnce<(Response,)> for BackFang {
//         type Output = Response;
//         extern "rust-call" fn call_once(self, (res,): (Response,)) -> Self::Output {
//             self.0(res)
//         }
//     }
//     impl FnMut<(Response,)> for BackFang {
//         extern "rust-call" fn call_mut(&mut self, (res,): (Response,)) -> Self::Output {
//             self.0(res)
//         }
//     }
// };
// 