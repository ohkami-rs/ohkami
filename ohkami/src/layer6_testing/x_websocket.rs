use std::cell::UnsafeCell;
use std::pin::Pin;
use std::sync::Arc;
use crate::__rt__::Mutex;


pub struct TestWebSocket {
    client: TestStream,
} impl TestWebSocket {
    pub(crate) fn new(stream: TestStream) -> Self {
        Self { client: stream }
    }
}


pub(crate) struct TestStream {
    read:  Arc<UnsafeCell<Vec<u8>>>,
    write: Arc<UnsafeCell<Vec<u8>>>,
}
/// SAFETY: Only one of the client - server needs `&mut _` to `write` into
/// each `Arc<UnsafeCell<Vec<u8>>>` at a time
impl TestStream {
    fn read_half(&self) -> &[u8] {
        unsafe {&*self.read.get()}
    }
    fn write_half(&self) -> &mut Vec<u8> {
        unsafe {&mut *self.write.get()}
    }
}
impl Clone for TestStream {
    fn clone(&self) -> Self {
        Self {
            read:  self.read.clone(),
            write: self.write.clone(),
        }
    }
}
#[cfg(feature="rt_tokio")] const _: () = {
    impl tokio::io::AsyncRead for TestStream {
        fn poll_read(
            self: Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
            buf: &mut tokio::io::ReadBuf<'_>,
        ) -> std::task::Poll<std::io::Result<()>> {
            let mut read = &self.read[..];
            let read = unsafe {Pin::new_unchecked(&mut read)};
            read.poll_read(cx, buf)
        }
    }

    impl tokio::io::AsyncWrite for TestStream {
        fn poll_write(
            self: Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
            buf: &[u8],
        ) -> std::task::Poll<Result<usize, std::io::Error>> {
            unsafe {self.map_unchecked_mut(|this| &mut this.write)}
                .poll_write(cx, buf)
        }

        fn poll_flush(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), std::io::Error>> {
            unsafe {self.map_unchecked_mut(|this| &mut this.write)}
                .poll_flush(cx)
        }

        fn poll_shutdown(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), std::io::Error>> {
            unsafe {self.map_unchecked_mut(|this| &mut this.write)}
                .poll_shutdown(cx)
        }
    }
};
#[cfg(feature="rt_async-std")] const _: () = {
    impl async_std::io::Read for TestStream {
        fn poll_read(
            self: Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
            buf: &mut [u8],
        ) -> std::task::Poll<std::io::Result<usize>> {
            let mut read = self.read_half();
            let read = unsafe {Pin::new_unchecked(&mut read)};
            read.poll_read(cx, buf)
        }
    }

    impl async_std::io::Write for TestStream {
        fn poll_write(
            self: Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
            buf: &[u8],
        ) -> std::task::Poll<std::io::Result<usize>> {
            unsafe {self.map_unchecked_mut(|this| this.write_half())}
                .poll_write(cx, buf)
        }

        fn poll_flush(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<std::io::Result<()>> {
            unsafe {self.map_unchecked_mut(|this| this.write_half())}
                .poll_flush(cx)
        }

        fn poll_close(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<std::io::Result<()>> {
            unsafe {self.map_unchecked_mut(|this| this.write_half())}
                .poll_close(cx)
        }
    }
};
