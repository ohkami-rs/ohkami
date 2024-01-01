mod into_fang; pub use into_fang::IntoFang;

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

    pub(crate) fn is_front(&self) -> bool {
        matches!(self.proc, FangProc::Front(_))
    }
}

/// <br/>
/// 
/// ## available `f` signatures
/// 
/// <br/>
/// 
/// #### To make *back fang*：
/// - `Fn(&Response)`
/// - `Fn(Response) -> Response`
/// 
/// <br/>
/// 
/// #### To make *front fang*：
/// - `Fn( {&/&mut Context} )`
/// - `Fn( {&/&mut Request} )`
/// - `Fn( {&/&mut Context}, {&/&mut Request} )`
/// - `_ -> Result<(), Response>` version of them
/// 
/// <br/>
/// 
/// ## Example
/// 
/// ```
/// use ohkami::prelude::*;
/// 
/// struct AppendHeader;
/// impl IntoFang for AppendHeader {
///     fn into_fang(self) -> Fang {
///         Fang(|c: &mut Context| {
///             c.set_headers()
///                 .Server("ohkami");
///         })
///     }
/// }
/// ```
#[allow(non_snake_case)]
pub fn Fang<Args>(f: impl IntoFang<Args>) -> Fang {
    f.into_fang()
        .unwrap()
}
