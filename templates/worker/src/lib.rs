use ohkami::prelude::*;

#[ohkami::worker]
async fn my_worker() -> Ohkami {
    Ohkami::new((
        "/".GET(|| async {"Hello, world!"}),
    ))
}
