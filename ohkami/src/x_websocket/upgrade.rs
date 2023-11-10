use std::{sync::{Arc, OnceLock}, future::Future, pin::Pin};
use crate::__rt__::{TcpStream, Mutex};
type UpgradeLock<T> = Mutex<T>;


pub static UPGRADE_STREAMS: OnceLock<UpgradeStreams> = OnceLock::new();
pub async fn wait_upgrade(arc_stream: Arc<Mutex<TcpStream>>) -> UpgradeID {
    UPGRADE_STREAMS.get_or_init(UpgradeStreams::new)
        .push(arc_stream).await
}
pub async fn assume_upgraded(id: UpgradeID) -> TcpStream {
    UPGRADE_STREAMS.get_or_init(UpgradeStreams::new)
        .get(id).await
}

pub struct UpgradeID(usize);

pub struct UpgradeStreams {
    streams: UpgradeLock<Vec<UpgradeStream>>,
} const _: () = {
    impl UpgradeStreams {
        fn new() -> Self {
            Self {
                streams: UpgradeLock::new(Vec::new()),
            }
        }
    }
    impl UpgradeStreams {
        async fn push(&self, arc_stream: Arc<Mutex<TcpStream>>) -> UpgradeID {
            let mut this = self.streams.lock().await;
            let id = match this.iter().position(|us| us.is_available()) {
                Some(i) => {this[i] = UpgradeStream::new(arc_stream);  i}
                None    => {this.push(UpgradeStream::new(arc_stream)); this.len()-1}
            };
            UpgradeID(id)
        }
        async fn get(&self, id: UpgradeID) -> TcpStream {
            let mut this = self.streams.lock().await;
            Pin::new(this.get_mut(id.0).unwrap()).await
        }
    }
};

struct UpgradeStream(
    Option<Arc<Mutex<TcpStream>>>
); const _: () = {
    impl UpgradeStream {
        fn new(arc_stream: Arc<Mutex<TcpStream>>) -> Self {
            Self(Some(arc_stream))
        }
        fn is_available(&self) -> bool {
            self.0.is_none()
        }
    }
    impl Default for UpgradeStream {
        fn default() -> Self {Self(None)}
    }
    impl Future for UpgradeStream {
        type Output = TcpStream;
        fn poll(self: std::pin::Pin<&mut Self>, _: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
            match Arc::strong_count(self.0.as_ref().unwrap()) {
                1 => std::task::Poll::Ready(Arc::into_inner(self.get_mut().0.take().unwrap()).unwrap().into_inner()),
                _ => std::task::Poll::Pending,
            }
        }
    }
};

