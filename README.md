<div align="center">
    <h1>ohkami</h1>
    ohkami <em>- [狼] wolf in Japanese -</em> is <strong>declarative</strong> web framework for Rust.
</div>

## Features
- *macro free, declarative APIs*
- *multi runtime* support：`tokio`, `async-std` (and more in future)

<div align="right">
    <img alt="build check status of ohkami" src="https://github.com/kana-rus/ohkami/actions/workflows/check.yml/badge.svg"/>
    <img alt="test status of ohkami" src="https://github.com/kana-rus/ohkami/actions/workflows/test.yml/badge.svg"/>
</div>

<br/>

## Quick start
1. Add to `dependencies` :

```toml
# This sample uses `tokio` runtime.
# You can choose `async-std` instead by feature "rt_async-std".

[dependencies]
ohkami = { version = "0.10", features = ["rt_tokio"] }
tokio  = { version = "1",    features = ["full"] }
```

2. Write your first code with ohkami : [eamples/quick_start](https://github.com/kana-rus/ohkami/blob/main/examples/quick_start/src/main.rs)

```rust
use ohkami::prelude::*;
use ohkami::typed::{OK, NoContent};

async fn health_check() -> NoContent {
    NoContent
}

async fn hello(name: &str) -> OK<String> {
    OK(format!("Hello, {name}!"))
}

#[tokio::main]
async fn main() {
    Ohkami::new((
        "/hc".
            GET(health_check),
        "/hello/:name".
            GET(hello),
    )).howl(3000).await
}
```

3. Run and check the behavior :

```sh
$ cargo run
```
```sh
$ curl http://localhost:3000/hc
$ curl http://localhost:3000/hello/your_name
Hello, your_name!
```

<br/>

## Snippets

### handle path params
```rust
use ohkami::prelude::*;

#[tokio::main]
async fn main() {
    Ohkami::new((
        "/api/users/:id".
            GET(get_user),
    )).howl("localhost:5000").await
}

async fn get_user(id: usize) -> impl IntoResponse { /* */ }
```

<br/>

### handle query params / request body
```rust
use ohkami::prelude::*;
use ohkami::utils;   // <--

#[utils::Query]
struct SearchQuery<'q> {
    q: &'q str,
}
async fn search(condition: SearchQuery) -> Response { /* */ }

#[utils::Payload(JSON)]
#[derive(utils::Deserialize)]
struct CreateUserRequest<'req> {
    name:     &'req str,
    password: &'req str,
}
async fn create_user(body: CreateUserRequest<'_>) -> Response { /* */ }
```
`#[Query]`, `#[Payload( 〜 )]` implements `FromRequest` trait for the struct.

( with path params : `({path params}, {FromRequest values...})` )

<br/>

### use middlewares
ohkami's middlewares are called "**fang**s".

```rust
use ohkami::prelude::*;

struct AppendHeaders;
impl IntoFang for AppendHeaders {
    fn bite(self) -> Fang {
        Fang(|res: &mut Response| {
            res.headers.set()
                .Server("ohkami");
        })
    }
}

struct Log;
impl IntoFang for Log {
    fn bite(self) -> Fang {
        Fang(|res: &Response| {
            println!("{res:?}");
        })
    }
}

#[tokio::main]
async fn main() {
    Ohkami::with((AppendHeaders, Log), (
        "/"  .GET(root),
        "/hc".GET(health_check),
        "/api/users".
            GET(get_users).
            POST(create_user),
    )).howl(":8080").await
}

```
`Fang` schema :

#### To make a *back fang* :
- `Fn({&/&mut Response})`
- `Fn(Response) -> Response`

#### To make a *front fang* :
- `Fn()`
- `Fn({&/&mut Request})`
- or `_ -> Result<(), Response>` version of them (for early returning an error response)

<br/>

### pack of Ohkamis
```rust
#[tokio::main]
async fn main() {
    // ...

    let users_ohkami = Ohkami::new((
        "/".
            POST(create_user),
        "/:id".
            GET(get_user).
            PATCH(update_user).
            DELETE(delete_user),
    ));

    Ohkami::new((
        "/hc"       .GET(health_check),
        "/api/users".By(users_ohkami), // <-- nest by `By`
    )).howl(5000).await
}
```

<br/>

### web socket (VERY experimental feature)
Activate `"websocket"` feature.

```rust
use ohkami::prelude::*;
use ohkami::websocket::{WebSocketContext, Message};

fn handle_websocket(c: WebSocketContext) -> Response {
    c.on_upgrade(|ws| async move {
        while let Some(Ok(message)) = ws.recv().await {
            match message {
                Message::Text(text) => {
                    let response = Message::from(text);
                    if let Err(e) = ws.send(response).await {
                        tracing::error!("{e}");
                        break
                    }
                }
                Message::Close(_) => break,
                other => tracing::warning!("Unsupported message type: {other}"),
            }
        }
    }).await
}

#[tokio::main]
async fn main() {
    Ohkami::new((
        "/websocket"
            .GET(handle_websocket)
    )).howl(8080).await
}
```

<br/>

### testing
```rust
use ohkami::prelude::*;
use ohkami::typed::OK;
use ohkami::testing::*; // <--

fn hello_ohkami() -> Ohkami {
    Ohkami::new((
        "/hello".GET(|| async move {
            OK("Hello, world!")
        })
    ))
}

#[tokio::main]
async fn main() {
    hello_ohkami()
        .howl(5050).await
}

#[cfg(test)]
#[tokio::test]
async fn test_my_ohkami() {
    let hello_ohkami = hello_ohkami();

    let res = hello_ohkami.oneshot(TestRequest::GET("/")).await;
    assert_eq!(res.status, http::Status::NotFound);

    let res = hello_ohkami.oneshot(TestRequest::GET("/hello")).await;
    assert_eq!(res.status, http::Status::OK);
    assert_eq!(res.content.unwrap().text(), Some("Hello, world!"));
}
```

<br/>

## License
`ohkami` is licensed under MIT LICENSE ([LICENSE-MIT](https://github.com/kana-rus/ohkami/blob/main/LICENSE-MIT) or [https://opensource.org/licenses/MIT](https://opensource.org/licenses/MIT)).
