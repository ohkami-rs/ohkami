use std::{
    sync::{OnceLock, atomic::{AtomicBool, Ordering}},
    pin::Pin, cell::UnsafeCell,
    future::Future,
};
use crate::__rt__::{TcpStream};

#[cfg(feature="rt_tokio")] use {
    std::sync::Arc,
    crate::__rt__::Mutex,
};


pub async fn request_upgrade_id() -> UpgradeID {
    struct ReserveUpgrade;
    impl Future for ReserveUpgrade {
        type Output = UpgradeID;
        fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
            let Some(mut streams) = UpgradeStreams().request_reservation()
                else {cx.waker().wake_by_ref(); return std::task::Poll::Pending};

            let id = UpgradeID(match streams.iter().position(|cell| cell.is_empty()) {
                Some(i) => i,
                None    => {streams.push(StreamCell::new()); streams.len() - 1},
            });

            streams[id.as_usize()].reserved = true;

            std::task::Poll::Ready(id)
        }
    }

    ReserveUpgrade.await
}

/// SAFETY: This must be called after the corresponded `reserve_upgrade`
pub unsafe fn reserve_upgrade(
    id: UpgradeID,
    #[cfg(feature="rt_tokio")]     stream: Arc<Mutex<TcpStream>>,
    #[cfg(feature="rt_async-std")] stream: TcpStream,
) {
    #[cfg(debug_assertions)] assert!(
        UpgradeStreams().get().get(id.as_usize()).is_some_and(
            |cell| cell.reserved && cell.stream.is_some()),
        "Cell not reserved"
    );

    (UpgradeStreams().get_mut())[id.as_usize()].stream = Some(stream);
}

pub async fn assume_upgradable(id: UpgradeID) -> TcpStream {
    struct AssumeUpgraded{id: UpgradeID}
    impl Future for AssumeUpgraded {
        type Output = TcpStream;
        fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
            let Some(StreamCell { reserved, stream }) = (unsafe {UpgradeStreams().get_mut()}).get_mut(self.id.as_usize())
                else {cx.waker().wake_by_ref(); return std::task::Poll::Pending};

            #[cfg(feature="rt_tokio")]
            if !stream.as_ref().is_some_and(|arc| Arc::strong_count(arc) == 1)
                {cx.waker().wake_by_ref(); return std::task::Poll::Pending};
            #[cfg(feature="rt_async-std")]
            if !stream.is_some()
                {cx.waker().wake_by_ref(); return std::task::Poll::Pending};

            *reserved = false;

            #[cfg(feature="rt_tokio")] {
            std::task::Poll::Ready(unsafe {
                Mutex::into_inner(
                    Arc::into_inner(
                        Option::take(stream)
                            .unwrap_unchecked())
                                .unwrap_unchecked())})
            }
            #[cfg(feature="rt_async-std")] {
            std::task::Poll::Ready(unsafe {
                Option::take(stream)
                    .unwrap_unchecked()})
            }
        }
    }

    AssumeUpgraded{id}.await
}


static UPGRADE_STREAMS: OnceLock<UpgradeStreams> = OnceLock::new();
#[allow(non_snake_case)] fn UpgradeStreams() -> &'static UpgradeStreams {
    UPGRADE_STREAMS.get_or_init(UpgradeStreams::new)
}

struct UpgradeStreams {
    in_scanning: AtomicBool,
    streams:     UnsafeCell<Vec<StreamCell>>,
} const _: () = {
    unsafe impl Sync for UpgradeStreams {}

    impl UpgradeStreams {
        fn new() -> Self {
            Self {
                in_scanning: AtomicBool::new(false),
                streams:     UnsafeCell::new(Vec::new()),
            }
        }
        fn get(&self) -> &Vec<StreamCell> {
            unsafe {&*self.streams.get()}
        }
        unsafe fn get_mut(&self) -> &mut Vec<StreamCell> {
            &mut *self.streams.get()
        }
        fn request_reservation(&self) -> Option<ReservationLock<'_>> {
            self.in_scanning.compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
                .ok().and(Some(ReservationLock(unsafe {self.get_mut()})))
        }
    }

    struct ReservationLock<'scan>(&'scan mut Vec<StreamCell>);
    impl<'scan> Drop for ReservationLock<'scan> {
        fn drop(&mut self) {
            UpgradeStreams().in_scanning.store(false, Ordering::Release)
        }
    }
    impl<'scan> std::ops::Deref for ReservationLock<'scan> {
        type Target = Vec<StreamCell>;
        fn deref(&self) -> &Self::Target {&*self.0}
    }
    impl<'scan> std::ops::DerefMut for ReservationLock<'scan> {
        fn deref_mut(&mut self) -> &mut Self::Target {self.0}
    }
}; 

struct StreamCell {
    reserved: bool,

    #[cfg(feature="rt_tokio")]     stream: Option<Arc<Mutex<TcpStream>>>,
    #[cfg(feature="rt_async-std")] stream: Option<TcpStream>,
} const _: () = {
    impl StreamCell {
        fn new() -> Self {
            Self {
                reserved: false,
                stream:   None,
            }
        }
        fn is_empty(&self) -> bool {
            (!self.reserved) && self.stream.is_none()
        }
    }
};

#[derive(Clone, Copy)]
pub struct UpgradeID(usize);
const _: () = {
    impl UpgradeID {
        fn as_usize(&self) -> usize {
            self.0
        }
    }
};
