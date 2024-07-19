use ohkami::prelude::*;
use ohkami::websocket::{WebSocketContext, WebSocket, Message};


async fn echo_text(c: WebSocketContext<'_>) -> WebSocket {
    c.connect(|mut ws| async move {
        while let Ok(Some(Message::Text(text))) = ws.recv().await {
            ws.send(Message::Text(text)).await.expect("Failed to send text");
        }
    })
}

#[tokio::main]
async fn main() {
    Ohkami::new((
        "/ws".GET(echo_text),
    )).howl("localhost:3030").await
}
