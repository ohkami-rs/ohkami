The key idea is, becasue ohkami itself is completely **independent** of any speciffic runtime unlike axum, what we have to do to support Cloudflare Workers is _**maybe**_ just implementing contertion

- `worker::Request`  → `ohkami::Request`
- `ohkami::Response` → `worker::Response`

and an attribute like

```rs
#[ohkami::worker]
async fn my_worker() -> Ohkami {
    Ohkami::new((
        "/healthz"
            .GET(health_check),
        "/hello/:name"
            .GET(hello)
    ))
}
```

that expands like

```rs
async fn my_worker() -> Ohkami {
    Ohkami::new((
        "/healthz"
            .GET(health_check),
        "/hello/:name"
            .GET(hello)
    ))
}

#[::ohkami::__internal__::worker::event(fetch)]
async fn main(
    req: ::worker::Request,
    env: ::worker::Env,
    ctx: ::worker::Context,
) -> Result<Response> {
    let mut ohkami_req = Request::init();
    let mut ohkami_req = unsafe {Pin::new_unchecked(&mut ohkami_req)};
    ohkami_req.take_over(req, env, ctx);
    
    /*
        We'd like to skip radixizing and directly handle request by TrieRouter,
        since in most cases radixizing is unacceptable overhead
        *in Edge*, compared to the RadixRouter's some advantage in performance.
    */
    let ohkami_res = my_ohkami().into_router()
        .handle(ohkami_req).await;

    Ok(ohkami_res.into())
}
```