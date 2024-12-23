#![allow(non_snake_case, non_camel_case_types)]

mod _util;
#[cfg(test)] mod _test;

pub mod schema;

pub mod security;

pub mod request;
pub use request::{Parameter, RequestBody};

pub mod response;
pub use response::{Responses, Response};

pub mod paths;
pub use paths::Operation;

pub mod document;

pub enum Input {
    Param(Parameter),
    Params(Vec<Parameter>),
    Body(RequestBody),
}

pub trait Schema {
    fn schema() -> impl Into<schema::SchemaRef>;
}

pub fn component<T: schema::Type::SchemaType>(name: &'static str, schema: schema::Schema<T>) -> schema::Schema<T> {
    schema::Schema::component(name, schema)
}

pub fn string() -> schema::Schema<schema::Type::string> {
    schema::Schema::string()
}
pub fn number() -> schema::Schema<schema::Type::number> {
    schema::Schema::number()
}
pub fn integer() -> schema::Schema<schema::Type::integer> {
    schema::Schema::integer()
}
pub fn bool() -> schema::Schema<schema::Type::bool> {
    schema::Schema::bool()
}
pub fn array() -> schema::Schema<schema::Type::array> {
    schema::Schema::array()
}
pub fn object() -> schema::Schema<schema::Type::object> {
    schema::Schema::object()
}
pub fn anyOf<const N: usize>(schema_refs: [&'static str; N]) -> schema::Schema<schema::Type::any> {
    schema::Schema::anyOf(schema_refs)
}
pub fn allOf<const N: usize>(schema_refs: [&'static str; N]) -> schema::Schema<schema::Type::any> {
    schema::Schema::allOf(schema_refs)
}
pub fn oneOf<const N: usize>(schema_refs: [&'static str; N]) -> schema::Schema<schema::Type::any> {
    schema::Schema::oneOf(schema_refs)
}
