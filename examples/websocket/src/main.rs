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
        #[cfg(feature="DEBUG")] {
            println!("WebSocket handler is called");
        }

        #[cfg(feature="DEBUG")] {
            loop {
                let r = dbg!(ws.recv().await);
                let Ok(Some(Message::Text(text))) = r else {
                    break
                };
                println!("recv: `{text}`");
                ws.send(Message::Text(text)).await.expect("Failed to send text");
            }
        }
        #[cfg(not(feature="DEBUG"))] {
            while let Ok(Some(Message::Text(text))) = ws.recv().await {
                ws.send(Message::Text(text)).await.expect("Failed to send text");
            }
        }
    })
}

#[tokio::main]
async fn main() {
    Ohkami::with(Logger, (
        "/".Dir("./template").omit_extensions([".html"]),
        "/echo".GET(echo_text),
    )).howl("localhost:3030").await
}
