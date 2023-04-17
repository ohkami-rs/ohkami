use async_std::{
    task,
    io::ReadExt,
    stream::StreamExt,
    sync::{Arc, Mutex},
    net::{TcpListener, TcpStream},
};
use crate::{
    fang::Fangs,
    handler::Handlers,
    context::{store::Store, Context},
    router::{trie_tree::TrieTree, Router},
    request::{REQUEST_BUFFER_SIZE, Request},
};

pub struct Ohkami {
    router: TrieTree,
}

impl Ohkami {
    pub fn default<const N: usize>(handlers: [Handlers; N]) -> Self {
        let router = TrieTree::new(handlers);
        Self { router }
    }
    pub fn with<const N: usize>(fangs: Fangs, handlers: [Handlers; N]) -> Self {
        let mut router = TrieTree::new(handlers);
        router.apply(fangs);
        Self { router }
    }

    pub async fn howl(
        self,
        tcp_address: &'static str,
    ) -> crate::Result<()> {
        let address = {
            if tcp_address.starts_with(":") {
                "0.0.0.0".to_owned() + tcp_address
            } else if tcp_address.starts_with("localhost") {
                tcp_address.replace("localhost", "127.0.0.1")
            } else {
                tcp_address.to_owned()
            }
        };
        
        let store = Arc::new(
            Mutex::new(
                Store::new()
            )
        );

        let router = Arc::new(
            self.router.into_radix()
        );

        let listener = TcpListener::bind(&address).await?;
        tracing::info!("ohkami started on {address}");

        while let Some(Ok(stream)) = listener.incoming().next().await {
            task::spawn({
                let buffer = [b' '; REQUEST_BUFFER_SIZE];
                handle(stream, buffer, Arc::clone(&store), Arc::clone(&router))
            });
        }

        Ok(())
    }
}


#[inline] async fn handle(
    mut stream: TcpStream,
    mut buffer: [u8; REQUEST_BUFFER_SIZE],
    store:      Arc<Mutex<Store>>,
    router:     Arc<Router>,
) {
    let c = Context::new(store);
    let request = {
        if let Err(e) = stream.read(&mut buffer).await {
            tracing::error!("{e}"); panic!()
        }
        Request::parse(buffer)
    };
    router.handle(c, stream, request).await
}
