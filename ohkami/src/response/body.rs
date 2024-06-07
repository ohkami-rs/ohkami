use std::future::Future;


pub trait Body {
    #[cfg(any(feature="rt_tokio",feature="rt_async-std"))]
    fn write_into(self, stream: &mut crate::__rt__::TcpStream) -> impl Future<Output = ()> + Send;

    #[cfg(feature="rt_worker")]
    fn into_worker(self) -> worker::Response;
}

#[cfg(any(feature="rt_tokio",feature="rt_async-std"))]
const _: () = {
    /*
    use std::future::Future;
    use crate::__rt__::{TcpStream, AsyncWriter};


    pub trait Body {
        fn write_into(self, stream: &mut TcpStream) -> impl Future<Output = ()> + Send;
    }

    impl Body for Vec<u8> {
        async fn write_into(self, stream: &mut TcpStream) {
            stream.write_all(&self).await.expect("Failed to send response");
            stream.flush().await.expect("Failed to flush stream");
        }
    }

    // impl<S> Body for S
    // where
    //     S: IntoIterator,
    //     <S as IntoIterator>::Item: Future,
    //     <<S as IntoIterator>::Item as Future>::Output: Into<Vec<u8>>,
    //     S: Send,
    //     <S as IntoIterator>::Item: Send,
    //     <S as IntoIterator>::IntoIter: Send,
    // {
    //     async fn write_into(self, stream: &mut TcpStream) {
    //         for chunk in self {
    //             let chunk: Vec<u8> = chunk.await.into();
    //             stream.write_all(&chunk).await.expect("Failed to write stream chunk");
    //         }
    //         stream.flush().await.expect("Failed to flush stream");
    //     }
    // }

    struct StreamBuf<'s>(&'s mut TcpStream);

    impl<F, Fut> Body for F
    where
        F:   FnOnce() -> Fut + Send,
        Fut: Future,
    {
        async fn write_into(self, stream: &mut TcpStream) {

        }
    }
    */
};

#[cfg(feature="rt_worker")]
const _: () = {
    fn __() {
        //::worker::Response::from_stream(stream)
    }
};
