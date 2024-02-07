<div align="center">
    <h1>ohkami</h1>
    ohkami <em>- [狼] wolf in Japanese -</em> is <strong>declarative</strong> web framework for Rust.
</div>

<br>

- *macro less, declarative APIs*
- *multi runtime* support：`tokio`, `async-std`

<div align="right">
    <img alt="build check status of ohkami" src="https://github.com/kana-rus/ohkami/actions/workflows/check.yml/badge.svg"/>
    <img alt="test status of ohkami" src="https://github.com/kana-rus/ohkami/actions/workflows/test.yml/badge.svg"/>
</div>

<br>

## Quick start
1. Add to `dependencies` :

```toml
# This sample uses `tokio` runtime.
# You can choose `async-std` instead by feature "rt_async-std".

[dependencies]
ohkami = { version = "0.13", features = ["rt_tokio"] }
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
        "/healthz"
            .GET(health_check),
        "/hello/:name"
            .GET(hello),
    )).howl(3000).await
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

<br>

### use middlewares
ohkami's middlewares are called "**fang**s".

```rust,no_run
use ohkami::prelude::*;

struct AppendHeaders;
impl IntoFang for AppendHeaders {
    fn into_fang(self) -> Fang {
        Fang::back(|res: &mut Response| {
            res.headers.set()
                .Server("ohkami");
        })
    }
}

struct LogRequest;
impl IntoFang for LogRequest {
    fn into_fang(self) -> Fang {
        Fang::front(|req: &Request| {
            println!("{req:?}");
        })
    }
}

#[tokio::main]
async fn main() {
    Ohkami::with((AppendHeaders, LogRequest), (
        "/".GET(|| async {"Hello!"})
    )).howl(":8080").await
}

```
`Fang` schema :

#### To make a *front fang* :
- `Fn(&/&mut Request)`
- `Fn(&/&mut Request) -> Result<(), Response>`

#### To make a *back fang* :
- `Fn(&/&mut Response)`
- `Fn(&/&mut Response) -> Result<(), Response>`
- `Fn(&/&mut Response, &Request)`
- `Fn(&/&mut Response, &Request) -> Result<(), Response>`

<br>

### pack of Ohkamis
```rust,no_run
use ohkami::prelude::*;
use ohkami::typed::{Created, NoContent, ResponseBody};

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
    )).howl(5000).await
}
```

<br>

### testing
```rust
use ohkami::prelude::*;
use ohkami::testing::*; // <--

fn hello_ohkami() -> Ohkami {
    Ohkami::new((
        "/hello".GET(|| async move {
            OK("Hello, world!")
        }),
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

## MSRV (Minimum Supported Rust Version)
Latest stable.

<br>

## License
`ohkami` is licensed under MIT LICENSE ([LICENSE-MIT](https://github.com/kana-rus/ohkami/blob/main/LICENSE-MIT) or [https://opensource.org/licenses/MIT](https://opensource.org/licenses/MIT)).
