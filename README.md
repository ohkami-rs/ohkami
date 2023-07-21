<div align="center">
    <h1>ohkami</h1>
</div>

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
tokio  = { version = "1",     fetures  = ["full"] }
```
(And check if your Rust toolchains are **nightly** ones)

2. Write your first code with ohkami：

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
    Ohkami::new()(
        "/hc"         .GET(health_check),
        "/hello/:name".GET(hello),
    ).howl(3000).await
}
```

<br/>

## Samples

### handle path/query params
```rust
use ohkami::prelude::*;
use ohkami::utils::Query;


#[tokio::main]
async fn main() {
    Ohkami::new()(
        "/api/users/:id".
            GET(get_user).
            PATCH(update_user),
    ).howl("localhost:5000").await
}

async fn get_user(c: Context,
    id: usize /* <-- path param */
) -> Response {

    // ...

    c.OK().json(found_user)
}


#[Query]
struct UpdateUserQuery {
    q: Option<u64>
}

async fn update_user(c: Context,
    id:    usize,        /* <-- path  param */
    query: GetUserQuery, /* <-- query params */
) -> Response {

    // ...

    c.NoContent()
}
```
Use tuple like `(verion, id): (u8, usize),` for multiple path params.

<br/>

### handle request body
```rust
use ohkami::{prelude::*, utils::Payload};


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
) -> Response {

    // ...

    let token = // ...

    c.OK().json(Credential { token })
}
```

<br/>

### use middlewares
ohkami's middlewares are called "**fang**s".
```rust
#[tokio::main]
async fn main() {
    Ohkami::with((append_server))(
        "/"  .GET(root),
        "/hc".GET(health_check),
        "/api/users".
            GET(get_users).
            POST(create_user),
    ).howl(":8080").await
}

fn append_server(c: &mut Context) {
    c.headers
        .Server("ohkami");
}
```

<br/>

### pack of Ohkamis
```rust
#[tokio::main]
async fn main() {
    // ...

    let users_ohkami = Ohkami::new()(
        "/".
            POST(create_user),
        "/:id".
            GET(get_user).
            PATCH(update_user).
            DELETE(delete_user),
    );

    Ohkami::new()(
        "/hc"       .GET(health_check),
        "/api/users".By(users_ohkami), // <-- nest by `By`
    ).howl(5000).await
}
```

<br/>

### error handling
Use **`.map_err(|e| c. /* error_method */ )?`** in most cases：

```rust
async fn handler1(c: Context) -> Response {
    make_result()
        .map_err(|e| c.InternalServerError())?;

    // ...
}

async fn handler2(c: Context) -> Response {
    let user = generate_dummy_user()
        .map_err(|e| c.InternalServerError()
            .text("in `generate_dummy_user`"))?;
    
    // ...
}
```

<br/>

## License
`ohkami` is licensed under MIT LICENSE ([LICENSE-MIT](https://github.com/kana-rus/ohkami/blob/main/LICENSE-MIT) or [https://opensource.org/licenses/MIT](https://opensource.org/licenses/MIT)).
