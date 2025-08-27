<div align="center">
    <h1>Ohkami</h1>
    Ohkami <em>- [狼] wolf in Japanese -</em> is a performant, declarative, and runtime-flexible web framework for Rust.
</div>

<br>

- *macro-less and type-safe* APIs for declarative, ergonomic code
- *runtime-flexible* ： `tokio`, `smol`, `nio`, `glommio` and `worker` (Cloudflare Workers), `lambda` (AWS Lambda)
- good performance, no-network testing, well-structured middlewares, Server-Sent Events, WebSocket, highly integrated OpenAPI document generation, ...

<div align="right">
    <a href="https://github.com/ohkami-rs/ohkami/blob/main/LICENSE"><img alt="License" src="https://img.shields.io/crates/l/ohkami.svg" /></a>
    <a href="https://github.com/ohkami-rs/ohkami/actions"><img alt="build check status of ohkami" src="https://github.com/ohkami-rs/ohkami/actions/workflows/CI.yml/badge.svg"/></a>
    <a href="https://crates.io/crates/ohkami"><img alt="crates.io" src="https://img.shields.io/crates/v/ohkami" /></a>
</div>

<br>

## Quick Start

1. Add to `dependencies` :

```toml
[dependencies]
ohkami = { version = "0.24", features = ["rt_tokio"] }
tokio  = { version = "1",    features = ["full"] }
```

