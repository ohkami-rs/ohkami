use ohkami::prelude::*;

#[inline(always)]
async fn echo_id(id: String) -> String {
    id//.into()
}

#[tokio::main]
async fn main() {
    Ohkami::new((
        "/user/:id"
            .GET(echo_id),
            //.GET(|id: String| async {id}),
    )).howl("0.0.0.0:3000").await
}
