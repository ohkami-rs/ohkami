mod util;

mod serde;
mod from_request;

#[cfg(feature="openapi")]
mod openapi;

#[cfg(feature="worker")]
mod worker;

#[cfg(feature="openapi")]
/// # Deriving `openapi::Schema`
/// 
/// register the struct as a `schema` of OpenAPI document
/// 
/// <br>
/// 
/// ## Helper attributes
/// 
/// ### Container attributes
/// 
/// #### `#[openapi(component)]`
/// Define the schema in `components`
/// 
/// ### Field attributes
/// 
/// #### `#[openapi(schema_with = "schema_fn")]`
/// Use `schema_fn()` instead for the field. `schema_fn`:
/// 
/// - must be callable as `fn() -> impl Into<ohkami::openapi::SchemaRef>`
/// - can be a path like `schema_fns::a_schema`
/// 
/// ### Variant attributes
/// 
/// #### `#[openapi(schema_with = "schema_fn")]`
/// Use `schema_fn()` instead for the variant. `schema_fn`:
/// 
/// - must be callable as `fn() -> impl Into<ohkami::openapi::SchemaRef>`
/// - can be a path like `schema_fns::a_schema`
/// 
/// <br>
/// 
/// ## Example
/// 
/// ```ignore
/// use ohkami::prelude::*;
/// use ohkami::openapi;
/// 
/// #[derive(Deserialize, openapi::Schema)]
/// struct HelloRequest<'req> {
///     name: Option<&'req str>,
///     repeat: Option<usize>,
/// }
/// 
/// async fn hello(
///     JSON(req): JSON<HelloRequest<'_>>,
/// ) -> String {
///     let name = req.name.unwrap_or("world");
///     let repeat = req.name.repeat.unwrap_or(1);
///     vec![format!("Hello, {name}!"); repeat].join(" ")
/// }
/// 
/// #[tokio::main]
/// async fn main() {
///     Ohkami::new((
///         "/hello".GET(hello),
///     )).howl("localhost:3000").await
/// }
/// ```
#[proc_macro_derive(Schema, attributes(openapi))]
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
/// ---
/// 
/// `#[worker]` accepts an argument in following format for *document purpose*:
/// 
/// ```ts
/// {
///     title: string,
///     version: string | number,
///     servers: [
///         {
///             url: string,
///             description: string,
///             variables: {
///                 [string]: {
///                     default: string,
///                     enum: [string],
///                 }
///             }
///         }
///     ]
/// }
/// ```
/// 
/// Actually **every field is optional** and **any other fields are acceptable**,
/// but when `openapi` feature is activated, these fields are used for the
/// document generation ( if missing, some default values will be used ).
#[proc_macro_attribute]
pub fn worker(args: proc_macro::TokenStream, ohkami_fn: proc_macro::TokenStream) -> proc_macro::TokenStream {
    worker::worker(args.into(), ohkami_fn.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[cfg(feature="worker")]
/// Integrate the struct with Workers runtime as a Durable Object.\
/// This requires to impl `DurableObject` trait and the trait requires this attribute.
/// 
/// ### Example
/// 
/// ```
/// use worker::{State, Env};
/// use ohkami::DurableObject;
/// 
/// # struct User;
/// # struct Message;
/// 
/// #[DurableObject]
/// pub struct Chatroom {
///     users: Vec<User>,
///     messages: Vec<Message>,
///     state: State,
///     env: Env, // access `Env` across requests, use inside `fetch`
/// }
/// 
/// impl DurableObject for Chatroom {
///     fn new(state: State, env: Env) -> Self {
///         Self {
///             users: vec![],
///             messages: vec![],
///             state,
///             env,
///         }
///     }
/// 
///     async fn fetch(&mut self, _req: Request) -> worker::Result<worker::Response> {
///         // do some work when a worker makes a request to this DO
///         worker::Response::ok(&format!("{} active users.", self.users.len()))
///     }
/// }
/// ```
/// 
/// ### Note
/// 
/// You can specify the usage of the Durable Object via an argument in order to control WASM/JS outout:
/// 
/// * `fetch`: simple `fetch` target
/// * `alarm`: with [Alarms API](https://developers.cloudflare.com/durable-objects/examples/alarms-api/)
/// * `websocket`: [WebSocket server](https://developers.cloudflare.com/durable-objects/examples/websocket-hibernation-server/)
/// 
/// ```ignore
/// #[DurableObject(fetch)]
/// pub struct Chatroom {
///     users: Vec<User>,
///     messages: Vec<Message>,
///     state: State,
///     env: Env, // access `Env` across requests, use inside `fetch`
/// }
/// ```
#[proc_macro_attribute]
#[allow(non_snake_case)]
pub fn DurableObject(args: proc_macro::TokenStream, input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    worker::DurableObject(args.into(), input.into())
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
/// <br>
/// 
/// ## 2 ways of binding
/// 
/// following wrangler.toml for example :
/// 
/// ```ignore
/// [[kv_namespaces]]
/// binding = "MY_KV"
/// id      = "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
/// ```
/// 
/// ### Auto binding mode
/// 
/// For **unit struct**, `#[bindings]` automatically collects **all** bindings from
/// your *wrangler.toml* and generates fields for them.
/// 
/// ```ignore
/// #[ohkami::bindings]
/// struct Bindings;
/// 
/// async fn handler(b: Bindings) -> String {
///     let data = b.MY_KV.get("data").text().await
///         .expect("Failed to get data");
/// 
///     //...
/// }
/// ```
/// 
/// ### Manual binding mode
/// 
/// For **struct with named fields**, `#[bindings]` just collects bindings
/// that have the **same name as its fields** from  your *wrangler.toml*,
/// 
/// In this way, types in `ohkami::bindings` module are useful to avoid
/// inconsistency and unclear namings of `worker` crate's binding types.
/// 
/// ```ignore
/// use ohkami::bindings;
/// 
/// #[bindings]
/// struct Bindings {
///     MY_KV: bindings::KV,
/// }
/// 
/// async fn handler(b: Bindings) -> String {
///     let data = b.MY_KV.get("data").text().await
///         .expect("Failed to get data");
/// 
///     //...
/// }
/// ```
/// 
/// <br>
/// 
/// ## Note
/// 
/// - `#[bindings]` currently supports
///   - AI
///   - KV
///   - R2
///   - D1
///   - Queue (producer)
///   - Service
///   - Variables
///   - Durable Objects
/// - `Queue` may cause a lot of *WARNING*s on `npm run dev`, but
///   it's not an actual problem and `Queue` binding does work.
/// 
/// <br>
/// 
/// ## Tips
/// 
/// - You can switch between multiple `env`s by feature flags
///   like `#[cfg_attr(feature = "...", bindings(env_name))]`.
/// - For `rust-analyzer` user : When you edit wrangler.toml around bindings in **auto binding mode**,
///   you'll need to notify the change of `#[bindings]` if you're using auto binding mode.
///   For that, all you have to do is just **deleting `;` and immediate restoring it**.
#[proc_macro_attribute]
pub fn bindings(env_name: proc_macro::TokenStream, bindings_struct: proc_macro::TokenStream) -> proc_macro::TokenStream {
    worker::bindings(env_name.into(), bindings_struct.into())
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

/// Deriving `FromRequest` impl for a struct composed of
/// `FromRequest` types
/// 
/// <br>
/// 
/// *example.rs*
/// ```ignore
/// use ohkami::fang::Context;
/// use sqlx::PgPool;
/// 
/// #[derive(ohkami::FromRequest)]
/// struct MyItems1<'req> {
///     db: Context<'req, PgPool>,
/// }
/// 
/// #[derive(FromRequest)]
/// struct MyItems2<'req>(
///     MyItems1<'req>,
/// );
/// ```
#[proc_macro_derive(FromRequest)]
pub fn derive_from_request(target: proc_macro::TokenStream) -> proc_macro::TokenStream {
    from_request::derive_from_request(target.into())
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

#[doc(hidden)]
#[proc_macro_attribute]
pub fn consume_struct(_: proc_macro::TokenStream, _: proc_macro::TokenStream) -> proc_macro::TokenStream {
    proc_macro::TokenStream::new()
}
