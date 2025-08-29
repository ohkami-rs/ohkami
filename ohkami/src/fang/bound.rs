use super::FangProcCaller;
use std::future::Future;


pub use dispatch::*;

#[cfg(feature="__rt_threaded__")]
mod dispatch {
    pub trait SendSyncOnThreaded: Send + Sync {}
    impl<T: Send + Sync> SendSyncOnThreaded for T {}

    #[allow(unused)]
    pub trait SendOnThreaded: Send {}
    impl<T: Send> SendOnThreaded for T {}
}
#[cfg(not(feature="__rt_threaded__"))]
mod dispatch {
    pub trait SendSyncOnThreaded {}
    impl<T> SendSyncOnThreaded for T {}

    pub trait SendOnThreaded {}
    impl<T> SendOnThreaded for T {}
}

#[allow(unused)]
pub trait SendOnThreadedFuture<T>: Future<Output = T> + SendOnThreaded {}
impl<T, F: Future<Output = T> + SendOnThreaded> SendOnThreadedFuture<T> for F {}

pub(crate) trait FPCBound: FangProcCaller + SendSyncOnThreaded {}
impl<T: FangProcCaller + SendSyncOnThreaded> FPCBound for T {}
