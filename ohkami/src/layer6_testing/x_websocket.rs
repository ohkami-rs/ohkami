use crate::x_websocket::{Message};
use crate::x_websocket::{Config, send, write, flush};

use std::cell::UnsafeCell;
use std::pin::Pin;
use std::sync::Arc;
use std::io::{Error};
use std::sync::atomic::{AtomicBool, Ordering};
use std::task::Poll;


/// Web socket client for test with upgrade
pub struct TestWebSocket {
    stream:     TestStream,
    n_buffered: usize,
}
impl TestWebSocket {
    pub(crate) fn new(client_stream: TestStream) -> Self {
        Self {
            stream:     client_stream,
            n_buffered: 0,
        }
    }
}
impl TestWebSocket {
    pub async fn recv(&mut self) -> Result<Option<Message>, Error> {
        Message::read_from(&mut self.stream, &Config::default()).await
    }

    pub async fn send(&mut self, message: Message) -> Result<(), Error> {
        send(message, &mut self.stream, &Config::default(), &mut self.n_buffered).await
    }
    pub async fn write(&mut self, message: Message) -> Result<usize, Error> {
        write(message, &mut self.stream, &Config::default(), &mut self.n_buffered).await
    }
    pub async fn flush(&mut self) -> Result<(), Error> {
        flush(&mut self.stream, &mut self.n_buffered).await
    }
}


/// 
/// ```txt
///   client ------------- server
///      |                   |
///   [read  ============= write] |
///   =========================== | : TestStream
///   [write =============  read] |
///      |                   |
/// ```
pub struct TestStream {
    read:  Arc<HalfStream>,
    write: Arc<HalfStream>,
}
pub struct HalfStream {
    locked: AtomicBool, // It could be more efficient, but now using very simple lock
    buf:    UnsafeCell<Vec<u8>>
}

impl TestStream {
    pub(crate) fn new_pair() -> (Self, Self) {
        let (client_read, client_write) = (
            Arc::new(HalfStream {
                locked: AtomicBool::new(false),
                buf:    UnsafeCell::new(Vec::new()),
            }),
            Arc::new(HalfStream {
                locked: AtomicBool::new(false),
                buf:    UnsafeCell::new(Vec::new()),
            }),
        );

        let (server_write, server_read) = (
            client_read.clone(),
            client_write.clone(),
        );

        (
            Self { read:client_read, write:client_write },
            Self { read:server_read, write:server_write },
        )
    }
}

const _: () = {
    unsafe impl Sync for TestStream {}
    unsafe impl Send for TestStream {}

    impl TestStream {
        fn read_lock(self: Pin<&mut Self>) -> Poll<ReadLock<'_>> {
            match self.read.locked.compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed) {
                Ok(_)  => Poll::Ready(ReadLock(self.get_mut())),
                Err(_) => Poll::Pending,
            }
        }
        fn write_lock(self: Pin<&mut Self>) -> Poll<WriteLock<'_>> {
            match self.write.locked.compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed) {
                Ok(_)  => Poll::Ready(WriteLock(self.get_mut())),
                Err(_) => Poll::Pending,
            }
        }
    }

    struct ReadLock<'stream>(&'stream mut TestStream);
    impl<'stream> Drop for ReadLock<'stream> {
        fn drop(&mut self) {
            self.0.read.locked.store(false, Ordering::Release);
        }
    }
    impl<'stream> std::ops::Deref for ReadLock<'stream> {
        type Target = Vec<u8>;
        fn deref(&self) -> &Self::Target {
            unsafe {&*self.0.read.buf.get()}
        }
    }
    impl<'stream> std::ops::DerefMut for ReadLock<'stream> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            unsafe {&mut *self.0.read.buf.get()}
        }
    }

    struct WriteLock<'stream>(&'stream mut TestStream);
    impl<'stream> Drop for WriteLock<'stream> {
        fn drop(&mut self) {
            self.0.write.locked.store(false, Ordering::Release);
        }
    }
    impl<'stream> std::ops::Deref for WriteLock<'stream> {
        type Target = Vec<u8>;
        fn deref(&self) -> &Self::Target {
            unsafe {&*self.0.write.buf.get()}
        }
    }
    impl<'stream> std::ops::DerefMut for WriteLock<'stream> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            unsafe {&mut *self.0.write.buf.get()}
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
            let Poll::Ready(mut this) = self.read_lock()
                else {cx.waker().wake_by_ref(); return Poll::Pending};

            let size = (this.len()).min(buf.remaining());
            let (a, b) = this.split_at(size);
            buf.put_slice(a);
            *this = b.to_vec();

            Poll::Ready(Ok(()))
        }
    }

    impl tokio::io::AsyncWrite for TestStream {
        fn poll_write(
            self: Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
            buf: &[u8],
        ) -> std::task::Poll<Result<usize, std::io::Error>> {
            let Poll::Ready(mut this) = self.write_lock()
                else {cx.waker().wake_by_ref(); return Poll::Pending};

            this.extend_from_slice(buf);
            Poll::Ready(Ok(buf.len()))
        }

        fn poll_flush(self: Pin<&mut Self>, _: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), std::io::Error>> {
            Poll::Ready(Ok(()))
        }

        fn poll_shutdown(self: Pin<&mut Self>, _: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), std::io::Error>> {
            Poll::Ready(Ok(()))
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
            let Poll::Ready(mut this) = self.read_lock()
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
            let Poll::Ready(mut this) = self.write_lock()
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
