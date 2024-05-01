mod serde;
mod query;
mod payload;
mod from_request;

#[cfg(feature="worker")]
mod worker;


#[cfg(feature="worker")]
#[proc_macro_attribute]
pub fn worker(_: proc_macro::TokenStream, ohkami_fn: proc_macro::TokenStream) -> proc_macro::TokenStream {
    worker::worker(ohkami_fn.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[cfg(feature="worker")]
#[proc_macro_attribute]
pub fn bindings(_: proc_macro::TokenStream, bindings_struct: proc_macro::TokenStream) -> proc_macro::TokenStream {
    worker::bindings(bindings_struct.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}


/// The *perfect* reexport of [serde](https://crates.io/crates/serde)'s `Serialize`.
/// 
/// <br>
/// 
/// *example.rs*
/// ```ignore
/// use ohkami::serde::Serialize;
/// 
/// #[derive(Serialize)]
/// struct User {
///     #[serde(rename = "username")]
///     name: String,
///     bio:  Option<String>,
/// }
/// ```
#[proc_macro_derive(Serialize, attributes(serde))] #[allow(non_snake_case)]
pub fn Serialize(data: proc_macro::TokenStream) -> proc_macro::TokenStream {
    serde::Serialize(data.into())
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}
/// The *perfect* reexport of [serde](https://crates.io/crates/serde)'s `Deserialize`.
/// 
/// <br>
/// 
/// *example.rs*
/// ```ignore
/// use ohkami::serde::Deserialize;
/// 
/// #[derive(Deserialize)]
/// struct CreateUser<'req> {
///     #[serde(rename = "username")]
///     name: &'req str,
///     bio:  Option<&'req str>,
/// }
/// ```
#[proc_macro_derive(Deserialize, attributes(serde))] #[allow(non_snake_case)]
pub fn Deserialize(data: proc_macro::TokenStream) -> proc_macro::TokenStream {
    serde::Deserialize(data.into())
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

#[doc(hidden)]
#[proc_macro_attribute]
pub fn consume_struct(_: proc_macro::TokenStream, _: proc_macro::TokenStream) -> proc_macro::TokenStream {
    proc_macro::TokenStream::new()
}


/// ## Query parameters
/// 
/// _NOTE_: NOT available for tuple struct ( like `struct S(usize, usize);` ) or unit struct ( like `struct X;` ).
/// 
/// `#[Query]` supports `#[serde]`-conpatible `#[query]` attributes for struct fields.
/// ( They are used in internal parsing process based on [ohkami_lib](https://crates.io/crates/ohkami_lib)'s `serde_urlencoded`. )
/// 
/// <br/>
/// 
/// *example.rs*
/// ```ignore
/// use ohkami::prelude::*;
/// use ohkami::typed::Query; // <--
/// 
/// #[Query]
/// struct HelloQuery<'q> {
///     name:     &'q str,
///     #[query(rename = "n-repeat")]
///     n_repeat: Option<usize>,
/// }
/// 
/// async fn hello(queries: HelloQuery<'_>) -> String {
///     let HelloQuery { name, n_repeat } = queries;
/// 
///     match n_repeat {
///         None    => format!("Hello"),
///         Some(n) => format!("Hello, {name}! ").repeat(n),
///     }
/// }
/// ```
#[proc_macro_attribute] #[allow(non_snake_case)]
pub fn Query(_: proc_macro::TokenStream, data: proc_macro::TokenStream) -> proc_macro::TokenStream {
    query::Query(data.into())
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}


/// ## Request / Response payload
/// 
/// Derives `Payload` implementaion with specified `PayloadType`.
/// 
/// - `Payload + Serialize` types can be used as response or response body in `typed::status`.
/// - `Payload + Deserialize` types can be used as request body passed via a handler argument.
/// 
/// <br>
/// 
/// In current version, ohkami provides following 5 builtin `PayloadType`s :
/// 
/// - `JSON` (for `application/json`)
/// - `URLEncoded` (for `application/www-x-urlencoded`)
/// - `Text` (for `text/plain`)
/// - `HTML` (for `text/html`)
/// - `Multipart` (for `multipart/form-data`)
/// 
/// Of course, other `PayloadType`s can be implemented by you or anyone else !
/// 
/// <br/>
/// 
/// ---
/// *example_with_builtin_json.rs*
/// ```ignore
/// use ohkami::prelude::*;
/// use ohkami::typed::Payload; // <--
/// use ohkami::builtin::payload::JSON; // <--
/// 
/// #[Payload(JSON)]
/// #[derive(ohkami::serde::Desrialize)]
/// struct HelloRequest<'req> {
///     name:     &'req str,
///     n_repeat: Option<usize>,
/// }
/// /* expected payload examples :
///     {"name":"your name"}
///     {"name":"you_name","n_repeat":2}
/// */
/// 
/// async fn hello(body: HelloRequest<'_>) -> String {
///     let HelloRequest { name, n_repeat } = queries;
/// 
///     match n_repeat {
///         None    => format!("Hello"),
///         Some(n) => format!("Hello, {name}! ").repeat(n),
///     }
/// }
/// ```
/// ---
/// 
/// <br>
/// 
/// Additionally, `#[Payload]` supports shortcuts for automatic deriving `Serialize` and `Deserialize` :
/// 
/// - `/ S` ... automatically derive `Serilize`
/// - `/ D` ... automatically derive `Deserilize`
/// - `/ SD` or `/ DS` ... automatically derive `Serialize` and `Deserialize`
/// 
/// <br>
/// 
/// ---
/// *shorthand.rs*
/// ```ignore
/// #[Payload(JSON/D)]
/// struct HelloRequest<'req> {
///     name:     &'req str,
///     n_repeat: Option<usize>,
/// }
/// ```
/// ---
#[proc_macro_attribute] #[allow(non_snake_case)]
pub fn Payload(format: proc_macro::TokenStream, data: proc_macro::TokenStream) -> proc_macro::TokenStream {
    payload::Payload(format.into(), data.into())
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}


/// # `#[derive(FromRequest)]`
/// 
/// Automatically impl `FromRequest` for a struct composed of
/// `FromRequest` types
/// 
/// <br>
/// 
/// *example.rs*
/// ```ignore
/// use ohkami::FromRequest;
/// use sqlx::PgPool;
/// 
/// #[derive(FromRequest)]
/// struct MyItems1<'req> {
///     db: ohkami::Memory<'req, PgPool>,
/// }
/// 
/// #[derive(FromRequest)]
/// struct MyItems2(
///     MyItems<'req>,
/// );
/// ```
#[proc_macro_derive(FromRequest)]
pub fn derive_from_request(target: proc_macro::TokenStream) -> proc_macro::TokenStream {
    from_request::derive_from_request(target.into())
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}
