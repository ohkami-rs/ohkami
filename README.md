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

async fn health_check(c: Context) -> Response {
    c.NoContent()
}

async fn hello(c: Context, name: String) -> Response {
    c.OK().text(format!("Hello, {name}!"))
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

async fn get_user(c: Context,
    id: usize /* <-- path param */
) -> Response { /* */ }
```
Use tuple like `(verion, id): (u8, usize),` for multiple path params.

<br/>

### handle query params / request body
```rust
use ohkami::prelude::*;
use ohkami::utils;   // <--

#[utils::Query]
struct SearchCondition {
    q: String,
}
async fn search(c: Context,
    condition: SearchCondition
) -> Response { /* */ }

#[utils::Payload(JSON)]
#[derive(serde::Deserialize)]
struct CreateUserRequest {
    name:     String,
    password: String,
}
async fn create_user(c: Context,
    body: CreateUserRequest
) -> Response { /* */ }
```
`#[Query]`, `#[Payload( 〜 )]` implements `FromRequest` trait for the struct.

( with path params : `(Context, {path params}, {FromRequest values...})` )

<br/>

### use middlewares
ohkami's middlewares are called "**fang**s".

```rust
use ohkami::prelude::*;
use ohkami::{Fang, IntoFang};

struct AppendHeaders;
impl IntoFang for AppendHeaders {
    fn bite(self) -> Fang {
        Fang(|c: &mut Context, req: &mut Request| {
            c.headers
                .Server("ohkami");
        })
    }
}

struct Log;
impl IntoFang for Log {
    fn bite(self) -> Fang {
        Fang(|res: Response| {
            println!("{res:?}");
            res
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

- to make *back fang* : `Fn(Response) -> Response`
- to make *front fang* : `Fn(&mut Context) | Fn(&mut Request) | Fn(&mut Context, &mut Request)`, or `_ -> Result<(), Response>` for early returning error responses

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

### web socket
Activate `websocket` feature.

```rust
use ohkami::prelude::*;
use ohkami::websocket::{WebSocket, Message};

async fn handle_websocket(ws: WebSocket) {
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
use ohkami::testing::*; // <--

fn hello_ohkami() -> Ohkami {
    Ohkami::new((
        "/hello".GET(|c: Context| async move {
            c.OK().text("Hello, world!")
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
    use ohkami::http::Status;

    let hello_ohkami = hello_ohkami();

    let res = hello_ohkami.oneshot(TestRequest::GET("/")).await;
    assert_eq!(res.status, Status::NotFound);

    let res = hello_ohkami.oneshot(TestRequest::GET("/hello")).await;
    assert_eq!(res.status, Status::OK);
    assert_eq!(res.content.unwrap().text().unwrap(), "Hello, world!");
}
```

<br/>

## License
`ohkami` is licensed under MIT LICENSE ([LICENSE-MIT](https://github.com/kana-rus/ohkami/blob/main/LICENSE-MIT) or [https://opensource.org/licenses/MIT](https://opensource.org/licenses/MIT)).
