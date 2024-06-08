use ohkami_lib::CowSlice;
use ::futures_core::stream::{BoxStream, Stream};


pub enum Content {
    Payload(CowSlice),

    #[cfg(feature="sse")]
    Stream(BoxStream<'static, Result<CowSlice, String>>),
}

#[cfg(any(feature="rt_tokio",feature="rt_async-std"))]
const _: () = {
    use crate::__rt__::{TcpStream, AsyncWriter};

    impl Content {
        pub(crate) async fn write_into(self, conn: &mut TcpStream) {
            match self {
                Self::Payload(bytes) => {
                    conn.write_all(&bytes).await.expect("Failed to write response to TCP connection");
                }

                #[cfg(feature="sse")]
                Self::Stream(mut stream) => {
                    struct NextChunk<'c>(
                        &'c mut BoxStream<'static, Result<CowSlice, String>>
                    ); const _: () = {
                        use std::pin::Pin;
                        use std::task::{Context, Poll};

                        impl<'c> std::future::Future for NextChunk<'c> {
                            type Output = Option<Result<CowSlice, String>>;

                            #[inline(always)]
                            fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
                                (unsafe {self.map_unchecked_mut(|pin| &mut *pin.0)})
                                    .poll_next(cx)
                            }
                        }
                    };

                    while let Some(chunk) = NextChunk(&mut stream).await {
                        match chunk {
                            Ok(bytes) => conn.write_all(&bytes).await.expect("Failed to write response"),
                            Err(msg)  => {crate::warning!("Error in stream: {msg}"); break}
                        }
                    }
                }
            }; conn.flush().await.expect("Failed to flush TCP connection")
        }
    }
};

#[cfg(feature="rt_worker")]
const _: () = {
    impl Content {
        pub(crate) fn into_worker(self) -> ::worker::Response {
            match self {
                Self::Payload(bytes) => ::worker::Response::from_bytes(bytes.into()).unwrap(),

                #[cfg(feature="sse")]
                Self::Stream(stream) => ::worker::Response::from_stream(stream).unwrap()
            }
        }
    }
};
