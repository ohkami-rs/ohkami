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

## 0.7 → 0.8
Reorganized middleware system and added **after**-handling middleware：

```rust
fn main() -> Result<()> {
    let middleware = Middleware::new()
        .beforeGET("/", async |c| {
            tracing::info!("Helllo, middleware!");
            c
        })
        .afterANY("/api/*", async |res| {
            res.add_header(
                Header::AccessControlAllowOrigin,
                "mydomain:8000"
            );
            res
        });
    
    // ...
}
```

- Before-handling middleware takes `Context` and returns `Context`
- After-handling middlware takes `Response` and returns `Response`
- Middleware routes can use wildcard ( `*` ). In current ohkami, wildcard **doesn't** match empty string. This design may change in future version.

<br/>

## 0.8.3
Added `Context::store` (experimentally)：

```rust
async fn handler(c: Context, id: u64) -> Result<Response> {
    let cache = c.store().await;

    let object = match cache.get(&id.to_string()) {
        Some(name) => Object::new(name),
        None => sqlx::query_as::<_, Object>(
            "SELECT id, name FROM table WHERE id = $1"
        ).bind(id)
            .fetch_one(ctx.pool())
            .await?
    };

    c.OK(object)
}
```

<br/>

## Quick start
1. Add dependencies:

```toml
[dependencies]
ohkami = "0.8.3"
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

3. If you're interested in ohkami, learn more by [examples](https://github.com/kana-rus/ohkami/tree/main/ohkami/examples) and [documentation](https://docs.rs/ohkami/latest/ohkami/) !

<br/>

## signature of handler
```rust
async fn ( Context?, {path param 1}?, {path param 2}?, {impl JSON}? ) -> Result<Response>

// `?` means "this is optional".
```
- path param：`String | usize | u64 | usize | i64 | i32`
- Current ohkami doesn't handle more than 2 path parameters. This design may change in future version.

## Snippets
### handle query params
```rust
// c: Context

let name: &str = c.req.query("name")?;

let count: usize = c.req.query("count")?;
```
### handle path params
```rust
use std::{thread::sleep, time::Duration};

fn main() -> Result<()> {
    Ohkami::default()
        .GET("/sleepy/:time/:name", sleepy_hello)
        .howl("localhost:8080")
}

async fn sleepy_hello(time: u64, name: String) -> Result<Response> {
    (time < 30)
        ._else(|| Response::BadRequest(
            "sleeping time (sec) must be less than 30."
        ))?;
        
    sleep(Duration::from_secs(time));
    Response::OK(format!("Hello {name}, I'm so sleepy..."))
}
```
### handle request body
Add `serde = { version = "1.0", features = ["derive"] }` in your dependencies ( `JSON` requires it internally )
```rust
#[derive(JSON)]
struct User {
    id:   i64,
    name: String,
}

async fn reflect(user: User) -> Result<Response> {
    Response::OK(user)
}

async fn reflect_name(user: User) -> Result<Response> {
    let name = user.name;
    Response::OK(name)
}
```
### group handlers (like axum)
```rust
use ohkami::{
    prelude::*,
    group::{GET, POST} // import this
};

#[derive(JSON)]
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

async fn reflect(payload: User) -> Result<Response> {
    Response::OK(payload)
}
```
### get request headers
```rust
let host = c.req.header(Header::Host)?;
```
```rust
async fn reflect_xcustom_header_value(c: Context) -> Result<Response> {
    let custom_header_value = c.req.header("X-Custom")?;
    c.OK(format!("`X-Custom`'s value is {custom_header_value}"))
}
```
### add response headers
```rust
c.add_header(Header::AccessControlAllowOrigin, "mydomain:8000");
// or
c.add_header("Access-Control-Allow-Origin", "mydomain:8000");

// `Response` also has the same method
```
```rust
use ohkami::prelude::*;
use ohkami::Header::AccessControlAllowOrigin;

async fn cors(mut res: Response) -> Response {
    res.add_header(AccessControlAllowOrigin, "mydomain:8000");
    res
}

fn main() -> Result<()> {
    let my_middleware = Middleware::new()
        .afterANY("/api/*", cors);

    // ...
```
### OK response with `text/plain`
```rust
Response::OK("Hello, world!")
```
```rust
c.OK("Hello, world!")
```
### OK response with `application/json`
```rust
Response::OK(json!({"ok": true}))

c.OK(json!(100))

c.OK(json!("Hello, world!"))
```
```rust
async fn reflect_id(id: u64) -> Result<Response> {
    Response::OK(json!{"id": id})
}
```

`OK` can take `JSON`-derived value directly：
```rust
#[derive(JSON)]
struct User {
    id:   u64,
    name: String,
}

// ...

let user = User { id: 1, name: String::from("John") };

Response::OK(user)
// or
c.OK(user)
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
let handler = self.handler
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
Add `tracing` and `tracing_subscriber` in your `dependencies`.
```rust
fn main() -> Result<()> {
    let config = Config {
        log_subscribe: Some(
            tracing_subscriber::fmt()
                .with_max_level(tracing::Level::TRACE)

            /* default value:

            tracing_subscriber::fmt()
                .with_mac_level(tracing::Level::DEBUG)

            */
        ),
        ..Default::default()
    };
    Ohkami::with(config)
        .GET("/", || async {Response::OK("Hello!")})
}
```
### DB config
Eneble one of following pairs of features：
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
Eneble one of following pairs of features：
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
        .beforeANY("*", |c| async {
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
        .beforeANY("*", |c| async {
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
        .GET("/", || async {
            Response::OK("Hello!")
        })
}

fn main() -> Result<()> {
    server().howl(":3000")
}
```
2. import `testing::Test` and other utils
```rust
#[cfg(test)]
mod test {
    use ohkami::{Ohkami, response::Response, testing::{Test, Request, Method}};
    use once_cell::sync::Lazy;

    static SERVER: Lazy<Ohkami> = Lazy::new(|| super::server());

    #[test]
    fn test_hello() {
        let req = Request::new(Method::GET, "/");
        SERVER.assert_to_res(&req, Response::OK("Hello!"));
    }
}
```

<br/>

## Development
ohkami is not for producntion use now.\
Please give me your feedback ! → [GetHub issue](https://github.com/kana-rus/ohkami/issues)

<br/>

## License
This project is licensed under MIT LICENSE ([LICENSE-MIT](https://github.com/kana-rus/ohkami/blob/main/LICENSE-MIT) or [https://opensource.org/licenses/MIT](https://opensource.org/licenses/MIT)).
