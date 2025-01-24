<div align="center">
    <h1>Ohkami</h1>
    Ohkami <em>- [狼] wolf in Japanese -</em> is intuitive and declarative web framework.
</div>

<br>

- *macro-less and type-safe* APIs for intuitive and declarative code
- *multiple runtimes* are supported：`tokio`, `async-std`, `smol`, `nio`, `glommio`, `worker` (Cloudflare Workers)

<div align="right">
    <a href="https://github.com/ohkami-rs/ohkami/blob/main/LICENSE"><img alt="License" src="https://img.shields.io/crates/l/ohkami.svg" /></a>
    <a href="https://github.com/ohkami-rs/ohkami/actions"><img alt="build check status of ohkami" src="https://github.com/ohkami-rs/ohkami/actions/workflows/CI.yml/badge.svg"/></a>
    <a href="https://crates.io/crates/ohkami"><img alt="crates.io" src="https://img.shields.io/crates/v/ohkami" /></a>
</div>

<br>

## Benchmark Results

- [Web Frameworks Benchmark](https://web-frameworks-benchmark.netlify.app/result)

<!--

- [TechEmpower's Benchmark](https://www.techempower.com/benchmarks)

-->

<br>

## Quick Start

1. Add to `dependencies` :

```toml
[dependencies]
ohkami = { version = "0.21", features = ["rt_tokio"] }
tokio  = { version = "1",    features = ["full"] }
```

2. Write your first code with Ohkami : [examples/quick_start](https://github.com/ohkami-rs/ohkami/blob/main/examples/quick_start/src/main.rs)

```rust,no_run
use ohkami::prelude::*;
use ohkami::typed::status;

async fn health_check() -> status::NoContent {
    status::NoContent
}

async fn hello(name: &str) -> String {
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

## Feature flags

### `"rt_tokio"`, `"rt_async-std"`, `"rt_smol"`, `"rt_nio"`, `"rt_glommio"`：native async runtime

- [tokio](https://github.com/tokio-rs/tokio)
- [async-std](https://github.com/async-rs/async-std)
- [smol](https://github.com/smol-rs/smol)
- [nio](https://github.com/nurmohammed840/nio)
- [glommio](https://github.com/DataDog/glommio)

### `"rt_worker"`：Cloudflare Workers

```sh
npm create cloudflare ./path/to/project -- --template https://github.com/ohkami-rs/ohkami-templates/worker
```

then your project directory has `wrangler.toml`, `package.json` and a Rust library crate. Local dev by `npm run dev` and deploy by `npm run deploy` !

See README of the [template](https://github.com/ohkami-rs/ohkami-templates/tree/main/worker) for details.

### `"sse"`：Server-Sent Events

Ohkami responds with HTTP/1.1 `Transfer-Encoding: chunked`.\
Use some reverse proxy to do with HTTP/2,3.

```rust,no_run
use ohkami::prelude::*;
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

### `"ws"`：WebSocket

Ohkami only handles `ws://`.\
Use some reverse proxy to do with `wss://`.

WebSocket on Durable Object is available on `"rt_worker"`!

```rust,no_run
use ohkami::prelude::*;
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

### `"openapi"`：OpenAPI document generation

Ohkami supports *as consistent as possible* OpenAPI document generation, where most of the consistency between document and behavior is automatically assured by Ohkami's internal work.

Only you have to

- Derive `openapi::Schema` for all your schema structs
- Make your `Ohkami` call `.generate({openapi::OpenAPI})`

to generate consistent OpenAPI document. You don't need to take care of writing accurate methods, paths, parameters, contents, ... for this OpenAPI feature; All they are done by Ohkami.

Of course, you can flexibly customize schemas ( by hand-implemetation of `Schema` ), descriptions or other parts ( by `#[operation]` attribute and `openapi_*` hooks ).

```rust,ignore
use ohkami::prelude::*;
use ohkami::format::JSON;
use ohkami::typed::status;
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
    JSON(CreateUser { name }): JSON<CreateUser<'_>>
) -> status::Created<JSON<User>> {
    status::Created(JSON(User {
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
async fn list_users() -> JSON<Vec<User>> {
    JSON(vec![])
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
        servers: vec![
            openapi::Server::at("localhost:5000"),
        ]
    });

    o.howl("localhost:5000").await;
}
```

- Currently, only **JSON** is supported as the document format.
- When the binary size matters, you should prepare a feature flag activating `ohkami/openapi` in your package, and put all your codes around `openapi` behind that feature via `#[cfg(feature = ...)]` or `#[cfg_attr(feature = ...)]`.
- In `rt_worker`, `.generate` is not available because `Ohkami` can't have access to your local filesystem by `wasm32` binary on Minifalre. So ohkami provides [a CLI tool](./scripts/workers_openapi.js) to generate document from `#[ohkami::worker] Ohkami` with `openapi` feature.

### `"nightly"`：nightly-only functionalities

- try response

<br>

## Snippets

### Middlewares

Ohkami's request handling system is called "**fang**s", and middlewares are implemented on this.

*builtin fang* : `CORS`, `JWT`, `BasicAuth`, `Timeout`, `Context`

```rust,no_run
use ohkami::prelude::*;

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
        // register fangs to a Ohkami
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

### Typed payload

*builtin payload* : `JSON`, `Text`, `HTML`, `URLEncoded`, `Multipart`

```rust
use ohkami::prelude::*;
use ohkami::typed::status;

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
    JSON(req): JSON<CreateUserRequest<'_>>
) -> status::Created<JSON<User>> {
    status::Created(JSON(
        User {
            name: String::from(req.name)
        }
    ))
}
```

### Typed params

```rust,no_run
use ohkami::prelude::*;

#[tokio::main]
async fn main() {
    Ohkami::new((
        "/hello/:name"
            .GET(hello),
        "/hello/:name/:n"
            .GET(hello_n),
        "/search"
            .GET(search),
    )).howl("localhost:5000").await
}

async fn hello(name: &str) -> String {
    format!("Hello, {name}!")
}

async fn hello_n((name, n): (&str, usize)) -> String {
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
) -> JSON<Vec<SearchResult>> {
    JSON(vec![
        SearchResult { title: String::from("ohkami") },
    ])
}
```

### Static directory serving

```rust,no_run
use ohkami::prelude::*;

#[tokio::main]
async fn main() {
    Ohkami::new((
        "/".Dir("./dist"),
    )).howl("0.0.0.0:3030").await
}
```

### File upload

```rust,no_run
use ohkami::prelude::*;
use ohkami::typed::status;
use ohkami::format::{Multipart, File};

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
use ohkami::prelude::*;
use ohkami::typed::status;

#[derive(Serialize)]
struct User {
    name: String
}

async fn list_users() -> JSON<Vec<User>> {
    JSON(vec![
        User { name: String::from("actix") },
        User { name: String::from("axum") },
        User { name: String::from("ohkami") },
    ])
}

async fn create_user() -> status::Created<JSON<User>> {
    status::Created(JSON(User {
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
use ohkami::prelude::*;
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

<br>

## Supported protocols

- [x] HTTP/1.1
- [ ] HTTP/2
- [ ] HTTP/3
- [ ] HTTPS
- [x] Server-Sent Events
- [x] WebSocket


## MSRV ( Minimum Supported Rust Version )

Latest stable


## License

ohkami is licensed under MIT LICENSE ( [LICENSE](https://github.com/ohkami-rs/ohkami/blob/main/LICENSE) or [https://opensource.org/licenses/MIT](https://opensource.org/licenses/MIT) ).
