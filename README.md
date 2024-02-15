<div align="center">
    <h1>ohkami</h1>
    ohkami <em>- [狼] wolf in Japanese -</em> is intuitive and declarative web framework.
</div>

<br>

- *macro-less and type-safe* APIs for intuitive and declarative code
- *multi runtime* support：`tokio`, `async-std`

<div align="right">
    <a href="https://github.com/kana-rus/ohkami/actions"><img alt="build check status of ohkami" src="https://github.com/kana-rus/ohkami/actions/workflows/CI.yml/badge.svg"/></a>
    <a href="https://crates.io/crates/ohkami"><img alt="crates.io" src="https://img.shields.io/crates/v/ohkami" /></a>
    <a href="https://github.com/kana-rus/ohkami/blob/main/LICENSE"><img alt="License" src="https://img.shields.io/crates/l/ohkami.svg" /></a>
</div>

<br>

## Quick start
1. Add to `dependencies` :

```toml
# This sample uses `tokio` runtime.
# `async-std` is available by feature "rt_async-std".

[dependencies]
ohkami = { version = "0.14", features = ["rt_tokio"] }
tokio  = { version = "1",    features = ["full"] }
```

2. Write your first code with ohkami : [examples/quick_start](https://github.com/kana-rus/ohkami/blob/main/examples/quick_start/src/main.rs)

```rust,no_run
use ohkami::prelude::*;
use ohkami::typed::status::{NoContent};

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

## Snippets

### handle path params
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

### handle query params / request body
```rust
use ohkami::prelude::*;
use ohkami::typed::status::{OK, Created};
use ohkami::typed::{Query, Payload, ResponseBody};

#[Query]
struct SearchQuery<'q> {
    q: &'q str,
}

async fn search(condition: SearchQuery<'_>) -> OK<String> {
    OK(format!("Something found"))
}

#[Payload(JSOND)]
struct CreateUserRequest<'req> {
    name:     &'req str,
    password: &'req str,
}

#[ResponseBody(JSONS)]
struct User {
    name: String,
}

async fn create_user(body: CreateUserRequest<'_>) -> Created<User> {
    Created(User {
        name: format!("ohkami web framework")
    })
}
```
`#[Query]`, `#[Payload( 〜 )]` implements `FromRequest` trait for the struct.

( with path params : `({path params}, {FromRequest value}s...)` )

<br>

### use middlewares
ohkami's middlewares are called "**fang**s".

```rust,no_run
use ohkami::prelude::*;

struct LogRequest;
impl FrontFang for LogRequest {
    async fn bite(&self, req: &mut Request) -> Result<(), Response> {
        println!("{req:?}");
        Ok(())
    }
}

struct AppendServer;
impl BackFang for AppendServer {
    async fn bite(&self, res: &mut Response, _req: &Request) -> Result<(), Response> {
        res.headers.set()
            .Server("ohkami");
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    Ohkami::with((AppendServer, LogRequest), (
        "/".GET(|| async {"Hello!"})
    )).howl("localhost:8080").await
}
```

<br>

### pack of Ohkamis
```rust,no_run
use ohkami::prelude::*;
use ohkami::typed::ResponseBody;
use ohkami::typed::status::{Created, NoContent};

#[ResponseBody(JSONS)]
struct User {
    name: String
}

async fn create_user() -> Created<User> {
    Created(User {
        name: "ohkami web framework".to_string()
    })
}

async fn health_check() -> NoContent {
    NoContent
}

#[tokio::main]
async fn main() {
    // ...

    let users_ohkami = Ohkami::new((
        "/".POST(create_user),
    ));

    Ohkami::new((
        "/healthz"  .GET(health_check),
        "/api/users".By(users_ohkami), // <-- nest by `By`
    )).howl("localhost:5000").await
}
```

<br>

### testing
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
    let ho = hello_ohkami();

    let req = TestRequest::GET("/");
    let res = ho.oneshot(req).await;
    assert_eq!(res.status(), Status::NotFound);

    let req = TestRequest::GET("/hello");
    let res = ho.oneshot(req).await;
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

## MSRV (Minimum Supported rustc Version)
Latest stable at the time of publication.

## License
ohkami is licensed under MIT LICENSE ([LICENSE](https://github.com/kana-rus/ohkami/blob/main/LICENSE) or [https://opensource.org/licenses/MIT](https://opensource.org/licenses/MIT)).
