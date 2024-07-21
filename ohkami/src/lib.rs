#![doc(html_root_url = "https://docs.rs/ohkami/latest/ohkami/")]

/* Execute static tests for sample codes in README */
#![cfg_attr(feature="DEBUG", doc = include_str!("../../README.md"))]

//! <div align="center">
//!     <h1>Ohkami</h1>
//!     Ohkami <em>- [狼] wolf in Japanese -</em> is intuitive and declarative web framework.
//! </div>
//! 
//! <br>
//! 
//! - *macro-less and type-safe* APIs for intuitive and declarative code
//! - *multi runtime* support：`tokio`, `async-std`, `worker` (Cloudflare Workers)


#![cfg_attr(feature="nightly", feature(
    min_specialization,
    try_trait_v2,
))]


#[cfg(any(
    all(feature="rt_tokio",     feature="rt_async-std"),
    all(feature="rt_async-std", feature="rt_worker"),
    all(feature="rt_worker",    feature="rt_tokio"),
))] compile_error!("
    Can't activate multiple `rt_*` features!
");


#[allow(unused)]
mod __rt__ {
    #[cfg(all(feature="rt_tokio", feature="DEBUG"))]
    pub(crate) use tokio::test;
    #[allow(unused)]
    #[cfg(all(feature="rt_async-std", feature="DEBUG"))]
    pub(crate) use async_std::test;

    #[cfg(feature="rt_tokio")]
    pub(crate) use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
    #[cfg(feature="rt_async-std")]
    pub(crate) use async_std::net::{TcpListener, TcpStream, ToSocketAddrs};

    #[cfg(feature="rt_tokio")]
    pub(crate) use tokio::task;
    #[cfg(feature="rt_async-std")]
    pub(crate) use async_std::task;

    #[cfg(feature="rt_tokio")]
    pub(crate) use tokio::time::sleep;
    #[cfg(feature="rt_async-std")]
    pub(crate) use async_std::task::sleep;

    #[cfg(feature="rt_tokio")]
    pub(crate) use tokio::io::AsyncReadExt as AsyncReader;
    #[cfg(feature="rt_async-std")]
    pub(crate) use async_std::io::ReadExt as AsyncReader;

    #[cfg(feature="rt_tokio")]
    pub(crate) use tokio::io::AsyncWriteExt as AsyncWriter;
    #[cfg(feature="rt_async-std")]
    pub(crate) use async_std::io::WriteExt as AsyncWriter;
}


mod request;
pub use request::{Request, Method, FromRequest, FromParam, Memory};
pub use ::ohkami_macros::FromRequest;

mod response;
pub use response::{Response, Status, IntoResponse};

mod fangs;
pub use fangs::{Fang, FangProc};

mod session;
#[cfg(any(feature="rt_tokio",feature="rt_async-std"))]
use session::Session;

mod ohkami;
#[cfg(any(feature="rt_tokio",feature="rt_async-std",feature="rt_worker"))]
pub use ohkami::{Ohkami, Route};

pub mod header;

pub mod builtin;

pub mod typed;

#[cfg(all(feature="ws", any(feature="rt_tokio",feature="rt_async-std")))]
pub mod ws;

#[cfg(feature="testing")]
#[cfg(any(feature="rt_tokio",feature="rt_async-std",feature="rt_worker"))]
pub mod testing;

pub mod utils {
    #[doc(hidden)]
    #[macro_export]
    macro_rules! warning {
        ( $( $t:tt )* ) => {{
            eprintln!( $( $t )* );

            #[cfg(feature="rt_worker")]
            worker::console_log!( $( $t )* );
        }};
    }

    #[doc(hidden)]
    #[macro_export]
    macro_rules! push_unchecked {
        ($buf:ident <- $bytes:expr) => {
            {
                let (buf_len, bytes_len) = ($buf.len(), $bytes.len());
                std::ptr::copy_nonoverlapping(
                    $bytes.as_ptr(),
                    $buf.as_mut_ptr().add(buf_len),
                    bytes_len
                );
                $buf.set_len(buf_len + bytes_len);
            }
        };
    }

    #[doc(hidden)]
    #[macro_export]
    macro_rules! DEBUG {
        ( $( $t:tt )* ) => {
            #[cfg(feature="DEBUG")] {
                println!( $( $t )* );
            }
        };
    }

    pub use crate::fangs::util::FangAction;

    #[cfg(feature="sse")]
    pub use ohkami_lib::stream::{self, Stream, StreamExt};

    #[cfg(not(feature="rt_worker"))]
    /// ```
    /// # let _ =
    /// {
    ///     std::time::SystemTime::now()
    ///         .duration_since(std::time::UNIX_EPOCH)
    ///         .unwrap()
    ///         .as_secs()
    /// }
    /// # ;
    /// ```
    #[inline] pub fn unix_timestamp() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
    #[cfg(feature="rt_worker")]
    /// JavaScript `Date.now() / 1000` --as--> Rust `u64`
    #[inline] pub fn unix_timestamp() -> u64 {
        (worker::js_sys::Date::now() / 1000.) as _
    }

    pub struct ErrorMessage(pub String);
    const _: () = {
        impl std::fmt::Debug for ErrorMessage {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(&self.0)
            }
        }
        impl std::fmt::Display for ErrorMessage {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(&self.0)
            }
        }
        impl std::error::Error for ErrorMessage {}
        impl super::IntoResponse for ErrorMessage {
            fn into_response(self) -> crate::Response {
                crate::Response::InternalServerError().with_text(self.0)
            }
        }
    };

    #[cfg(any(feature="rt_tokio",feature="rt_async-std"))]
    pub fn timeout_in<T>(
        duration: std::time::Duration,
        proc:     impl std::future::Future<Output = T>
    ) -> impl std::future::Future<Output = Option<T>> {
        use std::task::Poll;
        use std::pin::Pin;

        struct Timeout<Sleep, Proc> { sleep: Sleep, proc: Proc }

        impl<Sleep, Proc, T> std::future::Future for Timeout<Sleep, Proc>
        where
            Sleep: std::future::Future<Output = ()>,
            Proc:  std::future::Future<Output = T>,
        {
            type Output = Option<T>;

            #[inline]
            fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
                let Timeout { sleep, proc } = unsafe {self.get_unchecked_mut()};
                match unsafe {Pin::new_unchecked(proc)}.poll(cx) {
                    Poll::Ready(t) => Poll::Ready(Some(t)),
                    Poll::Pending  => unsafe {Pin::new_unchecked(sleep)}.poll(cx).map(|_| None)
                }
            }
        }

        Timeout { proc, sleep: crate::__rt__::sleep(duration) }
    }
}

