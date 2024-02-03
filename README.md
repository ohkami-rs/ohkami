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

```rust,no_run
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
        "/healthz".
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
```rust,no_run
use ohkami::prelude::*;

#[tokio::main]
async fn main() {
    Ohkami::new((
        "/api/hello/:name".
            GET(hello),
    )).howl("localhost:5000").await
}

async fn hello(name: &str) -> impl IntoResponse {
    format!("Hello, {name}!")
}
```

<br/>

### handle query params / request body
```rust
use ohkami::prelude::*;
use ohkami::typed;   // <--

#[typed::Query]
struct SearchQuery<'q> {
    q: &'q str,
}

async fn search(condition: SearchQuery<'_>) -> OK<String> {
    OK(format!("Something found"))
}

#[typed::Payload(JSOND)]
struct CreateUserRequest<'req> {
    #[serde(rename = "username")]
    name:     &'req str,
    password: &'req str,
}

#[typed::ResponseBody(JSONS)]
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

( with path params : `({path params}, {FromRequest values...})` )

<br/>

### use middlewares
ohkami's middlewares are called "**fang**s".

```rust,ignore
use ohkami::prelude::*;

struct AppendHeaders;
impl IntoFang for AppendHeaders {
    fn into_fang(self) -> Fang {
        Fang(|res: &mut Response| {
            res.headers.set()
                .Server("ohkami");
        })
    }
}

struct Log;
impl IntoFang for Log {
    fn into_fang(self) -> Fang {
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
```rust,ignore
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

### testing
```rust,no_run
use ohkami::prelude::*;
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
