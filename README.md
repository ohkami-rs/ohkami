<div align="center">
    <h1>ohkami</h1>
</div>

### ＊This README is my working draft. So codes in "Quick start" or "Snippets" don't work yet.<br/>

ohkami *- [狼] means wolf in Japanese -* is **simple** and **macro free** web framework for **nightly** Rust.

<br/>

## Features
- *simple*: Less things to learn / Less code to write / Less time to hesitate.
- *macro free*: No need for using macros.
- *nightly only*
- async handlers
- easy error handling

<br/>

## Quick start
1. Add to `dependencies`:

```toml
[dependencies]
ohkami = "0.9.0"
```
(And, if needed, change your Rust toolchains into nightly ones)

2. Write your first code with ohkami：

```rust
use ohkami::prelude::*;

async fn hello(c: Context) -> Response<&'static str> {
    c.OK("Hello!")
}

async fn health_check(c: Context) -> Response<()> {
    c.NoContent()
}

#[main]
async fn main() -> Result<(), Error> {
    Ohkami::new([
        "/"      .GET(hello),
        "/api/hc".GET(health_check),
    ]).howl(":3000").await
}
```

<br/>

## Snippets
### handle path/query params
```rust
use ohkami::prelude::*;
use ohkami::request::QueryParams;

#[main]
async fn main() -> Result<()> {
    Ohkami::new([
        "/api/users/:id"
            .GET(get_user)
            .POST(create_user)
    ]).howl("localhost:5000").await
}

#[QueryParams]
struct GetUserQuery {
    q: u64,
}

async fn get_user(c: Context, id: usize,
    query: GetUserQuery
) -> Response<User> {

    // ...
}
```
### handle request body
```rust
use ohkami::prelude::*;
use ohkami::RequestBody;

#[RequestBody(JSON)]
struct CreateUserRequest {
    name:     String,
    password: String,
}

async fn create_user(c: Context,
    req: CreateUserRequest
) -> Response<()> {

    // ...

    c.OK(())
}

#[RequestBody(Form)]
struct LoginInput {
    name:     String,
    password: String,
}

async fn post_login(c: Context,
    input: LoginInput
) -> Response<JWT> {

    // ...

    c.OK(token)
}
```
### use middlewares
ohkami's middlewares are called "**fang**s".
```rust
#[main]
async fn main() -> Result<()> {
    let fangs = Fangs::new()
        .before("/api/*", my_fang);

    Ohkami::with(fangs, [
        "/"
            .GET(route),
        "/hc"
            .GET(health_check),
        "/api/users"
            .GET(get_users)
            .POST(create_user),
    ]).howl(":8080").await
}

async fn my_fang(
    c: mut Context
) -> Result<Context, Response> {
    // ...
}
```

### pack of Ohkamis
```rust
#[main]
async fn main() -> Result<()> {
    // ...

    let users_ohkami = Ohkami::with(users_fangs, [
        "/"
            .POST(create_user),
        "/:id"
            .GET(get_user)
            .PATCH(update_user)
            .DELETE(delete_user),
    ]);

    let tasks_ohkami = Ohkami::with(tasks_fangs, [
        // ...

    Ohkami::new([
        "/hc"       .GET(health_check),
        "/api/users".by(users_ohkami),
        "/api/tasks".by(tasks_ohkami),
    ]).howl(":5000").await
}
```

### error handling
bool / Option
```rust
async fn handler(c: Context, id: usize) -> Response</* ... */> {
    (id < 1000)
        ._else(|| c.BadRequest("`id` must be less than 1000."))?;

    //...
}
```

Result
```rust
async fn handler(c: Context) -> Response</* ... */> {
    make_result()
        ._else(|err| c.InternalServerError(
            err.to_string()
        ))?;
}
```
### global configuration
```rust
#[main]
async fn main() -> Result<()> {
    ohkami::config(|conf| conf
        .log_subscribe(
            tracing_subscriber::fmt()
                .with_max_level(tracing::Level::TRACE)
        )
    );

    // ...
}
```
### use DB
```rust
#[main]
async fn main() -> Result<()> {
    let pool = PoolOptions::new()
        .max_connections(20)
        .connect("db_url")
        .await?;

    ohkami::setup(|conf| conf
        .connection_pool(pool)
    );

    Ohkami::new([
        "/sample"
            .GET(sample_handler)
    ]).howl(":3000").await
}

async fn sample_handler(c: Context) -> Response</* ... */> {
    let user = sqlx::query_as::<_, User>(
        "SELECT id, name FROM users WHERE id = $1"
    ).bind(1)
        .fetch_one(c.pool())
        .await?;

    // ...
}
```
<br/>

```rust
#[main]
async fn main() -> Result<()> {
    let q = Qujila("db_url")
        .max_connections(20)
        .await?;

    ohkami::config(|conf| conf
        .connection_pool(q)
    );

    Ohkami::new([
        "/sample"
            .GET(sample_handler)
    ]).howl(":3000").await
}

async fn sample_handler(c: Context) -> Response</* ... */> {
    let user = c.qujila().First::<User>()
        .WHERE(|u| u.id.eq(1))
        .await?;

    // ...
}
```

### test
1. Split setup process from `main` function:
```rust
fn setup() -> Ohkami {
    Ohkami::new([
        "/".GET(move |c: Context| async {
            c.OK("Hello!")
        })
    ])
}

#[main]
fn main() -> Result<()> {
    setup().howl(":3000")
}
```
2. import `testing::Test` and other utils
```rust
#[cfg(test)]
mod test {
    use ohkami::{Ohkami, response::Response, testing::{Test, Request, Method}};
    use once_cell::sync::Lazy;

    #[test]
    fn test_hello() {
        let req = Request::new(Method::GET, "/");
        super::setup()
            .assert_to_res(&req, Response::OK("Hello!"));
    }
}
```

<br/>

## Development
ohkami is not for producntion use yet.\
Please give me your feedback ! → [GetHub issue](https://github.com/kana-rus/ohkami/issues)

<br/>

## License
This project is licensed under MIT LICENSE ([LICENSE-MIT](https://github.com/kana-rus/ohkami/blob/main/LICENSE-MIT) or [https://opensource.org/licenses/MIT](https://opensource.org/licenses/MIT)).
