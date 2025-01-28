```toml:Cargo.toml
[dependencies]
ohkami = { version = "0.22", features = ["rt_lambda"] }
lambda_runtime = "0.13"
```

```rust:main.rs
use ohkami::prelude::*;

#[ohkami::lambda]
async fn lambda() -> Result<Ohkami, impl std::error::Error + 'static> {
    Ohkami::new((
        "/".GET(|| async {"Hello, lambda!"}),
    ))
}
```

↓

```rust
fn main() -> Result<(), ::std::boxed::Box<dyn ::std::error::Error + Send + Sync>> {
    async fn lambda() -> Result<Ohkami, impl std::error::Error + 'static> {
        Ohkami::new((
            "/".GET(|| async {"Hello, lambda!"}),
        ))
    }

    /// APIGateway{Request, Response} :
    /// 
    /// [note] only handle version 2
    /// 
    /// https://docs.aws.amazon.com/ja_jp/lambda/latest/dg/urls-invocation.html
    /// https://docs.aws.amazon.com/apigateway/latest/developerguide/http-api-develop-integrations-lambda.html#http-api-develop-integrations-lambda.proxy-format
    async fn __lambda__(
        req: ::lambda_runtime::LambdaEvent<::ohkami::__internal__::APIGatewayRequest>
    ) -> Result<
        ::lambda_runtime::FunctionResponse<::ohkami::__internal__::APIGatewayResponse>,
        ::std::boxed::Box<dyn ::std::error::Error + Send + Sync>
    > {
        let o: ::ohkami::Ohkami = lambda().await?;

        let mut ohkami_req = crate::Request::init();
        let mut ohkami_req = unsafe {std::pin::Pin::new_unchecked(&mut ohkami_req)};
        ohkami_req.as_mut().take_over(req).await?;

        let (router, _) = o.into_router().finalize();
        let mut ohkami_res = router.handle(&mut ohkami_req).await;
        ohkami_res.complete();

        Ok(ohkami_res.into())
    }

    ::ohkami::__internal__::tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(::lambda_runtime::run(__lambda__))
}
```
