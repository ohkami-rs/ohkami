//! <div align="center">
//!     <h1>ohkami</h1>
//!     ohkami <em>- [狼] wolf in Japanese -</em> is <strong>declarative</strong> web framework for Rust.
//! </div>
//! 
//! ## Quick start
//! ```ignore
//! use ohkami::prelude::*;
//! use ohkami::utils::Text;
//! 
//! async fn health_check() -> impl IntoResponse {
//!     Status::NoContent
//! }
//! 
//! async fn hello(name: String) -> Text {
//!     c.OK().text(format!("Hello, {name}!"))
//! }
//! 
//! #[tokio::main]
//! async fn main() {
//!     Ohkami::new((
//!         "/hc"         .GET(health_check),
//!         "/hello/:name".GET(hello),
//!     )).howl(3000).await
//! }
//! ```
//! <br/>
//! 
//! ### handle path params
//! ```ignore
//! use ohkami::prelude::*;
//! 
//! #[tokio::main]
//! async fn main() {
//!     Ohkami::new((
//!         "/api/users/:id".
//!             GET(get_user),
//!     )).howl("localhost:5000").await
//! }
//! 
//! async fn get_user(
//!     id: usize /* <-- path param */
//! ) -> Status { Status::OK }
//! ```
//! Use tuple like `(verion, id): (u8, usize),` for multiple path params.
//! 
//! <br/>
//! 
//! ### handle query params / request body
//! ```
//! use ohkami::prelude::*;
//! use ohkami::utils;   // <--
//! 
//! #[utils::Query]
//! struct SearchCondition {
//!     q: String,
//! }
//! async fn search(
//!     condition: SearchCondition
//! ) -> impl IntoResponse { Status::OK }
//! 
//! #[utils::Payload(JSON)]
//! #[derive(serde::Deserialize)]
//! struct CreateUserRequest {
//!     name:     String,
//!     password: String,
//! }
//! 
//! async fn create_user(
//!     body: CreateUserRequest
//! ) -> impl IntoResponse { Status::Created }
//! ```
//! `#[Query]`, `#[Payload( 〜 )]` implements `FromRequest` trait for the struct.
//! 
//! ( with path params : `(Context, {path params}, {FromRequest values...})` )
//! 
//! <br/>
//! 
//! ### use middlewares
//! ohkami's middlewares are called "**fang**s".
//! 
//! ```
//! use ohkami::prelude::*;
//! 
//! struct AppendHeaders;
//! impl IntoFang for AppendHeaders {
//!     fn into_fang(self) -> Fang {
//!         Fang(|res: &mut Response| {
//!             res.headers.set()
//!                 .Server("ohkami");
//!         })
//!     }
//! }
//! 
//! struct Log;
//! impl IntoFang for Log {
//!     fn into_fang(self) -> Fang {
//!         Fang(|res: &Response| {
//!             println!("{res:?}");
//!         })
//!     }
//! }
//! ```
//! `Fang` schema :
//! 
//! - *back fang*  : `Fn(Response) -> Response`
//! - *front fang* : `Fn(&mut Context) | Fn(&mut Request) | Fn(&mut Context, &mut Request)`, or `_ -> Result<(), Response>` for early returning error responses
//! 
//! <br/>
//! 
//! ### pack of Ohkamis
//! ```ignore
//! #[tokio::main]
//! async fn main() {
//!     // ...
//! 
//!     let users_ohkami = Ohkami::new((
//!         "/".
//!             POST(create_user),
//!         "/:id".
//!             GET(get_user).
//!             PATCH(update_user).
//!             DELETE(delete_user),
//!     ));
//! 
//!     Ohkami::new((
//!         "/hc"       .GET(health_check),
//!         "/api/users".By(users_ohkami), // <-- nest by `By`
//!     )).howl(5000).await
//! }
//! ```
//! 
//! <br/>
//! 
//! ### testing
//! ```ignore
//! use ohkami::prelude::*;
//! use ohkami::testing::*; // <--
//! 
//! fn hello_ohkami() -> Ohkami {
//!     Ohkami::new((
//!         "/hello".GET(|| async move {
//!             ohkami::utils::Text::OK("Hello, world!")
//!         })
//!     ))
//! }
//! 
//! #[tokio::main]
//! async fn main() {
//!     hello_ohkami()
//!         .howl(5050).await
//! }
//! 
//! #[cfg(test)]
//! #[tokio::test]
//! async fn test_my_ohkami() {
//!     let hello_ohkami = hello_ohkami();
//! 
//!     let res = hello_ohkami.oneshot(TestRequest::GET("/")).await;
//!     assert_eq!(res.status, Status::NotFound);
//! 
//!     let res = hello_ohkami.oneshot(TestRequest::GET("/hello")).await;
//!     assert_eq!(res.status, Status::OK);
//!     assert_eq!(res.content.unwrap().text().unwrap(), "Hello, world!");
//! }
//! ```


