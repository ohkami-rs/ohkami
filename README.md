<div align="center">
    <h1>Ohkami</h1>
    Ohkami <em>- [狼] wolf in Japanese -</em> is intuitive and declarative web framework.
</div>

<br>

- *macro-less and type-safe* APIs for intuitive and declarative code
- *multiple runtimes* are supported：`tokio`, `async-std`, `worker` (Cloudflare Workers)

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
ohkami = { version = "0.20", features = ["rt_tokio"] }
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

### `"rt_tokio"`, `"rt_async-std"`

Select a native async runtime

<br>

### `"rt_worker"`：Cloudflare Workers

```sh
npm create cloudflare ./path/to/project -- --template https://github.com/ohkami-rs/ohkami-templates/worker
```

Then your project directory has `wrangler.toml`, `package.json` and a Rust library crate.

Local dev by `npm run dev` and deploy by `npm run deploy` !

See README of the [template](https://github.com/ohkami-rs/ohkami-templates/tree/main/worker) for details.

<br>

### `"sse"`：Server-Sent Events

Ohkami responds with HTTP/1.1 `Transfer-Encoding: chunked`.\
Use some reverse proxy to do with HTTP/2,3.

```rust,no_run
use ohkami::prelude::*;
use ohkami::typed::DataStream;
use tokio::time::sleep;

async fn sse() -> DataStream<String> {
    DataStream::from_iter_async((1..=5).map(|i| async move {
        sleep(std::time::Duration::from_secs(1)).await;
        Ok(format!("Hi, I'm message #{i} !"))
    }))
}

#[tokio::main]
async fn main() {
    Ohkami::new((
        "/sse".GET(sse),
    )).howl("localhost:5050").await
}
```

<br>

### `"ws"`：WebSocket

Currently, WebSocket on `rt_worker` is *not* supported.

```rust,no_run
use ohkami::prelude::*;
use ohkami::ws::{WebSocketContext, WebSocket, Message};

async fn echo_text(c: WebSocketContext<'_>) -> WebSocket {
    c.connect(|mut ws| async move {
        while let Ok(Some(Message::Text(text))) = ws.recv().await {
            ws.send(Message::Text(text)).await.expect("Failed to send text");
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

<br>

## Snippets

### Middlewares

Ohkami's request handling system is called "**fang**s", and middlewares are implemented on this.

*builtin fang* : `CORS`, `JWT`, `BasicAuth`, `Timeout`

```rust,no_run
use ohkami::prelude::*;

#[derive(Clone)]
struct GreetingFang;

/* utility trait; automatically impl `Fang` trait */
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

### Typed payload

*builtin payload* : `JSON`, `Text`, `HTML`, `URLEncoded`, `Multipart`

```rust
use ohkami::prelude::*;
use ohkami::typed::{status, Payload};
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
/* using `/S` shorthand */
#[Payload(JSON/S)]
struct User {
    name: String,
}

async fn create_user(
    req: CreateUserRequest<'_>
) -> status::Created<User> {
    status::Created(User {
        name: String::from(req.name)
    })
}
```

<br>

### Payload validation

`where ＜validation expression＞` in `#[Payload( 〜 )]` performs the validation when responding with it or parsing request body to it.

`＜validation expression＞` is an expression with `self: &Self` returning `Result<(), impl Display>`.

```rust
use ohkami::prelude::*;
use ohkami::{typed::Payload, builtin::payload::JSON};

#[Payload(JSON/D where self.valid())]
struct Hello<'req> {
    name:   &'req str,
    repeat: usize,
}

impl Hello<'_> {
    fn valid(&self) -> Result<(), String> {
        (self.name.len() > 0).then_some(())
            .ok_or_else(|| format!("`name` must not be empty"))?;
        (self.repeat > 0).then_some(())
            .ok_or_else(|| format!("`repeat` must be positive"))?;
        Ok(())
    }
}
```

<br>

### Typed params

```rust,no_run
use ohkami::prelude::*;
use ohkami::typed::Query;

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

#[Query] /* Params like `?lang=rust&q=framework` */
struct SearchQuery<'q> {
    lang:    &'q str,
    #[query(rename = "q")] /* #[serde]-compatible #[query] attribute */
    keyword: &'q str,
}

#[Payload(JSON/S)]
struct SearchResult {
    title: String,
}

async fn search(
    query: SearchQuery<'_>
) -> Vec<SearchResult> {
    vec![
        SearchResult { title: String::from("ohkami") },
    ]
}
```

<br>

### File upload

```rust,no_run
use ohkami::prelude::*;
use ohkami::typed::{status, Payload};
use ohkami::builtin::{payload::Multipart, item::File};


#[Payload(Multipart/D)]
struct FormData<'req> {
    #[serde(rename = "account-name")]
    account_name: Option<&'req str>,
    pics: Vec<File<'req>>,
}

async fn post_submit(form_data: FormData<'_>) -> status::NoContent {
    println!("\n\
        ===== submit =====\n\
        [account name] {:?}\n\
        [  pictures  ] {} files (mime: [{}])\n\
        ==================",
        form_data.account_name,
        form_data.pics.len(),
        form_data.pics.iter().map(|f| f.mimetype).collect::<Vec<_>>().join(", "),
    );

    status::NoContent
}
```

<br>

### Pack of Ohkamis

```rust,no_run
use ohkami::prelude::*;
use ohkami::typed::{status, Payload};
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

async fn create_user() -> status::Created<User> {
    status::Created(User {
        name: String::from("ohkami web framework")
    })
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

- [x] HTTP/1.1
- [ ] HTTP/2
- [ ] HTTP/3
- [ ] HTTPS
- [x] Server-Sent Events
- [x] WebSocket

## Benchmark Results

- [Web Frameworks Benchmark](https://web-frameworks-benchmark.netlify.app/result?l=rust)

## MSRV ( Minimum Supported Rust Version )

Latest stable

## License

ohkami is licensed under MIT LICENSE ( [LICENSE](https://github.com/ohkami-rs/ohkami/blob/main/LICENSE) or [https://opensource.org/licenses/MIT](https://opensource.org/licenses/MIT) ).
