use std::{sync::Arc};
use super::{Ohkami};
use crate::{__rt__, Session, fang::Fangs};

#[cfg(feature="rt_async-std")] use crate::__rt__::StreamExt;
#[cfg(feature="websocket")]    use crate::websocket::reserve_upgrade;


impl Ohkami {
    /// Start serving at `address`!
    /// 
    /// `address` is `{runtime}::net::ToSocketAddrs`：
    /// 
    /// - `tokio::net::ToSocketAddrs` if you use `tokio`
    /// - `async_std::net::ToSocketAddrs` if you use `async-std`
    /// 
    /// <br>
    /// 
    /// *example.rs*
    /// ```
    /// 
    /// ```
    pub async fn howl(self, address: impl __rt__::ToSocketAddrs) {
        self.howl_with((), address).await
    }

    pub async fn howl_with<T>(self, global_fangs: impl Fangs<T>, address: impl __rt__::ToSocketAddrs) {
        let mut router = self.into_router();
        for (methods, fang) in global_fangs.collect() {
            router.register_global_fang(methods, fang)
        }
        let router = Arc::new(router.into_radix());
        
        let listener = match __rt__::TcpListener::bind(address).await {
            Ok(listener) => listener,
            Err(e)       => panic!("Failed to bind TCP listener: {e}"),
        };

        #[cfg(feature="rt_async-std")]
        while let Some(connection) = listener.incoming().next().await {
            let Ok(connection) = connection else {continue};

            __rt__::task::spawn({
                Session::new(
                    router.clone(),
                    connection,
                ).manage()
            });
        }
        
        #[cfg(feature="rt_tokio")]
        loop {
            let Ok((connection, _)) = listener.accept().await else {continue};

            __rt__::task::spawn({
                Session::new(
                    router.clone(),
                    connection,
                ).manage()
            });
        }
    }
}
