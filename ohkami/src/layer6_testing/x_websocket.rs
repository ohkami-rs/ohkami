use std::cell::UnsafeCell;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::task::Poll;


pub struct TestWebSocket {
    client: TestStream,
} impl TestWebSocket {
    pub(crate) fn new(stream: TestStream) -> Self {
        Self { client: stream }
    }
}


/// 
/// ```txt
///   client ------------- server
///      |                   |
///   [read  ============= write] : TestStream
///   [write =============  read] : TestStream
///      |                   |
/// TestWebSocket      TestWebSocket
/// ```
pub(crate) struct TestStream {
    locked: AtomicBool, // It could be more efficient, but now use very simple lock
    buf:    Arc<UnsafeCell<Vec<u8>>>,
}
const _: () = {
    impl TestStream {
        fn lock(self: Pin<&mut Self>) -> Poll<Lock<'_>> {
            match self.locked.compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed) {
                Ok(_)  => Poll::Ready(Lock(self.get_mut())),
                Err(_) => Poll::Pending,
            }
        }
    }

    struct Lock<'stream>(&'stream mut TestStream);
    impl<'stream> Drop for Lock<'stream> {
        fn drop(&mut self) {
            self.0.locked.store(false, Ordering::Release);
        }
    }
    impl<'stream> std::ops::Deref for Lock<'stream> {
        type Target = Vec<u8>;
        fn deref(&self) -> &Self::Target {
            unsafe {&*self.0.buf.get()}
        }
    }
    impl<'stream> std::ops::DerefMut for Lock<'stream> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            unsafe {&mut *self.0.buf.get()}
        }
    }
};
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
            let Poll::Ready(mut this) = self.lock()
                else {cx.waker().wake_by_ref(); return Poll::Pending};

            let size = (this.len()).min(buf.len());
            let (a, b) = this.split_at(size);
            buf.copy_from_slice(a);
            *this = b.to_vec();

            Poll::Ready(Ok(size))
        }
    }

    impl async_std::io::Write for TestStream {
        fn poll_write(
            self: Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
            buf: &[u8],
        ) -> std::task::Poll<std::io::Result<usize>> {
            let Poll::Ready(mut this) = self.lock()
                else {cx.waker().wake_by_ref(); return Poll::Pending};

            this.extend_from_slice(buf);
            Poll::Ready(Ok(buf.len()))
        }

        fn poll_flush(self: Pin<&mut Self>, _: &mut std::task::Context<'_>) -> std::task::Poll<std::io::Result<()>> {
            Poll::Ready(Ok(()))
        }

        fn poll_close(self: Pin<&mut Self>, _: &mut std::task::Context<'_>) -> std::task::Poll<std::io::Result<()>> {
            Poll::Ready(Ok(()))
        }
    }
};
