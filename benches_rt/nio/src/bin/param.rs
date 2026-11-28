use ohkami::prelude::*;

#[nio::main]
async fn main() {
    Ohkami::new((
        "/user/:id"
            .GET(|Path(id): Path<String>| async {id}),
    )).run("0.0.0.0:3000").await
}
