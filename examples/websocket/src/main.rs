use ohkami::prelude::*;
use ohkami::ws::{WebSocketContext, WebSocket, Message};


async fn echo_text(c: WebSocketContext<'_>) -> WebSocket {
    c.upgrade(|mut c| async move {
        while let Ok(Some(Message::Text(text))) = c.recv().await {
            if text == "close" {
                break
            }
            c.send(text).await.expect("Failed to send text");
        }
    })
}


async fn echo_text_2(
    Path(name): Path<String>,
    ctx: WebSocketContext<'_>
) -> EchoTextSession<'_> {
    EchoTextSession { name, ctx }
}

struct EchoTextSession<'ws> {
    name: String,
    ctx: WebSocketContext<'ws>,
}
impl IntoResponse for EchoTextSession<'_> {
    fn into_response(self) -> Response {
        self.ctx.upgrade(|mut c| async move {
            c.send(format!("Hello, {}!", self.name)).await.expect("failed to send");
        
            while let Ok(Some(Message::Text(text))) = c.recv().await {
                if text == "close" {
                    break
                }
                c.send(text).await.expect("failed to send text");
            }
        }).into_response()
    }
}


async fn echo_text_3(
    Path(name): Path<String>,
    ctx: WebSocketContext<'_>
) -> WebSocket {
    ctx.upgrade(|c| async {
        let (mut r, mut w) = c.split();

        let incoming = std::sync::Arc::new(tokio::sync::RwLock::new(std::collections::VecDeque::new()));
        let (close_tx, close_rx) = tokio::sync::watch::channel(());

        tokio::try_join! {
            tokio::spawn({
                let (close, incoming) = (close_rx.clone(), incoming.clone());
                async move {
                    loop {
                        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                        println!("close: {close:?}, incoming: {incoming:?}");

                        if !close.has_changed().is_ok_and(|yes|!yes) {println!("break 1"); break}
                    }
                }
            }),
            tokio::spawn({
                let (mut close, incoming) = (close_rx.clone(), incoming.clone());
                async move {
                    loop {
                        tokio::select! {
                            _ = close.changed() => {
                                println!("break 2"); break
                            }
                            recv = r.recv() => {
                                if let Ok(Some(Message::Text(text))) = recv {
                                    {let mut incoming = incoming.write().await;
                                        incoming.push_back(text);
                                    }
                                }
                            }
                        }
                    }
                }
            }),
            tokio::spawn({
                let (name, close, closer, incoming) = (name, close_rx.clone(), close_tx, incoming.clone());
                async move {
                    w.send(Message::Text(format!("Hello, {name}!"))).await.expect("failed to send");

                    loop {
                        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

                        w.send("tick").await.expect("failed to send");
                        
                        let poped = {
                            let mut incoming = incoming.write().await;
                            let poped = incoming.pop_front();
                            dbg!(poped)
                        };
                        if let Some(text) = poped {
                            if text == "close" {closer.send(()).unwrap()}
                            
                            w.send(text).await.expect("failed to send");
                        }

                        if !close.has_changed().is_ok_and(|yes|!yes) {println!("break 3"); break}
                    }
                }
            })
        }.inspect_err(|e| println!("error in echo_text_3: {e}")).ok();
    })
}


async fn echo4(Path(name): Path<String>, ws: WebSocketContext<'_>) -> WebSocket {
    ws.upgrade(|mut c| async {
        /* spawn but not join the handle */
        tokio::spawn(async move {
            #[cfg(feature="DEBUG")] println!("\n{c:#?}");

            c.send(name).await.expect("failed to send");
            while let Ok(Some(Message::Text(text))) = c.recv().await {
                #[cfg(feature="DEBUG")] println!("\n{c:#?}");

                if dbg!(&text) == "close" {break}
                c.send(text).await.expect("failed to send");
            }
        });
    })
}


#[tokio::main]
async fn main() {
    #[derive(Clone)]
    struct Logger;
    impl FangAction for Logger {
        async fn fore<'a>(&'a self, req: &'a mut Request) -> Result<(), Response> {
            Ok(println!("\n{req:#?}"))
        }
        async fn back<'a>(&'a self, res: &'a mut Response) {
            println!("\n{res:#?}")
        }
    }
    
    #[cfg(feature="tls")]
    let tls_config = {        
        use rustls::ServerConfig;
        use rustls::pki_types::{CertificateDer, PrivateKeyDer};
        use std::fs::File;
        use std::io::BufReader;
        
        // Initialize rustls crypto provider
        rustls::crypto::ring::default_provider().install_default()
            .expect("Failed to install rustls crypto provider");
    
        // Load certificates and private key
        let cert_file = File::open("./cert.pem").expect("Failed to open certificate file");
        let key_file = File::open("./key.pem").expect("Failed to open private key file");
        
        let cert_chain = rustls_pemfile::certs(&mut BufReader::new(cert_file))
            .map(|cd| cd.map(CertificateDer::from))
            .collect::<Result<Vec<_>, _>>()
            .expect("Failed to read certificate chain");
        
        let key = rustls_pemfile::read_one(&mut BufReader::new(key_file))
            .expect("Failed to read private key")
            .map(|p| match p {
                rustls_pemfile::Item::Pkcs1Key(k) => PrivateKeyDer::Pkcs1(k),
                rustls_pemfile::Item::Pkcs8Key(k) => PrivateKeyDer::Pkcs8(k),
                _ => panic!("Unexpected private key type"),
            })
            .expect("Failed to read private key");
    
        // Build TLS configuration
        ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(cert_chain, key)
            .expect("Failed to build TLS configuration")
    };
    
    let o = Ohkami::new((
        Logger,
        "/".Mount("./template").omit_extensions(&["html"]),
        "/echo1".GET(echo_text),
        "/echo2/:name".GET(echo_text_2),
        "/echo3/:name".GET(echo_text_3),
        "/echo4/:name".GET(echo4),
    ));
    
    o.howl("localhost:3030", #[cfg(feature="tls")] tls_config).await;
}
