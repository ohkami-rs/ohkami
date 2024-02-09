mod fangs;
pub mod builtin;

pub use fangs::{FrontFang, BackFang};
pub(crate) use fangs::{Fangs, FrontFangCaller, BackFangCaller};
use std::any::TypeId;


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
///     async fn bite(&self, res: &mut Response, _req: &Request) -> Result<(), Response> {
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
    id:              TypeId,
    pub(crate) proc: proc::FangProc,
}
impl Fang {
    pub(crate) fn is_front(&self) -> bool {
        matches!(self.proc, proc::FangProc::Front(_))
    }
}
const _: () = {
    impl<'f> PartialEq for &'f Fang {
        fn eq(&self, other: &Self) -> bool {
            self.id == other.id
        }
    }
};

pub(crate) mod proc {
    use std::sync::Arc;
    use super::{BackFangCaller, FrontFangCaller};

    
    #[derive(Clone)]
    pub enum FangProc {
        Front(FrontFang),
        Back (BackFang),
    }

    #[derive(Clone)]
    pub struct FrontFang(pub(crate) Arc<dyn FrontFangCaller>);

    #[derive(Clone)]
    pub struct BackFang(pub(crate) Arc<dyn BackFangCaller>);
}
