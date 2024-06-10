#![cfg(feature="sse")]

use crate::utils::ErrorMessage;
use ::futures_core::stream::BoxStream;


pub struct DataStream<Data: Into<String>>(
    BoxStream<'static, Result<Data, ErrorMessage>>
);

const _: () = {
    use crate::prelude::*;
    use crate::utils::StreamExt;
    use std::task::{Poll, Context};
    use std::future::Future;
    use std::pin::Pin;
    use std::marker::PhantomData;
    use std::ptr::NonNull;
    use ::futures_core::Stream;


    impl<D: Into<String> + 'static> IntoResponse for DataStream<D> {
        #[inline(always)]
        fn into_response(self) -> Response {
            Response::OK().with_stream(self.0.map(|res| res.map(Into::into)))
        }
    }
    impl<D: Into<String>> Stream for DataStream<D> {
        type Item = Result<D, ErrorMessage>;
        fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
            (unsafe {self.get_unchecked_mut().0.as_mut()})
                .poll_next(cx)
        }
    }

    impl<D: Into<String> + Send + Unpin + 'static> DataStream<D> {
        #[inline]
        pub fn from_stream<S, E>(stream: S) -> Self
        where
            S: Stream<Item = Result<D, E>> + Send + 'static,
            E: std::error::Error,
        {
            Self(Box::pin(stream.map(|res| res
                .map_err(|e| ErrorMessage(e.to_string()))
            )))
        }

        pub fn from_iter<I, E>(iter: I) -> Self
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
                type Item = Result<D, ErrorMessage>;
                fn poll_next(mut self: std::pin::Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Option<Self::Item>> {
                    Poll::Ready(
                        Iterator::next(&mut self.iter).map(|res| res
                            .map_err(|e| ErrorMessage(e.to_string()))
                        )
                    )
                }
            }

            Self(Box::pin(FromIter {
                __pt: PhantomData,
                iter: iter.into_iter()
            }))
        }

        pub fn from_iter_async<I, E>(iter: I) -> Self
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
                type Item = Result<D, ErrorMessage>;

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
                            Poll::Ready(Some(res.map_err(|e| ErrorMessage(e.to_string()))))
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
