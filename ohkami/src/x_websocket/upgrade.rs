use std::{sync::{Arc, OnceLock}, future::Future, pin::Pin};
use crate::__rt__::{TcpStream, Mutex};
type UpgradeLock<T> = Mutex<T>;


pub static UPGRADE_STREAMS: OnceLock<UpgradeStreams> = OnceLock::new();

pub async fn request_upgrade_id() -> UpgradeID {
    UPGRADE_STREAMS.get_or_init(UpgradeStreams::new)
        .reserve().await
}
pub async fn reserve_upgrade(id: UpgradeID, stream: Arc<Mutex<TcpStream>>) {
    UPGRADE_STREAMS.get_or_init(UpgradeStreams::new)
        .set(id, stream).await
}
//pub async fn cancel_upgrade
pub async fn assume_upgraded(id: UpgradeID) -> TcpStream {
    UPGRADE_STREAMS.get_or_init(UpgradeStreams::new)
        .get(id).await
}

#[derive(Clone, Copy)]
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
        async fn reserve(&self) -> UpgradeID {
            let mut this = self.streams.lock().await;
            match this.iter().position(UpgradeStream::is_empty) {
                Some(i) => {
                    this[i].reserved = true;
                    UpgradeID(i)
                }
                None => {
                    this.push(UpgradeStream {
                        reserved: true,
                        stream:   None,
                    });
                    UpgradeID(this.len() - 1)
                },
            }
        }
        async fn set(&self, id: UpgradeID, stream: Arc<Mutex<TcpStream>>) {
            let mut this = self.streams.lock().await;
            this[id.0].stream = Some(stream)
        }
        async fn get(&self, id: UpgradeID) -> TcpStream {
            let mut this = self.streams.lock().await;
            Pin::new(this.get_mut(id.0).unwrap()).await
        }
    }
};

struct UpgradeStream {
    reserved: bool,
    stream:   Option<Arc<Mutex<TcpStream>>>,
} const _: () = {
    impl UpgradeStream {
        fn is_empty(&self) -> bool {
            self.stream.is_none() && !self.reserved
        }
    }
    impl Default for UpgradeStream {
        fn default() -> Self {
            Self { reserved: false, stream: None }
        }
    }
    impl Future for UpgradeStream {
        type Output = TcpStream;
        fn poll(self: std::pin::Pin<&mut Self>, _: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
            match Arc::strong_count(self.stream.as_ref().unwrap()) {
                1 => std::task::Poll::Ready(Arc::into_inner(self.get_mut().stream.take().unwrap()).unwrap().into_inner()),
                _ => std::task::Poll::Pending,
            }
        }
    }
};

