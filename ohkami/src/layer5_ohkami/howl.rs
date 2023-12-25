use std::{sync::Arc, pin::Pin};
use super::{Ohkami};
use crate::{__rt__, Request, Context};

#[cfg(feature="rt_async-std")] use crate::__rt__::StreamExt;
#[cfg(feature="websocket")]    use crate::websocket::reserve_upgrade;


pub trait TCPAddress {
    fn parse(self) -> String;
} const _: () = {
    impl TCPAddress for u16 {
        fn parse(self) -> String {
            if self > 49151 {panic!("port must be 0 〜 49151")}
            format!("0.0.0.0:{self}")
        }
    }
    impl TCPAddress for &'static str {
        fn parse(self) -> String {
            if self.starts_with(":") {
                format!("0.0.0.0{self}")
            } else if self.starts_with("localhost") {
                self.replace("localhost", "127.0.0.1")
            } else {
                self.to_owned()
            }
        }
    }
};


impl Ohkami {
    /// address :
    /// 
    /// - u16 `PORT` like `8080`
    /// - literal `":{PORT}"` like `":5000"`
    /// - literal `"localhost:{PORT}"` like `"localhost:3000"`
    pub async fn howl(self, address: impl TCPAddress) {
        let router = Arc::new(self.into_router().into_radix());
        
        let listener = match __rt__::TcpListener::bind(address.parse()).await {
            Ok(listener) => listener,
            Err(e)       => panic!("Failed to bind TCP listener: {e}"),
        };

        #[cfg(feature="rt_async-std")]
        while let Some(Ok(mut stream)) = listener.incoming().next().await {
            let router = Arc::clone(&router);

            __rt__::task::spawn(async move {
                let mut req = Request::init();
                let mut req = unsafe {Pin::new_unchecked(&mut req)};
                req.as_mut().read(&mut stream).await;

                #[cfg(not(feature="websocket"))]
                let res = router.handle(Context::new(), req.get_mut()).await;
                #[cfg(feature="websocket")]
                let (res, upgrade_id) = router.handle(Context::new(), req.get_mut()).await;

                res.send(&mut stream).await;

                #[cfg(feature="websocket")]
                if let Some(id) = upgrade_id {
                    unsafe{reserve_upgrade(id, stream)}
                }
            }).await
        }
        
        #[cfg(feature="rt_tokio")]
        loop {
            let stream = Arc::new(__rt__::Mutex::new(
                match listener.accept().await {
                    Ok((stream, _)) => stream,
                    Err(e)          => panic!("Failed to accept TCP stream: {e}"),
                }
            ));

            match __rt__::task::spawn({
                let router = router.clone();
                let stream = stream.clone();
                
                async move {
                    let stream = &mut *stream.lock().await;

                    let mut req = Request::init();
                    let mut req = unsafe {Pin::new_unchecked(&mut req)};
                    req.as_mut().read(stream).await;

                    #[cfg(not(feature="websocket"))]
                    let res = router.handle(Context::new(), req.get_mut()).await;
                    #[cfg(feature="websocket")]
                    let (res, upgrade_id) = router.handle(Context::new(), req.get_mut()).await;

                    res.send(stream).await;

                    #[cfg(feature="websocket")]
                    upgrade_id
                }
            }).await {
                #[cfg(not(feature="websocket"))]
                Ok(_) => (),

                #[cfg(feature="websocket")]
                Ok(upgrade_id) => {
                    if let Some(id) = upgrade_id {
                        let stream = __rt__::Mutex::into_inner(Arc::into_inner(stream).unwrap());
                        unsafe{reserve_upgrade(id, stream)}
                    }
                }

                Err(e) => (|| async {
                    println!("Fatal error: {e}");
                    let res = Context::new().InternalServerError();
                    res.send(&mut *stream.lock().await).await
                })().await,
            }
        }
    }
}
