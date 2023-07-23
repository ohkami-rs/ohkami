#![doc(html_root_url = "https://docs.rs/ohkami/")]


/*===== language features =====*/
#![feature(
    try_trait_v2,
    fn_traits, unboxed_closures,
)]


/*===== crate features =====*/
#[cfg(any(
    all(feature="rt_tokio", feature="rt_async-std")
))] compile_error!("
    Can't activate multiple `rt_*` feature!
");

#[cfg(not(any(
    feature="rt_tokio",
    feature="rt_async-std",
)))] compile_error!("
    Activate 1 of `rt_*` features：
    - rt_tokio
    - rt_async-std
");


/*===== dependency injection layer =====*/
mod __dep__ {
    #[cfg(feature="rt_tokio")]
    pub(crate) use tokio::net::TcpStream as TcpStream;
    #[cfg(feature="rt_async-std")]
    pub(crate) use async_std::net::TcpStream as TcpStream;

    #[cfg(feature="rt_tokio")]
    pub(crate) use tokio::net::TcpListener as TcpListener;
    #[cfg(feature="rt_async-std")]
    pub(crate) use async_std::net::TcpListener as TcpListener;

    #[cfg(feature="rt_tokio")]
    pub(crate) use tokio::task as task;
    #[cfg(feature="rt_async-std")]
    pub(crate) use async_std::task as task;

    #[cfg(feature="rt_tokio")]
    pub(crate) use tokio::io::AsyncReadExt as AsyncReader;
    #[cfg(feature="rt_async-std")]
    pub(crate) use async_std::io::ReadExt as AsyncReader;

    #[cfg(feature="rt_tokio")]
    pub(crate) use tokio::io::AsyncWriteExt as AsyncWriter;
    #[cfg(feature="rt_async-std")]
    pub(crate) use async_std::io::WriteExt as AsyncWriter;

    #[cfg(feature="rt_async-std")]
    pub(crate) use async_std::stream::StreamExt;
}


/*===== modules =====*/
mod layer0_lib;
mod layer1_req_res;
mod layer2_context;
mod layer3_fang_handler;
mod layer4_router;
mod layer5_ohkami;


/*===== visibility managements =====*/
pub(crate) use layer1_req_res     ::{QUERIES_LIMIT, HEADERS_LIMIT};
pub(crate) use layer3_fang_handler::{PATH_PARAMS_LIMIT};

pub(crate) mod cors {
    pub(crate) use crate::layer1_req_res::{
        CORS, headers::{CORS, CORSAllowOrigin}
    };
}

pub use layer0_lib         ::{Error, Status, Method};
pub use layer1_req_res     ::{Request, Response, FromRequest};
pub use layer2_context     ::{Context};
pub use layer3_fang_handler::{Route};
pub use layer5_ohkami      ::{Ohkami};

pub mod prelude {
    pub use crate::{Response, Context, Route, Ohkami};
}

pub mod utils {
    pub use crate::layer3_fang_handler::{cors, not_found};
    pub use ohkami_macros             ::{Query, Payload};

    use crate::{Context, Request, Response};
    /// Utility trait just to make writing `Fang`s easier :
    /// 
    /// ```ignore
    /// use ohkami::prelude::*;
    /// use ohkami::utils::Fang;
    /// 
    /// struct Log;
    /// impl Fang for Log {
    ///     // ↓ schema will be auto-completed ↓
    ///     fn front(c: &mut Context, req: Request) -> Result<Request, Response> {
    /// 
    ///         // you only have to write what you do
    /// 
    ///         let (method, path) = (req.method(), req.path());
    ///         tracing::info!("method: {method}, path: '{path}'");
    /// 
    ///         Ok(req)
    ///     }
    /// }
    /// 
    /// async fn main() {
    ///     tracing_subscriber::fmt()
    ///         /* ... config ... */.init();
    /// 
    ///     Ohkami::with((Log::front,))( // <----
    ///         "/hello/:name".
    ///             GET(move |c: Context, name: String| async {
    ///                 c.OK().text("Hello, {name}!")
    ///             })
    ///     ).howl(8080).await
    /// }
    /// ```
    pub trait Fang {
        #[allow(unused)]
        fn front(c: &mut Context, req: Request) -> Result<Request, Response> {
            Ok(req)
        }

        #[allow(unused)]
        fn back(res: Response) -> Response {
            res
        }
    }
}

#[doc(hidden)]
pub mod __internal__ {
    pub use crate::layer1_req_res::{parse_json, parse_urlencoded, FromBuffer};
}


/*===== usavility =====*/
#[cfg(test)] #[allow(unused)] async fn __() {
// fangs
    fn server(c: &mut Context, req: Request) -> Request {
        c.headers.Server("ohkami");
        req
    }

// handlers
    async fn health_check(c: Context) -> Response {
        c.NoContent()
    }

    async fn hello(c: Context, name: String) -> Response {
        c.OK().text(format!("Hello, {name}!"))
    }

// run
    Ohkami::with((server))(
        "/hc".
            GET(health_check),
        "/hello/:name".
            GET(hello),
    ).howl(3000).await
}
