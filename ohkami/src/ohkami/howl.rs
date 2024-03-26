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
    /// 
    /// struct LogRequest;
    /// impl FrontFang for LogRequest {
    ///     type Error = std::convert::Infallible;
    ///     async fn bite(&self, req: &mut Request) -> Result<(), Self::Error> {
    ///         println!("{req:?}");
    ///         Ok(())
    ///     }
    /// }
    /// 
    /// struct CustomNotFound;
    /// impl BackFang for CustomNotFound {
    ///     type Error = std::convert::Infallible;
    ///     async fn bite(&self, res: &mut Response, _req: &Request) -> Result<(), Self::Error> {
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
    ///     let hello_ohkami = Ohkami::with((
    ///         LogRequest, CustomNotFound
    ///     ),
    ///         "/".GET(|| async {"Hello, world!"})
    ///     );
    /// 
    ///     hello_ohkami.howl("localhost:5000").await
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