#[cfg(feature="rt_worker")]
pub use ::ohkami_macros::{worker, bindings};

pub mod prelude {
    pub use crate::{Request, Response, IntoResponse, Method, Status};
    pub use crate::utils::FangAction;

    #[cfg(any(feature="rt_tokio",feature="rt_async-std",feature="rt_worker"))]
    pub use crate::{Route, Ohkami};
}

/// Somthing almost [serde](https://crates.io/crates/serde) + [serde_json](https://crates.io/crates/serde_json).
/// 
/// ---
/// *not_need_serde_in_your_dependencies.rs*
/// ```
/// use ohkami::serde::{json, Serialize};
/// 
/// #[derive(Serialize)]
/// struct User {
///     #[serde(rename = "username")]
///     name: String,
///     age:  u8,
/// }
/// 
/// # fn _user() {
/// let user = User {
///     name: String::from("ABC"),
///     age:  200,
/// };
/// assert_eq!(json::to_string(&user).unwrap(), r#"
///     {"age":200,"username":"ABC"}
/// "#);
/// # }
/// ```
/// ---
pub mod serde {
    pub use ::ohkami_macros::{Serialize, Deserialize};
    pub use ::serde::ser::{self, Serialize, Serializer};
    pub use ::serde::de::{self, Deserialize, Deserializer};
    pub use ::serde_json as json;
}

#[doc(hidden)]
pub mod __internal__ {
    pub use ::serde;

    pub use ohkami_macros::consume_struct;

    pub use crate::fangs::Fangs;

    /* for benchmarks */
    #[cfg(feature="DEBUG")]
    #[cfg(any(feature="rt_tokio",feature="rt_async-std",feature="rt_worker"))]
    pub use crate::{
        request::{RequestHeader, RequestHeaders},
        response::{ResponseHeader, ResponseHeaders},
    };
}