#![doc(html_root_url = "https://docs.rs/ohkami")]

#![allow(incomplete_features)]
#![cfg_attr(feature="nightly", feature(
    try_trait_v2,
    generic_arg_infer,

    /* imcomplete features */
    generic_const_exprs,
))]


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


/*===== async runtime dependency layer =====*/

mod __rt__ {
    #[allow(unused)]
    #[cfg(all(feature="rt_tokio", feature="DEBUG"))]
    pub(crate) use tokio::test;
    #[allow(unused)]
    #[cfg(all(feature="rt_async-std", feature="DEBUG"))]
    pub(crate) use async_std::test;

    #[cfg(all(feature="websocket", feature="rt_tokio"))]
    pub(crate) use tokio::net::TcpStream;
    #[cfg(all(feature="websocket", feature="rt_async-std"))]
    pub(crate) use async_std::net::TcpStream;

    #[cfg(feature="rt_tokio")]
    pub(crate) use tokio::net::TcpListener;
    #[cfg(feature="rt_async-std")]
    pub(crate) use async_std::net::TcpListener;

    #[cfg(feature="rt_tokio")]
    pub(crate) use tokio::sync::Mutex;
    #[allow(unused)]
    #[cfg(feature="rt_async-std")]
    pub(crate) use async_std::sync::Mutex;

    #[cfg(feature="rt_tokio")]
    pub(crate) use tokio::task;
    #[cfg(feature="rt_async-std")]
    pub(crate) use async_std::task;

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
mod layer2_fang_handler;
mod layer3_router;
mod layer4_ohkami;

#[cfg(feature="testing")]
mod x_testing;

#[cfg(feature="utils")]
mod x_utils;

#[cfg(feature="websocket")]
mod x_websocket;


/*===== visibility managements =====*/

pub use layer1_req_res     ::{Request, Response, FromRequest, FromParam, IntoResponse, Memory};
pub use layer2_fang_handler::{Route, Fang};
pub use layer4_ohkami      ::{Ohkami, IntoFang};

pub mod prelude {
    pub use crate::{Request, Response, Route, Ohkami, Fang, IntoFang, IntoResponse, http::Status, utils::{Text, JSON}};
}

pub mod http {
    pub use crate::layer0_lib::{Status, Method, append};
}

#[cfg(feature="testing")]
pub mod testing {
    pub use crate::x_testing::*;
}

#[cfg(feature="utils")]
pub mod utils {
    pub use crate::x_utils::{now, CORS, JWT, File, JSON, Text, HTML, Redirect};
    pub use ohkami_macros ::{Query, Payload};
}

#[cfg(feature="websocket")]
pub mod websocket {
    pub use crate::x_websocket::*;
}

#[doc(hidden)]
pub mod __internal__ {
    #[cfg(feature="utils")]
    pub use crate::x_utils::{
        parse_json,
        parse_formparts,
        parse_urlencoded,
    };
}


/*===== usavility =====*/

#[cfg(feature="utils")]
#[cfg(test)] #[allow(unused)] async fn __() {
    use http::Method;

// fangs
    struct AppendHeader;
    impl IntoFang for AppendHeader {
        //const METHODS: &'static [Method] = &[Method::GET];
        fn into_fang(self) -> Fang {
            Fang(|res: &mut Response| {
                res.headers.set().Server("ohkami");
            })
        }
    }

    struct Log;
    impl IntoFang for Log {
        //const METHODS: &'static [Method] = &[];
        fn into_fang(self) -> Fang {
            Fang(|res: Response| {
                println!("{res:?}");
                res
            })
        }
    }

// handlers
    async fn health_check() -> http::Status {
        http::Status::NoContent
    }

    async fn hello(name: &str) -> utils::Text {
        utils::Text::OK(format!("Hello, {name}!"))
    }

// run
    Ohkami::with((
        Log,
        AppendHeader,
        utils::CORS("https://kanarusblog.software")
            .AllowCredentials()
            .AllowHeaders(&["Content-Type"])
            .AllowMethods(&[Method::GET, Method::PUT, Method::POST, Method::DELETE])
            .MaxAge(3600)
    ), (
        "/hc".
            GET(health_check),
        "/hello/:name".
            GET(hello),
    )).howl(3000).await
}
