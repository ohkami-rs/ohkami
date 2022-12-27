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

# 0.4 → 0.5
- `struct JSON` -> `enum JSON<T>`
- reuqest body `JSON<T>` as handler arg

<br/>

## Quick start
1. Add dependencies:

```toml
[dependencies]
ohkami = "0.5.0"
```

2. Write your first code with ohkami:

```rust
use ohkami::prelude::*;

fn main() -> Result<()> {
    Server::setup()
        .GET("/", || async {
            Response::OK("Hello, world!")
        })
        .serve_on(":3000")
}
```

3. If you're interested in ohkami, learn more by [examples](https://github.com/kana-rus/ohkami/tree/main/examples) and [documentation](https://docs.rs/ohkami/latest/ohkami/) !

<br/>

## Snippets
### handle query params
```rust
let name = c.req.query::<&str>("name")?;
// `::<&str>` isn't needed when it's presumable
```
```rust
let count = c.req.query::<usize>("count")?;
// `::<usize>` isn't needed when it's presumable
```
### handle request body
```rust
fn main() -> Result<()> {
    Server::setup()
        .GET("/api/users", reflect)
        .GET("/api/users/name", reflect_name)
        .serve_on(":3000")
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
    Server::setup()
        .GET("/sleepy/:time/:name", sleepy_hello)
        .serve_on("localhost:8080")
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
Response::OK(JSON("Hello, world!"))
// or
c.OK(JSON("Hello, world!"))
```
```rust
Response::OK(json!("ok": true))
// or
c.OK(json!("ok": true))
```
```rust
Response::OK(json(user)?)
//or
c.OK(json(user)?)
// serialize Rust value into JSON
// value's type has to be `serde::Serialize`
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
make_some_error() // can't use `?`
    ._else(|e| Response::InternalServerError(e.to_string()))?;

make_some_error()
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
```rust
fn main() -> Result<()> {
    let config = Config {
        log_subscribe: Some(
            tracing_subscriber::fmt()
                .with_max_level(tracing::Level::TRACE)
        ),
        ..Default::default()
    };
    Server::setup_with(config)
        .GET("/", || async {Response::OK("Hello!")})
}
```
### DB config
```rust
let config = Config {
    db_profile: DBprofile {
        pool_options: PgPoolOptions::new().max_connections(20),
        url:          DB_URL.as_str(),
    },
    ..Default::default()
};
```
### use sqlx
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

    Server::setup_with(middleware)
        .GET("/", || async {
            Response::OK("Hello!")
        })
        .serve_on("localhost:3000")
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
        .ANY("/*", |c| async {
            tracing::info!("Hello, middleware!");
            c
        });

    let thirdparty_middleware = some_external_crate::x;

    Server::setup_with(config.and(middleware).and(x))
        .GET("/", || async {
            Response::OK("Hello!")
        })
        .serve_on("localhost:3000")
}
```
### test
1. split setup process from `main` function:
```rust
fn server() -> Server {
    Server::setup()
        .GET("/", || async {Response::OK("Hello!")})
}

fn main() -> Result<()> {
    server().serve_on(":3000")
}
```
2. import `test::Test` and others, and write tests using `assert_to_res` , `assert_not_to_res`:
```rust
#[cfg(test)]
mod test {
    use ohkami::{server::Server, response::Response, test::{Test, Request, Method}};
    use once_cell::sync::Lazy;

    static SERVER: Lazy<Server> = Lazy::new(|| super::server());

    #[test]
    fn test_hello() {
        let request = Request::new(Method::GET, "/");
        (*SERVER).assert_to_res(&request, Response::OK("Hello!"));
        (*SERVER).assert_not_to_res(&request, Response::BadRequest(None));
    }
}
```

<br/>

## Development
ohkami is on early stage now and not for producntion use.\
Please give me your feedback! → [GetHub issue](https://github.com/kana-rus/ohkami/issues)

<br/>

## License
This project is under MIT LICENSE ([LICENSE-MIT](https://github.com/kana-rus/ohkami/blob/main/LICENSE-MIT) or [https://opensource.org/licenses/MIT](https://opensource.org/licenses/MIT)).
