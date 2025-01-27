mod room;

use ohkami::prelude::*;
use ohkami::ws::{WebSocketContext, WebSocket};

#[ohkami::bindings]
struct Bindings;

#[ohkami::worker]
async fn main() -> Ohkami {
    Ohkami::new((
        "/ws/:room_name".GET(ws_chatroom),
    ))
}

async fn ws_chatroom(room_name: &str,
    ctx: WebSocketContext<'_>,
    Bindings { ROOMS, .. }: Bindings,
) -> Result<WebSocket, worker::Error> {
    let room = ROOMS
        .id_from_name(room_name)?
        .get_stub()?;
    ctx.upgrade_durable(room).await
}
