#![allow(unused/* just for sample of WebSocket with `ohkami::DurableObject` */)]

use ohkami::serde::{Serialize, Deserialize};
use ohkami::ws::SessionMap;
use ohkami::DurableObject; // <--

#[DurableObject]
struct Room {
    name:     Option<String>,
    state:    worker::State,
    sessions: SessionMap<Session>,
}

#[derive(Serialize, Deserialize)]
struct Session {
    username: String,
}

impl DurableObject for Room {
    fn new(state: worker::State, env: worker::Env) -> Self {
        let mut sessions = SessionMap::new();

        // restore sessions if woken up from hibernation
        for ws in state.get_websockets() {
            if let Ok(Some(session)) = ws.deserialize_attachment() {
                sessions.insert(ws, session).unwrap();
            }
        }
        
        Self { name: None, state, sessions }
    }

    async fn fetch(
        &mut self,
        req: worker::Request
    ) -> worker::Result<worker::Response> {
        todo!()
    }

    async fn websocket_message(
        &mut self,
        ws:      worker::WebSocket,
        message: worker::WebSocketIncomingMessage,
    ) -> worker::Result<()> {
        todo!()
    }
}
