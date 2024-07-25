use ohkami::prelude::*;
use ohkami::ws::{WebSocketContext, WebSocket, Message};


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


async fn echo_text(c: WebSocketContext<'_>) -> WebSocket {
    c.connect(|mut ws| async move {
        while let Ok(Some(Message::Text(text))) = ws.recv().await {
            if text == "close" {
                break
            }
            ws.send(Message::Text(text)).await.expect("Failed to send text");
        }
    })
}


async fn echo_text_2(
    name: String,
    ctx:  WebSocketContext<'_>
) -> EchoTextSession<'_> {
    EchoTextSession { name, ctx }
}

struct EchoTextSession<'ws> {
    name: String,
    ctx:  WebSocketContext<'ws>,
}
impl IntoResponse for EchoTextSession<'_> {
    fn into_response(self) -> Response {
        self.ctx.connect(|mut ws| async move {
                ws.send(Message::Text(format!("Hello, {}!", self.name))).await.expect("failed to send");
        
                while let Ok(Some(Message::Text(text))) = ws.recv().await {
                    if text == "close" {
                        break
                    }
                    ws.send(Message::Text(text)).await.expect("failed to send text");
                }
        }).into_response()
    }
}


async fn echo_text_3(name: String,
    ctx: WebSocketContext<'_>
) -> WebSocket {
    ctx.connect(|ws| async {
        let (mut r, mut w) = ws.split();
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
            tokio::task::spawn({
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
            tokio::task::spawn({
                let (name, close, closer, incoming) = (name, close_rx.clone(), close_tx, incoming.clone());
                async move {
                    w.send(Message::Text(format!("Hello, {name}!"))).await.expect("failed to send");

                    loop {
                        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

                        w.send(Message::Text(format!("tick"))).await.expect("failed to send");
                        
                        let poped = {
                            let mut incoming = incoming.write().await;
                            let poped = incoming.pop_front();
                            dbg!(poped)
                        };
                        if let Some(text) = poped {
                            if text == "close" {closer.send(()).unwrap()}
                            
                            w.send(Message::Text(text)).await.expect("failed to send");
                        }

                        if !close.has_changed().is_ok_and(|yes|!yes) {println!("break 3"); break}
                    }
                }
            })
        }.inspect_err(|e| println!("error in echo_text_3: {e}")).ok();
    })
}

#[tokio::main]
async fn main() {
    Ohkami::with(Logger, (
        "/".Dir("./template").omit_extensions([".html"]),
        "/echo1".GET(echo_text),
        "/echo2/:name".GET(echo_text_2),
        "/echo3/:name".GET(echo_text_3),
    )).howl("localhost:3030").await
}
