<div align="center">
    <h1>ohkami</h1>
</div>

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
1. Add dependencies:

```toml
[dependencies]
ohkami = "0.9.0"
```
(And, if needed, change your Rust toolchains into nightly ones)

2. Write your first code with ohkami：

```rust
use ohkami::prelude::*;

#[main]
async fn main() -> Result<(), Error> {
    Ohkami::default().handle([
        "/"      .GET(hello),
        "/api/hc".GET(health_check),
    ]).howl(":3000").await
}

async fn hello(c: Context) -> Response<&'static str> {
    c.OK("Hello!")
}

async fn health_check(c: Context) -> Response<()> {
    c.NoContent()
}
```

3. If you're interested in ohkami, learn more by [examples](https://github.com/kana-rus/ohkami/tree/main/ohkami/examples) and [documentation](https://docs.rs/ohkami/latest/ohkami/) !

<br/>

## Snippets
### handle path/query params
```rust
#[main]
async fn main() -> Result<()> {
    Ohkami::default().handle([
        "/api/users/:id".GET(handler)
    ]).howl("localhost:5000").await
}

async fn handler(c: Context,
    Path((p1, p2)):  Path<(usize, String)>,
    Query([q1, q1]): Query<["q1", "q2"]>,
) -> Response</* ... */> {
    // ...
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

async fn reflect_user_name(c: Context,
    Body(user): Body<User>
) -> Response<String> {
    c.OK(user.name)
}
```
### use middlewares
ohkami's middlewares are called "**fang**s".
```rust
#[main]
async fn main() -> Result<()> {
    let fangs = Fangs::new()
        .ANY("/api/*", my_fang);

    Ohkami::with(fangs).handle([
        "/"         .GET(route),
        "/hc"       .GET(health_check),
        "/api/users".GET(get_users).POST(create_user)
    ]).howl(":8080").await
}

async fn my_fang(c: &mut Context,
    Header([content_type]): Header<["Content-Type"]>
) -> Response</* ... */> {
    // ...
}
```

`Fangs` can be combine by `.and(/* another */)`. This enables use thirdparties' fangs easily：
```rust
use external_crate::x_fangs;

#[main]
async fn main() -> Result<()> {
    let my_fangs = Fangs::new()
        .ANY("/api/*", my_fang);

    Ohkami::with(my_fangs.and(x_fangs))
        .handle([
    
    // ...
}
```

### pack of Ohkamis
```rust
#[main]
async fn main() -> Result<()> {
    // ...

    let users = Ohkami::with(users_fangs).handle([
        "/".POST(create_user),
        "/:id"
            .GET(get_user)
            .PATCH(update_user)
            .DELETE(delete_user),
    ]);

    let tasks = Ohkami::with(tasks_fangs).handle([
        // ...

    Ohkami::default().handle([
        "/hc"       .GET(health_check),
        "/api/users".by(users),
        "/api/tasks".by(tasks),
    ]).howl(":5000").await
}
```

### error response
bool / Option
```rust
async fn handler(c: Context,
    Path(id): Path<usize>
) -> Response</* ... */> {
    (id < 1000)
        ._else(|| c.BadRequest("`id` must be less than 1000."))?;

    //...
}
```

Result
```rust
async fn handler(c: Context,
    Query([q]): Query<["q"]>
) -> Response</* ... */> {
    let q: u8 = q.parse()
        ._else(|err| c.BadRequest(format!(
            "can't parse `q`: {}",
            err.to_string()
        )))?;

    // or

    let q: u8 = q.should_parse(c)?;
}
```
### global configuration
```rust
#[main]
async fn main() -> Result<()> {
    CONFIG
        .log_subscribe(
            tracing_subscriber::fmt()
                .with_max_level(tracing::Level::TRACE)
        );

    // ...
}
```
### use DB by sqlx
1. Add sqlx to your `Cargo.toml`.
2. Eneble one of：
- `sqlx-postgres`
- `sqlx-mysql`
- `deadpool-postgres`
```rust
// this sample uses `sqlx-postgres`

#[main]
async fn main() -> Result<()> {
    let pool = PoolOptions::new()
        .max_connection(20)
        .connect("db_url")
        .await?;

    CONFIG
        .connection_pool(pool);

    Ohkami::default().handle([
        "sample".GET(sample_handler)
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
### test
1. Split setup process from `main` function:
```rust
fn setup() -> Ohkami {
    Ohkami::default().handle([
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
