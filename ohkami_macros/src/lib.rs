use proc_macro::TokenStream;
mod internals;

#[proc_macro_attribute]
pub fn consume_struct(_: TokenStream, derived_struct: TokenStream) -> TokenStream {
    internals::consume_struct(derived_struct.into())
        .unwrap_or_else(|err| err.into_compile_error())
        .into()
}

/// Implements `Json<'j>`, `serde::Serialize`, `serde::Deserialize<'de>`. Only structs implementing all of them can be handled as
/// - request body type by handler functions
/// - type of `application/json` response body by {`Context` / `Response`}`::`{`OK` / `Created`}
/// 
/// **required dependency**: `serde = { version = "1.0", features = ["derive"] }`\
/// ( If needed, copy & paste this into your \[dependencies\] of Cargo.toml )
/// 
/// ```no_run
/// use ohkami::prelude::*;
/// 
/// #[derive(JSON)]
/// struct User {
///     id:   u64,
///     name: String,
/// }
/// 
/// async fn handler(c: Context, payload: User) -> Result<Response> {
///     // ...
/// }
/// ```
#[proc_macro_derive(JSON)]
pub fn derive_json(struct_stream: TokenStream) -> TokenStream {
    internals::derive_json(struct_stream.into())
        .unwrap_or_else(|err| err.into_compile_error())
        .into()
}

#[proc_macro]
pub fn json_str(content: TokenStream) -> TokenStream {
    internals::json_str(content.into())
        .unwrap_or_else(|err| err.into_compile_error())
        .into()
}