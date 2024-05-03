<div align="center">
    <h1>Ohkami</h1>
    Ohkami <em>- [狼] wolf in Japanese -</em> is intuitive and declarative web framework.
</div>

<br>

- *macro-less and type-safe* APIs for intuitive and declarative code
- *multi runtime* support：`tokio`, `async-std`, `worker` (Cloudflare Workers)

<div align="right">
    <a href="https://github.com/kana-rus/ohkami/blob/main/LICENSE"><img alt="License" src="https://img.shields.io/crates/l/ohkami.svg" /></a>
    <a href="https://github.com/kana-rus/ohkami/actions"><img alt="build check status of ohkami" src="https://github.com/kana-rus/ohkami/actions/workflows/CI.yml/badge.svg"/></a>
    <a href="https://crates.io/crates/ohkami"><img alt="crates.io" src="https://img.shields.io/crates/v/ohkami" /></a>
</div>

<br>

## Quick Start
1. Add to `dependencies` :

```toml
# This sample uses `tokio` runtime.
# `async-std` is available by feature "rt_async-std".

[dependencies]
ohkami = { version = "0.18", features = ["rt_tokio"] }
tokio  = { version = "1",    features = ["full"] }
```

2. Write your first code with Ohkami : [examples/quick_start](https://github.com/kana-rus/ohkami/blob/main/examples/quick_start/src/main.rs)

```rust,no_run
use ohkami::prelude::*;
use ohkami::typed::status::NoContent;

async fn health_check() -> NoContent {
    NoContent
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

## Cloudflare Workers is supported by `rt_worker` feature
You can easily write Ohkami app and deploy it to Cloudflare Workers :

```sh
npm create cloudflare ./path/to/project -- --template https://github.com/kana-rus/ohkami-templates/worker
```

Then your `project` has `wrangler.toml`, `package.json` and

`Cargo.toml`
```toml
# ...

[dependencies]
ohkami = { version = "0.18", features = ["rt_worker"] }
worker = { version = "0.1" }
```

`src/lib.rs`
```rust,ignore
use ohkami::prelude::*;

#[ohkami::bindings]
struct Bindings;

#[ohkami::worker]
async fn my_worker() -> Ohkami {
    #[cfg(feature = "DEBUG")]
    console_error_panic_hook::set_once();

    Ohkami::new((
        "/".GET(|| async {"Hello, world!"}),
    ))
}
```

You can deploy by :

```sh
npm run deploy
```

<br>

## Snippets

### Handle path params
```rust,no_run
use ohkami::prelude::*;

#[tokio::main]
async fn main() {
    Ohkami::new((
        "/api/hello/:name"
            .GET(hello),
    )).howl("localhost:5000").await
}

async fn hello(name: &str) -> String {
    format!("Hello, {name}!")
}
```

<br>

### Handle request body / query params
```rust
use ohkami::prelude::*;
use ohkami::typed::status::Created;

use ohkami::typed::{Query, Payload};
use ohkami::builtin::payload::JSON;

/* `serde = 〜` is not needed in your [dependencies] */
use ohkami::serde::{Serialize, Deserialize};

/* Payload + Deserialize for request */
#[Payload(JSON)]
#[derive(Deserialize)]
struct CreateUserRequest<'req> {
    name:     &'req str,
    password: &'req str,
}

/* Payload + Serialize for response */
#[Payload(JSON)]
#[derive(Serialize)]
struct User {
    name: String,
}

async fn create_user(body: CreateUserRequest<'_>) -> Created<User> {
    Created(User {
        name: String::from("ohkami")
    })
}

/* Shorthand for Payload + Serialize */
#[Payload(JSON / S)]
struct SearchResult {
    title: String,
}

#[Query] /* Params like `?lang=rust&q=framework` */
struct SearchQuery<'q> {
    lang:    &'q str,
    #[query(rename = "q")] /* #[serde]-compatible #[query] attribute */
    keyword: &'q str,
}

async fn search(condition: SearchQuery<'_>) -> Vec<SearchResult> {
    vec![
        SearchResult { title: String::from("ohkami") },
    ]
}
```

<br>

### Use middlewares
Ohkami's request handling system is called "**fang**s", and middlewares are implemented on this :

```rust,no_run
use ohkami::prelude::*;

#[derive(Clone)]
struct GreetingFang;
impl FangAction for GreetingFang {
    async fn fore<'a>(&'a self, req: &'a mut Request) -> Result<(), Response> {
        println!("Welcomm request!: {req:?}");
        Ok(())
    }
    async fn back<'a>(&'a self, res: &'a mut Response) {
        println!("Go, response!: {res:?}");
    }
}

#[tokio::main]
async fn main() {
    Ohkami::with(GreetingFang, (
        "/".GET(|| async {"Hello, fangs!"})
    )).howl("localhost:3000").await
}
```

<br>

### Pack of Ohkamis
```rust,no_run
use ohkami::prelude::*;
use ohkami::typed::status::{Created, NoContent};
use ohkami::typed::Payload;
use ohkami::builtin::payload::JSON;

#[Payload(JSON/S)]
struct User {
    name: String
}

async fn list_users() -> Vec<User> {
    vec![
        User { name: String::from("actix") },
        User { name: String::from("axum") },
        User { name: String::from("ohkami") },
    ]
}

async fn create_user() -> Created<User> {
    Created(User {
        name: String::from("ohkami web framework")
    })
}

async fn health_check() -> NoContent {
    NoContent
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

<br>

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
- [ ] HTTPS
- [x] HTTP/1.1
- [ ] HTTP/2
- [ ] HTTP/3
- [ ] WebSocket

## MSRV (Minimum Supported Rust Version)
Latest stable

## License
ohkami is licensed under MIT LICENSE ([LICENSE](https://github.com/kana-rus/ohkami/blob/main/LICENSE) or [https://opensource.org/licenses/MIT](https://opensource.org/licenses/MIT)).
