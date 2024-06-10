#![cfg(feature="sse")]

use crate::utils::ErrorMessage;
use ::futures_core::stream::BoxStream;


pub struct EventStream(
    BoxStream<'static, Result<String, ErrorMessage>>
);

const _: () = {
    use crate::prelude::*;
    use crate::utils::StreamExt;
    use ::futures_core::Stream;
    use std::task::{Poll, Context};
    use std::future::Future;
    use std::pin::Pin;
    use std::marker::PhantomData;


    impl IntoResponse for EventStream {
        #[inline(always)]
        fn into_response(self) -> Response {
            Response::OK().with_stream(self.0)
        }
    }

    impl EventStream {
        #[inline]
        pub fn from_stream<S, T, E>(stream: S) -> Self
        where
            S: Stream<Item = Result<T, E>> + Send + Unpin + 'static,
            T: Into<String>,
            E: std::error::Error,
        {
            Self(Box::pin(stream.map(|res| res
                .map(Into::into)
                .map_err(|e| ErrorMessage(e.to_string()))
            )))
        }

        pub fn from_iter<I, T, E>(iter: I) -> Self
        where
            I: IntoIterator<Item = Result<T, E>>,
            T: Into<String> + Send + Unpin + 'static,
            E: std::error::Error + Send + Unpin + 'static,
            I::IntoIter: Send + Unpin + 'static,
        {
            struct FromIter<I, Item> {
                iter: I,
                __pt: PhantomData<Item>,
            }
            impl<I, T, E> Stream for FromIter<I, Result<T, E>>
            where
                I: Iterator<Item = Result<T, E>>,
                T: Into<String>,
                E: std::error::Error,
                Self: Unpin
            {
                type Item = Result<String, ErrorMessage>;
                fn poll_next(mut self: std::pin::Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Option<Self::Item>> {
                    Poll::Ready(
                        Iterator::next(&mut self.iter).map(|res| res
                            .map(Into::into)
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

        pub fn from_iter_async<I, T, E>(iter: I) -> Self
        where
            I: IntoIterator,
            I::Item: Future<Output = Result<T, E>>,
            T: Into<String> + Send + Unpin + 'static,
            E: std::error::Error + Send + Unpin + 'static,
            I::IntoIter: Send + Unpin + 'static,
        {
            struct FromIter<I, Item> {
                iter: I,
                __pt: PhantomData<Item>,
            }
            impl<I, T, E> Stream for FromIter<I, Result<T, E>>
            where
                I: Iterator,
                I::Item: Future<Output = Result<T, E>>,
                T: Into<String>,
                E: std::error::Error,
                Self: Unpin
            {
                type Item = Result<String, ErrorMessage>;

                fn poll_next(mut self: std::pin::Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
                    match Iterator::next(&mut self.iter) {
                        None           => Poll::Ready(None),
                        Some(ref mut item) => match (unsafe {Pin::new_unchecked(item)}).poll(cx) {
                            Poll::Pending       => Poll::Pending,
                            Poll::Ready(output) => Poll::Ready(Some(output
                                .map(Into::into)
                                .map_err(|e| ErrorMessage(e.to_string()))
                            ))
                        }
                    }
                }
            }

            Self(Box::pin(FromIter {
                iter: iter.into_iter(),
                __pt: PhantomData,
            }))
        }
    }
};