pub use send_bound::*;

#[cfg(feature = "__threaded__")]
mod send_bound {
    pub trait SendOnThreaded: Send {}
    impl<T: Send> SendOnThreaded for T {}
}

#[cfg(not(feature = "__threaded__"))]
mod send_bound {
    pub trait SendOnThreaded {}
    impl<T> SendOnThreaded for T {}
}
