mod components;

mod serde;
mod query;
mod payload;
mod response;


#[proc_macro_derive(Serialize, attributes(serde))] #[allow(non_snake_case)]
pub fn Serialize(data: proc_macro::TokenStream) -> proc_macro::TokenStream {
    serde::Serialize(data.into())
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}
#[proc_macro_derive(Deserialize, attributes(serde))] #[allow(non_snake_case)]
pub fn Deserialize(data: proc_macro::TokenStream) -> proc_macro::TokenStream {
    serde::Deserialize(data.into())
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

#[proc_macro_attribute]
pub fn consume_struct(_: proc_macro::TokenStream, _: proc_macro::TokenStream) -> proc_macro::TokenStream {
    proc_macro::TokenStream::new()
}


/// ## Query parameters
/// 
/// - Value types：types that impls `FromParam`, or `Option<_>` of them
/// - NOT available for tuple struct ( like `struct S(usize, usize);` ) or unit struct ( like `struct X;` ).
/// 
/// <br/>
/// 
/// *example.rs*
/// ```ignore
/// use ohkami::prelude::*;
/// use ohkami::utils::Queries; // <-- import me
/// 
/// #[Query]
/// struct HelloQuery<'q> {
///     name:     &'q str,
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


/// ## Request payload
/// 
/// - NOT available for tuple struct ( like `struct S(usize, usize);` ) or unit struct ( like `struct X;` ).
/// 
/// ### Valid format :
/// 
/// - `#[Payload(JSON)]` ( for `application/json` )
/// - `#[Payload(JSOND)]` ( `JSON + #[derive(Deserialize)]` )
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
/// use ohkami::utils::{Payload, Deseriailize}; // <-- import me and `Deserialize`
/// 
/// #[Payload(JSON)]
/// #[derive(Deserialize)]
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
/// struct HelloRequest<'req> {
///     name:     &'req str,
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


/// # Response body
/// 
/// Implements `ResponseBody` and `IntoResponse (as 200 OK)`
/// 
/// <br>
/// 
/// ## Valid format
/// - `#[ResponseBody(JSON)]`（for `application/json`）
/// - `#[ResponseBody(JSONS)]`（shorthand for `JSON + #[derive(Serialize)]`）
/// 
/// <br>
/// 
/// *example.rs*
/// ```ignore
/// use ohkami::prelude::*;
/// use ohkami::utils::{Payload, ResponseBody};
/// use ohkami::typed::{Created};
/// use sqlx::postgres::PgPool;
/// 
/// #[Payload(JSOND)]
/// struct CreateUserRequest<'c> {
///     name:     &'c str,
///     password: &'c str,
///     bio:      Option<&'c str>,
/// }
/// 
/// #[ResponseBody(JSONS)]
/// struct User {
///     name: String,
///     bio:  Option<String>,
/// }
/// 
/// async fn create_user(
///     req:  CreateUserRequest<'_>,
///     pool: Memory<'_, PgPool>,
/// ) -> Result<Created<User>, MyError> {
///     let hashed_password = crate::hash_password(req.password);
/// 
///     sqlx::query!(r#"
///         INSERT INTO users (name, password, bio)
///         VALUES ($1, $2, $3)
///     "#, req.name, hashed_password, req.bio)
///         .execute(*pool).await
///         .map_err(MyError::DB)?;
/// 
///     Ok(Created(User {
///         name: req.name.into(),
///         bio:  req.bio.map(String::from),
///     }))
/// }
/// ```
#[proc_macro_attribute] #[allow(non_snake_case)]
pub fn ResponseBody(format: proc_macro::TokenStream, data: proc_macro::TokenStream) -> proc_macro::TokenStream {
    response::ResponseBody(format.into(), data.into())
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}
