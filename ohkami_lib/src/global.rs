use std::{sync::OnceLock, panic::RefUnwindSafe, ops::Deref, future::Future};

use async_std::task::block_on;

pub struct Global<T, F: FnOnce()->T + Clone = fn()->T> {
    value: OnceLock<T>,
    initializer: F,
}
impl<T, F: FnOnce()->T + Clone> Global<T, F> {
    pub const fn new(initializer: F) -> Self {
        Self { initializer, value: OnceLock::new() }
    }
    pub const fn wait<
        Fut:  Future<Output = T>,
        Init: FnOnce()->Fut,
    >(initializer: F) -> Self {
        fn f() -> T {
            block_on(initializer())
        }
        Self {
            value: OnceLock::new(),
            initializer: || block_on(initializer()),
        }
    }
}
const _: () = {
    impl<T, F: FnOnce()->T + Clone> RefUnwindSafe for Global<T, F> {}

    impl<T, F: FnOnce()->T + Clone> Deref for Global<T, F> {
        type Target = T;
        fn deref(&self) -> &Self::Target {
            self.value.get_or_init(|| self.initializer.clone()())
        }
    }
};



// trait Initializer<F; > {
//     fn initialize(&self) -> 
// }
// 