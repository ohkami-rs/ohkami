use super::{Ohkami};

use {
    std::sync::Arc,
    crate::{__rt__, Session},
};

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
    /// ```no_run
    /// use ohkami::prelude::*;
    /// use ohkami::typed::status::NoContent;
    /// 
    /// async fn hello() -> &'static str {
    ///     "Hello, ohkami!"
    /// }
    /// 
    /// async fn health_check() -> NoContent {
    ///     NoContent
    /// }
    /// 
    /// #[tokio::main]
    /// async fn main() {
    ///     Ohkami::new((
    ///         "/".GET(hello),
    ///         "/healthz".GET(health_check),
    ///     )).howl("localhost:5000").await
    /// }
    /// ```
    #[cfg(any(feature="rt_tokio", feature="rt_async-std"))]
    pub async fn howl(self, address: impl __rt__::ToSocketAddrs) {
        let router = Arc::new(self.into_router().into_radix());
        
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
