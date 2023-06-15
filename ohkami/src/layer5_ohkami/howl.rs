use std::sync::Arc;
use super::Ohkami;
use crate::{__dep__, __dep__::StreamIterater, Request, Context};


pub trait TCPAddress {
    fn parse(self) -> String;
} const _: () = {
    impl TCPAddress for u16 {
        fn parse(self) -> String {
            if !(self <= 49151) {panic!("port must be 0 〜 49151")}
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
    pub async fn howl(self, address: impl TCPAddress) {
        let router = Arc::new(
            self.routes.into_radix()
        );

        let listener = match __dep__::TcpListener::bind(address.parse()).await {
            Ok(listener) => listener,
            Err(e) => panic!("Failed to bind TCP listener: {e}"),
        };

        while let Some(Ok(mut stream)) = listener.incoming().next().await {
            let router = Arc::clone(&router);
            let c = Context::new();

            __dep__::task::spawn(async move {
                let req = Request::new(&mut stream).await;
                router.handle(c, req).await;
            }).await;
        }
    }
}
