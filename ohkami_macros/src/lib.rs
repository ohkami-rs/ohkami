use proc_macro::TokenStream;
mod internals;

/// **required dependency**: `serde = { version = "1.0", features = ["derive"] }`\
/// ( If needed, copy & paste this into your \[dependencies\] of Cargo.toml )
/// 
/// ```no_run
/// use ohkami::prelude::*;
/// 
/// #[JSON]
/// struct User {
///     id:   u64,
///     name: String,
/// }
/// 
/// async fn handler(c: Context, payload: User) -> Result<Response> {
///     // ...
/// }
/// ```
#[proc_macro_attribute]
#[allow(non_snake_case)]
pub fn JSON( _: TokenStream, struct_stream: TokenStream) -> TokenStream {
    internals::JSON(struct_stream.into())
        .unwrap_or_else(|err| err.into_compile_error())
        .into()
}

// #[proc_macro_derive(JSON)]
// pub fn derive_json(struct_stream: TokenStream) -> TokenStream {
//     internals::derive_ohkami_json(struct_stream.into())
//         .unwrap_or_else(|err| err.into_compile_error())
//         .into()
// }