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

### handler format
```rust
async fn $handler(c: Context,
    (
        $path_param: $PathType1,
        | ($path_param,): ($PathType,),
        | ($p1, $p2): ($P1, $P2),
    )?
    ( $query_params: $QueryType, )?
    ( $request_body: $BodyType,  )?
) -> Response<$OkResponseType> {
    // ...
}
```

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
You can register validating function：
```rust
#[RequestBody(JSON @ Self::validate)]
struct CreateUserRequest {
    name: String,
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
#[main]
async fn main() -> Result<()> {
    let fangs = Fangs::new()
        .before("/api/*", my_fang);

    Ohkami::with(fangs, [
        "/"
            .GET(root),
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
    ohkami::config()
        .log_subscribe(
            tracing_subscriber::fmt()
                .with_max_level(tracing::Level::TRACE)
        );

    // ...
}
```

### use DB
`main.rs`
```rust
use ohkami::prelude::*;
use crate::handler::{
    users::*,
};

#[main]
async fn main() -> Result<()> {
    qujila::config(std::env::var("DB_URL"))
        .max_connections(20)
        .await?;

    Ohkami::new([
        "/api/users"
            .POST(create_user),
        "/api/users/:id"
            .PATCH(update_user),
    ]).howl(":3000").await
}
```

`handler/users.rs`
```rust
use ohkami::prelude::*;
use ohkami::RequestBody;

use crate::schema::User;
use qujila::query::{Count, Create, update};

#[RequestBody(JSON @ Self::validate)]
struct CreateUserRequest {
    name:     String,
    password: String,
} impl CreateUserRequest {
    fn validate(&self) -> Result<()> {
        if &self.password == "password" {
            Err(Error::validation("crazy password!!!"))
        } else {
            Ok(())
        }
    }
}

async fn create_user(c: Context,
    payload: CreateUserRequest
) -> Response<User> {
    let CreateUserRequest {
        name, password
    } = payload;

    if Count!(User).WHERE(|u|
        u.name.eq(name) &
        u.password.eq(hash_func(password))
    ).await? != 0 {
        return c.InternalServerError(
            "user already exists"
        )
    }

    let created_user = Create!(User {
        name,
        password: hash_func(password),
    }).await?;

    c.Created(created_user)
}

#[RequestBody(JSON @ Self::validate)]
struct UpdateUserRequest {
    name:     Option<String>,
    password: Option<String>,
} impl UpdateUserRequest {
    fn validate(&self) -> Result<()> {
        if self.password.contains("password") {
            Err(Error::validation("crazy password!!!"))
        } else {
            Ok(())
        }
    }
}

async fn update_user(c: Context, id: usize,
    payload: UpdateUserRequest
) -> Response<()> {
    update!(User)
        .WHERE(|u| u.id.eq(id))
        .SET(|u| u
            .name_optional(payload.name)
            .password_optional(
                payload.password.map(hash_func)
            )
        )
        .await?;

    c.OK(())
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
    use ohkami::{Ohkami, response::Response, testing::{Test, Request}};
    use once_cell::sync::Lazy;

    #[test]
    fn test_hello() {
        let req = Request::GET("/");
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
