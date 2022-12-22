# Middleware insertion system

## sample code

```rust
use ohkami::prelude::*;

fn main() -> Result<()> {
    let config = Config {
        // ...
    };

    let middleware = Middleware::init()
        .route("*", || async {
            tracing::info!("Hello, middleware!")
        });

    Server::setup_with(config.and(middleware))
        .GET("/", || async {
            Response::OK("Hello!")
        })
        .serve_on(":3000")
}
```

this is the same as

```rust
use ohkami::prelude::*;

fn main() -> Result<()> {
    let config = Config {
        // ...
    };
    
    let middleware = Middleware::init()
        .route("*", || async {
            tracing::info!("Hello, middleware!")
        });

    Server::setup_with(middleware.and(config))
        .GET("/", || async {
            Response::OK("Hello!")
        })
        .serve_on(":3000")
}
```

or 

```rust
use ohkami::prelude::*;

fn main() -> Result<()> {
    let server_conf = Config {
        // ...
    }.and(Middleware::init()
        .route("*", || async {
            tracing::info!("Hello, middleware!")
        })
    );

    Server::setup_with(server_conf)
        .GET("/", || async {
            Response::OK("Hello!")
        })
        .serve_on(":3000")
}
```

<br/>

## plan
1. Add struct `ServerSetting`：
```rust
struct ServerSetting {
    config:     Config,
    middleware: Middleware,
}
```
2. Change the signature of `setup_with` to take an argument implementing `IntoServerSetting`：
```rust
trait IntoServerSetting {
    fn into(self) -> ServerSetting;
    fn and<ISS: IntoServerSetting>(self, another: ISS) -> ServerSetting;
}
```
3. Impl `IntoServerSetting` to
- `ServerSetting`
- `Config`
- `Middleware`

4. Impl a system to execute `MiddlewareFunc`s based on request paths. This will need to edit at least
- handler.rs