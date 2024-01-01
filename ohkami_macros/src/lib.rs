mod components;

mod query;
mod payload;

/// ## Query parameters
/// 
/// - Value types：types that impls `FromParam`, or `Option<_>` of them
/// - NOT available for tuple struct ( like `struct S(usize, usize);` ) or unit struct ( like `struct X;` ).
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
#[proc_macro_attribute] #[allow(non_snake_case)]
pub fn Query(_: proc_macro::TokenStream, data: proc_macro::TokenStream) -> proc_macro::TokenStream {
    query::Query(data.into())
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

/// ## Request payload
/// 
/// - NOT available for tuple struct ( like `struct S(usize, usize);` ) or unit struct ( like `struct X;` ).
/// 
/// ### Valid format :
/// 
/// - `#[Payload(JSON)]` ( for `application/json` )
/// - `#[Payload(Form)]` ( for `multipart/form-data` )
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
/// #[derive(serde::Deserialize)]
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
/// - Available value types : types that impl `FromParam`, or `Option<_>` of them.
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
/// <br/>
/// 
/// ### Form
/// 
/// **NOTE**：This can't handle reference types like `&str` in current version. Wait for the development!
/// 
/// - Available value types : `String`, `File`, `Vec<File>`.
/// - Form part of kebab-case-name is handled by field of snake_case version of the name ( example: `name="submitter-name"` is handled by field `submitter_name` ).
/// 
/// 
/// ```ignore
/// use ohkami::prelude::*;
/// use ohkami::utils::{Payload, File}; // <-- import me
/// 
/// #[Payload(Form)]
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
/// ```
#[proc_macro_attribute] #[allow(non_snake_case)]
pub fn Payload(format: proc_macro::TokenStream, data: proc_macro::TokenStream) -> proc_macro::TokenStream {
    payload::Payload(format.into(), data.into())
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}
