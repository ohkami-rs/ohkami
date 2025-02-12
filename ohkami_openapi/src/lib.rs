#![allow(non_snake_case, non_camel_case_types)]

mod _util;
#[cfg(test)] mod _test;

pub mod schema;
pub use schema::SchemaRef;

pub mod security;
pub use security::SecurityScheme;

pub mod request;
pub use request::{Parameter, RequestBody};

pub mod response;
pub use response::{Responses, Response, Status};

pub mod paths;
pub use paths::Operation;

pub mod document;

pub enum Inbound {
    None,
    Param(Parameter),
    Params(Vec<Parameter>),
    Body(RequestBody),
    Security { scheme: SecurityScheme, scopes: &'static [&'static str] },
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
    impl Schema for &str {
        fn schema() -> impl Into<schema::SchemaRef> {
            string()
        }
    }
    impl Schema for String {
        fn schema() -> impl Into<schema::SchemaRef> {
            string()
        }
    }

    impl Schema for u8 {
        fn schema() -> impl Into<schema::SchemaRef> {
            integer()
        }
    }
    impl Schema for u16 {
        fn schema() -> impl Into<schema::SchemaRef> {
            integer()
        }
    }
    impl Schema for u32 {
        fn schema() -> impl Into<schema::SchemaRef> {
            integer()
        }
    }
    impl Schema for u64 {
        fn schema() -> impl Into<schema::SchemaRef> {
            integer()
        }
    }
    impl Schema for usize {
        fn schema() -> impl Into<schema::SchemaRef> {
            integer()
        }
    }

    impl Schema for i8 {
        fn schema() -> impl Into<schema::SchemaRef> {
            integer()
        }
    }
    impl Schema for i16 {
        fn schema() -> impl Into<schema::SchemaRef> {
            integer()
        }
    }
    impl Schema for i32 {
        fn schema() -> impl Into<schema::SchemaRef> {
            integer().format("int32")
        }
    }
    impl Schema for i64 {
        fn schema() -> impl Into<schema::SchemaRef> {
            integer().format("int64")
        }
    }
    impl Schema for isize {
        fn schema() -> impl Into<schema::SchemaRef> {
            integer()
        }
    }

    impl Schema for f32 {
        fn schema() -> impl Into<schema::SchemaRef> {
            number().format("float")
        }
    }
    impl Schema for f64 {
        fn schema() -> impl Into<schema::SchemaRef> {
            number().format("double")
        }
    }

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

    impl<S: Schema + ToOwned> Schema for std::borrow::Cow<'_, S> {
        fn schema() -> impl Into<schema::SchemaRef> {
            S::schema()
        }
    }
    impl<S: Schema> Schema for std::sync::Arc<S> {
        fn schema() -> impl Into<schema::SchemaRef> {
            S::schema()
        }
    }

    impl<S: Schema> Schema for &S {
        fn schema() -> impl Into<schema::SchemaRef> {
            S::schema()
        }
    }
};
