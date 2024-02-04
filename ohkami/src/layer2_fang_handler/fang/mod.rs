mod into_fang; pub use into_fang::IntoFang;

use std::{any::TypeId, sync::Arc};
use crate::{Request, Response};


/// # Fang ー ohkami's middleware system
/// 
/// <br>
/// 
/// *example.rs*
/// ```no_run
/// use ohkami::{Ohkami, Route};
/// use ohkami::{Fang, IntoFang, Response};
/// 
/// struct SetServer;
/// impl IntoFang for SetServer {
///     fn into_fang(self) -> Fang {
///         Fang(|res: &mut Response| {
///             res.headers.set()
///                 .Server("ohkami");
///         })
///     }
/// }
/// 
/// #[tokio::main]
/// async fn main() {
///     // Use `with` to give
///     // fangs for your Ohkami...
///     Ohkami::with((SetServer,),
///         "/".GET(|| async {
///             "Hello!"
///         })
///     ).howl(5000).await
/// }
/// ```
/// 
/// <br>
/// 
/// ---
/// 
/// $ cargo run
/// 
/// ---
/// 
/// $ curl -i http://localhost:5000\
/// HTTP/1.1 200 OK\
/// Content-Length: 6\
/// Content-Type: text/plain; charset=UTF-8\
/// Server: ohkami\
/// \
/// Hello!
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
    Fn(&mut Request) -> Result<(), Response>
    + Send
    + Sync
    + 'static
>);
#[derive(Clone)]
pub struct BackFang(pub(crate) Arc<dyn
    Fn(&Request, Response) -> Response
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

/// Create `Fang` from a function.
/// 
/// <br>
/// 
/// ## available `f` signatures
/// 
/// #### To make a *back fang*：
/// - `Fn({&/&mut Response})`
/// - `Fn(Response) -> Response`
/// 
/// #### To make a *front fang*：
/// - `Fn()`
/// - `Fn({&/&mut Request})`
/// - `_ -> Result<(), Response>` version of them
/// 
/// <br>
/// 
/// *example.rs*
/// ```
/// use ohkami::prelude::*;
/// 
/// struct AppendHeader;
/// impl IntoFang for AppendHeader {
///     fn into_fang(self) -> Fang {
///         Fang(|res: &mut Response| {
///             res.headers.set()
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
