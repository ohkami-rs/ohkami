#![cfg(feature="sse")]

use ohkami_lib::{Stream, StreamExt};
use std::pin::Pin;


/// # Simple typed stream response
/// 
/// <br>
/// 
/// Expects one type param: `DataStream<{message type}>` and, \
/// optional second param: `DataStream<{message type}, {error type}>` \
/// ( default error type = `std::convert::Infallible` ).
/// 
/// <br>
/// 
/// ---
/// *example.rs*
/// ```no_run
/// use ohkami::prelude::*;
/// use ohkami::typed::DataStream;
/// use tokio::time::sleep;
/// 
/// async fn sse() -> DataStream<String> {
///     DataStream::from_iter_async((1..=5).map(|i| async move {
///         sleep(std::time::Duration::from_secs(1)).await;
///         Ok(format!("Hi, I'm message #{i} !"))
///     }))
/// }
/// 
/// #[tokio::main]
/// async fn main() {
///     Ohkami::new((
///         "/sse".GET(sse),
///     )).howl("localhost:5050").await
/// }
/// ```
pub struct DataStream<
    D: Into<String>,
    E: std::error::Error = std::convert::Infallible
>(
    Pin<Box<dyn Stream<Item = Result<D, E>> + Send>>
);

const _: () = {
    use crate::prelude::*;
    use std::task::{Poll, Context};
    use std::future::Future;
    use std::marker::PhantomData;
    use std::ptr::NonNull;


    impl<D: Into<String> + 'static, E: std::error::Error + 'static>
    IntoResponse for DataStream<D, E> {
        #[inline(always)]
        fn into_response(self) -> Response {
            Response::OK().with_stream(self.0.map(|res| res.map(Into::into)))
        }
    }

    impl<D: Into<String> + 'static, E: std::error::Error + 'static>
    Stream for DataStream<D, E> {
        type Item = Result<D, E>;
        fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
            (unsafe {self.get_unchecked_mut().0.as_mut()})
                .poll_next(cx)
        }
    }

    impl<D: Into<String> + Send + Unpin + 'static, E: std::error::Error + 'static>
    DataStream<D, E> {
        #[cfg(not(feature="rt_worker"))]
        #[inline]
        pub fn from_stream<S>(stream: S) -> Self
        where
            S: Stream<Item = Result<D, E>> + Send + 'static,
            E: std::error::Error,
        {
            Self(Box::pin(stream))
        }
        #[cfg(feature="rt_worker")]
        #[inline]
        pub fn from_stream<S>(stream: S) -> Self
        where
            S: Stream<Item = Result<D, E>> + 'static,
            E: std::error::Error,
        {
            struct SendStream<S: Stream>(S);
            unsafe impl<S: Stream> Send for SendStream<S> {}
            unsafe impl<S: Stream> Sync for SendStream<S> {}
            impl<S: Stream> Stream for SendStream<S> {
                type Item = S::Item;
                fn size_hint(&self) -> (usize, Option<usize>) {
                    self.0.size_hint()
                }
                #[inline(always)]
                fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
                    (unsafe {self.map_unchecked_mut(|this| &mut this.0)})
                        .poll_next(cx)
                }
            }
            
            Self(Box::pin(SendStream(stream)))
        }

        pub fn from_iter<I>(iter: I) -> Self
        where
            I: IntoIterator<Item = Result<D, E>>,
            E: std::error::Error + Send + Unpin + 'static,
            I::IntoIter: Send + Unpin + 'static,
        {
            struct FromIter<I, Item> {
                iter: I,
                __pt: PhantomData<Item>,
            }
            impl<I, D, E> Stream for FromIter<I, Result<D, E>>
            where
                I: Iterator<Item = Result<D, E>>,
                D: Into<String>,
                E: std::error::Error,
                Self: Unpin
            {
                type Item = Result<D, E>;
                fn poll_next(mut self: std::pin::Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Option<Self::Item>> {
                    Poll::Ready(
                        Iterator::next(&mut self.iter)
                    )
                }
            }

            Self(Box::pin(FromIter {
                __pt: PhantomData,
                iter: iter.into_iter()
            }))
        }

        pub fn from_iter_async<I>(iter: I) -> Self
        where
            I: IntoIterator,
            I::Item: Future<Output = Result<D, E>>,
            E: std::error::Error + Send + Unpin + 'static,
            I::IntoIter: Send + Unpin + 'static,
        {
            struct FromIter<I: Iterator, Item> {
                iter: I,
                next: Option<I::Item>,
                pend: Option<NonNull<I::Item>>,
                __pt: PhantomData<Item>,
            }

            unsafe impl<I: Iterator + Send, Item: Send> Send for FromIter<I, Item> {}

            impl<I, D, E> Stream for FromIter<I, Result<D, E>>
            where
                I: Iterator,
                I::Item: Future<Output = Result<D, E>>,
                D: Into<String>,
                E: std::error::Error,
            {
                type Item = Result<D, E>;

                fn poll_next(self: std::pin::Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
                    let this = unsafe {self.get_unchecked_mut()};

                    if this.next.is_none() {
                        let Some(next) = this.iter.next() else {
                            return Poll::Ready(None)
                        };
                        this.next = Some(next);
                        this.pend = Some(NonNull::new(this.next.as_mut().unwrap() as *mut _).unwrap());
                    }

                    match unsafe {Pin::new_unchecked(
                        this.pend.as_mut().map(|nn| nn.as_mut()).unwrap()
                    )}.poll(cx) {
                        Poll::Pending => {
                            cx.waker().wake_by_ref();
                            Poll::Pending
                        }
                        Poll::Ready(res) => {
                            this.next = None;
                            this.pend = None;
                            Poll::Ready(Some(res))
                        }
                    }
                }
            }

            Self(Box::pin(FromIter {
                iter: iter.into_iter(),
                next: None,
                pend: None,
                __pt: PhantomData,
            }))
        }
    }
};


#[cfg(test)]
async fn __handler() -> DataStream<String> {
    DataStream::from_iter_async((1..=5).map(|i| async move {
        Ok(format!("I'm message #{i} !"))
    }))
}
