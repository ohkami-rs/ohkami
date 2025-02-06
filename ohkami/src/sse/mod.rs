#![cfg(feature="sse")]

use ohkami_lib::{Stream, StreamExt};
use ohkami_lib::stream::impls::{QueueStream, Queue};
use std::{pin::Pin, future::Future, marker::PhantomData};

/// Streaming response with data of type `T` (default: `String`).
/// 
/// **note**: `T` is requried to impl `sse::Data` to be encoded to `String`
/// for the specification of Server-Sent Events (see, for example,
/// <https://html.spec.whatwg.org/multipage/server-sent-events.html#event-stream-interpretation>)
/// 
/// ---
/// 
/// *example.rs*
/// ```
/// use ohkami::sse::DataStream;
/// use tokio::time::{sleep, Duration};
/// 
/// async fn handler() -> DataStream {
///     DataStream::new(|mut s| async move {
///         s.send("starting streaming...");
///         for i in 1..=5 {
///             sleep(Duration::from_secs(1)).await;
///             s.send(format!("MESSAGE #{i}"));
///         }
///         s.send("streaming finished!");
///     })
/// }
/// 
/// # use ohkami::prelude::*;
/// fn my_ohkami() -> Ohkami {
///     Ohkami::new((
///         "/sse".GET(handler),
///     ))
/// }
/// ```
pub struct DataStream<T: Data = String>(
    Pin<Box<dyn Stream<Item = String> + Send>>,
    PhantomData<fn()->T>
);

#[cfg(not(feature="openapi"))]
mod bound {
    pub trait Bound {}
    impl<T> Bound for T {}
}
#[cfg(feature="openapi")]
mod bound {
    pub trait Bound: crate::openapi::Schema {}
    impl<T: crate::openapi::Schema> Bound for T {}
}

pub trait Data: bound::Bound + 'static {
    fn encode(self) -> String;
}
const _: () = {
    impl Data for String {
        fn encode(self) -> String {self}
    }
    impl Data for &'static str {
        fn encode(self) -> String {self.into()}
    }
};

impl<T: Data> crate::IntoResponse for DataStream<T> {
    #[inline]
    fn into_response(self) -> crate::Response {
        let mut res = crate::Response::OK();
        res.set_stream_raw(self.0);/* no additional boxing */
        res
    }

    #[cfg(feature="openapi")]
    fn openapi_responses() -> crate::openapi::Responses {
        crate::openapi::Responses::new([(
            200,
            crate::openapi::Response::when("Streaming")
                .content("text/event-stream", <T as crate::openapi::Schema>::schema())
        )])
    }
}

impl<T: Data, S> From<S> for DataStream<T>
where
    S: Stream<Item = T> + Send + 'static
{
    fn from(stream: S) -> Self {
        Self(Box::pin(stream.map(Data::encode)), PhantomData)
    }
}

impl<T: Data + Send + 'static> DataStream<T> {
    /// Create `DataStream` from an async proccess with stream handle.
    /// 
    /// **note**: Use `DataStream::from` if you already has a `Stream<Item = T>`
    /// instance.
    /// 
    /// ---
    /// 
    /// *example.rs*
    /// ```
    /// # use tokio::time::{sleep, Duration};
    /// use ohkami::sse::DataStream;
    /// 
    /// async fn handler() -> DataStream {
    ///     DataStream::new(|mut s| async move {
    ///         s.send("starting streaming...");
    ///         for i in 1..=5 {
    ///             sleep(Duration::from_secs(1)).await;
    ///             s.send(format!("MESSAGE #{i}"));
    ///         }
    ///         s.send("streaming finished!");
    ///     })
    /// }
    /// ```
    pub fn new<F, Fut>(f: F) -> Self
    where
        F:   FnOnce(handle::Stream<T>) -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        Self(Box::pin(QueueStream::new(|q| f(handle::Stream::from(q)))), PhantomData)
    }
}

pub mod handle {
    use super::*;

    pub struct Stream<T>(
        pub(super) Queue<String>,
        pub(super) PhantomData<fn()->T>
    );
    impl<T> From<Queue<String>> for self::Stream<T> {
        fn from(q: Queue<String>) -> Self {
            Self(q, PhantomData)
        }
    }
    impl<T: Data> self::Stream<T> {
        #[inline]
        pub fn send(&mut self, data: impl Into<T>) {
            self.0.push(Data::encode(data.into()));
        }
    }
}
