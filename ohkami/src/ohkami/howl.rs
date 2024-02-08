use std::{sync::Arc};
use super::{Ohkami};
use crate::{__rt__, Session};

#[cfg(feature="rt_async-std")] use crate::__rt__::StreamExt;
#[cfg(feature="websocket")]    use crate::websocket::reserve_upgrade;


impl Ohkami {
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
