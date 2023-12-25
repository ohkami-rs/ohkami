use std::{
    sync::{OnceLock, atomic::{AtomicBool, Ordering}},
    pin::Pin, cell::UnsafeCell,
    future::Future,
};
use crate::__rt__::{TcpStream};
#[cfg(test)] use crate::layer6_testing::TestStream;

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

/// SAFETY: This must be called after the corresponded `request_upgrade_id`
pub unsafe fn reserve_upgrade(id: UpgradeID, stream: TcpStream) {
    #[cfg(debug_assertions)] assert!(
        UpgradeStreams().get().get(id.as_usize()).is_some_and(
            |cell| cell.reserved && cell.stream.is_some()),
        "Cell not reserved"
    );

    (UpgradeStreams().get_mut())[id.as_usize()].stream = Some(stream);
}
/// SAFETY: This must be called after the corresponded `request_upgrade_id_in_test`
#[cfg(test)] pub unsafe fn reserve_upgrade_in_test(id: UpgradeID, stream: TestStream) {
    #[cfg(debug_assertions)] assert!(
        UpgradeStreams().get().get(id.as_usize()).is_some_and(
            |cell| cell.reserved && cell.stream.is_some()),
        "Cell not reserved"
    );

    (UpgradeStreamsInTest().get_mut())[id.as_usize()].stream = Some(stream);
}

#[cfg(not(test))] pub async fn assume_upgradable(id: UpgradeID) -> TcpStream {
    struct AssumeUpgradable{id: UpgradeID}
    impl Future for AssumeUpgradable {
        type Output = TcpStream;
        fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
            let Some(StreamCell { reserved, stream }) = (unsafe {UpgradeStreams().get_mut()}).get_mut(self.id.as_usize())
                else {cx.waker().wake_by_ref(); return std::task::Poll::Pending};

            if stream.is_none()
                {cx.waker().wake_by_ref(); return std::task::Poll::Pending};

            *reserved = false;

            std::task::Poll::Ready(unsafe {stream.take().unwrap_unchecked()})
        }
    }

    AssumeUpgradable{id}.await
}
#[cfg(test)] pub async fn assume_upgradable_in_test(id: UpgradeID) -> TestStream {
    struct AssumeUpgradableInTest{id: UpgradeID}
    impl Future for AssumeUpgradableInTest {
        type Output = TestStream;
        fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
            let Some(StreamCell { reserved, stream }) = (unsafe {UpgradeStreamsInTest().get_mut()}).get_mut(self.id.as_usize())
                else {cx.waker().wake_by_ref(); return std::task::Poll::Pending};

            if stream.is_none()
                {cx.waker().wake_by_ref(); return std::task::Poll::Pending};

            *reserved = false;

            std::task::Poll::Ready(unsafe {stream.take().unwrap_unchecked()})
        }
    }

    AssumeUpgradableInTest{id}.await
}


static UPGRADE_STREAMS: OnceLock<UpgradeStreams> = OnceLock::new();
#[cfg(test)] static UPGRADE_STREAMS_IN_TEST: OnceLock<UpgradeStreams<TestStream>> = OnceLock::new();

#[allow(non_snake_case)] fn UpgradeStreams() -> &'static UpgradeStreams {
    UPGRADE_STREAMS.get_or_init(UpgradeStreams::new)
}
#[cfg(test)] #[allow(non_snake_case)] fn UpgradeStreamsInTest() -> &'static UpgradeStreams<TestStream> {
    UPGRADE_STREAMS_IN_TEST.get_or_init(UpgradeStreams::<TestStream>::new)
}

struct UpgradeStreams<Stream = TcpStream> {
    in_scanning: AtomicBool,
    streams:     UnsafeCell<Vec<StreamCell<Stream>>>,
} const _: () = {
    unsafe impl<Stream> Sync for UpgradeStreams<Stream> {}

    impl<Stream> UpgradeStreams<Stream> {
        fn new() -> Self {
            Self {
                in_scanning: AtomicBool::new(false),
                streams:     UnsafeCell::new(Vec::new()),
            }
        }
        fn get(&self) -> &Vec<StreamCell<Stream>> {
            unsafe {&*self.streams.get()}
        }
        unsafe fn get_mut(&self) -> &mut Vec<StreamCell<Stream>> {
            &mut *self.streams.get()
        }
        fn request_reservation(&self) -> Option<ReservationLock<'_, Stream>> {
            self.in_scanning.compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
                .ok().and(Some(ReservationLock(unsafe {self.get_mut()})))
        }
    }

    struct ReservationLock<'scan, Stream = TcpStream>(&'scan mut Vec<StreamCell<Stream>>);
    impl<'scan, Stream> Drop for ReservationLock<'scan, Stream> {
        fn drop(&mut self) {
            UpgradeStreams().in_scanning.store(false, Ordering::Release)
        }
    }
    impl<'scan, Stream> std::ops::Deref for ReservationLock<'scan, Stream> {
        type Target = Vec<StreamCell<Stream>>;
        fn deref(&self) -> &Self::Target {&*self.0}
    }
    impl<'scan, Stream> std::ops::DerefMut for ReservationLock<'scan, Stream> {
        fn deref_mut(&mut self) -> &mut Self::Target {self.0}
    }
}; 

struct StreamCell<Stream = TcpStream> {
    reserved: bool,
    stream:   Option<Stream>,
} const _: () = {
    impl<Stream> StreamCell<Stream> {
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
