mod fangs;

pub use fangs::{FrontFang, BackFang};
use std::any::TypeId;

#[cfg(any(feature="rt_tokio",feature="rt_async-std"))]
pub(crate) use fangs::{FrontFangCaller, BackFangCaller};

#[cfg(any(feature="rt_tokio",feature="rt_async-std"))]
pub use fangs::Fangs;


/// # Fang ー ohkami's middleware system
/// 
/// <br>
/// 
/// *example.rs*
/// ```no_run
/// use ohkami::{Ohkami, Route};
/// use ohkami::{BackFang, Response, Request};
/// 
/// struct SetServer;
/// impl BackFang for SetServer {
///     type Error = std::convert::Infallible;
///     async fn bite(&self, res: &mut Response, _req: &Request) -> Result<(), Self::Error> {
///         res.headers.set()
///             .Server("ohkami");
///         Ok(())
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
///     ).howl("localhost:5000").await
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
    #[cfg(any(feature="rt_tokio",feature="rt_async-std"))]
    pub(crate) proc: proc::FangProc,
}
const _: () = {
    impl<'f> PartialEq for &'f Fang {
        fn eq(&self, other: &Self) -> bool {
            self.id == other.id
        }
    }
};

#[cfg(any(feature="rt_tokio",feature="rt_async-std"))]
pub(crate) mod proc {
    use super::{BackFangCaller, FrontFangCaller};
    use std::{future::Future, pin::Pin, sync::Arc};
    use crate::{Request, Response};

    
    #[derive(Clone)]
    pub enum FangProc {
        Front(FrontFang),
        Back (BackFang),

        /* Builtin specials */
        Timeout(crate::builtin::Timeout),
    }

    #[derive(Clone)]
    pub struct FrontFang(
        pub(super) Arc<dyn FrontFangCaller>
    );
    impl FrontFang {
        #[cfg(any(feature="rt_tokio",feature="rt_async-std"))]
        #[inline(always)] pub fn call<'c>(&'c self, req: &'c mut Request) -> Pin<Box<dyn Future<Output = Result<(), Response>> + Send + 'c>> {
            self.0.call(req)
        }
    }

    #[derive(Clone)]
    pub struct BackFang(   
        pub(super) Arc<dyn BackFangCaller>
    );
    impl BackFang {
        #[inline(always)] pub fn call<'c>(&'c self, res: &'c mut Response, req: &'c Request) -> Pin<Box<dyn Future<Output = Result<(), Response>> + Send + 'c>> {
            self.0.call(res, req)
        }
    }
}
