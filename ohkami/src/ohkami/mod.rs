mod config;

use async_std::{net::TcpListener, stream::StreamExt, task};
use config::CONFIG;
use tracing_subscriber::util::SubscriberInitExt;
use crate::{router::Router, fang::Fangs, handler::Handler, context::store::Store};

pub struct Ohkami<'router> {
    router: Router<'router>,
}

impl<'router> Ohkami<'router> {
    #[inline] pub fn new<const N: usize>(handlers: [Handler; N]) -> Self {
        let mut router = Router::new();
        for handler in handlers {
            router.register(handler)
        }
        Self { router }
    }
    #[inline] pub fn with<const N: usize>(fangs: Fangs, handlers: [Handler; N]) -> Self {
        let mut router = Router::new();
        for handler in handlers {
            router.register(handler)
        }
        router.apply(fangs);
        Self { router }
    }

    pub async fn howl(mut self, tcp_address: &'static str) -> crate::Result<()> {
        if let Some(subscriber) = CONFIG.0.get_mut().log_subscribe {
            subscriber.init()
        }

        let store = Store::new();
        let address = {
            if tcp_address.starts_with(":") {
                "0.0.0.0".to_owned() + tcp_address
            } else if tcp_address.starts_with("localhost") {
                tcp_address.replace("localhost", "127.0.0.1")
            } else {
                tcp_address.to_owned()
            }
        };

        let listener = TcpListener::bind(address).await?;
        tracing::info!("Ohkami started on {address}");

        while let Some(stream) = listener.incoming().next().await {
            let stream = stream?;
            
        }

        Ok(())
    }
}
