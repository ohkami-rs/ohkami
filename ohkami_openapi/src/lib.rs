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

/// handle the `schema` as a component schema named `name`.
/// This is useful for reusing schemas in the OpenAPI document.
pub fn component<T: schema::Type::SchemaType>(name: &'static str, schema: schema::Schema<T>) -> schema::Schema<T> {
    schema::Schema::component(name, schema)
}

/// `type: string`
pub fn string() -> schema::Schema<schema::Type::string> {
    schema::Schema::string()
}
/// `type: number`
pub fn number() -> schema::Schema<schema::Type::number> {
    schema::Schema::number()
}
/// `type: integer`
pub fn integer() -> schema::Schema<schema::Type::integer> {
    schema::Schema::integer()
}
/// `type: boolean`
pub fn bool() -> schema::Schema<schema::Type::bool> {
    schema::Schema::bool()
}
/// ```txt
/// type: array
/// items:
///   type: ###`items`'s schema###
/// ```
pub fn array(items: impl Into<schema::SchemaRef>) -> schema::Schema<schema::Type::array> {
    schema::Schema::array(items)
}
/// `type: object`
pub fn object() -> schema::Schema<schema::Type::object> {
    schema::Schema::object()
}
/// `anyOf: [...{schemas}]`
pub fn any_of(schemas: impl schema::SchemaList) -> schema::Schema<schema::Type::any> {
    schema::Schema::any_of(schemas)
}
/// `allOf: [...{schemas}]`
pub fn all_of(schemas: impl schema::SchemaList) -> schema::Schema<schema::Type::any> {
    schema::Schema::all_of(schemas)
}
/// `oneOf: [...{schemas}]`
pub fn one_of(schemas: impl schema::SchemaList) -> schema::Schema<schema::Type::any> {
    schema::Schema::one_of(schemas)
}

/// # OpenAPI Schema trait
/// 
/// ## Required
/// 
/// - `schema() -> impl Into<schema::SchemaRef>`
///   - this `impl Into<schema::SchemaRef>` mostly means `schema::Schema<{something}>`.
/// 
/// ## Implementation Notes
/// 
/// Generally, you can implement this trait for your types by `#[derive(Schema)]`.
/// See it's documentation for more details.
/// 
/// But of course, you can implement it manually if you want to.
/// In that case, start from **base schemas**:
/// 
/// - [`string()`](fn@string)
/// - [`number()`](fn@number)
/// - [`integer()`](fn@integer)
/// - [`bool()`](fn@bool)
/// - [`array({items})`](fn@array)
/// - [`object()`](fn@object)
/// - [`any_of({schemas})`](fn@any_of)
/// - [`all_of({schemas})`](fn@all_of)
/// - [`one_of({schemas})`](fn@one_of)
/// 
/// and, [`component({name}, {schema})`](fn@component) if you want to name and reuse
/// the schema in the OpenAPI document.
/// 
/// ## Example
/// 
/// ```rust,ignore
/// use ohkami::openapi;
/// 
/// #[derive(openapi::Schema)]
/// struct MySchema {
///     pub id: u32,
///     pub name: String,
///     pub age: Option<u8>,
/// }
/// /* equivalent to: */
/// impl openapi::Schema for MySchema {
///     fn schema() -> impl Into<openapi::schema::SchemaRef> {
///         openapi::object()
///             .property("id", openapi::integer().format("uint32"))
///             .property("name", openapi::string())
///             .optional("age", openapi::integer().format("uint8"))
///     }
/// }
/// 
/// #[derive(openapi::Schema)]
/// #[openapi(component)]
/// struct MyComponentSchema {
///     pub id: u32,
///     pub name: String,
///     pub age: Option<u8>,
/// }
/// /* equivalent to: */
/// impl openapi::Schema for MyComponentSchema {
///     fn schema() -> impl Into<openapi::schema::SchemaRef> {
///         openapi::component("MyComponentSchema",  openapi::object()
///             .property("id", openapi::integer().format("uint32"))
///             .property("name", openapi::string())
///             .optional("age", openapi::integer().format("uint8"))
///         )
///     }
/// }
/// ```
/// 
/// ## Default Implementations
/// 
/// - `str`, `String`
/// - `u8`, `u16`, `u32`, `u64`, `usize`
/// - `i8`, `i16`, `i32`, `i64`, `isize`
/// - `f32`, `f64`
/// - `uuid::Uuid`
/// - `Vec<S>`, `[S]`, `[S; N]`, `Cow<'_, S>`, `Arc<S>`, `&S` where `S: Schema`
pub trait Schema {
    fn schema() -> impl Into<schema::SchemaRef>;
}
const _: () = {
    impl Schema for str {
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
            integer().format("uint8")
        }
    }
    impl Schema for u16 {
        fn schema() -> impl Into<schema::SchemaRef> {
            integer().format("uint16")
        }
    }
    impl Schema for u32 {
        fn schema() -> impl Into<schema::SchemaRef> {
            integer().format("uint32")
        }
    }
    impl Schema for u64 {
        fn schema() -> impl Into<schema::SchemaRef> {
            integer().format("uint64")
        }
    }
    impl Schema for usize {
        fn schema() -> impl Into<schema::SchemaRef> {
            integer()
        }
    }

    impl Schema for i8 {
        fn schema() -> impl Into<schema::SchemaRef> {
            integer().format("int8")
        }
    }
    impl Schema for i16 {
        fn schema() -> impl Into<schema::SchemaRef> {
            integer().format("int16")
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
    
    impl Schema for uuid::Uuid {
        fn schema() -> impl Into<schema::SchemaRef> {
            string().format("uuid")
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

    impl<S: Schema + ToOwned + ?Sized> Schema for std::borrow::Cow<'_, S> {
        fn schema() -> impl Into<schema::SchemaRef> {
            S::schema()
        }
    }
    impl<S: Schema + ?Sized> Schema for std::sync::Arc<S> {
        fn schema() -> impl Into<schema::SchemaRef> {
            S::schema()
        }
    }

    impl<S: Schema + ?Sized> Schema for &S {
        fn schema() -> impl Into<schema::SchemaRef> {
            S::schema()
        }
    }
};
