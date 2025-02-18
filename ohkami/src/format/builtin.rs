mod json;
pub use json::JSON;

mod multipart;
pub use multipart::{Multipart, File};

mod urlencoded;
pub use urlencoded::URLEncoded;

mod text;
pub use text::Text;

mod html;
pub use html::HTML;

mod query;
pub use query::Query;


#[cold] #[inline(never)]
fn reject(msg: impl std::fmt::Display) -> crate::Response {
    crate::Response::BadRequest().with_text(msg.to_string())
}

#[cfg(feature="openapi")]
pub mod bound {
    use crate::openapi;
    use serde::{Serialize, Deserialize};

    pub trait Schema: openapi::Schema {}
    impl<S: openapi::Schema> Schema for S {}

    pub trait Incoming<'req>: Deserialize<'req> + openapi::Schema {}
    impl<'req, T> Incoming<'req> for T where T: Deserialize<'req> + openapi::Schema {}

    pub trait Outgoing: Serialize + openapi::Schema {}
    impl<T> Outgoing for T where T: Serialize + openapi::Schema {}
}
#[cfg(not(feature="openapi"))]
pub mod bound {
    use serde::{Serialize, Deserialize};

    pub trait Schema: {}
    impl<S> Schema for S {}

    pub trait Incoming<'req>: Deserialize<'req> {}
    impl<'req, T> Incoming<'req> for T where T: Deserialize<'req> {}

    pub trait Outgoing: Serialize {}
    impl<T> Outgoing for T where T: Serialize {}
}
