<div align="center">
    <h1>ohkami</h1>
</div>

### ＊This README is my working draft. So codes in "Quick start" or "Samples" don't work yet.<br/>

ohkami *- [狼] wolf in Japanese -* is **ergonomic** web framework for Rust.

## Features
- *macro free, ergonomic APIs*
- supporting *multi runtime*：`tokio`, `async-std` (Maybe more in future)

<br/>

## Quick start
1. Add to `dependencies`:

```toml
# this sample uses `tokio` runtime.
# you can choose `async-std` (and `lunatic` in future) instead.

[dependencies]
ohkami = { version = "0.9.0", features = ["tokio"] }
tokio = { version = "1.27", fetures = ["full"] }
```
(And, if needed, change your Rust toolchains into **nightly** ones)

2. Write your first code with ohkami：

```rust
use ohkami::prelude::*;

async fn hello(mut c: Context) -> Response<&'static str> {
    c.text("Hello!")
}

async fn health_check(mut c: Context) -> Response<()> {
    c.NoContent()
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    Ohkami::new([
        "/"  .GET(hello),
        "/hc".GET(health_check),
    ]).howl(":3000").await
}
```

<br/>

## handler format
```rust
async fn $handler(mut c: Context,
    (
        ($p1,): ($P1,) | ($p1, $p2): ($P1, $P2),
    )?
    ( $query_params: $QueryType, )?
    ( $request_body: $BodyType,  )?
) -> Response<$OkResponseType> {
    // ...
}
```

<br/>

## Samples

### handle path/query params
```rust
use ohkami::prelude::*;
use ohkami::QueryParams;

#[tokio::main]
async fn main() -> Result<()> {
    Ohkami::new([
        "/api/users/:id"
            .GET(get_user)
            .PATCH(update_user)
    ]).howl("localhost:5000").await
}

#[Queries]
struct GetUserQuery {
    q: Option<u64>,
}

async fn get_user(mut c: Context,
    (id,): (usize,),
    query: GetUserQuery
) -> Response<User> {

    // ...

    c.json(found_user)
}
```

### handle request body
```rust
use ohkami::{
    prelude::*,
    Payload,
    f,
};

#[Payload(JSON)]
struct CreateUserRequest {
    name:     String,
    password: String,
}

async fn create_user(mut c: Context,
    req: CreateUserRequest
) -> Response<()> {

    // ...

    c.NoContent()
}

#[Payload(Form)]
struct LoginInput {
    name:     String,
    password: String,
}

async fn post_login(mut c: Context,
    input: LoginInput
) -> Response<JWT> {

    // ...

    c.json(f!({
        "token": token
    }))
}
```
You can register validating function：
```rust
#[Payload(JSON @ Self::validate)]
struct CreateUserRequest {
    name:     String,
    password: String,
} impl CreateUserRequest {
    fn validate(&self) -> Result<()> {
        if self.name.is_empty()
        || self.password.is_empty() {
            return Err(Error::validation(
                "name or password is empty"
            ))
        }

        if &self.password == "password" {
            return Err(Error::valdation(
                "dangerous password"
            ))
        }

        Ok(())
    }
}
```
`validator` crate will be also available for this.

### use middlewares
ohkami's middlewares are called "**fang**s".
```rust
#[tokio::main]
async fn main() -> Result<(), Error> {
    let fangs = Fangs::new()
        .before("/api/*", my_fang);

    Ohkami::with(fangs, [
        "/"  .GET(root),
        "/hc".GET(health_check),
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
#[tokio::main]
async fn main() -> Result<(), Error> {
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
```rust
use ohkami::prelude::*;
use ohkami::f;

async fn handler(mut c: Context) -> Response</* ... */> {
    make_result()
        .map_err(|e| c.InternalError().text(
            f!("Got error: {e}")
        ))?;
}
```

<br/>

## License
`ohkami` is licensed under MIT LICENSE ([LICENSE-MIT](https://github.com/kana-rus/ohkami/blob/main/LICENSE-MIT) or [https://opensource.org/licenses/MIT](https://opensource.org/licenses/MIT)).
