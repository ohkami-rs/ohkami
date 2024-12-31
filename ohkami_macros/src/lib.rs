mod serde;
mod from_request;

#[cfg(feature="openapi")]
mod openapi;

#[cfg(feature="worker")]
mod worker;


#[cfg(feature="openapi")]
#[proc_macro_derive(Schema, attributes())]
pub fn derive_schema(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    openapi::derive_schema(input.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[cfg(feature="openapi")]
/// ```ignore
/// /* custom operationId (default: name of the fn) */
/// #[operation(get_hello)]
/// /// description for `get_hello` operation
/// async fn hello() -> Result<String, MyError> {
///     //...
/// }
/// 
/// /* custom operationId and summary */
/// #[operation(get_hello2 { summary: "HELLO greeting" })]
/// /// description for `get_hello2` operation
/// async fn hello2() -> Result<String, MyError> {
///     //...
/// }
/// 
/// /* custom summary */
/// #[operation({ summary: "HELLO greeting" })]
/// /// description for `hello3` operation
/// async fn hello3() -> Result<String, MyError> {
///     //...
/// }
/// 
/// /* custom operationId and some descriptions */
/// #[operation(get_hello4 {
///     requestBody: "User name (text/plain).",
///     200: "Successfully returning a HELLO greeting for the user",
/// })]
/// /// description for `get_hello4` operation
/// async fn hello4(
///     Text(name): Text,
/// ) -> Result<String, MyError> {
///     //...
/// }
/// ```
#[proc_macro_attribute]
pub fn operation(args: proc_macro::TokenStream, handler: proc_macro::TokenStream) -> proc_macro::TokenStream {
    openapi::operation(args.into(), handler.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[cfg(feature="worker")]
/// Create an worker Ohkami, running on Cloudflare Workers !
/// 
/// - This only handle `fetch` event.
/// - Expected signature: `() -> Ohkami` ( both sync/async are available )
/// 
/// ---
/// *lib.rs*
/// ```ignore
/// use ohkami::prelude::*;
/// 
/// #[ohkami::worker]
/// fn my_ohkami() -> Ohkami {
///     Ohkami::new((
///         "/".GET(|| async {"Hello, world!"})
///     ))
/// }
/// ```
#[proc_macro_attribute]
pub fn worker(_: proc_macro::TokenStream, ohkami_fn: proc_macro::TokenStream) -> proc_macro::TokenStream {
    worker::worker(ohkami_fn.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[cfg(feature="worker")]
/// Automatically bind bindings in wrangler.toml to Rust struct.
/// 
/// - This uses the default (top-level) env by default. You can configure it
///   by argument: `#[bindings(dev)]`
/// - Binded struct implements `FromRequest` and it can be used as an
///   handler argument
/// 
/// _**note**_ : `#[bindings]` supports
/// 
/// - KV
/// - D1
/// - Queue (producer)
/// - Service
/// - Variables
/// - Durable Objects
/// 
/// in cuurent version, as `worker` crate does.
/// ( `worker` supports secrets, but secrets aren't written in wrangler.toml... )
/// 
/// <br>
/// 
/// ---
/// *wrangler.toml*
/// ```ignore
/// [[kv_namespaces]]
/// binding = "MY_KV"
/// id      = "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
/// ```
/// ---
/// *lib.rs*
/// ```ignore
/// #[bindings]
/// struct Bindings;
/// 
/// async fn handler(b: Bindings) -> String {
///     let data = b.MY_KV.get("data").text().await
///         .expect("Failed to get data");
/// 
///     //...
/// }
/// ```
/// ---
/// 
/// <br>
/// 
/// _**tips**_ :
/// 
/// - You can switch between multiple `env`s by feature flags like `#[cfg_attr(feature = "...", bindings(env_name))]`
/// - For `rust-analyzer` user : When you edit wrangler.toml around bindings,
///   you'll need to reload `#[bindings] struct ...;` to notice the new bindings to rust-analyer.
///   For that, what you have to do is just **deleting `;` and immediate restoring it**.
#[proc_macro_attribute]
pub fn bindings(env: proc_macro::TokenStream, bindings_struct: proc_macro::TokenStream) -> proc_macro::TokenStream {
    worker::bindings(env.into(), bindings_struct.into())
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
