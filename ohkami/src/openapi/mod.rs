#![cfg(debug_assertions)]
#![allow(non_snake_case, non_camel_case_types)]

mod _util;
#[cfg(test)] mod _test;

pub mod schema;
pub use schema::Schema;

pub mod request;
pub use request::RequestBody;

pub mod response;
