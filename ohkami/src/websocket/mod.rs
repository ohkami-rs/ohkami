#[cfg(not(target_pointer_width = "64"))]
compile_error!{ "pointer width must be 64" }

mod websocket;
mod context;
mod message;
mod upgrade;
mod frame;

pub use {
    message::{Message},
    websocket::{WebSocket},
    context::{WebSocketContext, UpgradeError},
};
pub(crate) use {
    upgrade::{UpgradeID, request_upgrade_id, reserve_upgrade},
};
pub(crate) use upgrade::{
    assume_upgradable,
};
