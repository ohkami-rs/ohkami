//! <div align="center">
//!     <h1>ohkami</h1>
//!     ohkami <em>- [狼] wolf in Japanese -</em> is <strong>declarative</strong> web framework for Rust.
//! </div>
//! 
//! ## Quick start
//! ```ignore
//! use ohkami::prelude::*;
//! 
//! async fn health_check(c: Context) -> Response {
//!     c.NoContent()
//! }
//! 
//! async fn hello(c: Context, name: String) -> Response {
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
//! async fn get_user(c: Context,
//!     id: usize /* <-- path param */
//! ) -> Response { /* */ }
//! ```
//! Use tuple like `(verion, id): (u8, usize),` for multiple path params.
//! 
//! <br/>
//! 
//! ### handle query params / request body
//! ```ignore
//! use ohkami::prelude::*;
//! use ohkami::utils;   // <--
//! 
//! #[utils::Query]
//! struct SearchCondition {
//!     q: String,
//! }
//! async fn search(c: Context,
//!     condition: SearchCondition
//! ) -> Response { /* */ }
//! 
//! #[utils::Payload(JSON)]
//! #[derive(serde::Deserialize)]
//! struct CreateUserRequest {
//!     name:     String,
//!     password: String,
//! }
//! async fn create_user(c: Context,
//!     body: CreateUserRequest
//! ) -> Response { /* */ }
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
//! ```ignore
//! use ohkami::prelude::*;
//! use ohkami::{Fang, IntoFang};
//! 
//! struct AppendHeaders;
//! impl IntoFang for AppendHeaders {
//!     fn bite(self) -> Fang {
//!         Fang(|c: &mut Context, req: &mut Request| {
//!             c.headers
//!                 .Server("ohkami");
//!         })
//!     }
//! }
//! 
//! struct Log;
//! impl IntoFang for Log {
//!     fn bite(self) -> Fang {
//!         Fang(|res: Response| {
//!             println!("{res:?}");
//!             res
//!         })
//!     }
//! }
//! 
//! #[tokio::main]
//! async fn main() {
//!     Ohkami::with((AppendHeaders, Log), (
//!         "/"  .GET(root),
//!         "/hc".GET(health_check),
//!         "/api/users".
//!             GET(get_users).
//!             POST(create_user),
//!     )).howl(":8080").await
//! }
//! 
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
//!         "/hello".GET(|c: Context| async move {
//!             c.OK().text("Hello, world!")
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
//!     use ohkami::http::::Status;
//! 
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
//! 

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


/*===== runtime dependency injection layer =====*/

mod __rt__ {
    #[cfg(all(feature="rt_tokio", feature="DEBUG"))]
    #[allow(unused)]
    pub(crate) use tokio::test;
    #[cfg(all(feature="rt_async-std", feature="DEBUG"))]
    #[allow(unused)]
    pub(crate) use async_std::test;

    #[cfg(feature="rt_tokio")]
    pub(crate) use tokio::sync::Mutex;
    #[cfg(feature="rt_async-std")]
    pub(crate) use async_std::sync::Mutex;

    #[cfg(feature="rt_tokio")]
    pub(crate) use tokio::sync::RwLock;
    #[cfg(feature="rt_async-std")]
    pub(crate) use async_std::sync::RwLock;

    #[cfg(feature="rt_tokio")]
    pub(crate) use tokio::net::TcpListener;
    #[cfg(feature="rt_async-std")]
    pub(crate) use async_std::net::TcpListener;

    #[cfg(feature="rt_tokio")]
    pub(crate) use tokio::net::TcpStream;
    #[cfg(feature="rt_async-std")]
    pub(crate) use async_std::net::TcpStream;

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
mod layer2_context;
mod layer3_fang_handler;
mod layer4_router;
mod layer5_ohkami;

#[cfg(test)]
mod layer6_testing;

#[cfg(feature="websocket")]
mod x_websocket;


/*===== visibility managements =====*/

pub use layer1_req_res     ::{Request, Response, FromRequest};
pub use layer2_context     ::{Context};
pub use layer3_fang_handler::{Route, Fang};
pub use layer5_ohkami      ::{Ohkami, IntoFang};

pub mod prelude {
    pub use crate::{Request, Response, Context, Route, Ohkami};
}

pub mod http {
    pub use crate::layer0_lib::{Status, Method, ContentType};
}

pub mod utils {
    pub use crate::layer1_req_res     ::{File};
    pub use crate::layer3_fang_handler::{builtin::*};
    pub use ohkami_macros             ::{Query, Payload};
}

#[cfg(test)]
pub mod testing {
    pub use crate::layer6_testing::*;
}

#[cfg(feature="websocket")]
pub mod websocket {
    pub use crate::x_websocket::*;
}

#[doc(hidden)]
pub mod __internal__ {
    pub use crate::layer1_req_res::{
        parse_json,
        parse_formparts,
        parse_urlencoded,
        FromBuffer,
    };
}


/*===== usavility =====*/

#[cfg(feature="DEBUG")] #[allow(unused)] async fn __() {
    use http::Method;

// fangs
    struct AppendHeader;
    impl IntoFang for AppendHeader {
        fn bite(self) -> Fang {
            Fang(|c: &mut Context, _: &mut Request| {
                c.headers.Server("ohkami");
            })
        }
    }

    struct Log;
    impl IntoFang for Log {
        fn bite(self) -> Fang {
            Fang(|res: Response| {
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
    Ohkami::with((
        Log,
        AppendHeader,
        utils::cors("https://kanarusblog.software")
            .AllowCredentials()
            .AllowHeaders(["Content-Type"])
            .AllowMethods([Method::GET, Method::PUT, Method::POST, Method::DELETE])
            .MaxAge(3600)
    ), (
        "/hc".
            GET(health_check),
        "/hello/:name".
            GET(hello),
    )).howl(3000).await
}
