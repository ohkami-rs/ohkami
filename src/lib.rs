pub mod server;
pub mod result;
pub mod context;
pub mod response;
pub mod db;

pub(crate) mod components;
pub(crate) mod utils;


pub mod prelude {
    pub use super::{
        server::Server,
        result::{Result, ElseResponse, ElseResponseWithErr},
        context::Context,
        response::Response,
        components::json::JSON,
        db::useDB,
    };
}