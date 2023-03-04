mod config;

use async_std::{
    task,
    io::ReadExt,
    stream::StreamExt,
    sync::{Arc, Mutex},
    net::{TcpListener, TcpStream},
};
use crate::{
    fang::Fangs,
    handler::Handler,
    response::Response,
    context::{store::Store, Context},
    router::{trie_tree::TrieTree, Router},
    request::{REQUEST_BUFFER_SIZE, parse::parse_request, Request},
};

pub struct Ohkami<'router> {
    router: TrieTree<'router>,
}

impl Ohkami<'static> {
    pub fn default<const N: usize>(handlers: [Handler; N]) -> Self {
        let mut router = TrieTree::new(handlers);
        Self { router }
    }
    pub fn with<const N: usize>(fangs: Fangs, handlers: [Handler; N]) -> Self {
        let mut router = TrieTree::new(handlers);
        router.apply(fangs);
        Self { router }
    }

    pub async fn howl(self, tcp_address: &'static str) -> crate::Result<()> {
        let address = {
            if tcp_address.starts_with(":") {
                "0.0.0.0".to_owned() + tcp_address
            } else if tcp_address.starts_with("localhost") {
                tcp_address.replace("localhost", "127.0.0.1")
            } else {
                tcp_address.to_owned()
            }
        };
        
        let store = Arc::new(Mutex::new(Store::new()));
        let router = Arc::new(self.router.into_radix());

        let listener = TcpListener::bind(&address).await?;
        tracing::info!("ohkami started on {address}");

        while let Some(Ok(stream)) = listener.incoming().next().await {
            task::spawn(
                handle(stream, Arc::clone(&store), Arc::clone(&router))
            );
        }

        Ok(())
    }
}

#[inline] async fn handle<'router>(
    mut stream: TcpStream,
    store:      Arc<Mutex<Store>>,
    router:     Arc<Router<'router>>,
) {
    let mut c = Context::new(store);

    let mut buffer = [b' '; REQUEST_BUFFER_SIZE];
    if let Err(e) = stream.read(&mut buffer).await {
        tracing::error!("{e}"); panic!()
    }
    
    let (
        method, path, query_params, headers, body
    ) = parse_request(&buffer);
    
    let (
        fangs, path_params, handle_func
    ) = router.search(method, path);
    let mut request = Request {
        path_params,
        query_params,
        headers,
        body,
    };

    //for fang in fangs {
    //    (c, request) = fang(c, request).await;
    //}
    if let Some(handle_func) = handle_func {
        handle_func(stream, c, request).await
    } else {
        
    }
}
