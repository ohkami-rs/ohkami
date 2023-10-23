mod components;

mod query;
mod payload;

/// ## Query parameters
/// 
/// <br/>
/// 
/// ```ignore
/// use ohkami::prelude::*;
/// use ohkami::utils::Queries; // <-- import me
/// 
/// #[Query]
/// struct HelloQuery {
///     name:     String,
///     n_repeat: Option<usize>,
/// }
/// 
/// async fn hello(c: Context, queries: HelloQuery) -> Response {
///     let HelloQuery { name, n_repeat } = queries;
/// 
///     let message = match n_repeat {
///         None    => format!("Hello"),
///         Some(n) => format!("Hello, {name}! ").repeat(n),
///     };
/// 
///     c.OK().text(message)
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
/// - NOT available for tuple struct ( like `struct S(usize, usize);` ) or tag struct ( like `struct X;` ).
/// 
/// ### Valid format :
/// 
/// - `#[Payload(JSON)]` ( for `application/json` )
/// - `#[Payload(FormData)]` ( for `multipart/form-data` )
/// - `#[Payload(URLEncoded)]` ( for `application/x-www-form-urlencoded` )
/// 
/// <br/>
/// 
/// ### JSON
/// 
/// - Requires that the struct implements `serde::Deserialize`
/// 
/// ```ignore
/// use ohkami::prelude::*;
/// use ohkami::utils::Payload; // <-- import me
/// 
/// #[Payload(JSON)]
/// #[derive(serde::Deserialize)] // <-- This may be not required in future version
/// struct HelloRequest {
///     name:     String,
///     n_repeat: Option<usize>,
/// }
/// /* expected payload examples :
///     {"name":"your name"}
///     {"name":"you_name","n_repeat":2}
/// */
/// 
/// async fn hello(c: Context, body: HelloRequest) -> Response {
///     let HelloRequest { name, n_repeat } = queries;
/// 
///     let message = match n_repeat {
///         None    => format!("Hello"),
///         Some(n) => format!("Hello, {name}! ").repeat(n),
///     };
/// 
///     c.OK().text(message)
/// }
/// ```
/// 
/// <br/>
/// 
/// ### URLEncoded
/// 
/// ```ignore
/// use ohkami::prelude::*;
/// use ohkami::utils::Payload; // <-- import me
/// 
/// #[Payload(URLEncoded)]
/// struct HelloRequest {
///     name:     String,
///     n_repeat: Option<usize>,
/// }
/// /* expected payload examples :
///     name=yourname
///     name=yourname&n_repeat=2
/// */
/// ```
/// 
/// - Available value types : `String` `u8` `u16` `u32` `u64` `u128` `usize` and `Option` of them.
/// 
/// <br/>
/// 
/// ### FormData
/// 
/// ```ignore
/// use ohkami::prelude::*;
/// use ohkami::utils::Payload; // <-- import me
/// 
/// #[Payload(FormData)]
/// struct ProfileData {
///     submitter_name: String,
///     pics:           Vec<File>,
/// }
/// /* expected form :
///     <form action="http://server.dom/cgi/handle" enctype="multiprt/form-data" method="post">
///         What is your name? <input type="text" name="submitter-name" />
///         What files are you sending? <input="file" name="pics" />
///     </form>
/// */ 
///
/// ```
/// 
/// - Available value types : `String` or `File` or `Vec<File>`.
/// - Form part of kebab-case-name is handled by field of snake_case version of the name ( example: `name="submitter-name"` is handled by field `submitter_name` ).
/// 
#[proc_macro_attribute] #[allow(non_snake_case)]
pub fn Payload(format: proc_macro::TokenStream, data: proc_macro::TokenStream) -> proc_macro::TokenStream {
    payload::Payload(format.into(), data.into())
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}
