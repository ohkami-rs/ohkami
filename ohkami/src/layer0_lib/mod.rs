mod list; pub(crate) use list::{List};
mod cors; pub use cors::{CORS};
mod slice; pub(crate) use slice::{Slice, CowSlice};
mod status; pub use status::{Status};
mod method; pub use method::{Method};
mod headers; pub(crate) use headers::{client as client_header, server as server_header};
mod datetime; pub use datetime::{now};
