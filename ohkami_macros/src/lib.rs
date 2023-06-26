mod components;

mod queries;
mod payload;


/// ## Query parameters
/// 
/// <br/>
/// 
/// ```ignore
/// use ohkami::{Context, Response};
/// use ohkami::Queries; // <-- import me
/// 
/// #[Queries]
/// struct HelloQueries<'name> {
///     name:     &'name str,
///     n_repeat: Option<usize>,
/// }
/// 
/// async fn hello<'q>(c: Context, queries: HelloQueries<'q>) -> Response<String> {
///     let HelloQueries {name, n_repeat} = queries;
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
/// - Possible value types : `&'lifetime str` `String` `u8` `u16` `u32` `u64` `u128` `usize` `i8` `i16` `i32` `i64` `i128` `isize`
/// 
/// If you need support for other structs or types, plaese let me know that in [GitHub issue](https://github.com/kana-rus/ohkami/issues) !
#[proc_macro_attribute] #[allow(non_snake_case)]
pub fn Queries(_: proc_macro::TokenStream, data: proc_macro::TokenStream) -> proc_macro::TokenStream {
    queries::Queries(data.into())
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
/// - `#[Payload(Form)]` ( for `multipart/form-data` )
/// 
/// <br/>
/// 
/// ### Notification :
/// 
/// - In current version, `#[Payload(JSON)]` requires the struct impls `serde::Deserialize`.
/// - In current version, `#[Payload(Form)]` is **NOT** implemented yet. Please wait for development, or, if you need this imediately, you can implement and create a [Pull request](https://github.com/kana-rus/ohkami/pulls) !
/// 
/// <br/>
/// 
/// ```ignore
/// use ohkami::{Context, Response};
/// use ohkami::Payload; // <-- import me
/// 
/// #[Payload(JSON)]
/// #[derive(serde::Deserialize)] // <-- This may be not required in future version
/// struct HelloRequest<'name> {
///     name:     &'name str,
///     n_repeat: Option<usize>,
/// }
/// 
/// async fn hello<'q>(c: Context, body: HelloRequest<'q>) -> Response<String> {
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
/// - Possible value types : `&'lifetime str` `String` `u8` `u16` `u32` `u64` `u128` `usize` `i8` `i16` `i32` `i64` `i128` `isize`
/// 
/// If you need support for other structs or types, plaese let me know that in [GitHub issue](https://github.com/kana-rus/ohkami/issues) !

#[proc_macro_attribute] #[allow(non_snake_case)]
pub fn Payload(format: proc_macro::TokenStream, data: proc_macro::TokenStream) -> proc_macro::TokenStream {
    payload::Payload(format.into(), data.into())
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}
