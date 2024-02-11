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
    /// ```no_run
    /// use ohkami::prelude::*;
    /// 
    /// #[tokio::main]
    /// async fn main() {
    ///     Ohkami::new(
    ///         "/".GET(|| async {"Hello, world!"})
    ///     ).howl("localhost:3000").await
    /// }
    /// ```
    pub async fn howl(self, address: impl __rt__::ToSocketAddrs) {
        self.howl_with((), address).await
    }

    /// `howl` with *global fang*s, that are fangs executed regardless of request's path.
    /// 
    /// <br>
    /// 
    /// *example.rs*
    /// ```no_run
    /// use ohkami::prelude::*;
    /// 
    /// struct LogRequest;
    /// impl FrontFang for LogRequest {
    ///     async fn bite(&self, req: &mut Request) -> Result<(), Response> {
    ///         println!("{req:?}");
    ///         Ok(())
    ///     }
    /// }
    /// 
    /// struct CustomNotFound;
    /// impl BackFang for CustomNotFound {
    ///     async fn bite(&self, res: &mut Response, _req: &Request) -> Result<(), Response> {
    ///         if res.status == Status::NotFound {
    ///             res.set_html(r#"
    ///                 <!DOCTYPE html>
    ///                 <html lang="en">
    ///                     <title>The page is not found</title>
    ///                     <body>
    ///                         <h1>Not Found</h1>
    ///                         <p>
    ///                             Something has triggered missing webpage on this website.
    ///                             This is custom 404 error page for <strong>ohkami</strong>.
    ///                          </p>
    ///                     </body>
    ///                 </html>
    ///             "#);
    ///         }
    /// 
    ///         Ok(())
    ///     }
    /// }
    /// 
    /// #[tokio::main]
    /// async fn main() {
    ///     let hello_ohkami = Ohkami::new(
    ///         "/".GET(|| async {"Hello, world!"})
    ///     );
    /// 
    ///     hello_ohkami.howl_with(
    ///         (LogRequest, CustomNotFound),
    ///         "localhost:5000"
    ///     ).await
    /// }
    /// ```
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
