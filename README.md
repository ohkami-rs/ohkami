<div align="center">
    <h1>ohkami</h1>
</div>

### ＊This README is my working draft. So codes in "Quick start" or "Samples" don't work yet.<br/>

ohkami *- [狼] wolf in Japanese -* is **declarative** web framework for *nightly* Rust.

## Features
- *macro free, declarative APIs*
- supporting *multi runtime*：`tokio`, `async-std` (and more in future)

<br/>

## Quick start
1. Add to `dependencies`:

```toml
# this sample uses `tokio` runtime.
# you can choose `async-std` instead.

[dependencies]
ohkami = { version = "0.9.0", features = ["rt_tokio"] }
tokio = { version = "1.27", fetures = ["full"] }
```
(And check if your Rust toolchains are **nightly** ones)

2. Write your first code with ohkami：

```rust
use ohkami::prelude::*;

async fn health_check(c: Context) -> Response {
    c.NoContent()
}

async fn hello(c: Context, name: String) -> Response<String> {
    c.text(format!("Hello, {name}!"))
}

#[tokio::main]
async fn main() {
    Ohkami::new()(
        "/hc"   .GET(health_check),
        "/:name".GET(hello),
    ).howl(3000).await
}
```

## Samples

### handle path/query params
```rust
use ohkami::prelude::*;
use ohkami::Query;

#[tokio::main]
async fn main() {
    Ohkami::new()(
        "/api/users/:id"
            .GET(get_user)
            .PATCH(update_user)
    ).howl("localhost:5000").await
}

#[Query]
struct GetUserQuery {
    q: Option<u64>
}

async fn get_user(c: Context,
    (id,): (usize,),
    query: GetUserQuery,
) -> Response<User> {

    // ...

    c.json(found_user)
}
```

### handle request body
```rust
use ohkami::{prelude::*, Payload};

#[Payload(JSON)]
struct CreateUserRequest {
    name:     String,
    password: String,
}

async fn create_user(c: Context,
    req: CreateUserRequest
) -> Response {

    // ...

    c.NoContent()
}

#[Payload(URLEncoded)]
struct LoginInput {
    name:     String,
    password: String,
}

#[derive(serde::Serialize)]
struct Credential {
    token: String,
}

async fn post_login(c: Context,
    input: LoginInput
) -> Response<JSON> {

    // ...

    let token = // ...

    c.JSON(Credential { token })
}
```

### use middlewares
ohkami's middlewares are called "**fang**s".
```rust
#[tokio::main]
async fn main() -> Result<(), Error> {
    Ohkami::with(append_server)(
        "/"  .GET(root),
        "/hc".GET(health_check),
        "/api/users"
            .GET(get_users)
            .POST(create_user),
    ).howl(":8080").await
}

async fn append_server(c: &mut Context) {
    c.headers
        .Server("ohkami");
}
```

### pack of Ohkamis
```rust
#[tokio::main]
async fn main() -> Result<(), Error> {
    // ...

    let users_ohkami = Ohkami::new()(
        "/"
            .POST(create_user),
        "/:id"
            .GET(get_user)
            .PATCH(update_user)
            .DELETE(delete_user),
    );

    Ohkami::new()(
        "/hc"       .GET(health_check),
        "/api/users".by(users_ohkami),
    ).howl(":5000").await
}
```

### error handling
Use **`.map_err(|e| c. /* error_method */ )?`**：

```rust
async fn handler(c: Context) -> Response</* ... */> {
    make_result()
        .map_err(|e| c.InternalError())?;
}
```
You can add error message :

```rust
async fn handler(c: Context) -> Response</* ... */> {
    make_result()
        .map_err(|e| c.InternalError()
            .Text(format!("Cause: {e}"))  // <--
        )?;
}
```

<br/>

## License
`ohkami` is licensed under MIT LICENSE ([LICENSE-MIT](https://github.com/kana-rus/ohkami/blob/main/LICENSE-MIT) or [https://opensource.org/licenses/MIT](https://opensource.org/licenses/MIT)).
