<div align="center">
    <h1>ohkami</h1>
</div>

ohkami *- [狼] wolf in Japanese -* is **declarative** web framework for Rust.

## Features
- *macro free, declarative APIs*
- supporting *multi runtime*：`tokio`, `async-std` (and more in future)

<div align="right">
    <img alt="build check status of ohkami" src="https://github.com/kana-rus/ohkami/actions/workflows/check.yml/badge.svg"/>
    <img alt="test status of ohkami" src="https://github.com/kana-rus/ohkami/actions/workflows/test.yml/badge.svg"/>
</div>

<br/>

## Quick start
1. Add to `dependencies` :

```toml
# this sample uses `tokio` runtime.
# you can choose `async-std` instead by feature "rt_async-std".

[dependencies]
ohkami = { version = "0.9.3", features = ["rt_tokio"] }
tokio  = { version = "1",     features = ["full"] }
```

2. Write your first code with ohkami : [eamples/quick_start](https://github.com/kana-rus/ohkami/blob/main/examples/quick_start/src/main.rs)

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
    Ohkami::new((
        "/hc".
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

## Samples

### handle path/query params
```rust
use ohkami::prelude::*;
use ohkami::utils::Query;


#[tokio::main]
async fn main() {
    Ohkami::new((
        "/api/users/:id".
            GET(get_user).
            PATCH(update_user)
    )).howl("localhost:5000").await
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
    id:    usize,           /* <-- path  param */
    query: UpdateUserQuery, /* <-- query params */
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
#[derive(serde::Deserialize)] // <-- This may not be needed in future version
struct CreateUserRequest {
    name:     String,
    password: String,
}

async fn create_user(c: Context,
    body: CreateUserRequest
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
use ohkami::prelude::*;

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

struct AppendHeaders;
impl IntoFang for AppendHeaders {
    fn bite(self) -> Fang {
        Fang::new(|c: &mut Context, req: Request| {
            c.headers
                .Server("ohkami");
            req
        })
    }
}

struct Log;
impl IntoFang for Log {
    fn bite(self) -> Fang {
        Fang::new(|res: Response| {
            println!("{res:?}");
            res
        })
    }
}
```
`Fang::new` schema :

- to make *back fang* : `Fn(Response) -> Response`
- to make *front fang* : `Fn(&mut Context, Request) -> Request`, or `_ -> Result<Request, Response>` for early returning error response

<br/>

### pack of Ohkamis
```rust
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

## License
`ohkami` is licensed under MIT LICENSE ([LICENSE-MIT](https://github.com/kana-rus/ohkami/blob/main/LICENSE-MIT) or [https://opensource.org/licenses/MIT](https://opensource.org/licenses/MIT)).
