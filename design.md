## HTTP

```toml:Cargo.toml
[dependencies]
ohkami = { version = "0.22", features = ["rt_lambda"] }
lambda_runtime = "0.13"
```

```rust:main.rs
use ohkami::prelude::*;
use lambda_runtime::Error;

#[ohkami::lambda]
async fn main() -> Result<(), Error> {
    // Just an ordinary Ohkami 
    let o = Ohkami::new((
        "/".GET(|| async {"Hello, lambda!"}),
    ));
    
    lambda_runtime::run(o).await
}
```


## WebSocket

Ohkami doesn't handle WebSocket on `rt_lambda`, just provides some utilities.

```toml:Cargo.toml
[dependencies]
ohkami = { version = "0.22", features = [
    "rt_lambda",

    # `apigateway` is also required
    "apigateway", "ws"
] }
```

```rust:main.rs
use ohkami::{LambdaWebSocket, LambdaWebSocketMESSAGE};
use lambda_runtime::Error;

#[ohkami::lambda]
async fn main() -> Result<(), Error> {
    lambda_runtime::run(LambdaWebSocket::handle(echo)).await
}

async fn echo(
    ws: LambdaWebSocket<LamdaWebSocketMESSAGE>
) -> Result<(), Error> {
    /*
        generic ( ws: LambdaWebSocket<_ = LambdaWebScoketEvent> )
    */
    // match ws.event {
    //     | LambdaWebSocketEvent::CONNECT(_)
    //     | LambdaWebSocketEvent::DISCONNECT(_)
    //     => unreachable!("This Lambda is for MESSAGE event!"),
    // 
    //     LambdaWebSocketEvent::MESSAGE(LambdaWebSocketMESSAGE {
    //         body,
    //         ..
    //     }) => ws.send(body).await
    // }

    ws.send(ws.event.body()).await?;

    Ok(())
}
```
