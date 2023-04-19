<div align="center">
    <h1>ohkami</h1>
</div>

### ＊This README is my working draft. So codes in "Quick start" or "Samples" don't work yet.<br/>

ohkami *- [狼] wolf in Japanese -* is **ergonomic** web framework for **nightly** Rust.

## Features
- *nightly only*
- *macro free, ergonomic APIs*
- *multi runtime*：`tokio`, `async-std`, `lunatic (in future)`

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

async fn hello(c: Context) -> Response<&'static str> {
    c.text("Hello!")
}

async fn health_check(c: Context) -> Response<()> {
    c.NoContent()
}

#[tokio::main]
async fn main() -> Result<()> {
    Ohkami::new([
        "/"  .GET(hello),
        "/hc".GET(health_check),
    ]).howl(":3000").await
}
```

<br/>

## handler format
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

#[QueryParams]
struct GetUserQuery {
    q: Option<u64>,
}

async fn get_user(c: Context,
    (id,): (usize,),
    query: GetUserQuery
) -> Response<User> {

    // ...

    c.json(found_user)
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

    c.NoContent()
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

    c.json(f!({
        "token": token
    }))
}
```
You can register validating function：
```rust
#[RequestBody(JSON @ Self::validate)]
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
async fn main() -> Result<()> {
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
```rust
use ohkami::prelude::*;
use ohkami::CatchError; // <--

async fn handler(c: Context) -> Response</* ... */> {
    make_result()
        .catch(|err| c.InternalServerError(
            f!("Got error: {err}")
        ))?;
}
```

### use DB
Add `qujila` to dependencies：
```toml
[dependencies]
tokio = { version = "1.27", features = ["full"] }
ohkami = { version = "0.9", features = ["tokio"] }
qujila = { version = "0.1", features = ["tokio", "postgres"] }
```

`src/schema.rs`
```rust
qujila::schema! {
    User {
        id:         __ID__,
        name:       VARCHAR(20) where NOT_NULL,
        password:   VARCHAR(64) where NOT_NULL,
        created_at: __CREATED_AT__,
        updated_at: __UPDATED_AT__,
    },
    Task {
        id:          __ID__,
        user_id:     REFERENCES(User::id),
        title:       VARCHAR(20) where NOT_NULL,
        description: TEXT where NOT_NULL,
        created_at:  __CREATED_AT__,
        updated_at:  __UPDATED_AT__,
    }
}
```
```sh
$ qujila sync ＜DB URL＞
```

`src/main.rs`
```rust
mod handler;
mod schema;

use ohkami::prelude::*;
use crate::handler::{
    users::*,
    tasks::*,
};

#[tokio::main]
async fn main() -> Result<()> {
    qujila::spawn("DB_URL")
        .max_connections(1024)
        .await?;

    let users_ohkami = Ohkami::new([
        "/"
            .POST(create_user),
        "/:id"
            .GET(get_user)
            .PATCH(update_user)
            .DELETE(delete_user),
    ]);

    Ohkami::new([
        "/api/users".by(users_ohkami),
    ]).howl(":3000").await
}
```

`src/handler/users.rs`
```rust
use ohkami::prelude::*;
use ohkami::RequestBody;
use crate::schema::User;

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

    if User::exists(|u|
        u.name.eq(&name) &
        u.password.eq(hash_func(&password))
    ).await? {
        c.InternalServerError("user already exists")
    } else {
        let new_user = User::Create(|u| u
            .name(name)
            .password(hash_func(&password))
        ).await?;
        c.Created(new_user)
    }
}
async fn create_user(c: Context,
    payload: CreateUserRequest
) -> Response<User> {
    let CreateUserRequest{ name, password } = payload;

    // ===

    if q::<User>(|u|
        u.name.eq(&name) &
        u.password.eq(hash_func(&password))
    ).exists().await? {
        c.InternalServerError("user already exists")
    } else {
        let new_user = q.Create::<User>()
            .name(name)
            .password(hash_func(&password))
            .await?;
        c.Created(new_user)
    }

    // ===

    if q.users(|u|
        u.name.eq(&name) &
        q.password.eq(hash_func(&password))
    ).exists().await? {
        c.InternalServerError("user already exists")
    } else {
        let new_user = q.users.Create()
            .name(name)
            .password(hash_func(&password))
            .await?;
        c.Created(new_user)
    }

    // ===
}

async fn get_user(c: Context, id: usize) -> Response<User> {
    let user = User::Single(|u| u.id.eq(id)).await?;
    c.json(user)
}
async fn get_user(c: Context, id: usize) -> Response<User> {
    let user = q.users(|u| u.id.eq(id)).Single().await?;
    c.json(user)
}
async fn get_user(c: Context, id: usize) -> Response<User> {
    let user = users(|u| u.id.eq(id)).Single().await?;
    c.json(user)
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

async fn update_user(c: Context,
    (id,): (usize,),
    payload: UpdateUserRequest,
) -> Response<()> {
    let UpdateUserRequest {
        name, password
    } = payload;

    if User::isnot_sinle(|u| u.id.eq(id)).await? {
        c.InternalServerError("user is not single")
    } else {
        let updater = User::update(|u|
            u.id.eq(id)
        );
        if let Some(new_name) = name {
            updater.set_name(new_name)
        }
        if let Some(new_password) = password {
            updater.set_password(hash_func(new_password))
        }
        updater.await?;

        c.NoContent()
    }
}
async fn update_user(c: Context
    (id,): (usize,),
    payload: UpdateUserRequest,
) -> Response<()> {
    let target = q.users(|u| u.id.eq(id));

    if target.isnot_single() {
        c.InternalServerError("user is not single")
    } else {
        let updater = target.update();
        if let Some(new_name) = payload.name {
            updater.set_name(new_name)
        }
        if let Some(new_password) = payload.password {
            updater.set_password(hash_func(new_password))
        }
        updater.await?;

        c.NoContent()
    }
}

async fn delete_user(
    c: Context, id: usize
) -> Response<()> {
    if User::isnot_single(|u| u.id.eq(id)).await? {
        c.InternalServerError("user not single")
    } else {
        User::delete(|u| u.id.eq(id)).await?;
        c.NoContent()
    }
}
async fn delete_user(c: Context, id: usize) -> Response<()> {
    let target = q.users(|u| u.id.eq(id));
    
    if target.isnot_single().await? {
        c.InternalServerError("user not single")
    } else {
        target.delete().await?;
        c.NoContent()
    }
}
```

<br/>

## License
ohkami is licensed under MIT LICENSE ([LICENSE-MIT](https://github.com/kana-rus/ohkami/blob/main/LICENSE-MIT) or [https://opensource.org/licenses/MIT](https://opensource.org/licenses/MIT)).
