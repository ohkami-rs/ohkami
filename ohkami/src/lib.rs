#![doc(html_root_url = "https://docs.rs/ohkami")]


/*===== language features =====*/
#![feature(
    try_trait_v2,
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

mod cors {
    use std::sync::OnceLock;
    pub(crate) use crate::layer0_lib::CORS;

    /// set by builtin fang `CORS`
    static _CORS:    OnceLock<Option<CORS>> = OnceLock::new();
    static CORS_STR: OnceLock<&'static str> = OnceLock::new();

    #[allow(non_snake_case)]
    pub(crate) fn setCORS(cors: CORS) {
        if CORS_STR.set(Box::leak(Box::new(cors.to_string()))).is_err() {
            panic!("Can't set CORS config!")
        }
        if _CORS.set(Some(cors)).is_err() {
            panic!("Can't set CORS config!")
        }
    }

    #[allow(non_snake_case)]
    pub(crate) fn CORS() -> Option<&'static CORS> {
        _CORS.get_or_init(|| None).as_ref()
    }
    #[allow(non_snake_case)]
    pub(crate) fn CORSstr() -> &'static str {
        CORS_STR.get_or_init(|| "")
    }
} pub(crate) use cors::*;

pub use layer0_lib         ::{Status, Method, ContentType};
pub use layer1_req_res     ::{Request, Response, FromRequest};
pub use layer2_context     ::{Context};
pub use layer3_fang_handler::{Fang, Route};
pub use layer5_ohkami      ::{Ohkami, IntoFang};

pub mod prelude {
    pub use crate::{Response, Context, Route, Ohkami};
}

pub mod utils {
    pub use crate::layer3_fang_handler::{builtin::*};
    pub use ohkami_macros             ::{Query, Payload};
}

#[doc(hidden)]
pub mod __internal__ {
    pub use crate::layer1_req_res::{parse_json, parse_urlencoded, FromBuffer};
}


/*===== usavility =====*/
#[cfg(test)] #[allow(unused)] async fn __() {
// fangs
    struct AppendHeader;
    impl IntoFang for AppendHeader {
        fn bite(self) -> Fang {
            Fang::new(|c: &mut Context, req: Request| {
                c.headers.Server("ohkami");
                req
            })
        }
    }

    struct Log;
    impl IntoFang for Log {
        fn bite(self) -> Fang {
            Fang::new(|res: Response| {
                println!("{res:?}");
                res
            })
        }
    }

// handlers
    async fn health_check(c: Context) -> Response {
        c.NoContent()
    }

    async fn hello(c: Context, name: String) -> Response {
        c.OK().text(format!("Hello, {name}!"))
    }

// run
    Ohkami::with((AppendHeader, Log), (
        "/hc".
            GET(health_check),
        "/hello/:name".
            GET(hello),
    )).howl(3000).await
}
