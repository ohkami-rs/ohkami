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
    ( $( $t:tt )* ) => {{
        #[cfg(feature="DEBUG")] {
            eprintln!( $( $t )* );

            #[cfg(feature="rt_worker")]
            worker::console_debug!( $( $t )* );
        }
    }};
}

pub use crate::fang::FangAction;

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

        #[cfg(feature="openapi")]
        fn openapi_responses() -> crate::openapi::Responses {
            crate::openapi::Responses::new([(
                500,
                crate::openapi::Response::when("Something went wrong")
                    .content("text/plain", crate::openapi::string())
            )])
        }
    }
};

#[cfg(feature="__rt_native__")]
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

pub const IP_0000: std::net::IpAddr = std::net::IpAddr::V4(std::net::Ipv4Addr::new(0, 0, 0, 0));

#[cfg(feature="rt_glommio")]
pub use num_cpus;
