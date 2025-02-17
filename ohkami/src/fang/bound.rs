use super::FangProcCaller;
use std::future::Future;


pub use dispatch::*;

#[cfg(not(feature="rt_worker"))]
mod dispatch {
    pub trait SendSyncOnNative: Send + Sync {}
    impl<T: Send + Sync> SendSyncOnNative for T {}

    #[allow(unused)]
    pub trait SendOnNative: Send {}
    impl<T: Send> SendOnNative for T {}
}
#[cfg(feature="rt_worker")]
mod dispatch {
    pub trait SendSyncOnNative {}
    impl<T> SendSyncOnNative for T {}

    pub trait SendOnNative {}
    impl<T> SendOnNative for T {}
}

#[allow(unused)]
pub trait SendOnNativeFuture<T>: Future<Output = T> + SendOnNative {}
impl<T, F: Future<Output = T> + SendOnNative> SendOnNativeFuture<T> for F {}

pub(crate) trait FPCBound: FangProcCaller + SendSyncOnNative {}
impl<T: FangProcCaller + SendSyncOnNative> FPCBound for T {}
