#![cfg(feature="sse")]

use crate::utils::{ErrorMessage, MapStream};
use ::futures_core::stream::BoxStream;


pub struct EventStream(
    BoxStream<'static, Result<String, ErrorMessage>>
);

const _: () = {
    use crate::prelude::*;
    use ::futures_core::Stream;
    use std::task::{Poll, Context};

    impl IntoResponse for EventStream {
        #[inline]
        fn into_response(self) -> Response {
            Response::OK().with_stream(self.0)
        }
    }

    impl EventStream {
        pub fn from_stream<S>(stream: S) -> Self {
            

            todo!()
        }

//        pub fn from_iter<I, T, E>(iter: I) -> Self
//        where
//            I: Iterator<Item = Result<T, E>>,
//            T: Into<std::borrow::Cow<'static, [u8]>>,
//            E: std::error::Error,
//        {
//            struct FromIter<I, T, E>(
//                I, std::marker::PhantomData<(T, E)>
//            );
//            impl<I, T, E: std::error::Error> Stream for FromIter<I, T, E>
//            where
//                I: Iterator<Item = Result<T, E>>,
//                T: Into<std::borrow::Cow<'static, [u8]>>,
//                E: std::error::Error,
//                Self: Unpin
//            {
//                type Item = Result<
//                    std::borrow::Cow<'static, [u8]>,
//                    crate::utils::ErrorMessage
//                >;
//                fn poll_next(mut self: std::pin::Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Option<Self::Item>> {
//                    Poll::Ready(self.as_mut().0.next().map(|r| r
//                        .map(Into::into)
//                        .map_err(|e| crate::utils::ErrorMessage(e.to_string()))
//                    ))
//                }
//            }
//
//            Self::from_stream(FromIter(iter, std::marker::PhantomData)
//        }
    }
};