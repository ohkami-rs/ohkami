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
    Fn(&mut Context, &mut Request) -> Result<(), Response>
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

/// <br/>
/// 
/// ## available `f` signatures
/// 
/// - to make *back fang* : `Fn(Response) -> Response`
/// 
/// - to make *front fang*: `Fn(&mut Context, Request) -> Request` or, return `Result<Request, Response>` instead of `Request` to early return error response
/// 
/// <br/>
/// 
/// ## example
/// 
/// ```
/// use ohkami::prelude::*;
/// use ohkami::{IntoFang, Fang};
/// 
/// struct AppendHeader;
/// impl IntoFang for AppendHeader {
///     fn bite(self) -> Fang {
///         Fang(|c: &mut Context, req: Request| {
///             c.headers.Server("ohkami");
///             req
///         })
///     }
/// }
/// ```
#[allow(non_snake_case)]
pub fn Fang<Args>(f: impl IntoFang<Args>) -> Fang {
    f.into_fang()
        .unwrap()//
}
