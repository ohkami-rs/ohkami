#[cfg(all(feature = "__tokio_io__", feature = "__futures_io__"))]
compile_error!("`__tokio_io__` and `__futures_io__` features can't be activated at once");

mod send;
pub use send::*;

mod driver;
