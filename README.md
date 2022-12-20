<div align="center">
    <h1>ohkami</h1>
</div>

ohkami *- [狼] means wolf in Japanese -* is **simple** and **non macro-based** web framework for Rust.

<br/>

## Features
- *simple*: Less things to learn / Less code to write / Less time to hesitate.
- *non macro-based*: No need for using macros.
- async handlers
- easy error handling

<br/>

## Quick start
1. Add dependencies:

```toml
[dependencies]
ohkami = "0.2"
```

2. Write your first code with ohkami:

```rust
use ohkami::prelude::*;

fn main() -> Result<()> {
    Server::setup()
        .GET("/", |_| async {Response::OK("Hello, world!")})
        .serve_on(":3000")
}
```

3. If you're interested in ohkami, learn more by [examples](https://github.com/kana-rus/ohkami/tree/main/examples) and documentations(**WIP**)!

<br/>

## Snippets
### get query param
```rust
let name: Result<&str> = ctx.query::<&str>("name");
```
```rust
let count: Result<usize> = ctx,query::<usize>("count");
```
### deserialize request body
```rust
let body: Result<D> = ctx.body::<D>();
```
### return OK response with `text/plain`
```rust
Response::OK("Hello, world!")
```
### return OK response with `application/json`
```rust
Response::OK(JSON("Hello, world!"))
```
```rust
Response::OK(json!("ok": true))
```
```rust
Response::OK(json(user)?) // serialize Rust value into JSON
```
### handle errors
```rust
let user = ctx.body::<User>()?;

// or, you can add an error context message:
let user = ctx.body::<User>()
    ._else(|e| e.error_context("failed to get user data"))?;

// or discard original error:
let user = ctx.body::<User>()
    ._else(|_| Response::InternalServerError("can't get user"))?;
    // or
    ._else(|_| Response::InternalServerError(None))?;
```
### handle Option values
```rust
let handler = self.handler.as_ref()._else(|| Response::NotFound(None))?;
```
### assert boolean condition
```rust
(count < 10)._else(|| Response::BadRequest("`count` must be less than 10" /* or `None` */))
```
### log config
```rust
fn main() -> Result<()> {
    let config = Config {
        log_subscribe:
            Some(tracing_subscriber::fmt()
                .with_max_level(tracing::Level::TRACE)
            ),
        ..Default::default()
    };
    Server::setup_with(config)
        .GET("/", |_| async {Response::OK("Hello!")})
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
    .fetch_one(ctx.pool())
    .await?; // `Response` implements `From<sqlx::Error>`
```
### handle path params
```rust
fn main() -> Result<()> {
    Server::setup()
        .GET("/", |_| async {Response::OK("Welcome to dinosaur API!")})
        .GET("/api/:dinosaur", get_one_by_name)
        .serve_on(":8000")
}
async fn get_one_by_name(_: Context, name: String) -> Result<Response> {
    let index = DATA
        .binary_search_by_key(&name.to_ascii_lowercase().as_str(), |data| &data.name)
        ._else(|_| Response::BadRequest("No dinosaurs found"))?;
    Response::OK(json(&DATA[index])?)
}
```
```rust
fn main() -> Result<()> {
    let config = Config {
        db_profile: DBprofile {
            pool_options: PgPoolOptions::new().max_connections(20),
            url:          DB_URL.as_str(),
        },
        ..Default::default()
    };
    Server::setup_with(config)
        .GET("/services/:region/api/users/:id")
        .serve_on(":8080")
}
async fn get_user_userid(ctx: Context, region: String, id: i64) -> Result<Response> {
    // ...
}
```
### test responses
1. split server-setup and running:
```rust
fn server() -> Server {
    Server::setup()
        .GET("/", |_| async {Response::OK("Hello!")})
}
fn main() -> Result<()> {
    server().serve_on(":3000")
}
```
2. write tests using `assert_to_res` , `assert_not_to_res`:
```rust
#[cfg(test)]
mod test {
    use ohkami::{server::Server, response::Response, test_system::{Test, Request, Method}};
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
ohkami is on early stage now and not for producntion use.

<br/>

## License
This project is under MIT LICENSE ([LICENSE-MIT](https://github.com/kana-rus/ohkami/blob/main/LICENSE-MIT) or [https://opensource.org/licenses/MIT](https://opensource.org/licenses/MIT)).
