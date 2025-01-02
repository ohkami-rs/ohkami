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
pub fn array(items: impl Into<schema::SchemaRef>) -> schema::Schema<schema::Type::array> {
    schema::Schema::array(items)
}
pub fn object() -> schema::Schema<schema::Type::object> {
    schema::Schema::object()
}
pub fn anyOf(schemas: impl schema::SchemaList) -> schema::Schema<schema::Type::any> {
    schema::Schema::anyOf(schemas)
}
pub fn allOf(schemas: impl schema::SchemaList) -> schema::Schema<schema::Type::any> {
    schema::Schema::allOf(schemas)
}
pub fn oneOf(schemas: impl schema::SchemaList) -> schema::Schema<schema::Type::any> {
    schema::Schema::oneOf(schemas)
}

pub trait Schema {
    fn schema() -> impl Into<schema::SchemaRef>;
}
const _: () = {
    impl<S: Schema> Schema for Vec<S> {
        fn schema() -> impl Into<schema::SchemaRef> {
            array(S::schema())
        }
    }

    impl<S: Schema> Schema for [S] {
        fn schema() -> impl Into<schema::SchemaRef> {
            array(S::schema())
        }
    }
    
    impl<const N: usize, S: Schema> Schema for [S; N] {
        fn schema() -> impl Into<schema::SchemaRef> {
            array(S::schema())
        }
    }
};
