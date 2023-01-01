<div align="center">
    <h1>ohkami</h1>
</div>

ohkami *- [狼] means wolf in Japanese -* is **simple** and **macro free** web framework for Rust.

<br/>

## Features
- *simple*: Less things to learn / Less code to write / Less time to hesitate.
- *macro free*: No need for using macros.
- async handlers
- easy error handling

<br/>

## 0.6.4 → 0.6.5
- fixed `json!` macro implementation
- changed testing module: `test` -> `testing`

<br/>

## Quick start
1. Add dependencies:

```toml
[dependencies]
ohkami = "0.6.5"
```

2. Write your first code with ohkami:

```rust
use ohkami::prelude::*;

fn main() -> Result<()> {
    Ohkami::default()
        .GET("/", || async {
            Response::OK("Hello, world!")
        })
        .howl(":3000")
}
```

3. If you're interested in ohkami, learn more by [examples](https://github.com/kana-rus/ohkami/tree/main/examples) and [documentation](https://docs.rs/ohkami/latest/ohkami/) !

<br/>

## Snippets
### handle query params
```rust
let name: &str = c.req.query("name")?;

let count: usize = c.req.query("count")?;
```
### handle request body
```rust
fn main() -> Result<()> {
    Ohkami::default()
        .GET("/api/users", reflect)
        .GET("/api/users/name", reflect_name)
        .howl(":3000")
}

#[derive(Serialize, Deserialize)]
struct User {
    id:   i64,
    name: String,
}

async fn reflect(user: JSON<User>) -> Result<Response> {
    Response::OK(user)
}

async fn reflect_name(user: JSON<User>) -> Result<Response> {
    let name = user.name;
    // `JSON` implements `Deref`

    Response::OK(name)
}
```
### handle path params
```rust
fn main() -> Result<()> {
    Ohkami::default()
        .GET("/sleepy/:time/:name", sleepy_hello)
        .howl("localhost:8080")
}

async fn sleepy_hello(time: u64, name: String) -> Result<Response> {
    (time < 30)
        ._else(|| Response::BadRequest("sleeping time (sec) must be less than 30."))?;
    std::thread::sleep(
        std::time::Duration::from_secs(time)
    );
    Response::OK(format!("Hello {name}, I'm extremely sleepy..."))
}
```
### grouping handlers on the same path (like axum)
```rust
use serde::{Serialize, Deserialize};
use ohkami::{
    prelude::*,
    group::{GET, POST} // import this
};

#[derive(Serialize, Deserialize)]
struct User {
    id:   usize,
    name: String,
}

fn main() -> Result<()> {
    Ohkami::default()
        .GET("/", || async {
            Response::OK("Hello!")
        })
        .route("/api",
            GET(hello_api).POST(reflect)
        )
        .howl(":3000")
}

async fn hello_api() -> Result<Response> {
    Response::OK("Hello, api!")
}

async fn reflect(payload: JSON<User>) -> Result<Response> {
    Response::OK(payload)
}
```
### parse request headers
```rust
let host = c.req.header(Header::Host)?;
```
```rust
async fn reflect_header_custom(c: Context) -> Result<Response> {
    let custom_header_value = c.req.header("X-Custom")?;
    c.OK(format!("`X-Custom`'s value is {custom_header_value}"))
}
```
### add response headers
```rust
c.header(Header::AccessControlAllowOrigin, "mydomain:8000");
// or
c.header("Access-Control-Allow-Origin", "mydomain:8000");
```
```rust
use ohkami::prelude::*;
use Header::{AccessControlAllowOrigin};

async fn cors(c: Context) -> Context {
    c.header(AccessControlAllowOrigin, "mydomain:8000");
    c
}

fn main() -> Result<()> {
    let middleware = Middleware::new()
        .ANY("/api/*", cors);

    // ...
```
### return OK response with `text/plain`
```rust
Response::OK("Hello, world!")
// without Context
```
```rust
c.OK("Hello, world!")
// with Context
```
### return OK response with `application/json`
```rust
Response::OK(json!{"ok": true})
// or
c.OK(json!{"ok": true})
```
```rust
Response::OK(json("Hello!"))
Response::OK(json(user))
//or
c.OK(json("Hello!"))
c.OK(json(user))
// `json()` serializes Rust value into JSON
// value has to implemant `serde::Serialize`
```
### handle errors
```rust
make_ohkami_result()?;

// or, you can add an error context message:
make_ohkami_result()
    ._else(|e| e.error_context("failed to get user data"))?;

// or discard original error:
make_ohkami_result()
    ._else(|_| Response::InternalServerError("can't get user"))?;
    // or
    ._else(|_| Response::InternalServerError(None))?;
```
```rust
make_some_result(/* can't use `?` */)
    ._else(|e| Response::InternalServerError(e.to_string()))?;

make_some_result()
    ._else(|_| Response::InternalServerError(None))?;
```
### handle Option values
```rust
let handler = self.handler.as_ref()
    ._else(|| Response::NotFound("handler not found"))?;
    // or
    ._else(|| Response::NotFound(None))?;
```
### assert boolean conditions
```rust
(count < 10)
    ._else(|| Response::BadRequest("`count` must be less than 10"))?;
    // or
    ._else(|| Response::BadRequest(None))?;
```
### log config
add `tracing` and `tracing_subscriber` to your `Cargo.toml`.
```rust
fn main() -> Result<()> {
    let config = Config {
        log_subscribe: Some(
            tracing_subscriber::fmt()
                .with_max_level(tracing::Level::TRACE)
        ),
        ..Default::default()
    };
    Ohkami::with(config)
        .GET("/", || async {Response::OK("Hello!")})
}
```
### DB config
eneble one of following pairs of features：
- `sqlx` and `postgres`
- `sqlx` and `mysql`
```rust
let config = Config {
    db_profile: DBprofile {
        options: PgPoolOptions::new().max_connections(20),
        url:     DB_URL.as_str(),
    },
    ..Default::default()
};
```
### use sqlx
eneble one of following pairs of features：
- `sqlx` and `postgres`
- `sqlx` and `mysql`
```rust
let user = sqlx::query_as::<_, User>(
    "SELECT id, name FROM users WHERE id = $1"
).bind(1)
    .fetch_one(c.pool())
    .await?; // `Response` implements `From<sqlx::Error>`
```
### use middlewares
```rust
fn main() -> Result<()> {
    let middleware = Middleware::new()
        .ANY("*", |c| async {
            tracing::info!("Hello, middleware!");
            c
        });

    Ohkami::with(middleware)
        .GET("/", || async {
            Response::OK("Hello!")
        })
        .howl("localhost:3000")
}
```
```rust
fn main() -> Result<()> {
    let config = Config {
        log_subscribe: Some(
            tracing_subscriber::fmt()
                .with_max_level(tracing::Level::TRACE)
        ),
        ..Default::default()
    };

    let middleware = Middleware::new()
        .ANY("*", |c| async {
            tracing::info!("Hello, middleware!");
            c
        });

    let thirdparty_middleware = some_external_crate::x;

    Ohkami::with(config.and(middleware).and(x))
        .GET("/", || async {
            Response::OK("Hello!")
        })
        .howl("localhost:3000")
}
```
### test
1. split setup process from `main` function:
```rust
fn server() -> Ohkami {
    Ohkami::default()
        .GET("/", || async {Response::OK("Hello!")})
}

fn main() -> Result<()> {
    server().howl(":3000")
}
```
2. import `test::Test` and other utils
```rust
#[cfg(test)]
mod test {
    use ohkami::{Ohkami, response::Response, testing::{Test, Request, Method}};
    use once_cell::sync::Lazy;

    static SERVER: Lazy<Ohkami> = Lazy::new(|| super::server());

    #[test]
    fn test_hello() {
        let request = Request::new(Method::GET, "/");
        SERVER.assert_to_res(&request, Response::OK("Hello!"));
        SERVER.assert_not_to_res(&request, Response::BadRequest(None));
    }
}
```

<br/>

## Development
ohkami is on early stage now and not for producntion use.\
Please give me your feedback ! → [GetHub issue](https://github.com/kana-rus/ohkami/issues)

<br/>

## License
This project is licensed under MIT LICENSE ([LICENSE-MIT](https://github.com/kana-rus/ohkami/blob/main/LICENSE-MIT) or [https://opensource.org/licenses/MIT](https://opensource.org/licenses/MIT)).