2. Write your first code with Ohkami : [examples/quick_start](https://github.com/ohkami-rs/ohkami/blob/main/examples/quick_start/src/main.rs)

```rust,no_run
use ohkami::{Ohkami, Route};
use ohkami::claw::{Path, status};

async fn health_check() -> status::NoContent {
    status::NoContent
}

async fn hello(Path(name): Path<&str>) -> String {
    format!("Hello, {name}!")
}

#[tokio::main]
async fn main() {
    Ohkami::new((
        "/healthz"
            .GET(health_check),
        "/hello/:name"
            .GET(hello),
    )).howl("localhost:3000").await
}
```

3. Run and check the behavior :

```sh
$ cargo run
```
```sh
$ curl http://localhost:3000/healthz
$ curl http://localhost:3000/hello/your_name
Hello, your_name!
```

<br>
    
## Core APIs

### `Route`

`Route` is the core trait to define Ohkami's routing:

- `.GET()`, `.POST()`, `.PUT()`, `.PATCH()`, `.DELETE()`, `.OPTIONS()` to define API endpoints
- `.By({another Ohkami})` to nest `Ohkami`s
- `.Mount({directory path})` to serve static directory
  (pre-compressed files with `gzip`, `deflate`, `br`, `zstd` are supported)

Here `GET`, `POST`, etc. takes a *handler* function:

```rust,ignore
async fn({FromRequest type},*) -> {IntoResponse type}
```

On native runtimes, whole a handler must be `Send + Sync + 'static`
and the return future must be `Send + 'static`.

### `claw`s

Ohkami provides `claw` API: handler parts for declarative way to
extract request data and construct response data.

- `content` - typed content {extracted from request / for response} of specific format
  - built-in: `Json<T>`, `Text<T>`, `Html<T>`, `UrlEncoded<T>`, `Multipart<T>`
- `param` - typed parameters extracted from request
  - built-in: `Path<P>`, `Query<T>`
- `header` - types for specific header extracted from request
  - built-in: types for standard request headers
- `status` - types for response with specific status code
  - built-in: types for standard response status codes

<sm><i>(
here <code>T</code> means a type that implements
<code>serde::Deserialize</code> for request and
<code>serde::Serialize</code> for response,
and <code>P</code> means a type that implements
<code>FromParam</code> or
a tuple of such types.
)</i></sm>

The number of path parameters extracted by `Path` is **automatically asserted**
to be the same or less than the number of path parameters contained in the route path
when the handler is registered to routing.

```rust,ignore
async fn handler0(
    Path(param): Path<FromParamType>,
) -> Json<SerializeType> {
    // ...
}

async fn handler1(
    Json(req): Json<Deserialize0>,
    Path((param0, param1)): Path<(FromParam0, FromParam1)>,
    Query(query): Query<Deserialize1>,
) -> status::Created<Json<Serialize0>> {
    // ...
}
```

### `fang`s

Ohkami's request handling system is called `fang`; all handlers and middlewares are built on it.

```rust,ignore
/* simplified for description */

pub trait Fang<Inner: FangProc> {
    type Proc: FangProc;
    fn chain(&self, inner: Inner) -> Self::Proc;
}

pub trait FangProc {
    async fn bite<'b>(&'b self, req: &'b mut Request) -> Response;
}
```

built-in:

- `BasicAuth`, `Cors`, `Csrf`, `Jwt` (authentication/security)
- `Context` (reuqest context)
- `Enamel` (security headers; experimantal)
- `Timeout` (handling timeout; native runtime only)
- `openapi::Tag` (tag for OpenAPI document generation; `openapi` feature only)

Ohkami provides `FangAction` utility trait to implement `Fang` trait easily:

```rust,ignore
/* simplified for description */

pub trait FangAction {
    async fn fore<'a>(&'a self, req: &'a mut Request) -> Result<(), Response> {
        // default implementation is empty
        Ok(())
    }
    async fn back<'a>(&'a self, res: &'a mut Response) {
        // default implementation is empty
    }
}
```

Additionally, you can apply fangs both as **global fangs** to an `Ohkami` or
as **local fangs** to a specific handler (described below).

### `Ohkami`

`Ohkami` is the main entry point of Ohkami application:
a collection of `Route`s and `Fang`s, and provides `.howl()`/`.howls()` method to run the application.

```rust,ignore
Ohkami::new((
    // global fangs
    Fang1,
    Fang2,
    // routes
    "/hello"
        .GET(hello_handler)
        .POST(hello_post_handler),
    "/goodbye"
        .GET((
            // local fangs
            Fang3,
            Fang4,
            goodbye_handler // handler
        )),
)).howl("localhost:3000").await;
```

`.howls()` (`tls` feature only) is used to run Ohkami with TLS (HTTPS) support
upon [`rustls`](https://github.com/rustls) ecosystem.

`howl(s)` supports graceful shutdown by `Ctrl-C` ( `SIGINT` ) on native runtimes.

<br>

## Feature flags

### `"rt_tokio"`, `"rt_smol"`, `"rt_nio"`, `"rt_glommio"` : native async runtime

- [tokio](https://github.com/tokio-rs/tokio) _v1.\*.\*_
- [smol](https://github.com/smol-rs/smol) _v2.\*.\*_
- [nio](https://github.com/nurmohammed840/nio) _v0.0.\*_
- [glommio](https://github.com/DataDog/glommio) _v0.9.\*_

### `"rt_worker"` : Cloudflare Workers

- [worker](https://github.com/cloudflare/workers-rs) _v0.6.\*_

Ohkami has first-class support for Cloudflare Workers:

- `#[worker]` macro to define a Worker
- `#[bindings]`, `ws::SessionMap` helper
- better `DurableObject`
- not require `Send` `Sync` bound for handlers or fangs
- [worker_openapi.js](https://github.com/ohkami-rs/ohkami/tree/main/scripts/worker_openapi.js) script to generate OpenAPI document from `#[worker]` fn

And also maintains useful project template. Run :

```sh
npm create cloudflare ＜project dir＞ -- --template https://github.com/ohkami-rs/templates/worker
```

then `＜project dir＞` will have `wrangler.jsonc`, `package.json` and a Rust library crate.

`#[ohkami::worker] async? fn({bindings}?) -> Ohkami` is the Worker definition.

Local dev by `npm run dev` and deploy by `npm run deploy` !

See

- `worker.*` temaplates in [template repository](https://github.com/ohkami-rs/templates)
- `worker.*` samples in [samples directory](https://github.com/ohkami-rs/ohkami/tree/main/samples)
- `#[worker]`'s documentation comment in [macro definitions](https://github.com/ohkami-rs/ohkami/tree/main/ohkami_macros/src/lib.rs)

for wokring examples and detailed usage of `#[worker]` (and/or `openapi`).

### `"rt_lambda"` : AWS Lambda

- [lambda_runtime](https://github.com/awslabs/aws-lambda-rust-runtime) _v0.14.\*_ with `tokio`

Both `Function URLs` and `API Gateway` are supported, and WebSocket is not supported.

[cargo lambda](https://crates.io/crates/cargo-lambda) will be good partner. Let's run :

```sh
cargo lambda new ＜project dir＞ --template https://github.com/ohkami-rs/templates
```

`lambda_runtime::run(your_ohkami)` make `you_ohkami` run on Lambda Function.

Local dev by

```sh
cargo lambda watch
```

and deploy by

```sh
cargo lambda build --release [--compiler cargo] [and more]
cargo lambda deploy [--role ＜arn-of-a-iam-role＞] [and more]
```

See

* README of [template](https://github.com/ohkami-rs/templates/tree/main/template)
* [Cargo Lambda document](https://www.cargo-lambda.info)

for details.

### `"sse"` : Server-Sent Events

Ohkami responds with HTTP/1.1 `Transfer-Encoding: chunked`.\
Use some reverse proxy to do with HTTP/2,3.

```rust,no_run
use ohkami::{Ohkami, Route};
use ohkami::sse::DataStream;
use tokio::time::{sleep, Duration};

async fn handler() -> DataStream {
    DataStream::new(|mut s| async move {
        s.send("starting streaming...");
        for i in 1..=5 {
            sleep(Duration::from_secs(1)).await;
            s.send(format!("MESSAGE #{i}"));
        }
        s.send("streaming finished!");
    })
}

#[tokio::main]
async fn main() {
    Ohkami::new((
        "/sse".GET(handler),
    )).howl("localhost:3020").await
}
```

### `"ws"` : WebSocket

```rust,no_run
use ohkami::{Ohkami, Route};
use ohkami::ws::{WebSocketContext, WebSocket, Message};

async fn echo_text(ctx: WebSocketContext<'_>) -> WebSocket {
    ctx.upgrade(|mut conn| async move {
        while let Ok(Some(Message::Text(text))) = conn.recv().await {
            conn.send(text).await.expect("failed to send text");
        }
    })
}

#[tokio::main]
async fn main() {
    Ohkami::new((
        "/ws".GET(echo_text),
    )).howl("localhost:3030").await
}
```

* On `"rt_worker"`, both normal ( stateless ) WebSocket and WebSocket on Durable Object are available!
* On `"rt_lambda"`, WebSocket is currently not supported.

### `"openapi"` : OpenAPI document generation

`"openapi"` provides highly integrated OpenAPI support.

This enables **macro-less**, *as consistent as possible* OpenAPI document generation, where most of the consistency between document and behavior is automatically assured by Ohkami's internal work.

Only you have to

- Derive `openapi::Schema` for all your schema structs
- Make your `Ohkami` call `.generate(openapi::OpenAPI { ... })`

to generate consistent OpenAPI document.

You don't need to take care of writing accurate methods, paths, parameters, contents, ... for this OpenAPI feature; All they are done by Ohkami.

Of course, you can flexibly

- customize schemas by manual implemetation of `Schema` trait
- customize descriptions or other parts by `#[operation]` attribute and `openapi_*` hooks of `FromRequest`, `IntoResponse`, `Fang (Action)`
- put `tag`s for grouping operations by `openapi::Tag` fang

```rust,ignore
use ohkami::{Ohkami, Route};
use ohkami::claw::{Json, status};
use ohkami::openapi;

// Derive `Schema` trait to generate
// the schema of this struct in OpenAPI document.
#[derive(Deserialize, openapi::Schema)]
struct CreateUser<'req> {
    name: &'req str,
}

#[derive(Serialize, openapi::Schema)]
// `#[openapi(component)]` to define it as component
// in OpenAPI document.
#[openapi(component)]
struct User {
    id: usize,
    name: String,
}

async fn create_user(
    Json(CreateUser { name }): Json<CreateUser<'_>>
) -> status::Created<Json<User>> {
    status::Created(Json(User {
        id: 42,
        name: name.to_string()
    }))
}

// (optionally) Set operationId, summary,
// or override descriptions by `operation` attribute.
#[openapi::operation({
    summary: "...",
    200: "List of all users",
})]
/// This doc comment is used for the
/// `description` field of OpenAPI document
async fn list_users() -> Json<Vec<User>> {
    Json(vec![])
}

#[tokio::main]
async fn main() {
    let o = Ohkami::new((
        "/users"
            .GET(list_users)
            .POST(create_user),
    ));

    // This make your Ohkami spit out `openapi.json`
    // ( the file name is configurable by `.generate_to` ).
    o.generate(openapi::OpenAPI {
        title: "Users Server",
        version: "0.1.0",
        servers: &[
            openapi::Server::at("localhost:5000"),
        ]
    });

    o.howl("localhost:5000").await;
}
```

- Currently, only JSON is supported as the document format.
- When the binary size matters, you should prepare a feature flag activating `ohkami/openapi` in your package, and put all your codes around `openapi` behind that feature via `#[cfg(feature = ...)]` or `#[cfg_attr(feature = ...)]`.
- In `rt_worker`, `.generate` is not available because `Ohkami` can't have access to your local filesystem by `wasm32` binary on Minifalre. So ohkami provides [a CLI tool](./scripts/workers_openapi.js) to generate document from `#[ohkami::worker] Ohkami` with `openapi` feature.

### `"tls"`

HTTPS support up on [rustls](https://github.com/rustls) ecosystem.

- Call `howls` ( as `https` to `http`, `wss` to `ws` ) instead of `howl` to run with TLS.
- You must prepare your own certificate and private key files.
- Currently, only HTTP/1.1 over TLS is supported.

Example :

```sh
$ openssl req -x509 -newkey rsa:4096 -nodes -keyout server.key -out server.crt -days 365 -subj "/CN=localhost"
```

```toml
[dependencies]
ohkami = { version = "0.24", features = ["rt_tokio", "tls"] }
tokio  = { version = "1",    features = ["full"] }
rustls = { version = "0.23", features = ["ring"] }
rustls-pemfile = "2.2"
```

```rust,no_run
use ohkami::{Ohkami, Route};
use rustls::ServerConfig;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use std::fs::File;
use std::io::BufReader;

async fn hello() -> &'static str {
    "Hello, secure ohkami!"
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Initialize rustls crypto provider
    rustls::crypto::ring::default_provider().install_default()
        .expect("Failed to install rustls crypto provider");

    // Load certificates and private key
    let cert_file = File::open("server.crt")?;
    let key_file = File::open("server.key")?;
    
    let cert_chain = rustls_pemfile::certs(&mut BufReader::new(cert_file))
        .map(|cd| cd.map(CertificateDer::from))
        .collect::<Result<Vec<_>, _>>()?;
    
    let key = rustls_pemfile::read_one(&mut BufReader::new(key_file))?
        .map(|p| match p {
            rustls_pemfile::Item::Pkcs1Key(k) => PrivateKeyDer::Pkcs1(k),
            rustls_pemfile::Item::Pkcs8Key(k) => PrivateKeyDer::Pkcs8(k),
            _ => panic!("Unexpected private key type"),
        })
        .expect("Failed to read private key");

    // Build TLS configuration
    let tls_config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(cert_chain, key)
        .expect("Failed to build TLS configuration");

    // Create and run Ohkami with HTTPS
    Ohkami::new((
        "/".GET(hello),
    )).howls("0.0.0.0:8443", tls_config).await;
    
    Ok(())
}
```

```sh
$ cargo run
```

```sh
$ curl https://localhost:8443 --insecure  # for self-signed certificate
Hello, secure ohkami!
```

For localhost-testing with browser (or `curl` without `--insecure`),
[`mkcert`](https://github.com/FiloSottile/mkcert) is highly recommended.

### `"nightly"` : nightly-only functionalities

- try response
- internal performance optimizations

<br>

## Snippets

### Typed content

```rust
use ohkami::claw::{Json, status};
use ohkami::serde::{Deserialize, Serialize};

/* Deserialize for request */
#[derive(Deserialize)]
struct CreateUserRequest<'req> {
    name:     &'req str,
    password: &'req str,
}

/* Serialize for response */
#[derive(Serialize)]
struct User {
    name: String,
}

async fn create_user(
    Json(req): Json<CreateUserRequest<'_>>
) -> status::Created<Json<User>> {
    status::Created(Json(
        User {
            name: String::from(req.name)
        }
    ))
}
```

### Typed params

```rust,no_run
use ohkami::{Ohkami, Route};
use ohkami::claw::{Path, Query, Json};
use ohkami::serde::{Deserialize, Serialize};

#[tokio::main]
async fn main() {
    Ohkami::new((
        "/hello/:name/:n"
            .GET(hello_n),
        "/hello/:name"
            .GET(hello),
        "/search"
            .GET(search),
    )).howl("localhost:5000").await
}

async fn hello(Path(name): Path<&str>) -> String {
    format!("Hello, {name}!")
}

async fn hello_n(
    Path((name, n)): Path<(&str, usize)>
) -> String {
    vec![format!("Hello, {name}!"); n].join(" ")
}

#[derive(Deserialize)]
struct SearchQuery<'q> {
    #[serde(rename = "q")]
    keyword: &'q str,
    lang:    &'q str,
}

#[derive(Serialize)]
struct SearchResult {
    title: String,
}

async fn search(
    Query(query): Query<SearchQuery<'_>>
) -> Json<Vec<SearchResult>> {
    Json(vec![
        SearchResult { title: String::from("ohkami") },
    ])
}
```

### Middlewares

```rust,no_run
use ohkami::{Ohkami, Route, FangAction, Request, Response};

#[derive(Clone)]
struct GreetingFang(usize);

/* utility trait; automatically impl `Fang` trait */
impl FangAction for GreetingFang {
    async fn fore<'a>(&'a self, req: &'a mut Request) -> Result<(), Response> {
        let Self(id) = self;
        println!("[{id}] Welcome request!: {req:?}");
        Ok(())
    }
    async fn back<'a>(&'a self, res: &'a mut Response) {
        let Self(id) = self;
        println!("[{id}] Go, response!: {res:?}");
    }
}

#[tokio::main]
async fn main() {
    Ohkami::new((
        // register *global fangs* to an Ohkami
        GreetingFang(1),
        "/hello"
            .GET(|| async {"Hello, fangs!"})
            .POST((
                // register *local fangs* to a handler
                GreetingFang(2),
                || async {"I'm `POST /hello`!"}
            ))
    )).howl("localhost:3000").await
}
```

### Database connection management with `Context`

```rust,no_run
use ohkami::{Ohkami, Route};
use ohkami::claw::status;
use ohkami::fang::Context;
use sqlx::postgres::{PgPoolOptions, PgPool};

#[tokio::main]
async fn main() {
    let pool = PgPoolOptions::new()
        .connect("postgres://ohkami:password@localhost:5432/db").await
        .expect("failed to connect");

    Ohkami::new((
        Context::new(pool),
        "/users".POST(create_user),
    )).howl("localhost:5050").await
}

async fn create_user(
    Context(pool): Context<'_, PgPool>,
) -> status::Created {
    //...

    status::Created(())
}
```

### Typed errors

```rust,no_run
use ohkami::{Response, IntoResponse};
use ohkami::claw::{Path, Json};
use ohkami::serde::Serialize;
use ohkami::fang::Context;

enum MyError {
    Sqlx(sqlx::Error),
}
impl IntoResponse for MyError {
    fn into_response(self) -> Response {
        match self {
            Self::Sqlx(e) => Response::InternalServerError(),
        }
    }
}

#[derive(Serialize)]
struct User {
    id: u32,
    name: String,
}

async fn get_user(
    Path(id): Path<u32>,
    Context(pool): Context<'_, sqlx::PgPool>,
) -> Result<Json<User>, MyError> {
    let sql = r#"
        SELECT name FROM users WHERE id = $1
    "#;
    let name = sqlx::query_scalar::<_, String>(sql)
        .bind(id as i64)
        .fetch_one(pool)
        .await
        .map_err(MyError::Sqlx)?;

    Ok(Json(User { id, name }))
}
```

[thiserror](https://crates.io/crates/thiserror) may improve such error conversion:

```rust,ignore
    let name = sqlx::query_salor_as::<_, String>(sql)
        .bind(id)
        .fetch_one(pool)
        // .await
        // .map_err(MyError::Sqlx)?;
        .await?;
```

### Static directory serving

```rust,no_run
use ohkami::{Ohkami, Route};

#[tokio::main]
async fn main() {
    Ohkami::new((
        "/".Mount("./dist"),
    )).howl("0.0.0.0:3030").await
}
```

### File upload

`Multipart` built-in claw and `File` helper:

```rust,no_run
use ohkami::claw::{status, content::{Multipart, File}};
use ohkami::serde::Deserialize;

#[derive(Deserialize)]
struct FormData<'req> {
    #[serde(rename = "account-name")]
    account_name: Option<&'req str>,
    pics: Vec<File<'req>>,
}

async fn post_submit(
    Multipart(data): Multipart<FormData<'_>>
) -> status::NoContent {
    println!("\n\
        ===== submit =====\n\
        [account name] {:?}\n\
        [  pictures  ] {} files (mime: [{}])\n\
        ==================",
        data.account_name,
        data.pics.len(),
        data.pics.iter().map(|f| f.mimetype).collect::<Vec<_>>().join(", "),
    );

    status::NoContent
}
```

### Pack of Ohkamis

```rust,no_run
use ohkami::{Ohkami, Route};
use ohkami::claw::{Json, status};
use serde::Serialize;

#[derive(Serialize)]
struct User {
    name: String
}

async fn list_users() -> Json<Vec<User>> {
    Json(vec![
        User { name: String::from("actix") },
        User { name: String::from("axum") },
        User { name: String::from("ohkami") },
    ])
}

async fn create_user() -> status::Created<Json<User>> {
    status::Created(Json(User {
        name: String::from("ohkami web framework")
    }))
}

async fn health_check() -> status::NoContent {
    status::NoContent
}

#[tokio::main]
async fn main() {
    // ...

    let users_ohkami = Ohkami::new((
        "/"
            .GET(list_users)
            .POST(create_user),
    ));

    Ohkami::new((
        "/healthz"
            .GET(health_check),
        "/api/users"
            .By(users_ohkami), // nest by `By`
    )).howl("localhost:5000").await
}
```

### Testing

```rust
use ohkami::{Ohkami, Route};
use ohkami::testing::*; // <--

fn hello_ohkami() -> Ohkami {
    Ohkami::new((
        "/hello".GET(|| async {"Hello, world!"}),
    ))
}

#[cfg(test)]
#[tokio::test]
async fn test_my_ohkami() {
    let t = hello_ohkami().test();

    let req = TestRequest::GET("/");
    let res = t.oneshot(req).await;
    assert_eq!(res.status(), Status::NotFound);

    let req = TestRequest::GET("/hello");
    let res = t.oneshot(req).await;
    assert_eq!(res.status(), Status::OK);
    assert_eq!(res.text(), Some("Hello, world!"));
}
```

### DI by generics

```rust,no_run
use ohkami::{Ohkami, Route, Response, IntoResponse};
use ohkami::claw::{Json, Path};
use ohkami::fang::Context;
use ohkami::serde::Serialize;

//////////////////////////////////////////////////////////////////////
/// errors

enum MyError {
    Sqlx(sqlx::Error),
}
impl IntoResponse for MyError {
    fn into_response(self) -> Response {
        match self {
            Self::Sqlx(e) => Response::InternalServerError(),
        }
    }
}

//////////////////////////////////////////////////////////////////////
/// repository

trait UserRepository: Send + Sync + 'static {
    fn get_user_by_id(
        &self,
        id: i64,
    ) -> impl Future<Output = Result<UserRow, MyError>> + Send;
}

#[derive(sqlx::FromRow)]
struct UserRow {
    id: i64,
    name: String,
}

#[derive(Clone)]
struct PostgresUserRepository(sqlx::PgPool);
impl UserRepository for PostgresUserRepository {
    async fn get_user_by_id(&self, id: i64) -> Result<UserRow, MyError> {
        let sql = r#"
            SELECT id, name FROM users WHERE id = $1
        "#;
        sqlx::query_as::<_, UserRow>(sql)
            .bind(id)
            .fetch_one(&self.0)
            .await
            .map_err(MyError::Sqlx)
    }
}

//////////////////////////////////////////////////////////////////////
/// routes

#[derive(Serialize)]
struct User {
    id: u32,
    name: String,
}

async fn get_user<R: UserRepository>(
    Path(id): Path<u32>,
    Context(r): Context<'_, R>,
) -> Result<Json<User>, MyError> {
    let user_row = r.get_user_by_id(id as i64).await?;

    Ok(Json(User {
        id: user_row.id as u32,
        name: user_row.name,
    }))
}

fn users_ohkami<R: UserRepository>() -> Ohkami {
    Ohkami::new((
        "/:id".GET(get_user::<R>),
    ))
}

//////////////////////////////////////////////////////////////////////
/// entry point

#[tokio::main]
async fn main() {
    let pool = sqlx::PgPool::connect("postgres://ohkami:password@localhost:5432/db")
        .await
        .expect("failed to connect to database");
    
    Ohkami::new((
        Context::new(PostgresUserRepository(pool)),
        "/users".By(users_ohkami::<PostgresUserRepository>()),
    )).howl("0.0.0.0:4040").await
}
```

<br>

## Supported protocols

- [x] HTTP/1.1
- [ ] HTTP/2
- [ ] HTTP/3
- [x] HTTPS
- [x] Server-Sent Events
- [x] WebSocket

## MSRV ( Minimum Supported Rust Version )

Latest stable

## License

Ohkami is licensed under MIT LICENSE ( [LICENSE](https://github.com/ohkami-rs/ohkami/blob/main/LICENSE) or [https://opensource.org/licenses/MIT](https://opensource.org/licenses/MIT) ).
