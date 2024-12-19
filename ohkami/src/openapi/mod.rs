#![cfg(debug_assertions)]
#![allow(non_snake_case, non_camel_case_types)]

mod _util;
#[cfg(test)] mod _test;

pub mod schema;
pub use schema::Schema;

pub mod request;
pub use request::{Parameter, RequestBody};

pub mod response;
pub use response::{Responses, Response, ResponseHeader};

pub mod paths;
pub use paths::{Operations, Operation, ExternalDoc};

pub mod document;
pub use document::{Document, Server};

#[doc(hidden)]
pub mod private {
    use super::schema::SchemaRef;

    #[doc(hidden)]
    pub trait Schema {
        const NAME: &'static str;
        fn schema() -> impl Into<SchemaRef>;
    }
}
