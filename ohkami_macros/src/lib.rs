mod components;

mod query;
mod payload;


/// ## Query parameters
/// 
/// <br/>
/// 
/// ```ignore
/// use ohkami::{Context, Response};
/// use ohkami::Queries; // <-- import me
/// 
/// #[Query]
/// struct HelloQuery {
///     name:     String,
///     n_repeat: Option<usize>,
/// }
/// 
/// async fn hello(c: Context, queries: HelloQuery) -> Response<String> {
///     let HelloQuery {name, n_repeat} = queries;
/// 
///     let message = match n_repeat {
///         None    => format!("Hello"),
///         Some(n) => format!("Hello, {name}! ").repeat(n),
///     };
/// 
///     c.Text(message)
/// }
/// ```
/// 
/// <br/>
/// 
/// - Possible value types : `String` `u8` `u16` `u32` `u64` `u128` `usize` and `Option` of them.
/// - NOT available for tuple struct ( like `struct S(usize, usize);` ) or tag struct ( like `struct X;` ).
/// 
/// If you need support for other structs or types, plaese let me know that in [GitHub issue](https://github.com/kana-rus/ohkami/issues) !
#[proc_macro_attribute] #[allow(non_snake_case)]
pub fn Query(_: proc_macro::TokenStream, data: proc_macro::TokenStream) -> proc_macro::TokenStream {
    query::Query(data.into())
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

/// ## Request payload
/// 
/// <br/>
/// 
/// ### Valid format :
/// 
/// - `#[Payload(JSON)]` ( for `application/json` )
/// - `#[Payload(URLEncoded)]` ( for `application/x-www-form-urlencoded` )
/// 
/// <br/>
/// 
/// ### Notification :
/// 
/// - In current version, `#[Payload(JSON)]` requires the struct impls `serde::Deserialize`.
/// - In current version, `#[Payload(Form)]` is not implemented yet. Please wait for development, or, if you need this imediately, you can implement and create a [Pull request](https://github.com/kana-rus/ohkami/pulls) !
/// 
/// <br/>
/// 
/// ```ignore
/// use ohkami::{Context, Response};
/// use ohkami::Payload; // <-- import me
/// 
/// #[Payload(JSON)]
/// #[derive(serde::Deserialize)] // <-- This may be not required in future version
/// struct HelloRequest {
///     name:     String,
///     n_repeat: Option<usize>,
/// }
/// 
/// async fn hello(c: Context, body: HelloRequest) -> Response<String> {
///     let HelloRequest {name, n_repeat} = queries;
/// 
///     let message = match n_repeat {
///         None    => format!("Hello"),
///         Some(n) => format!("Hello, {name}! ").repeat(n),
///     };
/// 
///     c.Text(message)
/// }
/// ```
/// 
/// <br/>
/// 
/// - NOT available for tuple struct ( like `struct S(usize, usize);` ) or tag struct ( like `struct X;` ).
/// - Possible value types : `String` `u8` `u16` `u32` `u64` `u128` `usize` and `Option` of them.
/// 
/// If you need support for other structs or types, plaese let me know that in [GitHub issue](https://github.com/kana-rus/ohkami/issues) !
#[proc_macro_attribute] #[allow(non_snake_case)]
pub fn Payload(format: proc_macro::TokenStream, data: proc_macro::TokenStream) -> proc_macro::TokenStream {
    payload::Payload(format.into(), data.into())
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}
